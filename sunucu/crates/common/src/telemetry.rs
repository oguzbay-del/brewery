//! Logging setup (structured JSON logs via `tracing`).

use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{Error, Result};

/// Initialises the global `tracing` subscriber with JSON output on stdout.
///
/// The filter is taken from `RUST_LOG`, defaulting to `info`.
/// Returns an error if a global subscriber is already installed.
pub fn init_tracing(service_name: &str) -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().json().flatten_event(true))
        .try_init()
        .map_err(|e| Error::Telemetry(e.to_string()))?;
    tracing::info!(service = service_name, "tracing initialised");
    Ok(())
}
