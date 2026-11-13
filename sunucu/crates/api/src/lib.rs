//! `api` service: HTTP core (ADR-0008). Error envelope, request context, idempotency, `ETag`,
//! pagination, rate limit, request ids and tracing, and the `OpenAPI` document. Resource
//! endpoints come in later jobs and use these building blocks.

pub mod app_version;
pub mod concurrency;
pub mod config;
pub mod context;
pub mod engine;
pub mod error;
pub mod events;
pub mod idempotency;
pub mod openapi;
pub mod pagination;
pub mod ratelimit;
pub mod redis_client;
pub mod request;
pub mod routes;

use std::sync::Arc;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware::{from_fn, from_fn_with_state},
    routing::get,
};
use sqlx::PgPool;

pub use config::ApiConfig;
pub use context::{Authenticator, RequestContext};
pub use error::{ApiError, ApiResult, ErrorCode};
pub use openapi::openapi_json;

use error::FieldIssue;
use ratelimit::RateLimiter;
use redis_client::Redis;

/// Service name reported in logs and the health response.
pub const SERVICE_NAME: &str = "api";
/// Default listen port (override with `PORT`).
pub const DEFAULT_PORT: u16 = 8080;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL pool (`app_api` role, RLS applies).
    pub db: PgPool,
    /// Redis; may be unavailable at any time.
    pub redis: Redis,
    /// Resolves callers and memberships (T2.11).
    pub auth: Arc<dyn Authenticator>,
    /// Configuration.
    pub config: Arc<ApiConfig>,
    /// Rate limiter (over `redis`).
    pub rate: RateLimiter,
    /// NATS `JetStream` for SSE (T2.15); `None` answers the events endpoint with `503`.
    pub events: Option<events::JetStream>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState").finish_non_exhaustive()
    }
}

impl AppState {
    /// Assembles the state; the rate limiter is built from `config` and `redis`.
    #[must_use]
    pub fn new(db: PgPool, redis: Redis, auth: Arc<dyn Authenticator>, config: ApiConfig) -> Self {
        let rate = RateLimiter::new(redis.clone(), config.rate.clone());
        Self {
            db,
            redis,
            auth,
            config: Arc::new(config),
            rate,
            events: None,
        }
    }

    /// Connects the state to NATS `JetStream` (SSE).
    #[must_use]
    pub fn with_events(mut self, js: events::JetStream) -> Self {
        self.events = Some(js);
        self
    }
}

/// Wraps routes that need a signed-in caller: resolves the identity and workspace
/// membership and stores the [`RequestContext`]. Apply [`idempotent`] first (inner) to
/// record-creating `POST` routes.
pub fn authenticated(state: &AppState, routes: Router<AppState>) -> Router<AppState> {
    routes.route_layer(from_fn_with_state(state.clone(), context::layer))
}

/// Wraps record-creating `POST` routes with the idempotency layer (ADR-0008 rule 6).
pub fn idempotent(state: &AppState, routes: Router<AppState>) -> Router<AppState> {
    routes.route_layer(from_fn_with_state(state.clone(), idempotency::layer))
}

/// Builds the service router: the public endpoints plus `extra` (resource routes already
/// wrapped with [`authenticated`] / [`idempotent`]), under the shared layers: request id and
/// trace, rate limit, minimum app version and the 1 MiB body limit.
pub fn build(state: AppState, extra: Router<AppState>) -> Router {
    Router::new()
        .route("/healthz", get(routes::healthz))
        .route("/readyz", get(routes::readyz))
        .route("/v1/version", get(routes::version))
        .route("/v1/meta/fields", get(routes::meta_fields))
        .route(
            "/v1/meta/client-requirements",
            get(routes::client_requirements),
        )
        .merge(events::routes(&state))
        .merge(extra)
        .fallback(error::not_found)
        .method_not_allowed_fallback(error::not_found)
        .layer(DefaultBodyLimit::max(config::BODY_LIMIT_BYTES))
        .layer(from_fn_with_state(state.clone(), app_version::enforce))
        .layer(from_fn_with_state(state.clone(), ratelimit::layer))
        .layer(from_fn(request::layer))
        .with_state(state)
}

/// The router without resource endpoints.
pub fn router(state: AppState) -> Router {
    build(state, Router::new())
}

/// Checks a bulk request against the item limit (500, ADR-0008 rule 11.1).
///
/// # Errors
/// `422 validation_failed` with `too_many_items` (and `min` 1 for an empty list).
pub fn ensure_bulk_len(field: &str, len: usize) -> ApiResult<()> {
    if (1..=config::BULK_MAX_ITEMS).contains(&len) {
        return Ok(());
    }
    let code = if len == 0 {
        "too_few_items"
    } else {
        "too_many_items"
    };
    Err(ApiError::validation(vec![
        FieldIssue::new(field, code)
            .with_params(serde_json::json!({"min": 1, "max": config::BULK_MAX_ITEMS})),
    ]))
}
