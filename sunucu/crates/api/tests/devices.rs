//! Device intake end to end (T2.25): register and pair a device with the API, send an iSpindel
//! request to `ingest`, the worker consumer writes the measurement, and the SSE stream shows it.
//! A key rotation makes the old address stop working. Needs PostgreSQL, Redis-free, and NATS
//! (`NATS_URL`, compose default `nats://127.0.0.1:4222`); the engine runs in process.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

mod domain;
mod support;

use std::{sync::Arc, time::Duration};

use async_nats::jetstream;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use domain::{ALL, World, call_as, get, ipa, post, published_recipe, world};
use events::{consumer, relay};
use http_body_util::BodyExt;
use ingest::IngestState;
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::sync::watch;
use tower::ServiceExt;
use uuid::Uuid;

fn nats_url() -> String {
    std::env::var("NATS_URL").unwrap_or_else(|_| "nats://127.0.0.1:4222".to_owned())
}

struct Stack {
    ingest: Router,
    sse: Router,
    worker: PgPool,
    js: jetstream::Context,
    ingest_pool: PgPool,
    stop: watch::Sender<bool>,
    task: tokio::task::JoinHandle<()>,
}

async fn stack(w: &World) -> Stack {
    let mut perms: Vec<&str> = ALL.to_vec();
    perms.push("devices.manage");
    w.auth.add(w.admin, w.ws, &perms);
    let client = events::connect(&nats_url()).await.unwrap();
    let js = jetstream::new(client);
    events::ensure_streams(&js, 1).await.unwrap();
    let ingest_pool = support::app_pool(&w.pool).await;
    let worker = PgPoolOptions::new()
        .max_connections(4)
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("SET ROLE app_worker").execute(conn).await?;
                Ok(())
            })
        })
        .connect_with((*w.pool.connect_options()).clone())
        .await
        .unwrap();
    let state = support::state_with(
        &w.pool,
        api::redis_client::Redis::disabled(),
        w.auth.clone(),
        support::config("postgres://unused"),
    )
    .await
    .with_events(js.clone());
    let ingest = ingest::router_with(IngestState {
        pool: ingest_pool.clone(),
        apple: ingest::jws::JwsVerifier::apple(),
        bundle_id: None,
        web_payment: None,
        github_secret: None,
    });
    let (stop, rx) = watch::channel(false);
    let task = tokio::spawn(consumer::run(
        worker.clone(),
        js.clone(),
        consumer::ConsumerConfig::new(
            &format!("dev-{}", Uuid::now_v7().simple()),
            &events::workspace_filter(w.ws),
        ),
        Arc::new(devices::apply::ReadingHandler),
        async move {
            let mut rx = rx;
            let _ = rx.wait_for(|s| *s).await;
        },
    ));
    Stack {
        ingest,
        sse: api::router(state),
        worker,
        js,
        ingest_pool,
        stop,
        task,
    }
}

impl Stack {
    async fn relay(&self) {
        relay::run_once(&self.worker, &self.js, 100).await.unwrap();
    }

