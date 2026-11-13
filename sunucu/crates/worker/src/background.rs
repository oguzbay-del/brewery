//! Background work: the outbox relay and the event consumers (T2.15).
//!
//! Needs `DATABASE_URL` (role `app_worker`) and `NATS_URL`; without them the worker only serves
//! its health endpoint and says so in the log.

use std::time::Duration;

use events::{consumer, relay};
use tokio::sync::watch;

/// How long handled-event markers are kept (far beyond the stream retention).
const PROCESSED_KEEP: Duration = Duration::from_hours(7 * 24);

async fn stopped(mut rx: watch::Receiver<bool>) {
    while !*rx.borrow() {
        if rx.changed().await.is_err() {
            return;
        }
    }
}

/// Runs until `stop` turns `true`.
pub async fn run(stop: watch::Receiver<bool>) {
    let (Ok(database_url), Ok(nats_url)) = (db::database_url(), std::env::var("NATS_URL")) else {
        tracing::warn!("DATABASE_URL or NATS_URL missing; outbox relay and consumers are off");
        return;
    };
    let pool = match db::connect_lazy(&database_url, 8) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "database setup failed; background work is off");
            return;
        }
    };
    let client = match events::connect(&nats_url).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "NATS setup failed; background work is off");
            return;
        }
    };
    let js = async_nats::jetstream::new(client);
    // The streams may not exist yet (first start, or NATS data lost): create them, retrying
    // while NATS is down.
    loop {
        match events::ensure_streams(&js, 1).await {
            Ok(()) => break,
            Err(e) => {
                tracing::warn!(error = %e, "waiting for NATS JetStream");
                tokio::select! {
                    () = stopped(stop.clone()) => return,
                    () = tokio::time::sleep(Duration::from_secs(2)) => {}
                }
            }
        }
    }

    let relay_task = tokio::spawn(relay::run(
        pool.clone(),
        js.clone(),
        relay::RelayConfig::default(),
        stopped(stop.clone()),
    ));
    // Consumers of later jobs register here: `consumer::run(pool, js, config, handler, stop)`.
    let janitor = {
        let pool = pool.clone();
        let stop = stop.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = consumer::cleanup(&pool, PROCESSED_KEEP).await {
                    tracing::warn!(error = %e, "processed_events cleanup failed");
                }
                tokio::select! {
                    () = stopped(stop.clone()) => return,
                    () = tokio::time::sleep(Duration::from_hours(1)) => {}
                }
            }
        })
    };
    let _ = tokio::join!(relay_task, janitor);
}
