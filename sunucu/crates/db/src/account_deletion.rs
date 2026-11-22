//! Account deletion request and anonymisation operations (T2.31a).
//!
//! The caller owns the transaction and must set the user context for API operations.
//! Worker operations use the narrowly granted database functions in migration 0220.

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, Row, types::Uuid};

use crate::Result;

/// A user's deletion request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountDeletionRequest {
    /// Request identifier.
    pub id: Uuid,
    /// Account identifier. This remains as a stable tombstone reference.
    pub user_id: Uuid,
    /// When the user requested deletion.
    pub requested_at: DateTime<Utc>,
    /// Earliest time at which the worker may purge the account.
    pub purge_at: DateTime<Utc>,
    /// Current request state.
    pub status: String,
    /// When the user restored the account by cancelling the request.
    pub restored_at: Option<DateTime<Utc>>,
}

impl AccountDeletionRequest {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            requested_at: row.try_get("requested_at")?,
            purge_at: row.try_get("purge_at")?,
            status: row.try_get("status")?,
            restored_at: row.try_get("restored_at")?,
        })
    }
}

/// A due request claimed by the worker in the caller's transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DueAccountDeletion {
    /// Request identifier.
    pub request_id: Uuid,
    /// Account identifier.
    pub user_id: Uuid,
    /// Scheduled purge time.
    pub purge_at: DateTime<Utc>,
}

/// Creates a pending request, or returns the user's existing pending request.
/// The caller must set `app.user_id` in the transaction.
pub async fn request_account_deletion(
    conn: &mut PgConnection,
    grace_days: i32,
) -> Result<AccountDeletionRequest> {
    let row = sqlx::query("SELECT * FROM request_account_deletion($1)")
        .bind(grace_days)
        .fetch_one(&mut *conn)
        .await?;
    AccountDeletionRequest::from_row(&row)
}

/// Cancels the caller's pending request when its purge time has not arrived.
pub async fn cancel_account_deletion(
    conn: &mut PgConnection,
) -> Result<Option<AccountDeletionRequest>> {
    let row = sqlx::query("SELECT * FROM cancel_account_deletion()")
        .fetch_optional(&mut *conn)
        .await?;
    row.as_ref().map(AccountDeletionRequest::from_row).transpose()
}

/// Claims due pending requests. Keep the transaction open while processing the returned rows.
pub async fn due_account_deletions(
    conn: &mut PgConnection,
    limit: i32,
) -> Result<Vec<DueAccountDeletion>> {
    let rows = sqlx::query("SELECT * FROM due_account_deletions($1)")
        .bind(limit)
        .fetch_all(&mut *conn)
        .await?;
    rows.iter()
        .map(|row| {
            Ok(DueAccountDeletion {
                request_id: row.try_get("request_id")?,
                user_id: row.try_get("user_id")?,
                purge_at: row.try_get("purge_at")?,
            })
        })
        .collect()
}

/// Replaces personal account fields and revokes local credentials.
/// Apple refresh-token ciphertext remains for the separate Apple revocation task.
pub async fn anonymize_user(conn: &mut PgConnection, user_id: Uuid) -> Result<()> {
    sqlx::query("SELECT anonymize_user($1)")
        .bind(user_id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}
