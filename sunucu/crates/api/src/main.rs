//! `api` service binary.

use std::{net::SocketAddr, sync::Arc};

use api::{ApiConfig, AppState, context::DenyAll, redis_client::Redis};
use common::{otel, shutdown::shutdown_signal};

#[tokio::main]
async fn main() -> common::Result<()> {
    let config = ApiConfig::from_env()?;
    let _telemetry = otel::init_tracing_otel(&config.service.service_name)?;

    let db = db::connect_lazy(&config.database_url, db::DEFAULT_MAX_CONNECTIONS)
        .map_err(|e| common::Error::Telemetry(format!("database setup failed: {e}")))?;
    let redis = if let Some(url) = &config.redis_url {
        Redis::connect_lazy(url)
            .map_err(|e| common::Error::Telemetry(format!("invalid REDIS_URL: {e}")))?
    } else {
        tracing::warn!("REDIS_URL not set; rate limit and caches are off");
        Redis::disabled()
    };
    let port = config.service.port;
    // Real authentication arrives with T2.11; until then every sign-in-only route answers 401.
    let state = AppState::new(db, redis, Arc::new(DenyAll), config);
    let router = api::router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
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
