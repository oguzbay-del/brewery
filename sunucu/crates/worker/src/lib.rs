//! `worker` service: HTTP router.

pub mod account_deletion;
pub mod audit_anchor;
pub mod background;
pub mod export;
pub mod maintenance;
pub mod retention;

use axum::Router;

/// Service name reported in logs and the health response.
pub const SERVICE_NAME: &str = "worker";
/// Default listen port (override with `PORT`).
pub const DEFAULT_PORT: u16 = 8081;

/// Errors of the maintenance jobs.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Database failure.
    #[error(transparent)]
    Db(#[from] db::Error),
    /// Object store failure.
    #[error(transparent)]
    Storage(#[from] storage::StorageError),
    /// Data that cannot be written or computed.
    #[error("{0}")]
    Format(String),
}

/// Result of a maintenance job.
pub type Result<T> = std::result::Result<T, Error>;

/// Builds the service router.
pub fn router() -> Router {
    common::http::health_router(SERVICE_NAME, env!("CARGO_PKG_VERSION"))
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

    #[tokio::test]
    async fn healthz_returns_ok() {
        let response = router()
            .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["service"], "worker");
        assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
    }
}
