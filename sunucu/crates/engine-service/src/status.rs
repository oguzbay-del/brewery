//! Mapping of `engine::Error` to `tonic::Status` (ADR-0005 item 5), shared by all services.

use tonic::Status;
use tonic_types::{ErrorDetails, StatusExt};

/// Converts an engine error into a gRPC status.
///
/// `InvalidInput`, `InvalidBands`, `EmptyInput`, `DivisionByZero` and `Overflow` become
/// `INVALID_ARGUMENT` (with a `google.rpc.BadRequest` detail naming the field where known);
/// `NoConvergence` and `Unreachable` become `FAILED_PRECONDITION`.
#[must_use]
pub fn from_engine(err: &engine::Error) -> Status {
    use engine::Error as E;
    let message = err.to_string();
    match err {
        E::InvalidInput { field, reason } => bad_request(&message, field, reason),
        E::EmptyInput { field } | E::DivisionByZero { field } => {
            bad_request(&message, field, &message)
        }
        E::InvalidBands => bad_request(&message, "bands", &message),
        E::Overflow { what } => bad_request(&message, what, &message),
        E::NoConvergence { .. } | E::Unreachable { .. } => Status::failed_precondition(message),
    }
}

/// `INVALID_ARGUMENT` with a `BadRequest` field violation.
#[must_use]
pub fn bad_request(message: &str, field: &str, description: &str) -> Status {
    Status::with_error_details(
        tonic::Code::InvalidArgument,
        message,
        ErrorDetails::with_bad_request_violation(field, description),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[test]
    fn maps_codes_per_adr_0005() {
        let invalid = [
            engine::Error::invalid("x", "bad"),
            engine::Error::InvalidBands,
            engine::Error::EmptyInput { field: "f" },
            engine::Error::DivisionByZero { field: "f" },
            engine::Error::Overflow { what: "w" },
        ];
        for e in &invalid {
            assert_eq!(from_engine(e).code(), Code::InvalidArgument, "{e}");
        }
        for e in [
            engine::Error::NoConvergence { what: "w" },
            engine::Error::Unreachable { what: "w" },
        ] {
            assert_eq!(from_engine(&e).code(), Code::FailedPrecondition);
        }
    }

    #[test]
    fn invalid_input_carries_field_detail() {
        let s = from_engine(&engine::Error::invalid("sg", "must be finite"));
        let details = s.get_error_details();
        let bad = details.bad_request().unwrap();
        assert_eq!(bad.field_violations[0].field, "sg");
    }
}
