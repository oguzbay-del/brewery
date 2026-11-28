//! Helpers of the recipe and batch endpoint tests (T2.21, T2.22): a Home workspace, two users,
//! the real engine service in process, and request shortcuts.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(dead_code)] // each test binary uses a subset

use std::sync::Arc;

use api::{
    engine::{EngineClient, EngineConfig},
    redis_client::Redis,
};
use axum::{Router, body::Body, http::header};
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::{net::TcpListener, sync::oneshot};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use uuid::Uuid;

use crate::support::{
    FakeAuth, Reply, config, insert_user, insert_workspace, req, send, state_with,
};

pub const ALL: &[&str] = &[
    "recipe.view",
    "recipe.edit",
    "recipe.approve",
    "batch.plan",
    "batch.record",
    "settings.equipment",
];

pub struct World {
    pub app: Router,
    pub auth: Arc<FakeAuth>,
    pub admin: Uuid,
    pub other: Uuid,
    pub ws: Uuid,
    pub pool: PgPool,
    pub stop: Option<oneshot::Sender<()>>,
}

async fn start_engine() -> (String, oneshot::Sender<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (stop, rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        Server::builder()
            .add_service(engine_service::service::server())
            .add_service(engine_service::recipe::server())
            .add_service(engine_service::tool::server())
            .add_service(engine_service::mash::server())
            .add_service(engine_service::water::server())
            .add_service(engine_service::yeast::server())
            .add_service(engine_service::fermentation::server())
            .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                let _ = rx.await;
            })
            .await
    });
    (url, stop)
}

pub async fn world(pool: &PgPool) -> World {
    init_log();
    let admin = Uuid::now_v7();
    let other = Uuid::now_v7();
    let ws = Uuid::now_v7();
    insert_user(pool, admin).await;
    insert_user(pool, other).await;
    insert_workspace(pool, ws).await;
    sqlx::query("UPDATE workspaces SET mode = 'home' WHERE id = $1")
        .bind(ws)
        .execute(pool)
        .await
        .unwrap();
    let auth = Arc::new(FakeAuth::default());
    auth.add(admin, ws, ALL);
    auth.add(other, ws, ALL);
    let (url, stop) = start_engine().await;
    let mut cfg = EngineConfig::new(url);
    cfg.version_ttl = std::time::Duration::ZERO;
    let engine = EngineClient::connect_lazy(cfg, Redis::disabled()).unwrap();
    let state = state_with(
        pool,
        Redis::disabled(),
        auth.clone(),
        config("postgres://unused"),
    )
    .await
    .with_engine(engine);
    World {
        app: api::router(state),
        auth,
        admin,
        other,
        ws,
        pool: pool.clone(),
        stop: Some(stop),
    }
}

pub async fn call_as(
    w: &World,
    user: Uuid,
    method: &str,
    path: &str,
    body: Option<Value>,
    extra: &[(&str, &str)],
) -> Reply {
    let mut b = req(
        method,
        &format!("/v1/workspaces/{}{path}", w.ws),
        Some(user),
    );
    for (k, v) in extra {
        b = b.header(*k, *v);
    }
    let request = match body {
        Some(v) => {
            let ct = if method == "PATCH" {
                "application/merge-patch+json"
            } else {
                "application/json"
            };
            b.header(header::CONTENT_TYPE, ct)
                .body(Body::from(v.to_string()))
                .unwrap()
        }
        None => b.body(Body::empty()).unwrap(),
    };
    send(&w.app, request).await
}

pub async fn post(w: &World, path: &str, body: Value) -> Reply {
    let key = Uuid::now_v7().to_string();
    call_as(
        w,
        w.admin,
        "POST",
        path,
        Some(body),
        &[("idempotency-key", &key)],
    )
    .await
}

pub async fn post_with(w: &World, path: &str, body: Value, extra: &[(&str, &str)]) -> Reply {
    let key = Uuid::now_v7().to_string();
    let mut headers = vec![("idempotency-key", key.as_str())];
    headers.extend_from_slice(extra);
    call_as(w, w.admin, "POST", path, Some(body), &headers).await
}

pub async fn patch(w: &World, path: &str, body: Value, etag: &str) -> Reply {
    call_as(w, w.admin, "PATCH", path, Some(body), &[("if-match", etag)]).await
}

pub async fn get(w: &World, path: &str) -> Reply {
    call_as(w, w.admin, "GET", path, None, &[]).await
}

pub async fn delete(w: &World, path: &str) -> Reply {
    call_as(w, w.admin, "DELETE", path, None, &[]).await
}

/// A simple all-grain recipe body.
pub fn ipa() -> Value {
    json!({
        "name": "House IPA",
        "version": {
            "kind": "all_grain", "batch_volume_l": 20.0, "boil_volume_l": 25.0,
            "boil_time_min": 60.0, "efficiency_pct": 72.0,
            "fermentables": [
                {"name": "Pale malt", "amount_kg": 4.5, "grain_type": "grain", "yield_pct": 80.0, "color_srm": 2.0},
                {"name": "Crystal 60", "amount_kg": 0.4, "grain_type": "grain", "yield_pct": 74.0, "color_srm": 60.0}
            ],
            "hops": [
                {"name": "Cascade", "amount_kg": 0.03, "alpha_pct": 5.5, "usage": "boil", "time_min": 60.0},
                {"name": "Citra", "amount_kg": 0.05, "alpha_pct": 12.0, "usage": "boil", "time_min": 5.0}
            ],
            "yeasts": [
                {"name": "US-05", "form": "dry", "min_attenuation_pct": 73.0, "max_attenuation_pct": 80.0,
                 "total_cells_billion": 200.0, "packs": 1.0}
            ]
        }
    })
}

/// Creates the recipe and publishes the draft; returns (recipe id, published version id).
pub async fn published_recipe(w: &World, body: Value) -> (String, String) {
    let r = post(w, "/recipes", body).await;
    assert_eq!(r.status, axum::http::StatusCode::CREATED, "{}", r.body);
    let recipe = r.body["id"].as_str().unwrap().to_owned();
    let version = r.body["draft_version_id"].as_str().unwrap().to_owned();
    let v = get(w, &format!("/recipe-versions/{version}")).await;
    let etag = v.header("etag").to_owned();
    let p = post_with(
        w,
        &format!("/recipe-versions/{version}/publish"),
        json!({}),
        &[("if-match", &etag)],
    )
    .await;
    assert_eq!(p.status, axum::http::StatusCode::OK, "{}", p.body);
    (recipe, version)
}

/// Prints the logs of the API when `TEST_LOG` is set.
pub fn init_log() {
    if std::env::var("TEST_LOG").is_ok() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("error")
            .try_init();
    }
}
