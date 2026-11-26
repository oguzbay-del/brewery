//! Subscription end to end (T2.28): a signed App Store notification enters `ingest`, goes through
//! `external_events`, the outbox, NATS `JetStream` and the worker consumer, and changes the
//! entitlement. Needs PostgreSQL (`DATABASE_URL`) and NATS (`NATS_URL`, default
//! `nats://127.0.0.1:4222`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::{sync::Arc, time::Duration};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use billing::{
    access::{self, AccessLevel, AccessSource},
    provider::FakeProvider,
};
use chrono::Utc;
use db::{ActorKind, DbContext};
use events::{
    consumer::{self, Handler},
    relay,
};
use http_body_util::BodyExt;
use ingest::{IngestState, jws::JwsVerifier};
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use support::{Notification, Pki};
use tokio::sync::watch;
use tower::ServiceExt;
use uuid::Uuid;

const DAY_MS: i64 = 86_400_000;

fn nats_url() -> String {
    std::env::var("NATS_URL").unwrap_or_else(|_| "nats://127.0.0.1:4222".to_owned())
}

async fn pool_as(base: &PgPool, role: &'static str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(6)
        .after_connect(move |conn, _| {
            Box::pin(async move {
                sqlx::query(sqlx::AssertSqlSafe(format!("SET ROLE {role}")))
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .connect_with((*base.connect_options()).clone())
        .await
        .unwrap()
}

struct World {
    base: PgPool,
    api: PgPool,
    worker: PgPool,
    app: Router,
    pki: Pki,
    fake: Arc<FakeProvider>,
    js: async_nats::jetstream::Context,
    ws: Uuid,
    stop: watch::Sender<bool>,
    task: tokio::task::JoinHandle<()>,
}

impl World {
    /// A workspace whose trial ended (created two months ago) and a running consumer.
    async fn new(base: PgPool, trial_over: bool) -> Self {
        Self::with_product_ids(base, trial_over, Vec::new()).await
    }

    async fn with_product_ids(
        base: PgPool,
        trial_over: bool,
        apple_product_ids: Vec<String>,
    ) -> Self {
        let ws = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO workspaces (id, name, mode, region, country, time_zone, support_pseudonym,
                                     created_at)
             VALUES ($1, 'W', 'business', 'us', 'US', 'America/New_York', $2,
                     now() - make_interval(months => $3))",
        )
        .bind(ws)
        .bind(format!("pseudonym-{ws}"))
        .bind(if trial_over { 2 } else { 0 })
        .execute(&base)
        .await
        .unwrap();
        let api = pool_as(&base, "app_api").await;
        let worker = pool_as(&base, "app_worker").await;
        let pki = Pki::new(true);
        let fake = Arc::new(FakeProvider::new("s3cret"));
        let app = ingest::router_with(IngestState {
            pool: api.clone(),
            apple: JwsVerifier::with_roots(vec![pki.root_der.clone()], true),
            bundle_id: Some("app.demleme".to_owned()),
            apple_environment: "Production".to_owned(),
            apple_product_ids,
            web_payment: Some(fake.clone()),
            github_secret: None,
        });
        let client = events::connect(&nats_url()).await.unwrap();
        let js = async_nats::jetstream::new(client);
        events::ensure_streams(&js, 1).await.unwrap();
        let (stop, rx) = watch::channel(false);
        let task = tokio::spawn(consumer::run(
            worker.clone(),
            js.clone(),
            consumer::ConsumerConfig::new(
                &format!("sub-{}", Uuid::now_v7().simple()),
                &events::workspace_filter(ws),
            ),
            Arc::new(billing::apply::NotificationHandler),
            async move {
                let mut rx = rx;
                let _ = rx.wait_for(|s| *s).await;
            },
        ));
        Self {
            base,
            api,
            worker,
            app,
            pki,
            fake,
            js,
            ws,
            stop,
            task,
        }
    }

    async fn finish(self) {
        let _ = self.stop.send(true);
        let _ = self.task.await;
    }

    fn note(&self, id: &str, kind: &'static str, signed_ms: i64, expires_ms: i64) -> Notification {
        Notification {
            id: id.to_owned(),
            kind,
            subtype: None,
            workspace: self.ws.to_string(),
            original_transaction_id: format!("otx-{}", self.ws.simple()),
            product_id: "pro.monthly",
            signed_ms,
            expires_ms,
        }
    }

    async fn post(&self, signed_payload: &str) -> (StatusCode, Value) {
        let request = Request::post("/v1/webhooks/app-store")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "signedPayload": signed_payload }).to_string(),
            ))
            .unwrap();
        self.send(request).await
    }

    async fn send(&self, request: Request<Body>) -> (StatusCode, Value) {
        let response = self.app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    async fn relay(&self) {
        relay::run_once(&self.worker, &self.js, 100).await.unwrap();
    }

    async fn access(&self) -> access::Access {
        let mut tx = db::begin(
            &self.api,
            &DbContext {
                user_id: None,
                workspace_id: Some(self.ws),
                actor_kind: ActorKind::System,
            },
        )
        .await
        .unwrap();
        access::access(&mut tx, self.ws).await.unwrap()
    }

    async fn count(&self, sql: &'static str) -> i64 {
        sqlx::query_scalar(sql).fetch_one(&self.base).await.unwrap()
    }

    /// Relays the outbox and waits until every external event is processed.
    async fn settle(&self, expected_processed: i64) {
        for _ in 0..200 {
            self.relay().await;
            let n = self
                .count("SELECT count(*) FROM external_events WHERE processed_at IS NOT NULL")
                .await;
            if n >= expected_processed {
                return;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        panic!("notifications were not processed");
    }
}

fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn purchase_renewal_and_refund(base: PgPool) {
    let w = World::new(base, true).await;
    assert_eq!(
        w.access().await.level,
        AccessLevel::ReadOnly,
        "trial is over"
    );

    // The store signs in the past: signed dates are before the database time.
    let t = now_ms() - 60_000;
    let buy = w
        .pki
        .notification(&w.note("n-buy", "SUBSCRIBED", t, t + 30 * DAY_MS));
    assert_eq!(w.post(&buy).await.0, StatusCode::OK);
    w.settle(1).await;
    let a = w.access().await;
    assert_eq!(
        (a.level, a.source),
        (AccessLevel::Full, AccessSource::Subscription)
    );

    // Renewal: the paid period moves forward, there is still one subscription and one entitlement.
    let renew = w
        .pki
        .notification(&w.note("n-renew", "DID_RENEW", t + 1000, t + 60 * DAY_MS));
    assert_eq!(w.post(&renew).await.0, StatusCode::OK);
    w.settle(2).await;
    let until = w.access().await.valid_until.unwrap().timestamp_millis();
    assert!(
        (until - (t + 60 * DAY_MS)).abs() < 1000,
        "renewed to 60 days"
    );
    assert_eq!(w.count("SELECT count(*) FROM subscriptions").await, 1);
    assert_eq!(w.count("SELECT count(*) FROM entitlements").await, 1);

    // Refund: access ends at once.
    let refund = w
        .pki
        .notification(&w.note("n-refund", "REFUND", t + 2000, t + 60 * DAY_MS));
    assert_eq!(w.post(&refund).await.0, StatusCode::OK);
    w.settle(3).await;
    let a = w.access().await;
    assert_eq!(
        (a.level, a.source),
        (AccessLevel::ReadOnly, AccessSource::None)
    );
    let status: String = sqlx::query_scalar("SELECT status FROM subscriptions")
        .fetch_one(&w.base)
        .await
        .unwrap();
    assert_eq!(status, "cancelled");
    // Each change wrote a `subscription.changed` event for the clients.
    assert_eq!(
        w.count("SELECT count(*) FROM outbox WHERE topic = 'subscription.changed'")
            .await,
        3
    );
    w.finish().await;
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn only_allowed_apple_products_are_stored(base: PgPool) {
    let w = World::with_product_ids(base, true, vec!["pro.monthly".to_owned()]).await;
    let t = now_ms() - 60_000;
    let mut unknown = w.note("n-unknown", "SUBSCRIBED", t, t + 30 * DAY_MS);
    unknown.product_id = "other.monthly";
    let (status, body) = w.post(&w.pki.notification(&unknown)).await;
    assert_eq!(
        (status, body["status"].as_str()),
        (StatusCode::OK, Some("ignored"))
    );
    assert_eq!(w.count("SELECT count(*) FROM external_events").await, 0);
    assert_eq!(w.count("SELECT count(*) FROM outbox").await, 0);
    assert_eq!(w.count("SELECT count(*) FROM subscriptions").await, 0);

    let allowed = w.note("n-allowed", "SUBSCRIBED", t, t + 30 * DAY_MS);
    let (status, body) = w.post(&w.pki.notification(&allowed)).await;
    assert_eq!(
        (status, body["status"].as_str()),
        (StatusCode::OK, Some("accepted"))
    );
    w.settle(1).await;
    assert_eq!(w.count("SELECT count(*) FROM subscriptions").await, 1);
    assert_eq!(w.access().await.source, AccessSource::Subscription);
    w.finish().await;
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn the_same_notification_is_not_processed_twice(base: PgPool) {
    let w = World::new(base, true).await;
    let t = now_ms();
    let buy = w
        .pki
        .notification(&w.note("n-dup", "SUBSCRIBED", t, t + 30 * DAY_MS));

    let (s1, b1) = w.post(&buy).await;
    let (s2, b2) = w.post(&buy).await;
    assert_eq!(
        (s1, b1["status"].as_str()),
        (StatusCode::OK, Some("accepted"))
    );
    assert_eq!(
        (s2, b2["status"].as_str()),
        (StatusCode::OK, Some("duplicate"))
    );
    assert_eq!(w.count("SELECT count(*) FROM external_events").await, 1);
    assert_eq!(
        w.count("SELECT count(*) FROM outbox WHERE topic = 'subscription.notification_received'")
            .await,
        1
    );
    w.settle(1).await;
    // A third delivery after processing is a duplicate too.
    assert_eq!(w.post(&buy).await.1["status"], "duplicate");
    w.relay().await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(w.count("SELECT count(*) FROM subscriptions").await, 1);
    assert_eq!(w.count("SELECT count(*) FROM entitlements").await, 1);
    assert_eq!(
        w.count("SELECT count(*) FROM outbox WHERE topic = 'subscription.changed'")
            .await,
        1,
        "the effect happened once"
    );

    // The worker replays the stored event (a lost acknowledgement): the marker stops it.
    let event_id: Uuid = sqlx::query_scalar("SELECT id FROM external_events")
        .fetch_one(&w.base)
        .await
        .unwrap();
    let event = consumer::Event {
        id: Uuid::now_v7(),
        subject: String::new(),
        topic: billing::NOTIFICATION_TOPIC.to_owned(),
        workspace_id: Some(w.ws),
        body: json!({ "entity": { "kind": "external_event", "id": event_id } }),
        attempt: 1,
    };
    let mut conn = w.worker.acquire().await.unwrap();
    billing::apply::NotificationHandler
        .handle(&event, &mut conn)
        .await
        .unwrap();
    drop(conn);
    assert_eq!(
        w.count("SELECT count(*) FROM outbox WHERE topic = 'subscription.changed'")
            .await,
        1
    );
    w.finish().await;
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn an_older_notification_does_not_undo_a_newer_one(base: PgPool) {
    let w = World::new(base, true).await;
    let t = now_ms();
    let renew = w
        .pki
        .notification(&w.note("n-new", "DID_RENEW", t + 5000, t + 30 * DAY_MS));
    let late_expired = w
        .pki
        .notification(&w.note("n-old", "EXPIRED", t + 1000, t + DAY_MS));
    w.post(&renew).await;
    w.settle(1).await;
    w.post(&late_expired).await;
    w.settle(2).await;
    assert_eq!(w.access().await.level, AccessLevel::Full);
    let status: String = sqlx::query_scalar("SELECT status FROM subscriptions")
        .fetch_one(&w.base)
        .await
        .unwrap();
    assert_eq!(status, "active");
    w.finish().await;
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn grace_keeps_access_and_expiry_ends_it(base: PgPool) {
    let w = World::new(base, true).await;
    let t = now_ms();
    let mut grace = w.note("n-grace", "DID_FAIL_TO_RENEW", t, t - DAY_MS);
    grace.subtype = Some("GRACE_PERIOD");
    w.post(&w.pki.notification(&grace)).await;
    w.settle(1).await;
    assert_eq!(
        w.access().await.level,
        AccessLevel::Full,
        "default grace of 16 days"
    );

    let expired =
        w.pki
            .notification(&w.note("n-exp", "GRACE_PERIOD_EXPIRED", t + 1000, t - DAY_MS));
    w.post(&expired).await;
    w.settle(2).await;
    assert_eq!(w.access().await.level, AccessLevel::ReadOnly);
    w.finish().await;
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn trial_lasts_one_month_then_paid_features_are_read_only(base: PgPool) {
    let w = World::new(base.clone(), false).await;
    let a = w.access().await;
    assert_eq!(
        (a.level, a.source),
        (AccessLevel::Full, AccessSource::Trial)
    );
    let span = a.trial_ends_at - Utc::now();
    assert!((27..=31).contains(&span.num_days()), "about one month");

    // Time passes: the workspace is now 32 days old.
    sqlx::query("UPDATE workspaces SET created_at = now() - interval '32 days' WHERE id = $1")
        .bind(w.ws)
        .execute(&base)
        .await
        .unwrap();
    let a = w.access().await;
    assert_eq!(
        (a.level, a.source),
        (AccessLevel::ReadOnly, AccessSource::None)
    );

    // A manual correction by support reopens access (valid, with reason and staff user is
    // required by the table; a promo entitlement needs neither).
    sqlx::query(
        "INSERT INTO entitlements (id, workspace_id, feature, source, valid_until)
         VALUES ($1, $2, 'paid_features', 'promo', now() + interval '7 days')",
    )
    .bind(Uuid::now_v7())
    .bind(w.ws)
    .execute(&base)
    .await
    .unwrap();
    let a = w.access().await;
    assert_eq!(
        (a.level, a.source),
        (AccessLevel::Full, AccessSource::Promo)
    );
    w.finish().await;
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn bad_signature_and_bad_chain_are_rejected_and_nothing_is_stored(base: PgPool) {
    let w = World::new(base, true).await;
    let t = now_ms();
    let good = w
        .pki
        .notification(&w.note("n-1", "SUBSCRIBED", t, t + DAY_MS));

    // Tampered payload.
    let other = w
        .pki
        .notification(&w.note("n-2", "SUBSCRIBED", t, t + 400 * DAY_MS));
    let mut parts: Vec<&str> = good.split('.').collect();
    parts[1] = other.split('.').nth(1).unwrap();
    assert_eq!(w.post(&parts.join(".")).await.0, StatusCode::UNAUTHORIZED);

    // A chain of another root.
    let stranger = Pki::new(true);
    let forged = stranger.notification(&w.note("n-3", "SUBSCRIBED", t, t + DAY_MS));
    assert_eq!(w.post(&forged).await.0, StatusCode::UNAUTHORIZED);

    // Not a JWS.
    assert_eq!(w.post("garbage").await.0, StatusCode::BAD_REQUEST);
    // Not the JSON body.
    let request = Request::post("/v1/webhooks/app-store")
        .header("content-type", "application/json")
        .body(Body::from("{}"))
        .unwrap();
    assert!(w.send(request).await.0.is_client_error());

    assert_eq!(w.count("SELECT count(*) FROM external_events").await, 0);
    assert_eq!(w.count("SELECT count(*) FROM outbox").await, 0);
    assert_eq!(w.post(&good).await.0, StatusCode::OK);
    w.finish().await;
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn ignored_notification_types_store_nothing(base: PgPool) {
    let w = World::new(base, true).await;
    let t = now_ms();
    let test = w
        .pki
        .notification(&w.note("n-t", "DID_CHANGE_RENEWAL_STATUS", t, t));
    let (status, body) = w.post(&test).await;
    assert_eq!(
        (status, body["status"].as_str()),
        (StatusCode::OK, Some("ignored"))
    );
    assert_eq!(w.count("SELECT count(*) FROM external_events").await, 0);
    w.finish().await;
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn web_payment_provider_flows_through_the_same_path(base: PgPool) {
    let w = World::new(base, true).await;
    let t = Utc::now();
    let update = billing::SubscriptionUpdate {
        schema_version: 1,
        source: billing::Source::Web,
        notification_id: "web-evt-1".to_owned(),
        kind: billing::UpdateKind::Active,
        workspace_id: Some(w.ws),
        external_id: "sub_web_1".to_owned(),
        product_key: "pro.monthly".to_owned(),
        period_ends_at: Some(t + chrono::Duration::days(30)),
        occurred_at: t,
    };
    let post = |signature: Option<&'static str>, body: String| {
        let mut b = Request::post("/v1/webhooks/web-payment");
        if let Some(s) = signature {
            b = b.header("x-signature", s);
        }
        w.send(b.body(Body::from(body)).unwrap())
    };
    let body = serde_json::to_string(&update).unwrap();
    assert_eq!(
        post(Some("wrong"), body.clone()).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(post(None, body.clone()).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(
        post(Some("s3cret"), "nope".into()).await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        post(Some("s3cret"), r#"{"ignore":true}"#.into()).await.1["status"],
        "ignored"
    );
    assert_eq!(
        post(Some("s3cret"), body.clone()).await.1["status"],
        "accepted"
    );
    assert_eq!(post(Some("s3cret"), body).await.1["status"], "duplicate");
    w.settle(1).await;
    let a = w.access().await;
    assert_eq!(
        (a.level, a.source),
        (AccessLevel::Full, AccessSource::Subscription)
    );
    let source: String = sqlx::query_scalar("SELECT source FROM subscriptions")
        .fetch_one(&w.base)
        .await
        .unwrap();
    assert_eq!(source, "web");
    assert_eq!(w.fake.accepted(), 3);
    w.finish().await;
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn notification_without_a_known_workspace_goes_to_the_dead_letter_path(base: PgPool) {
    let w = World::new(base, true).await;
    let t = now_ms();
    let mut n = w.note("n-lost", "SUBSCRIBED", t, t + DAY_MS);
    n.workspace = Uuid::now_v7().to_string();
    let (status, _) = w.post(&w.pki.notification(&n)).await;
    assert_eq!(status, StatusCode::OK);
    // The relay publishes under the unknown workspace, so this consumer never sees it. The
    // handler rejects it for good when it does (permanent error, no retry).
    let event_id: Uuid = sqlx::query_scalar("SELECT id FROM external_events")
        .fetch_one(&w.base)
        .await
        .unwrap();
    let event = consumer::Event {
        id: Uuid::now_v7(),
        subject: String::new(),
        topic: billing::NOTIFICATION_TOPIC.to_owned(),
        workspace_id: None,
        body: json!({ "entity": { "kind": "external_event", "id": event_id } }),
        attempt: 1,
    };
    let mut conn = w.worker.acquire().await.unwrap();
    let result = billing::apply::NotificationHandler
        .handle(&event, &mut conn)
        .await;
    assert!(
        matches!(result, Err(consumer::HandlerError::Permanent(_))),
        "{result:?}"
    );
    drop(conn);
    assert_eq!(w.count("SELECT count(*) FROM subscriptions").await, 0);
    w.finish().await;
}
