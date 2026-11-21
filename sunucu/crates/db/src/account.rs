//! Account deletion requests (T2.31, ADR-0007 rule 6.2).
//!
//! A request keeps the account for [`RETENTION_DAYS`] days. The user can restore it in that
//! time. After it, the worker deletes the account for good (`account_purge`). These helpers run
//! inside a transaction with the user context (see [`crate::begin`]).

use chrono::{DateTime, Utc};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::Result;

/// Days between a deletion request and the permanent delete. To be verified (product owner).
pub const RETENTION_DAYS: i64 = 30;

/// State of a deletion request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Deletion {
    /// When the user asked.
    pub requested_at: DateTime<Utc>,
    /// When the worker deletes the account.
    pub purge_at: DateTime<Utc>,
}

/// The pending request of `user_id`, if there is one.
pub async fn pending(conn: &mut PgConnection, user_id: Uuid) -> Result<Option<Deletion>> {
    let row: Option<(DateTime<Utc>, DateTime<Utc>)> = sqlx::query_as(
        "SELECT requested_at, purge_at FROM account_deletions \
         WHERE user_id = $1 AND restored_at IS NULL AND purged_at IS NULL",
    )
    .bind(user_id)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(row.map(|(requested_at, purge_at)| Deletion {
        requested_at,
        purge_at,
    }))
}

/// Creates the request. Returns `(request, created)`; a pending request stays as it is.
pub async fn request(conn: &mut PgConnection, user_id: Uuid) -> Result<(Deletion, bool)> {
    let locked: Option<(DateTime<Utc>, DateTime<Utc>)> = sqlx::query_as(
        "SELECT requested_at, purge_at FROM account_deletions \
         WHERE user_id = $1 AND restored_at IS NULL AND purged_at IS NULL FOR UPDATE",
    )
    .bind(user_id)
    .fetch_optional(&mut *conn)
    .await?;
    if let Some((requested_at, purge_at)) = locked {
        return Ok((
            Deletion {
                requested_at,
                purge_at,
            },
            false,
        ));
    }
    let (requested_at, purge_at): (DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
        "INSERT INTO account_deletions (user_id, requested_at, purge_at) \
         VALUES ($1, now(), now() + make_interval(days => $2)) \
         ON CONFLICT (user_id) DO UPDATE SET requested_at = now(), \
           purge_at = now() + make_interval(days => $2), restored_at = NULL, attempts = 0, \
           last_attempt_at = NULL, last_error = NULL \
         RETURNING requested_at, purge_at",
    )
    .bind(user_id)
    .bind(i32::try_from(RETENTION_DAYS).unwrap_or(30))
    .fetch_one(&mut *conn)
    .await?;
    Ok((
        Deletion {
            requested_at,
            purge_at,
        },
        true,
    ))
}

/// Restores the account. Returns `false` when no request is pending or the time ran out.
pub async fn restore(conn: &mut PgConnection, user_id: Uuid) -> Result<bool> {
    let n = sqlx::query(
        "UPDATE account_deletions SET restored_at = now() \
         WHERE user_id = $1 AND restored_at IS NULL AND purged_at IS NULL AND purge_at > now()",
    )
    .bind(user_id)
    .execute(&mut *conn)
    .await?
    .rows_affected();
    Ok(n == 1)
}
