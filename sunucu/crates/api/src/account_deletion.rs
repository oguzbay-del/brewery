//! Account deletion request, cancellation and status (T2.31b).
//!
//! The grace period is configurable. A request signs out every session immediately. Business
//! workspaces keep their records; a sole active administrator must transfer that role first.

use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::State,
    routing::{delete, get, post},
};
use chrono::{DateTime, Utc};
use db::{AccountDeletionRequest, AuditEvent, NewAuditEntry};
use serde::Serialize;
use sqlx::PgConnection;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState, RequestContext,
    auth::{AuthServices, step_up::ProtectedAction},
    authenticated,
    error::{ApiError, ApiResult, ErrorCode, FieldIssue},
    idempotent,
};

type Svc = Extension<Arc<AuthServices>>;

/// Public view of the caller's deletion request.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AccountDeletionView {
    /// Request identifier.
    pub id: Uuid,
    /// When the request was created.
    pub requested_at: DateTime<Utc>,
    /// Earliest permanent deletion time.
    pub purge_at: DateTime<Utc>,
    /// Current request state.
    pub status: String,
    /// When the request was cancelled.
    pub restored_at: Option<DateTime<Utc>>,
}

impl From<AccountDeletionRequest> for AccountDeletionView {
    fn from(request: AccountDeletionRequest) -> Self {
        Self {
            id: request.id,
            requested_at: request.requested_at,
            purge_at: request.purge_at,
            status: request.status,
            restored_at: request.restored_at,
        }
    }
}

/// Routes are disabled until the auth services are configured.
pub fn routes(state: &AppState) -> Router<AppState> {
    if state.auth_services.is_none() {
        return Router::new();
    }
    let create = idempotent(
        state,
        Router::new().route("/v1/me/deletion", post(account_deletion_request)),
    );
    authenticated(
        state,
        create
            .route("/v1/me/deletion", get(account_deletion_get))
            .route("/v1/me/deletion", delete(account_deletion_cancel)),
    )
}

fn last_admin() -> ApiError {
    ApiError::new(ErrorCode::Conflict).with_errors(vec![FieldIssue::new("/", "last_admin")])
}

fn not_found() -> ApiError {
    ApiError::new(ErrorCode::NotFound)
}

/// Refuses deletion when the caller is the only active administrator of a Business workspace.
/// The user-scoped member policy permits listing the caller's memberships; the workspace context
/// is then switched one workspace at a time to inspect its mode and administrator assignments.
async fn ensure_business_admin_can_leave(tx: &mut PgConnection, user_id: Uuid) -> ApiResult<()> {
    const ADMINS: &str = "SELECT count(DISTINCT m.id) FROM workspace_members m
        JOIN role_assignments a ON a.workspace_id = m.workspace_id AND a.member_id = m.id
             AND a.deleted_at IS NULL AND a.scope_kind IS NULL
        JOIN roles r ON r.workspace_id = a.workspace_id AND r.id = a.role_id
             AND r.deleted_at IS NULL
        JOIN role_permissions p ON p.workspace_id = r.workspace_id AND p.role_id = r.id
             AND p.permission = 'users.manage'
        WHERE m.workspace_id = $1 AND m.status = 'active' AND m.deleted_at IS NULL";

    let memberships: Vec<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT id, workspace_id FROM workspace_members
         WHERE user_id = $1 AND status = 'active' AND deleted_at IS NULL
         ORDER BY workspace_id",
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;

    for (member_id, workspace_id) in memberships {
        db::set_workspace_id(tx, workspace_id).await?;
        let mode: Option<String> =
            sqlx::query_scalar("SELECT mode FROM workspaces WHERE id = $1 AND deleted_at IS NULL")
                .bind(workspace_id)
                .fetch_optional(&mut *tx)
                .await?;
        let Some(mode) = mode else {
            continue;
        };
        if mode != "business" {
            continue;
        }
        let admins: i64 = sqlx::query_scalar(ADMINS)
            .bind(workspace_id)
            .fetch_one(&mut *tx)
            .await?;
        let caller_is_admin: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM workspace_members m
             JOIN role_assignments a ON a.workspace_id = m.workspace_id AND a.member_id = m.id
                  AND a.deleted_at IS NULL AND a.scope_kind IS NULL
             JOIN roles r ON r.workspace_id = a.workspace_id AND r.id = a.role_id
                  AND r.deleted_at IS NULL
             JOIN role_permissions p ON p.workspace_id = r.workspace_id AND p.role_id = r.id
                  AND p.permission = 'users.manage'
             WHERE m.workspace_id = $1 AND m.id = $2 AND m.status = 'active'
               AND m.deleted_at IS NULL)",
        )
        .bind(workspace_id)
        .bind(member_id)
        .fetch_one(&mut *tx)
        .await?;
        if caller_is_admin && admins <= 1 {
            return Err(last_admin());
        }
    }
    Ok(())
}

