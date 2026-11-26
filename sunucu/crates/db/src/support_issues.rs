//! Anonymisation of the GitHub Issues of deleted accounts (T2.31e, migration 0251).
//!
//! The purge saves the Issue numbers on the deletion request before it deletes the support rows.
//! The worker removes a number only after GitHub accepted the edit.

use sqlx::{PgConnection, Row, types::Uuid};

use crate::Result;

/// A saved Issue number that still needs an anonymisation edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupportIssuePending {
    /// The deletion request that holds the number.
    pub request_id: Uuid,
    /// The GitHub Issue number.
    pub issue_number: i32,
    /// Failed attempts of the request.
    pub attempts: i32,
}

/// Saves the Issue numbers of the support requests of a due user. Call it before the purge.
pub async fn account_deletion_support_issues(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<i32> {
    Ok(
        sqlx::query_scalar("SELECT account_deletion_support_issues($1)")
            .bind(user_id)
            .fetch_one(&mut *conn)
            .await?,
    )
}

/// Lists saved Issue numbers of deleted accounts that still need an edit.
pub async fn support_issue_anonymize_pending(
    conn: &mut PgConnection,
    limit: i32,
) -> Result<Vec<SupportIssuePending>> {
    let rows = sqlx::query("SELECT * FROM support_issue_anonymize_pending($1)")
        .bind(limit)
        .fetch_all(&mut *conn)
        .await?;
    rows.iter()
        .map(|row| {
            Ok(SupportIssuePending {
                request_id: row.try_get("request_id")?,
                issue_number: row.try_get("issue_number")?,
                attempts: row.try_get("attempts")?,
            })
        })
        .collect()
}

/// Removes one Issue number after GitHub accepted the edit.
pub async fn support_issue_anonymize_done(
    conn: &mut PgConnection,
    request_id: Uuid,
    issue_number: i32,
) -> Result<bool> {
    Ok(
        sqlx::query_scalar("SELECT support_issue_anonymize_done($1, $2)")
            .bind(request_id)
            .bind(issue_number)
            .fetch_one(&mut *conn)
            .await?,
    )
}

/// Counts one failed edit for the request. Returns the new count.
pub async fn support_issue_anonymize_failed(
    conn: &mut PgConnection,
    request_id: Uuid,
) -> Result<Option<i32>> {
    Ok(
        sqlx::query_scalar("SELECT support_issue_anonymize_failed($1)")
            .bind(request_id)
            .fetch_one(&mut *conn)
            .await?,
    )
}
