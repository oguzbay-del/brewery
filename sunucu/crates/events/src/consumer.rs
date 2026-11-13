//! Consumer framework: durable pull consumers with once-only effect, retry with back-off and a
//! dead-letter stream.
//!
//! For every message the framework opens a transaction, inserts `(consumer, event id)` into
//! `processed_events` and calls the handler with that transaction. If the insert conflicts the
//! event was already handled: it is acknowledged without calling the handler. Handler success
//! commits (effect and marker together) and acknowledges. [`HandlerError::Retry`] rolls back and
//! asks `JetStream` to redeliver after the back-off delay for that attempt; after `max_attempts`
//! (or on [`HandlerError::Permanent`]) the event is published to `dlq.<consumer>.<subject>` and
//! terminated. A crash after commit but before the acknowledgement redelivers the message and the
//! marker turns it into a no-op.

use std::{pin::Pin, sync::Arc, time::Duration};

use async_nats::{
    HeaderMap,
    jetstream::{self, AckKind, consumer::pull},
};
use futures_util::StreamExt;
use serde_json::Value;
use sqlx::{PgConnection, PgPool, types::Uuid};

use crate::{DLQ_PREFIX, MSG_ID_HEADER, Result, STREAM, nats_err};

/// Boxed future (keeps [`Handler`] object safe).
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A delivered event.
#[derive(Debug, Clone)]
pub struct Event {
    /// Outbox id (`UUIDv7`).
    pub id: Uuid,
    /// NATS subject.
    pub subject: String,
    /// Event type (`batch.measurement_added`).
    pub topic: String,
    /// Workspace, `None` for system events.
    pub workspace_id: Option<Uuid>,
    /// The envelope (`type`, `workspace_id`, `entity`, `occurred_at`, `data`).
    pub body: Value,
    /// Delivery attempt, from 1.
    pub attempt: u32,
}

/// What a handler reports on failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandlerError {
    /// Try again later (back-off), until `max_attempts`.
    Retry(String),
    /// Never succeeds; goes to the dead-letter stream at once.
    Permanent(String),
}

/// An event handler. It runs inside the framework's transaction and must keep its side effects
/// in the database through `tx`; effects elsewhere need their own idempotency.
pub trait Handler: Send + Sync + 'static {
    /// Handles one event.
    fn handle<'a>(
        &'a self,
        event: &'a Event,
        tx: &'a mut PgConnection,
    ) -> BoxFuture<'a, std::result::Result<(), HandlerError>>;
}

/// Consumer settings.
#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    /// Durable name; also the key in `processed_events`.
    pub name: String,
    /// Subject filter (`ev.>` or narrower).
    pub filter: String,
    /// Attempts before the dead letter.
    pub max_attempts: u32,
    /// Delay before attempt n+1 is `backoff[n-1]`; the last entry repeats.
    pub backoff: Vec<Duration>,
    /// Time a delivered message may stay unacknowledged.
    pub ack_wait: Duration,
}

impl ConsumerConfig {
    /// Defaults: 5 attempts, 1 s, 5 s, 30 s, 2 min back-off.
    #[must_use]
    pub fn new(name: &str, filter: &str) -> Self {
        Self {
            name: name.to_owned(),
            filter: filter.to_owned(),
            max_attempts: 5,
            backoff: vec![
                Duration::from_secs(1),
                Duration::from_secs(5),
                Duration::from_secs(30),
                Duration::from_mins(2),
            ],
            ack_wait: Duration::from_secs(30),
        }
    }

    fn delay(&self, attempt: u32) -> Duration {
        let i = usize::try_from(attempt.saturating_sub(1)).unwrap_or(usize::MAX);
        self.backoff
            .get(i)
            .or_else(|| self.backoff.last())
            .copied()
            .unwrap_or(Duration::from_secs(1))
    }
}

fn header(msg: &async_nats::Message, name: &str) -> Option<String> {
    msg.headers
        .as_ref()
        .and_then(|h| h.get(name))
        .map(ToString::to_string)
}

fn parse_event(msg: &jetstream::Message, attempt: u32) -> Option<Event> {
    let id = Uuid::parse_str(&header(&msg.message, MSG_ID_HEADER)?).ok()?;
    let body: Value = serde_json::from_slice(&msg.message.payload).ok()?;
    let topic = body.get("type")?.as_str()?.to_owned();
    let workspace_id = body
        .get("workspace_id")
        .and_then(Value::as_str)
        .and_then(|s| Uuid::parse_str(s).ok());
    Some(Event {
        id,
        subject: msg.message.subject.to_string(),
        topic,
        workspace_id,
        body,
        attempt,
    })
}

enum Outcome {
    Done,
    Retry(String),
    Dead(String),
}

