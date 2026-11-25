//! Paddle webhook ingestion test. Requires PostgreSQL and NATS.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use billing::paddle::PaddleProvider;
use ingest::{IngestState, jws::JwsVerifier};
use ring::hmac;
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{collections::HashMap, fmt::Write as _, sync::Arc};
use tower::ServiceExt;

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn signed_delivery_is_deduplicated_and_bad_signature_is_rejected(base: PgPool) {
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://127.0.0.1:4222".to_owned());
    let nats = events::connect(&nats_url).await.unwrap();
    let jetstream = async_nats::jetstream::new(nats);
    events::ensure_streams(&jetstream, 1).await.unwrap();

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
    let provider = PaddleProvider::new(
        "test-secret",
        HashMap::from([("pri_home".to_owned(), "home_pro".to_owned())]),
        16,
        300,
    );
    let app = ingest::router_with(IngestState {
        pool: api,
        apple: JwsVerifier::apple(),
        bundle_id: None,
        web_payment: Some(Arc::new(provider)),
        github_secret: None,
    });

    let event_id = format!("evt_{}", uuid::Uuid::now_v7().simple());
    let body = serde_json::to_vec(&json!({
        "event_id": event_id,
        "event_type": "subscription.activated",
        "occurred_at": "2026-11-25T12:00:00Z",
        "notification_id": "ntf_different",
        "data": {
            "id": "sub_test",
            "status": "active",
            "items": [{"price": {"id": "pri_home"}}],
            "custom_data": null,
            "current_billing_period": {"ends_at": "2026-12-25T12:00:00Z"}
        }
    }))
    .unwrap();
    let timestamp = chrono::Utc::now().timestamp();
    let mut signed = timestamp.to_string().into_bytes();
    signed.push(b':');
    signed.extend_from_slice(&body);
    let tag = hmac::sign(&hmac::Key::new(hmac::HMAC_SHA256, b"test-secret"), &signed);
    let hex = tag.as_ref().iter().fold(String::new(), |mut output, byte| {
        let _ = write!(&mut output, "{byte:02x}");
        output
    });
    let signature = format!("ts={timestamp};h1={hex}");

    let send = |sig: String, payload: Vec<u8>| {
        let app = app.clone();
        async move {
            let response = app
                .oneshot(
                    Request::post("/v1/webhooks/web-payment")
                        .header("paddle-signature", sig)
                        .body(Body::from(payload))
                        .unwrap(),
                )
                .await
                .unwrap();
            let status = response.status();
            let bytes = http_body_util::BodyExt::collect(response.into_body())
                .await
                .unwrap()
                .to_bytes();
            (status, serde_json::from_slice::<Value>(&bytes).unwrap())
        }
    };

    let (first_status, first_body) = send(signature.clone(), body.clone()).await;
    assert_eq!(first_status, StatusCode::OK);
    assert_eq!(first_body["status"], "accepted");

    let (second_status, second_body) = send(signature, body.clone()).await;
    assert_eq!(second_status, StatusCode::OK);
    assert_eq!(second_body["status"], "duplicate");

    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM external_events WHERE source='web_payment' AND delivery_id=$1",
    )
    .bind(&event_id)
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!(count, 1);

    let (bad_status, _) = send(format!("ts={timestamp};h1=00"), body).await;
    assert_eq!(bad_status, StatusCode::UNAUTHORIZED);
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM external_events WHERE source='web_payment' AND delivery_id=$1",
    )
    .bind(&event_id)
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!(count, 1);
}
