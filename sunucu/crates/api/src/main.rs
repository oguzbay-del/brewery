//! `api` service binary.

use common::{ServiceConfig, http, telemetry};

#[tokio::main]
async fn main() -> common::Result<()> {
    let config = ServiceConfig::from_env(api::SERVICE_NAME, api::DEFAULT_PORT)?;
    telemetry::init_tracing(&config.service_name)?;
    http::serve(api::router(), config.port).await
}
