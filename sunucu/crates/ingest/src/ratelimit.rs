//! Rate limits of the device endpoint (T2.54): fixed one-minute windows in Redis.
//!
//! - Per device key (`INGEST_DEVICE_RATE_PER_MINUTE`, default 12): a leaked key cannot fill the
//!   measurement table or the event stream.
//! - Per client IP for invalid keys (`INGEST_INVALID_KEY_RATE_PER_MINUTE`, default 20): guessing
//!   keys is slowed down. Only failed requests count.
//!
//! The key is a secret. Redis key names hold only the first 16 hex characters of its SHA-256.
//! Nothing here logs the key. Redis down (or not configured) fails open, as in the API. The
//! client IP follows the rule of `api::ratelimit`: the first `X-Forwarded-For` entry only when
//! `TRUST_FORWARDED_FOR` is `1` or `true`, otherwise the socket address.

use std::{
    future::Future,
    net::SocketAddr,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::{ConnectInfo, FromRequestParts},
    http::{HeaderValue, StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use common::redis_client::Redis;
use ring::digest;

/// Window length in seconds.
pub const WINDOW_SECS: u64 = 60;
/// Default `INGEST_DEVICE_RATE_PER_MINUTE`.
pub const DEFAULT_DEVICE_PER_MINUTE: u32 = 12;
/// Default `INGEST_INVALID_KEY_RATE_PER_MINUTE`.
pub const DEFAULT_INVALID_PER_MINUTE: u32 = 20;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Counter storage. `None` means "unavailable": the limiter then lets the request pass.
pub trait CounterStore: Send + Sync {
    /// Adds one to `key` (expiry `ttl_secs` on the first add) and returns the new count.
    fn incr(&self, key: String, ttl_secs: u64) -> BoxFuture<'_, Option<i64>>;
    /// Current count of `key` (0 when it does not exist).
    fn get(&self, key: String) -> BoxFuture<'_, Option<i64>>;
}

/// Redis implementation.
#[derive(Debug, Clone)]
pub struct RedisStore(pub Redis);

impl CounterStore for RedisStore {
    fn incr(&self, key: String, ttl_secs: u64) -> BoxFuture<'_, Option<i64>> {
        Box::pin(self.0.incr_expire(key, ttl_secs))
    }

    fn get(&self, key: String) -> BoxFuture<'_, Option<i64>> {
        Box::pin(self.0.get_count(key))
    }
}

/// Limits and settings.
#[derive(Debug, Clone, Copy)]
pub struct IngestRateConfig {
    /// Requests per minute for one device key.
    pub device_per_minute: u32,
    /// Failed requests per minute for one client IP.
    pub invalid_per_minute: u32,
    /// Read the client IP from `X-Forwarded-For` (only behind a trusted proxy).
    pub trust_forwarded_for: bool,
}

impl Default for IngestRateConfig {
    fn default() -> Self {
        Self {
            device_per_minute: DEFAULT_DEVICE_PER_MINUTE,
            invalid_per_minute: DEFAULT_INVALID_PER_MINUTE,
            trust_forwarded_for: false,
        }
    }
}

impl IngestRateConfig {
    /// Reads `INGEST_DEVICE_RATE_PER_MINUTE`, `INGEST_INVALID_KEY_RATE_PER_MINUTE` and
    /// `TRUST_FORWARDED_FOR` through `lookup`.
    ///
    /// # Errors
    /// A limit that is not a number.
    pub fn from_lookup<F>(lookup: F) -> Result<Self, String>
    where
        F: Fn(&str) -> Option<String>,
    {
        let defaults = Self::default();
        let number = |name: &str, default: u32| -> Result<u32, String> {
            match lookup(name)
                .map(|v| v.trim().to_owned())
                .filter(|v| !v.is_empty())
            {
                None => Ok(default),
                Some(v) => v
                    .parse()
                    .map_err(|_| format!("{name} must be a whole number, got {v:?}")),
            }
        };
        Ok(Self {
            device_per_minute: number("INGEST_DEVICE_RATE_PER_MINUTE", defaults.device_per_minute)?,
            invalid_per_minute: number(
                "INGEST_INVALID_KEY_RATE_PER_MINUTE",
                defaults.invalid_per_minute,
            )?,
            trust_forwarded_for: lookup("TRUST_FORWARDED_FOR")
                .is_some_and(|v| matches!(v.trim(), "1" | "true")),
        })
    }
}

