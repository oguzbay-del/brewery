//! RFC 9457 problem details and the error code list (ADR-0008 rule 3).

use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Request},
    http::{HeaderName, HeaderValue, StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use utoipa::ToSchema;

tokio::task_local! {
    /// The id of the request being served (`X-Request-Id`), set by the request-id layer.
    pub static REQUEST_ID: String;
}

/// The request id of the current task, if inside a request.
#[must_use]
pub fn current_request_id() -> Option<String> {
    REQUEST_ID.try_with(Clone::clone).ok()
}

/// Content type of problem responses.
pub const PROBLEM_CONTENT_TYPE: &str = "application/problem+json";

/// Every error code the API returns. The single list: the `OpenAPI` document and the text
/// catalog are generated from it (ADR-0008 rule 3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// 400: JSON cannot be parsed, wrong type.
    MalformedRequest,
    /// 401: no session, expired or closed.
    Unauthenticated,
    /// 403: member without permission; writing a hidden field.
    Forbidden,
    /// 403: second factor (passkey) required.
    StepUpRequired,
    /// 403: the subscription or trial does not unlock this feature.
    EntitlementRequired,
    /// 404: no such record, deleted, or in another workspace.
    NotFound,
    /// 409: business rule conflict.
    Conflict,
    /// 409: a request with the same idempotency key is still being processed.
    IdempotencyInProgress,
    /// 412: `If-Match` does not match the current version.
    VersionMismatch,
    /// 413: body limit exceeded.
    PayloadTooLarge,
    /// 415: wrong `Content-Type`.
    UnsupportedMediaType,
    /// 422: field validation.
    ValidationFailed,
    /// 422: same idempotency key with a different body.
    IdempotencyMismatch,
    /// 422: the calculation engine rejected the input.
    CalculationFailed,
    /// 426: app version below the minimum supported.
    UpgradeRequired,
    /// 428: `If-Match` or `Idempotency-Key` required but missing.
    PreconditionRequired,
    /// 429: rate limit.
    RateLimited,
    /// 500: unexpected failure.
    Internal,
    /// 503: a dependency (engine, database) does not answer.
    Unavailable,
}

impl ErrorCode {
    /// All codes, in table order.
    pub const ALL: [Self; 19] = [
        Self::MalformedRequest,
        Self::Unauthenticated,
        Self::Forbidden,
        Self::StepUpRequired,
        Self::EntitlementRequired,
        Self::NotFound,
        Self::Conflict,
        Self::IdempotencyInProgress,
        Self::VersionMismatch,
        Self::PayloadTooLarge,
        Self::UnsupportedMediaType,
        Self::ValidationFailed,
        Self::IdempotencyMismatch,
        Self::CalculationFailed,
        Self::UpgradeRequired,
        Self::PreconditionRequired,
        Self::RateLimited,
        Self::Internal,
        Self::Unavailable,
    ];

    /// HTTP status of the code.
    #[must_use]
    pub const fn status(self) -> StatusCode {
        match self {
            Self::MalformedRequest => StatusCode::BAD_REQUEST,
            Self::Unauthenticated => StatusCode::UNAUTHORIZED,
            Self::Forbidden | Self::StepUpRequired | Self::EntitlementRequired => {
                StatusCode::FORBIDDEN
            }
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict | Self::IdempotencyInProgress => StatusCode::CONFLICT,
            Self::VersionMismatch => StatusCode::PRECONDITION_FAILED,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::ValidationFailed | Self::IdempotencyMismatch | Self::CalculationFailed => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            Self::UpgradeRequired => StatusCode::UPGRADE_REQUIRED,
            Self::PreconditionRequired => StatusCode::PRECONDITION_REQUIRED,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    /// The stable `snake_case` code string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MalformedRequest => "malformed_request",
            Self::Unauthenticated => "unauthenticated",
            Self::Forbidden => "forbidden",
            Self::StepUpRequired => "step_up_required",
            Self::EntitlementRequired => "entitlement_required",
            Self::NotFound => "not_found",
            Self::Conflict => "conflict",
            Self::IdempotencyInProgress => "idempotency_in_progress",
            Self::VersionMismatch => "version_mismatch",
            Self::PayloadTooLarge => "payload_too_large",
            Self::UnsupportedMediaType => "unsupported_media_type",
            Self::ValidationFailed => "validation_failed",
            Self::IdempotencyMismatch => "idempotency_mismatch",
            Self::CalculationFailed => "calculation_failed",
            Self::UpgradeRequired => "upgrade_required",
            Self::PreconditionRequired => "precondition_required",
            Self::RateLimited => "rate_limited",
            Self::Internal => "internal",
            Self::Unavailable => "unavailable",
        }
    }

