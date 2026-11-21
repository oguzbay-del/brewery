//! Endpoint tests of T2.31: the account deletion request (second step, sessions and push
//! targets closed, restore, request again), the block for the only administrator of a Business
//! workspace, the Apple code exchange at sign-in, and the account data export job. The purge
//! itself is tested in `crates/worker/tests/account.rs`. Needs PostgreSQL (`#[sqlx::test]`).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

mod support;

use std::{collections::HashMap, sync::Arc};

use account::{Envelope, FakeApple};
use api::{
    ApiConfig, AppState,
    auth::{AuthServices, email::MemoryMailer, tokens::hex},
    redis_client::Redis,
};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use fake_oidc::{MintSpec, Provider, Running};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use support::{Reply, app_pool, send};
use uuid::Uuid;

const APPLE_AUD: &str = "com.example.brewery";
const ORIGIN: &str = "https://app.example.com";

struct World {
    app: Router,
    oidc: Running,
    apple: Arc<FakeApple>,
    envelope_key: [u8; 32],
}

async fn world(pool: &PgPool) -> World {
    let oidc = fake_oidc::spawn_local().await;
    let map: HashMap<&str, String> = HashMap::from([
        ("DATABASE_URL", "postgres://unused".to_owned()),
        ("TRUST_FORWARDED_FOR", "true".to_owned()),
        ("APPLE_CLIENT_IDS", APPLE_AUD.to_owned()),
        ("APPLE_ISSUER", oidc.oidc.issuer(Provider::Apple)),
        ("APPLE_JWKS_URL", oidc.oidc.jwks_url(Provider::Apple)),
        ("WEB_BASE_URL", ORIGIN.to_owned()),
        ("CSRF_HMAC_KEY", "test-csrf-key-0123456789".to_owned()),
        ("JWKS_MIN_REFRESH_SECS", "0".to_owned()),
    ]);
    let cfg = ApiConfig::from_lookup(|n| map.get(n).cloned()).unwrap();
    let app_db = app_pool(pool).await;
    let apple = Arc::new(FakeApple::default());
    let envelope_key = [9u8; 32];
    let svc = Arc::new(
        AuthServices::new(
            app_db.clone(),
            Redis::disabled(),
            cfg.auth.clone(),
            Arc::new(MemoryMailer::default()),
        )
        .with_apple_exchange(apple.clone(), Envelope::from_bytes(&envelope_key).unwrap()),
    );
    let state =
        AppState::new(app_db, Redis::disabled(), svc.clone(), cfg).with_auth_services(svc.clone());
    World {
        app: api::router(state),
        oidc,
        apple,
        envelope_key,
    }
}

async fn call(
    w: &World,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<Value>,
) -> Reply {
    let mut b = Request::builder()
        .method(method)
        .uri(path)
        .header("x-forwarded-for", Uuid::new_v4().to_string());
    for (k, v) in headers {
        b = b.header(*k, *v);
    }
    let req = match body {
        Some(v) => b
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(v.to_string()))
            .unwrap(),
        None => b.body(Body::empty()).unwrap(),
    };
    send(&w.app, req).await
}

async fn as_user(w: &World, token: &str, method: &str, path: &str, body: Option<Value>) -> Reply {
    let auth = format!("Bearer {token}");
    let key = Uuid::new_v4().to_string();
    call(
        w,
        method,
        path,
        &[("authorization", &auth), ("idempotency-key", &key)],
        body,
    )
    .await
}

/// Apple sign-in on iOS; returns `(session token, user id)`.
async fn sign_in_with(w: &World, sub: &str, code: Option<&str>) -> (String, Uuid) {
    let n = call(w, "POST", "/v1/auth/nonce", &[], None).await.body["nonce"]
        .as_str()
        .unwrap()
        .to_owned();
    let token = w.oidc.oidc.mint(
        Provider::Apple,
        &MintSpec {
            sub: sub.to_owned(),
            aud: APPLE_AUD.to_owned(),
            nonce: Some(hex(&Sha256::digest(n.as_bytes()))),
            ..MintSpec::default()
        },
    );
    let mut body = json!({"id_token": token, "nonce": n, "client": "ios", "display_name": "Sam"});
    if let Some(code) = code {
        body["authorization_code"] = json!(code);
    }
    let r = call(w, "POST", "/v1/auth/apple", &[], Some(body)).await;
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
    (
        r.body["session"]["token"].as_str().unwrap().to_owned(),
        r.body["user"]["id"].as_str().unwrap().parse().unwrap(),
    )
}

