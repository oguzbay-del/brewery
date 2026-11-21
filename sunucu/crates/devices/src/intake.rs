//! Write side for `ingest` (role `app_api`): find the device by its key, store a checked reading.

use chrono::{DateTime, Utc};
use db::outbox::{Entity, NewOutboxEvent, enqueue};
use serde_json::Value;
use sqlx::{FromRow, PgConnection, PgPool};
use uuid::Uuid;

use crate::{ENTITY, Reading, ReadingBatch, SCHEMA_VERSION, TOPIC, key_hash};

/// A device found by its key.
#[derive(Debug, Clone, FromRow)]
pub struct Device {
    /// Device id.
    pub id: Uuid,
    /// Workspace of the device.
    pub workspace_id: Uuid,
    /// `ispindel`, `rapt`, `tilt` or `other`.
    pub kind: String,
    /// `devices.calibration`.
    pub calibration: Option<Value>,
}

/// Finds the device whose current key is `key`. A rotated or deleted device does not match.
///
/// # Errors
/// Database errors.
pub async fn find(pool: &PgPool, key: &str) -> db::Result<Option<Device>> {
    let row = sqlx::query_as::<_, Device>(
        "SELECT id, workspace_id, kind, calibration FROM find_device_by_key_hash($1)",
    )
    .bind(key_hash(key))
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Stores the readings in `external_events` and writes the outbox event, in the caller's
/// transaction. Returns the `external_events` id.
///
/// # Errors
/// Database errors.
pub async fn store(
    conn: &mut PgConnection,
    device: &Device,
    readings: Vec<Reading>,
    received_at: DateTime<Utc>,
) -> db::Result<Uuid> {
    let id = Uuid::now_v7();
    let payload = serde_json::to_value(ReadingBatch {
        schema_version: SCHEMA_VERSION,
        device_id: device.id,
        workspace_id: device.workspace_id,
        received_at,
        readings,
    })
    .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    sqlx::query(
        "INSERT INTO external_events (id, source, delivery_id, payload) VALUES ($1, 'device', $2, $3)",
    )
    .bind(id)
    .bind(id.to_string())
    .bind(payload)
    .execute(&mut *conn)
    .await?;
    enqueue(
        conn,
        &NewOutboxEvent {
            topic: TOPIC.to_owned(),
            workspace_id: Some(device.workspace_id),
            entity: Some(Entity {
                kind: ENTITY.to_owned(),
                id,
                version: 1,
            }),
            data: None,
        },
    )
    .await?;
    Ok(id)
}
