//! Corrections of the support panel (T2.30): only `app_worker` writes `subscriptions` and
//! `entitlements`; the worker applies a correction request and writes the audit row.
//! Needs PostgreSQL (`DATABASE_URL`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{Seed, api_pool, seed, worker_pool};
use notifier::enter_workspace;
use serde_json::json;
use sqlx::PgPool;
use support::panel;
use uuid::Uuid;

async fn staff(base: &PgPool) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO staff_users (id, google_subject, email, role) VALUES ($1, $2, $3, 'support')",
    )
    .bind(id)
    .bind(format!("sub-{id}"))
    .bind(format!("{id}@company.example"))
    .execute(base)
    .await
    .unwrap();
    id
}

async fn correction(
    base: &PgPool,
    s: &Seed,
    staff: Uuid,
    kind: &str,
    params: serde_json::Value,
) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO support_corrections (id, workspace_id, staff_id, kind, params, reason)
         VALUES ($1, $2, $3, $4, $5, 'Store outage compensation')",
    )
    .bind(id)
    .bind(s.ws)
    .bind(staff)
    .bind(kind)
    .bind(params)
    .execute(base)
    .await
    .unwrap();
    id
}

async fn apply(pool: &PgPool, s: &Seed, id: Uuid) -> Option<String> {
    let mut tx = pool.begin().await.unwrap();
    enter_workspace(&mut tx, s.ws).await.unwrap();
    let out = panel::apply_correction(&mut tx, s.ws, id).await.unwrap();
    tx.commit().await.unwrap();
    out
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn the_worker_applies_corrections_and_the_api_cannot_write_entitlements(base: PgPool) {
    let s = seed(&base).await;
    let staff = staff(&base).await;
    let worker = worker_pool(&base).await;
    let api = api_pool(&base).await;

    // The API role has no write right on the entitlement tables.
    let mut tx = api.begin().await.unwrap();
    enter_workspace(&mut tx, s.ws).await.unwrap();
    let denied = sqlx::query(
        "INSERT INTO entitlements (id, workspace_id, feature, source, reason, corrected_by_staff_id)
         VALUES ($1, $2, 'paid_features', 'manual', 'x', $3)",
    )
    .bind(Uuid::now_v7())
    .bind(s.ws)
    .bind(staff)
    .execute(&mut *tx)
    .await;
    assert!(denied.is_err(), "app_api must not write entitlements");
    drop(tx);

    // A grant: a manual entitlement with the reason and the staff member, and an audit row.
    let id = correction(
        &base,
        &s,
        staff,
        "entitlement_grant",
        json!({"schema_version": 1}),
    )
    .await;
    assert_eq!(
        apply(&worker, &s, id).await.as_deref(),
        Some("manual entitlement written")
    );
    let (source, reason, by, open): (String, String, Uuid, bool) = sqlx::query_as(
        "SELECT source, reason, corrected_by_staff_id, valid_until > now() FROM entitlements",
    )
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!(
        (source.as_str(), reason.as_str(), by, open),
        ("manual", "Store outage compensation", staff, true)
    );
    let audit: (String, Option<String>) = sqlx::query_as(
        "SELECT actor_kind, reason FROM audit_log WHERE action = 'entitlement.manual_adjust'",
    )
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!(audit.0, "support");
    assert_eq!(audit.1.as_deref(), Some("Store outage compensation"));
    let status: String = sqlx::query_scalar("SELECT status FROM support_corrections WHERE id = $1")
        .bind(id)
        .fetch_one(&base)
        .await
        .unwrap();
    assert_eq!(status, "applied");
    // A second run does nothing.
    assert_eq!(apply(&worker, &s, id).await, None);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM entitlements")
            .fetch_one(&base)
            .await
            .unwrap(),
        1
    );

    // Revoke ends it.
    let id = correction(
        &base,
        &s,
        staff,
        "entitlement_revoke",
        json!({"schema_version": 1}),
    )
    .await;
    assert_eq!(
        apply(&worker, &s, id).await.as_deref(),
        Some("1 entitlements ended")
    );
    let open: bool = sqlx::query_scalar("SELECT valid_until > now() FROM entitlements")
        .fetch_one(&base)
        .await
        .unwrap();
    assert!(!open);

    // Re-verification is queued for the store client; a deletion request waits for a person.
    let id = correction(
        &base,
        &s,
        staff,
        "subscription_reverify",
        json!({"schema_version": 1}),
    )
    .await;
    assert_eq!(
        apply(&worker, &s, id).await.as_deref(),
        Some("re-verification queued")
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM outbox WHERE topic = 'subscription.reverify_requested'"
        )
        .fetch_one(&base)
        .await
        .unwrap(),
        1
    );
    let id = correction(
        &base,
        &s,
        staff,
        "deletion_request",
        json!({"schema_version": 1, "user_id": s.user}),
    )
    .await;
    let result = apply(&worker, &s, id).await.unwrap();
    assert!(result.starts_with("account deletion scheduled until "));
    let status: String = sqlx::query_scalar("SELECT status FROM support_corrections WHERE id = $1")
        .bind(id)
        .fetch_one(&base)
        .await
        .unwrap();
    assert_eq!(status, "applied");
    let target_retained: bool = sqlx::query_scalar(
        "SELECT params ? 'user_id' FROM support_corrections WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&base)
    .await
    .unwrap();
    assert!(!target_retained);
    let (request_status, purge_at): (String, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        "SELECT status, purge_at FROM account_deletion_requests WHERE user_id = $1",
    )
    .bind(s.user)
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!(request_status, "pending");
    assert!(purge_at > chrono::Utc::now());
}
