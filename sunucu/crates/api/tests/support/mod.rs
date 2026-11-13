//! Shared test helpers: a fake authenticator, a demo resource router that exercises every
//! ADR-0008 building block, and a Redis proxy that can be cut mid-test.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(dead_code)] // each test binary uses a subset

use std::{
    collections::{BTreeSet, HashMap},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use api::{
    ApiConfig, ApiError, ApiResult, AppState, Authenticator, ErrorCode, RequestContext,
    concurrency::{self, Concurrency},
    context::{BoxFuture, Identity, Membership},
    error::{ApiJson, ApiQuery, FieldIssue, MergePatch},
    pagination::{Cursor, ListParams, Page, SortSpec},
    redis_client::Redis,
};
use axum::{
    Extension, Json, Router,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Request, StatusCode, header},
    response::Response,
    routing::{get, post},
};
use db::ActorKind;
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;
use uuid::Uuid;

/// Redis of the compose stack (`REDIS_URL` overrides, as in CI).
pub fn redis_url() -> String {
    std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_owned())
}

/// Pool whose connections act as `app_api` (RLS applies, as in production).
pub async fn app_pool(base: &PgPool) -> PgPool {
    PgPoolOptions::new()
        .max_connections(8)
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("SET ROLE app_api").execute(conn).await?;
                Ok(())
            })
        })
        .connect_with((*base.connect_options()).clone())
        .await
        .expect("connect as app_api")
}

pub async fn insert_workspace(base: &PgPool, ws: Uuid) {
    sqlx::query(
        "INSERT INTO workspaces (id, name, mode, region, country, time_zone, support_pseudonym)
         VALUES ($1, 'W', 'business', 'us', 'US', 'America/New_York', $2)",
    )
    .bind(ws)
    .bind(format!("pseudonym-{ws}"))
    .execute(base)
    .await
    .unwrap();
}

pub async fn insert_user(base: &PgPool, user: Uuid) {
    sqlx::query("INSERT INTO users (id, display_name) VALUES ($1, 'T')")
        .bind(user)
        .execute(base)
        .await
        .unwrap();
}

/// Bearer token = user id; memberships are listed per (user, workspace).
#[derive(Default)]
pub struct FakeAuth {
    pub members: Mutex<HashMap<(Uuid, Uuid), BTreeSet<String>>>,
    pub actor: Mutex<Option<ActorKind>>,
}

impl FakeAuth {
    pub fn add(&self, user: Uuid, ws: Uuid, permissions: &[&str]) {
        self.members.lock().unwrap().insert(
            (user, ws),
            permissions.iter().map(|p| (*p).to_owned()).collect(),
        );
    }
}

impl Authenticator for FakeAuth {
    fn authenticate<'a>(
        &'a self,
        headers: &'a axum::http::HeaderMap,
    ) -> BoxFuture<'a, ApiResult<Identity>> {
        Box::pin(async move {
            let user = headers
                .get(header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .and_then(|v| Uuid::parse_str(v).ok())
                .ok_or_else(|| ApiError::new(ErrorCode::Unauthenticated))?;
            Ok(Identity {
                user_id: user,
                session_id: None,
                actor_kind: self.actor.lock().unwrap().unwrap_or(ActorKind::User),
            })
        })
    }

    fn membership<'a>(
        &'a self,
        identity: &'a Identity,
        workspace_id: Uuid,
    ) -> BoxFuture<'a, ApiResult<Option<Membership>>> {
        Box::pin(async move {
            Ok(self
                .members
                .lock()
                .unwrap()
                .get(&(identity.user_id, workspace_id))
                .map(|p| Membership {
                    permissions: p.clone(),
                }))
        })
    }
}

pub fn config(database_url: &str) -> ApiConfig {
    ApiConfig::from_lookup(|n| match n {
        "DATABASE_URL" => Some(database_url.to_owned()),
        "TRUST_FORWARDED_FOR" => Some("true".to_owned()),
        _ => None,
    })
    .unwrap()
}

// --- demo resource -----------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub version: i64,
}

#[derive(Default)]
pub struct Demo {
    pub items: Mutex<Vec<Item>>,
    pub creates: AtomicUsize,
    pub gate: tokio::sync::Notify,
    pub entered: tokio::sync::Notify,
}

