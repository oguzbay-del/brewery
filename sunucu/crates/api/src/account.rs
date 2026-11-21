//! Account deletion and data export request (T2.31, ADR-0007 rule 6.2, ADR-0009 rule 1.5).
//!
//! - `POST /v1/me/deletion`: asks for the deletion. It needs the second step. It closes all
//!   sessions and removes the push targets at once, and the account waits
//!   [`db::account::RETENTION_DAYS`] days. The worker deletes the account for good after that.
//! - `DELETE /v1/me/deletion`: restores the account inside that time. The user signs in again
//!   first (sign-in stays possible while a request is pending; workspace endpoints do not
//!   answer, see `AuthServices::membership`).
//! - `GET /v1/me/deletion`: the state of the request.
//! - `POST /v1/me/data-export`: queues an `export_jobs` row with scope `account_data`.
//!   `GET /v1/me/data-export/{workspace_id}/{id}` reads the job and gives the download link.
//!
//! A user who is the only administrator of a Business workspace cannot delete the account
//! (`409 sole_business_admin`): the workspace would have nobody left. To be verified (product
//! owner): transfer of ownership or deletion of the Business workspace first.

use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use db::{ActorKind, AuditEvent, NewAuditEntry};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgConnection;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    auth::{AuthServices, step_up::ProtectedAction},
    context::RequestContext,
    error::{ApiError, ApiJson, ApiResult, ErrorCode, FieldIssue},
    import_export::{ExportJob, ExportRow, SELECT_EXPORT, export_job},
};

type Svc = Extension<Arc<AuthServices>>;

/// One account data export for each user in this many hours.
const DATA_EXPORT_INTERVAL_HOURS: i64 = 24;

/// State of the deletion request.
#[derive(Debug, Serialize, ToSchema)]
pub struct DeletionStatus {
    /// `none` or `pending`.
    pub state: String,
    /// When the user asked.
    pub requested_at: Option<DateTime<Utc>>,
    /// When the account is deleted for good. The user can restore it before this time.
    pub purge_at: Option<DateTime<Utc>>,
    /// Days between the request and the permanent delete.
    pub retention_days: i64,
}

impl DeletionStatus {
    fn of(d: Option<db::account::Deletion>) -> Self {
        Self {
            state: if d.is_some() { "pending" } else { "none" }.to_owned(),
            requested_at: d.map(|d| d.requested_at),
            purge_at: d.map(|d| d.purge_at),
            retention_days: db::account::RETENTION_DAYS,
        }
    }
}

/// Body of `POST /v1/me/data-export`.
#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct NewDataExport {
    /// The workspace that carries the job (default: the oldest active membership).
    #[serde(default)]
    pub workspace_id: Option<Uuid>,
}

fn conflict(field: &str, code: &str) -> ApiError {
    ApiError::new(ErrorCode::Conflict).with_errors(vec![FieldIssue::new(field, code)])
}

fn user_ctx(user: Uuid, ws: Option<Uuid>) -> db::DbContext {
    db::DbContext::user(user, ws)
}

/// Active memberships `(workspace, status)` of `user`, oldest first.
async fn memberships(state: &AppState, user: Uuid) -> ApiResult<Vec<(Uuid, String)>> {
    let mut tx = db::begin(&state.db, &user_ctx(user, None)).await?;
    let rows: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT workspace_id, status FROM workspace_members \
         WHERE user_id = $1 AND deleted_at IS NULL ORDER BY joined_at, id",
    )
    .bind(user)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(rows)
}

