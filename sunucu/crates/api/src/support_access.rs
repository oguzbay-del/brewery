//! Support access of the customer (T2.30): give a grant, revoke it, and read the history
//! ("Destek erişim geçmişi").
//!
//! Support staff can read customer content only while a grant is valid. The customer gives it for
//! at most 72 hours (optionally for one support request) and can revoke it at any time. The
//! history lists every grant (the customer's and the emergency ones) and every operation of the
//! support team in the workspace. It shows the role of the staff member, not the name.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use chrono::{DateTime, Utc};
use db::{ActorKind, AuditEvent, NewAuditEntry};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    context::RequestContext,
    error::{ApiJson, ApiResult},
    library::{created_response, issue, not_found, workspace},
};

/// Permission that gives and reads support access.
pub const PERMISSION: &str = "support.grant";
/// Longest grant, in hours.
pub const MAX_HOURS: i32 = 72;

/// A grant.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct Grant {
    /// Identifier.
    pub id: Uuid,
    /// The support request the grant belongs to.
    pub request_id: Option<Uuid>,
    /// Start.
    pub starts_at: DateTime<Utc>,
    /// End (at most 72 hours after the start).
    pub ends_at: DateTime<Utc>,
    /// Revoked at.
    pub revoked_at: Option<DateTime<Utc>>,
    /// Opened by support as an emergency (two approvals); the customer is told afterwards.
    pub emergency: bool,
    /// Record version.
    pub version: i64,
}

const COLUMNS: &str = "id, request_id, starts_at, ends_at, revoked_at, emergency, version";

/// One operation of the support team.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct SupportActionView {
    /// When.
    pub at: DateTime<Utc>,
    /// What (`content.read.batches`, `request.reply`, ...).
    pub action: String,
    /// Kind of the record.
    pub record_kind: Option<String>,
    /// The reason that the staff member gave.
    pub reason: String,
    /// Role of the staff member (`support` or `support_admin`).
    pub staff_role: String,
}

/// Grants and operations.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AccessHistory {
    /// Grants, newest first.
    pub grants: Vec<Grant>,
    /// Operations, newest first (the last 200).
    pub actions: Vec<SupportActionView>,
}

/// A new grant.
#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct NewGrant {
    /// A support request of the workspace.
    pub request_id: Option<Uuid>,
    /// Hours, 1..=72 (default 72).
    pub hours: Option<i32>,
}

fn grant_example() -> serde_json::Value {
    json!({"id": "0192f1c07a3b7000800000000000000b", "request_id": null,
        "starts_at": "2026-11-28T12:00:00Z", "ends_at": "2026-12-01T12:00:00Z",
        "revoked_at": null, "emergency": false, "version": 1})
}

async fn audit_grant(
    conn: &mut sqlx::PgConnection,
    ws: Uuid,
    user: Uuid,
    grant: Uuid,
    what: &str,
) -> ApiResult<()> {
    db::append_audit(
        conn,
        ws,
        &NewAuditEntry {
            actor_kind: ActorKind::User,
            actor_id: Some(user),
            event: AuditEvent::SupportGrantChange,
            record_kind: "support_grant".to_owned(),
            record_id: Some(grant),
            previous_value: None,
            new_value: Some(json!({"change": what})),
            reason: None,
        },
    )
    .await?;
    Ok(())
}

/// Gives support access to the content of the workspace.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/support/grants",
    tag = "support",
    operation_id = "supportgrant_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = NewGrant,
    responses((status = 201, description = "The grant", body = Grant, example = json!(grant_example())))
)]
pub async fn supportgrant_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(new): ApiJson<NewGrant>,
) -> ApiResult<Response> {
    ctx.require(PERMISSION)?;
    let ws = workspace(&ctx)?;
    let hours = new.hours.unwrap_or(MAX_HOURS);
    if !(1..=MAX_HOURS).contains(&hours) {
        return Err(issue("/hours", "out_of_range"));
    }
    let user = ctx.identity.user_id;
    let mut tx = ctx.begin(&state.db).await?;
    if let Some(request) = new.request_id {
        let ok: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM support_requests WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(request)
        .fetch_optional(&mut *tx)
        .await?;
        if ok.is_none() {
            return Err(issue("/request_id", "not_found"));
        }
    }
    let sql = sqlx::AssertSqlSafe(format!(
        "INSERT INTO support_grants (id, workspace_id, request_id, granted_by_member_id, starts_at,
             ends_at, created_by, updated_by)
         SELECT $1, $2, $3, m.id, now(), now() + make_interval(hours => $4), $5, $5
         FROM workspace_members m WHERE m.user_id = $5 AND m.status = 'active'
         RETURNING {COLUMNS}"
    ));
    let grant: Grant = sqlx::query_as(sql)
        .bind(Uuid::now_v7())
        .bind(ws)
        .bind(new.request_id)
        .bind(hours)
        .bind(user)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(not_found)?;
    audit_grant(&mut tx, ws, user, grant.id, "given").await?;
    tx.commit().await?;
    let location = format!("/v1/workspaces/{ws}/support/grants/{}", grant.id);
    Ok(created_response(&location, grant.version, &grant))
}

