//! Scheduled account purge (T2.31c).
//!
//! The due-request row stays locked until all workspace cleanup, object deletion and account
//! anonymisation have committed. A failure rolls the transaction back and the maintenance claim
//! is released for retry on the next hourly tick.

use std::collections::BTreeSet;

use db::{ActorKind, AuditEvent, NewAuditEntry};
use sqlx::PgConnection;
use storage::ObjectStore;
use uuid::Uuid;

use crate::{Error, Result};

const BATCH_LIMIT: i32 = 100;

/// Processes one batch of due deletion requests in a single transaction.
pub async fn purge_due(
    pool: &sqlx::PgPool,
    store: Option<&dyn ObjectStore>,
) -> Result<usize> {
    let mut tx = pool.begin().await?;
    db::set_actor_kind(&mut tx, ActorKind::System).await?;
    let due = db::due_account_deletions(&mut tx, BATCH_LIMIT).await?;
    for request in &due {
        purge_user(&mut tx, request.user_id, store).await?;
    }
    tx.commit().await?;
    Ok(due.len())
}

async fn purge_user(
    tx: &mut PgConnection,
    user_id: Uuid,
    store: Option<&dyn ObjectStore>,
) -> Result<()> {
    clear_workspace_context(tx).await?;
    db::set_user_id(tx, user_id).await?;

    // The user-scoped member policy exposes all of this user's memberships, including disabled
    // or soft-deleted rows. A workspace context is then set for each workspace operation.
    let memberships: Vec<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT id, workspace_id FROM workspace_members
         WHERE user_id = $1 ORDER BY workspace_id, id",
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;

    for (member_id, workspace_id) in memberships {
        db::set_workspace_id(tx, workspace_id).await?;
        let mode: Option<String> =
            sqlx::query_scalar("SELECT mode FROM workspaces WHERE id = $1")
                .bind(workspace_id)
                .fetch_optional(&mut *tx)
                .await?;
        match mode.as_deref() {
            Some("home") => purge_home_workspace(tx, workspace_id, user_id, store).await?,
            Some("business") => {
                remove_business_member(tx, workspace_id, member_id, user_id).await?;
            }
            Some(other) => {
                return Err(Error::Format(format!(
                    "unsupported workspace mode {other} for {workspace_id}"
                )));
            }
            None => continue,
        }
    }

    clear_workspace_context(tx).await?;
    db::anonymize_user(tx, user_id).await?;
    Ok(())
}

