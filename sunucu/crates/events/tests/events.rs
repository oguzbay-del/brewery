//! Outbox relay and consumer framework against real PostgreSQL and NATS `JetStream`.
//!
//! Needs `DATABASE_URL` (`#[sqlx::test]`) and `NATS_URL` (compose default
//! `nats://127.0.0.1:4222`). The restart test starts its own NATS container from the compose
//! image (`nats:2-alpine`) and skips itself when Docker is not available.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::{
    process::Command,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use async_nats::jetstream;
use db::outbox::{Entity, NewOutboxEvent, enqueue};
use events::{
    consumer::{self, BoxFuture, ConsumerConfig, Event, Handler, HandlerError},
    relay,
};
use futures_util::StreamExt;
use sqlx::{PgConnection, PgPool, postgres::PgPoolOptions};
use tokio::sync::watch;
use uuid::Uuid;

fn nats_url() -> String {
    std::env::var("NATS_URL").unwrap_or_else(|_| "nats://127.0.0.1:4222".to_owned())
}

async fn worker_pool(base: &PgPool) -> PgPool {
    PgPoolOptions::new()
        .max_connections(6)
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("SET ROLE app_worker").execute(conn).await?;
                Ok(())
            })
        })
        .connect_with((*base.connect_options()).clone())
        .await
        .unwrap()
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

async fn js(url: &str) -> jetstream::Context {
    let client = events::connect(url).await.unwrap();
    let js = jetstream::new(client);
    events::ensure_streams(&js, 1).await.unwrap();
    js
}

async fn emit(api: &PgPool, ws: Uuid, topic: &str, version: i64) -> Uuid {
    let mut tx = api.begin().await.unwrap();
    // As a request does: the outbox accepts the API's events only for its own workspace.
    db::set_workspace_id(&mut tx, ws).await.unwrap();
    let id = enqueue(
        &mut tx,
        &NewOutboxEvent {
            topic: topic.to_owned(),
            workspace_id: Some(ws),
            entity: Some(Entity {
                kind: "batch".to_owned(),
                id: Uuid::now_v7(),
                version,
            }),
            data: None,
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    id
}

/// Records the ids it sees; `fail` decides the outcome.
struct Recorder {
    seen: Mutex<Vec<Uuid>>,
    calls: AtomicUsize,
    fail: Option<HandlerError>,
}

impl Recorder {
    fn new(fail: Option<HandlerError>) -> Arc<Self> {
        Arc::new(Self {
            seen: Mutex::default(),
            calls: AtomicUsize::new(0),
            fail,
        })
    }
}

impl Handler for Recorder {
    fn handle<'a>(
        &'a self,
        event: &'a Event,
        _tx: &'a mut PgConnection,
    ) -> BoxFuture<'a, Result<(), HandlerError>> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if let Some(e) = &self.fail {
                return Err(e.clone());
            }
            self.seen.lock().unwrap().push(event.id);
            Ok(())
        })
    }
}

