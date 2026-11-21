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
#[allow(clippy::too_many_lines)]
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
    // Maintenance (audit digests, retention) needs the database only, not NATS.
    let maintenance_task = tokio::spawn(crate::maintenance::run(
        pool.clone(),
        crate::maintenance::digest_store_from_env(),
        stop.clone(),
    ));
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
    let subscription_task = tokio::spawn(consumer::run(
        pool.clone(),
        js.clone(),
        consumer::ConsumerConfig::new(billing::CONSUMER_NAME, &billing::notification_filter()),
        std::sync::Arc::new(billing::apply::NotificationHandler),
        stopped(stop.clone()),
    ));
    let device_task = tokio::spawn(consumer::run(
        pool.clone(),
        js.clone(),
        consumer::ConsumerConfig::new(devices::CONSUMER_NAME, &devices::reading_filter()),
        std::sync::Arc::new(devices::apply::ReadingHandler),
        stopped(stop.clone()),
    ));
    let deliverer = std::sync::Arc::new(notifier::deliver::Deliverer::from_env());
    let alert_task = tokio::spawn(consumer::run(
        pool.clone(),
        js.clone(),
        consumer::ConsumerConfig::new(notifier::ALERT_CONSUMER, &notifier::alert_filter()),
        std::sync::Arc::new(notifier::alert::AlertHandler),
        stopped(stop.clone()),
    ));
    let delivery_task = tokio::spawn(consumer::run(
        pool.clone(),
        js.clone(),
        consumer::ConsumerConfig::new(notifier::DELIVERY_CONSUMER, &notifier::delivery_filter()),
        std::sync::Arc::new(notifier::deliver::DeliveryHandler::new(deliverer.clone())),
        stopped(stop.clone()),
    ));
    let notify_tick = tokio::spawn(notifier::tick::run(
        pool.clone(),
        deliverer.clone(),
        stop.clone(),
    ));
    let issue_client = support::github::GitHub::from_env().map(std::sync::Arc::new);
    let bot = issue_client.as_ref().map_or_else(
        || "demleme-support[bot]".to_owned(),
        |g| g.bot_login().to_owned(),
    );
    let support_sync = tokio::spawn(consumer::run(
        pool.clone(),
        js.clone(),
        consumer::ConsumerConfig::new(support::SYNC_CONSUMER, &support::sync_filter()),
        std::sync::Arc::new(support::sync::SyncHandler::new(issue_client.clone())),
        stopped(stop.clone()),
    ));
    let support_hook = tokio::spawn(consumer::run(
        pool.clone(),
        js.clone(),
        consumer::ConsumerConfig::new(support::GITHUB_CONSUMER, &support::github_filter()),
        std::sync::Arc::new(support::webhook::GithubHandler::new(&bot)),
        stopped(stop.clone()),
    ));
    let support_corrections = tokio::spawn(consumer::run(
        pool.clone(),
        js.clone(),
        consumer::ConsumerConfig::new(
            support::panel::CORRECTION_CONSUMER,
            &support::panel::correction_filter(),
        ),
        std::sync::Arc::new(support::panel::CorrectionHandler),
        stopped(stop.clone()),
    ));
    let support_jobs = tokio::spawn(support::sync::run_jobs(
        pool.clone(),
        issue_client.clone(),
        stop.clone(),
    ));
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
    let _ = tokio::join!(
        relay_task,
        subscription_task,
        device_task,
        alert_task,
        delivery_task,
        notify_tick,
        support_sync,
        support_hook,
        support_corrections,
        support_jobs,
        janitor,
        maintenance_task
    );
}
