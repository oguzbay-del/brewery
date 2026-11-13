//! Audit log hash chain (T2.14): append API, verification, tamper detection, concurrency.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{Role, create_workspace, id, pool_as, set_context};
use db::{
    ActorKind, AuditEvent, ChainBreak, GENESIS_HASH, NewAuditEntry, append_audit,
    verify_all_chains, verify_chain,
};
use serde_json::json;
use sqlx::{PgPool, types::Uuid};

fn entry(event: AuditEvent, n: i64) -> NewAuditEntry {
    NewAuditEntry {
        actor_kind: ActorKind::User,
        actor_id: Some(id(500)),
        event,
        record_kind: "role".into(),
        record_id: Some(id(600)),
        previous_value: Some(json!({ "n": n - 1, "ratio": 1.5 })),
        new_value: Some(json!({ "n": n, "name": value_name() })),
        reason: Some("test".into()),
    }
}

fn value_name() -> serde_json::Value {
    db::value_digest("Brewer Name")
}

async fn append_n(app: &PgPool, ws: Uuid, n: i64) {
    for i in 1..=n {
        let mut tx = app.begin().await.unwrap();
        set_context(&mut tx, Some(ws), None).await.unwrap();
        append_audit(&mut tx, ws, &entry(AuditEvent::RoleUpdate, i))
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }
}

async fn setup(pool: &PgPool, rows: i64) -> (PgPool, Uuid) {
    let app = pool_as(pool, Role::Api).await;
    let ws = id(1);
    create_workspace(&app, ws, "A").await.unwrap();
    append_n(&app, ws, rows).await;
    (app, ws)
}

async fn verify(pool: &PgPool, ws: Uuid) -> db::ChainReport {
    let mut conn = pool.acquire().await.unwrap();
    verify_chain(&mut conn, ws).await.unwrap()
}

