//! Rate limiting in Redis (ADR-0008 rule 11.2).
//!
//! Fixed one-minute windows per IP, user and workspace. `RateLimit` and `RateLimit-Policy`
//! (IETF httpapi draft) report the tightest policy that was checked. Without Redis the limit
//! is not applied and the state is logged (fail open).

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{
    AppState,
    config::RateLimitConfig,
    error::{ApiError, ApiResult, ErrorCode},
    redis_client::Redis,
};

/// Window length in seconds.
pub const WINDOW_SECS: u64 = 60;

/// `RateLimit` response header.
pub static RATE_LIMIT: HeaderName = HeaderName::from_static("ratelimit");
/// `RateLimit-Policy` response header.
pub static RATE_LIMIT_POLICY: HeaderName = HeaderName::from_static("ratelimit-policy");

/// What one check found.
#[derive(Debug, Clone, Copy)]
pub struct Snapshot {
    policy: &'static str,
    quota: u32,
    remaining: u32,
    reset_secs: u64,
}

impl Snapshot {
    fn headers(&self) -> [(HeaderName, HeaderValue); 2] {
        let limit = format!(
            "\"{}\";r={};t={}",
            self.policy, self.remaining, self.reset_secs
        );
        let policy = format!("\"{}\";q={};w={}", self.policy, self.quota, WINDOW_SECS);
        [
            (
                RATE_LIMIT.clone(),
                HeaderValue::from_str(&limit).unwrap_or_else(|_| HeaderValue::from_static("")),
            ),
            (
                RATE_LIMIT_POLICY.clone(),
                HeaderValue::from_str(&policy).unwrap_or_else(|_| HeaderValue::from_static("")),
            ),
        ]
    }
}

/// Per-request slot collecting the tightest snapshot; read by the IP layer after the handler.
#[derive(Debug, Clone, Default)]
pub struct Slot(Arc<Mutex<Option<Snapshot>>>);

impl Slot {
    fn record(&self, s: Snapshot) {
        if let Ok(mut guard) = self.0.lock()
            && guard.is_none_or(|old| s.remaining < old.remaining)
        {
            *guard = Some(s);
        }
    }

    fn get(&self) -> Option<Snapshot> {
        self.0.lock().ok().and_then(|g| *g)
    }
}

/// The limiter.
#[derive(Debug, Clone)]
pub struct RateLimiter {
    redis: Redis,
    config: RateLimitConfig,
}

const SCRIPT: &str = "local c = redis.call('INCR', KEYS[1]) \
    if c == 1 then redis.call('EXPIRE', KEYS[1], ARGV[1]) end \
    return c";

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

impl RateLimiter {
    /// A limiter over `redis`.
    #[must_use]
    pub const fn new(redis: Redis, config: RateLimitConfig) -> Self {
        Self { redis, config }
    }

    /// The configuration.
    #[must_use]
    pub const fn config(&self) -> &RateLimitConfig {
        &self.config
    }

    /// Counts one request for `scope`/`id` against `limit`. `Ok` also when Redis is down.
    pub async fn check(
        &self,
        slot: &Slot,
        scope: &'static str,
        id: &str,
        limit: u32,
    ) -> ApiResult<()> {
        let now = now_secs();
        let window = now / WINDOW_SECS;
        let reset_secs = WINDOW_SECS - now % WINDOW_SECS;
        let key = format!("rl:{scope}:{id}:{window}");
        let count = self
            .redis
            .run(|mut c| async move {
                redis::Script::new(SCRIPT)
                    .key(key)
                    .arg(WINDOW_SECS * 2)
                    .invoke_async::<i64>(&mut c)
                    .await
            })
            .await;
        let Some(count) = count else {
            return Ok(());
        };
        let used = u32::try_from(count).unwrap_or(u32::MAX);
        let snapshot = Snapshot {
            policy: scope,
            quota: limit,
            remaining: limit.saturating_sub(used),
            reset_secs,
        };
        slot.record(snapshot);
        if used > limit {
            let mut error = ApiError::new(ErrorCode::RateLimited).with_retry_after(reset_secs);
            for (name, value) in snapshot.headers() {
                error = error.with_header(name, value);
            }
            return Err(error);
        }
        Ok(())
    }

    /// Counts one request for `scope`/`id` in a fixed window of `window_secs` (hours for the
    /// e-mail link limits, ADR-0009 rule 2.3). `Ok` also when Redis is down.
    pub async fn check_window(
        &self,
        scope: &'static str,
        id: &str,
        limit: u32,
        window_secs: u64,
    ) -> ApiResult<()> {
        let now = now_secs();
        let window = now / window_secs;
        let reset_secs = window_secs - now % window_secs;
        let key = format!("rl:{scope}:{id}:{window_secs}:{window}");
        let count = self
            .redis
            .run(|mut c| async move {
                redis::Script::new(SCRIPT)
                    .key(key)
                    .arg(window_secs * 2)
                    .invoke_async::<i64>(&mut c)
                    .await
            })
            .await;
        match count {
            Some(count) if u32::try_from(count).unwrap_or(u32::MAX) > limit => {
                Err(ApiError::new(ErrorCode::RateLimited).with_retry_after(reset_secs))
            }
            _ => Ok(()),
        }
    }

    /// User and workspace checks, run once the request is authenticated.
    pub async fn check_account(
        &self,
        slot: &Slot,
        user_id: uuid::Uuid,
        workspace_id: Option<uuid::Uuid>,
    ) -> ApiResult<()> {
        self.check(
            slot,
            "user",
            &user_id.to_string(),
            self.config.user_per_minute,
        )
        .await?;
        if let Some(ws) = workspace_id {
            self.check(
                slot,
                "workspace",
                &ws.to_string(),
                self.config.workspace_per_minute,
            )
            .await?;
        }
        Ok(())
    }
}

fn client_ip(req: &Request, trust_forwarded: bool) -> String {
    client_ip_of(
        req.headers(),
        req.extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|c| c.0),
        trust_forwarded,
    )
}

/// The caller's IP: the first `X-Forwarded-For` entry when behind a trusted proxy, else the
/// socket address.
pub(crate) fn client_ip_of(
    headers: &axum::http::HeaderMap,
    peer: Option<SocketAddr>,
    trust_forwarded: bool,
) -> String {
    if trust_forwarded
        && let Some(first) = headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .map(str::trim)
            .filter(|v| !v.is_empty())
    {
        return first.to_owned();
    }
    peer.map_or_else(|| "unknown".to_owned(), |a| a.ip().to_string())
}

/// Middleware: per-IP limit, installs the [`Slot`] for later checks and writes the
/// `RateLimit` headers on the way out. `/healthz` and `/readyz` are not limited.
pub async fn layer(State(state): State<AppState>, mut req: Request, next: Next) -> Response {
    if matches!(req.uri().path(), "/healthz" | "/readyz") {
        return next.run(req).await;
    }
    let slot = Slot::default();
    req.extensions_mut().insert(slot.clone());
    let ip = client_ip(&req, state.rate.config().trust_forwarded_for);
    let limit = state.rate.config().ip_per_minute;
    let mut response = match state.rate.check(&slot, "ip", &ip, limit).await {
        Ok(()) => next.run(req).await,
        Err(e) => e.into_response(),
    };
    if let Some(snapshot) = slot.get() {
        for (name, value) in snapshot.headers() {
            response.headers_mut().insert(name, value);
        }
    }
    response
}
