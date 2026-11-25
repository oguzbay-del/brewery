//! Worker maintenance jobs (T2.39): audit digests and their weekly check, retention of
//! external events, job claims. Needs PostgreSQL (`DATABASE_URL`). The object store is an
//! in-memory implementation of `ObjectStore`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::{collections::HashMap, sync::Mutex, time::Duration};

use chrono::{DateTime, Months, Utc};
use db::{ActorKind, AuditEvent, NewAuditEntry, append_audit, maintenance};
use sqlx::{PgPool, postgres::PgPoolOptions, types::Uuid};
use storage::{
    BoxFuture, Digest, ObjectMeta, ObjectStore, PresignedRequest, StorageError, StorageResult,
    UploadLimits,
};
use worker::{
    audit_anchor::{self, DigestOutcome},
    retention,
};

/// Body and lock date of an object.
type Stored = (Vec<u8>, Option<DateTime<Utc>>);

#[derive(Default)]
struct MemoryStore {
    objects: Mutex<HashMap<String, Stored>>,
}

impl MemoryStore {
    fn locked_until(&self, key: &str) -> Option<DateTime<Utc>> {
        self.objects.lock().unwrap().get(key).and_then(|(_, l)| *l)
    }
}

impl ObjectStore for MemoryStore {
    fn presign_put(
        &self,
        _: &str,
        _: &UploadLimits,
        _: Duration,
    ) -> StorageResult<PresignedRequest> {
        Err(StorageError::Config("not supported".into()))
    }
    fn presign_get(&self, _: &str, _: Duration) -> StorageResult<PresignedRequest> {
        Err(StorageError::Config("not supported".into()))
    }
    fn head<'a>(&'a self, key: &'a str) -> BoxFuture<'a, StorageResult<Option<ObjectMeta>>> {
        Box::pin(async move {
            Ok(self
                .objects
                .lock()
                .unwrap()
                .get(key)
                .map(|(b, _)| ObjectMeta {
                    size_bytes: b.len() as u64,
                    content_type: None,
                    sha256: None,
                }))
        })
    }
    fn digest<'a>(&'a self, _: &'a str, _: u64) -> BoxFuture<'a, StorageResult<Option<Digest>>> {
        Box::pin(async { Ok(None) })
    }
    fn put<'a>(
        &'a self,
        key: &'a str,
        _: &'a str,
        body: Vec<u8>,
    ) -> BoxFuture<'a, StorageResult<()>> {
        Box::pin(async move {
            self.objects
                .lock()
                .unwrap()
                .insert(key.to_owned(), (body, None));
            Ok(())
        })
    }
    fn get<'a>(&'a self, key: &'a str, _: u64) -> BoxFuture<'a, StorageResult<Option<Vec<u8>>>> {
        Box::pin(async move {
            Ok(self
                .objects
                .lock()
                .unwrap()
                .get(key)
                .map(|(b, _)| b.clone()))
        })
    }
    fn put_locked<'a>(
        &'a self,
        key: &'a str,
        _: &'a str,
        body: Vec<u8>,
        retain_until: DateTime<Utc>,
    ) -> BoxFuture<'a, StorageResult<()>> {
        Box::pin(async move {
            let mut objects = self.objects.lock().unwrap();
            // Like a locked object: it cannot be replaced.
            if objects.get(key).is_some_and(|(_, l)| l.is_some()) {
                return Err(StorageError::Rejected {
                    status: 403,
                    body: "locked".into(),
                });
            }
            objects.insert(key.to_owned(), (body, Some(retain_until)));
            Ok(())
        })
    }
    fn delete<'a>(&'a self, key: &'a str) -> BoxFuture<'a, StorageResult<()>> {
        Box::pin(async move {
            self.objects.lock().unwrap().remove(key);
            Ok(())
        })
    }
    fn ensure_bucket(&self) -> BoxFuture<'_, StorageResult<()>> {
        Box::pin(async { Ok(()) })
    }
}

fn id(n: u128) -> Uuid {
    Uuid::from_u128(0x0192_f1c0_7a3b_7000_8000_0000_0000_0000 + n)
}