async fn append_account_audit(
    tx: &mut PgConnection,
    user_id: Uuid,
    request: &AccountDeletionRequest,
    status: &str,
) -> ApiResult<()> {
    let workspaces: Vec<Uuid> = sqlx::query_scalar(
        "SELECT workspace_id FROM workspace_members
         WHERE user_id = $1 AND status = 'active' AND deleted_at IS NULL
         ORDER BY workspace_id",
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;

    for workspace_id in workspaces {
        db::set_workspace_id(tx, workspace_id).await?;
        db::append_audit(
            tx,
            workspace_id,
            &NewAuditEntry {
                actor_kind: db::ActorKind::User,
                actor_id: Some(user_id),
                event: AuditEvent::AccountDelete,
                record_kind: "account_deletion_request".to_owned(),
                record_id: Some(request.id),
                previous_value: None,
                new_value: Some(serde_json::json!({
                    "status": status,
                    "purge_at": request.purge_at,
                })),
                reason: None,
            },
        )
        .await?;
    }
    Ok(())
}

/// Current deletion request. A null result means there is no request.
#[utoipa::path(
    get,
    path = "/v1/me/deletion",
    tag = "account deletion",
    operation_id = "account_deletion_get",
    responses(
        (status = 200, description = "Current deletion request", body = Option<AccountDeletionView>),
        (status = 401, description = "Unauthenticated"),
        (status = 429, description = "Rate limited")
    )
)]
pub async fn account_deletion_get(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> ApiResult<Json<Option<AccountDeletionView>>> {
    let mut tx = ctx.begin(&state.db).await?;
    let row = sqlx::query_as::<
        _,
        (
            Uuid,
            DateTime<Utc>,
            DateTime<Utc>,
            String,
            Option<DateTime<Utc>>,
        ),
    >(
        "SELECT id, requested_at, purge_at, status, restored_at
         FROM account_deletion_requests WHERE user_id = $1
         ORDER BY requested_at DESC LIMIT 1",
    )
    .bind(ctx.identity.user_id)
    .fetch_optional(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(row.map(
        |(id, requested_at, purge_at, status, restored_at)| AccountDeletionView {
            id,
            requested_at,
            purge_at,
            status,
            restored_at,
        },
    )))
}

/// Requests permanent account deletion after the configured grace period.
#[utoipa::path(
    post,
    path = "/v1/me/deletion",
    tag = "account deletion",
    operation_id = "account_deletion_request",
    responses(
        (status = 200, description = "Deletion request", body = AccountDeletionView),
        (status = 401, description = "Unauthenticated"),
        (status = 403, description = "A fresh second step is required"),
        (status = 409, description = "The caller is the only Business workspace administrator"),
        (status = 429, description = "Rate limited")
    )
)]
pub async fn account_deletion_request(
    State(state): State<AppState>,
    ctx: RequestContext,
    auth: Svc,
) -> ApiResult<Json<AccountDeletionView>> {
    auth.require_step_up(&ctx, ProtectedAction::AccountDelete)
        .await?;
    let user_id = ctx.identity.user_id;
    let grace_days = db::account_deletion_grace_days()?;
    let mut tx = ctx.begin(&state.db).await?;
    ensure_business_admin_can_leave(&mut tx, user_id).await?;

    let already_pending: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM account_deletion_requests
         WHERE user_id = $1 AND status = 'pending')",
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;
    let request = db::request_account_deletion(&mut tx, grace_days).await?;
    if !already_pending {
        append_account_audit(&mut tx, user_id, &request, "pending").await?;
    }
    tx.commit().await?;

    // This also removes the Redis session-cache entries.
    auth.sessions.revoke_all(user_id, None).await?;
    Ok(Json(request.into()))
}

/// Cancels a pending request while its grace period remains open.
#[utoipa::path(
    delete,
    path = "/v1/me/deletion",
    tag = "account deletion",
    operation_id = "account_deletion_cancel",
    responses(
        (status = 200, description = "Deletion request cancelled", body = AccountDeletionView),
        (status = 401, description = "Unauthenticated"),
        (status = 404, description = "No pending request"),
        (status = 409, description = "The grace period has ended"),
        (status = 429, description = "Rate limited")
    )
)]
pub async fn account_deletion_cancel(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> ApiResult<Json<AccountDeletionView>> {
    let user_id = ctx.identity.user_id;
    let mut tx = ctx.begin(&state.db).await?;
    let purge_at: Option<DateTime<Utc>> = sqlx::query_scalar(
        "SELECT purge_at FROM account_deletion_requests
         WHERE user_id = $1 AND status = 'pending'
         ORDER BY requested_at DESC LIMIT 1 FOR UPDATE",
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(purge_at) = purge_at else {
        return Err(not_found());
    };
    if purge_at <= Utc::now() {
        return Err(ApiError::new(ErrorCode::Conflict)
            .with_errors(vec![FieldIssue::new("/", "grace_period_ended")]));
    }
    let request = db::cancel_account_deletion(&mut tx)
        .await?
        .ok_or_else(|| ApiError::new(ErrorCode::Conflict))?;
    append_account_audit(&mut tx, user_id, &request, "cancelled").await?;
    tx.commit().await?;
    Ok(Json(request.into()))
}

mod spec {
    use crate::error::ErrorCode;
    use crate::error::ErrorCode::{
        Conflict, Internal, NotFound, RateLimited, StepUpRequired, Unauthenticated, Unavailable,
        UpgradeRequired,
    };
    use crate::openapi::{OperationSpec, file_op};

    const READ: &[ErrorCode] = &[
        Unauthenticated,
        NotFound,
        RateLimited,
        UpgradeRequired,
        Internal,
        Unavailable,
    ];
    const WRITE: &[ErrorCode] = &[
        Unauthenticated,
        StepUpRequired,
        Conflict,
        RateLimited,
        UpgradeRequired,
        Internal,
        Unavailable,
    ];

    pub(super) const OPERATIONS: &[OperationSpec] = &[
        file_op("account_deletion_get", None, None, READ),
        file_op("account_deletion_request", None, None, WRITE),
        file_op("account_deletion_cancel", None, None, WRITE),
    ];
}

#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(account_deletion_get, account_deletion_request, account_deletion_cancel),
    components(schemas(AccountDeletionView)),
    tags((name = "account deletion", description = "Account deletion and restore during the grace period"))
)]
struct ModuleDoc;

/// Registry entry.
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "account_deletion",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