    /// English title (not shown to end users).
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::MalformedRequest => "Malformed request",
            Self::Unauthenticated => "Unauthenticated",
            Self::Forbidden => "Forbidden",
            Self::StepUpRequired => "Second factor required",
            Self::EntitlementRequired => "Subscription required",
            Self::NotFound => "Not found",
            Self::Conflict => "Conflict",
            Self::IdempotencyInProgress => "Request with this idempotency key is in progress",
            Self::VersionMismatch => "Version mismatch",
            Self::PayloadTooLarge => "Payload too large",
            Self::UnsupportedMediaType => "Unsupported media type",
            Self::ValidationFailed => "Validation failed",
            Self::IdempotencyMismatch => "Idempotency key reused with a different request",
            Self::CalculationFailed => "Calculation failed",
            Self::UpgradeRequired => "Upgrade required",
            Self::PreconditionRequired => "Precondition required",
            Self::RateLimited => "Rate limited",
            Self::Internal => "Internal error",
            Self::Unavailable => "Service unavailable",
        }
    }

    /// Relative problem `type` address.
    #[must_use]
    pub fn type_uri(self) -> String {
        format!("/errors/{}", self.as_str())
    }
}

/// One field-level problem (`errors[]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct FieldIssue {
    /// JSON Pointer (RFC 6901) into the request body; a query parameter name for query errors.
    pub field: String,
    /// Stable code of the problem (`out_of_range`, ...).
    pub code: String,
    /// Parameters for the user text (limits and the like).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl FieldIssue {
    /// A problem without parameters.
    #[must_use]
    pub fn new(field: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            code: code.into(),
            params: None,
        }
    }

    /// Adds parameters.
    #[must_use]
    pub fn with_params(mut self, params: Value) -> Self {
        self.params = Some(params);
        self
    }
}

/// RFC 9457 problem details body (`application/problem+json`).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProblemDetails {
    /// Relative address of the problem type (`/errors/<code>`).
    #[serde(rename = "type")]
    pub type_: String,
    /// English summary; not shown to end users.
    pub title: String,
    /// HTTP status.
    pub status: u16,
    /// Stable code clients branch on.
    pub code: ErrorCode,
    /// Request id (`X-Request-Id`).
    pub request_id: Option<String>,
    /// Field-level problems.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<FieldIssue>,
    /// Current state of the record (`version_mismatch` only; shape depends on the resource).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current: Option<Value>,
}

/// An error that renders as a problem response.
#[derive(Debug, Clone)]
pub struct ApiError {
    code: ErrorCode,
    errors: Vec<FieldIssue>,
    current: Option<Value>,
    headers: Vec<(HeaderName, HeaderValue)>,
}

/// Convenience result alias.
pub type ApiResult<T> = Result<T, ApiError>;

impl ApiError {
    /// An error with just a code.
    #[must_use]
    pub const fn new(code: ErrorCode) -> Self {
        Self {
            code,
            errors: Vec::new(),
            current: None,
            headers: Vec::new(),
        }
    }

    /// `validation_failed` with the given problems.
    #[must_use]
    pub fn validation(errors: Vec<FieldIssue>) -> Self {
        Self::new(ErrorCode::ValidationFailed).with_errors(errors)
    }

    /// `412 version_mismatch` carrying the current record.
    #[must_use]
    pub fn version_mismatch(current: Value) -> Self {
        let mut e = Self::new(ErrorCode::VersionMismatch);
        e.current = Some(current);
        e
    }

    /// `503 unavailable` with `Retry-After`.
    #[must_use]
    pub fn unavailable(retry_after_secs: u64) -> Self {
        Self::new(ErrorCode::Unavailable).with_retry_after(retry_after_secs)
    }

    /// Adds field problems.
    #[must_use]
    pub fn with_errors(mut self, errors: Vec<FieldIssue>) -> Self {
        self.errors = errors;
        self
    }

    /// Adds a `Retry-After` header (seconds).
    #[must_use]
    pub fn with_retry_after(self, secs: u64) -> Self {
        self.with_header(header::RETRY_AFTER, HeaderValue::from(secs))
    }

    /// Adds a response header.
    #[must_use]
    pub fn with_header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.headers.push((name, value));
        self
    }

    /// The code.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    /// The field problems.
    #[must_use]
    pub fn errors(&self) -> &[FieldIssue] {
        &self.errors
    }

    /// The body, with the request id of the current task.
    #[must_use]
    pub fn problem(&self) -> ProblemDetails {
        ProblemDetails {
            type_: self.code.type_uri(),
            title: self.code.title().to_owned(),
            status: self.code.status().as_u16(),
            code: self.code,
            request_id: current_request_id(),
            errors: self.errors.clone(),
            current: self.current.clone(),
        }
    }
}

