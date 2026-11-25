//! Retention of external events (T2.39, F2.1 decision 11).
//!
//! The payload of an event that nobody processed can hold customer content. After
//! [`PAYLOAD_DAYS`] days the worker clears it. After [`ID_MONTHS`] months it deletes the row,
//! and with it the delivery ID that protects against repeated deliveries.

use chrono::{DateTime, Duration, Months, Utc};
use sqlx::PgPool;

use crate::{Error, Result};

/// Age at which the payload of an unprocessed event is cleared.
pub const PAYLOAD_DAYS: i64 = 30;
/// Age at which an event row (its delivery ID) is deleted.
pub const ID_MONTHS: u32 = 13;

/// Days after resolution before a support request is closed (T2.43).
pub const CLOSE_AFTER_DAYS: i64 = 14;

/// Closes resolved support requests older than `CLOSE_AFTER_DAYS` days and queues their
/// GitHub synchronization events. Running it again changes no rows.
pub async fn close_resolved_support_requests(pool: &PgPool, now: DateTime<Utc>) -> Result<u64> {
    db::maintenance::close_resolved_support_requests(pool, now - Duration::days(CLOSE_AFTER_DAYS))
        .await
}

/// Days a soft-deleted recipe, batch, ingredient or equipment record can be restored (T2.37).
pub const SOFT_DELETE_DAYS: i64 = 30;

/// Deletes for good the records that were soft-deleted more than [`SOFT_DELETE_DAYS`] days
/// before `now`. A record that is in use stays (F2.1 decision 9).
pub async fn purge_soft_deletes(
    pool: &PgPool,
    now: DateTime<Utc>,
) -> Result<Vec<db::maintenance::PurgeCount>> {
    Ok(
        db::maintenance::purge_expired_soft_deletes(pool, now - Duration::days(SOFT_DELETE_DAYS))
            .await?,
    )
}

/// Rows touched by one run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionReport {
    /// Payloads cleared.
    pub payloads_cleared: u64,
    /// Rows deleted.
    pub events_deleted: u64,
}

/// Applies both rules as of `now`. Running it again changes nothing more.
pub async fn run(pool: &PgPool, now: DateTime<Utc>) -> Result<RetentionReport> {
    let id_cutoff = now
        .checked_sub_months(Months::new(ID_MONTHS))
        .ok_or_else(|| Error::Format("retention date out of range".into()))?;
    // Delete first: a row that is due for deletion needs no clearing.
    let events_deleted = db::maintenance::delete_old_external_events(pool, id_cutoff).await?;
    let payloads_cleared = db::maintenance::clear_stale_external_payloads(
        pool,
        now - Duration::days(PAYLOAD_DAYS),
        now,
    )
    .await?;
    Ok(RetentionReport {
        payloads_cleared,
        events_deleted,
    })
}
