//! Apple token revoke in the account purge job (T2.31d). Needs PostgreSQL (`#[sqlx::test]`).
//! The Apple endpoint is an in-process fake, except for the `WireMock` mapping test.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

mod common;

use std::sync::{Arc, Mutex};

use apple_auth::{
    Aes256GcmTokenCipher, AppleClient, AppleTokenSettings, RevokeOutcome, TokenCipher,
};
use axum::{Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use common::{api_pool, seed, worker_pool};
use sqlx::PgPool;
use uuid::Uuid;
use worker::apple_revoke::AppleRevoker;

/// A P-256 PKCS#8 key that exists only for these tests.
const TEST_KEY: &str = "MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgSaGE3ZKkQZJgH+LcM3n4yNgtBBMlEXJ0bSbyY8oLOW+hRANCAAQ7q1JSd3lW7ElucYe49rIMvJe3tEjBETAsZLHAeKkSy77zsTmu6cxrFvruNCLYlqlFglQCIlHmrEQXTVMeKpvr";

#[derive(Clone, Default)]
struct Fake {
    bodies: Arc<Mutex<Vec<String>>>,
    status: Arc<Mutex<u16>>,
}

async fn revoke(State(fake): State<Fake>, body: String) -> impl IntoResponse {
    fake.bodies.lock().unwrap().push(body);
    let status = *fake.status.lock().unwrap();
    (
        StatusCode::from_u16(status).unwrap(),
        [("content-type", "application/json")],
        "{}",
    )
}

async fn start_fake(status: u16) -> (String, Fake) {
    let fake = Fake {
        status: Arc::new(Mutex::new(status)),
        ..Fake::default()
    };
    let app = Router::new()
        .route("/auth/revoke", post(revoke))
        .with_state(fake.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/auth/revoke", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (url, fake)
}

fn settings(revoke_url: &str) -> AppleTokenSettings {
    AppleTokenSettings {
        team_id: "TEAM123".to_owned(),
        key_id: "KEY456".to_owned(),
        private_key: STANDARD.decode(TEST_KEY).unwrap(),
        client_ids: vec!["com.example.ios".to_owned()],
        token_url: "http://127.0.0.1:1/auth/token".to_owned(),
        revoke_url: revoke_url.to_owned(),
    }
}

fn revoker(revoke_url: &str) -> AppleRevoker {
    AppleRevoker::new(
        AppleClient::new(settings(revoke_url)).unwrap(),
        Box::new(Aes256GcmTokenCipher::new([3; 32])),
    )
}

/// Inserts an Apple identity with a stored v1 payload. Returns the identity id.
async fn add_identity(base: &PgPool, user_id: Uuid, token: &str) -> Uuid {
    let cipher = Aes256GcmTokenCipher::new([3; 32]);
    let plaintext = apple_auth::ApplePayload::encode_v1("com.example.ios", token).unwrap();
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO user_identities (id, user_id, provider, subject, provider_refresh_token_enc)
         VALUES ($1, $2, 'apple', $3, $4)",
    )
    .bind(id)
    .bind(user_id)
    .bind(format!("apple-subject-{id}"))
    .bind(cipher.encrypt(&plaintext).unwrap())
    .execute(base)
    .await
    .unwrap();
    id
}

async fn make_due(base: &PgPool, user_id: Uuid) {
    let api = api_pool(base).await;
    let mut tx = db::begin(
        &api,
        &db::DbContext {
            user_id: Some(user_id),
            workspace_id: None,
            actor_kind: db::ActorKind::User,
        },
    )
    .await
    .unwrap();
    let request = db::request_account_deletion(&mut tx, 30).await.unwrap();
    tx.commit().await.unwrap();
    sqlx::query(
        "UPDATE account_deletion_requests
         SET requested_at = now() - interval '2 seconds', purge_at = now() - interval '1 second'
         WHERE id = $1",
    )
    .bind(request.id)
    .execute(base)
    .await
    .unwrap();
}

async fn stored(base: &PgPool, id: Uuid) -> (Option<Vec<u8>>, i16) {
    sqlx::query_as("SELECT provider_refresh_token_enc, apple_revoke_attempts FROM user_identities WHERE id = $1")
        .bind(id)
        .fetch_one(base)
        .await
        .unwrap()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn successful_revoke_clears_the_ciphertext(base: PgPool) {
    let s = seed(&base).await;
    let identity = add_identity(&base, s.user, "r.secret.token").await;
    let other = seed(&base).await;
    let other_identity = add_identity(&base, other.user, "r.other.token").await;
    make_due(&base, s.user).await;
    let (url, fake) = start_fake(200).await;
    let worker = worker_pool(&base).await;

    let (_, detail) =
        worker::account_deletion::purge_due_and_revoke(&worker, None, Some(&revoker(&url)))
            .await
            .unwrap();

    assert_eq!(stored(&base, identity).await.0, None);
    assert!(stored(&base, other_identity).await.0.is_some());
    let bodies = fake.bodies.lock().unwrap();
    assert_eq!(bodies.len(), 1);
    assert!(bodies[0].contains("token=r.secret.token"));
    assert!(bodies[0].contains("client_id=com.example.ios"));
    assert!(bodies[0].contains("client_secret="));
    assert!(detail.contains(r#""revoked":1"#));
    assert!(!detail.contains("r.secret.token"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn ciphertext_is_cleared_after_the_third_failed_revoke(base: PgPool) {
    let s = seed(&base).await;
    let identity = add_identity(&base, s.user, "r.secret.token").await;
    make_due(&base, s.user).await;
    let (url, fake) = start_fake(500).await;
    let worker = worker_pool(&base).await;
    let apple = revoker(&url);

    for attempt in 1..=2 {
        worker::account_deletion::purge_due_and_revoke(&worker, None, Some(&apple))
            .await
            .unwrap();
        let (ciphertext, attempts) = stored(&base, identity).await;
        assert!(
            ciphertext.is_some(),
            "attempt {attempt} keeps the ciphertext"
        );
        assert_eq!(attempts, attempt);
    }
    let (status, detail) =
        worker::account_deletion::purge_due_and_revoke(&worker, None, Some(&apple))
            .await
            .unwrap();
    let (ciphertext, attempts) = stored(&base, identity).await;
    assert_eq!(ciphertext, None);
    assert_eq!(attempts, 3);
    assert!(detail.contains(r#""given_up":1"#));
    assert_eq!(status, db::maintenance::RunStatus::Findings);
    assert_eq!(fake.bodies.lock().unwrap().len(), 3);

    // No ciphertext is left, so a later run does not call Apple again.
    worker::account_deletion::purge_due_and_revoke(&worker, None, Some(&apple))
        .await
        .unwrap();
    assert_eq!(fake.bodies.lock().unwrap().len(), 3);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn missing_apple_settings_keep_the_ciphertext(base: PgPool) {
    let s = seed(&base).await;
    let identity = add_identity(&base, s.user, "r.secret.token").await;
    make_due(&base, s.user).await;
    let worker = worker_pool(&base).await;
    let (_, detail) = worker::account_deletion::purge_due_and_revoke(&worker, None, None)
        .await
        .unwrap();
    assert!(stored(&base, identity).await.0.is_some());
    assert!(detail.contains(r#""skipped":1"#));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn api_role_cannot_call_the_revoke_functions(base: PgPool) {
    let api = api_pool(&base).await;
    assert!(
        sqlx::query("SELECT * FROM apple_revoke_pending(10)")
            .fetch_all(&api)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("SELECT apple_revoke_clear($1)")
            .bind(Uuid::now_v7())
            .fetch_one(&api)
            .await
            .is_err()
    );
}

/// The compose `WireMock` mapping `apple-revoke.json`. Skipped when the mock does not run.
#[tokio::test]
async fn revoke_matches_the_wiremock_mapping() {
    let url = std::env::var("APPLE_REVOKE_URL")
        .ok()
        .filter(|url| !url.contains("apple-mock:8080"))
        .unwrap_or_else(|| "http://127.0.0.1:8092/auth/revoke".to_owned());
    let host = url::Url::parse(&url).unwrap();
    let addr = format!("{}:{}", host.host_str().unwrap(), host.port().unwrap_or(80));
    if tokio::net::TcpStream::connect(&addr).await.is_err() {
        eprintln!("apple-mock is not running at {addr}; skipping the mapping test");
        return;
    }
    let client = AppleClient::new(settings(&url)).unwrap();
    assert_eq!(
        client
            .revoke("local-apple-refresh-token", "com.example.ios")
            .await,
        RevokeOutcome::Revoked
    );
}
