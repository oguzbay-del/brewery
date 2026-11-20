//! Subscription state endpoint and the read-only guard (T2.28). Needs PostgreSQL.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::sync::Arc;

use api::{error::ErrorCode, redis_client::Redis, subscription::require_paid_write};
use axum::http::StatusCode;
use db::{ActorKind, DbContext};
use sqlx::PgPool;
use support::{FakeAuth, config, insert_user, insert_workspace, req, send, state_with};
use uuid::Uuid;

async fn app(pool: &PgPool, auth: Arc<FakeAuth>) -> axum::Router {
    let state = state_with(pool, Redis::disabled(), auth, config("postgres://unused")).await;
    api::build(state, axum::Router::new())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn state_endpoint_shows_trial_then_read_only_then_subscription(pool: PgPool) {
    let (ws, other_ws, user) = (Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7());
    insert_workspace(&pool, ws).await;
    insert_workspace(&pool, other_ws).await;
    insert_user(&pool, user).await;
    let auth = Arc::new(FakeAuth::default());
    auth.add(user, ws, &[]);
    let app = app(&pool, auth).await;
    let path = format!("/v1/workspaces/{ws}/subscription");

    let r = send(
        &app,
        req("GET", &path, Some(user))
            .body(axum::body::Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body["access"], "full");
    assert_eq!(r.body["access_source"], "trial");
    assert_eq!(r.body["subscriptions"].as_array().unwrap().len(), 0);

    sqlx::query("UPDATE workspaces SET created_at = now() - interval '40 days' WHERE id = $1")
        .bind(ws)
        .execute(&pool)
        .await
        .unwrap();
    let r = send(
        &app,
        req("GET", &path, Some(user))
            .body(axum::body::Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(r.body["access"], "read_only");
    assert_eq!(r.body["access_source"], "none");

    let sub = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO subscriptions (id, workspace_id, source, external_id, product_key, status,
                                    period_ends_at)
         VALUES ($1, $2, 'app_store', 'otx-1', 'pro.monthly', 'active', now() + interval '20 days')",
    )
    .bind(sub)
    .bind(ws)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO entitlements (id, workspace_id, feature, source, subscription_id, valid_until)
         VALUES ($1, $2, 'paid_features', 'subscription', $3, now() + interval '20 days')",
    )
    .bind(Uuid::now_v7())
    .bind(ws)
    .bind(sub)
    .execute(&pool)
    .await
    .unwrap();
    let r = send(
        &app,
        req("GET", &path, Some(user))
            .body(axum::body::Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(r.body["access"], "full");
    assert_eq!(r.body["access_source"], "subscription");
    assert_eq!(r.body["subscriptions"][0]["status"], "active");

    // Another workspace is not visible.
    let other = format!("/v1/workspaces/{other_ws}/subscription");
    let r = send(
        &app,
        req("GET", &other, Some(user))
            .body(axum::body::Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    // No sign-in.
    let r = send(
        &app,
        req("GET", &path, None)
            .body(axum::body::Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn guard_blocks_paid_writes_after_the_trial(pool: PgPool) {
    let (ws, user) = (Uuid::now_v7(), Uuid::now_v7());
    insert_workspace(&pool, ws).await;
    insert_user(&pool, user).await;
    let api = support::app_pool(&pool).await;
    let ctx = DbContext {
        user_id: Some(user),
        workspace_id: Some(ws),
        actor_kind: ActorKind::User,
    };

    let mut tx = db::begin(&api, &ctx).await.unwrap();
    require_paid_write(&mut tx, ws)
        .await
        .expect("trial allows writes");
    drop(tx);

    sqlx::query("UPDATE workspaces SET created_at = now() - interval '2 months' WHERE id = $1")
        .bind(ws)
        .execute(&pool)
        .await
        .unwrap();
    let mut tx = db::begin(&api, &ctx).await.unwrap();
    let err = require_paid_write(&mut tx, ws).await.unwrap_err();
    assert_eq!(err.code(), ErrorCode::EntitlementRequired);
}