async fn pool_as(base: &PgPool, role: &'static str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(4)
        .after_connect(move |conn, _| {
            Box::pin(async move {
                sqlx::query(if role == "app_api" {
                    "SET ROLE app_api"
                } else {
                    "SET ROLE app_worker"
                })
                .execute(conn)
                .await?;
                Ok(())
            })
        })
        .connect_with((*base.connect_options()).clone())
        .await
        .unwrap()
}

async fn create_workspace(api: &PgPool, ws: Uuid) {
    let mut tx = api.begin().await.unwrap();
    db::set_workspace_id(&mut tx, ws).await.unwrap();
    sqlx::query(
        "INSERT INTO workspaces (id, name, mode, region, country, time_zone, support_pseudonym)
         VALUES ($1, 'W', 'business', 'us', 'US', 'America/New_York', $2)",
    )
    .bind(ws)
    .bind(format!("pseudonym-{ws}"))
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

async fn append_rows(api: &PgPool, ws: Uuid, rows: i64) {
    for n in 1..=rows {
        let mut tx = api.begin().await.unwrap();
        db::set_workspace_id(&mut tx, ws).await.unwrap();
        let entry = NewAuditEntry {
            actor_kind: ActorKind::User,
            actor_id: None,
            event: AuditEvent::RoleUpdate,
            record_kind: "role".into(),
            record_id: Some(id(600)),
            previous_value: None,
            new_value: Some(serde_json::json!({ "n": n })),
            reason: None,
        };
        append_audit(&mut tx, ws, &entry).await.unwrap();
        tx.commit().await.unwrap();
    }
}

/// Runs `sql` as the superuser with the append-only triggers of `audit_log` off.
async fn tamper(pool: &PgPool, sql: &'static str, workspace: Uuid) {
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("ALTER TABLE audit_log DISABLE TRIGGER USER")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query(sql)
        .bind(workspace)
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("ALTER TABLE audit_log ENABLE TRIGGER USER")
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

/// Two workspaces with chains (3 and 2 rows) and the worker pool.
async fn setup(pool: &PgPool) -> (PgPool, Uuid, Uuid) {
    let api = pool_as(pool, "app_api").await;
    let (a, b) = (id(1), id(2));
    create_workspace(&api, a).await;
    create_workspace(&api, b).await;
    append_rows(&api, a, 3).await;
    append_rows(&api, b, 2).await;
    (pool_as(pool, "app_worker").await, a, b)
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn daily_job_writes_a_locked_digest_once(pool: PgPool) {
    let (worker, a, b) = setup(&pool).await;
    let store = MemoryStore::default();
    let now = Utc::now();

    let outcome = audit_anchor::write_daily_digest(&worker, &store, now)
        .await
        .unwrap();
    let key = audit_anchor::digest_key(now.date_naive());
    assert_eq!(
        outcome,
        DigestOutcome::Written {
            key: key.clone(),
            heads: 2
        }
    );
    assert!(key.starts_with("audit-digests/"));
    let retain = store.locked_until(&key).expect("digest is locked");
    assert!(retain > now + chrono::Duration::days(399));

    let body = store.get(&key, 1 << 20).await.unwrap().unwrap();
    let digest: audit_anchor::Digest = serde_json::from_slice(&body).unwrap();
    let heads = maintenance::list_audit_heads(&worker).await.unwrap();
    assert_eq!(digest.heads.len(), 2);
    assert!(
        digest
            .heads
            .iter()
            .any(|h| h.workspace_id == a && h.last_seq == 3)
    );
    assert!(
        digest
            .heads
            .iter()
            .any(|h| h.workspace_id == b && h.last_seq == 2)
    );
    assert_eq!(heads.len(), 2);
    assert!(digest.deletions.is_empty());

    // The same day again changes nothing.
    let again = audit_anchor::write_daily_digest(&worker, &store, now)
        .await
        .unwrap();
    assert_eq!(again, DigestOutcome::Exists { key });
}

async fn findings_events(pool: &PgPool) -> Vec<(Option<Uuid>, serde_json::Value)> {
    sqlx::query_as(
        "SELECT workspace_id, payload FROM outbox WHERE topic = 'audit.digest_check_failed'",
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

#[test]
fn finding_kinds_have_stable_event_codes() {
    use audit_anchor::FindingKind::{ChainBroken, DigestMismatch, DigestMissing, Other};
    for (kind, expected) in [
        (ChainBroken, "chain_broken"),
        (DigestMismatch, "digest_mismatch"),
        (DigestMissing, "digest_missing"),
        (Other, "other"),
    ] {
        assert_eq!(kind.code(), expected);
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn weekly_check_passes_on_an_untouched_chain(pool: PgPool) {
    let (worker, ..) = setup(&pool).await;
    let store = MemoryStore::default();
    let now = Utc::now();
    audit_anchor::write_daily_digest(&worker, &store, now)
        .await
        .unwrap();
    // The chain grows after the digest: still fine.
    let api = pool_as(&pool, "app_api").await;
    append_rows(&api, id(1), 1).await; // appends rows 1..=1 again: seq 4

    let report = audit_anchor::verify_against_digests(&worker, &store, now)
        .await
        .unwrap();
    assert!(report.is_ok(), "{report:?}");
    assert_eq!(report.chains_verified, 2);
    assert_eq!(report.digests_checked, 1);
    assert_eq!(
        report.digests_missing,
        usize::try_from(audit_anchor::VERIFY_WINDOW_DAYS).unwrap()
    );
    worker::maintenance::tick(&worker, Some(&store)).await;
    assert!(findings_events(&pool).await.is_empty());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn weekly_check_fails_when_a_chain_row_changes(pool: PgPool) {
    let (worker, a, _) = setup(&pool).await;
    let store = MemoryStore::default();
    let now = Utc::now();
    audit_anchor::write_daily_digest(&worker, &store, now)
        .await
        .unwrap();

    tamper(
        &pool,
        "UPDATE audit_log SET reason = 'x' WHERE workspace_id = $1 AND seq = 2",
        a,
    )
    .await;

    let report = audit_anchor::verify_against_digests(&worker, &store, now)
        .await
        .unwrap();
    assert!(!report.is_ok());
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.detail.contains(&a.to_string())),
        "{report:?}"
    );
    worker::maintenance::tick(&worker, Some(&store)).await;
    worker::maintenance::tick(&worker, Some(&store)).await;
    assert_eq!(
        findings_events(&pool).await,
        vec![(
            None,
            serde_json::json!({
                "schema_version": 1,
                "data": {
                    "chains_verified": 2,
                    "digests_checked": 1,
                    "digests_missing": audit_anchor::VERIFY_WINDOW_DAYS,
                    "findings": 1,
                    "kinds": ["chain_broken"],
                }
            })
        )]
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn enqueue_failure_preserves_recorded_findings(pool: PgPool) {
    let (worker, ws, _) = setup(&pool).await;
    tamper(
        &pool,
        "UPDATE audit_log SET reason = 'x' WHERE workspace_id = $1 AND seq = 2",
        ws,
    )
    .await;
    sqlx::query("ALTER TABLE outbox ADD CHECK (topic <> 'audit.digest_check_failed')")
        .execute(&pool)
        .await
        .unwrap();
    worker::maintenance::tick(&worker, Some(&MemoryStore::default())).await;
    let status: String =
        sqlx::query_scalar("SELECT last_status FROM maintenance_runs WHERE job = $1")
            .bind(worker::maintenance::JOB_VERIFY)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "findings");
    assert!(findings_events(&pool).await.is_empty());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn weekly_check_fails_when_rows_and_head_are_cut_off_together(pool: PgPool) {
    let (worker, a, _) = setup(&pool).await;
    let store = MemoryStore::default();
    let now = Utc::now();
    audit_anchor::write_daily_digest(&worker, &store, now)
        .await
        .unwrap();

    // Delete the last row and move the head back: the chain itself stays consistent.
    tamper(
        &pool,
        "DELETE FROM audit_log WHERE workspace_id = $1 AND seq = 3",
        a,
    )
    .await;
    sqlx::query(
        "UPDATE audit_heads SET last_seq = 2,
                last_hash = (SELECT hash FROM audit_log WHERE workspace_id = $1 AND seq = 2)
         WHERE workspace_id = $1",
    )
    .bind(a)
    .execute(&pool)
    .await
    .unwrap();

    let report = audit_anchor::verify_against_digests(&worker, &store, now)
        .await
        .unwrap();
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.detail.contains("is missing")),
        "{report:?}"
    );
    // The in-database chain check alone does not see this.
    assert!(
        report
            .findings
            .iter()
            .all(|f| !f.detail.contains("is broken")),
        "{report:?}"
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_purged_workspace_is_explained_by_its_deletion_record(pool: PgPool) {
    let (worker, a, _) = setup(&pool).await;
    let store = MemoryStore::default();
    let now = Utc::now();
    audit_anchor::write_daily_digest(&worker, &store, now)
        .await
        .unwrap();

    sqlx::query("UPDATE workspaces SET deleted_at = now() WHERE id = $1")
        .bind(a)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("SELECT purge_workspace($1, NULL, 'test')")
        .bind(a)
        .execute(&worker)
        .await
        .unwrap();
    let report = audit_anchor::verify_against_digests(&worker, &store, now)
        .await
        .unwrap();
    assert!(report.is_ok(), "{report:?}");
    assert_eq!(report.chains_verified, 1);

    // The next digest carries the deletion record; removing the record is then a finding.
    let next = now + chrono::Duration::days(1);
    audit_anchor::write_daily_digest(&worker, &store, next)
        .await
        .unwrap();
    let body = store
        .get(&audit_anchor::digest_key(next.date_naive()), 1 << 20)
        .await
        .unwrap()
        .unwrap();
    let digest: audit_anchor::Digest = serde_json::from_slice(&body).unwrap();
    assert_eq!(digest.deletions.len(), 1);
    assert_eq!(digest.deletions[0].former_workspace_id, a);
    assert_eq!(digest.deletions[0].audit_last_seq, 3);
}

async fn insert_event(pool: &PgPool, n: u128, received_at: DateTime<Utc>, processed: bool) {
    sqlx::query(
        "INSERT INTO external_events (id, source, delivery_id, payload, received_at, processed_at)
         VALUES ($1, 'github', $2, '{\"body\": \"customer text\"}', $3, $4)",
    )
    .bind(id(n))
    .bind(format!("d-{n}"))
    .bind(received_at)
    .bind(Option::<DateTime<Utc>>::None)
    .execute(pool)
    .await
    .unwrap();
    if processed {
        // The trigger blanks the payload when the event is processed.
        sqlx::query("UPDATE external_events SET processed_at = $2 WHERE id = $1")
            .bind(id(n))
            .bind(received_at)
            .execute(pool)
            .await
            .unwrap();
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn retention_clears_and_deletes_the_correct_rows(pool: PgPool) {
    let worker = pool_as(&pool, "app_worker").await;
    let now = Utc::now();
    let days = |d| now - chrono::Duration::days(d);
    let old = now.checked_sub_months(Months::new(14)).unwrap();
    let edge = now.checked_sub_months(Months::new(12)).unwrap();

    insert_event(&pool, 1, days(5), false).await; // fresh, unprocessed: keep
    insert_event(&pool, 2, days(31), false).await; // stale, unprocessed: clear
    insert_event(&pool, 3, days(31), true).await; // stale, processed (payload already blank)
    insert_event(&pool, 4, edge, false).await; // 12 months: cleared, row kept
    insert_event(&pool, 5, old, false).await; // 14 months: row deleted
    insert_event(&pool, 6, old, true).await; // 14 months: row deleted

    let report = retention::run(&worker, now).await.unwrap();
    assert_eq!(report.events_deleted, 2);
    assert_eq!(report.payloads_cleared, 2);

    let rows: Vec<(Uuid, serde_json::Value, bool)> = sqlx::query_as(
        "SELECT id, payload, payload_cleared_at IS NOT NULL FROM external_events ORDER BY id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let ids: Vec<Uuid> = rows.iter().map(|r| r.0).collect();
    assert_eq!(ids, vec![id(1), id(2), id(3), id(4)]);
    assert_eq!(rows[0].1["body"], "customer text");
    assert!(!rows[0].2);
    assert_eq!(rows[1].1, serde_json::json!({}));
    assert!(rows[1].2);
    assert_eq!(rows[2].1, serde_json::json!({})); // trigger blanked it when it was processed
    assert!(!rows[2].2);
    assert_eq!(rows[3].1, serde_json::json!({}));
    assert!(rows[3].2);

    // A second run touches nothing.
    let again = retention::run(&worker, now).await.unwrap();
    assert_eq!((again.events_deleted, again.payloads_cleared), (0, 0));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_job_runs_once_per_interval_and_retries_after_an_error(pool: PgPool) {
    let worker = pool_as(&pool, "app_worker").await;
    let now = Utc::now();
    let day = Duration::from_hours(23);
    assert!(
        maintenance::claim_job(&worker, "j", now, day)
            .await
            .unwrap()
    );
    assert!(
        !maintenance::claim_job(&worker, "j", now, day)
            .await
            .unwrap()
    );
    maintenance::finish_job(&worker, "j", now, maintenance::RunStatus::Findings, "x")
        .await
        .unwrap();
    assert!(
        !maintenance::claim_job(&worker, "j", now, day)
            .await
            .unwrap()
    );
    let later = now + chrono::Duration::hours(24);
    assert!(
        maintenance::claim_job(&worker, "j", later, day)
            .await
            .unwrap()
    );
    maintenance::finish_job(&worker, "j", later, maintenance::RunStatus::Error, "down")
        .await
        .unwrap();
    assert!(
        maintenance::claim_job(&worker, "j", later, day)
            .await
            .unwrap()
    );
}

async fn insert_soft_deleted(pool: &PgPool, ws: Uuid, n: u128, days_ago: i32, with_lot: bool) {
    let ing = id(1000 + n);
    sqlx::query(
        "INSERT INTO ingredients (id, workspace_id, kind, name, source, deleted_at)
         VALUES ($1, $2, 'hop', 'H', 'user', now() - make_interval(days => $3))",
    )
    .bind(ing)
    .bind(ws)
    .bind(days_ago)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO recipes (id, workspace_id, name, deleted_at)
         VALUES ($1, $2, 'R', now() - make_interval(days => $3))",
    )
    .bind(id(2000 + n))
    .bind(ws)
    .bind(days_ago)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO equipment (id, workspace_id, name, batch_volume_l, efficiency_pct, deleted_at)
         VALUES ($1, $2, 'E', 20, 70, now() - make_interval(days => $3))",
    )
    .bind(id(3000 + n))
    .bind(ws)
    .bind(days_ago)
    .execute(pool)
    .await
    .unwrap();
    if with_lot {
        sqlx::query("INSERT INTO stock_lots (id, workspace_id, ingredient_id) VALUES ($1, $2, $3)")
            .bind(id(4000 + n))
            .bind(ws)
            .bind(ing)
            .execute(pool)
            .await
            .unwrap();
    }
}

async fn count(pool: &PgPool, table: &'static str) -> i64 {
    sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT count(*) FROM {table}")))
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn expired_soft_deletes_are_purged_unless_in_use(pool: PgPool) {
    let (worker, a, b) = setup(&pool).await;
    // Workspace a: old (31 days) and recent (10 days); workspace b: old, with a stock lot.
    insert_soft_deleted(&pool, a, 1, 31, false).await;
    insert_soft_deleted(&pool, a, 2, 10, false).await;
    insert_soft_deleted(&pool, b, 3, 40, true).await;

    let totals = retention::purge_soft_deletes(&worker, Utc::now())
        .await
        .unwrap();
    let get = |k: &str| totals.iter().find(|t| t.kind == k).unwrap().clone();
    assert_eq!((get("recipe").purged, get("recipe").kept), (2, 0));
    assert_eq!((get("equipment").purged, get("equipment").kept), (2, 0));
    // The ingredient with a stock lot stays (F2.1 decision 9).
    assert_eq!((get("ingredient").purged, get("ingredient").kept), (1, 1));
    assert_eq!(count(&pool, "recipes").await, 1);
    assert_eq!(count(&pool, "equipment").await, 1);
    assert_eq!(count(&pool, "ingredients").await, 2);

    // A second run deletes nothing more.
    let again = retention::purge_soft_deletes(&worker, Utc::now())
        .await
        .unwrap();
    assert!(again.iter().all(|t| t.purged == 0));
}
