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
