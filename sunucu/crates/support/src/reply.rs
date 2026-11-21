//! Writes that start on the support side: a staff message (support panel, or a `/yanit` comment)
//! and the sync request.

use db::outbox::{Entity, NewOutboxEvent, enqueue};
use notifier::create::{NewNotification, create};
use serde_json::json;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{REQUEST_ENTITY, Result, SYNC_TOPIC};

/// Writes the outbox event that asks the worker to sync a request with GitHub. Call it in the
/// transaction of the change.
///
/// # Errors
/// Database errors.
pub async fn enqueue_sync(
    conn: &mut PgConnection,
    workspace_id: Uuid,
    request_id: Uuid,
) -> Result<()> {
    let version: i64 = sqlx::query_scalar("SELECT version FROM support_requests WHERE id = $1")
        .bind(request_id)
        .fetch_one(&mut *conn)
        .await?;
    enqueue(
        conn,
        &NewOutboxEvent {
            topic: SYNC_TOPIC.to_owned(),
            workspace_id: Some(workspace_id),
            entity: Some(Entity {
                kind: REQUEST_ENTITY.to_owned(),
                id: request_id,
                version,
            }),
            data: None,
        },
    )
    .await?;
    Ok(())
}

/// Tells the customer who opened the request that a reply arrived: inbox, push and e-mail. The
/// text of the reply is not in the notification.
///
/// # Errors
/// Database errors.
pub async fn notify_customer(
    conn: &mut PgConnection,
    workspace_id: Uuid,
    request_id: Uuid,
    message_id: Uuid,
) -> Result<()> {
    let row: Option<(i64, Option<Uuid>)> = sqlx::query_as(
        "SELECT r.public_number, m.id
         FROM support_requests r
         LEFT JOIN workspace_members m ON m.workspace_id = r.workspace_id
              AND m.user_id = r.created_by AND m.status = 'active' AND m.deleted_at IS NULL
         WHERE r.id = $1",
    )
    .bind(request_id)
    .fetch_optional(&mut *conn)
    .await?;
    let Some((number, Some(member))) = row else {
        return Ok(());
    };
    create(
        conn,
        &NewNotification {
            workspace_id,
            recipient_member_id: member,
            priority: 2,
            kind: "support.reply".to_owned(),
            text_key: "notification.support.reply".to_owned(),
            params: json!({"request_number": number, "request_id": request_id}),
            batch_id: None,
            batch_step_id: None,
            dedupe_key: Some(format!("support.reply:{message_id}")),
        },
    )
    .await?;
    Ok(())
}

/// Adds a message of the support team to a request (the support panel calls this). A customer
/// message (`internal` false) sets the status to `answered` and notifies the customer. An
/// internal note stays on the support side.
///
/// # Errors
/// Database errors.
pub async fn staff_message(
    conn: &mut PgConnection,
    workspace_id: Uuid,
    request_id: Uuid,
    staff_id: Uuid,
    body: &str,
    internal: bool,
) -> Result<Uuid> {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO support_messages
             (id, workspace_id, request_id, author_kind, author_staff_id, body, visibility)
         VALUES ($1, $2, $3, 'staff', $4, $5, $6)",
    )
    .bind(id)
    .bind(workspace_id)
    .bind(request_id)
    .bind(staff_id)
    .bind(body)
    .bind(if internal { "internal" } else { "customer" })
    .execute(&mut *conn)
    .await?;
    if !internal {
        sqlx::query(
            "UPDATE support_requests SET status = 'answered'
             WHERE id = $1 AND status IN ('open', 'answered', 'resolved')",
        )
        .bind(request_id)
        .execute(&mut *conn)
        .await?;
        notify_customer(conn, workspace_id, request_id, id).await?;
    }
    enqueue_sync(conn, workspace_id, request_id).await?;
    Ok(id)
}
