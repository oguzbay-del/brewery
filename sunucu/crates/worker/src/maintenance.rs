//! Scheduled maintenance jobs (T2.39). A tick runs every hour; `maintenance_runs` makes each
//! job run once per interval, even with several worker replicas or restarts.

use std::{collections::BTreeSet, sync::Arc, time::Duration};

use chrono::Utc;
use db::maintenance::{RunStatus, claim_job, finish_job};
use sqlx::PgPool;
use storage::{ObjectLockMode, ObjectStore, S3Config, S3Store};
use tokio::sync::watch;

use crate::{Result, apple_revoke::AppleRevoker, audit_anchor, retention, upload_cleanup};

/// Job name: daily audit digest.
pub const JOB_DIGEST: &str = "audit-digest-daily";
/// Job name: weekly check of the chains against the digests.
pub const JOB_VERIFY: &str = "audit-digest-verify-weekly";
/// Job name: retention of external events.
pub const JOB_RETENTION: &str = "external-events-retention";
/// Job name: permanent delete of records soft-deleted more than 30 days ago (T2.37).
pub const JOB_SOFT_DELETES: &str = "soft-delete-purge";
/// Job name: close resolved support requests after 14 days (T2.43).
pub const JOB_SUPPORT_CLOSE: &str = "support-close-resolved";
/// Job name: delete object bytes of deleted and incomplete uploads (T2.44).
pub const JOB_UPLOAD_CLEANUP: &str = "upload-cleanup";
/// Job name: account deletion after its grace period (T2.31c).
pub const JOB_ACCOUNT_DELETIONS: &str = "account-deletion-purge";

const DAY: Duration = Duration::from_hours(23);
const WEEK: Duration = Duration::from_hours(7 * 24 - 1);
const TICK: Duration = Duration::from_hours(1);

/// Builds the store for the digests from `S3_*`. Returns `None` when `S3_ENDPOINT` is not set.
///
/// - `S3_DIGEST_BUCKET` names a separate bucket for the digests (use one with object lock).
/// - `S3_OBJECT_LOCK_MODE` is `governance` or `compliance`. Without it, digests are plain
///   objects.
///
/// TODO: Set `S3_OBJECT_LOCK_MODE` and a lock-enabled digest bucket in the cluster (T2.32).
#[must_use]
pub fn digest_store_from_env() -> Option<Arc<dyn ObjectStore>> {
    let mut config = match S3Config::from_env() {
        Ok(Some(c)) => c,
        Ok(None) => return None,
        Err(e) => {
            tracing::error!(error = %e, "object store settings are invalid; audit digests are off");
            return None;
        }
    };
    if let Some(bucket) = std::env::var("S3_DIGEST_BUCKET")
        .ok()
        .filter(|b| !b.trim().is_empty())
    {
        bucket.trim().clone_into(&mut config.bucket);
    }
    let mut store = match S3Store::new(config) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "object store setup failed; audit digests are off");
            return None;
        }
    };
    let mode = std::env::var("S3_OBJECT_LOCK_MODE").ok();
    match mode.as_deref().map(ObjectLockMode::parse) {
        Some(Some(mode)) => store = store.with_object_lock(mode),
        Some(None) => tracing::error!("S3_OBJECT_LOCK_MODE must be governance or compliance"),
        None => tracing::warn!("S3_OBJECT_LOCK_MODE is not set; audit digests are not locked"),
    }
    Some(Arc::new(store))
}

async fn record(pool: &PgPool, job: &str, outcome: Result<(RunStatus, String)>) {
    let (status, detail) = outcome.unwrap_or_else(|e| (RunStatus::Error, e.to_string()));
    match status {
        RunStatus::Ok => tracing::info!(job, %detail, "maintenance job finished"),
        RunStatus::Findings => tracing::error!(job, %detail, "maintenance job found problems"),
        RunStatus::Error => tracing::warn!(job, %detail, "maintenance job failed; will retry"),
    }
    if let Err(e) = finish_job(pool, job, Utc::now(), status, &detail).await {
        tracing::warn!(job, error = %e, "cannot record the result of a maintenance job");
    }
}

/// Summarizes one completed verification, emitting at most one system event.
pub async fn report_findings(
    pool: &PgPool,
    report: &audit_anchor::VerifyReport,
) -> (RunStatus, String) {
    let status = if report.is_ok() {
        RunStatus::Ok
    } else {
        if let Err(error) = enqueue_findings(pool, report).await {
            tracing::error!(%error, "could not enqueue the audit digest findings event");
        }
        RunStatus::Findings
    };
    let details: Vec<_> = report
        .findings
        .iter()
        .map(|finding| finding.detail.as_str())
        .collect();
    let detail = format!(
        "chains {}, digests {}, missing days {}, findings: {}",
        report.chains_verified,
        report.digests_checked,
        report.digests_missing,
        details.join("; ")
    );
    (status, detail)
}

