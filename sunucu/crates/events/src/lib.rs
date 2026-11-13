//! Events: the outbox relay, the consumer framework and the shared NATS `JetStream` names
//! (ADR-0007 rule 2.5, ADR-0008 rule 9).
//!
//! Flow: a request writes a row into `outbox` in its own transaction ([`db::outbox::enqueue`]).
//! The worker's [`relay`] publishes the rows, in order, to the `EVENTS` stream and marks them
//! published (at least once; the `Nats-Msg-Id` header lets `JetStream` drop repeats inside its
//! duplicate window). Consumers ([`consumer`]) read the stream with durable pull consumers and
//! record every processed event in `processed_events`, so an event takes effect once. The API
//! reads the same stream for SSE.
//!
//! Subjects: `ev.<workspace|_>.<entity kind|_>.<entity id|_>.<topic>` where the topic keeps its
//! dots (`ev.<ws>.batch.<id>.batch.measurement_added`). Dead letters go to
//! `dlq.<consumer>.<original subject>` in the `EVENTS_DLQ` stream.

pub mod consumer;
pub mod relay;

use std::time::Duration;

use async_nats::jetstream::{self, stream};
use serde_json::{Value, json};
use uuid::Uuid;

/// Stream of all outbox events.
pub const STREAM: &str = "EVENTS";
/// Stream of events a consumer gave up on.
pub const DLQ_STREAM: &str = "EVENTS_DLQ";
/// Prefix of event subjects.
pub const SUBJECT_PREFIX: &str = "ev";
/// Prefix of dead-letter subjects.
pub const DLQ_PREFIX: &str = "dlq";
/// How long `EVENTS` keeps messages, hence how far `Last-Event-ID` can replay (ADR-0008 9.4;
/// the one hour is still to be confirmed).
pub const RETENTION: Duration = Duration::from_hours(1);
/// Dead letters are kept this long.
pub const DLQ_RETENTION: Duration = Duration::from_hours(30 * 24);
/// Header carrying the outbox event id (`JetStream` de-duplication key).
pub const MSG_ID_HEADER: &str = "Nats-Msg-Id";

/// Errors of this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// NATS or `JetStream` failure.
    #[error("nats: {0}")]
    Nats(String),
    /// Database failure.
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// Result alias.
pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn nats_err(e: impl std::fmt::Display) -> Error {
    Error::Nats(e.to_string())
}

/// Connects to NATS. Retries the first connection in the background and reconnects forever, so
/// a service can start before NATS and survives its restarts.
pub async fn connect(url: &str) -> Result<async_nats::Client> {
    async_nats::ConnectOptions::new()
        .retry_on_initial_connect()
        .connect(url)
        .await
        .map_err(nats_err)
}

/// Creates (or updates) the `EVENTS` and `EVENTS_DLQ` streams. `replicas` is 1 locally.
pub async fn ensure_streams(js: &jetstream::Context, replicas: usize) -> Result<()> {
    js.get_or_create_stream(stream::Config {
        name: STREAM.to_owned(),
        subjects: vec![format!("{SUBJECT_PREFIX}.>")],
        max_age: RETENTION,
        storage: stream::StorageType::File,
        duplicate_window: Duration::from_mins(2),
        num_replicas: replicas,
        ..Default::default()
    })
    .await
    .map_err(nats_err)?;
    js.get_or_create_stream(stream::Config {
        name: DLQ_STREAM.to_owned(),
        subjects: vec![format!("{DLQ_PREFIX}.>")],
        max_age: DLQ_RETENTION,
        storage: stream::StorageType::File,
        num_replicas: replicas,
        ..Default::default()
    })
    .await
    .map_err(nats_err)?;
    Ok(())
}

/// Subject of an event. Missing parts become `_`.
#[must_use]
pub fn subject(
    workspace_id: Option<Uuid>,
    entity_kind: Option<&str>,
    entity_id: Option<&str>,
    topic: &str,
) -> String {
    let ws = workspace_id.map_or_else(|| "_".to_owned(), |w| w.to_string());
    format!(
        "{SUBJECT_PREFIX}.{ws}.{}.{}.{topic}",
        entity_kind.unwrap_or("_"),
        entity_id.unwrap_or("_")
    )
}

/// Subject filter of all events of one workspace.
#[must_use]
pub fn workspace_filter(workspace_id: Uuid) -> String {
    format!("{SUBJECT_PREFIX}.{workspace_id}.>")
}

/// Splits an event subject into (workspace, kind, entity id).
#[must_use]
pub fn parse_subject(subject: &str) -> Option<(&str, &str, &str)> {
    let mut parts = subject.split('.');
    if parts.next()? != SUBJECT_PREFIX {
        return None;
    }
    Some((parts.next()?, parts.next()?, parts.next()?))
}

/// The message body published for an outbox row; this is also the `data` of the SSE event
/// (ADR-0008 rule 9.2).
#[must_use]
pub fn envelope(
    topic: &str,
    workspace_id: Option<Uuid>,
    payload: &Value,
    occurred_at: chrono::DateTime<chrono::Utc>,
) -> Value {
    let mut v = json!({
        "type": topic,
        "workspace_id": workspace_id,
        "occurred_at": occurred_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    });
    if let Some(e) = payload.get("entity") {
        v["entity"] = e.clone();
    }
    if let Some(d) = payload.get("data") {
        v["data"] = d.clone();
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subject_round_trip() {
        let ws = Uuid::now_v7();
        let id = Uuid::now_v7().to_string();
        let s = subject(
            Some(ws),
            Some("batch"),
            Some(&id),
            "batch.measurement_added",
        );
        assert_eq!(
            parse_subject(&s),
            Some((ws.to_string().as_str(), "batch", id.as_str()))
        );
        assert!(s.starts_with(&workspace_filter(ws).replace(".>", ".")));
        assert_eq!(subject(None, None, None, "x"), "ev._._._.x");
    }

    #[test]
    fn envelope_has_no_content() {
        let payload =
            json!({"schema_version": 1, "entity": {"kind": "batch", "id": "i", "version": 2}});
        let at = chrono::DateTime::from_timestamp(0, 0).unwrap();
        let v = envelope("batch.updated", None, &payload, at);
        assert_eq!(v["type"], "batch.updated");
        assert_eq!(v["entity"]["version"], 2);
        assert_eq!(v["occurred_at"], "1970-01-01T00:00:00Z");
        assert!(v.get("data").is_none());
    }
}
