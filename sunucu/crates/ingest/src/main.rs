//! `ingest` service binary.

use std::net::SocketAddr;

use common::{ServiceConfig, shutdown::shutdown_signal, telemetry};

#[tokio::main]
async fn main() -> common::Result<()> {
    let config = ServiceConfig::from_env(ingest::SERVICE_NAME, ingest::DEFAULT_PORT)?;
    telemetry::init_tracing(&config.service_name)?;
    let state = ingest::state_from_env().map_err(std::io::Error::other)?;
    let router = if let Some(state) = state {
        let limiter = ingest::limiter_from_env().map_err(std::io::Error::other)?;
        ingest::router_with_limiter(state, limiter)
    } else {
        tracing::warn!("DATABASE_URL missing; the webhook routes are off");
        ingest::router()
    };
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "listening");
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;
    tracing::info!("server stopped");
    Ok(())
}