async fn enqueue_findings(pool: &PgPool, report: &audit_anchor::VerifyReport) -> db::Result<()> {
    let kinds: BTreeSet<_> = report
        .findings
        .iter()
        .map(|finding| finding.kind.code())
        .collect();
    let event = db::outbox::NewOutboxEvent {
        topic: "audit.digest_check_failed".into(),
        workspace_id: None,
        entity: None,
        data: Some(serde_json::json!({
            "chains_verified": report.chains_verified,
            "digests_checked": report.digests_checked,
            "digests_missing": report.digests_missing,
            "findings": report.findings.len(),
            "kinds": kinds,
        })),
    };
    let mut tx = pool.begin().await?;
    db::outbox::enqueue(&mut tx, &event).await?;
    tx.commit().await?;
    Ok(())
}

/// Runs every due job once.
pub async fn tick(pool: &PgPool, store: Option<&dyn ObjectStore>) {
    tick_with_files(pool, store, None).await;
}

/// Runs the maintenance jobs with the normal workspace object store as well as the digest store.
pub async fn tick_with_files(
    pool: &PgPool,
    digest_store: Option<&dyn ObjectStore>,
    file_store: Option<&dyn ObjectStore>,
) {
    tick_with_apple(pool, digest_store, file_store, None).await;
}

/// Like [`tick_with_files`], with the Apple revoker for the account purge job (T2.31d).
pub async fn tick_with_apple(
    pool: &PgPool,
    digest_store: Option<&dyn ObjectStore>,
    file_store: Option<&dyn ObjectStore>,
    apple: Option<&AppleRevoker>,
) {
    let now = Utc::now();
    if let Some(store) = digest_store {
        if claim(pool, JOB_DIGEST, DAY).await {
            let outcome = audit_anchor::write_daily_digest(pool, store, now)
                .await
                .map(|o| (RunStatus::Ok, format!("{o:?}")));
            record(pool, JOB_DIGEST, outcome).await;
        }
        if claim(pool, JOB_VERIFY, WEEK).await {
            let outcome = match audit_anchor::verify_against_digests(pool, store, now).await {
                Ok(report) => Ok(report_findings(pool, &report).await),
                Err(error) => Err(error),
            };
            record(pool, JOB_VERIFY, outcome).await;
        }
    }
    if claim(pool, JOB_RETENTION, DAY).await {
        let outcome = retention::run(pool, now).await.map(|r| {
            (
                RunStatus::Ok,
                format!(
                    "payloads cleared {}, events deleted {}",
                    r.payloads_cleared, r.events_deleted
                ),
            )
        });
        record(pool, JOB_RETENTION, outcome).await;
    }
    if claim(pool, JOB_ACCOUNT_DELETIONS, TICK).await {
        let outcome = crate::account_deletion::purge_due_and_revoke(pool, file_store, apple).await;
        record(pool, JOB_ACCOUNT_DELETIONS, outcome).await;
    }
    if claim(pool, JOB_SOFT_DELETES, DAY).await {
        let outcome = retention::purge_soft_deletes(pool, now)
            .await
            .map(|counts| {
                let detail = counts
                    .iter()
                    .map(|c| format!("{}: purged {}, kept {}", c.kind, c.purged, c.kept))
                    .collect::<Vec<_>>()
                    .join("; ");
                (RunStatus::Ok, detail)
            });
        record(pool, JOB_SOFT_DELETES, outcome).await;
    }
    if claim(pool, JOB_SUPPORT_CLOSE, DAY).await {
        let outcome = retention::close_resolved_support_requests(pool, now)
            .await
            .map(|closed| (RunStatus::Ok, format!("closed {closed}")));
        record(pool, JOB_SUPPORT_CLOSE, outcome).await;
    }
    if let Some(store) = file_store
        && claim(pool, JOB_UPLOAD_CLEANUP, TICK).await
    {
        let outcome = upload_cleanup::run(pool, store, now).await.map(|report| {
            if report.failed > 0 {
                (RunStatus::Error, format!("failed {}", report.failed))
            } else {
                (
                    RunStatus::Ok,
                    format!(
                        "deleted {}, incomplete {}, failed 0",
                        report.deleted, report.incomplete
                    ),
                )
            }
        });
        record(pool, JOB_UPLOAD_CLEANUP, outcome).await;
    }
}

async fn claim(pool: &PgPool, job: &str, interval: Duration) -> bool {
    match claim_job(pool, job, Utc::now(), interval).await {
        Ok(claimed) => claimed,
        Err(e) => {
            tracing::warn!(job, error = %e, "cannot claim a maintenance job");
            false
        }
    }
}

/// Runs the ticks until `stop` turns `true`.
pub async fn run(
    pool: PgPool,
    digest_store: Option<Arc<dyn ObjectStore>>,
    file_store: Option<Arc<dyn ObjectStore>>,
    mut stop: watch::Receiver<bool>,
) {
    if digest_store.is_none() {
        tracing::warn!("no object store; audit digests and their check are off");
    }
    let apple = AppleRevoker::from_env();
    loop {
        tick_with_apple(
            &pool,
            digest_store.as_deref(),
            file_store.as_deref(),
            apple.as_ref(),
        )
        .await;
        tokio::select! {
            _ = stop.wait_for(|stopped| *stopped) => return,
            () = tokio::time::sleep(TICK) => {}
        }
    }
}