/// `true` when `user` is the only member with `users.manage` for the whole of the Business
/// workspace `ws`. Same rule as `account_workspaces_of` in migration 0220.
async fn sole_business_admin(state: &AppState, user: Uuid, ws: Uuid) -> ApiResult<bool> {
    let mut tx = db::begin(&state.db, &user_ctx(user, Some(ws))).await?;
    let mode: Option<String> =
        sqlx::query_scalar("SELECT mode FROM workspaces WHERE id = $1 AND deleted_at IS NULL")
            .bind(ws)
            .fetch_optional(&mut *tx)
            .await?;
    if mode.as_deref() != Some("business") {
        tx.commit().await?;
        return Ok(false);
    }
    let (mine, others): (i64, i64) = sqlx::query_as(
        "SELECT count(DISTINCT x.id) FILTER (WHERE x.user_id = $2), \
                count(DISTINCT x.id) FILTER (WHERE x.user_id <> $2) \
         FROM (SELECT mm.id, mm.user_id FROM workspace_members mm \
               JOIN role_assignments a ON a.workspace_id = mm.workspace_id AND a.member_id = mm.id \
                    AND a.deleted_at IS NULL AND a.scope_kind IS NULL \
               JOIN roles r ON r.workspace_id = a.workspace_id AND r.id = a.role_id \
                    AND r.deleted_at IS NULL \
               JOIN role_permissions p ON p.workspace_id = r.workspace_id AND p.role_id = r.id \
                    AND p.permission = 'users.manage' \
               WHERE mm.workspace_id = $1 AND mm.status = 'active' AND mm.deleted_at IS NULL) x",
    )
    .bind(ws)
    .bind(user)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(mine > 0 && others == 0)
}

/// Writes one audit row in each workspace of the user. The row names the user by UUID only.
async fn audit_everywhere(
    state: &AppState,
    user: Uuid,
    event: AuditEvent,
    new_value: serde_json::Value,
) -> ApiResult<()> {
    for (ws, _) in memberships(state, user).await? {
        let mut tx = db::begin(&state.db, &user_ctx(user, Some(ws))).await?;
        append(&mut tx, ws, user, event, user, "user", new_value.clone()).await?;
        tx.commit().await?;
    }
    Ok(())
}

async fn append(
    tx: &mut PgConnection,
    ws: Uuid,
    user: Uuid,
    event: AuditEvent,
    record: Uuid,
    kind: &str,
    new_value: serde_json::Value,
) -> ApiResult<()> {
    db::append_audit(
        tx,
        ws,
        &NewAuditEntry {
            actor_kind: ActorKind::User,
            actor_id: Some(user),
            event,
            record_kind: kind.to_owned(),
            record_id: Some(record),
            previous_value: None,
            new_value: Some(new_value),
            reason: None,
        },
    )
    .await?;
    Ok(())
}

/// Asks for the deletion of the account. The second step is required.
#[utoipa::path(
    post,
    path = "/v1/me/deletion",
    tag = "account",
    operation_id = "me_deletion_request",
    responses(
        (status = 202, description = "The request is saved; all sessions are closed", body = DeletionStatus,
            example = json!({"state": "pending", "requested_at": "2026-11-21T10:00:00Z",
                "purge_at": "2026-12-21T10:00:00Z", "retention_days": 30})),
        (status = 200, description = "A request was pending already", body = DeletionStatus)
    )
)]
pub async fn deletion_request(
    State(state): State<AppState>,
    Extension(svc): Svc,
    ctx: RequestContext,
) -> ApiResult<Response> {
    svc.require_step_up(&ctx, ProtectedAction::AccountDelete)
        .await?;
    let user = ctx.identity.user_id;
    for (ws, _) in memberships(&state, user).await? {
        if sole_business_admin(&state, user, ws).await? {
            return Err(conflict("/workspaces", "sole_business_admin"));
        }
    }
    let mut tx = db::begin(&state.db, &user_ctx(user, None)).await?;
    let (deletion, created) = db::account::request(&mut tx, user).await?;
    sqlx::query("DELETE FROM push_targets WHERE user_id = $1")
        .bind(user)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    if created {
        audit_everywhere(
            &state,
            user,
            AuditEvent::AccountDelete,
            json!({"state": "requested", "purge_at": deletion.purge_at}),
        )
        .await?;
    }
    // Also on a repeat: a failed first attempt must not leave a session open.
    let closed = svc.sessions.revoke_all(user, None).await?;
    tracing::info!(user_id = %user, closed, created, "account deletion requested");
    let status = if created {
        StatusCode::ACCEPTED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(DeletionStatus::of(Some(deletion)))).into_response())
}

