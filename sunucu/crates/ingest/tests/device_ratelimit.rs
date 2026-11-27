//! Rate limits of the device endpoint (T2.54). Needs PostgreSQL (`DATABASE_URL`); the Redis test
//! needs Redis (`REDIS_URL`, default `redis://127.0.0.1:6379`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use axum::{
    Router,
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use common::redis_client::Redis;
use http_body_util::BodyExt;
use ingest::{
    IngestState,
    jws::JwsVerifier,
    ratelimit::{CounterStore, DeviceLimiter, IngestRateConfig},
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;
use uuid::Uuid;

const ISPINDEL: &str = r#"{"name":"iSpindel000","ID":1234,"token":"unused","angle":40.1,
    "temperature":20.5,"temp_units":"C","battery":4.05,"gravity":1.0123,"interval":900,"RSSI":-70}"#;

type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

/// In-memory counters that remember every key name.
#[derive(Default)]
struct Memory {
    counts: Mutex<HashMap<String, i64>>,
    down: bool,
}

impl Memory {
    fn keys(&self) -> Vec<String> {
        self.counts.lock().unwrap().keys().cloned().collect()
    }
}

impl CounterStore for Memory {
    fn incr(&self, key: String, _ttl: u64) -> BoxFuture<'_, Option<i64>> {
        Box::pin(async move {
            if self.down {
                return None;
            }
            let mut m = self.counts.lock().unwrap();
            let c = m.entry(key).or_insert(0);
            *c += 1;
            Some(*c)
        })
    }

    fn get(&self, key: String) -> BoxFuture<'_, Option<i64>> {
        Box::pin(async move {
            if self.down {
                return None;
            }
            Some(self.counts.lock().unwrap().get(&key).copied().unwrap_or(0))
        })
    }
}

struct World {
    app: Router,
    base: PgPool,
    ws: Uuid,
}

async fn api_pool(base: &PgPool) -> PgPool {
    PgPoolOptions::new()
        .max_connections(4)
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("SET ROLE app_api").execute(conn).await?;
                Ok(())
            })
        })
        .connect_with((*base.connect_options()).clone())
        .await
        .unwrap()
}

async fn world(base: &PgPool, limiter: DeviceLimiter) -> World {
    let ws = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO workspaces (id, name, mode, region, country, time_zone, support_pseudonym)
         VALUES ($1, 'W', 'business', 'us', 'US', 'America/New_York', $2)",
    )
    .bind(ws)
    .bind(format!("pseudonym-{ws}"))
    .execute(base)
    .await
    .unwrap();
    let app = ingest::router_with_limiter(
        IngestState {
            pool: api_pool(base).await,
            apple: JwsVerifier::apple(),
            bundle_id: None,
            apple_environment: "Production".to_owned(),
            apple_product_ids: Vec::new(),
            apple_product_plans: std::collections::HashMap::new(),
            web_payment: None,
            github_secret: None,
        },
        limiter,
    );
    World {
        app,
        base: base.clone(),
        ws,
    }
}

impl World {
    async fn device(&self) -> String {
        let key = devices::new_key();
        sqlx::query(
            "INSERT INTO devices (id, workspace_id, kind, name, ingest_key_hash)
             VALUES ($1, $2, 'ispindel', 'D', $3)",
        )
        .bind(Uuid::now_v7())
        .bind(self.ws)
        .bind(devices::key_hash(&key))
        .execute(&self.base)
        .await
        .unwrap();
        key
    }

    async fn post(&self, key: &str, ip: &str) -> (StatusCode, Option<String>, String) {
        let mut request = Request::post(format!("/v1/ingest/{key}"))
            .body(Body::from(ISPINDEL))
            .unwrap();
        let peer: SocketAddr = format!("{ip}:4000").parse().unwrap();
        request.extensions_mut().insert(ConnectInfo(peer));
        let response = self.app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let retry = response
            .headers()
            .get("retry-after")
            .map(|v| v.to_str().unwrap().to_owned());
        let body = response.into_body().collect().await.unwrap().to_bytes();
        (status, retry, String::from_utf8_lossy(&body).into_owned())
    }
}

