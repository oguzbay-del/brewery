//! Helpers of the worker integration tests: seed data, a worker pool and in-process fake HTTP
//! servers (fake APNs, fake GitHub). They need PostgreSQL (`#[sqlx::test]`).

#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use axum::{
    Router,
    body::{Body, to_bytes},
    extract::State,
    http::{Request, Response, StatusCode},
};
use serde_json::Value;
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;

pub fn nats_url() -> String {
    std::env::var("NATS_URL").unwrap_or_else(|_| "nats://127.0.0.1:4222".to_owned())
}

/// A pool whose connections run as `app_worker` (RLS applies).
pub async fn worker_pool(base: &PgPool) -> PgPool {
    role_pool(base, "app_worker").await
}

/// A pool whose connections run as `app_api` (RLS applies).
pub async fn api_pool(base: &PgPool) -> PgPool {
    role_pool(base, "app_api").await
}

async fn role_pool(base: &PgPool, role: &'static str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(6)
        .after_connect(move |conn, _| {
            Box::pin(async move {
                sqlx::query(sqlx::AssertSqlSafe(format!("SET ROLE {role}")))
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .connect_with((*base.connect_options()).clone())
        .await
        .unwrap()
}

/// One workspace with one member and one approved batch.
#[derive(Debug, Clone, Copy)]
pub struct Seed {
    pub ws: Uuid,
    pub user: Uuid,
    pub member: Uuid,
    pub batch: Uuid,
}

/// Seeds through the superuser pool (RLS does not apply).
pub async fn seed(base: &PgPool) -> Seed {
    let s = Seed {
        ws: Uuid::now_v7(),
        user: Uuid::now_v7(),
        member: Uuid::now_v7(),
        batch: Uuid::now_v7(),
    };
    let (recipe, version) = (Uuid::now_v7(), Uuid::now_v7());
    sqlx::query(
        "INSERT INTO workspaces (id, name, mode, region, country, time_zone, support_pseudonym)
         VALUES ($1, 'W', 'home', 'us', 'US', 'America/New_York', $2)",
    )
    .bind(s.ws)
    .bind(format!("pseudonym-{}", s.ws))
    .execute(base)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO users (id, display_name, contact_email, contact_email_verified_at)
         VALUES ($1, 'T', $2, now())",
    )
    .bind(s.user)
    .bind(format!("brewer-{}@example.com", s.user.simple()))
    .execute(base)
    .await
    .unwrap();
    sqlx::query("INSERT INTO workspace_members (id, workspace_id, user_id) VALUES ($1, $2, $3)")
        .bind(s.member)
        .bind(s.ws)
        .bind(s.user)
        .execute(base)
        .await
        .unwrap();
    sqlx::query("INSERT INTO recipes (id, workspace_id, name) VALUES ($1, $2, 'Pale Ale')")
        .bind(recipe)
        .bind(s.ws)
        .execute(base)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO recipe_versions (id, workspace_id, recipe_id, number, status, batch_volume_l,
                                      efficiency_pct, approved_by, approved_at)
         VALUES ($1, $2, $3, 1, 'approved', 20, 72, $4, now())",
    )
    .bind(version)
    .bind(s.ws)
    .bind(recipe)
    .bind(s.user)
    .execute(base)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO batches (id, workspace_id, code, recipe_version_id)
         VALUES ($1, $2, 'B-001', $3)",
    )
    .bind(s.batch)
    .bind(s.ws)
    .bind(version)
    .execute(base)
    .await
    .unwrap();
    s
}

/// A request a fake server received.
#[derive(Debug, Clone)]
pub struct Seen {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Value,
}

impl Seen {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

type Responder = dyn Fn(&Seen) -> (u16, Value) + Send + Sync;

struct FakeState {
    seen: Mutex<Vec<Seen>>,
    responder: Box<Responder>,
    /// Statuses to answer before the responder takes over (for failures).
    forced: Mutex<VecDeque<u16>>,
}

/// An HTTP server on a free local port that records requests and answers with `responder`.
#[derive(Clone)]
pub struct Fake {
    pub url: String,
    state: Arc<FakeState>,
}

impl Fake {
    pub async fn start(responder: impl Fn(&Seen) -> (u16, Value) + Send + Sync + 'static) -> Self {
        let state = Arc::new(FakeState {
            seen: Mutex::default(),
            responder: Box::new(responder),
            forced: Mutex::default(),
        });
        async fn handle(State(s): State<Arc<FakeState>>, req: Request<Body>) -> Response<Body> {
            let (parts, body) = req.into_parts();
            let bytes = to_bytes(body, 1 << 20).await.unwrap_or_default();
            let seen = Seen {
                method: parts.method.to_string(),
                path: parts.uri.path().to_owned(),
                headers: parts
                    .headers
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_owned()))
                    .collect(),
                body: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
            };
            s.seen.lock().unwrap().push(seen.clone());
            let forced = s.forced.lock().unwrap().pop_front();
            let (status, body) = match forced {
                Some(code) => (code, Value::Null),
                None => (s.responder)(&seen),
            };
            Response::builder()
                .status(StatusCode::from_u16(status).unwrap())
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap()
        }
        let app = Router::new().fallback(handle).with_state(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        Self { url, state }
    }

    /// The next `n` requests get this status and no body (before the responder).
    pub fn fail_next(&self, n: usize, status: u16) {
        let mut q = self.state.forced.lock().unwrap();
        for _ in 0..n {
            q.push_back(status);
        }
    }

    pub fn seen(&self) -> Vec<Seen> {
        self.state.seen.lock().unwrap().clone()
    }
}

/// Waits until `check` is true (50 ms steps, at most `secs` seconds).
pub async fn wait_for<F, Fut>(secs: u64, mut check: F) -> bool
where
    F: FnMut() -> Fut,
    Fut: Future<Output = bool>,
{
    for _ in 0..secs * 20 {
        if check().await {
            return true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    false
}