async fn wait_for(what: &str, mut cond: impl FnMut() -> bool) {
    for _ in 0..200 {
        if cond() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("timed out waiting for {what}");
}

fn unique(prefix: &str) -> String {
    format!("{prefix}-{}", Uuid::now_v7().simple())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn relay_publishes_in_order_once_and_marks_rows(pool: PgPool) {
    let js = js(&nats_url()).await;
    let api = api_pool(&pool).await;
    let worker = worker_pool(&pool).await;
    let ws = Uuid::now_v7();
    let ids = [
        emit(&api, ws, "batch.created", 1).await,
        emit(&api, ws, "batch.updated", 2).await,
        emit(&api, ws, "batch.updated", 3).await,
    ];

    // Consumer first, so nothing but this workspace's events counts.
    let handler = Recorder::new(None);
    let (stop_tx, stop_rx) = watch::channel(false);
    let cfg = ConsumerConfig::new(&unique("order"), &events::workspace_filter(ws));
    let task = tokio::spawn(consumer::run(
        worker.clone(),
        js.clone(),
        cfg,
        handler.clone(),
        async move {
            let mut rx = stop_rx;
            let _ = rx.wait_for(|s| *s).await;
        },
    ));

    let report = relay::run_once(&worker, &js, 100).await.unwrap();
    assert_eq!(report.published, 3);
    wait_for("three events", || handler.seen.lock().unwrap().len() == 3).await;
    assert_eq!(*handler.seen.lock().unwrap(), ids, "order is kept");

    let unpublished: i64 =
        sqlx::query_scalar("SELECT count(*) FROM outbox WHERE published_at IS NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(unpublished, 0);
    // A second pass has nothing to do and publishes nothing again.
    assert_eq!(
        relay::run_once(&worker, &js, 100).await.unwrap().published,
        0
    );

    // The API role cannot read, mark or delete outbox rows.
    assert!(
        sqlx::query("SELECT id FROM outbox")
            .fetch_all(&api)
            .await
            .is_err()
    );
    let _ = stop_tx.send(true);
    let _ = task.await;
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn malformed_row_is_skipped_not_blocking(pool: PgPool) {
    let js = js(&nats_url()).await;
    let api = api_pool(&pool).await;
    let worker = worker_pool(&pool).await;
    let ws = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO outbox (id, topic, workspace_id, payload)
         VALUES ($1, 'Bad Topic', $2, '{\"schema_version\":1}')",
    )
    .bind(Uuid::now_v7())
    .bind(ws)
    .execute(&pool)
    .await
    .unwrap();
    emit(&api, ws, "batch.created", 1).await;
    let report = relay::run_once(&worker, &js, 100).await.unwrap();
    assert_eq!((report.published, report.quarantined), (1, 1));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn same_event_is_not_processed_twice(pool: PgPool) {
    let js = js(&nats_url()).await;
    let api = api_pool(&pool).await;
    let worker = worker_pool(&pool).await;
    let ws = Uuid::now_v7();
    let id = emit(&api, ws, "batch.created", 1).await;
    relay::run_once(&worker, &js, 10).await.unwrap();

    let name = unique("dedup");
    let handler = Recorder::new(None);
    let run = |handler: Arc<Recorder>| {
        let (tx, rx) = watch::channel(false);
        let task = tokio::spawn(consumer::run(
            worker.clone(),
            js.clone(),
            ConsumerConfig::new(&name, &events::workspace_filter(ws)),
            handler,
            async move {
                let mut rx = rx;
                let _ = rx.wait_for(|s| *s).await;
            },
        ));
        (tx, task)
    };
    let (stop, task) = run(handler.clone());
    wait_for("first delivery", || {
        handler.calls.load(Ordering::SeqCst) == 1
    })
    .await;
    let _ = stop.send(true);
    let _ = task.await;

    // Simulates "committed but the acknowledgement was lost": the durable consumer is gone, a new
    // one with the same name reads the stream from the start and gets the event again.
    let stream = js.get_stream(events::STREAM).await.unwrap();
    stream.delete_consumer(&name).await.unwrap();
    let (stop, task) = run(handler.clone());
    // A second event proves the consumer is alive and past the repeat.
    let id2 = emit(&api, ws, "batch.updated", 2).await;
    relay::run_once(&worker, &js, 10).await.unwrap();
    wait_for("second event", || handler.seen.lock().unwrap().len() == 2).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let _ = stop.send(true);
    let _ = task.await;

    assert_eq!(*handler.seen.lock().unwrap(), [id, id2]);
    assert_eq!(
        handler.calls.load(Ordering::SeqCst),
        2,
        "no second call for the first event"
    );
    let marked: i64 =
        sqlx::query_scalar("SELECT count(*) FROM processed_events WHERE consumer = $1")
            .bind(&name)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(marked, 2);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn failing_event_is_retried_then_dead_lettered(pool: PgPool) {
    let js = js(&nats_url()).await;
    let api = api_pool(&pool).await;
    let worker = worker_pool(&pool).await;
    let ws = Uuid::now_v7();
    let id = emit(&api, ws, "batch.created", 1).await;
    relay::run_once(&worker, &js, 10).await.unwrap();

    let name = unique("dlq");
    let handler = Recorder::new(Some(HandlerError::Retry("boom".to_owned())));
    let mut cfg = ConsumerConfig::new(&name, &events::workspace_filter(ws));
    cfg.max_attempts = 3;
    cfg.backoff = vec![Duration::from_millis(100)];
    let (stop_tx, stop_rx) = watch::channel(false);
    let task = tokio::spawn(consumer::run(
        worker.clone(),
        js.clone(),
        cfg,
        handler.clone(),
        async move {
            let mut rx = stop_rx;
            let _ = rx.wait_for(|s| *s).await;
        },
    ));

    let dlq = js.get_stream(events::DLQ_STREAM).await.unwrap();
    let consumer = dlq
        .create_consumer(jetstream::consumer::pull::Config {
            filter_subject: format!("dlq.{name}.>"),
            ..Default::default()
        })
        .await
        .unwrap();
    let mut messages = consumer.messages().await.unwrap();
    let msg = tokio::time::timeout(Duration::from_secs(20), messages.next())
        .await
        .expect("dead letter arrives")
        .unwrap()
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&msg.payload).unwrap();
    assert_eq!(body["type"], "batch.created");
    assert_eq!(handler.calls.load(Ordering::SeqCst), 3, "three attempts");
    let marked: i64 =
        sqlx::query_scalar("SELECT count(*) FROM processed_events WHERE consumer = $1")
            .bind(&name)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(marked, 0, "failed attempts leave no marker");
    let _ = id;
    let _ = stop_tx.send(true);
    let _ = task.await;
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn permanent_error_goes_straight_to_the_dead_letters(pool: PgPool) {
    let js = js(&nats_url()).await;
    let api = api_pool(&pool).await;
    let worker = worker_pool(&pool).await;
    let ws = Uuid::now_v7();
    emit(&api, ws, "batch.created", 1).await;
    relay::run_once(&worker, &js, 10).await.unwrap();
    let name = unique("perm");
    let handler = Recorder::new(Some(HandlerError::Permanent("never".to_owned())));
    let (stop_tx, stop_rx) = watch::channel(false);
    let task = tokio::spawn(consumer::run(
        worker.clone(),
        js.clone(),
        ConsumerConfig::new(&name, &events::workspace_filter(ws)),
        handler.clone(),
        async move {
            let mut rx = stop_rx;
            let _ = rx.wait_for(|s| *s).await;
        },
    ));
    let dlq = js.get_stream(events::DLQ_STREAM).await.unwrap();
    let consumer = dlq
        .create_consumer(jetstream::consumer::pull::Config {
            filter_subject: format!("dlq.{name}.>"),
            ..Default::default()
        })
        .await
        .unwrap();
    let mut messages = consumer.messages().await.unwrap();
    tokio::time::timeout(Duration::from_secs(10), messages.next())
        .await
        .expect("dead letter arrives")
        .unwrap()
        .unwrap();
    assert_eq!(handler.calls.load(Ordering::SeqCst), 1);
    let _ = stop_tx.send(true);
    let _ = task.await;
}

// --- NATS restart -------------------------------------------------------------------------

struct NatsContainer {
    name: String,
    port: u16,
}

impl NatsContainer {
    fn start() -> Option<Self> {
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .ok()?
            .local_addr()
            .ok()?
            .port();
        let name = format!("t215-nats-{}", Uuid::now_v7().simple());
        let ok = Command::new("docker")
            .args([
                "run",
                "-d",
                "--rm=false",
                "--name",
                &name,
                "-p",
                &format!("127.0.0.1:{port}:4222"),
                "nats:2-alpine",
                "-js",
                "-sd",
                "/data",
            ])
            .output()
            .ok()?
            .status
            .success();
        ok.then_some(Self { name, port })
    }

    fn docker(&self, verb: &str) {
        let _ = Command::new("docker").args([verb, &self.name]).output();
    }

    fn url(&self) -> String {
        format!("nats://127.0.0.1:{}", self.port)
    }
}

impl Drop for NatsContainer {
    fn drop(&mut self) {
        let _ = Command::new("docker")
            .args(["rm", "-f", &self.name])
            .output();
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn events_survive_a_nats_restart(pool: PgPool) {
    let Some(nats) = NatsContainer::start() else {
        eprintln!("docker unavailable: skipping the NATS restart test");
        return;
    };
    let api = api_pool(&pool).await;
    let worker = worker_pool(&pool).await;
    let ws = Uuid::now_v7();

    // Wait for the container, then set everything up and run the relay and a consumer.
    let mut js_ctx = None;
    for _ in 0..50 {
        if let Ok(client) = async_nats::connect(nats.url()).await {
            js_ctx = Some(jetstream::new(client));
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    let _ = js_ctx;
    let js = js(&nats.url()).await;
    let (stop_tx, stop_rx) = watch::channel(false);
    let stopped = |rx: watch::Receiver<bool>| async move {
        let mut rx = rx;
        let _ = rx.wait_for(|s| *s).await;
    };
    let relay_task = tokio::spawn(relay::run(
        worker.clone(),
        js.clone(),
        relay::RelayConfig {
            idle: Duration::from_millis(100),
            backoff: Duration::from_millis(200),
            max_backoff: Duration::from_secs(1),
            ..relay::RelayConfig::default()
        },
        stopped(stop_rx.clone()),
    ));
    let handler = Recorder::new(None);
    let consumer_task = tokio::spawn(consumer::run(
        worker.clone(),
        js.clone(),
        ConsumerConfig::new(&unique("restart"), &events::workspace_filter(ws)),
        handler.clone(),
        stopped(stop_rx.clone()),
    ));

    let before = emit(&api, ws, "batch.created", 1).await;
    wait_for("event before restart", || {
        handler.seen.lock().unwrap().contains(&before)
    })
    .await;

    // NATS goes down; events written meanwhile wait in the outbox.
    nats.docker("stop");
    let during = [
        emit(&api, ws, "batch.updated", 2).await,
        emit(&api, ws, "batch.updated", 3).await,
    ];
    tokio::time::sleep(Duration::from_secs(2)).await;
    let waiting: i64 = sqlx::query_scalar("SELECT count(*) FROM outbox WHERE published_at IS NULL")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(waiting, 2, "nothing is marked published while NATS is down");

    nats.docker("start");
    let after = emit(&api, ws, "batch.updated", 4).await;
    wait_for("events after restart", || {
        let seen = handler.seen.lock().unwrap();
        during.iter().chain([&after]).all(|id| seen.contains(id))
    })
    .await;
    let seen = handler.seen.lock().unwrap().clone();
    let mut unique_seen = seen.clone();
    unique_seen.sort();
    unique_seen.dedup();
    assert_eq!(unique_seen.len(), seen.len(), "no event was handled twice");
    assert_eq!(seen, [before, during[0], during[1], after]);

    let _ = stop_tx.send(true);
    let _ = relay_task.await;
    let _ = consumer_task.await;
}
