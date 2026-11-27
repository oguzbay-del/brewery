//! Billing endpoints (B3): `GET /v1/billing`, `POST /v1/billing/checkout` (web only) and
//! `POST /v1/billing/portal`.
//!
//! The workspace comes from the query (`GET`) or the body (`POST`), because these routes are
//! not under `/v1/workspaces/{ws}`. The caller must be an active member; checkout and portal
//! need the permission `billing.manage`. The API never writes `subscriptions` or `entitlements`:
//! the payment provider webhook does, through `ingest` and the worker.
//!
//! One workspace has one live subscription source. When the other source is live, checkout and
//! portal answer `409` (`other_source_active`).
//!
//! Paddle settings: `PADDLE_API_KEY` (or `_FILE`), `PADDLE_API_BASE`, `PADDLE_PRICE_HOME`,
//! `PADDLE_PRICE_BUSINESS`. There are no defaults. When one is missing, checkout and portal answer
//! `503 billing_unavailable`.

use std::{fmt, sync::Arc, time::Duration};

use axum::{Json, Router, extract::State, http::HeaderMap, routing::get, routing::post};
use billing::{PlanKey, access};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::PgConnection;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState, RequestContext, authenticated,
    error::{ApiError, ApiJson, ApiQuery, ApiResult, ErrorCode, FieldIssue},
    subscription::{SubscriptionState, state_view},
};

/// Permission that checkout and portal need.
const MANAGE: &str = "billing.manage";
/// Largest seat count a request can ask for (the same as the database check).
const MAX_SEATS: u32 = 100_000;
/// Time limit of one call to the payment provider.
const PROVIDER_TIMEOUT: Duration = Duration::from_secs(10);

/// Client of the Paddle Billing API (checkout and customer portal).
#[derive(Clone)]
pub struct PaddleApi {
    client: reqwest::Client,
    base: String,
    api_key: String,
    price_home: String,
    price_business: String,
}

impl fmt::Debug for PaddleApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PaddleApi")
            .field("base", &self.base)
            .finish_non_exhaustive()
    }
}

impl PaddleApi {
    /// A client for `base` (for example `https://api.paddle.com`).
    #[must_use]
    pub fn new(base: &str, api_key: &str, price_home: &str, price_business: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(PROVIDER_TIMEOUT)
            .build()
            .unwrap_or_default();
        Self {
            client,
            base: base.trim_end_matches('/').to_owned(),
            api_key: api_key.to_owned(),
            price_home: price_home.to_owned(),
            price_business: price_business.to_owned(),
        }
    }

    /// Reads the four settings. `None` when one is missing or empty.
    ///
    /// # Errors
    /// A configuration error when the key file cannot be read.
    pub fn from_lookup<F>(lookup: F) -> common::Result<Option<Self>>
    where
        F: Fn(&str) -> Option<String>,
    {
        let key =
            common::config::secret_from_lookup(&lookup, "PADDLE_API_KEY", "PADDLE_API_KEY_FILE")?;
        let get = |name: &str| {
            lookup(name)
                .map(|v| v.trim().to_owned())
                .filter(|v| !v.is_empty())
        };
        let (base, home, business) = (
            get("PADDLE_API_BASE"),
            get("PADDLE_PRICE_HOME"),
            get("PADDLE_PRICE_BUSINESS"),
        );
        Ok(match (key, base, home, business) {
            (Some(key), Some(base), Some(home), Some(business)) => {
                Some(Self::new(&base, &key, &home, &business))
            }
            _ => None,
        })
    }

    /// Reads the settings from the process environment.
    ///
    /// # Errors
    /// See [`Self::from_lookup`].
    pub fn from_env() -> common::Result<Option<Self>> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    fn price(&self, plan: PlanKey) -> &str {
        match plan {
            PlanKey::Home => &self.price_home,
            PlanKey::Business => &self.price_business,
        }
    }

