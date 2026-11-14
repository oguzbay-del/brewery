//! Mandatory retention periods for Business mode records (F2.1 finding 11).
//!
//! The database holds the mechanism (`retention_until` and `legal_hold` columns, delete guards).
//! The periods are configuration. The defaults are placeholders, to be verified: stage 6 sets
//! the real values after a check of US (TTB) and UK (HMRC) rules. Do not take the defaults as
//! legal advice.

use crate::{Error, Result};

/// Variable that sets the US period in years.
pub const ENV_RETENTION_YEARS_US: &str = "RETENTION_YEARS_US";
/// Variable that sets the UK period in years.
pub const ENV_RETENTION_YEARS_UK: &str = "RETENTION_YEARS_UK";

/// Placeholder default for the US (to be verified).
const DEFAULT_YEARS_US: u32 = 3;
/// Placeholder default for the UK (to be verified).
const DEFAULT_YEARS_UK: u32 = 6;
/// Longest period accepted from the environment.
const MAX_YEARS: u32 = 100;

/// Retention periods in years, by workspace region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionPeriods {
    /// Region `us`.
    pub us_years: u32,
    /// Region `uk`.
    pub uk_years: u32,
}

impl Default for RetentionPeriods {
    fn default() -> Self {
        Self {
            us_years: DEFAULT_YEARS_US,
            uk_years: DEFAULT_YEARS_UK,
        }
    }
}

fn years(lookup: &dyn Fn(&str) -> Option<String>, name: &'static str, default: u32) -> Result<u32> {
    match lookup(name).filter(|v| !v.trim().is_empty()) {
        None => Ok(default),
        Some(raw) => {
            let value: u32 =
                raw.trim()
                    .parse()
                    .map_err(|e: std::num::ParseIntError| Error::InvalidEnv {
                        name,
                        reason: e.to_string(),
                    })?;
            if (1..=MAX_YEARS).contains(&value) {
                Ok(value)
            } else {
                Err(Error::InvalidEnv {
                    name,
                    reason: format!("must be between 1 and {MAX_YEARS}"),
                })
            }
        }
    }
}

impl RetentionPeriods {
    /// Reads the periods from `lookup` (the environment in production).
    ///
    /// # Errors
    ///
    /// [`Error::InvalidEnv`] for a value that is not a whole number between 1 and 100.
    pub fn from_lookup(lookup: &dyn Fn(&str) -> Option<String>) -> Result<Self> {
        Ok(Self {
            us_years: years(lookup, ENV_RETENTION_YEARS_US, DEFAULT_YEARS_US)?,
            uk_years: years(lookup, ENV_RETENTION_YEARS_UK, DEFAULT_YEARS_UK)?,
        })
    }

    /// The period for a workspace region (`us` or `uk`); `None` for another region.
    #[must_use]
    pub fn years_for(&self, region: &str) -> Option<u32> {
        match region {
            "us" => Some(self.us_years),
            "uk" => Some(self.uk_years),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_overrides() {
        let none = |_: &str| None;
        let p = RetentionPeriods::from_lookup(&none).unwrap();
        assert_eq!((p.years_for("us"), p.years_for("uk")), (Some(3), Some(6)));
        assert_eq!(p.years_for("de"), None);
        let set = |n: &str| (n == ENV_RETENTION_YEARS_UK).then(|| "7".to_owned());
        assert_eq!(RetentionPeriods::from_lookup(&set).unwrap().uk_years, 7);
    }

    #[test]
    fn bad_values_are_refused() {
        for bad in ["0", "abc", "101", "-1"] {
            let set = |n: &str| (n == ENV_RETENTION_YEARS_US).then(|| bad.to_owned());
            assert!(RetentionPeriods::from_lookup(&set).is_err(), "{bad}");
        }
    }
}