async fn purge_home_workspace(
    tx: &mut PgConnection,
    workspace_id: Uuid,
    user_id: Uuid,
    store: Option<&dyn ObjectStore>,
) -> Result<()> {
    let keys = workspace_object_keys(tx, workspace_id).await?;
    if !keys.is_empty() && store.is_none() {
        return Err(Error::Format(format!(
            "workspace {workspace_id} has {} objects but object storage is unavailable",
            keys.len()
        )));
    }

    db::append_audit(
        tx,
        workspace_id,
        &NewAuditEntry {
            actor_kind: ActorKind::System,
            actor_id: None,
            event: AuditEvent::AccountDelete,
            record_kind: "workspace".to_owned(),
            record_id: Some(workspace_id),
            previous_value: None,
            new_value: Some(serde_json::json!({
                "action": "home_workspace_purged",
                "user_id": user_id,
            })),
            reason: Some("account deletion grace period elapsed".to_owned()),
        },
    )
    .await?;

    // purge_workspace only accepts soft-deleted workspaces. Null updated_by so the tombstone
    // does not retain the deleted user's identity as deleted_by.
    sqlx::query(
        "UPDATE workspaces SET deleted_at = COALESCE(deleted_at, now()), updated_by = NULL
         WHERE id = $1",
    )
    .bind(workspace_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query("SELECT purge_workspace($1, NULL, 'account deletion')")
        .bind(workspace_id)
        .execute(&mut *tx)
        .await?;

    if let Some(store) = store {
        for key in keys {
            store.delete(&key).await?;
        }
    }
    Ok(())
}

async fn remove_business_member(
    tx: &mut PgConnection,
    workspace_id: Uuid,
    member_id: Uuid,
    user_id: Uuid,
) -> Result<()> {
    db::append_audit(
        tx,
        workspace_id,
        &NewAuditEntry {
            actor_kind: ActorKind::User,
            actor_id: Some(user_id),
            event: AuditEvent::AccountDelete,
            record_kind: "workspace_member".to_owned(),
            record_id: Some(member_id),
            previous_value: None,
            new_value: Some(serde_json::json!({"action": "business_membership_removed"})),
            reason: Some("account deletion grace period elapsed".to_owned()),
        },
    )
    .await?;

    // Keep the business records, but remove the deleted user's creator/updater attribution.
    // The users row remains as the anonymised tombstone referenced by immutable audit rows.
    scrub_workspace_user_references(tx, workspace_id, user_id).await?;
    sqlx::query("UPDATE workspaces SET created_by = NULL, updated_by = NULL WHERE id = $1")
        .bind(workspace_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "DELETE FROM workspace_members
         WHERE workspace_id = $1 AND id = $2 AND user_id = $3",
    )
    .bind(workspace_id)
    .bind(member_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    Ok(())
}

async fn scrub_workspace_user_references(
    tx: &mut PgConnection,
    workspace_id: Uuid,
    user_id: Uuid,
) -> Result<()> {
    let columns: Vec<(String, String, bool)> = sqlx::query_as(
        "SELECT c.table_name, c.column_name, c.is_nullable = 'YES'
         FROM information_schema.columns c
         WHERE c.table_schema = 'public'
           AND c.column_name IN ('user_id', 'created_by', 'updated_by')
           AND c.table_name <> 'workspace_members'
           AND EXISTS (
               SELECT 1 FROM information_schema.columns w
               WHERE w.table_schema = c.table_schema
                 AND w.table_name = c.table_name
                 AND w.column_name = 'workspace_id'
           )
         ORDER BY c.table_name, c.column_name",
    )
    .fetch_all(&mut *tx)
    .await?;

    for (table, column, nullable) in columns {
        ensure_identifier(&table)?;
        ensure_identifier(&column)?;
        let sql = format!(
            "SELECT EXISTS (SELECT 1 FROM \"{table}\" \
             WHERE workspace_id = $1 AND \"{column}\" = $2)"
        );
        if !nullable {
            let references_user: bool = sqlx::query_scalar(&sql)
                .bind(workspace_id)
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await?;
            if references_user {
                return Err(Error::Format(format!(
                    "non-nullable {table}.{column} still references deleted user {user_id}"
                )));
            }
            continue;
        }
        let sql = format!(
            "UPDATE \"{table}\" SET \"{column}\" = NULL \
             WHERE workspace_id = $1 AND \"{column}\" = $2"
        );
        sqlx::query(&sql)
            .bind(workspace_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
    }
    Ok(())
}

async fn workspace_object_keys(
    tx: &mut PgConnection,
    workspace_id: Uuid,
) -> Result<Vec<String>> {
    let columns: Vec<(String, String)> = sqlx::query_as(
        "SELECT c.table_name, c.column_name
         FROM information_schema.columns c
         WHERE c.table_schema = 'public'
           AND c.data_type IN ('text', 'character varying')
           AND (c.column_name = 'object_key' OR c.column_name LIKE '%_object_key')
           AND EXISTS (
               SELECT 1 FROM information_schema.columns w
               WHERE w.table_schema = c.table_schema
                 AND w.table_name = c.table_name
                 AND w.column_name = 'workspace_id'
           )
         ORDER BY c.table_name, c.column_name",
    )
    .fetch_all(&mut *tx)
    .await?;

    let mut keys = BTreeSet::new();
    for (table, column) in columns {
        ensure_identifier(&table)?;
        ensure_identifier(&column)?;
        let sql = format!(
            "SELECT \"{column}\" FROM \"{table}\" \
             WHERE workspace_id = $1 AND \"{column}\" IS NOT NULL"
        );
        let found: Vec<String> = sqlx::query_scalar(&sql)
            .bind(workspace_id)
            .fetch_all(&mut *tx)
            .await?;
        keys.extend(found);
    }
    Ok(keys.into_iter().collect())
}

fn ensure_identifier(value: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(Error::Format("unsafe database identifier".to_owned()));
    }
    Ok(())
}

async fn clear_workspace_context(tx: &mut PgConnection) -> Result<()> {
    sqlx::query("SELECT set_config('app.workspace_id', '', true)")
        .execute(&mut *tx)
        .await?;
    Ok(())
}
