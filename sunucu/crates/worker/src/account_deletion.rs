//! Scheduled account purge (T2.31c).
//!
//! Database changes are committed once per user before object storage is touched. Failed object
//! deletions are persisted in `maintenance_runs.last_detail` and retried on the next hourly tick.

use std::collections::BTreeSet;

use db::{ActorKind, maintenance::RunStatus};
use serde_json::{Value, json};
use sqlx::PgPool;
use storage::ObjectStore;
use uuid::Uuid;

use crate::Result;

const BATCH_LIMIT: i32 = 100;
/// Processes due users independently and retries object keys left by an earlier run.
pub async fn purge_due(
    pool: &PgPool,
    store: Option<&dyn ObjectStore>,
) -> Result<(RunStatus, String)> {
    let mut pending_keys = BTreeSet::new();
    let mut object_delete_errors = Vec::new();

    for key in load_pending_object_keys(pool).await? {
        match delete_object(store, &key).await {
            Ok(()) => {}
            Err(error) => {
                pending_keys.insert(key.clone());
                object_delete_errors.push(json!({"key": key, "error": error}));
            }
        }
    }

    let due_users = due_user_ids(pool).await?;
    let mut processed_users = Vec::new();
    let mut failed_users = Vec::new();
    let mut membership_summary = Vec::new();

    for user_id in due_users {
        match process_user(pool, user_id).await {
            Ok((memberships, object_keys)) => {
                processed_users.push(user_id);
                membership_summary.push(json!({
                    "user_id": user_id,
                    "memberships": memberships.iter().map(|m| json!({
                        "workspace_id": m.workspace_id,
                        "mode": m.mode,
                        "member_status": m.member_status,
                        "other_active_members": m.other_active_members,
                    })).collect::<Vec<_>>()
                }));
                for key in object_keys {
                    match delete_object(store, &key).await {
                        Ok(()) => {}
                        Err(error) => {
                            pending_keys.insert(key.clone());
                            object_delete_errors.push(json!({"key": key, "error": error}));
                        }
                    }
                }
            }
            Err(error) => {
                let message = error.to_string();
                let code = if message.contains("last_owner") {
                    "last_owner"
                } else {
                    "account_purge_failed"
                };
                failed_users.push(json!({
                    "user_id": user_id,
                    "code": code,
                    "message": message,
                }));
            }
        }
    }

    let status = if failed_users.is_empty() && pending_keys.is_empty() {
        RunStatus::Ok
    } else {
        RunStatus::Findings
    };
    let detail = json!({
        "processed_users": processed_users,
        "failed_users": failed_users,
        "membership_summary": membership_summary,
        "pending_object_keys": pending_keys.into_iter().collect::<Vec<_>>(),
        "object_delete_errors": object_delete_errors,
    });
    Ok((status, detail.to_string()))
}

async fn due_user_ids(pool: &PgPool) -> Result<Vec<Uuid>> {
    let mut tx = pool.begin().await.map_err(db::Error::from)?;
    let due = db::due_account_deletions(&mut tx, BATCH_LIMIT).await?;
    tx.commit().await.map_err(db::Error::from)?;
    Ok(due.into_iter().map(|request| request.user_id).collect())
}

async fn process_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(Vec<db::AccountDeletionMembership>, Vec<String>)> {
    let mut tx = pool.begin().await.map_err(db::Error::from)?;
    let outcome = process_user_transaction(&mut tx, user_id).await;
    match outcome {
        Ok(result) => {
            tx.commit().await.map_err(db::Error::from)?;
            Ok(result)
        }
        Err(error) => {
            let _ = tx.rollback().await;
            Err(error)
        }
    }
}

async fn process_user_transaction(
    tx: &mut sqlx::PgConnection,
    user_id: Uuid,
) -> Result<(Vec<db::AccountDeletionMembership>, Vec<String>)> {
    db::set_actor_kind(tx, ActorKind::System).await?;

    // This guarded function locks and rechecks the due request before exposing memberships.
    let memberships = db::account_deletion_memberships(tx, user_id).await?;

    // 1. Leave Business workspaces. A last-owner exception aborts only this user's transaction.
    db::account_deletion_leave_business(tx, user_id).await?;

    // 2. Soft-delete sole-member Home workspaces; shared Home memberships are disabled.
    let home_workspaces = db::account_deletion_soft_delete_home(tx, user_id).await?;

    // 3. Collect keys, then purge each Home workspace in the same transaction.
    let mut object_keys = BTreeSet::new();
    for workspace_id in home_workspaces {
        object_keys.extend(db::account_deletion_object_keys(tx, workspace_id).await?);
        db::set_workspace_id(tx, workspace_id).await?;
        sqlx::query("SELECT purge_workspace($1, NULL, 'account deletion')")
            .bind(workspace_id)
            .execute(&mut *tx)
            .await
            .map_err(db::Error::from)?;
    }

    // 4. The existing guarded function marks the request done and anonymises the account.
    db::anonymize_user(tx, user_id).await?;
    Ok((memberships, object_keys.into_iter().collect()))
}

async fn load_pending_object_keys(pool: &PgPool) -> Result<Vec<String>> {
    let detail: Option<Option<String>> =
        sqlx::query_scalar("SELECT last_detail FROM maintenance_runs WHERE job = $1")
            .bind(crate::maintenance::JOB_ACCOUNT_DELETIONS)
            .fetch_optional(pool)
            .await
            .map_err(db::Error::from)?;
    let Some(Some(detail)) = detail else {
        return Ok(Vec::new());
    };
    let value: Value = serde_json::from_str(&detail).unwrap_or(Value::Null);
    Ok(value
        .get("pending_object_keys")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect())
}

async fn delete_object(
    store: Option<&dyn ObjectStore>,
    key: &str,
) -> std::result::Result<(), String> {
    let Some(store) = store else {
        return Err("workspace object store is unavailable".to_owned());
    };
    store.delete(key).await.map_err(|error| error.to_string())
}

