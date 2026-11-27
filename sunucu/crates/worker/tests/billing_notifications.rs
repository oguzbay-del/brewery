//! Billing notifications (B7): the trial ends in 7 days and in 1 day, a payment failed, the
//! subscription ended. Each one fires once; a rerun or a repeated event creates nothing new.
//! Needs PostgreSQL (`DATABASE_URL`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use billing::notify::{BillingNotificationHandler, run_trial_reminders};
use chrono::Utc;
use common::{Seed, seed, worker_pool};
use events::consumer::{Event, Handler};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// Gives the seeded member the permission `billing.manage`.
async fn make_billing_manager(base: &PgPool, s: &Seed) {
    let role = Uuid::now_v7();
    sqlx::query("INSERT INTO roles (id, workspace_id, name) VALUES ($1, $2, 'Billing')")
        .bind(role)
        .bind(s.ws)
        .execute(base)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO role_permissions (workspace_id, role_id, permission)
         VALUES ($1, $2, 'billing.manage')",
    )
    .bind(s.ws)
    .bind(role)
    .execute(base)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO role_assignments (id, workspace_id, member_id, role_id)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(s.ws)
    .bind(s.member)
    .bind(role)
    .execute(base)
    .await
    .unwrap();
}

async fn count(base: &PgPool, kind: &str) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM notifications WHERE kind = $1")
        .bind(kind)
        .fetch_one(base)
        .await
        .unwrap()
}

fn changed(s: &Seed, subscription: Uuid, version: i64, status: &str) -> Event {
    Event {
        id: Uuid::now_v7(),
        subject: String::new(),
        topic: billing::CHANGED_TOPIC.to_owned(),
        workspace_id: Some(s.ws),
        body: json!({
            "entity": {"kind": "subscription", "id": subscription, "version": version},
            "data": {"status": status},
        }),
        attempt: 1,
    }
}

async fn deliver(pool: &PgPool, event: &Event) {
    // The consumer framework runs a handler inside a transaction; the workspace context is
    // `SET LOCAL`, so it needs one here too.
    let mut tx = pool.begin().await.unwrap();
    BillingNotificationHandler
        .handle(event, &mut tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn trial_reminders_fire_once_for_each_threshold(base: PgPool) {
    let s = seed(&base).await;
    make_billing_manager(&base, &s).await;
    let pool = worker_pool(&base).await;

    // Far from the end: nothing.
    assert_eq!(run_trial_reminders(&pool, Utc::now()).await.unwrap(), 0);

    // Home trial is 3 months: 5 days left gives the 7-day reminder, once.
    sqlx::query(
        "UPDATE workspaces SET created_at = now() - interval '3 months' + interval '5 days'
         WHERE id = $1",
    )
    .bind(s.ws)
    .execute(&base)
    .await
    .unwrap();
    assert_eq!(run_trial_reminders(&pool, Utc::now()).await.unwrap(), 1);
    assert_eq!(run_trial_reminders(&pool, Utc::now()).await.unwrap(), 0);
    assert_eq!(count(&base, "billing.trial_ending").await, 1);

    // 12 hours left: the 1-day reminder, once.
    sqlx::query(
        "UPDATE workspaces SET created_at = now() - interval '3 months' + interval '12 hours'
         WHERE id = $1",
    )
    .bind(s.ws)
    .execute(&base)
    .await
    .unwrap();
    assert_eq!(run_trial_reminders(&pool, Utc::now()).await.unwrap(), 1);
    assert_eq!(run_trial_reminders(&pool, Utc::now()).await.unwrap(), 0);
    assert_eq!(count(&base, "billing.trial_ending").await, 2);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_paid_workspace_gets_no_trial_reminder(base: PgPool) {
    let s = seed(&base).await;
    make_billing_manager(&base, &s).await;
    sqlx::query(
        "UPDATE workspaces SET created_at = now() - interval '3 months' + interval '12 hours'
         WHERE id = $1",
    )
    .bind(s.ws)
    .execute(&base)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO entitlements (id, workspace_id, feature, source, reason, corrected_by_staff_id,
                                   valid_until)
         SELECT $1, $2, 'paid_features', 'promo', NULL, NULL, now() + interval '30 days'",
    )
    .bind(Uuid::now_v7())
    .bind(s.ws)
    .execute(&base)
    .await
    .unwrap();
    let pool = worker_pool(&base).await;
    assert_eq!(run_trial_reminders(&pool, Utc::now()).await.unwrap(), 0);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn payment_failed_and_ended_fire_once_for_each_change(base: PgPool) {
    let s = seed(&base).await;
    make_billing_manager(&base, &s).await;
    let pool = worker_pool(&base).await;
    let subscription = Uuid::now_v7();

    let failed = changed(&s, subscription, 2, "grace");
    deliver(&pool, &failed).await;
    deliver(&pool, &failed).await; // the same event again
    assert_eq!(count(&base, "billing.payment_failed").await, 1);
    // An active change creates nothing.
    deliver(&pool, &changed(&s, subscription, 3, "active")).await;
    assert_eq!(count(&base, "billing.payment_failed").await, 1);
    // A later failed payment is a new change.
    deliver(&pool, &changed(&s, subscription, 4, "grace")).await;
    assert_eq!(count(&base, "billing.payment_failed").await, 2);

    let ended = changed(&s, subscription, 5, "expired");
    deliver(&pool, &ended).await;
    deliver(&pool, &ended).await;
    assert_eq!(count(&base, "billing.ended").await, 1);
}
