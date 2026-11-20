//! `ingest` service: HTTP router and webhook intake.
//!
//! - `POST /v1/webhooks/app-store`: App Store Server Notifications v2. The service verifies the
//!   JWS (certificate chain to the Apple root) and stores the normalized update in
//!   `external_events` plus one outbox event, in one transaction. A repeated notification is
//!   acknowledged and not stored again. The worker applies the update (`billing` crate).
//! - `POST /v1/webhooks/web-payment`: the same flow for the web payment provider interface
//!   (`billing::provider::WebPaymentProvider`; Paddle per decision O6, adapter not written yet).

pub mod apple;
pub mod jws;

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use billing::{
    SubscriptionUpdate, apply,
    provider::{ProviderError, WebPaymentProvider},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::{apple::AppleError, jws::JwsError};

/// Service name reported in logs and the health response.
pub const SERVICE_NAME: &str = "ingest";
/// Default listen port (override with `PORT`).
pub const DEFAULT_PORT: u16 = 8082;

/// State of the webhook routes.
#[derive(Clone)]
pub struct IngestState {
    /// Pool for role `app_api` (inserts into `external_events` and `outbox`).
    pub pool: PgPool,
    /// JWS verifier for App Store notifications.
    pub apple: jws::JwsVerifier,
    /// Accepted bundle id (`APPLE_BUNDLE_ID`); `None` accepts any app.
    pub bundle_id: Option<String>,
    /// Web payment provider; `None` until decision O6.
    pub web_payment: Option<Arc<dyn WebPaymentProvider>>,
}

impl std::fmt::Debug for IngestState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IngestState")
            .field("bundle_id", &self.bundle_id)
            .field("web_payment", &self.web_payment.is_some())
            .finish_non_exhaustive()
    }
}

/// Builds the router with the health endpoint only.
pub fn router() -> Router {
    common::http::health_router(SERVICE_NAME, env!("CARGO_PKG_VERSION"))
}

/// Builds the router with the health endpoint and the webhook routes.
pub fn router_with(state: IngestState) -> Router {
    let hooks = Router::new()
        .route("/v1/webhooks/app-store", post(app_store))
        .route("/v1/webhooks/web-payment", post(web_payment))
        .with_state(state);
    router().merge(hooks)
}

/// Builds the state from the environment: `DATABASE_URL` (role `app_api`) and the optional
/// `APPLE_BUNDLE_ID`. Returns `None` without a database.
#[must_use]
pub fn state_from_env() -> Option<IngestState> {
    let url = db::database_url().ok()?;
    let pool = db::connect_lazy(&url, 4).ok()?;
    Some(IngestState {
        pool,
        apple: jws::JwsVerifier::apple(),
        bundle_id: std::env::var("APPLE_BUNDLE_ID")
            .ok()
            .filter(|s| !s.is_empty()),
        web_payment: None,
    })
}

type Reply = (StatusCode, Json<Value>);

fn reply(status: StatusCode, text: &str) -> Reply {
    (status, Json(json!({ "status": text })))
}

fn now_unix() -> i64 {
    chrono::Utc::now().timestamp()
}

async fn store(state: &IngestState, update: &SubscriptionUpdate) -> Reply {
    let result = async {
        let mut tx = state.pool.begin().await?;
        let stored = apply::store(&mut tx, update).await?;
        tx.commit().await?;
        Ok::<_, db::Error>(stored)
    }
    .await;
    match result {
        Ok(apply::Stored::Queued(_)) => reply(StatusCode::OK, "accepted"),
        Ok(apply::Stored::Duplicate) => reply(StatusCode::OK, "duplicate"),
        Err(e) => {
            tracing::error!(error = %e, "storing a notification failed");
            // Non-2xx makes the sender retry.
            reply(StatusCode::SERVICE_UNAVAILABLE, "unavailable")
        }
    }
}

/// Body of an App Store Server Notification v2.
#[derive(Debug, Deserialize)]
struct AppStoreBody {
    #[serde(rename = "signedPayload")]
    signed_payload: String,
}

async fn app_store(State(state): State<IngestState>, body: Json<AppStoreBody>) -> Reply {
    match apple::parse_notification(
        &state.apple,
        &body.signed_payload,
        now_unix(),
        state.bundle_id.as_deref(),
    ) {
        Ok(apple::Parsed::Update(update)) => store(&state, &update).await,
        Ok(apple::Parsed::Ignored { notification_type }) => {
            tracing::info!(%notification_type, "App Store notification ignored");
            reply(StatusCode::OK, "ignored")
        }
        Err(AppleError::Jws(JwsError::Malformed(_)) | AppleError::Field(_)) => {
            reply(StatusCode::BAD_REQUEST, "malformed")
        }
        Err(e) => {
            tracing::warn!(error = %e, "App Store notification rejected");
            reply(StatusCode::UNAUTHORIZED, "rejected")
        }
    }
}

async fn web_payment(
    State(state): State<IngestState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Reply {
    let Some(provider) = state.web_payment.clone() else {
        return reply(StatusCode::SERVICE_UNAVAILABLE, "no_provider");
    };
    // `Paddle-Signature` is the header of the selected provider; `x-signature` is for tests.
    let signature = headers
        .get("paddle-signature")
        .or_else(|| headers.get("x-signature"))
        .and_then(|v| v.to_str().ok());
    match provider.parse_webhook(signature, &body) {
        Ok(Some(update)) => store(&state, &update).await,
        Ok(None) => reply(StatusCode::OK, "ignored"),
        Err(ProviderError::InvalidSignature) => reply(StatusCode::UNAUTHORIZED, "rejected"),
        Err(ProviderError::Malformed(_)) => reply(StatusCode::BAD_REQUEST, "malformed"),
    }
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
