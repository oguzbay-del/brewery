//! The engine's single public error surface (ADR-0004 madde 3 and 5).
//!
//! Wherever the Python reference raises (or its result would be NaN/inf) the engine
//! returns `Err`. There is no module-specific error type.

use core::fmt;

/// Engine error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// An input is non-finite, out of range, an unknown code, or makes the result undefined.
    InvalidInput {
        /// Parameter or concept that was rejected.
        field: &'static str,
        /// Why it was rejected.
        reason: &'static str,
    },
    /// A bounded iteration ended without converging.
    NoConvergence {
        /// The computation that did not converge.
        what: &'static str,
    },
    /// The requested target cannot be reached by any input.
    Unreachable {
        /// The target that cannot be reached.
        what: &'static str,
    },
    /// A series or list that must hold at least one element is empty.
    EmptyInput {
        /// The empty parameter.
        field: &'static str,
    },
    /// A divisor (rate, rate constant, dry matter) is zero.
    DivisionByZero {
        /// The divisor that is zero.
        field: &'static str,
    },
    /// A decimal computation overflowed.
    Overflow {
        /// The quantity that overflowed.
        what: &'static str,
    },
    /// A band or tier table is not strictly ascending, its open end is not last, or a rate is
    /// negative.
    InvalidBands,
}

impl Error {
    /// Shorthand for [`Error::InvalidInput`].
    #[must_use]
    pub const fn invalid(field: &'static str, reason: &'static str) -> Self {
        Self::InvalidInput { field, reason }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput { field, reason } => write!(f, "invalid input `{field}`: {reason}"),
            Self::NoConvergence { what } => write!(f, "{what} did not converge"),
            Self::Unreachable { what } => write!(f, "{what} is unreachable"),
            Self::EmptyInput { field } => write!(f, "`{field}` must not be empty"),
            Self::DivisionByZero { field } => write!(f, "`{field}` must not be 0"),
            Self::Overflow { what } => write!(f, "{what} overflowed"),
            Self::InvalidBands => {
                f.write_str("bands must be strictly ascending, open end last, rate >= 0")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Engine result.
pub type Result<T> = core::result::Result<T, Error>;

/// Rejects NaN and infinities.
pub fn ensure_finite(field: &'static str, x: f64) -> Result<()> {
    if x.is_finite() {
        Ok(())
    } else {
        Err(Error::invalid(field, "must be finite"))
    }
}

/// Rejects a non-finite or non-positive value.
pub fn ensure_positive(field: &'static str, x: f64) -> Result<()> {
    if x.is_finite() && x > 0.0 {
        Ok(())
    } else {
        Err(Error::invalid(field, "must be finite and > 0"))
    }
}

/// Rejects a non-finite or negative value.
pub fn ensure_non_negative(field: &'static str, x: f64) -> Result<()> {
    if x.is_finite() && x >= 0.0 {
        Ok(())
    } else {
        Err(Error::invalid(field, "must be finite and >= 0"))
    }
}

/// Returns `x` when finite, otherwise an error (for results of divisions and logs).
pub fn finite_result(what: &'static str, x: f64) -> Result<f64> {
    if x.is_finite() {
        Ok(x)
    } else {
        Err(Error::invalid(what, "result is not finite"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers_reject_non_finite() {
        assert!(ensure_finite("x", f64::NAN).is_err());
        assert!(ensure_finite("x", f64::INFINITY).is_err());
        assert!(ensure_finite("x", 1.0).is_ok());
        assert!(ensure_positive("x", 0.0).is_err());
        assert!(ensure_non_negative("x", -1.0).is_err());
        assert!(finite_result("y", f64::NAN).is_err());
    }

    #[test]
    fn display_names_the_field() {
        let e = Error::invalid("fg", "must be finite");
        assert_eq!(e.to_string(), "invalid input `fg`: must be finite");
    }
}
