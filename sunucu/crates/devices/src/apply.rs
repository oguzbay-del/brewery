//! Worker side (role `app_worker`): turns a stored reading into `measurements` rows.
//!
//! - The device must still exist. A deleted device drops the reading.
//! - A device that is not paired with a batch only gets `last_data_at`; its values are dropped.
//! - A batch in stage `completed` or `cancelled` takes no new values.
//! - Each value becomes one measurement with source `device`. The event
//!   `batch.measurement_added` then reaches the SSE stream.
//!
//! `external_events.processed_at` and the framework record `processed_events` make the work run
//! once, also when NATS delivers an event two times.

use chrono::Utc;
use db::outbox::{Entity, NewOutboxEvent, enqueue};
use events::consumer::{BoxFuture, Event, Handler, HandlerError};
use serde_json::{Value, json};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{ENTITY, ReadingBatch, SCHEMA_VERSION, TOPIC};

/// The consumer of [`TOPIC`] events.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReadingHandler;

fn classify(e: &sqlx::Error) -> HandlerError {
    // Constraint violations (class 23) never succeed on a retry.
    let permanent = e
        .as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .is_some_and(|c| c.starts_with("23"));
    if permanent {
        HandlerError::Permanent(e.to_string())
    } else {
        HandlerError::Retry(e.to_string())
    }
}

fn dberr(e: impl Into<sqlx::Error>) -> HandlerError {
    classify(&e.into())
}

impl Handler for ReadingHandler {
    fn handle<'a>(
        &'a self,
        event: &'a Event,
        tx: &'a mut PgConnection,
    ) -> BoxFuture<'a, Result<(), HandlerError>> {
        Box::pin(async move {
            if event.topic != TOPIC {
                return Ok(());
            }
            handle_reading(event, tx).await
        })
    }
}

async fn handle_reading(event: &Event, tx: &mut PgConnection) -> Result<(), HandlerError> {
    let external_id = event
        .body
        .pointer("/entity/id")
        .and_then(Value::as_str)
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| HandlerError::Permanent("event has no external event id".to_owned()))?;
    if event.body.pointer("/entity/kind").and_then(Value::as_str) != Some(ENTITY) {
        return Ok(());
    }
    let row: Option<(Value, bool)> = sqlx::query_as(
        "SELECT payload, processed_at IS NOT NULL FROM external_events
         WHERE id = $1 AND source = 'device' FOR UPDATE",
    )
    .bind(external_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(dberr)?;
    let Some((payload, processed)) = row else {
        return Err(HandlerError::Permanent(format!(
            "external event {external_id} not found"
        )));
    };
    if processed {
        return Ok(());
    }
    let batch: ReadingBatch = serde_json::from_value(payload)
        .map_err(|e| HandlerError::Permanent(format!("bad stored reading: {e}")))?;
    if batch.schema_version != SCHEMA_VERSION {
        return Err(HandlerError::Permanent(format!(
            "unknown reading schema {}",
            batch.schema_version
        )));
    }
    apply(tx, &batch).await?;
    sqlx::query("UPDATE external_events SET processed_at = now() WHERE id = $1")
        .bind(external_id)
        .execute(&mut *tx)
        .await
        .map_err(dberr)?;
    Ok(())
}

/// Writes the measurements of one reading. Returns how many it wrote.
///
/// # Errors
/// [`HandlerError`] for database errors.
pub async fn apply(tx: &mut PgConnection, batch: &ReadingBatch) -> Result<u32, HandlerError> {
    db::set_actor_kind(tx, db::ActorKind::System)
        .await
        .map_err(|e| HandlerError::Retry(e.to_string()))?;
    db::set_workspace_id(tx, batch.workspace_id)
        .await
        .map_err(|e| HandlerError::Retry(e.to_string()))?;
    let device: Option<(Option<Uuid>,)> = sqlx::query_as(
        "UPDATE devices SET last_data_at = $2 WHERE id = $1 AND deleted_at IS NULL
         RETURNING batch_id",
    )
    .bind(batch.device_id)
    .bind(batch.received_at)
    .fetch_optional(&mut *tx)
    .await
    .map_err(dberr)?;
    let Some((Some(batch_id),)) = device else {
        tracing::info!(device_id = %batch.device_id, "reading dropped: no device or no batch");
        return Ok(0);
    };
    let row: Option<(String, i64)> =
        sqlx::query_as("SELECT stage, version FROM batches WHERE id = $1 AND deleted_at IS NULL")
            .bind(batch_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(dberr)?;
    let Some((stage, version)) = row else {
        return Ok(0);
    };
    if matches!(stage.as_str(), "completed" | "cancelled") {
        return Ok(0);
    }
    if db::validate_measured_at(batch.received_at, Utc::now()).is_err() {
        tracing::warn!(device_id = %batch.device_id, "reading dropped: time out of range");
        return Ok(0);
    }
    let mut written = 0_u32;
    for r in &batch.readings {
        let n = sqlx::query(
            "INSERT INTO measurements (id, workspace_id, batch_id, kind, value, measured_at, \
                 source, device_id) \
             VALUES ($1, $2, $3, $4, $5, $6, 'device', $7) ON CONFLICT DO NOTHING",
        )
        .bind(Uuid::now_v7())
        .bind(batch.workspace_id)
        .bind(batch_id)
        .bind(&r.kind)
        .bind(r.value)
        .bind(batch.received_at)
        .bind(batch.device_id)
        .execute(&mut *tx)
        .await
        .map_err(dberr)?
        .rows_affected();
        written += u32::try_from(n).unwrap_or(0);
    }
    if written > 0 {
        enqueue(
            tx,
            &NewOutboxEvent {
                topic: "batch.measurement_added".to_owned(),
                workspace_id: Some(batch.workspace_id),
                entity: Some(Entity {
                    kind: "batch".to_owned(),
                    id: batch_id,
                    version,
                }),
                data: Some(json!({"count": written, "source": "device"})),
            },
        )
        .await
        .map_err(|e| HandlerError::Retry(e.to_string()))?;
    }
    Ok(written)
}
