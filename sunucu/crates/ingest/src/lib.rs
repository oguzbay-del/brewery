//! `ingest` service: HTTP router.

use axum::Router;

/// Service name reported in logs and the health response.
pub const SERVICE_NAME: &str = "ingest";
/// Default listen port (override with `PORT`).
pub const DEFAULT_PORT: u16 = 8082;

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
        assert_eq!(json["service"], "ingest");
        assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
    }
}
