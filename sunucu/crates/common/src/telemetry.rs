//! Logging setup (structured JSON logs via `tracing`).

use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{Error, Result};

/// Keeps the Sentry client alive in `init_tracing` (its signature has no guard to return).
#[cfg(feature = "sentry")]
static SENTRY_GUARD: std::sync::Mutex<Option<::sentry::ClientInitGuard>> =
    std::sync::Mutex::new(None);

/// Initialises the global `tracing` subscriber with JSON output on stdout.
///
/// The filter is taken from `RUST_LOG`, defaulting to `info`.
/// With the feature `sentry` and a `SENTRY_DSN`, `ERROR` events also go to Sentry.
/// Returns an error if a global subscriber is already installed.
pub fn init_tracing(service_name: &str) -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().json().flatten_event(true));
    #[cfg(feature = "sentry")]
    let registry = {
        let guard = crate::sentry::init(service_name, env!("CARGO_PKG_VERSION"));
        let layer = guard.as_ref().map(|_| crate::sentry::layer());
        if let Ok(mut slot) = SENTRY_GUARD.lock() {
            *slot = guard;
        }
        registry.with(layer)
    };
    registry
        .try_init()
        .map_err(|e| Error::Telemetry(e.to_string()))?;
    tracing::info!(service = service_name, "tracing initialised");
    Ok(())
}