/// Revokes a grant. The access stops at once.
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/support/grants/{id}",
    tag = "support",
    operation_id = "supportgrant_revoke",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Grant")
    ),
    responses((status = 204, description = "Revoked"))
)]
pub async fn supportgrant_revoke(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Response> {
    ctx.require(PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let n = sqlx::query(
        "UPDATE support_grants SET revoked_at = now(), updated_by = $2
         WHERE id = $1 AND revoked_at IS NULL AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if n == 0 {
        return Err(not_found());
    }
    audit_grant(&mut tx, ws, ctx.identity.user_id, id, "revoked").await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// The grants and the operations of the support team in the workspace.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/support/access-history",
    tag = "support",
    operation_id = "supportaccess_list",
    params(("workspace_id" = Uuid, Path, description = "Workspace")),
    responses((status = 200, description = "The history", body = AccessHistory,
        example = json!({"grants": [grant_example()], "actions": [{"at": "2026-11-28T12:05:00Z",
            "action": "content.read.batches", "record_kind": "support_grant",
            "reason": "Request #4182", "staff_role": "support"}]})))
)]
pub async fn supportaccess_list(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> ApiResult<Json<AccessHistory>> {
    ctx.require(PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let sql = sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM support_grants WHERE deleted_at IS NULL ORDER BY starts_at DESC LIMIT 100"
    ));
    let grants = sqlx::query_as(sql).fetch_all(&mut *tx).await?;
    let actions = sqlx::query_as(
        "SELECT a.created_at AS at, a.action, a.record_kind, a.reason, s.role AS staff_role
         FROM support_actions a JOIN staff_users s ON s.id = a.staff_id
         ORDER BY a.created_at DESC, a.id DESC LIMIT 200",
    )
    .fetch_all(&mut *tx)
    .await?;
    Ok(Json(AccessHistory { grants, actions }))
}

/// Routes of this module.
pub fn routes(state: &AppState) -> Router<AppState> {
    const BASE: &str = "/v1/workspaces/{ws}/support";
    let creates = crate::idempotent(
        state,
        Router::new().route(&format!("{BASE}/grants"), post(supportgrant_create)),
    );
    let others = Router::new()
        .route(
            &format!("{BASE}/grants/{{id}}"),
            delete(supportgrant_revoke),
        )
        .route(&format!("{BASE}/access-history"), get(supportaccess_list));
    crate::authenticated(state, creates.merge(others))
}

// --- registry entry ----------------------------------------------------------------------

mod spec {
    use crate::concurrency::Concurrency;
    use crate::error::ErrorCode;
    use crate::error::ErrorCode::{
        Conflict, Forbidden, IdempotencyInProgress, IdempotencyMismatch, Internal,
        MalformedRequest, NotFound, PayloadTooLarge, RateLimited, Unauthenticated, Unavailable,
        UnsupportedMediaType, UpgradeRequired, ValidationFailed,
    };
    use crate::openapi::{OperationSpec, file_op};

    const READ: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        ValidationFailed,
        RateLimited,
        UpgradeRequired,
        Internal,
        Unavailable,
    ];
    const CREATE: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        Conflict,
        IdempotencyInProgress,
        PayloadTooLarge,
        UnsupportedMediaType,
        ValidationFailed,
        IdempotencyMismatch,
        RateLimited,
        UpgradeRequired,
        Internal,
        Unavailable,
    ];
    const WRITE: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        Conflict,
        PayloadTooLarge,
        UnsupportedMediaType,
        ValidationFailed,
        RateLimited,
        UpgradeRequired,
        Internal,
        Unavailable,
    ];

    pub(super) const OPERATIONS: &[OperationSpec] = &[
        file_op(
            "supportgrant_create",
            Some("support.grant"),
            Some(Concurrency::Append),
            CREATE,
        ),
        file_op(
            "supportgrant_revoke",
            Some("support.grant"),
            Some(Concurrency::FieldLww),
            WRITE,
        ),
        file_op("supportaccess_list", Some("support.grant"), None, READ),
    ];
}

/// `OpenAPI` fragment of this module.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(supportgrant_create, supportgrant_revoke, supportaccess_list),
    components(schemas(Grant, SupportActionView, AccessHistory, NewGrant))
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "support_access",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
