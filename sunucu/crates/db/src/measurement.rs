//! Range check for `measurements.measured_at` (F2.1 finding 12).
//!
//! Partitions cover `measurements_legacy` (everything before 2026-01) and the months from
//! 2026-01 up to two months after today (the worker opens them ahead of time). A timestamp
//! outside what the partitions hold would fail with `23514`; a clock that was reset (an iSpindel
//! that sends 1970) would also land silently in the legacy partition. The API checks the value
//! first and answers with a field error.

use chrono::{DateTime, Duration, TimeZone, Utc};

/// The earliest accepted `measured_at`: before this date a device clock is taken as broken.
#[must_use]
pub fn earliest_measured_at() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0)
        .single()
        .unwrap_or(DateTime::<Utc>::MIN_UTC)
}

/// How far ahead of the server clock a measurement may be (clock skew of a device).
pub const MAX_FUTURE: Duration = Duration::hours(24);

/// Why a `measured_at` is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasuredAtError {
    /// Before [`earliest_measured_at`].
    TooOld,
    /// More than [`MAX_FUTURE`] after `now`.
    InFuture,
}

/// Checks `measured_at` against the accepted range at `now`.
///
/// # Errors
///
/// [`MeasuredAtError::TooOld`] or [`MeasuredAtError::InFuture`].
pub fn validate_measured_at(
    measured_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<(), MeasuredAtError> {
    if measured_at < earliest_measured_at() {
        Err(MeasuredAtError::TooOld)
    } else if measured_at > now + MAX_FUTURE {
        Err(MeasuredAtError::InFuture)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(y: i32, m: u32, d: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, 12, 0, 0).unwrap()
    }

    #[test]
    fn paper_records_of_2025_are_accepted_and_a_reset_clock_is_not() {
        let now = at(2026, 11, 14);
        assert_eq!(validate_measured_at(at(2025, 6, 1), now), Ok(()));
        assert_eq!(validate_measured_at(at(2026, 11, 15), now), Ok(()));
        assert_eq!(
            validate_measured_at(DateTime::UNIX_EPOCH, now),
            Err(MeasuredAtError::TooOld)
        );
        assert_eq!(
            validate_measured_at(at(2026, 11, 20), now),
            Err(MeasuredAtError::InFuture)
        );
    }
}