/// Runs `sql` as the superuser with the append-only triggers switched off.
async fn tamper(pool: &PgPool, sql: &'static str) {
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("ALTER TABLE audit_log DISABLE TRIGGER USER")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query(sql).execute(&mut *tx).await.unwrap();
    sqlx::query("ALTER TABLE audit_log ENABLE TRIGGER USER")
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn chain_is_built_and_verifies(pool: PgPool) {
    let (_app, ws) = setup(&pool, 5).await;
    let report = verify(&pool, ws).await;
    assert!(report.is_ok(), "{report:?}");
    assert_eq!(report.rows, 5);

    let rows: Vec<(i64, Vec<u8>, Vec<u8>)> =
        sqlx::query_as("SELECT seq, prev_hash, hash FROM audit_log ORDER BY seq")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        rows.iter().map(|r| r.0).collect::<Vec<_>>(),
        [1, 2, 3, 4, 5]
    );
    assert_eq!(rows[0].1, GENESIS_HASH.to_vec());
    for pair in rows.windows(2) {
        assert_eq!(pair[1].1, pair[0].2, "prev_hash links to previous hash");
    }
    let head: (i64, Vec<u8>) = sqlx::query_as("SELECT last_seq, last_hash FROM audit_heads")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(head, (5, rows[4].2.clone()));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn empty_chain_verifies_and_all_lists_workspaces(pool: PgPool) {
    let app = pool_as(&pool, Role::Api).await;
    create_workspace(&app, id(1), "A").await.unwrap();
    assert!(verify(&pool, id(1)).await.is_ok());
    append_n(&app, id(1), 2).await;
    create_workspace(&app, id(2), "B").await.unwrap();
    append_n(&app, id(2), 1).await;
    let mut conn = pool.acquire().await.unwrap();
    let reports = verify_all_chains(&mut conn).await.unwrap();
    assert_eq!(reports.len(), 2);
    assert!(reports.iter().all(db::ChainReport::is_ok));

    // A role under RLS cannot list the workspaces, and must not report empty success.
    let mut app_conn = app.acquire().await.unwrap();
    assert!(matches!(
        verify_all_chains(&mut app_conn).await,
        Err(db::Error::VerifyNeedsBypass)
    ));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn deleted_row_is_detected(pool: PgPool) {
    let (_app, ws) = setup(&pool, 5).await;
    tamper(&pool, "DELETE FROM audit_log WHERE seq = 3").await;
    let report = verify(&pool, ws).await;
    assert_eq!(
        report.broken,
        Some(ChainBreak::SeqGap {
            expected: 3,
            found: 4
        })
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn deleted_tail_is_detected(pool: PgPool) {
    let (_app, ws) = setup(&pool, 5).await;
    tamper(&pool, "DELETE FROM audit_log WHERE seq = 5").await;
    assert_eq!(
        verify(&pool, ws).await.broken,
        Some(ChainBreak::HeadMismatch {
            head_seq: 5,
            last_row_seq: 4
        })
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn changed_row_is_detected(pool: PgPool) {
    let (_app, ws) = setup(&pool, 5).await;
    tamper(
        &pool,
        "UPDATE audit_log SET new_value = '{\"n\": 99}' WHERE seq = 2",
    )
    .await;
    assert_eq!(
        verify(&pool, ws).await.broken,
        Some(ChainBreak::HashMismatch { seq: 2 })
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn changed_reason_and_actor_are_detected(pool: PgPool) {
    let (_app, ws) = setup(&pool, 3).await;
    tamper(&pool, "UPDATE audit_log SET reason = 'x' WHERE seq = 3").await;
    assert_eq!(
        verify(&pool, ws).await.broken,
        Some(ChainBreak::HashMismatch { seq: 3 })
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn inserted_row_is_detected(pool: PgPool) {
    let (_app, ws) = setup(&pool, 4).await;
    // Replace row 3 with a forged one that links to row 2 but carries other content.
    tamper(
        &pool,
        "WITH old AS (DELETE FROM audit_log WHERE seq = 3 RETURNING *)
         INSERT INTO audit_log (id, workspace_id, seq, actor_kind, action, record_kind,
                                prev_hash, hash)
         SELECT gen_random_uuid(), workspace_id, 3, 'system', 'forged', 'role', prev_hash,
                decode(repeat('ab', 32), 'hex') FROM old",
    )
    .await;
    assert_eq!(
        verify(&pool, ws).await.broken,
        Some(ChainBreak::HashMismatch { seq: 3 })
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn reordered_hash_chain_is_detected(pool: PgPool) {
    let (_app, ws) = setup(&pool, 3).await;
    tamper(
        &pool,
        "UPDATE audit_log SET prev_hash = decode(repeat('01', 32), 'hex') WHERE seq = 2",
    )
    .await;
    assert_eq!(
        verify(&pool, ws).await.broken,
        Some(ChainBreak::PrevHashMismatch { seq: 2 })
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn missing_head_is_detected(pool: PgPool) {
    let (_app, ws) = setup(&pool, 2).await;
    sqlx::query("DELETE FROM audit_heads")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        verify(&pool, ws).await.broken,
        Some(ChainBreak::MissingHead)
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn concurrent_appends_keep_the_chain_intact(pool: PgPool) {
    let app = pool_as(&pool, Role::Api).await;
    let ws = id(1);
    create_workspace(&app, ws, "A").await.unwrap();
    let mut tasks = Vec::new();
    for i in 1..=20 {
        let app = app.clone();
        tasks.push(tokio::spawn(async move {
            let mut tx = app.begin().await.unwrap();
            set_context(&mut tx, Some(ws), None).await.unwrap();
            let rec = append_audit(&mut tx, ws, &entry(AuditEvent::RoleAssign, i))
                .await
                .unwrap();
            tx.commit().await.unwrap();
            rec.seq
        }));
    }
    let mut seqs = Vec::new();
    for t in tasks {
        seqs.push(t.await.unwrap());
    }
    seqs.sort_unstable();
    assert_eq!(seqs, (1..=20).collect::<Vec<i64>>());
    let report = verify(&pool, ws).await;
    assert!(report.is_ok(), "{report:?}");
    assert_eq!(report.rows, 20);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn workspaces_have_separate_chains(pool: PgPool) {
    let app = pool_as(&pool, Role::Api).await;
    for n in [1, 2] {
        create_workspace(&app, id(n), "W").await.unwrap();
        append_n(&app, id(n), 2).await;
    }
    let first_hashes: Vec<Vec<u8>> =
        sqlx::query_scalar("SELECT prev_hash FROM audit_log WHERE seq = 1")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(first_hashes, vec![GENESIS_HASH.to_vec(); 2]);
    assert!(verify(&pool, id(1)).await.is_ok());
    assert!(verify(&pool, id(2)).await.is_ok());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn rolled_back_append_leaves_no_gap(pool: PgPool) {
    let (app, ws) = setup(&pool, 1).await;
    let mut tx = app.begin().await.unwrap();
    set_context(&mut tx, Some(ws), None).await.unwrap();
    append_audit(&mut tx, ws, &entry(AuditEvent::RoleCreate, 2))
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    append_n(&app, ws, 1).await;
    let report = verify(&pool, ws).await;
    assert!(report.is_ok(), "{report:?}");
    assert_eq!(report.rows, 2);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn invalid_entries_are_refused(pool: PgPool) {
    let (app, ws) = setup(&pool, 0).await;
    let mut tx = app.begin().await.unwrap();
    set_context(&mut tx, Some(ws), None).await.unwrap();
    let mut e = entry(AuditEvent::BatchClosedCorrection, 1);
    e.reason = None;
    assert!(matches!(
        append_audit(&mut tx, ws, &e).await,
        Err(db::Error::InvalidAuditEntry(_))
    ));
    e.reason = Some("  ".into());
    assert!(append_audit(&mut tx, ws, &e).await.is_err());
    e.reason = Some("scale was miscalibrated".into());
    e.record_kind = String::new();
    assert!(append_audit(&mut tx, ws, &e).await.is_err());
    e.record_kind = "batch".into();
    assert!(append_audit(&mut tx, ws, &e).await.is_ok());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn writing_into_another_workspace_is_refused(pool: PgPool) {
    let (app, _ws) = setup(&pool, 0).await;
    create_workspace(&app, id(2), "B").await.unwrap();
    let mut tx = app.begin().await.unwrap();
    set_context(&mut tx, Some(id(1)), None).await.unwrap();
    assert!(
        append_audit(&mut tx, id(2), &entry(AuditEvent::RoleCreate, 1))
            .await
            .is_err()
    );
}

#[test]
fn event_kinds_are_unique_and_fit_the_column() {
    let mut names: Vec<&str> = AuditEvent::ALL.iter().map(|e| e.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), AuditEvent::ALL.len());
    assert!(names.iter().all(|n| (1..=100).contains(&n.len())));
    assert!(AuditEvent::BatchClosedCorrection.requires_reason());
    assert!(!AuditEvent::RoleCreate.requires_reason());
}

#[test]
fn canonical_form_is_stable() {
    use chrono::TimeZone;
    let created = chrono::Utc.timestamp_micros(1_700_000_000_123_456).unwrap();
    let new_value = json!({ "b": 1.0, "a": [true, null] });
    let bytes = db::canonical_row(&db::CanonicalFields {
        id: id(1),
        workspace_id: id(2),
        seq: 7,
        created_at: created,
        actor_kind: "user",
        actor_id: None,
        action: "role.create",
        record_kind: "role",
        record_id: None,
        previous_value: None,
        new_value: Some(&new_value),
        reason: None,
    })
    .unwrap();
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.starts_with(r#"{"action":"role.create","actor_id":null,"actor_kind":"user","created_at":"2023-11-14T22:13:20.123456Z""#), "{text}");
    assert!(
        text.contains(r#""new_value":{"a":[true,null],"b":1}"#),
        "{text}"
    );
}