    async fn call(&self, path: &str, body: &Value) -> ApiResult<Value> {
        let response = self
            .client
            .post(format!("{}{path}", self.base))
            .bearer_auth(&self.api_key)
            .json(body)
            .send()
            .await
            .map_err(|e| {
                tracing::warn!(error = %e.without_url(), "Paddle request failed");
                ApiError::new(ErrorCode::BillingUnavailable)
            })?;
        let status = response.status();
        if !status.is_success() {
            tracing::warn!(%status, path, "Paddle answered with an error");
            return Err(ApiError::new(ErrorCode::BillingUnavailable));
        }
        response.json().await.map_err(|e| {
            tracing::warn!(error = %e.without_url(), "Paddle answer is not JSON");
            ApiError::new(ErrorCode::BillingUnavailable)
        })
    }

    /// Creates a transaction and returns its checkout address. The workspace id and the plan go
    /// in `custom_data`; Paddle copies them to the subscription and its webhooks.
    async fn create_checkout(&self, ws: Uuid, plan: PlanKey, seats: u32) -> ApiResult<String> {
        let answer = self
            .call(
                "/transactions",
                &json!({
                    "items": [{"price_id": self.price(plan), "quantity": seats}],
                    "custom_data": {"workspace_id": ws, "plan": plan.as_str()},
                }),
            )
            .await?;
        text_at(&answer, "/data/checkout/url")
    }

    /// Creates a customer portal session and returns its address.
    async fn create_portal(&self, customer: &str, subscriptions: &[String]) -> ApiResult<String> {
        let answer = self
            .call(
                &format!("/customers/{customer}/portal-sessions"),
                &json!({ "subscription_ids": subscriptions }),
            )
            .await?;
        text_at(&answer, "/data/urls/general/overview")
    }
}

fn text_at(answer: &Value, pointer: &str) -> ApiResult<String> {
    answer
        .pointer(pointer)
        .and_then(Value::as_str)
        .filter(|u| u.starts_with("https://") || u.starts_with("http://"))
        .map(str::to_owned)
        .ok_or_else(|| {
            tracing::warn!(pointer, "Paddle answer has no address");
            ApiError::new(ErrorCode::BillingUnavailable)
        })
}

fn conflict(code: &'static str) -> ApiError {
    ApiError::new(ErrorCode::Conflict).with_errors(vec![FieldIssue::new("/", code)])
}

fn paddle(state: &AppState) -> ApiResult<Arc<PaddleApi>> {
    state
        .paddle
        .clone()
        .ok_or_else(|| ApiError::new(ErrorCode::BillingUnavailable))
}

/// The caller as a member of `ws`. A workspace the caller does not belong to is `404`.
async fn member_context(
    state: &AppState,
    ctx: RequestContext,
    ws: Uuid,
) -> ApiResult<RequestContext> {
    let membership = state
        .auth
        .membership(&ctx.identity, ws)
        .await?
        .ok_or_else(|| ApiError::new(ErrorCode::NotFound))?;
    Ok(RequestContext {
        identity: ctx.identity,
        workspace_id: Some(ws),
        permissions: membership.permissions,
        scoped: membership.scoped,
    })
}

/// The source of a subscription that gives access now (`app_store` or `web`), if any.
async fn live_source(conn: &mut PgConnection, ws: Uuid) -> ApiResult<Option<String>> {
    Ok(sqlx::query_scalar(
        "SELECT source FROM subscriptions
         WHERE workspace_id = $1 AND deleted_at IS NULL AND status IN ('active', 'grace')
           AND (period_ends_at IS NULL OR period_ends_at > now())
         ORDER BY created_at, id LIMIT 1",
    )
    .bind(ws)
    .fetch_optional(conn)
    .await?)
}

/// Query of `GET /v1/billing`.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct BillingQuery {
    /// Workspace to inspect; the caller must be an active member.
    pub workspace_id: Uuid,
}

/// Billing state of a workspace.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct BillingView {
    /// Access level, plan, seats and subscriptions.
    pub subscription: SubscriptionState,
    /// Source of the subscription that gives access now (`app_store` or `web`); absent when
    /// none does.
    pub active_source: Option<String>,
    /// Whether web checkout and the customer portal are configured on this server.
    pub web_billing_available: bool,
}

