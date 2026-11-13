//! Shared request and result checks for the service handlers.

use tonic::Status;

use crate::status::bad_request;

/// Requires a finite value.
pub(crate) fn finite(field: &str, v: f64) -> Result<f64, Status> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(bad_request(
            &format!("{field} must be finite"),
            field,
            "must be finite",
        ))
    }
}

/// Requires a finite, non-negative value.
pub(crate) fn non_negative(field: &str, v: f64) -> Result<f64, Status> {
    finite(field, v)?;
    if v < 0.0 {
        return Err(bad_request(
            &format!("{field} must not be negative"),
            field,
            "must not be negative",
        ));
    }
    Ok(v)
}

/// Fails when a computed output is not finite.
pub(crate) fn out(field: &str, v: f64) -> Result<f64, Status> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(bad_request(
            &format!("{field} result is not finite"),
            field,
            "result is not finite",
        ))
    }
}
