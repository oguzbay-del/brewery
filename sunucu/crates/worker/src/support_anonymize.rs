//! Anonymises the GitHub Issues of deleted accounts (T2.31e).
//!
//! The purge job saves the Issue numbers on the deletion request (migration 0251). [`run`] edits
//! the Issue title, the Issue body and each customer comment to a fixed text and closes the Issue. It does not
//! delete the Issue (owner decision, 2026-11-24). A failed Issue stays saved and the next hourly
//! tick tries again. A failure does not stop the purge of other data or of other Issues.

use serde_json::{Value, json};
use sqlx::PgPool;
use support::{REMOVED_TEXT, github::GitHub, is_customer_comment};

use crate::Result;

const BATCH_LIMIT: i32 = 100;

/// Counts of one anonymisation run.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AnonymizeReport {
    /// Issues that GitHub accepted.
    pub anonymized: u32,
    /// Issues that failed. They are retried on the next run.
    pub retry_later: u32,
    /// Issues that were not tried because GitHub is not configured.
    pub skipped: u32,
}

impl AnonymizeReport {
    /// The summary as JSON for the maintenance detail.
    #[must_use]
    pub fn to_json(self) -> Value {
        json!({
            "anonymized": self.anonymized,
            "retry_later": self.retry_later,
            "skipped": self.skipped,
        })
    }
}

async fn anonymize_issue(github: &GitHub, issue: i64) -> std::result::Result<(), String> {
    let comments = github
        .list_comments(issue)
        .await
        .map_err(|error| error.to_string())?;
    for (id, body) in comments {
        if body != REMOVED_TEXT && is_customer_comment(&body) {
            github
                .edit_comment(id, REMOVED_TEXT)
                .await
                .map_err(|error| error.to_string())?;
        }
    }
    github
        .replace_body_and_close(issue, REMOVED_TEXT)
        .await
        .map_err(|error| error.to_string())
}

/// Anonymises the saved Issues of deleted accounts.
pub async fn run(pool: &PgPool, github: Option<&GitHub>) -> Result<AnonymizeReport> {
    let mut conn = pool.acquire().await.map_err(db::Error::from)?;
    let pending = db::support_issue_anonymize_pending(&mut conn, BATCH_LIMIT).await?;
    let mut report = AnonymizeReport::default();
    let Some(github) = github else {
        report.skipped = u32::try_from(pending.len()).unwrap_or(u32::MAX);
        if report.skipped > 0 {
            tracing::warn!(
                skipped = report.skipped,
                "GitHub settings are missing; support Issues of deleted accounts are not anonymized"
            );
        }
        return Ok(report);
    };
    for item in pending {
        match anonymize_issue(github, i64::from(item.issue_number)).await {
            Ok(()) => {
                db::support_issue_anonymize_done(&mut conn, item.request_id, item.issue_number)
                    .await?;
                report.anonymized += 1;
            }
            Err(error) => {
                let attempts = db::support_issue_anonymize_failed(&mut conn, item.request_id)
                    .await?
                    .unwrap_or(item.attempts);
                tracing::warn!(
                    issue = item.issue_number,
                    attempts,
                    %error,
                    "Issue anonymization failed; will retry"
                );
                report.retry_later += 1;
            }
        }
    }
    Ok(report)
}
