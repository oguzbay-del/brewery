//! `worker` service binary.

use common::{ServiceConfig, http, telemetry};

#[tokio::main]
async fn main() -> common::Result<()> {
    let config = ServiceConfig::from_env(worker::SERVICE_NAME, worker::DEFAULT_PORT)?;
    telemetry::init_tracing(&config.service_name)?;

    let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
    let background = tokio::spawn(worker::background::run(stop_rx));

    let served = http::serve(worker::router(), config.port).await;
    // The HTTP server returns after SIGTERM/Ctrl-C (or failure): stop the background tasks too.
    let _ = stop_tx.send(true);
    if let Err(e) = background.await {
        tracing::error!(error = %e, "background task failed");
    }
    served
}
