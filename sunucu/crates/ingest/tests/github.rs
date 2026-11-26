//! GitHub webhook of the support repository (T2.29): signature, repeated delivery, storage.
//! Needs PostgreSQL (`DATABASE_URL`).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::format_collect)]

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use ingest::{IngestState, jws::JwsVerifier};
use ring::hmac;
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use tower::ServiceExt;

const SECRET: &[u8] = b"hook-secret";

async fn app(base: &PgPool, secret: Option<&[u8]>) -> axum::Router {
    let api = PgPoolOptions::new()
        .max_connections(4)
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("SET ROLE app_api").execute(conn).await?;
                Ok(())
            })
        })
        .connect_with((*base.connect_options()).clone())
        .await
        .unwrap();
    ingest::router_with(IngestState {
        pool: api,
        apple: JwsVerifier::apple(),
        bundle_id: None,
        apple_environment: "Production".to_owned(),
        apple_product_ids: Vec::new(),
        web_payment: None,
        github_secret: secret.map(|s| Arc::new(s.to_vec())),
    })
}

fn sign(body: &[u8]) -> String {
    let tag = hmac::sign(&hmac::Key::new(hmac::HMAC_SHA256, SECRET), body);
    let hex: String = tag.as_ref().iter().map(|b| format!("{b:02x}")).collect();
    format!("sha256={hex}")
}

async fn post(
    app: &axum::Router,
    body: &Value,
    signature: Option<String>,
    delivery: &str,
) -> (StatusCode, Value) {
    let raw = body.to_string();
    let mut b = Request::post("/v1/webhooks/github")
        .header("content-type", "application/json")
        .header("x-github-event", "issue_comment")
        .header("x-github-delivery", delivery);
    if let Some(s) = signature.or_else(|| Some(sign(raw.as_bytes()))) {
        b = b.header("x-hub-signature-256", s);
    }
    let r = app
        .clone()
        .oneshot(b.body(Body::from(raw)).unwrap())
        .await
        .unwrap();
    let status = r.status();
    let bytes = r.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn signature_is_checked_and_a_repeat_is_stored_once(base: PgPool) {
    let app = app(&base, Some(SECRET)).await;
    let payload = json!({"action": "created", "issue": {"number": 7},
        "comment": {"id": 5, "body": "/yanit hi", "user": {"login": "alice"}}});

    let (status, _) = post(&app, &payload, Some("sha256=00".to_owned()), "d-1").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, body) = post(&app, &payload, None, "d-1").await;
    assert_eq!(
        (status, body["status"].as_str()),
        (StatusCode::OK, Some("accepted"))
    );
    let (status, body) = post(&app, &payload, None, "d-1").await;
    assert_eq!(
        (status, body["status"].as_str()),
        (StatusCode::OK, Some("duplicate"))
    );

    let events: i64 =
        sqlx::query_scalar("SELECT count(*) FROM external_events WHERE source = 'github'")
            .fetch_one(&base)
            .await
            .unwrap();
    let outbox: i64 =
        sqlx::query_scalar("SELECT count(*) FROM outbox WHERE topic = 'support.github_event'")
            .fetch_one(&base)
            .await
            .unwrap();
    assert_eq!((events, outbox), (1, 1));
    let stored: Value = sqlx::query_scalar("SELECT payload FROM external_events")
        .fetch_one(&base)
        .await
        .unwrap();
    assert_eq!(stored["event"], "issue_comment");
    assert_eq!(stored["payload"]["comment"]["id"], 5);

    // A new delivery id is a new event.
    let (_, body) = post(&app, &payload, None, "d-2").await;
    assert_eq!(body["status"], "accepted");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn without_a_secret_the_route_answers_503(base: PgPool) {
    let app = app(&base, None).await;
    let (status, _) = post(&app, &json!({}), None, "d-1").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}