/// Body of `POST /v1/billing/checkout`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CheckoutRequest {
    /// Workspace to subscribe.
    pub workspace_id: Uuid,
    /// `home` or `business`.
    pub plan: String,
    /// Number of seats; default 1.
    #[serde(default)]
    pub seats: Option<u32>,
}

/// Address of a checkout page.
#[derive(Debug, Serialize, ToSchema)]
pub struct CheckoutView {
    /// Open this address in the browser.
    pub checkout_url: String,
}

/// Body of `POST /v1/billing/portal`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PortalRequest {
    /// Workspace whose web subscription to manage.
    pub workspace_id: Uuid,
}

/// Address of a customer portal session.
#[derive(Debug, Serialize, ToSchema)]
pub struct PortalView {
    /// Open this address in the browser.
    pub portal_url: String,
}

/// Routes of this module.
pub fn routes(state: &AppState) -> Router<AppState> {
    authenticated(
        state,
        Router::new()
            .route("/v1/billing", get(billing_get))
            .route("/v1/billing/checkout", post(billing_checkout))
            .route("/v1/billing/portal", post(billing_portal)),
    )
}

/// Billing state: access, plan, seats, subscriptions and the live source.
#[utoipa::path(
    get,
    path = "/v1/billing",
    tag = "billing",
    operation_id = "billing_get",
    params(BillingQuery),
    responses((
        status = 200,
        description = "Billing state",
        body = BillingView,
        example = json!({
            "subscription": {
                "access": "full",
                "access_source": "subscription",
                "trial_ends_at": "2026-12-20T09:00:00Z",
                "valid_until": "2027-01-19T09:00:00Z",
                "plan_key": "business",
                "seats": 5,
                "subscriptions": [{
                    "source": "web",
                    "product_key": "business",
                    "status": "active",
                    "period_ends_at": "2027-01-19T09:00:00Z"
                }]
            },
            "active_source": "web",
            "web_billing_available": true
        })
    ))
)]
pub async fn billing_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(query): ApiQuery<BillingQuery>,
) -> ApiResult<Json<BillingView>> {
    let ws = query.workspace_id;
    let ctx = member_context(&state, ctx, ws).await?;
    let mut tx = ctx.begin(&state.db).await?;
    let a = access::access(&mut tx, ws).await?;
    let subs = access::subscriptions(&mut tx, ws).await?;
    let active_source = live_source(&mut tx, ws).await?;
    Ok(Json(BillingView {
        subscription: state_view(a, subs),
        active_source,
        web_billing_available: state.paddle.is_some(),
    }))
}

/// Starts a web checkout (Paddle). Clients on iOS use `StoreKit` instead.
#[utoipa::path(
    post,
    path = "/v1/billing/checkout",
    tag = "billing",
    operation_id = "billing_checkout",
    request_body = CheckoutRequest,
    responses((
        status = 200,
        description = "Checkout address",
        body = CheckoutView,
        example = json!({"checkout_url": "https://checkout.example/pay/txn_01"})
    ))
)]
pub async fn billing_checkout(
    State(state): State<AppState>,
    ctx: RequestContext,
    headers: HeaderMap,
    ApiJson(body): ApiJson<CheckoutRequest>,
) -> ApiResult<Json<CheckoutView>> {
    let ws = body.workspace_id;
    let ctx = member_context(&state, ctx, ws).await?;
    ctx.require(MANAGE)?;
    let plan = PlanKey::parse(&body.plan).ok_or_else(|| {
        ApiError::new(ErrorCode::ValidationFailed)
            .with_errors(vec![FieldIssue::new("/plan", "invalid_value")])
    })?;
    let seats = body.seats.unwrap_or(1);
    if !(1..=MAX_SEATS).contains(&seats) {
        return Err(ApiError::new(ErrorCode::ValidationFailed)
            .with_errors(vec![FieldIssue::new("/seats", "out_of_range")]));
    }
    let from_ios = headers
        .get(crate::app_version::HEADER)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.trim_start().starts_with("ios/"));
    if from_ios {
        return Err(conflict("web_only"));
    }
    let paddle = paddle(&state)?;
    let mut tx = ctx.begin(&state.db).await?;
    match live_source(&mut tx, ws).await?.as_deref() {
        Some("app_store") => return Err(conflict("other_source_active")),
        Some(_) => return Err(conflict("already_subscribed")),
        None => {}
    }
    drop(tx);
    let checkout_url = paddle.create_checkout(ws, plan, seats).await?;
    Ok(Json(CheckoutView { checkout_url }))
}

