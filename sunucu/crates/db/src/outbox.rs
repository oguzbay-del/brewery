//! Writing to the transactional outbox (ADR-0007 rule 2.5; ADR-0008 rule 9).
//!
//! [`enqueue`] runs inside the same transaction as the record change, so the event exists if and
//! only if the change commits. The worker relay (`events` crate) publishes the rows to NATS
//! `JetStream` afterwards. The row carries identity and a small payload only, never record content
//! (the SSE event is a notification; the client re-reads the record).

use serde_json::{Value, json};
use sqlx::{PgConnection, types::Uuid};

use crate::{Error, Result};

/// `schema_version` this code writes into the outbox payload.
pub const PAYLOAD_SCHEMA_VERSION: i64 = 1;

/// The record an event is about (`entity` of the SSE event).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    /// Record kind in `snake_case` (`batch`, `recipe`).
    pub kind: String,
    /// Record id.
    pub id: Uuid,
    /// Record version after the change.
    pub version: i64,
}

/// An event to enqueue.
#[derive(Debug, Clone, PartialEq)]
pub struct NewOutboxEvent {
    /// Event type, dotted `snake_case` (`batch.measurement_added`); also the SSE event name.
    pub topic: String,
    /// Workspace the event belongs to; `None` for system events.
    pub workspace_id: Option<Uuid>,
    /// The changed record, if the event is about one.
    pub entity: Option<Entity>,
    /// Small, permission-independent extra payload (soft lock holder, counters); never content.
    pub data: Option<Value>,
}

/// Whether `s` is usable as one NATS subject token and as an entity kind: 1..=64 characters of
/// `a-z0-9_-`.
#[must_use]
pub fn is_token(s: &str) -> bool {
    (1..=64).contains(&s.len())
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// Whether `topic` is a valid event type: dot-separated tokens, at most 200 characters.
#[must_use]
pub fn is_valid_topic(topic: &str) -> bool {
    topic.len() <= 200 && topic.split('.').all(is_token)
}

impl NewOutboxEvent {
    /// The JSON payload stored in `outbox.payload`.
    #[must_use]
    pub fn payload(&self) -> Value {
        let mut payload = json!({ "schema_version": PAYLOAD_SCHEMA_VERSION });
        if let Some(e) = &self.entity {
            payload["entity"] = json!({ "kind": e.kind, "id": e.id, "version": e.version });
        }
        if let Some(d) = &self.data {
            payload["data"] = d.clone();
        }
        payload
    }
}

/// Inserts the event into the outbox on `conn` (inside the caller's transaction) and returns its
/// id (`UUIDv7`, the SSE `id`).
///
/// # Errors
/// [`Error::InvalidOutboxEvent`] for a malformed topic or entity kind; database errors.
pub async fn enqueue(conn: &mut PgConnection, event: &NewOutboxEvent) -> Result<Uuid> {
    if !is_valid_topic(&event.topic) {
        return Err(Error::InvalidOutboxEvent(format!(
            "topic {:?} is not dotted snake_case",
            event.topic
        )));
    }
    if let Some(e) = &event.entity
        && !is_token(&e.kind)
    {
        return Err(Error::InvalidOutboxEvent(format!(
            "entity kind {:?} is not snake_case",
            e.kind
        )));
    }
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO outbox (id, topic, workspace_id, payload) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(&event.topic)
        .bind(event.workspace_id)
        .bind(event.payload())
        .execute(conn)
        .await?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topics_and_tokens() {
        assert!(is_valid_topic("batch.measurement_added"));
        assert!(is_valid_topic("recipe"));
        for bad in ["", "Batch.x", "a..b", "a b", "a.*", "a.>", ".a", "a."] {
            assert!(!is_valid_topic(bad), "{bad}");
        }
        assert!(!is_valid_topic(&"a".repeat(201)));
    }

    #[test]
    fn payload_carries_entity_and_data() {
        let id = Uuid::now_v7();
        let ev = NewOutboxEvent {
            topic: "batch.updated".to_owned(),
            workspace_id: None,
            entity: Some(Entity {
                kind: "batch".to_owned(),
                id,
                version: 3,
            }),
            data: Some(json!({"n": 1})),
        };
        let p = ev.payload();
        assert_eq!(p["schema_version"], 1);
        assert_eq!(p["entity"]["version"], 3);
        assert_eq!(p["entity"]["id"], id.to_string());
        assert_eq!(p["data"]["n"], 1);
    }
}