fn config(device: u32, invalid: u32) -> IngestRateConfig {
    IngestRateConfig {
        device_per_minute: device,
        invalid_per_minute: invalid,
        trust_forwarded_for: false,
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn device_limit_window_and_separate_counters(base: PgPool) {
    let store = Arc::new(Memory::default());
    let clock = Arc::new(AtomicU64::new(1_800_000_010));
    let tick = clock.clone();
    let limiter = DeviceLimiter::with_store_and_clock(
        store.clone(),
        config(3, 20),
        Arc::new(move || tick.load(Ordering::Relaxed)),
    );
    let w = world(&base, limiter).await;
    let (a, b) = (w.device().await, w.device().await);

    // (a) under the limit.
    for _ in 0..3 {
        assert_eq!(w.post(&a, "10.0.0.1").await.0, StatusCode::OK);
    }
    // (b) over the limit: 429 with Retry-After and a problem body.
    let (status, retry, body) = w.post(&a, "10.0.0.1").await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(retry.as_deref(), Some("50"));
    let problem: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(problem["code"], "rate_limited");
    assert_eq!(problem["status"], 429);
    // (d) another key has its own counter.
    assert_eq!(w.post(&b, "10.0.0.1").await.0, StatusCode::OK);
    // (c) the next window starts again.
    clock.fetch_add(60, Ordering::Relaxed);
    assert_eq!(w.post(&a, "10.0.0.1").await.0, StatusCode::OK);
    // (g) no Redis key name holds the key or a part of it.
    for name in store.keys() {
        for key in [&a, &b] {
            assert!(!name.contains(key.as_str()), "{name}");
            assert!(!name.contains(&key[..8]), "{name}");
        }
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn invalid_keys_over_the_ip_limit_get_429(base: PgPool) {
    let store = Arc::new(Memory::default());
    let limiter = DeviceLimiter::with_store(store, config(12, 2));
    let w = world(&base, limiter).await;
    let valid = w.device().await;
    for _ in 0..2 {
        assert_eq!(
            w.post(&devices::new_key(), "10.0.0.9").await.0,
            StatusCode::UNAUTHORIZED
        );
    }
    let (status, retry, _) = w.post(&devices::new_key(), "10.0.0.9").await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert!(retry.is_some());
    // The IP stays blocked in this window, even with a valid key.
    assert_eq!(
        w.post(&valid, "10.0.0.9").await.0,
        StatusCode::TOO_MANY_REQUESTS
    );
    // Another IP is not affected.
    assert_eq!(w.post(&valid, "10.0.0.10").await.0, StatusCode::OK);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn redis_down_does_not_block_a_valid_request(base: PgPool) {
    let store = Arc::new(Memory {
        down: true,
        ..Memory::default()
    });
    let limiter = DeviceLimiter::with_store(store, config(1, 1));
    let w = world(&base, limiter).await;
    let key = w.device().await;
    for _ in 0..3 {
        assert_eq!(w.post(&key, "10.0.0.1").await.0, StatusCode::OK);
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn limits_work_over_real_redis(base: PgPool) {
    let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_owned());
    let redis = Redis::connect_lazy(&url).unwrap();
    let limiter = DeviceLimiter::redis(redis, config(2, 20));
    let w = world(&base, limiter).await;
    let key = w.device().await;
    assert_eq!(w.post(&key, "10.0.0.1").await.0, StatusCode::OK);
    assert_eq!(w.post(&key, "10.0.0.1").await.0, StatusCode::OK);
    let (status, retry, _) = w.post(&key, "10.0.0.1").await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    let retry: u64 = retry.unwrap().parse().unwrap();
    assert!((1..=60).contains(&retry));
}
