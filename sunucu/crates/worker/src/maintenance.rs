//! Scheduled maintenance jobs (T2.39). A tick runs every hour; `maintenance_runs` makes each
//! job run once per interval, even with several worker replicas or restarts.

use std::{sync::Arc, time::Duration};

use chrono::Utc;
use db::maintenance::{RunStatus, claim_job, finish_job};
use sqlx::PgPool;
use storage::{ObjectLockMode, ObjectStore, S3Config, S3Store};
use tokio::sync::watch;

use crate::{Result, audit_anchor, retention};

/// Job name: daily audit digest.
pub const JOB_DIGEST: &str = "audit-digest-daily";
/// Job name: weekly check of the chains against the digests.
pub const JOB_VERIFY: &str = "audit-digest-verify-weekly";
/// Job name: retention of external events.
pub const JOB_RETENTION: &str = "external-events-retention";
/// Job name: permanent delete of records soft-deleted more than 30 days ago (T2.37).
pub const JOB_SOFT_DELETES: &str = "soft-delete-purge";
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
    let now = Utc::now();
    if let Some(store) = digest_store {
        if claim(pool, JOB_DIGEST, DAY).await {
            let outcome = audit_anchor::write_daily_digest(pool, store, now)
                .await
                .map(|o| (RunStatus::Ok, format!("{o:?}")));
            record(pool, JOB_DIGEST, outcome).await;
        }
        if claim(pool, JOB_VERIFY, WEEK).await {
            let outcome = audit_anchor::verify_against_digests(pool, store, now)
                .await
                .map(|r| {
                    let status = if r.is_ok() {
                        RunStatus::Ok
                    } else {
                        RunStatus::Findings
                    };
                    let detail = format!(
                        "chains {}, digests {}, missing days {}, findings: {}",
                        r.chains_verified,
                        r.digests_checked,
                        r.digests_missing,
                        r.findings.join("; ")
                    );
                    (status, detail)
                });
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
        let outcome = crate::account_deletion::purge_due(pool, file_store)
            .await
            .map(|count| {
                (
                    RunStatus::Ok,
                    format!("{count} account deletion requests completed"),
                )
            });
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
    loop {
        tick_with_files(&pool, digest_store.as_deref(), file_store.as_deref()).await;
        tokio::select! {
            _ = stop.wait_for(|stopped| *stopped) => return,
            () = tokio::time::sleep(TICK) => {}
        }
    }
}
