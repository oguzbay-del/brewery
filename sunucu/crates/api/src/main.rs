//! `api` service binary.

use std::{net::SocketAddr, sync::Arc};

use api::{
    ApiConfig, AppState,
    auth::{
        AuthServices,
        email::{Mailer, MemoryMailer, SmtpMailer},
    },
    redis_client::Redis,
};
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
    let mailer: Arc<dyn Mailer> = if let Some(smtp) = &config.auth.smtp {
        Arc::new(SmtpMailer::new(
            &smtp.host,
            smtp.port,
            &config.auth.mail_from,
        ))
    } else {
        tracing::warn!("SMTP_HOST not set; sign-in mails are kept in memory and not delivered");
        Arc::new(MemoryMailer::default())
    };
    let store = if let Some(s3) = &config.storage {
        {
            let store = Arc::new(storage::S3Store::new(s3.clone()).map_err(|e| {
                common::Error::Telemetry(format!("object store setup failed: {e}"))
            })?);
            if s3.create_bucket {
                ensure_bucket(store.as_ref()).await;
            }
            Some(store)
        }
    } else {
        tracing::warn!("S3_ENDPOINT not set; file endpoints answer 503");
        None
    };
    let auth = Arc::new(AuthServices::new(
        db.clone(),
        redis.clone(),
        config.auth.clone(),
        mailer,
    ));
    let mut state = AppState::new(db, redis, auth.clone(), config).with_auth_services(auth);
    // SSE needs NATS; without NATS_URL the events endpoint answers 503.
    if let Ok(nats_url) = std::env::var("NATS_URL") {
        match events::connect(&nats_url).await {
            Ok(client) => state = state.with_events(async_nats::jetstream::new(client)),
            Err(e) => tracing::error!(error = %e, "NATS setup failed; SSE is off"),
        }
    } else {
        tracing::warn!("NATS_URL not set; SSE is off");
    }
    if let Some(store) = store {
        state = state.with_store(store);
    }
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

/// Creates the bucket of the local stack; the store may still be starting, so retry briefly.
async fn ensure_bucket(store: &dyn storage::ObjectStore) {
    for attempt in 1..=10_u64 {
        match store.ensure_bucket().await {
            Ok(()) => return,
            Err(e) => {
                tracing::warn!(attempt, error = %e, "bucket creation failed; retrying");
                tokio::time::sleep(std::time::Duration::from_secs(attempt)).await;
            }
        }
    }
    tracing::error!("could not create the bucket; uploads will fail");
}
