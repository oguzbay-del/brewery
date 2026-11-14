//! SSE endpoint (ADR-0008 rule 9) end to end: record write -> outbox -> relay -> NATS `JetStream`
//! -> `GET /v1/workspaces/{ws}/events`. Needs PostgreSQL (`DATABASE_URL`) and NATS (`NATS_URL`,
//! compose default `nats://127.0.0.1:4222`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::{sync::Arc, time::Duration};

use async_nats::jetstream;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use db::outbox::{Entity, NewOutboxEvent, enqueue};
use http_body_util::BodyExt;
use sqlx::{PgPool, postgres::PgPoolOptions};
use support::{FakeAuth, app_pool, config, insert_user, insert_workspace};
use tower::ServiceExt;
use uuid::Uuid;

fn nats_url() -> String {
    std::env::var("NATS_URL").unwrap_or_else(|_| "nats://127.0.0.1:4222".to_owned())
}

struct World {
    app: Router,
    api: PgPool,
    worker: PgPool,
    js: jetstream::Context,
    ws: Uuid,
    other_ws: Uuid,
    member: Uuid,
    outsider: Uuid,
    auth: Arc<FakeAuth>,
}

async fn world(base: &PgPool) -> World {
    let client = events::connect(&nats_url()).await.unwrap();
    let js = jetstream::new(client);
    events::ensure_streams(&js, 1).await.unwrap();
    let (ws, other_ws, member, outsider) = (
        Uuid::now_v7(),
        Uuid::now_v7(),
        Uuid::now_v7(),
        Uuid::now_v7(),
    );
    for w in [ws, other_ws] {
        insert_workspace(base, w).await;
    }
    for u in [member, outsider] {
        insert_user(base, u).await;
    }
    let auth = Arc::new(FakeAuth::default());
    auth.add(member, ws, &[]);
    auth.add(outsider, other_ws, &[]);
    let api = app_pool(base).await;
    let worker = PgPoolOptions::new()
        .max_connections(4)
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("SET ROLE app_worker").execute(conn).await?;
                Ok(())
            })
        })
        .connect_with((*base.connect_options()).clone())
        .await
        .unwrap();
    let cfg = config(base.connect_options().get_database().unwrap_or("x"));
    let state = api::AppState::new(
        api.clone(),
        api::redis_client::Redis::disabled(),
        auth.clone(),
        cfg,
    )
    .with_events(js.clone());
    World {
        app: api::router(state),
        api,
        worker,
        js,
        ws,
        other_ws,
        member,
        outsider,
        auth,
    }
}