/// The limiter. Cheap to clone.
#[derive(Clone)]
pub struct DeviceLimiter {
    store: Option<Arc<dyn CounterStore>>,
    config: IngestRateConfig,
    clock: Arc<dyn Fn() -> u64 + Send + Sync>,
    last_warn_minute: Arc<AtomicU64>,
}

impl std::fmt::Debug for DeviceLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceLimiter")
            .field("enabled", &self.store.is_some())
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

fn system_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// First 16 hex characters of the SHA-256 of `secret`.
fn short_hash(secret: &str) -> String {
    use std::fmt::Write;
    let digest = digest::digest(&digest::SHA256, secret.as_bytes());
    digest
        .as_ref()
        .iter()
        .take(8)
        .fold(String::new(), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
}

/// The caller as far as the limiter needs it.
#[derive(Debug, Clone, Default)]
pub struct Caller {
    peer: Option<SocketAddr>,
    forwarded: Option<String>,
}

impl<S: Send + Sync> FromRequestParts<S> for Caller {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Self {
            peer: parts
                .extensions
                .get::<ConnectInfo<SocketAddr>>()
                .map(|c| c.0),
            forwarded: parts
                .headers
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.split(',').next())
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_owned),
        })
    }
}

/// A request over the limit. Renders `429` with `Retry-After` and a problem body.
#[derive(Debug, Clone, Copy)]
pub struct RateLimited {
    /// Seconds until the window ends.
    pub retry_after: u64,
}

impl IntoResponse for RateLimited {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "type": "/errors/rate_limited",
            "title": "Rate limited",
            "status": 429,
            "code": "rate_limited",
        });
        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            serde_json::to_vec(&body).unwrap_or_default(),
        )
            .into_response();
        let headers = response.headers_mut();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        headers.insert(header::RETRY_AFTER, HeaderValue::from(self.retry_after));
        response
    }
}

impl DeviceLimiter {
    /// A limiter that never limits.
    #[must_use]
    pub fn disabled() -> Self {
        Self::build(None, IngestRateConfig::default(), Arc::new(system_now))
    }

    /// A limiter over Redis. Without a configured Redis the limiter is off.
    #[must_use]
    pub fn redis(redis: Redis, config: IngestRateConfig) -> Self {
        if redis.is_configured() {
            Self::with_store(Arc::new(RedisStore(redis)), config)
        } else {
            Self::disabled()
        }
    }

    /// A limiter over any store.
    #[must_use]
    pub fn with_store(store: Arc<dyn CounterStore>, config: IngestRateConfig) -> Self {
        Self::build(Some(store), config, Arc::new(system_now))
    }

    /// Same as [`Self::with_store`] with a clock in Unix seconds (tests).
    #[must_use]
    pub fn with_store_and_clock(
        store: Arc<dyn CounterStore>,
        config: IngestRateConfig,
        clock: Arc<dyn Fn() -> u64 + Send + Sync>,
    ) -> Self {
        Self::build(Some(store), config, clock)
    }

    fn build(
        store: Option<Arc<dyn CounterStore>>,
        config: IngestRateConfig,
        clock: Arc<dyn Fn() -> u64 + Send + Sync>,
    ) -> Self {
        Self {
            store,
            config,
            clock,
            last_warn_minute: Arc::new(AtomicU64::new(u64::MAX)),
        }
    }

    fn window(&self) -> (u64, u64) {
        let now = (self.clock)();
        (now / WINDOW_SECS, WINDOW_SECS - now % WINDOW_SECS)
    }