/// Restores the account while the retention time runs.
#[utoipa::path(
    delete,
    path = "/v1/me/deletion",
    tag = "account",
    operation_id = "me_deletion_restore",
    responses(
        (status = 200, description = "The account is restored", body = DeletionStatus),
        (status = 404, description = "No pending request")
    )
)]
pub async fn deletion_restore(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> ApiResult<Json<DeletionStatus>> {
    let user = ctx.identity.user_id;
    let mut tx = db::begin(&state.db, &user_ctx(user, None)).await?;
    if !db::account::restore(&mut tx, user).await? {
        return Err(ApiError::new(ErrorCode::NotFound));
    }
    tx.commit().await?;
    audit_everywhere(
        &state,
        user,
        AuditEvent::AccountDelete,
        json!({"state": "restored"}),
    )
    .await?;
    Ok(Json(DeletionStatus::of(None)))
}

/// Reads the state of the deletion request.
#[utoipa::path(
    get,
    path = "/v1/me/deletion",
    tag = "account",
    operation_id = "me_deletion_get",
    responses((status = 200, description = "The state", body = DeletionStatus))
)]
pub async fn deletion_get(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> ApiResult<Json<DeletionStatus>> {
    let user = ctx.identity.user_id;
    let mut tx = db::begin(&state.db, &user_ctx(user, None)).await?;
    let pending = db::account::pending(&mut tx, user).await?;
    tx.commit().await?;
    Ok(Json(DeletionStatus::of(pending)))
}

/// Queues the export of all account data (ZIP). The second step is required.
#[utoipa::path(
    post,
    path = "/v1/me/data-export",
    tag = "account",
    operation_id = "me_data_export_create",
    request_body = NewDataExport,
    responses((status = 202, description = "The queued job", body = ExportJob))
)]
pub async fn data_export_create(
    State(state): State<AppState>,
    Extension(svc): Svc,
    ctx: RequestContext,
    ApiJson(new): ApiJson<NewDataExport>,
) -> ApiResult<Response> {
    svc.require_step_up(&ctx, ProtectedAction::FullExport)
        .await?;
    let user = ctx.identity.user_id;
    let all = memberships(&state, user).await?;
    let ws = match new.workspace_id {
        Some(w) if all.iter().any(|(id, _)| *id == w) => w,
        Some(_) => {
            return Err(ApiError::new(ErrorCode::ValidationFailed)
                .with_errors(vec![FieldIssue::new("/workspace_id", "not_found")]));
        }
        None => all
            .first()
            .map(|(id, _)| *id)
            .ok_or_else(|| conflict("/", "no_workspace"))?,
    };
    let mut tx = db::begin(&state.db, &user_ctx(user, Some(ws))).await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 31))")
        .bind(format!("account-export:{user}"))
        .execute(&mut *tx)
        .await?;
    // The limit counts the jobs of the user in all workspaces; each workspace is read in
    // its own context, so the check runs on the chosen one and on the account log below.
    let last: Option<DateTime<Utc>> = sqlx::query_scalar(
        "SELECT max(created_at) FROM export_jobs WHERE scope = 'account_data' \
         AND created_by = $1 AND status <> 'failed' \
         AND created_at > now() - make_interval(hours => $2::int)",
    )
    .bind(user)
    .bind(i32::try_from(DATA_EXPORT_INTERVAL_HOURS).unwrap_or(24))
    .fetch_one(&mut *tx)
    .await?;
    if let Some(last) = last {
        let retry = (last + chrono::Duration::hours(DATA_EXPORT_INTERVAL_HOURS) - Utc::now())
            .num_seconds()
            .max(1);
        return Err(ApiError::new(ErrorCode::RateLimited)
            .with_errors(vec![FieldIssue::new("/", "data_export_once_per_day")])
            .with_retry_after(u64::try_from(retry).unwrap_or(1)));
    }
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO export_jobs (id, workspace_id, scope, scope_id, format, params, created_by, \
         updated_by) VALUES ($1, $2, 'account_data', NULL, 'zip', '{}'::jsonb, $3, $3)",
    )
    .bind(id)
    .bind(ws)
    .bind(user)
    .execute(&mut *tx)
    .await?;
    append(
        &mut tx,
        ws,
        user,
        AuditEvent::DataExport,
        id,
        "export_job",
        json!({"scope": "account_data"}),
    )
    .await?;
    let row: ExportRow = sqlx::query_as(SELECT_EXPORT)
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    let job = export_job(row, &state)?;
    let mut response = (StatusCode::ACCEPTED, Json(job)).into_response();
    if let Ok(v) = format!("/v1/me/data-export/{ws}/{id}").parse() {
        response
            .headers_mut()
            .insert(axum::http::header::LOCATION, v);
    }
    Ok(response)
}

