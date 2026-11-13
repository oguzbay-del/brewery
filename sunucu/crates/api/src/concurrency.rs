//! `ETag`, `If-Match` and `If-None-Match` helpers (ADR-0008 rule 5).
//!
//! Every record response carries a strong `ETag: "<version>"` (the record's `version` column).
//! What a write does with `If-Match` depends on the resource's [`Concurrency`] mode:
//! `optimistic` requires it (`428` when missing, `412` with the current record on mismatch),
//! `field_lww` accepts it optionally, `append` records are never updated.

use axum::{
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::Value;

use crate::error::{ApiError, ApiResult, ErrorCode, FieldIssue};

/// How a resource type handles concurrent writes (`x-concurrency` in the `OpenAPI` document).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Concurrency {
    /// `If-Match` required; a stale version is a conflict the user resolves.
    Optimistic,
    /// `If-Match` optional; last writer wins per field (history keeps the old value).
    FieldLww,
    /// Records are only appended; corrections are new records.
    Append,
}

impl Concurrency {
    /// The `x-concurrency` value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Optimistic => "optimistic",
            Self::FieldLww => "field_lww",
            Self::Append => "append",
        }
    }
}

/// `"<version>"` as a header value.
#[must_use]
pub fn etag_value(version: i64) -> HeaderValue {
    HeaderValue::from_str(&format!("\"{version}\""))
        .unwrap_or_else(|_| HeaderValue::from_static("\"0\""))
}

/// Parses one strong entity tag (`"12"`); weak tags and anything else give `None`.
#[must_use]
pub fn parse_etag(raw: &str) -> Option<i64> {
    raw.trim()
        .strip_prefix('"')?
        .strip_suffix('"')?
        .parse()
        .ok()
}

fn invalid_header(name: &'static str) -> ApiError {
    ApiError::new(ErrorCode::MalformedRequest)
        .with_errors(vec![FieldIssue::new(name, "invalid_header")])
}

/// Versions listed in `If-Match`; `None` when the header is absent.
///
/// # Errors
/// `400 malformed_request` for weak tags, `*` or anything that is not a list of version tags.
pub fn if_match(headers: &HeaderMap) -> ApiResult<Option<Vec<i64>>> {
    let Some(raw) = headers.get(header::IF_MATCH) else {
        return Ok(None);
    };
    let raw = raw.to_str().map_err(|_| invalid_header("If-Match"))?;
    raw.split(',')
        .map(|tag| parse_etag(tag).ok_or_else(|| invalid_header("If-Match")))
        .collect::<ApiResult<Vec<_>>>()
        .map(Some)
}

/// Applies the write precondition for `mode` against the record's `current_version`.
/// `current` renders the record for the `412` body and is only called on a mismatch.
///
/// # Errors
/// `428 precondition_required` (optimistic, header missing) or `412 version_mismatch` with the
/// current record in `current`.
pub fn check_if_match(
    mode: Concurrency,
    headers: &HeaderMap,
    current_version: i64,
    current: impl FnOnce() -> Value,
) -> ApiResult<()> {
    if mode == Concurrency::Append {
        return Ok(());
    }
    match if_match(headers)? {
        None if mode == Concurrency::Optimistic => {
            Err(ApiError::new(ErrorCode::PreconditionRequired))
        }
        None => Ok(()),
        Some(versions) if versions.contains(&current_version) => Ok(()),
        Some(_) => Err(ApiError::version_mismatch(current())),
    }
}

/// Whether `If-None-Match` says the client already holds `version` (answer `304`).
#[must_use]
pub fn not_modified(headers: &HeaderMap, version: i64) -> bool {
    headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|raw| {
            raw.split(',')
                .map(str::trim)
                .any(|tag| tag == "*" || parse_etag(tag) == Some(version))
        })
}

/// `200`/`201` JSON response of a record with its `ETag` (and `Cache-Control: private, no-cache`).
pub fn record_response<T: Serialize>(status: StatusCode, version: i64, body: &T) -> Response {
    let mut response = (status, axum::Json(body)).into_response();
    let headers = response.headers_mut();
    headers.insert(header::ETAG, etag_value(version));
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-cache"),
    );
    response
}

/// `GET` of a record: `304` when `If-None-Match` matches, else the record with its `ETag`.
pub fn conditional_get<T: Serialize>(headers: &HeaderMap, version: i64, body: &T) -> Response {
    if not_modified(headers, version) {
        let mut response = StatusCode::NOT_MODIFIED.into_response();
        let h = response.headers_mut();
        h.insert(header::ETAG, etag_value(version));
        h.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, no-cache"),
        );
        response
    } else {
        record_response(StatusCode::OK, version, body)
    }
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderName;

    use super::*;

    fn headers(name: HeaderName, value: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(name, value.parse().unwrap());
        h
    }

    #[test]
    fn etag_roundtrip_and_rejects_weak() {
        assert_eq!(parse_etag(&format!("{:?}", "7")), Some(7));
        assert_eq!(parse_etag("W/\"7\""), None);
        assert_eq!(parse_etag("7"), None);
    }

    #[test]
    fn modes() {
        let none = HeaderMap::new();
        let stale = headers(header::IF_MATCH, "\"3\"");
        let fresh = headers(header::IF_MATCH, "\"4\"");
        let cur = || serde_json::json!({"version": 4});
        let code = |r: ApiResult<()>| r.unwrap_err().code();
        assert_eq!(
            code(check_if_match(Concurrency::Optimistic, &none, 4, cur)),
            ErrorCode::PreconditionRequired
        );
        assert_eq!(
            code(check_if_match(Concurrency::Optimistic, &stale, 4, cur)),
            ErrorCode::VersionMismatch
        );
        assert!(check_if_match(Concurrency::Optimistic, &fresh, 4, cur).is_ok());
        assert!(check_if_match(Concurrency::FieldLww, &none, 4, cur).is_ok());
        assert_eq!(
            code(check_if_match(Concurrency::FieldLww, &stale, 4, cur)),
            ErrorCode::VersionMismatch
        );
        assert!(check_if_match(Concurrency::Append, &stale, 4, cur).is_ok());
        let star = headers(header::IF_MATCH, "*");
        assert_eq!(
            code(check_if_match(Concurrency::Optimistic, &star, 4, cur)),
            ErrorCode::MalformedRequest
        );
    }

    #[test]
    fn if_none_match() {
        assert!(not_modified(
            &headers(header::IF_NONE_MATCH, "\"2\", \"5\""),
            5
        ));
        assert!(!not_modified(&headers(header::IF_NONE_MATCH, "\"2\""), 5));
        assert!(not_modified(&headers(header::IF_NONE_MATCH, "*"), 5));
        assert!(!not_modified(&HeaderMap::new(), 5));
    }
}