async fn process(
    pool: &PgPool,
    handler: &dyn Handler,
    consumer: &str,
    event: &Event,
) -> Result<Outcome> {
    let mut tx = pool.begin().await?;
    let fresh = sqlx::query(
        "INSERT INTO processed_events (consumer, event_id) VALUES ($1, $2)
         ON CONFLICT DO NOTHING",
    )
    .bind(consumer)
    .bind(event.id)
    .execute(&mut *tx)
    .await?
    .rows_affected()
        == 1;
    if !fresh {
        tracing::debug!(consumer, event_id = %event.id, "already processed, skipped");
        return Ok(Outcome::Done);
    }
    match handler.handle(event, &mut tx).await {
        Ok(()) => {
            tx.commit().await?;
            Ok(Outcome::Done)
        }
        Err(HandlerError::Retry(m)) => Ok(Outcome::Retry(m)),
        Err(HandlerError::Permanent(m)) => Ok(Outcome::Dead(m)),
    }
}

async fn dead_letter(
    js: &jetstream::Context,
    consumer: &str,
    msg: &jetstream::Message,
    reason: &str,
) -> Result<()> {
    let mut headers = HeaderMap::new();
    let id = header(&msg.message, MSG_ID_HEADER).unwrap_or_default();
    headers.insert(MSG_ID_HEADER, format!("{consumer}:{id}").as_str());
    headers.insert("X-Dead-Reason", reason.replace(['\r', '\n'], " ").as_str());
    let subject = format!("{DLQ_PREFIX}.{consumer}.{}", msg.message.subject);
    js.publish_with_headers(subject, headers, msg.message.payload.clone())
        .await
        .map_err(nats_err)?
        .await
        .map_err(nats_err)?;
    Ok(())
}

async fn handle_message(
    pool: &PgPool,
    js: &jetstream::Context,
    config: &ConsumerConfig,
    handler: &dyn Handler,
    msg: &jetstream::Message,
) {
    let attempt = msg
        .info()
        .map_or(1, |i| u32::try_from(i.delivered).unwrap_or(u32::MAX));
    let outcome = if attempt > config.max_attempts {
        Outcome::Dead("max attempts exceeded".to_owned())
    } else if let Some(event) = parse_event(msg, attempt) {
        match process(pool, handler, &config.name, &event).await {
            Ok(Outcome::Retry(m)) if attempt >= config.max_attempts => Outcome::Dead(m),
            Ok(o) => o,
            Err(e) => {
                if attempt >= config.max_attempts {
                    Outcome::Dead(e.to_string())
                } else {
                    Outcome::Retry(e.to_string())
                }
            }
        }
    } else {
        Outcome::Dead("malformed event".to_owned())
    };
    let result = match outcome {
        Outcome::Done => msg.ack().await,
        Outcome::Retry(reason) => {
            tracing::warn!(consumer = %config.name, attempt, %reason, "event handler failed, will retry");
            msg.ack_with(AckKind::Nak(Some(config.delay(attempt))))
                .await
        }
        Outcome::Dead(reason) => {
            tracing::error!(consumer = %config.name, attempt, %reason, subject = %msg.message.subject, "event dead-lettered");
            match dead_letter(js, &config.name, msg, &reason).await {
                Ok(()) => msg.ack_with(AckKind::Term).await,
                Err(e) => {
                    tracing::warn!(error = %e, "dead letter publish failed, will retry");
                    msg.ack_with(AckKind::Nak(Some(config.delay(attempt))))
                        .await
                }
            }
        }
    };
    if let Err(e) = result {
        tracing::warn!(consumer = %config.name, error = %e, "acknowledgement failed; the event may be redelivered");
    }
}

/// Runs one durable consumer until `shutdown` resolves. Reconnects after NATS or stream errors.
pub async fn run(
    pool: PgPool,
    js: jetstream::Context,
    config: ConsumerConfig,
    handler: Arc<dyn Handler>,
    shutdown: impl Future<Output = ()>,
) {
    tokio::pin!(shutdown);
    loop {
        let session = async {
            let stream = js.get_stream(STREAM).await.map_err(nats_err)?;
            let consumer = stream
                .get_or_create_consumer(
                    &config.name,
                    pull::Config {
                        durable_name: Some(config.name.clone()),
                        filter_subject: config.filter.clone(),
                        ack_policy: jetstream::consumer::AckPolicy::Explicit,
                        ack_wait: config.ack_wait,
                        max_ack_pending: 64,
                        ..Default::default()
                    },
                )
                .await
                .map_err(nats_err)?;
            let mut messages = consumer.messages().await.map_err(nats_err)?;
            while let Some(next) = messages.next().await {
                match next {
                    Ok(msg) => handle_message(&pool, &js, &config, handler.as_ref(), &msg).await,
                    Err(e) => return Err(nats_err(e)),
                }
            }
            Ok::<(), crate::Error>(())
        };
        tokio::select! {
            () = &mut shutdown => return,
            r = session => {
                if let Err(e) = r {
                    tracing::warn!(consumer = %config.name, error = %e, "consumer session ended, restarting");
                }
            }
        }
        tokio::select! {
            () = &mut shutdown => return,
            () = tokio::time::sleep(Duration::from_secs(1)) => {}
        }
    }
}

/// Deletes `processed_events` rows older than `keep`; returns the count.
pub async fn cleanup(pool: &PgPool, keep: Duration) -> Result<u64> {
    let r = sqlx::query(
        "DELETE FROM processed_events WHERE processed_at < now() - make_interval(secs => $1)",
    )
    .bind(keep.as_secs_f64())
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}