impl Demo {
    pub fn with_items(n: u128) -> Arc<Self> {
        let demo = Self::default();
        *demo.items.lock().unwrap() = (1..=n)
            .map(|i| Item {
                id: Uuid::from_u128(i),
                name: format!("item-{i:02}"),
                version: 1,
            })
            .collect();
        Arc::new(demo)
    }
}

#[derive(Deserialize)]
struct NewItem {
    name: String,
}

async fn list_items(
    Extension(demo): Extension<Arc<Demo>>,
    ApiQuery(params): ApiQuery<ListParams>,
) -> ApiResult<Json<Page<Item>>> {
    let limit = params.limit()?;
    let sort = SortSpec::parse(params.sort.as_deref(), &["name"], "name")?;
    let cursor = params.cursor(&sort)?;
    let mut rows = demo.items.lock().unwrap().clone();
    rows.sort_by(|a, b| (a.name.as_str(), a.id).cmp(&(b.name.as_str(), b.id)));
    if sort.descending {
        rows.reverse();
    }
    if let Some(c) = cursor {
        let key = c.key.as_str().unwrap_or_default().to_owned();
        rows.retain(|r| {
            let after = (r.name.as_str(), r.id) > (key.as_str(), c.id);
            if sort.descending {
                !after && (r.name.as_str(), r.id) != (key.as_str(), c.id)
            } else {
                after
            }
        });
    }
    rows.truncate(limit as usize + 1);
    Ok(Json(Page::from_rows(rows, limit, &sort, |r| Cursor {
        key: json!(r.name),
        id: r.id,
    })))
}

async fn get_item(
    Extension(demo): Extension<Arc<Demo>>,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let item = demo
        .items
        .lock()
        .unwrap()
        .iter()
        .find(|i| i.id == id)
        .cloned()
        .ok_or_else(|| ApiError::new(ErrorCode::NotFound))?;
    Ok(concurrency::conditional_get(&headers, item.version, &item))
}

async fn patch_item(
    Extension(demo): Extension<Arc<Demo>>,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    MergePatch(patch): MergePatch<Value>,
) -> ApiResult<Response> {
    let mut items = demo.items.lock().unwrap();
    let item = items
        .iter_mut()
        .find(|i| i.id == id)
        .ok_or_else(|| ApiError::new(ErrorCode::NotFound))?;
    let snapshot = serde_json::to_value(&*item).unwrap();
    concurrency::check_if_match(Concurrency::Optimistic, &headers, item.version, || snapshot)?;
    if let Some(name) = patch.get("name").and_then(Value::as_str) {
        name.clone_into(&mut item.name);
    }
    item.version += 1;
    Ok(concurrency::record_response(
        StatusCode::OK,
        item.version,
        &*item,
    ))
}

async fn create_item(
    Extension(demo): Extension<Arc<Demo>>,
    Path(ws): Path<Uuid>,
    ApiJson(new): ApiJson<NewItem>,
) -> ApiResult<Response> {
    if new.name == "slow" {
        demo.entered.notify_one();
        demo.gate.notified().await;
    }
    if new.name == "invalid" {
        return Err(ApiError::validation(vec![FieldIssue::new(
            "/name",
            "invalid_value",
        )]));
    }
    demo.creates.fetch_add(1, Ordering::SeqCst);
    let item = Item {
        id: Uuid::now_v7(),
        name: new.name,
        version: 1,
    };
    let location = format!("/v1/workspaces/{ws}/demo-items/{}", item.id);
    let mut response = concurrency::record_response(StatusCode::CREATED, 1, &item);
    response
        .headers_mut()
        .insert(header::LOCATION, location.parse().unwrap());
    Ok(response)
}