/// Opens a Paddle customer portal session for the web subscription of the workspace.
#[utoipa::path(
    post,
    path = "/v1/billing/portal",
    tag = "billing",
    operation_id = "billing_portal",
    request_body = PortalRequest,
    responses((
        status = 200,
        description = "Portal address",
        body = PortalView,
        example = json!({"portal_url": "https://portal.example/session/cpls_01"})
    ))
)]
pub async fn billing_portal(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<PortalRequest>,
) -> ApiResult<Json<PortalView>> {
    let ws = body.workspace_id;
    let ctx = member_context(&state, ctx, ws).await?;
    ctx.require(MANAGE)?;
    let paddle = paddle(&state)?;
    let mut tx = ctx.begin(&state.db).await?;
    if live_source(&mut tx, ws).await?.as_deref() == Some("app_store") {
        return Err(conflict("other_source_active"));
    }
    let web: Option<(String, String)> = sqlx::query_as(
        "SELECT paddle_customer_id, external_id FROM subscriptions
         WHERE workspace_id = $1 AND source = 'web' AND deleted_at IS NULL
           AND paddle_customer_id IS NOT NULL
         ORDER BY (status IN ('active', 'grace')) DESC, created_at DESC, id LIMIT 1",
    )
    .bind(ws)
    .fetch_optional(&mut *tx)
    .await?;
    drop(tx);
    let Some((customer, subscription)) = web else {
        return Err(conflict("no_web_subscription"));
    };
    let portal_url = paddle.create_portal(&customer, &[subscription]).await?;
    Ok(Json(PortalView { portal_url }))
}

// --- registry entry ----------------------------------------------------------------------

mod spec {
    use crate::error::ErrorCode::{
        BillingUnavailable, Conflict, Forbidden, Internal, MalformedRequest, NotFound, RateLimited,
        Unauthenticated, Unavailable, UnsupportedMediaType, UpgradeRequired, ValidationFailed,
    };
    use crate::openapi::{OperationSpec, op};

    pub(super) const OPERATIONS: &[OperationSpec] = &[
        op(
            "billing_get",
            false,
            None,
            &[
                MalformedRequest,
                Unauthenticated,
                NotFound,
                UpgradeRequired,
                RateLimited,
                Internal,
                Unavailable,
            ],
        ),
        op(
            "billing_checkout",
            false,
            Some("billing.manage"),
            &[
                MalformedRequest,
                Unauthenticated,
                Forbidden,
                NotFound,
                Conflict,
                UnsupportedMediaType,
                ValidationFailed,
                UpgradeRequired,
                RateLimited,
                Internal,
                Unavailable,
                BillingUnavailable,
            ],
        ),
        op(
            "billing_portal",
            false,
            Some("billing.manage"),
            &[
                MalformedRequest,
                Unauthenticated,
                Forbidden,
                NotFound,
                Conflict,
                UnsupportedMediaType,
                UpgradeRequired,
                RateLimited,
                Internal,
                Unavailable,
                BillingUnavailable,
            ],
        ),
    ];
}

/// `OpenAPI` fragment of this module.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(billing_get, billing_checkout, billing_portal),
    components(schemas(
        BillingView,
        CheckoutRequest,
        CheckoutView,
        PortalRequest,
        PortalView
    )),
    tags((name = "billing", description = "Web checkout, customer portal and billing state"))
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "billing",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
