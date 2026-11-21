//! Notifications (T2.24): measurement -> alert rule -> notification -> push to a fake APNs.
//! Needs PostgreSQL (`DATABASE_URL`) and NATS (`NATS_URL`, compose default).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::sync::Arc;

use async_nats::jetstream;
use chrono::{DateTime, Utc};
use common::{Fake, Seed, nats_url, seed, wait_for, worker_pool};
use db::outbox::{Entity, NewOutboxEvent, enqueue};
use events::{
    consumer::{self, ConsumerConfig},
    relay,
};
use notifier::{
    alert::AlertHandler,
    apns::ApnsConfig,
    deliver::{Deliverer, DeliveryHandler, Outcome, deliver_one},
    enter_workspace, tick,
};
use serde_json::json;
use sqlx::PgPool;
use tokio::sync::watch;
use uuid::Uuid;

fn apns(fake: &Fake) -> Deliverer {
    Deliverer::new(
        Some(ApnsConfig {
            base_url: Some(fake.url.clone()),
            topic: "app.demleme.test".to_owned(),
            signer: None,
        }),
        None,
    )
}

async fn add_push_target(base: &PgPool, user: Uuid, token: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO push_targets (id, user_id, environment, token) VALUES ($1, $2, 'sandbox', $3)",
    )
    .bind(id)
    .bind(user)
    .bind(token)
    .execute(base)
    .await
    .unwrap();
    id
}

async fn add_rule(base: &PgPool, s: &Seed, priority: i16) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO alert_rules (id, workspace_id, name, condition, priority)
         VALUES ($1, $2, 'Too warm', $3, $4)",
    )
    .bind(id)
    .bind(s.ws)
    .bind(json!({"schema_version": 1, "measurement": "temp_c", "op": "gt", "value": 22.0}))
    .bind(priority)
    .execute(base)
    .await
    .unwrap();
    id
}

async fn add_measurement(base: &PgPool, s: &Seed, value: f64) {
    sqlx::query("SELECT create_measurement_partition(current_date)")
        .execute(base)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO measurements (id, workspace_id, batch_id, kind, value, measured_at, source,
                                   created_by)
         VALUES ($1, $2, $3, 'temp_c', $4, now(), 'manual', $5)",
    )
    .bind(Uuid::now_v7())
    .bind(s.ws)
    .bind(s.batch)
    .bind(value)
    .bind(s.user)
    .execute(base)
    .await
    .unwrap();
}

async fn emit_measurement_event(base: &PgPool, s: &Seed) {
    let mut tx = base.begin().await.unwrap();
    enqueue(
        &mut tx,
        &NewOutboxEvent {
            topic: "batch.measurement_added".to_owned(),
            workspace_id: Some(s.ws),
            entity: Some(Entity {
                kind: "batch".to_owned(),
                id: s.batch,
                version: 1,
            }),
            data: Some(json!({"count": 1})),
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn measurement_to_rule_to_notification_to_apns(base: PgPool) {
    let s = seed(&base).await;
    let fake = Fake::start(|_| (200, json!({}))).await;
    add_push_target(&base, s.user, "abcdef0123456789").await;
    add_rule(&base, &s, 1).await;
    let pool = worker_pool(&base).await;

    let client = events::connect(&nats_url()).await.unwrap();
    let js = jetstream::new(client);
    events::ensure_streams(&js, 1).await.unwrap();
    let (stop_tx, stop_rx) = watch::channel(false);
    let stopped = |mut rx: watch::Receiver<bool>| async move {
        while !*rx.borrow() {
            if rx.changed().await.is_err() {
                return;
            }
        }
    };
    let relay_task = tokio::spawn(relay::run(
        pool.clone(),
        js.clone(),
        relay::RelayConfig::default(),
        stopped(stop_rx.clone()),
    ));
    let alert_task = tokio::spawn(consumer::run(
        pool.clone(),
        js.clone(),
        ConsumerConfig::new(
            &format!("test-alerts-{}", s.ws),
            &format!("ev.{}.batch.>", s.ws),
        ),
        Arc::new(AlertHandler),
        stopped(stop_rx.clone()),
    ));
    let delivery_task = tokio::spawn(consumer::run(
        pool.clone(),
        js.clone(),
        ConsumerConfig::new(
            &format!("test-delivery-{}", s.ws),
            &format!("ev.{}.notification.>", s.ws),
        ),
        Arc::new(DeliveryHandler::new(Arc::new(apns(&fake)))),
        stopped(stop_rx.clone()),
    ));

    // A cool measurement fires nothing; a warm one fires the rule.
    add_measurement(&base, &s, 19.0).await;
    emit_measurement_event(&base, &s).await;
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
    assert!(fake.seen().is_empty());

    add_measurement(&base, &s, 23.4).await;
    emit_measurement_event(&base, &s).await;
    assert!(
        wait_for(15, || async { !fake.seen().is_empty() }).await,
        "no push arrived"
    );

    let push = fake.seen().remove(0);
    assert_eq!(push.method, "POST");
    assert_eq!(push.path, "/3/device/abcdef0123456789");
    assert_eq!(push.header("apns-topic"), Some("app.demleme.test"));
    assert_eq!(push.header("apns-priority"), Some("10"));
    assert_eq!(push.body["aps"]["interruption-level"], "time-sensitive");
    assert_eq!(
        push.body["aps"]["alert"]["title-loc-key"],
        "notification.alert.threshold"
    );

    // The push reaches the fake before the consumer transaction commits: wait for the commit.
    let committed = wait_for(10, || {
        let base = base.clone();
        async move {
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*) FROM notifications WHERE delivered_at IS NOT NULL",
            )
            .fetch_one(&base)
            .await
            .unwrap()
                == 1
        }
    })
    .await;
    assert!(committed, "delivery was not committed");
    let (status, delivered, kind): (String, bool, String) = sqlx::query_as(
        "SELECT status, delivered_at IS NOT NULL, kind FROM notifications WHERE workspace_id = $1",
    )
    .bind(s.ws)
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!(
        (status.as_str(), delivered, kind.as_str()),
        ("open", true, "alert.threshold")
    );
    let sent: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM notification_events WHERE workspace_id = $1 AND kind = 'sent'",
    )
    .bind(s.ws)
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!(sent, 1);

    // The same event again in the same cooldown window: no second notification, no second push.
    emit_measurement_event(&base, &s).await;
    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM notifications WHERE workspace_id = $1")
            .bind(s.ws)
            .fetch_one(&base)
            .await
            .unwrap();
    assert_eq!(count, 1);
    assert_eq!(fake.seen().len(), 1);

    stop_tx.send(true).unwrap();
    let _ = tokio::join!(relay_task, alert_task, delivery_task);
}

