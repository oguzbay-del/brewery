//! Endpoints that need no sign-in: health, readiness, version, field definitions, client
//! requirements. Operation ids follow `<resource>_<action>` (ADR-0008 rule 12.2).

use std::{sync::OnceLock, time::Duration};

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use utoipa::ToSchema;

use crate::{
    AppState, SERVICE_NAME,
    app_version::ClientRequirements,
    error::{ApiError, ApiResult},
};

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

/// Response of `GET /readyz`.
#[derive(Debug, Serialize, ToSchema)]
pub struct ReadyResponse {
    /// `"ok"` when the database answers.
    pub status: String,
    /// Database state: `"ok"`.
    pub database: String,
    /// Redis state: `"ok"`, `"unavailable"` or `"disabled"`. Redis loss never fails readiness.
    pub redis: String,
}

/// Response of `GET /v1/version`.
#[derive(Debug, Serialize, ToSchema)]
pub struct VersionResponse {
    /// API version (the `api` crate version).
    pub api: String,
    /// Calculation engine version; `null` until the engine is wired in.
    pub engine: Option<String>,
}

/// Liveness probe (unversioned).
#[utoipa::path(
    get,
    path = "/healthz",
    tag = "system",
    operation_id = "health_check",
    responses((
        status = 200,
        description = "Service is alive",
        body = HealthResponse,
        example = json!({"status": "ok", "service": "api", "version": "0.1.0"})
    ))
)]
pub async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_owned(),
        service: SERVICE_NAME.to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
    })
}

/// Readiness probe (unversioned): the database must answer.
#[utoipa::path(
    get,
    path = "/readyz",
    tag = "system",
    operation_id = "health_ready",
    responses((
        status = 200,
        description = "Ready to serve",
        body = ReadyResponse,
        example = json!({"status": "ok", "database": "ok", "redis": "ok"})
    ))
)]
pub async fn readyz(State(state): State<AppState>) -> ApiResult<Json<ReadyResponse>> {
    let probe = tokio::time::timeout(
        Duration::from_secs(2),
        sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&state.db),
    )
    .await;
    match probe {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => return Err(e.into()),
        Err(_) => return Err(ApiError::unavailable(5)),
    }
    let redis = if !state.redis.is_configured() {
        "disabled"
    } else if state.redis.ping().await {
        "ok"
    } else {
        "unavailable"
    };
    Ok(Json(ReadyResponse {
        status: "ok".to_owned(),
        database: "ok".to_owned(),
        redis: redis.to_owned(),
    }))
}

/// Reports the API and engine versions.
#[utoipa::path(
    get,
    path = "/v1/version",
    tag = "system",
    operation_id = "version_get",
    responses((
        status = 200,
        description = "Version information",
        body = VersionResponse,
        example = json!({"api": "0.1.0", "engine": null})
    ))
)]
pub async fn version() -> Json<VersionResponse> {
    Json(VersionResponse {
        api: env!("CARGO_PKG_VERSION").to_owned(),
        engine: None,
    })
}

/// Lowest supported client versions; readable before sign-in (ADR-0008 rule 8.3).
#[utoipa::path(
    get,
    path = "/v1/meta/client-requirements",
    tag = "meta",
    operation_id = "meta_client_requirements_get",
    responses((
        status = 200,
        description = "Minimum supported client versions",
        body = ClientRequirements,
        example = json!({"min_ios_version": "1.4.0", "min_web_version": null})
    ))
)]
pub async fn client_requirements(State(state): State<AppState>) -> Json<ClientRequirements> {
    Json(ClientRequirements::from(&state.config.min_versions))
}

/// Seconds clients may reuse the field definitions without asking again.
const FIELDS_MAX_AGE_SECS: u32 = 3600;

fn fields_etag() -> &'static str {
    static ETAG: OnceLock<String> = OnceLock::new();
    ETAG.get_or_init(|| {
        let digest = Sha256::digest(common::catalog::RAW.as_bytes());
        let hex = digest.iter().take(8).fold(String::new(), |mut acc, b| {
            use std::fmt::Write as _;
            let _ = write!(acc, "{b:02x}");
            acc
        });
        format!("\"{hex}\"")
    })
}

/// Field definitions: limits, units, steps, modes and texts, from `common::catalog` (the one
/// source the server validation also reads; ADR-0008 rule 7).
#[utoipa::path(
    get,
    path = "/v1/meta/fields",
    tag = "meta",
    operation_id = "meta_fields_get",
    responses(
        (
            status = 200,
            description = "The field catalog (see sozlesme/katalog/katalog.schema.json)",
            body = serde_json::Value,
            example = json!({"version": 1, "fields": {}, "units": {}, "quantities": {}, "texts": {}})
        ),
        (status = 304, description = "Not modified (`If-None-Match` matched the `ETag`)")
    )
)]
pub async fn meta_fields(headers: HeaderMap) -> Response {
    let etag = fields_etag();
    let cache = HeaderValue::from_str(&format!("private, max-age={FIELDS_MAX_AGE_SECS}"))
        .unwrap_or_else(|_| HeaderValue::from_static("private, no-cache"));
    let matches = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|raw| raw.split(',').map(str::trim).any(|t| t == "*" || t == etag));
    let mut response = if matches {
        StatusCode::NOT_MODIFIED.into_response()
    } else {
        (
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            )],
            common::catalog::RAW,
        )
            .into_response()
    };
    let h = response.headers_mut();
    if let Ok(v) = HeaderValue::from_str(etag) {
        h.insert(header::ETAG, v);
    }
    h.insert(header::CACHE_CONTROL, cache);
    response
}

/// The core routes: health, readiness, version and meta. None needs a sign-in.
pub fn routes(_state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/v1/version", get(version))
        .route("/v1/meta/fields", get(meta_fields))
        .route("/v1/meta/client-requirements", get(client_requirements))
}

// --- registry entry ----------------------------------------------------------------------

/// Operation specs of this module (rows for `crate::openapi`).
mod spec {
    use crate::error::ErrorCode;
    use crate::error::ErrorCode::{Internal, MalformedRequest, RateLimited, UpgradeRequired};
    use crate::openapi::OperationSpec;

    pub(super) const OPERATIONS: &[OperationSpec] = &[
        OperationSpec {
            operation_id: "health_check",
            public: true,
            permission: None,
            concurrency: None,
            errors: &[Internal],
        },
        OperationSpec {
            operation_id: "health_ready",
            public: true,
            permission: None,
            concurrency: None,
            errors: &[Internal, ErrorCode::Unavailable],
        },
        OperationSpec {
            operation_id: "version_get",
            public: true,
            permission: None,
            concurrency: None,
            errors: &[MalformedRequest, UpgradeRequired, RateLimited, Internal],
        },
        OperationSpec {
            operation_id: "meta_client_requirements_get",
            public: true,
            permission: None,
            concurrency: None,
            errors: &[RateLimited, Internal],
        },
        OperationSpec {
            operation_id: "meta_fields_get",
            public: true,
            permission: None,
            concurrency: None,
            errors: &[MalformedRequest, UpgradeRequired, RateLimited, Internal],
        },
    ];
}

/// `OpenAPI` fragment of this module; `crate::openapi` merges it.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(
        healthz,
        readyz,
        version,
        client_requirements,
        meta_fields
    ),
    components(schemas(
        HealthResponse,
        ReadyResponse,
        VersionResponse,
        crate::app_version::ClientRequirements
    )),
    tags(
        (name = "system", description = "Service status and version"),
        (name = "meta", description = "Definitions and client requirements")
    )
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "system",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
