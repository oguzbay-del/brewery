//! End-to-end tests for account deletion requests (T2.31b).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::{collections::HashMap, sync::Arc};

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
use chrono::{Duration, Utc};
use fake_oidc::{MintSpec, Provider, Running};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use support::{Reply, app_pool, send};
use uuid::Uuid;

const APPLE_AUD: &str = "com.example.brewery";

struct World {
    app: Router,
    oidc: Running,
}

async fn world(pool: &PgPool) -> World {
    let oidc = fake_oidc::spawn_local().await;
    let map: HashMap<&str, String> = HashMap::from([
        ("DATABASE_URL", "postgres://unused".to_owned()),
        ("TRUST_FORWARDED_FOR", "true".to_owned()),
        ("APPLE_CLIENT_IDS", APPLE_AUD.to_owned()),
        ("APPLE_ISSUER", oidc.oidc.issuer(Provider::Apple)),
        ("APPLE_JWKS_URL", oidc.oidc.jwks_url(Provider::Apple)),
        ("WEB_BASE_URL", "https://app.example.com".to_owned()),
        ("CSRF_HMAC_KEY", "test-csrf-key-0123456789".to_owned()),
        ("JWKS_MIN_REFRESH_SECS", "0".to_owned()),
    ]);
    let cfg = ApiConfig::from_lookup(|n| map.get(n).cloned()).unwrap();
    let app_db = app_pool(pool).await;
    let svc = Arc::new(AuthServices::new(
        app_db.clone(),
        Redis::disabled(),
        cfg.auth.clone(),
        Arc::new(MemoryMailer::default()),
    ));
    let state = AppState::new(app_db, Redis::disabled(), svc.clone(), cfg).with_auth_services(svc);
    World {
        app: api::router(state),
        oidc,
    }
}

async fn call(
    w: &World,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> Reply {
    let mut b = Request::builder()
        .method(method)
        .uri(path)
        .header("x-forwarded-for", Uuid::new_v4().to_string());
    if let Some(token) = token {
        b = b.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    if method == "POST" {
        b = b.header("idempotency-key", Uuid::new_v4().to_string());
    }
    let req = match body {
        Some(value) => b
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(value.to_string()))
            .unwrap(),
        None => b.body(Body::empty()).unwrap(),
    };
    send(&w.app, req).await
}

async fn as_user(w: &World, token: &str, method: &str, path: &str) -> Reply {
    call(w, method, path, Some(token), None).await
}

/// New Apple user; the sign-in itself is a valid second step for a Home user for ten minutes.
async fn sign_in(w: &World, subject: &str) -> String {
    let nonce = call(w, "POST", "/v1/auth/nonce", None, None).await.body["nonce"]
        .as_str()
        .unwrap()
        .to_owned();
    let id_token = w.oidc.oidc.mint(
        Provider::Apple,
        &MintSpec {
            sub: subject.to_owned(),
            aud: APPLE_AUD.to_owned(),
            nonce: Some(hex(&Sha256::digest(nonce.as_bytes()))),
            ..MintSpec::default()
        },
    );
    let reply = call(
        w,
        "POST",
        "/v1/auth/apple",
        None,
        Some(json!({
            "id_token": id_token,
            "nonce": nonce,
            "client": "ios",
            "display_name": subject
        })),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.body);
    reply.body["session"]["token"].as_str().unwrap().to_owned()
}

async fn user_id(pool: &PgPool, subject: &str) -> Uuid {
    sqlx::query_scalar(
        "SELECT user_id FROM user_identities WHERE provider = 'apple' AND subject = $1",
    )
    .bind(subject)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn new_business_workspace(w: &World, token: &str) -> Uuid {
    let reply = call(
        w,
        "POST",
        "/v1/workspaces",
        Some(token),
        Some(json!({
            "name": "Brewery",
            "mode": "business",
            "region": "us",
            "country": "US",
            "time_zone": "America/New_York"
        })),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{:?}", reply.body);
    reply.body["id"].as_str().unwrap().parse().unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn request_returns_purge_date_and_revokes_every_session(pool: PgPool) {
    let w = world(&pool).await;
    let subject = "deletion-request";
    let token = sign_in(&w, subject).await;
    let user = user_id(&pool, subject).await;

    let reply = as_user(&w, &token, "POST", "/v1/me/deletion").await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.body);
    let purge_at = chrono::DateTime::parse_from_rfc3339(reply.body["purge_at"].as_str().unwrap())
        .unwrap()
        .with_timezone(&Utc);
    assert!(purge_at >= Utc::now() + Duration::days(29));

    let active_sessions: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM sessions WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(user)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(active_sessions, 0);
    assert_eq!(
        as_user(&w, &token, "GET", "/v1/me/deletion").await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn cancellation_works_during_grace_and_fails_after_it(pool: PgPool) {
    let w = world(&pool).await;
    let subject = "deletion-cancel";
    let token = sign_in(&w, subject).await;
    let user = user_id(&pool, subject).await;

    let requested = as_user(&w, &token, "POST", "/v1/me/deletion").await;
    assert_eq!(requested.status, StatusCode::OK, "{:?}", requested.body);

    let token = sign_in(&w, subject).await;
    let cancelled = as_user(&w, &token, "DELETE", "/v1/me/deletion").await;
    assert_eq!(cancelled.status, StatusCode::OK, "{:?}", cancelled.body);
    assert_eq!(cancelled.body["status"], "cancelled");

    let token = sign_in(&w, subject).await;
    let requested = as_user(&w, &token, "POST", "/v1/me/deletion").await;
    assert_eq!(requested.status, StatusCode::OK, "{:?}", requested.body);
    sqlx::query(
        "UPDATE account_deletion_requests SET purge_at = now() - interval '1 second'
         WHERE user_id = $1 AND status = 'pending'",
    )
    .bind(user)
    .execute(&pool)
    .await
    .unwrap();

    let token = sign_in(&w, subject).await;
    let too_late = as_user(&w, &token, "DELETE", "/v1/me/deletion").await;
    assert_eq!(too_late.status, StatusCode::CONFLICT, "{:?}", too_late.body);
    assert_eq!(too_late.body["errors"][0]["code"], "grace_period_ended");
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_business_administrator_gets_last_admin_conflict(pool: PgPool) {
    let w = world(&pool).await;
    let token = sign_in(&w, "business-last-admin").await;
    let _workspace = new_business_workspace(&w, &token).await;

    let reply = as_user(&w, &token, "POST", "/v1/me/deletion").await;
    assert_eq!(reply.status, StatusCode::CONFLICT, "{:?}", reply.body);
    assert_eq!(reply.body["errors"][0]["code"], "last_admin");
}

#[sqlx::test(migrations = "../../migrations")]
async fn request_without_recent_second_step_is_forbidden(pool: PgPool) {
    let w = world(&pool).await;
    let subject = "deletion-no-step-up";
    let token = sign_in(&w, subject).await;
    let user = user_id(&pool, subject).await;
    sqlx::query(
        "UPDATE sessions SET created_at = now() - interval '11 minutes' WHERE user_id = $1",
    )
    .bind(user)
    .execute(&pool)
    .await
    .unwrap();

    let reply = as_user(&w, &token, "POST", "/v1/me/deletion").await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN, "{:?}", reply.body);
    assert_eq!(reply.body["code"], "step_up_required");
}