async fn insert_notification(base: &PgPool, s: &Seed, priority: i16) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO notifications (id, workspace_id, recipient_member_id, priority, kind, text_key,
                                   deliver_after)
         VALUES ($1, $2, $3, $4, 'alert.threshold', 'notification.alert.threshold', '2000-01-01')",
    )
    .bind(id)
    .bind(s.ws)
    .bind(s.member)
    .bind(priority)
    .execute(base)
    .await
    .unwrap();
    id
}

async fn deliver(pool: &PgPool, d: &Deliverer, s: &Seed, id: Uuid, now: DateTime<Utc>) -> Outcome {
    let mut tx = pool.begin().await.unwrap();
    enter_workspace(&mut tx, s.ws).await.unwrap();
    let out = deliver_one(&mut tx, d, s.ws, id, now).await.unwrap();
    tx.commit().await.unwrap();
    out
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn quiet_hours_defer_but_critical_goes_through(base: PgPool) {
    let s = seed(&base).await;
    let fake = Fake::start(|_| (200, json!({}))).await;
    add_push_target(&base, s.user, "aa11").await;
    sqlx::query(
        "INSERT INTO notification_preferences (id, workspace_id, user_id, timezone, quiet_start,
                                                quiet_end)
         VALUES ($1, $2, $3, 'UTC', '22:00', '07:00')",
    )
    .bind(Uuid::now_v7())
    .bind(s.ws)
    .bind(s.user)
    .execute(&base)
    .await
    .unwrap();
    let pool = worker_pool(&base).await;
    let d = apns(&fake);
    let night: DateTime<Utc> = "2026-11-20T23:30:00Z".parse().unwrap();
    let morning: DateTime<Utc> = "2026-11-21T07:00:00Z".parse().unwrap();

    let normal = insert_notification(&base, &s, 2).await;
    let critical = insert_notification(&base, &s, 1).await;
    assert_eq!(
        deliver(&pool, &d, &s, normal, night).await,
        Outcome::Deferred(morning)
    );
    assert!(matches!(
        deliver(&pool, &d, &s, critical, night).await,
        Outcome::Delivered { pushes: 1, .. }
    ));
    assert_eq!(fake.seen().len(), 1);

    // The tick delivers the deferred one once the quiet time is over, and only once.
    let handled = tick::run_due(&pool, &d, night).await.unwrap();
    assert_eq!(handled, 0);
    assert_eq!(tick::run_due(&pool, &d, morning).await.unwrap(), 1);
    assert_eq!(fake.seen().len(), 2);
    assert_eq!(tick::run_due(&pool, &d, morning).await.unwrap(), 0);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn low_priority_goes_to_the_summary_and_snooze_wakes(base: PgPool) {
    let s = seed(&base).await;
    let fake = Fake::start(|_| (200, json!({}))).await;
    add_push_target(&base, s.user, "bb22").await;
    sqlx::query(
        "INSERT INTO notification_preferences (id, workspace_id, user_id, timezone, daily_summary)
         VALUES ($1, $2, $3, 'UTC', true)",
    )
    .bind(Uuid::now_v7())
    .bind(s.ws)
    .bind(s.user)
    .execute(&base)
    .await
    .unwrap();
    let pool = worker_pool(&base).await;
    let d = apns(&fake);
    let now: DateTime<Utc> = "2026-11-20T12:00:00Z".parse().unwrap();

    let low = insert_notification(&base, &s, 3).await;
    assert!(matches!(
        deliver(&pool, &d, &s, low, now).await,
        Outcome::Delivered { pushes: 0, .. }
    ));
    assert!(fake.seen().is_empty());
    let suppressed: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM notification_events WHERE notification_id = $1 AND kind = 'suppressed'",
    )
    .bind(low)
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!(suppressed, 1);

    // The summary is due once per local day.
    let handled = tick::run_due(&pool, &d, now).await.unwrap();
    assert_eq!(handled, 1);
    assert_eq!(tick::run_due(&pool, &d, now).await.unwrap(), 0);

    // A snoozed notification wakes up and is pushed.
    let snoozed = insert_notification(&base, &s, 2).await;
    sqlx::query(
        "UPDATE notifications SET status = 'snoozed', snoozed_until = $2, delivered_at = $2
         WHERE id = $1",
    )
    .bind(snoozed)
    .bind(now)
    .execute(&base)
    .await
    .unwrap();
    assert_eq!(
        tick::run_due(&pool, &d, now + chrono::Duration::minutes(1))
            .await
            .unwrap(),
        1
    );
    assert_eq!(fake.seen().len(), 1);
    let status: String = sqlx::query_scalar("SELECT status FROM notifications WHERE id = $1")
        .bind(snoozed)
        .fetch_one(&base)
        .await
        .unwrap();
    assert_eq!(status, "open");
}