/// Writes the event as a record change would: inside a transaction, through the outbox helper.
async fn write_record(w: &World, ws: Uuid, kind: &str, id: Uuid, version: i64) -> Uuid {
    let mut tx = w.api.begin().await.unwrap();
    // As a request does: the outbox accepts the API's events only for its own workspace.
    db::set_workspace_id(&mut tx, ws).await.unwrap();
    let event_id = enqueue(
        &mut tx,
        &NewOutboxEvent {
            topic: format!("{kind}.updated"),
            workspace_id: Some(ws),
            entity: Some(Entity {
                kind: kind.to_owned(),
                id,
                version,
            }),
            data: None,
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    event_id
}

async fn relay(w: &World) {
    events::relay::run_once(&w.worker, &w.js, 100)
        .await
        .unwrap();
}

fn get(w: &World, ws: Uuid, query: &str, user: Uuid, last: Option<&str>) -> Request<Body> {
    let mut b = Request::get(format!("/v1/workspaces/{ws}/events{query}"))
        .header(header::AUTHORIZATION, format!("Bearer {user}"))
        .header("X-Forwarded-For", "10.0.0.1");
    if let Some(last) = last {
        b = b.header("Last-Event-ID", last);
    }
    let _ = w;
    b.body(Body::empty()).unwrap()
}

/// An open SSE response whose frames are read one by one.
struct Sse {
    body: Body,
    buf: String,
}

impl Sse {
    async fn open(w: &World, req: Request<Body>) -> Self {
        let r = w.app.clone().oneshot(req).await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        assert_eq!(
            r.headers()[header::CONTENT_TYPE].to_str().unwrap(),
            "text/event-stream"
        );
        Self {
            body: r.into_body(),
            buf: String::new(),
        }
    }

    /// The next full event block, or `None` after `wait` of silence. Comments are skipped.
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

const WAIT: Duration = Duration::from_secs(10);
const QUIET: Duration = Duration::from_millis(700);

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_written_record_reaches_the_sse_client(pool: PgPool) {
    let w = world(&pool).await;
    let mut sse = Sse::open(&w, get(&w, w.ws, "", w.member, None)).await;
    let batch = Uuid::now_v7();
    let event_id = write_record(&w, w.ws, "batch", batch, 7).await;
    relay(&w).await;

    let block = sse.next(WAIT).await.expect("event arrives");
    assert_eq!(field(&block, "id"), Some(event_id.to_string().as_str()));
    assert_eq!(field(&block, "event"), Some("batch.updated"));
    let data: serde_json::Value = serde_json::from_str(field(&block, "data").unwrap()).unwrap();
    assert_eq!(data["type"], "batch.updated");
    assert_eq!(data["workspace_id"], w.ws.to_string());
    assert_eq!(data["entity"]["kind"], "batch");
    assert_eq!(data["entity"]["id"], batch.to_string());
    assert_eq!(data["entity"]["version"], 7);
    assert!(data["occurred_at"].as_str().unwrap().ends_with('Z'));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn another_workspaces_events_are_not_visible(pool: PgPool) {
    let w = world(&pool).await;
    let mut mine = Sse::open(&w, get(&w, w.ws, "", w.member, None)).await;
    let mut theirs = Sse::open(&w, get(&w, w.other_ws, "", w.outsider, None)).await;
    write_record(&w, w.other_ws, "batch", Uuid::now_v7(), 1).await;
    let own = write_record(&w, w.ws, "recipe", Uuid::now_v7(), 1).await;
    relay(&w).await;

    let block = mine.next(WAIT).await.expect("own event");
    assert_eq!(field(&block, "id"), Some(own.to_string().as_str()));
    assert!(
        mine.next(QUIET).await.is_none(),
        "nothing from the other workspace"
    );
    let block = theirs.next(WAIT).await.expect("their own event");
    assert_eq!(field(&block, "event"), Some("batch.updated"));
    assert!(theirs.next(QUIET).await.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn only_members_connect(pool: PgPool) {
    let w = world(&pool).await;
    // A member of another workspace sees 404, not 403 (ADR-0008 rule 3.3); no token is 401.
    let r = w
        .app
        .clone()
        .oneshot(get(&w, w.ws, "", w.outsider, None))
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
    let r = w
        .app
        .clone()
        .oneshot(
            Request::get(format!("/v1/workspaces/{}/events", w.ws))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn topics_filter_by_entity(pool: PgPool) {
    let w = world(&pool).await;
    let (wanted, other) = (Uuid::now_v7(), Uuid::now_v7());
    let mut sse = Sse::open(
        &w,
        get(&w, w.ws, &format!("?topics=batch:{wanted}"), w.member, None),
    )
    .await;
    write_record(&w, w.ws, "batch", other, 1).await;
    write_record(&w, w.ws, "recipe", wanted, 1).await;
    let hit = write_record(&w, w.ws, "batch", wanted, 2).await;
    relay(&w).await;
    let block = sse.next(WAIT).await.expect("matching event");
    assert_eq!(field(&block, "id"), Some(hit.to_string().as_str()));
    assert!(sse.next(QUIET).await.is_none());

    let r = w
        .app
        .clone()
        .oneshot(get(&w, w.ws, "?topics=nonsense", w.member, None))
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn last_event_id_replays_what_was_missed(pool: PgPool) {
    let w = world(&pool).await;
    let e1 = write_record(&w, w.ws, "batch", Uuid::now_v7(), 1).await;
    let e2 = write_record(&w, w.ws, "batch", Uuid::now_v7(), 2).await;
    let e3 = write_record(&w, w.ws, "batch", Uuid::now_v7(), 3).await;
    relay(&w).await;

    let mut sse = Sse::open(&w, get(&w, w.ws, "", w.member, Some(&e1.to_string()))).await;
    let first = sse.next(WAIT).await.expect("replayed event");
    assert_eq!(field(&first, "id"), Some(e2.to_string().as_str()));
    let second = sse.next(WAIT).await.expect("replayed event");
    assert_eq!(field(&second, "id"), Some(e3.to_string().as_str()));
    // Then it goes live.
    let e4 = write_record(&w, w.ws, "batch", Uuid::now_v7(), 4).await;
    relay(&w).await;
    let live = sse.next(WAIT).await.expect("live event");
    assert_eq!(field(&live, "id"), Some(e4.to_string().as_str()));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn unknown_or_expired_last_event_id_resets(pool: PgPool) {
    let w = world(&pool).await;
    // Not a UUIDv7 inside the retention window.
    let mut sse = Sse::open(&w, get(&w, w.ws, "", w.member, Some("garbage"))).await;
    let block = sse.next(WAIT).await.expect("reset");
    assert_eq!(field(&block, "event"), Some("reset"));
    // A fresh id that the stream never saw: nothing to replay, so reset as well.
    let unknown = Uuid::now_v7().to_string();
    let mut sse = Sse::open(&w, get(&w, w.ws, "", w.member, Some(&unknown))).await;
    let block = sse.next(WAIT).await.expect("reset");
    assert_eq!(field(&block, "event"), Some("reset"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn without_nats_the_endpoint_answers_503(pool: PgPool) {
    let w = world(&pool).await;
    let state = api::AppState::new(
        w.api.clone(),
        api::redis_client::Redis::disabled(),
        w.auth.clone(),
        config("x"),
    );
    let app = api::router(state);
    let r = app
        .oneshot(get(&w, w.ws, "", w.member, None))
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::SERVICE_UNAVAILABLE);
}