async fn bulk(ApiJson(body): ApiJson<Value>) -> ApiResult<StatusCode> {
    let len = body["items"].as_array().map_or(0, Vec::len);
    api::ensure_bulk_len("/items", len)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn context_probe(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> ApiResult<Json<Value>> {
    let mut tx = ctx.begin(&state.db).await?;
    let row: (Option<Uuid>, Option<Uuid>, Option<String>, bool) = sqlx::query_as(
        "SELECT app_workspace_id(), app_user_id(), app_actor_kind(), \
                current_setting('transaction_read_only') = 'on'",
    )
    .fetch_one(&mut *tx)
    .await?;
    Ok(Json(json!({
        "workspace_id": row.0, "user_id": row.1, "actor_kind": row.2, "read_only": row.3
    })))
}

async fn needs_permission(ctx: RequestContext) -> ApiResult<StatusCode> {
    ctx.require("recipe.edit")?;
    Ok(StatusCode::NO_CONTENT)
}

async fn trace_probe() -> Json<Value> {
    Json(json!({"trace_id": api::request::current_trace_id()}))
}

async fn broken() -> ApiResult<StatusCode> {
    Err(sqlx::Error::Protocol("secret SELECT * FROM internal".into()).into())
}

async fn unavailable() -> ApiResult<StatusCode> {
    Err(ApiError::unavailable(7))
}

async fn conflict() -> ApiResult<StatusCode> {
    Err(ApiError::new(ErrorCode::Conflict)
        .with_errors(vec![FieldIssue::new("/role_id", "last_admin")]))
}

/// Router with the demo resource under the real layers.
pub fn demo_router(state: &AppState, demo: Arc<Demo>) -> Router {
    let writes = api::idempotent(
        state,
        Router::new()
            .route("/v1/workspaces/{ws}/demo-items", post(create_item))
            .route("/v1/workspaces/{ws}/demo-bulk", post(bulk)),
    );
    let reads = Router::new()
        .route("/v1/workspaces/{ws}/demo-items", get(list_items))
        .route(
            "/v1/workspaces/{ws}/demo-items/{id}",
            get(get_item).patch(patch_item),
        )
        .route("/v1/workspaces/{ws}/context", get(context_probe))
        .route(
            "/v1/workspaces/{ws}/needs-permission",
            get(needs_permission),
        )
        .route("/v1/workspaces/{ws}/trace", get(trace_probe))
        .route("/v1/workspaces/{ws}/broken", get(broken))
        .route("/v1/workspaces/{ws}/unavailable", get(unavailable))
        .route("/v1/workspaces/{ws}/conflict", get(conflict))
        .route("/v1/me/context", get(context_probe));
    let extra = api::authenticated(state, writes.merge(reads)).layer(Extension(demo));
    api::build(state.clone(), extra)
}

// --- requests ----------------------------------------------------------------------------

pub struct Reply {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Value,
}

impl Reply {
    pub fn header(&self, name: &str) -> &str {
        self.headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
    }
}

pub async fn send(app: &Router, request: Request<Body>) -> Reply {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    Reply {
        status,
        headers,
        body,
    }
}

pub fn req(method: &str, path: &str, user: Option<Uuid>) -> axum::http::request::Builder {
    let mut b = Request::builder()
        .method(method)
        .uri(path)
        .header("x-forwarded-for", Uuid::new_v4().to_string());
    if let Some(u) = user {
        b = b.header(header::AUTHORIZATION, format!("Bearer {u}"));
    }
    b
}

pub fn json_post(path: &str, user: Uuid, key: Option<&str>, body: &Value) -> Request<Body> {
    let mut b = req("POST", path, Some(user)).header(header::CONTENT_TYPE, "application/json");
    if let Some(k) = key {
        b = b.header("idempotency-key", k);
    }
    b.body(Body::from(body.to_string())).unwrap()
}

// --- Redis proxy -------------------------------------------------------------------------

/// TCP proxy to the local Redis that can be cut at any moment (simulates Redis dying).
pub struct RedisProxy {
    pub url: String,
    tasks: tokio::task::JoinSet<()>,
}

impl RedisProxy {
    pub async fn start() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let upstream_addr = redis_url().trim_start_matches("redis://").to_owned();
        let mut tasks = tokio::task::JoinSet::new();
        tasks.spawn(async move {
            let mut conns = tokio::task::JoinSet::new();
            loop {
                let Ok((mut client, _)) = listener.accept().await else {
                    break;
                };
                let upstream_addr = upstream_addr.clone();
                conns.spawn(async move {
                    if let Ok(mut upstream) =
                        tokio::net::TcpStream::connect(upstream_addr.as_str()).await
                    {
                        let _ = tokio::io::copy_bidirectional(&mut client, &mut upstream).await;
                    }
                });
            }
        });
        Self {
            url: format!("redis://127.0.0.1:{port}"),
            tasks,
        }
    }

    /// Closes the listener and resets every open connection.
    pub async fn cut(&mut self) {
        self.tasks.abort_all();
        while self.tasks.join_next().await.is_some() {}
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

pub async fn state_with(
    base: &PgPool,
    redis: Redis,
    auth: Arc<FakeAuth>,
    cfg: ApiConfig,
) -> AppState {
    let pool = app_pool(base).await;
    AppState::new(pool, redis, auth, cfg)
}