async fn new_workspace(w: &World, token: &str, mode: &str) -> Uuid {
    let r = as_user(
        w,
        token,
        "POST",
        "/v1/workspaces",
        Some(json!({"name": "Brewery", "mode": mode, "region": "us",
                    "country": "US", "time_zone": "America/New_York"})),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{:?}", r.body);
    r.body["id"].as_str().unwrap().parse().unwrap()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn request_closes_sessions_and_restore_brings_the_account_back(pool: PgPool) {
    let w = world(&pool).await;
    let (token, user) = sign_in_with(&w, "sub-delete", None).await;
    let ws = new_workspace(&w, &token, "home").await;
    sqlx::query(
        "INSERT INTO push_targets (id, user_id, environment, token) VALUES ($1, $2, 'sandbox', 'tok')",
    )
    .bind(Uuid::now_v7())
    .bind(user)
    .execute(&pool)
    .await
    .unwrap();

    let none = as_user(&w, &token, "GET", "/v1/me/deletion", None).await;
    assert_eq!(none.body["state"], "none", "{:?}", none.body);

    let r = as_user(&w, &token, "POST", "/v1/me/deletion", None).await;
    assert_eq!(r.status, StatusCode::ACCEPTED, "{:?}", r.body);
    assert_eq!(r.body["state"], "pending");
    assert_eq!(r.body["retention_days"], db::account::RETENTION_DAYS);
    let purge_at: chrono::DateTime<chrono::Utc> =
        sqlx::query_scalar("SELECT purge_at FROM account_deletions WHERE user_id = $1")
            .bind(user)
            .fetch_one(&pool)
            .await
            .unwrap();
    let days = (purge_at - chrono::Utc::now()).num_days();
    assert!((29..=30).contains(&days), "purge in {days} days");

    // Sessions and push targets are gone at once.
    let old = as_user(&w, &token, "GET", "/v1/me/deletion", None).await;
    assert_eq!(old.status, StatusCode::UNAUTHORIZED);
    let pushes: i64 = sqlx::query_scalar("SELECT count(*) FROM push_targets WHERE user_id = $1")
        .bind(user)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(pushes, 0);
    // The request wrote an audit row that names the user by UUID only.
    let audit: Value = sqlx::query_scalar(
        "SELECT to_jsonb(a) FROM audit_log a WHERE workspace_id = $1 AND action LIKE 'account%' \
         ORDER BY seq LIMIT 1",
    )
    .bind(ws)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(audit["actor_id"], user.to_string());
    assert!(!audit.to_string().contains("Sam"));

    // The user can sign in again; the workspace stays closed until the restore.
    let (token2, user2) = sign_in_with(&w, "sub-delete", None).await;
    assert_eq!(user2, user);
    let status = as_user(&w, &token2, "GET", "/v1/me/deletion", None).await;
    assert_eq!(status.body["state"], "pending");
    let closed = as_user(
        &w,
        &token2,
        "GET",
        &format!("/v1/workspaces/{ws}/trash"),
        None,
    )
    .await;
    assert!(
        closed.status == StatusCode::FORBIDDEN || closed.status == StatusCode::NOT_FOUND,
        "{:?}",
        closed.body
    );
    // A repeat is not a new request.
    let again = as_user(&w, &token2, "POST", "/v1/me/deletion", None).await;
    assert_eq!(again.status, StatusCode::OK, "{:?}", again.body);
    let (token3, _) = sign_in_with(&w, "sub-delete", None).await;

    let restored = as_user(&w, &token3, "DELETE", "/v1/me/deletion", None).await;
    assert_eq!(restored.status, StatusCode::OK, "{:?}", restored.body);
    assert_eq!(restored.body["state"], "none");
    let nothing = as_user(&w, &token3, "DELETE", "/v1/me/deletion", None).await;
    assert_eq!(nothing.status, StatusCode::NOT_FOUND);
    let open = as_user(
        &w,
        &token3,
        "GET",
        &format!("/v1/workspaces/{ws}/trash"),
        None,
    )
    .await;
    assert_eq!(open.status, StatusCode::OK, "{:?}", open.body);

    // Request again after the restore: a new waiting time starts.
    let second = as_user(&w, &token3, "POST", "/v1/me/deletion", None).await;
    assert_eq!(second.status, StatusCode::ACCEPTED, "{:?}", second.body);
    let (restored_at,): (Option<chrono::DateTime<chrono::Utc>>,) =
        sqlx::query_as("SELECT restored_at FROM account_deletions WHERE user_id = $1")
            .bind(user)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(restored_at.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn the_only_administrator_of_a_business_workspace_cannot_delete(pool: PgPool) {
    let w = world(&pool).await;
    let (token, user) = sign_in_with(&w, "sub-business", None).await;
    new_workspace(&w, &token, "business").await;
    let r = as_user(&w, &token, "POST", "/v1/me/deletion", None).await;
    assert_eq!(r.status, StatusCode::CONFLICT, "{:?}", r.body);
    assert_eq!(r.body["errors"][0]["code"], "sole_business_admin");
    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM account_deletions WHERE user_id = $1")
        .bind(user)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 0);
    // The session stays open.
    let ok = as_user(&w, &token, "GET", "/v1/me/deletion", None).await;
    assert_eq!(ok.status, StatusCode::OK);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn the_apple_code_is_exchanged_and_stored_sealed(pool: PgPool) {
    let w = world(&pool).await;
    let (_, user) = sign_in_with(&w, "sub-apple", Some("code-1")).await;
    let blob: Vec<u8> = sqlx::query_scalar(
        "SELECT provider_refresh_token_enc FROM user_identities WHERE user_id = $1",
    )
    .bind(user)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!String::from_utf8_lossy(&blob).contains("fake-refresh"));
    let plain = Envelope::from_bytes(&w.envelope_key)
        .unwrap()
        .open(&blob)
        .unwrap();
    assert_eq!(plain, b"fake-refresh-code-1");
    assert!(w.apple.revoked_tokens().is_empty());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn data_export_queues_one_account_job_a_day(pool: PgPool) {
    let w = world(&pool).await;
    let (token, user) = sign_in_with(&w, "sub-export", None).await;
    let ws = new_workspace(&w, &token, "home").await;
    let r = as_user(&w, &token, "POST", "/v1/me/data-export", Some(json!({}))).await;
    assert_eq!(r.status, StatusCode::ACCEPTED, "{:?}", r.body);
    assert_eq!(r.body["status"], "queued");
    let id = r.body["id"].as_str().unwrap();
    let (scope, workspace, by): (String, Uuid, Uuid) =
        sqlx::query_as("SELECT scope, workspace_id, created_by FROM export_jobs WHERE id = $1")
            .bind(id.parse::<Uuid>().unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((scope.as_str(), workspace, by), ("account_data", ws, user));
    let again = as_user(&w, &token, "POST", "/v1/me/data-export", Some(json!({}))).await;
    assert_eq!(
        again.status,
        StatusCode::TOO_MANY_REQUESTS,
        "{:?}",
        again.body
    );
    let read = as_user(
        &w,
        &token,
        "GET",
        &format!("/v1/me/data-export/{ws}/{id}"),
        None,
    )
    .await;
    assert_eq!(read.status, StatusCode::OK, "{:?}", read.body);
    assert_eq!(read.body["status"], "queued");
    let other = as_user(
        &w,
        &token,
        "POST",
        "/v1/me/data-export",
        Some(json!({"workspace_id": Uuid::now_v7()})),
    )
    .await;
    assert_eq!(
        other.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{:?}",
        other.body
    );
}
