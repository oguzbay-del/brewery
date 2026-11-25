//! `ingest` service binary.

use common::{ServiceConfig, http, telemetry};

#[tokio::main]
async fn main() -> common::Result<()> {
    let config = ServiceConfig::from_env(ingest::SERVICE_NAME, ingest::DEFAULT_PORT)?;
    telemetry::init_tracing(&config.service_name)?;
    let router = if let Some(state) = ingest::state_from_env()? {
        ingest::router_with(state)
    } else {
        tracing::warn!("DATABASE_URL missing; the webhook routes are off");
        ingest::router()
    };
    http::serve(router, config.port).await
}
