//! `ingest` service binary.

use common::{ServiceConfig, http, telemetry};

#[tokio::main]
async fn main() -> common::Result<()> {
    let config = ServiceConfig::from_env(ingest::SERVICE_NAME, ingest::DEFAULT_PORT)?;
    telemetry::init_tracing(&config.service_name)?;
    http::serve(ingest::router(), config.port).await
}
