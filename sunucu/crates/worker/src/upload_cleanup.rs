//! Cleanup of deleted and incomplete upload objects (T2.44).

use std::collections::HashSet;

use chrono::{DateTime, Duration, Utc};
use db::maintenance::{self, PurgeableUpload};
use sqlx::{PgPool, types::Uuid};
use storage::ObjectStore;

use crate::Result;

/// An upload that was processed successfully.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UploadCleanupReport {
    /// Completed uploads that were already soft-deleted.
    pub deleted: u64,
    /// Uploads that never reached the complete step.
    pub incomplete: u64,
    /// Object-store deletions that failed and will be retried on the next run.
    pub failed: u64,
}

pub const INCOMPLETE_AFTER_HOURS: i64 = 24;
const BATCH_SIZE: usize = 200;
const MAX_ROWS_PER_RUN: usize = 5_000;

/// Deletes the bytes of soft-deleted uploads and uploads that stayed incomplete for 24 hours.
///
/// A failed object deletion is not marked in the database. It is skipped for the rest of this
/// run and will be retried on the next hourly tick. The run processes at most 5,000 rows.
pub async fn run(
    pool: &PgPool,
    store: &dyn ObjectStore,
    now: DateTime<Utc>,
) -> Result<UploadCleanupReport> {
    let incomplete_before = now - Duration::hours(INCOMPLETE_AFTER_HOURS);
    let mut report = UploadCleanupReport::default();
    let mut attempted = 0_usize;

    for workspace_id in maintenance::list_workspace_ids(pool).await? {
        let mut failed_rows: HashSet<(String, Uuid)> = HashSet::new();

        while attempted < MAX_ROWS_PER_RUN {
            let limit = i32::try_from(BATCH_SIZE.min(MAX_ROWS_PER_RUN - attempted))
                .map_err(|_| crate::Error::Format("upload cleanup batch size exceeds i32".into()))?;
            let rows =
                maintenance::list_purgeable_uploads(pool, workspace_id, incomplete_before, limit)
                    .await?;
            let pending: Vec<PurgeableUpload> = rows
                .into_iter()
                .filter(|row| !failed_rows.contains(&(row.kind.clone(), row.id)))
                .collect();

            if pending.is_empty() {
                break;
            }

            for row in pending {
                if attempted >= MAX_ROWS_PER_RUN {
                    break;
                }
                attempted += 1;
                if store.delete(&row.object_key).await.is_err() {
                    report.failed = report.failed.saturating_add(1);
                    failed_rows.insert((row.kind, row.id));
                    tracing::warn!("upload object deletion failed; will retry next run");
                    continue;
                }

                let marked =
                    maintenance::mark_upload_purged(pool, workspace_id, &row.kind, row.id).await?;
                if !marked {
                    report.failed = report.failed.saturating_add(1);
                    failed_rows.insert((row.kind, row.id));
                    tracing::warn!("upload row could not be marked as purged; will retry next run");
                    continue;
                }
                if row.incomplete {
                    report.incomplete = report.incomplete.saturating_add(1);
                } else {
                    report.deleted = report.deleted.saturating_add(1);
                }
            }
        }
    }

    Ok(report)
}