impl From<ErrorCode> for ApiError {
    fn from(code: ErrorCode) -> Self {
        Self::new(code)
    }
}

impl From<db::Error> for ApiError {
    fn from(e: db::Error) -> Self {
        match e {
            db::Error::Sqlx(e) => e.into(),
            other => {
                tracing::error!(error = %other, "database error");
                Self::new(ErrorCode::Internal)
            }
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed | sqlx::Error::Io(_) => {
                tracing::error!(error = %e, "database unavailable");
                Self::unavailable(5)
            }
            other => {
                // Detail goes to the log only (ADR-0008 rule 3.4).
                tracing::error!(error = %other, "database error");
                Self::new(ErrorCode::Internal)
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.code.status();
        let body = self.problem();
        let trace_id = crate::request::current_trace_id();
        if status.is_server_error() {
            tracing::error!(code = self.code.as_str(), request_id = ?body.request_id, trace_id = ?trace_id, "request failed");
        } else {
            tracing::info!(code = self.code.as_str(), request_id = ?body.request_id, trace_id = ?trace_id, "request rejected");
        }
        let bytes = serde_json::to_vec(&body).unwrap_or_default();
        let mut response = (status, bytes).into_response();
        let headers = response.headers_mut();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(PROBLEM_CONTENT_TYPE),
        );
        for (name, value) in self.headers {
            headers.insert(name, value);
        }
        response
    }
}

fn rejection_to_error(status: StatusCode, detail: &str) -> ApiError {
    let code = match status {
        StatusCode::PAYLOAD_TOO_LARGE => ErrorCode::PayloadTooLarge,
        StatusCode::UNSUPPORTED_MEDIA_TYPE => ErrorCode::UnsupportedMediaType,
        _ => ErrorCode::MalformedRequest,
    };
    tracing::debug!(detail, "request rejected by extractor");
    ApiError::new(code)
}

/// JSON body extractor with problem responses: 400 `malformed_request` (syntax or type),
/// 413 `payload_too_large`, 415 `unsupported_media_type`.
#[derive(Debug)]
pub struct ApiJson<T>(pub T);

impl<S: Send + Sync, T: DeserializeOwned> FromRequest<S> for ApiJson<T> {
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(v)) => Ok(Self(v)),
            Err(r) => Err(rejection_to_error(r.status(), &r.body_text())),
        }
    }
}

/// JSON Merge Patch body (RFC 7396, `application/merge-patch+json`; ADR-0008 rule 1.4).
#[derive(Debug)]
pub struct MergePatch<T>(pub T);

impl<S: Send + Sync, T: DeserializeOwned> FromRequest<S> for MergePatch<T> {
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let is_patch = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| {
                v.split(';').next().is_some_and(|m| {
                    m.trim()
                        .eq_ignore_ascii_case("application/merge-patch+json")
                })
            });
        if !is_patch {
            return Err(ApiError::new(ErrorCode::UnsupportedMediaType));
        }
        let (parts, body) = req.into_parts();
        let bytes = axum::body::Bytes::from_request(Request::from_parts(parts, body), state)
            .await
            .map_err(|r| rejection_to_error(r.status(), &r.body_text()))?;
        serde_json::from_slice(&bytes)
            .map(Self)
            .map_err(|e| rejection_to_error(StatusCode::BAD_REQUEST, &e.to_string()))
    }
}

/// Query extractor with a problem response (400 `malformed_request`).
#[derive(Debug)]
pub struct ApiQuery<T>(pub T);

impl<S: Send + Sync, T: DeserializeOwned> FromRequestParts<S> for ApiQuery<T> {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        axum::extract::Query::<T>::from_request_parts(parts, state)
            .await
            .map(|q| Self(q.0))
            .map_err(|r| rejection_to_error(r.status(), &r.body_text()))
    }
}

/// Fallback for unknown paths and wrong methods: `404 not_found`.
pub async fn not_found() -> ApiError {
    ApiError::new(ErrorCode::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_have_unique_strings_and_expected_statuses() {
        let mut seen = std::collections::HashSet::new();
        for c in ErrorCode::ALL {
            assert!(seen.insert(c.as_str()), "{c:?}");
            assert_eq!(serde_json::to_value(c).unwrap(), c.as_str());
        }
        assert_eq!(ErrorCode::VersionMismatch.status(), 412);
        assert_eq!(ErrorCode::UpgradeRequired.status(), 426);
        assert_eq!(ErrorCode::PreconditionRequired.status(), 428);
    }
}
