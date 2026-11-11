//! HTTP helpers shared by the axum-based services.

use std::net::SocketAddr;

use axum::{Json, Router, extract::State, routing::get};
use serde_json::{Value, json};

use crate::{Result, shutdown::shutdown_signal};

#[derive(Clone)]
struct HealthState {
    service: &'static str,
    version: &'static str,
}

async fn healthz(State(state): State<HealthState>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": state.service,
        "version": state.version,
    }))
}

/// Router exposing `GET /healthz` returning `{"status":"ok","service":...,"version":...}`.
///
/// Callers pass `env!("CARGO_PKG_VERSION")` so the version is the calling crate's.
pub fn health_router(service: &'static str, version: &'static str) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .with_state(HealthState { service, version })
}

/// Serves `router` on `0.0.0.0:port` until SIGTERM/Ctrl-C, then shuts down gracefully.
pub async fn serve(router: Router, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "listening");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    tracing::info!("server stopped");
    Ok(())
}
