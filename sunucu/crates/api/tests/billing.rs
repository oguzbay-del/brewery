//! Billing endpoints (B3): `GET /v1/billing`, checkout and portal against a fake Paddle API.
//! Needs PostgreSQL.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::sync::{Arc, Mutex};

use api::{billing_api::PaddleApi, redis_client::Redis};
use axum::{Json, Router, body::Body, extract::Path, http::StatusCode, routing::post};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{FakeAuth, config, insert_user, insert_workspace, json_post, req, send, state_with};
use uuid::Uuid;

type Seen = Arc<Mutex<Vec<(String, Value)>>>;

/// A fake Paddle API in the test process. Returns its base address and the requests it saw.
async fn fake_paddle() -> (String, Seen) {
    let seen: Seen = Arc::default();
    let log = seen.clone();
    let log2 = seen.clone();
    let router = Router::new()
        .route(
            "/transactions",
            post(move |Json(body): Json<Value>| async move {
                log.lock().unwrap().push(("transactions".to_owned(), body));
                Json(json!({"data": {"checkout": {"url": "https://checkout.example/txn_1"}}}))
            }),
        )
        .route(
            "/customers/{id}/portal-sessions",
            post(move |Path(id): Path<String>, Json(body): Json<Value>| async move {
                log2.lock().unwrap().push((format!("portal:{id}"), body));
                Json(json!({"data": {"urls": {"general": {"overview": "https://portal.example/s1"}}}}))
            }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    (base, seen)
}

struct World {
    pool: PgPool,
    app: Router,
    ws: Uuid,
    user: Uuid,
    seen: Seen,
}

async fn world(pool: PgPool, configured: bool, permissions: &[&str]) -> World {
    let (ws, user) = (Uuid::now_v7(), Uuid::now_v7());
    insert_workspace(&pool, ws).await;
    insert_user(&pool, user).await;
    let auth = Arc::new(FakeAuth::default());
    auth.add(user, ws, permissions);
    let mut state = state_with(&pool, Redis::disabled(), auth, config("postgres://unused")).await;
    let (base, seen) = fake_paddle().await;
    if configured {
        state = state.with_paddle(PaddleApi::new(&base, "key", "pri_home", "pri_business"));
    }
    World {
        app: api::build(state, Router::new()),
        pool,
        ws,
        user,
        seen,
    }
}

async fn add_subscription(w: &World, source: &str, customer: Option<&str>) {
    sqlx::query(
        "INSERT INTO subscriptions (id, workspace_id, source, external_id, product_key, status,
                                    period_ends_at, paddle_customer_id)
         VALUES ($1, $2, $3, $4, 'home', 'active', now() + interval '20 days', $5)",
    )
    .bind(Uuid::now_v7())
    .bind(w.ws)
    .bind(source)
    .bind(format!("ext-{}", Uuid::now_v7()))
    .bind(customer)
    .execute(&w.pool)
    .await
    .unwrap();
}

async fn post_json(w: &World, path: &str, body: &Value) -> support::Reply {
    send(&w.app, json_post(path, w.user, None, body)).await
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn get_shows_state_and_live_source(pool: PgPool) {
    let w = world(pool, true, &[]).await;
    let path = format!("/v1/billing?workspace_id={}", w.ws);
    let get = || async {
        send(
            &w.app,
            req("GET", &path, Some(w.user)).body(Body::empty()).unwrap(),
        )
        .await
    };
    let r = get().await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body["subscription"]["access_source"], "trial");
    assert_eq!(r.body["active_source"], Value::Null);
    assert_eq!(r.body["web_billing_available"], true);

    add_subscription(&w, "web", Some("ctm_1")).await;
    assert_eq!(get().await.body["active_source"], "web");

    // A workspace of another account is not visible; no sign-in is refused.
    let other = format!("/v1/billing?workspace_id={}", Uuid::now_v7());
    let r = send(
        &w.app,
        req("GET", &other, Some(w.user))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    let r = send(&w.app, req("GET", &path, None).body(Body::empty()).unwrap()).await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn checkout_returns_the_provider_address(pool: PgPool) {
    let w = world(pool, true, &["billing.manage"]).await;
    let body = json!({"workspace_id": w.ws, "plan": "business", "seats": 4});
    let r = post_json(&w, "/v1/billing/checkout", &body).await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body["checkout_url"], "https://checkout.example/txn_1");
    let seen = w.seen.lock().unwrap();
    let (kind, sent) = &seen[0];
    assert_eq!(kind, "transactions");
    assert_eq!(sent["items"][0]["price_id"], "pri_business");
    assert_eq!(sent["items"][0]["quantity"], 4);
    assert_eq!(sent["custom_data"]["workspace_id"], w.ws.to_string());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn checkout_rules(pool: PgPool) {
    let w = world(pool, true, &["billing.manage"]).await;
    let ok = json!({"workspace_id": w.ws, "plan": "home"});
    let code = |r: &support::Reply| {
        (
            r.status.as_u16(),
            r.body["code"].as_str().unwrap_or_default().to_owned(),
            r.body["errors"][0]["code"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
        )
    };

    let bad_plan = post_json(
        &w,
        "/v1/billing/checkout",
        &json!({"workspace_id": w.ws, "plan": "gold"}),
    )
    .await;
    assert_eq!(code(&bad_plan).0, 422);
    let bad_seats = post_json(
        &w,
        "/v1/billing/checkout",
        &json!({"workspace_id": w.ws, "plan": "home", "seats": 0}),
    )
    .await;
    assert_eq!(code(&bad_seats).0, 422);

    // iOS clients buy with StoreKit.
    let mut ios = json_post("/v1/billing/checkout", w.user, None, &ok);
    ios.headers_mut()
        .insert("x-app-version", "ios/1.0.0+1".parse().unwrap());
    let r = send(&w.app, ios).await;
    assert_eq!(
        code(&r),
        (409, "conflict".to_owned(), "web_only".to_owned())
    );

    // The other source is live.
    add_subscription(&w, "app_store", None).await;
    let r = post_json(&w, "/v1/billing/checkout", &ok).await;
    assert_eq!(
        code(&r),
        (409, "conflict".to_owned(), "other_source_active".to_owned())
    );
    assert!(w.seen.lock().unwrap().is_empty(), "no call to the provider");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn checkout_and_portal_need_permission_and_configuration(pool: PgPool) {
    // No permission.
    let w = world(pool.clone(), true, &[]).await;
    let ok = json!({"workspace_id": w.ws, "plan": "home"});
    assert_eq!(
        post_json(&w, "/v1/billing/checkout", &ok).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        post_json(&w, "/v1/billing/portal", &json!({"workspace_id": w.ws}))
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    // No Paddle settings.
    let w = world(pool, false, &["billing.manage"]).await;
    let ok = json!({"workspace_id": w.ws, "plan": "home"});
    let r = post_json(&w, "/v1/billing/checkout", &ok).await;
    assert_eq!(
        (r.status, r.body["code"].as_str()),
        (StatusCode::SERVICE_UNAVAILABLE, Some("billing_unavailable"))
    );
    let r = post_json(&w, "/v1/billing/portal", &json!({"workspace_id": w.ws})).await;
    assert_eq!(r.body["code"], "billing_unavailable");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn portal_uses_the_customer_of_the_web_subscription(pool: PgPool) {
    let w = world(pool, true, &["billing.manage"]).await;
    let body = json!({"workspace_id": w.ws});

    // No web subscription yet.
    let r = post_json(&w, "/v1/billing/portal", &body).await;
    assert_eq!(r.status, StatusCode::CONFLICT);
    assert_eq!(r.body["errors"][0]["code"], "no_web_subscription");

    add_subscription(&w, "web", Some("ctm_77")).await;
    let r = post_json(&w, "/v1/billing/portal", &body).await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body["portal_url"], "https://portal.example/s1");
    assert_eq!(w.seen.lock().unwrap()[0].0, "portal:ctm_77");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn portal_refuses_when_the_app_store_is_live(pool: PgPool) {
    let w = world(pool, true, &["billing.manage"]).await;
    add_subscription(&w, "app_store", None).await;
    let r = post_json(&w, "/v1/billing/portal", &json!({"workspace_id": w.ws})).await;
    assert_eq!(r.status, StatusCode::CONFLICT);
    assert_eq!(r.body["errors"][0]["code"], "other_source_active");
}
