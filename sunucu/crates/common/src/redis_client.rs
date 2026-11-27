//! Redis access that never takes the API down with it (plan: Redis loss only slows things).
//!
//! The connection manager connects lazily and reconnects by itself; every call has a short
//! timeout. Callers get `None`/`Err` when Redis is unavailable and decide whether to fail
//! open (rate limit) or fall back to PostgreSQL (caches).

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use redis::aio::{ConnectionManager, ConnectionManagerConfig};

/// Longest wait for one Redis command or connection attempt.
pub const OP_TIMEOUT: Duration = Duration::from_millis(250);

/// Handle to Redis; cloneable. Without a URL every call reports "unavailable".
#[derive(Clone)]
pub struct Redis {
    manager: Option<ConnectionManager>,
    healthy: Arc<AtomicBool>,
}

impl std::fmt::Debug for Redis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Redis")
            .field("configured", &self.manager.is_some())
            .finish_non_exhaustive()
    }
}

impl Redis {
    /// A handle with no Redis behind it.
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            manager: None,
            healthy: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Prepares a lazily connecting handle. Fails only on an unparsable URL, never because the
    /// server is down. Must be called inside the Tokio runtime.
    pub fn connect_lazy(url: &str) -> redis::RedisResult<Self> {
        let client = redis::Client::open(url)?;
        let config = ConnectionManagerConfig::new()
            .set_connection_timeout(Some(OP_TIMEOUT))
            .set_response_timeout(Some(OP_TIMEOUT))
            .set_number_of_retries(1);
        let manager = ConnectionManager::new_lazy_with_config(client, config)?;
        Ok(Self {
            manager: Some(manager),
            healthy: Arc::new(AtomicBool::new(true)),
        })
    }

    /// Whether a Redis URL is configured.
    #[must_use]
    pub const fn is_configured(&self) -> bool {
        self.manager.is_some()
    }

    /// Runs `f` with the connection under [`OP_TIMEOUT`]. `None` when Redis is not configured,
    /// down or too slow; outages are logged once per change of state.
    pub async fn run<T, F, Fut>(&self, f: F) -> Option<T>
    where
        F: FnOnce(ConnectionManager) -> Fut,
        Fut: Future<Output = redis::RedisResult<T>>,
    {
        let manager = self.manager.clone()?;
        match tokio::time::timeout(OP_TIMEOUT, f(manager)).await {
            Ok(Ok(value)) => {
                if !self.healthy.swap(true, Ordering::Relaxed) {
                    tracing::info!("redis available again");
                }
                Some(value)
            }
            Ok(Err(e)) => {
                self.note_down(&e.to_string());
                None
            }
            Err(_) => {
                self.note_down("timeout");
                None
            }
        }
    }

    fn note_down(&self, reason: &str) {
        if self.healthy.swap(false, Ordering::Relaxed) {
            tracing::warn!(reason, "redis unavailable; continuing without it");
        }
    }

    /// `PING`; `false` when unavailable or not configured.
    pub async fn ping(&self) -> bool {
        self.run(|mut c| async move { redis::cmd("PING").query_async::<String>(&mut c).await })
            .await
            .is_some()
    }
}
