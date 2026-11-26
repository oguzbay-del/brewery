//! Subscription state (T2.28): `GET /v1/workspaces/{ws}/subscription`, and the guard that other
//! modules call before a paid write.
//!
//! The API only reads. The worker writes `subscriptions` and `entitlements` (`billing` crate).
//! One month after the workspace was created the paid features become read-only. Export is
//! always available: an export endpoint never calls [`require_paid_write`].

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use billing::access::{self, AccessLevel, AccessSource};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgConnection;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{ApiError, ApiResult, AppState, RequestContext, authenticated, error::ErrorCode};

/// Access level to the paid features.
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccessLevelDto {
    /// Reads and writes.
    Full,
    /// Reads only; export stays available.
    ReadOnly,
}

/// Reason for the access level.
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccessSourceDto {
    /// The trial of the workspace.
    Trial,
    /// A paid subscription.
    Subscription,
    /// A promotion.
    Promo,
    /// A manual correction by support.
    Manual,
    /// No trial and no entitlement.
    None,
}

/// A subscription of the workspace.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SubscriptionView {
    /// `app_store` or `web`.
    pub source: String,
    /// Product key.
    pub product_key: String,
    /// `trial`, `active`, `grace`, `expired` or `cancelled`.
    pub status: String,
    /// End of the paid period.
    pub period_ends_at: Option<DateTime<Utc>>,
}

/// Subscription state of a workspace.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SubscriptionState {
    /// Access level now.
    pub access: AccessLevelDto,
    /// Reason for the access level.
    pub access_source: AccessSourceDto,
    /// End of the trial (3 months for Home, 1 month for Business).
    pub trial_ends_at: DateTime<Utc>,
    /// End of the entitlement that gives access now; absent for the trial.
    pub valid_until: Option<DateTime<Utc>>,
    /// Subscriptions, oldest first.
    pub subscriptions: Vec<SubscriptionView>,
}

/// Routes of this module.
pub fn routes(state: &AppState) -> Router<AppState> {
    authenticated(
        state,
        Router::new().route("/v1/workspaces/{ws}/subscription", get(get_subscription)),
    )
}

/// Guard for a write to a paid feature: after the trial without an entitlement, the feature is
/// read-only. Call it with the request transaction. Do not call it for export or any read.
///
/// # Errors
/// `403 entitlement_required` when the workspace is read-only.
pub async fn require_paid_write(conn: &mut PgConnection, workspace_id: Uuid) -> ApiResult<()> {
    let access = access::access(conn, workspace_id).await?;
    match access.level {
        AccessLevel::Full => Ok(()),
        AccessLevel::ReadOnly => Err(ApiError::new(ErrorCode::EntitlementRequired)),
    }
}

/// Subscription state of the workspace (trial, entitlement, subscriptions).
#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws}/subscription",
    tag = "subscription",
    operation_id = "subscription_get",
    params(("ws" = Uuid, Path, description = "Workspace id")),
    responses((
        status = 200,
        description = "Subscription state",
        body = SubscriptionState,
        example = json!({
            "access": "full",
            "access_source": "subscription",
            "trial_ends_at": "2026-12-20T09:00:00Z",
            "valid_until": "2027-01-19T09:00:00Z",
            "subscriptions": [{
                "source": "app_store",
                "product_key": "pro.monthly",
                "status": "active",
                "period_ends_at": "2027-01-19T09:00:00Z"
            }]
        })
    ))
)]
pub async fn get_subscription(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path(ws): Path<Uuid>,
) -> ApiResult<Json<SubscriptionState>> {
    if ctx.workspace_id != Some(ws) {
        return Err(ApiError::new(ErrorCode::NotFound));
    }
    let mut tx = ctx.begin(&state.db).await?;
    let a = access::access(&mut tx, ws).await?;
    let subs = access::subscriptions(&mut tx, ws).await?;
    Ok(Json(SubscriptionState {
        access: match a.level {
            AccessLevel::Full => AccessLevelDto::Full,
            AccessLevel::ReadOnly => AccessLevelDto::ReadOnly,
        },
        access_source: match a.source {
            AccessSource::Trial => AccessSourceDto::Trial,
            AccessSource::Subscription => AccessSourceDto::Subscription,
            AccessSource::Promo => AccessSourceDto::Promo,
            AccessSource::Manual => AccessSourceDto::Manual,
            AccessSource::None => AccessSourceDto::None,
        },
        trial_ends_at: a.trial_ends_at,
        valid_until: a.valid_until,
        subscriptions: subs
            .into_iter()
            .map(|s| SubscriptionView {
                source: s.source,
                product_key: s.product_key,
                status: s.status,
                period_ends_at: s.period_ends_at,
            })
            .collect(),
    }))
}

// --- registry entry ----------------------------------------------------------------------

mod spec {
    use crate::error::ErrorCode::{
        Internal, NotFound, RateLimited, Unauthenticated, Unavailable, UpgradeRequired,
    };
    use crate::openapi::{OperationSpec, op};

    pub(super) const OPERATIONS: &[OperationSpec] = &[op(
        "subscription_get",
        false,
        None,
        &[
            Unauthenticated,
            NotFound,
            UpgradeRequired,
            RateLimited,
            Internal,
            Unavailable,
        ],
    )];
}

/// `OpenAPI` fragment of this module.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(get_subscription),
    components(schemas(
        SubscriptionState,
        SubscriptionView,
        AccessLevelDto,
        AccessSourceDto
    )),
    tags((name = "subscription", description = "Trial, entitlement and subscriptions"))
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "subscription",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
