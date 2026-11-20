//! Creating a notification.

use db::outbox::{Entity, NewOutboxEvent, enqueue};
use serde_json::{Value, json};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{CREATED_TOPIC, ENTITY, Result};

/// A notification to create.
#[derive(Debug, Clone)]
pub struct NewNotification {
    /// Workspace.
    pub workspace_id: Uuid,
    /// Member who receives it.
    pub recipient_member_id: Uuid,
    /// 1 (critical), 2 (normal) or 3 (low).
    pub priority: i16,
    /// Kind (`alert.threshold`, `reminder`, `support.reply`).
    pub kind: String,
    /// Text key of the client catalog.
    pub text_key: String,
    /// Parameters of the text. `schema_version` is added.
    pub params: Value,
    /// Batch the notification is about.
    pub batch_id: Option<Uuid>,
    /// Batch step the notification is about.
    pub batch_step_id: Option<Uuid>,
    /// A second notification with the same key in the workspace is not created.
    pub dedupe_key: Option<String>,
}

/// Inserts the notification and the outbox event. Returns `None` when `dedupe_key` exists.
///
/// # Errors
/// Database errors.
pub async fn create(conn: &mut PgConnection, n: &NewNotification) -> Result<Option<Uuid>> {
    let mut params = n.params.clone();
    if let Value::Object(map) = &mut params {
        map.entry("schema_version").or_insert(json!(1));
    } else {
        params = json!({"schema_version": 1});
    }
    let id = Uuid::now_v7();
    let inserted: Option<(Uuid, i64)> = sqlx::query_as(
        "INSERT INTO notifications
             (id, workspace_id, recipient_member_id, priority, kind, text_key, params,
              batch_id, batch_step_id, dedupe_key)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (workspace_id, dedupe_key) WHERE dedupe_key IS NOT NULL DO NOTHING
         RETURNING id, version",
    )
    .bind(id)
    .bind(n.workspace_id)
    .bind(n.recipient_member_id)
    .bind(n.priority)
    .bind(&n.kind)
    .bind(&n.text_key)
    .bind(params)
    .bind(n.batch_id)
    .bind(n.batch_step_id)
    .bind(&n.dedupe_key)
    .fetch_optional(&mut *conn)
    .await?;
    let Some((id, version)) = inserted else {
        return Ok(None);
    };
    enqueue(
        conn,
        &NewOutboxEvent {
            topic: CREATED_TOPIC.to_owned(),
            workspace_id: Some(n.workspace_id),
            entity: Some(Entity {
                kind: ENTITY.to_owned(),
                id,
                version,
            }),
            data: Some(json!({"priority": n.priority})),
        },
    )
    .await?;
    Ok(Some(id))
}

/// Active members who record brewing data. When nobody holds the permission, every active member.
///
/// # Errors
/// Database errors.
pub async fn recipients(conn: &mut PgConnection, workspace_id: Uuid) -> Result<Vec<Uuid>> {
    let with_permission: Vec<Uuid> = sqlx::query_scalar(
        "SELECT DISTINCT m.id
         FROM workspace_members m
         JOIN role_assignments ra ON ra.workspace_id = m.workspace_id AND ra.member_id = m.id
              AND ra.deleted_at IS NULL
         JOIN role_permissions rp ON rp.workspace_id = ra.workspace_id AND rp.role_id = ra.role_id
         WHERE m.workspace_id = $1 AND m.status = 'active' AND m.deleted_at IS NULL
           AND rp.permission = 'batch.record'",
    )
    .bind(workspace_id)
    .fetch_all(&mut *conn)
    .await?;
    if !with_permission.is_empty() {
        return Ok(with_permission);
    }
    Ok(sqlx::query_scalar(
        "SELECT id FROM workspace_members
         WHERE workspace_id = $1 AND status = 'active' AND deleted_at IS NULL",
    )
    .bind(workspace_id)
    .fetch_all(&mut *conn)
    .await?)
}