/// Reads an account data export job. A finished job carries a download link.
#[utoipa::path(
    get,
    path = "/v1/me/data-export/{workspace_id}/{id}",
    tag = "account",
    operation_id = "me_data_export_get",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace that carries the job"),
        ("id" = Uuid, Path, description = "Export job")
    ),
    responses((status = 200, description = "The job", body = ExportJob))
)]
pub async fn data_export_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<ExportJob>> {
    let user = ctx.identity.user_id;
    let mut tx = db::begin(&state.db, &user_ctx(user, Some(ws))).await?;
    let row: ExportRow = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{SELECT_EXPORT} AND created_by = $2 AND scope = 'account_data'"
    )))
    .bind(id)
    .bind(user)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| ApiError::new(ErrorCode::NotFound))?;
    tx.commit().await?;
    Ok(Json(export_job(row, &state)?))
}

// --- routes -------------------------------------------------------------------------------

/// The routes, wrapped with the sign-in layer. Off without the auth services.
pub fn routes(state: &AppState) -> Router<AppState> {
    if state.auth_services.is_none() {
        return Router::new();
    }
    crate::authenticated(
        state,
        Router::new()
            .route(
                "/v1/me/deletion",
                get(deletion_get)
                    .post(deletion_request)
                    .delete(deletion_restore),
            )
            .route("/v1/me/data-export", post(data_export_create))
            .route(
                "/v1/me/data-export/{workspace_id}/{id}",
                get(data_export_get),
            ),
    )
}

// --- registry entry ----------------------------------------------------------------------

mod spec {
    use crate::concurrency::Concurrency;
    use crate::error::ErrorCode::{
        Conflict, Forbidden, Internal, MalformedRequest, NotFound, RateLimited, StepUpRequired,
        Unauthenticated, Unavailable, UnsupportedMediaType, UpgradeRequired, ValidationFailed,
    };
    use crate::openapi::OperationSpec;

    pub(super) const OPERATIONS: &[OperationSpec] = &[
        OperationSpec {
            operation_id: "me_deletion_request",
            public: false,
            permission: None,
            concurrency: Some(Concurrency::Append),
            errors: &[
                MalformedRequest,
                Unauthenticated,
                Forbidden,
                StepUpRequired,
                Conflict,
                UpgradeRequired,
                RateLimited,
                Internal,
            ],
        },
        OperationSpec {
            operation_id: "me_deletion_restore",
            public: false,
            permission: None,
            concurrency: None,
            errors: &[
                Unauthenticated,
                NotFound,
                UpgradeRequired,
                RateLimited,
                Internal,
            ],
        },
        OperationSpec {
            operation_id: "me_deletion_get",
            public: false,
            permission: None,
            concurrency: None,
            errors: &[Unauthenticated, UpgradeRequired, RateLimited, Internal],
        },
        OperationSpec {
            operation_id: "me_data_export_create",
            public: false,
            permission: None,
            concurrency: Some(Concurrency::Append),
            errors: &[
                MalformedRequest,
                Unauthenticated,
                Forbidden,
                StepUpRequired,
                Conflict,
                UnsupportedMediaType,
                ValidationFailed,
                UpgradeRequired,
                RateLimited,
                Internal,
                Unavailable,
            ],
        },
        OperationSpec {
            operation_id: "me_data_export_get",
            public: false,
            permission: None,
            concurrency: None,
            errors: &[
                Unauthenticated,
                NotFound,
                UpgradeRequired,
                RateLimited,
                Internal,
            ],
        },
    ];
}

/// `OpenAPI` fragment of this module.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(
        deletion_request,
        deletion_restore,
        deletion_get,
        data_export_create,
        data_export_get
    ),
    components(schemas(DeletionStatus, NewDataExport)),
    tags((name = "account", description = "Account deletion and data export"))
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "account",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