    /// One `warn` for each minute at most.
    fn warn_unavailable(&self, window: u64) {
        if self.last_warn_minute.swap(window, Ordering::Relaxed) != window {
            tracing::warn!("ingest rate limit store unavailable; letting requests pass");
        }
    }

    /// The caller's IP (rule of the API).
    fn client_ip(&self, caller: &Caller) -> String {
        if self.config.trust_forwarded_for
            && let Some(first) = &caller.forwarded
        {
            return first.clone();
        }
        caller
            .peer
            .map_or_else(|| "unknown".to_owned(), |a| a.ip().to_string())
    }

    /// Before the key lookup: has this IP already failed too often in this window?
    ///
    /// # Errors
    /// [`RateLimited`] over the limit.
    pub async fn check_ip_blocked(&self, caller: &Caller) -> Result<(), RateLimited> {
        let Some(store) = &self.store else {
            return Ok(());
        };
        let (window, reset) = self.window();
        let key = format!("rl:ingest-invalid:{}:{window}", self.client_ip(caller));
        match store.get(key).await {
            Some(count) if over(count, self.config.invalid_per_minute) => {
                Err(RateLimited { retry_after: reset })
            }
            Some(_) => Ok(()),
            None => {
                self.warn_unavailable(window);
                Ok(())
            }
        }
    }

    /// Counts a request with an invalid key for the caller's IP.
    ///
    /// # Errors
    /// [`RateLimited`] when this failure is over the limit.
    pub async fn note_invalid_key(&self, caller: &Caller) -> Result<(), RateLimited> {
        let Some(store) = &self.store else {
            return Ok(());
        };
        let (window, reset) = self.window();
        let key = format!("rl:ingest-invalid:{}:{window}", self.client_ip(caller));
        match store.incr(key, WINDOW_SECS * 2).await {
            Some(count) if over(count, self.config.invalid_per_minute) => {
                Err(RateLimited { retry_after: reset })
            }
            Some(_) => Ok(()),
            None => {
                self.warn_unavailable(window);
                Ok(())
            }
        }
    }

    /// Counts a request of a valid device key.
    ///
    /// # Errors
    /// [`RateLimited`] over the limit.
    pub async fn check_device(&self, key: &str) -> Result<(), RateLimited> {
        let Some(store) = &self.store else {
            return Ok(());
        };
        let (window, reset) = self.window();
        let redis_key = format!("rl:ingest-device:{}:{window}", short_hash(key));
        match store.incr(redis_key, WINDOW_SECS * 2).await {
            Some(count) if over(count, self.config.device_per_minute) => {
                Err(RateLimited { retry_after: reset })
            }
            Some(_) => Ok(()),
            None => {
                self.warn_unavailable(window);
                Ok(())
            }
        }
    }
}

fn over(count: i64, limit: u32) -> bool {
    u32::try_from(count).unwrap_or(u32::MAX) > limit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_hash_is_16_hex_characters_and_hides_the_key() {
        let h = short_hash("secret-key-value");
        assert_eq!(h.len(), 16);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(!"secret-key-value".contains(&h));
    }

    #[test]
    fn settings_defaults_and_overrides() {
        let none = IngestRateConfig::from_lookup(|_| None).unwrap();
        assert_eq!(none.device_per_minute, 12);
        assert_eq!(none.invalid_per_minute, 20);
        assert!(!none.trust_forwarded_for);
        let set = IngestRateConfig::from_lookup(|n| match n {
            "INGEST_DEVICE_RATE_PER_MINUTE" => Some("5".into()),
            "INGEST_INVALID_KEY_RATE_PER_MINUTE" => Some(" 7 ".into()),
            "TRUST_FORWARDED_FOR" => Some("true".into()),
            _ => None,
        })
        .unwrap();
        assert_eq!((set.device_per_minute, set.invalid_per_minute), (5, 7));
        assert!(set.trust_forwarded_for);
        assert!(
            IngestRateConfig::from_lookup(
                |n| (n == "INGEST_DEVICE_RATE_PER_MINUTE").then(|| "many".into())
            )
            .is_err()
        );
    }
}