    async fn send(&self, path: &str, body: &str) -> (StatusCode, Value) {
        let r = self
            .ingest
            .clone()
            .oneshot(
                Request::post(path)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body.to_owned()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = r.status();
        let bytes = r.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    async fn finish(self) {
        let _ = self.stop.send(true);
        let _ = self.task.await;
        let _ = &self.ingest_pool;
    }
}

struct Sse {
    body: Body,
    buf: String,
}

impl Sse {
    async fn open(app: &Router, ws: Uuid, user: Uuid) -> Self {
        let r = app
            .clone()
            .oneshot(
                Request::get(format!("/v1/workspaces/{ws}/events"))
                    .header(header::AUTHORIZATION, format!("Bearer {user}"))
                    .header("X-Forwarded-For", "10.0.0.1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        Self {
            body: r.into_body(),
            buf: String::new(),
        }
    }

    async fn next(&mut self, wait: Duration) -> Option<String> {
        loop {
            while let Some(pos) = self.buf.find("\n\n") {
                let block = self.buf[..pos].to_owned();
                self.buf.drain(..pos + 2);
                if !block.trim_start().starts_with(':') {
                    return Some(block);
                }
            }
            let frame = tokio::time::timeout(wait, self.body.frame()).await.ok()??;
            if let Ok(data) = frame.unwrap().into_data() {
                self.buf.push_str(std::str::from_utf8(&data).unwrap());
            }
        }
    }
}

fn field<'a>(block: &'a str, name: &str) -> Option<&'a str> {
    block
        .lines()
        .find_map(|l| l.strip_prefix(&format!("{name}: ")))
}

async fn device_measurements(pool: &PgPool, device: &str) -> Vec<(String, f64, String)> {
    sqlx::query_as(
        "SELECT kind, value, source FROM measurements WHERE device_id = $1::uuid ORDER BY kind",
    )
    .bind(device)
    .fetch_all(pool)
    .await
    .unwrap()
}

async fn wait_for_measurements(s: &Stack, pool: &PgPool, device: &str, n: usize) {
    for _ in 0..100 {
        s.relay().await;
        if device_measurements(pool, device).await.len() >= n {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("measurements did not arrive");
}

const ISPINDEL: &str = r#"{"name":"iSpindel000","ID":1234,"token":"unused","angle":40.1,
    "temperature":20.5,"temp_units":"C","battery":4.05,"gravity":1.0123,"interval":900,"RSSI":-70}"#;

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn ispindel_reading_becomes_a_measurement_and_an_sse_event(pool: PgPool) {
    let w = world(&pool).await;
    let s = stack(&w).await;
    let (_, version) = published_recipe(&w, ipa()).await;
    let batch = post(&w, "/batches", json!({"recipe_version_id": version})).await;
    assert_eq!(batch.status, StatusCode::CREATED, "{}", batch.body);
    let batch_id = batch.body["id"].as_str().unwrap().to_owned();

    // Register and pair. The key shows once; the list never shows it.
    let created = call_as(
        &w,
        w.admin,
        "POST",
        "/devices",
        Some(json!({"kind": "ispindel", "name": "Green iSpindel", "batch_id": batch_id})),
        &[],
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    let device_id = created.body["id"].as_str().unwrap().to_owned();
    let key = created.body["ingest_key"].as_str().unwrap().to_owned();
    assert_eq!(created.body["ingest_path"], format!("/v1/ingest/{key}"));
    assert_eq!(created.body["batch_id"], batch_id);
    let list = get(&w, "/devices").await;
    assert_eq!(list.body["items"].as_array().unwrap().len(), 1);
    assert!(!list.body.to_string().contains(&key));
    assert!(get(&w, &format!("/devices/{device_id}")).await.body["ingest_key"].is_null());

    let mut sse = Sse::open(&s.sse, w.ws, w.admin).await;

    // Out-of-range values are rejected and write nothing.
    let (status, body) = s
        .send(
            &format!("/v1/ingest/{key}"),
            r#"{"temperature":20.5,"gravity":0.5}"#,
        )
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["issues"][0]["field"], "gravity");
    let (status, _) = s.send(&format!("/v1/ingest/{key}"), "not json").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let stored: i64 =
        sqlx::query_scalar("SELECT count(*) FROM external_events WHERE source = 'device'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stored, 0);

    // The real reading.
    let (status, body) = s.send(&format!("/v1/ingest/{key}"), ISPINDEL).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    wait_for_measurements(&s, &pool, &device_id, 4).await;
    let rows = device_measurements(&pool, &device_id).await;
    let kinds: Vec<&str> = rows.iter().map(|r| r.0.as_str()).collect();
    assert_eq!(kinds, ["battery_v", "sg", "signal_dbm", "temp_c"]);
    assert!(rows.iter().all(|r| r.2 == "device"));
    assert!((rows[1].1 - 1.0123).abs() < 1e-9);
    let last: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT last_data_at FROM devices WHERE id = $1::uuid")
            .bind(&device_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(last.is_some());

    // The SSE client sees the measurement of the batch.
    let mut seen = None;
    for _ in 0..20 {
        s.relay().await;
        let Some(block) = sse.next(Duration::from_millis(500)).await else {
            continue;
        };
        if field(&block, "event") == Some("batch.measurement_added") {
            seen = Some(block);
            break;
        }
    }
    let block = seen.expect("batch.measurement_added reaches SSE");
    let data: Value = serde_json::from_str(field(&block, "data").unwrap()).unwrap();
    assert_eq!(data["entity"]["kind"], "batch");
    assert_eq!(data["entity"]["id"], batch_id);

    // The measurement shows in the batch list with source `device`.
    let list = get(&w, &format!("/batches/{batch_id}/measurements")).await;
    assert_eq!(list.body["items"].as_array().unwrap().len(), 4);

    // Rotation: the old key is rejected, the new key works.
    let rotated = call_as(
        &w,
        w.admin,
        "POST",
        &format!("/devices/{device_id}/rotate-key"),
        None,
        &[],
    )
    .await;
    assert_eq!(rotated.status, StatusCode::OK, "{}", rotated.body);
    let new_key = rotated.body["ingest_key"].as_str().unwrap().to_owned();
    assert_ne!(new_key, key);
    let (status, _) = s.send(&format!("/v1/ingest/{key}"), ISPINDEL).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = s.send(&format!("/v1/ingest/{new_key}"), ISPINDEL).await;
    assert_eq!(status, StatusCode::OK);
    let audits: i64 =
        sqlx::query_scalar("SELECT count(*) FROM audit_log WHERE action = 'device_key.renew'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(audits, 1);

    // Unknown key, and a deleted device.
    let (status, _) = s
        .send(&format!("/v1/ingest/{}", devices::new_key()), ISPINDEL)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let del = call_as(
        &w,
        w.admin,
        "DELETE",
        &format!("/devices/{device_id}"),
        None,
        &[],
    )
    .await;
    assert_eq!(del.status, StatusCode::NO_CONTENT);
    let (status, _) = s.send(&format!("/v1/ingest/{new_key}"), ISPINDEL).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    s.finish().await;
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn rapt_and_brewfather_formats_and_unpaired_devices(pool: PgPool) {
    let w = world(&pool).await;
    let s = stack(&w).await;
    let make = |kind: &'static str| {
        let w = &w;
        async move {
            let r = call_as(
                w,
                w.admin,
                "POST",
                "/devices",
                Some(json!({"kind": kind, "name": kind})),
                &[],
            )
            .await;
            assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
            (
                r.body["id"].as_str().unwrap().to_owned(),
                r.body["ingest_key"].as_str().unwrap().to_owned(),
            )
        }
    };
    let (rapt_id, rapt_key) = make("rapt").await;
    let (_, tilt_key) = make("tilt").await;
    let (status, _) = s
        .send(
            &format!("/v1/ingest/{rapt_key}"),
            r#"{"temperature":19.5,"gravity":1048.5,"battery":88,"rssi":-60}"#,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    // The Brewfather format is the default of a Tilt bridge; a wrong unit is malformed.
    let (status, _) = s
        .send(
            &format!("/v1/ingest/{tilt_key}"),
            r#"{"name":"Tilt","temp":68,"temp_unit":"F","gravity":1.05}"#,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = s
        .send(
            &format!("/v1/ingest/{tilt_key}"),
            r#"{"temp":1,"temp_unit":"X"}"#,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = s
        .send(&format!("/v1/ingest/{tilt_key}/nope"), r#"{"temp":1}"#)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    // Nobody paired the devices: the readings only set `last_data_at`.
    for _ in 0..50 {
        s.relay().await;
        let done: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM external_events WHERE source = 'device' AND processed_at IS NOT NULL",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        if done == 2 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(device_measurements(&pool, &rapt_id).await.is_empty());
    let last: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT last_data_at FROM devices WHERE id = $1::uuid")
            .bind(&rapt_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(last.is_some());
    s.finish().await;
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn device_endpoints_check_permission_and_input(pool: PgPool) {
    let w = world(&pool).await;
    // `world` gives no `devices.manage`: reads work, writes do not.
    let denied = call_as(
        &w,
        w.admin,
        "POST",
        "/devices",
        Some(json!({"kind": "rapt", "name": "x"})),
        &[],
    )
    .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
    assert_eq!(get(&w, "/devices").await.status, StatusCode::OK);
    let mut perms: Vec<&str> = ALL.to_vec();
    perms.push("devices.manage");
    w.auth.add(w.admin, w.ws, &perms);
    let bad = call_as(
        &w,
        w.admin,
        "POST",
        "/devices",
        Some(json!({"kind": "toaster", "name": " "})),
        &[],
    )
    .await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY);
    let unknown_batch = call_as(
        &w,
        w.admin,
        "POST",
        "/devices",
        Some(json!({"kind": "rapt", "name": "x", "batch_id": Uuid::now_v7()})),
        &[],
    )
    .await;
    assert_eq!(unknown_batch.status, StatusCode::UNPROCESSABLE_ENTITY);
    let ok = call_as(
        &w,
        w.admin,
        "POST",
        "/devices",
        Some(json!({"kind": "rapt", "name": "x"})),
        &[],
    )
    .await;
    let id = ok.body["id"].as_str().unwrap();
    let renamed = call_as(
        &w,
        w.admin,
        "PATCH",
        &format!("/devices/{id}"),
        Some(
            json!({"name": "Renamed", "calibration": {"schema_version": 1, "gravity_unit": "sg"}}),
        ),
        &[],
    )
    .await;
    assert_eq!(renamed.status, StatusCode::OK, "{}", renamed.body);
    assert_eq!(renamed.body["name"], "Renamed");
    let readonly = call_as(
        &w,
        w.admin,
        "PATCH",
        &format!("/devices/{id}"),
        Some(json!({"kind": "tilt"})),
        &[],
    )
    .await;
    assert_eq!(readonly.status, StatusCode::UNPROCESSABLE_ENTITY);
    // Another workspace cannot see the device.
    let other_ws = Uuid::now_v7();
    support::insert_workspace(&pool, other_ws).await;
    w.auth.add(w.other, other_ws, &perms);
    let r = support::send(
        &w.app,
        support::req(
            "GET",
            &format!("/v1/workspaces/{other_ws}/devices/{id}"),
            Some(w.other),
        )
        .body(Body::empty())
        .unwrap(),
    )
    .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
}
