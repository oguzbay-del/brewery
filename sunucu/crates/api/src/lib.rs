//! `api` service: HTTP router and `OpenAPI` document.

use axum::{Json, Router, routing::get};
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

/// Service name reported in logs and the health response.
pub const SERVICE_NAME: &str = "api";
/// Default listen port (override with `PORT`).
pub const DEFAULT_PORT: u16 = 8080;

/// Response of `GET /healthz`.
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// Always `"ok"` while the process is serving.
    pub status: String,
    /// Service name.
    pub service: String,
    /// Service version.
    pub version: String,
}

/// Response of `GET /v1/version`.
#[derive(Debug, Serialize, ToSchema)]
pub struct VersionResponse {
    /// API version (the `api` crate version).
    pub api: String,
    /// Calculation engine version; `null` until the engine is wired in.
    pub engine: Option<String>,
}

/// Machine-readable error detail.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    /// Stable error code (for clients to branch on).
    pub code: String,
    /// Human-readable message (English; not shown to end users).
    pub message: String,
}

/// Common error envelope returned by every `/v1` endpoint on failure.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Error detail.
    pub error: ErrorBody,
}

/// Liveness probe (unversioned).
#[utoipa::path(
    get,
    path = "/healthz",
    tag = "system",
    operation_id = "healthz",
    responses((status = 200, description = "Service is alive", body = HealthResponse))
)]
async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_owned(),
        service: SERVICE_NAME.to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
    })
}

/// Reports the API and engine versions.
#[utoipa::path(
    get,
    path = "/v1/version",
    tag = "system",
    operation_id = "getVersion",
    responses(
        (status = 200, description = "Version information", body = VersionResponse),
        (status = "default", description = "Error", body = ErrorResponse)
    )
)]
async fn version() -> Json<VersionResponse> {
    Json(VersionResponse {
        api: env!("CARGO_PKG_VERSION").to_owned(),
        engine: None,
    })
}

/// The `OpenAPI` document of the public API. Generated from code; never edited by hand.
#[derive(Debug, OpenApi)]
#[openapi(
    paths(healthz, version),
    components(schemas(HealthResponse, VersionResponse, ErrorResponse, ErrorBody)),
    tags((name = "system", description = "Service status and version"))
)]
pub struct ApiDoc;

/// Renders the `OpenAPI` document as deterministic pretty JSON with a trailing newline.
///
/// # Errors
/// Returns an error if serialization fails (never expected for this document).
pub fn openapi_json() -> serde_json::Result<String> {
    // Going through `Value` sorts object keys (serde_json without `preserve_order`).
    let value = serde_json::to_value(ApiDoc::openapi())?;
    let mut out = serde_json::to_string_pretty(&value)?;
    out.push('\n');
    Ok(out)
}

/// Builds the service router.
pub fn router() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/version", get(version))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn get_json(path: &str) -> (StatusCode, serde_json::Value) {
        let response = router()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn healthz_returns_ok() {
        let (status, json) = get_json("/healthz").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["status"], "ok");
        assert_eq!(json["service"], "api");
        assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn version_returns_api_version_and_null_engine() {
        let (status, json) = get_json("/v1/version").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["api"], env!("CARGO_PKG_VERSION"));
        assert!(json["engine"].is_null());
    }

    #[test]
    fn openapi_document_lists_paths_and_error_schema() {
        let doc: serde_json::Value = serde_json::from_str(&openapi_json().unwrap()).unwrap();
        assert!(doc["paths"]["/healthz"]["get"].is_object());
        assert!(doc["paths"]["/v1/version"]["get"].is_object());
        assert!(doc["components"]["schemas"]["ErrorResponse"].is_object());
        assert!(doc["components"]["schemas"]["ErrorBody"].is_object());
    }

    #[test]
    fn openapi_json_is_deterministic_and_newline_terminated() {
        let first = openapi_json().unwrap();
        assert_eq!(first, openapi_json().unwrap());
        assert!(first.ends_with("}\n"));
    }
}
