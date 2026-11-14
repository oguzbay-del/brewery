//! Roles, assignments, members, invitations and Home sharing (ADR-0009 rules 6 and 8; T2.13).
//!
//! Everything here needs `users.manage` except accepting an invitation (any signed-in user
//! holding the token). Every mutation runs in one transaction that first locks the workspace
//! row, so the last-administrator and self-lockout guards cannot be raced, bumps
//! `workspaces.authz_version` (which retires the cached permissions) and appends to the audit
//! chain with `append_audit`.

#![allow(clippy::too_many_lines, clippy::single_match_else)]

use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
};
use chrono::{DateTime, Duration, Utc};
use db::{AuditEvent, NewAuditEntry, changed_field};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sqlx::{PgConnection, Postgres, Transaction};
use utoipa::ToSchema;
use uuid::Uuid;

use super::{
    AuthServices,
    email::{Mail, normalize_email},
    tokens::{new_token, token_hash},
};
use crate::{
    AppState, authenticated,
    authz::{Permission, ScopeKind},
    concurrency::{Concurrency, check_if_match, record_response},
    context::RequestContext,
    error::{ApiError, ApiJson, ApiResult, ErrorCode, FieldIssue, MergePatch},
    idempotent,
};

/// SQL assembled from the constants of this module only (never from request data).
macro_rules! sql {
    ($($arg:tt)*) => {
        sqlx::AssertSqlSafe(format!($($arg)*))
    };
}

type Svc = Extension<Arc<AuthServices>>;
type Tx = Transaction<'static, Postgres>;

/// How long an invitation works (ADR-0009 rule 6.1).
pub const INVITATION_TTL_DAYS: i64 = 7;

/// The routes of this module: roles, members, assignments, invitations and shares.
pub fn routes(state: &AppState) -> Router<AppState> {
    const WS: &str = "/v1/workspaces/{ws}";
    // Without the auth services (see `AppState::with_auth_services`) these routes are off.
    if state.auth_services.is_none() {
        return Router::new();
    }
    let plain = Router::new()
        .route(&format!("{WS}/roles"), get(roles_list))
        .route(
            &format!("{WS}/roles/{{id}}"),
            patch(roles_update).delete(roles_delete),
        )
        .route(&format!("{WS}/members"), get(members_list))
        .route(
            &format!("{WS}/members/{{id}}"),
            patch(members_update).delete(members_remove),
        )
        .route(
            &format!("{WS}/members/{{id}}/assignments/{{aid}}"),
            delete(assignments_delete),
        )
        .route(&format!("{WS}/invitations"), get(invitations_list))
        .route(
            &format!("{WS}/invitations/{{id}}"),
            delete(invitations_revoke),
        )
        .route("/v1/invitations/accept", post(invitations_accept));
    let creates = idempotent(
        state,
        Router::new()
            .route(&format!("{WS}/roles"), post(roles_create))
            .route(&format!("{WS}/roles/{{id}}/copy"), post(roles_copy))
            .route(
                &format!("{WS}/members/{{id}}/assignments"),
                post(assignments_create),
            )
            .route(&format!("{WS}/invitations"), post(invitations_create))
            .route(&format!("{WS}/shares"), post(shares_create)),
    );
    authenticated(state, plain.merge(creates))
}

// --- shared helpers -------------------------------------------------------------------------

fn invalid(field: &str, code: &str) -> ApiError {
    ApiError::validation(vec![FieldIssue::new(field, code)])
}

fn conflict(field: &str, code: &str) -> ApiError {
    ApiError::new(ErrorCode::Conflict).with_errors(vec![FieldIssue::new(field, code)])
}

fn not_found() -> ApiError {
    ApiError::new(ErrorCode::NotFound)
}

fn pg_code(e: &sqlx::Error) -> Option<String> {
    e.as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .map(std::borrow::Cow::into_owned)
}

fn ws_of(ctx: &RequestContext) -> ApiResult<Uuid> {
    ctx.workspace_id.ok_or_else(not_found)
}

/// Opens the request transaction for a users.manage mutation: checks the permission, locks the
/// workspace row (serializes guards) and finds the caller's member id.
async fn begin_admin(state: &AppState, ctx: &RequestContext) -> ApiResult<(Tx, Uuid, Uuid)> {
    ctx.require_permission(Permission::UsersManage)?;
    let ws = ws_of(ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    sqlx::query("SELECT id FROM workspaces WHERE id = $1 FOR UPDATE")
        .bind(ws)
        .fetch_one(&mut *tx)
        .await?;
    let caller: Uuid = sqlx::query_scalar(
        "SELECT id FROM workspace_members \
         WHERE workspace_id = $1 AND user_id = $2 AND deleted_at IS NULL",
    )
    .bind(ws)
    .bind(ctx.identity.user_id)
    .fetch_one(&mut *tx)
    .await?;
    Ok((tx, ws, caller))
}

async fn bump_authz(tx: &mut PgConnection, ws: Uuid, user: Uuid) -> ApiResult<()> {
    sqlx::query(
        "UPDATE workspaces SET authz_version = authz_version + 1, updated_by = $2 WHERE id = $1",
    )
    .bind(ws)
    .bind(user)
    .execute(tx)
    .await?;
    Ok(())
}

/// The protections of plan > Ayarlarda yönetim, checked after a change inside its transaction:
/// at least one active administrator (`users.manage` for the whole workspace) remains and the
/// caller keeps their own `users.manage`.
async fn guard(tx: &mut PgConnection, ws: Uuid, caller_member: Uuid) -> ApiResult<()> {
    const HOLDERS: &str = "SELECT count(DISTINCT m.id) FROM workspace_members m \
        JOIN role_assignments a ON a.workspace_id = m.workspace_id AND a.member_id = m.id \
             AND a.deleted_at IS NULL AND a.scope_kind IS NULL \
        JOIN roles r ON r.workspace_id = a.workspace_id AND r.id = a.role_id AND r.deleted_at IS NULL \
        JOIN role_permissions p ON p.workspace_id = r.workspace_id AND p.role_id = r.id \
             AND p.permission = 'users.manage' \
        WHERE m.workspace_id = $1 AND m.status = 'active' AND m.deleted_at IS NULL";
    let admins: i64 = sqlx::query_scalar(HOLDERS)
        .bind(ws)
        .fetch_one(&mut *tx)
        .await?;
    if admins == 0 {
        return Err(conflict("/", "last_admin"));
    }
    let own: i64 = sqlx::query_scalar(sql!("{HOLDERS} AND m.id = $2"))
        .bind(ws)
        .bind(caller_member)
        .fetch_one(&mut *tx)
        .await?;
    if own == 0 {
        return Err(conflict("/", "self_lockout"));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn audit(
    tx: &mut PgConnection,
    ctx: &RequestContext,
    ws: Uuid,
    event: AuditEvent,
    kind: &str,
    id: Uuid,
    previous: Option<Value>,
    new: Option<Value>,
) -> ApiResult<()> {
    db::append_audit(
        tx,
        ws,
        &NewAuditEntry {
            actor_kind: ctx.identity.actor_kind,
            actor_id: Some(ctx.identity.user_id),
            event,
            record_kind: kind.to_owned(),
            record_id: Some(id),
            previous_value: previous,
            new_value: new,
            reason: None,
        },
    )
    .await?;
    Ok(())
}

fn valid_name(name: &str) -> bool {
    let n = name.trim();
    !n.is_empty() && n.chars().count() <= 100 && !n.chars().any(char::is_control)
}

fn valid_landing(s: &str) -> bool {
    s.len() <= 50
        && s.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn parse_permissions(list: &[String]) -> Result<Vec<String>, ApiError> {
    let mut out = Vec::new();
    let mut issues = Vec::new();
    for (i, code) in list.iter().enumerate() {
        match Permission::parse(code) {
            Some(p) if !out.iter().any(|x: &String| x == p.as_str()) => {
                out.push(p.as_str().to_owned());
            }
            Some(_) => {}
            None => issues.push(FieldIssue::new(
                format!("/permissions/{i}"),
                "unknown_permission",
            )),
        }
    }
    if issues.is_empty() {
        out.sort();
        Ok(out)
    } else {
        Err(ApiError::validation(issues))
    }
}

fn parse_scope(kind: Option<&str>, id: Option<Uuid>) -> ApiResult<Option<(ScopeKind, Uuid)>> {
    match (kind, id) {
        (None, None) => Ok(None),
        (Some(k), Some(id)) => ScopeKind::parse(k)
            .map(|k| Some((k, id)))
            .ok_or_else(|| invalid("/scope_kind", "invalid_choice")),
        (Some(_), None) => Err(invalid("/scope_id", "required")),
        (None, Some(_)) => Err(invalid("/scope_kind", "required")),
    }
}

/// A role holding a workspace-level permission cannot be scoped.
async fn ensure_scopable(tx: &mut PgConnection, ws: Uuid, role_id: Uuid) -> ApiResult<()> {
    let perms: Vec<String> = sqlx::query_scalar(
        "SELECT permission FROM role_permissions WHERE workspace_id = $1 AND role_id = $2",
    )
    .bind(ws)
    .bind(role_id)
    .fetch_all(tx)
    .await?;
    if perms
        .iter()
        .filter_map(|p| Permission::parse(p))
        .any(Permission::workspace_level)
    {
        return Err(invalid("/role_id", "role_not_scopable"));
    }
    Ok(())
}

async fn live_role(tx: &mut PgConnection, ws: Uuid, id: Uuid) -> ApiResult<()> {
    let n: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM roles WHERE workspace_id = $1 AND id = $2 AND deleted_at IS NULL",
    )
    .bind(ws)
    .bind(id)
    .fetch_one(tx)
    .await?;
    if n == 0 {
        return Err(invalid("/role_id", "not_found"));
    }
    Ok(())
}

// --- roles ----------------------------------------------------------------------------------

/// A role with its permissions.
#[derive(Debug, Serialize, ToSchema, sqlx::FromRow)]
pub struct RoleResponse {
    /// Role id.
    pub id: Uuid,
    /// Name.
    pub name: String,
    /// Preset the role came from (`admin`, `brewmaster`, ...), if any.
    pub template_key: Option<String>,
    /// Screen opened after sign-in.
    pub landing_screen: Option<String>,
    /// Permission codes, sorted.
    pub permissions: Vec<String>,
    /// Record version (`ETag`).
    pub version: i64,
    /// Creation time.
    pub created_at: DateTime<Utc>,
    /// Last change.
    pub updated_at: DateTime<Utc>,
}

const ROLE_SELECT: &str = "SELECT r.id, r.name, r.template_key, r.landing_screen, r.version, \
    r.created_at, r.updated_at, \
    COALESCE(array_agg(p.permission ORDER BY p.permission) FILTER (WHERE p.permission IS NOT NULL), \
             '{}') AS permissions \
    FROM roles r LEFT JOIN role_permissions p \
      ON p.workspace_id = r.workspace_id AND p.role_id = r.id \
    WHERE r.workspace_id = $1 AND r.deleted_at IS NULL";

async fn load_role(tx: &mut PgConnection, ws: Uuid, id: Uuid) -> ApiResult<RoleResponse> {
    sqlx::query_as::<_, RoleResponse>(sql!("{ROLE_SELECT} AND r.id = $2 GROUP BY r.id"))
        .bind(ws)
        .bind(id)
        .fetch_optional(tx)
        .await?
        .ok_or_else(not_found)
}

async fn set_permissions(
    tx: &mut PgConnection,
    ws: Uuid,
    role: Uuid,
    user: Uuid,
    perms: &[String],
) -> ApiResult<()> {
    sqlx::query("DELETE FROM role_permissions WHERE workspace_id = $1 AND role_id = $2")
        .bind(ws)
        .bind(role)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO role_permissions (workspace_id, role_id, permission, created_by) \
         SELECT $1, $2, p, $3 FROM unnest($4::text[]) AS p",
    )
    .bind(ws)
    .bind(role)
    .bind(user)
    .bind(perms)
    .execute(tx)
    .await?;
    Ok(())
}

fn role_json(r: &RoleResponse) -> Value {
    serde_json::to_value(r).unwrap_or(Value::Null)
}

/// List of roles.
#[derive(Debug, Serialize, ToSchema)]
pub struct RoleList {
    /// The roles.
    pub items: Vec<RoleResponse>,
}

/// Lists the workspace's roles.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/roles",
    tag = "access",
    operation_id = "roles_list",
    params(("workspace_id" = Uuid, Path, description = "Workspace")),
    responses((
        status = 200,
        description = "Roles with their permissions",
        body = RoleList,
        example = json!({"items": [{
            "id": "0192f1c07a3b7000800000000000000e", "name": "Operator", "template_key": "operator",
            "landing_screen": "today", "permissions": ["batch.record", "quality.record", "recipe.view"],
            "version": 1, "created_at": "2026-11-14T10:00:00Z", "updated_at": "2026-11-14T10:00:00Z"
        }]})
    ))
)]
pub async fn roles_list(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> ApiResult<Json<RoleList>> {
    ctx.require_permission(Permission::UsersManage)?;
    let ws = ws_of(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    let items = sqlx::query_as::<_, RoleResponse>(sql!(
        "{ROLE_SELECT} GROUP BY r.id ORDER BY lower(r.name), r.id"
    ))
    .bind(ws)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(RoleList { items }))
}

/// Body of `POST /roles`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NewRole {
    /// Name (1 to 100 characters, unique in the workspace).
    pub name: String,
    /// Screen opened after sign-in (`lower_snake`).
    pub landing_screen: Option<String>,
    /// Permission codes.
    pub permissions: Vec<String>,
}

/// Creates a role from scratch.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/roles",
    tag = "access",
    operation_id = "roles_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = NewRole,
    responses((
        status = 201,
        description = "Role created",
        body = RoleResponse,
        example = json!({
            "id": "0192f1c07a3b7000800000000000000e", "name": "Cellar", "template_key": null,
            "landing_screen": null, "permissions": ["batch.record", "recipe.view"],
            "version": 1, "created_at": "2026-11-14T10:00:00Z", "updated_at": "2026-11-14T10:00:00Z"
        })
    ))
)]
pub async fn roles_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(req): ApiJson<NewRole>,
) -> ApiResult<Response> {
    let (mut tx, ws, _) = begin_admin(&state, &ctx).await?;
    let mut issues = Vec::new();
    if !valid_name(&req.name) {
        issues.push(FieldIssue::new("/name", "invalid"));
    }
    if req
        .landing_screen
        .as_deref()
        .is_some_and(|s| !valid_landing(s))
    {
        issues.push(FieldIssue::new("/landing_screen", "invalid"));
    }
    let perms = match parse_permissions(&req.permissions) {
        Ok(p) => p,
        Err(e) => {
            let mut all = issues;
            all.extend(e.errors().iter().cloned());
            return Err(ApiError::validation(all));
        }
    };
    if !issues.is_empty() {
        return Err(ApiError::validation(issues));
    }
    let id = Uuid::now_v7();
    insert_role(
        &mut tx,
        ws,
        id,
        ctx.identity.user_id,
        req.name.trim(),
        None,
        req.landing_screen.as_deref(),
        &perms,
    )
    .await?;
    let role = load_role(&mut tx, ws, id).await?;
    audit(
        &mut tx,
        &ctx,
        ws,
        AuditEvent::RoleCreate,
        "role",
        id,
        None,
        Some(json!({"name": changed_field(), "permissions": role.permissions})),
    )
    .await?;
    tx.commit().await?;
    Ok(record_response(StatusCode::CREATED, role.version, &role))
}

#[allow(clippy::too_many_arguments)]
async fn insert_role(
    tx: &mut PgConnection,
    ws: Uuid,
    id: Uuid,
    user: Uuid,
    name: &str,
    template_key: Option<&str>,
    landing: Option<&str>,
    perms: &[String],
) -> ApiResult<()> {
    let r = sqlx::query(
        "INSERT INTO roles (id, workspace_id, name, template_key, landing_screen, created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $6)",
    )
    .bind(id)
    .bind(ws)
    .bind(name)
    .bind(template_key)
    .bind(landing)
    .bind(user)
    .execute(&mut *tx)
    .await;
    if let Err(e) = r {
        return Err(if pg_code(&e).as_deref() == Some("23505") {
            conflict("/name", "name_taken")
        } else {
            e.into()
        });
    }
    set_permissions(tx, ws, id, user, perms).await
}

/// Body of `POST /roles/{id}/copy`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CopyRole {
    /// Name of the copy.
    pub name: String,
}

/// Copies a role (preset or custom) under a new name.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/roles/{id}/copy",
    tag = "access",
    operation_id = "roles_copy",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Role to copy"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = CopyRole,
    responses((
        status = 201,
        description = "The copy",
        body = RoleResponse,
        example = json!({
            "id": "0192f1c07a3b7000800000000000000f", "name": "Night shift", "template_key": "operator",
            "landing_screen": "today", "permissions": ["batch.record", "quality.record", "recipe.view"],
            "version": 1, "created_at": "2026-11-14T10:00:00Z", "updated_at": "2026-11-14T10:00:00Z"
        })
    ))
)]
pub async fn roles_copy(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, source)): Path<(Uuid, Uuid)>,
    ApiJson(req): ApiJson<CopyRole>,
) -> ApiResult<Response> {
    let (mut tx, ws, _) = begin_admin(&state, &ctx).await?;
    if !valid_name(&req.name) {
        return Err(invalid("/name", "invalid"));
    }
    let from = load_role(&mut tx, ws, source).await?;
    let id = Uuid::now_v7();
    insert_role(
        &mut tx,
        ws,
        id,
        ctx.identity.user_id,
        req.name.trim(),
        from.template_key.as_deref(),
        from.landing_screen.as_deref(),
        &from.permissions,
    )
    .await?;
    let role = load_role(&mut tx, ws, id).await?;
    audit(
        &mut tx,
        &ctx,
        ws,
        AuditEvent::RoleCreate,
        "role",
        id,
        None,
        Some(json!({
            "name": changed_field(),
            "permissions": role.permissions,
            "copied_from": source
        })),
    )
    .await?;
    tx.commit().await?;
    Ok(record_response(StatusCode::CREATED, role.version, &role))
}

/// Merge-patch body of `PATCH /roles/{id}` (documentation shape; `landing_screen: null` clears).
#[derive(Debug, Deserialize, ToSchema)]
pub struct RolePatch {
    /// New name.
    pub name: Option<String>,
    /// New landing screen.
    pub landing_screen: Option<String>,
    /// The new complete permission set.
    pub permissions: Option<Vec<String>>,
}

/// Changes a role's name, landing screen or permissions. `If-Match` is required.
#[utoipa::path(
    patch,
    path = "/v1/workspaces/{workspace_id}/roles/{id}",
    tag = "access",
    operation_id = "roles_update",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Role"),
        ("If-Match" = String, Header, description = "Version `ETag` of the role")
    ),
    request_body(content = RolePatch, content_type = "application/merge-patch+json"),
    responses((
        status = 200,
        description = "The updated role",
        body = RoleResponse,
        example = json!({
            "id": "0192f1c07a3b7000800000000000000e", "name": "Cellar", "template_key": null,
            "landing_screen": null, "permissions": ["batch.record", "recipe.view", "stock.move"],
            "version": 2, "created_at": "2026-11-14T10:00:00Z", "updated_at": "2026-11-14T10:05:00Z"
        })
    ))
)]
pub async fn roles_update(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    MergePatch(patch): MergePatch<Value>,
) -> ApiResult<Response> {
    let (mut tx, ws, caller) = begin_admin(&state, &ctx).await?;
    let Value::Object(patch) = patch else {
        return Err(invalid("/", "invalid"));
    };
    let mut issues = Vec::new();
    for k in patch.keys() {
        if !matches!(k.as_str(), "name" | "landing_screen" | "permissions") {
            issues.push(FieldIssue::new(format!("/{k}"), "unknown_field"));
        }
    }
    let name = match patch.get("name") {
        None => None,
        Some(Value::String(s)) if valid_name(s) => Some(s.trim().to_owned()),
        Some(_) => {
            issues.push(FieldIssue::new("/name", "invalid"));
            None
        }
    };
    let landing = match patch.get("landing_screen") {
        None => None,
        Some(Value::Null) => Some(None),
        Some(Value::String(s)) if valid_landing(s) => Some(Some(s.clone())),
        Some(_) => {
            issues.push(FieldIssue::new("/landing_screen", "invalid"));
            None
        }
    };
    let perms = match patch.get("permissions") {
        None => None,
        Some(v) => match serde_json::from_value::<Vec<String>>(v.clone()) {
            Ok(list) => match parse_permissions(&list) {
                Ok(p) => Some(p),
                Err(e) => {
                    issues.extend(e.errors().iter().cloned());
                    None
                }
            },
            Err(_) => {
                issues.push(FieldIssue::new("/permissions", "invalid"));
                None
            }
        },
    };
    if !issues.is_empty() {
        return Err(ApiError::validation(issues));
    }
    let before = load_role(&mut tx, ws, id).await?;
    check_if_match(Concurrency::Optimistic, &headers, before.version, || {
        role_json(&before)
    })?;
    let user = ctx.identity.user_id;
    let res = sqlx::query(
        "UPDATE roles SET name = COALESCE($3, name), \
             landing_screen = CASE WHEN $4 THEN $5 ELSE landing_screen END, updated_by = $6 \
         WHERE workspace_id = $1 AND id = $2",
    )
    .bind(ws)
    .bind(id)
    .bind(name.as_deref())
    .bind(landing.is_some())
    .bind(landing.clone().flatten())
    .bind(user)
    .execute(&mut *tx)
    .await;
    if let Err(e) = res {
        return Err(if pg_code(&e).as_deref() == Some("23505") {
            conflict("/name", "name_taken")
        } else {
            e.into()
        });
    }
    let mut changed_perms = false;
    if let Some(p) = &perms
        && *p != before.permissions
    {
        set_permissions(&mut tx, ws, id, user, p).await?;
        bump_authz(&mut tx, ws, user).await?;
        guard(&mut tx, ws, caller).await?;
        changed_perms = true;
    }
    let after = load_role(&mut tx, ws, id).await?;
    if changed_perms {
        audit(
            &mut tx,
            &ctx,
            ws,
            AuditEvent::RolePermissionsSet,
            "role",
            id,
            Some(json!({"permissions": before.permissions})),
            Some(json!({"permissions": after.permissions})),
        )
        .await?;
    }
    if after.name != before.name || after.landing_screen != before.landing_screen {
        audit(
            &mut tx,
            &ctx,
            ws,
            AuditEvent::RoleUpdate,
            "role",
            id,
            Some(json!({"name": changed_field(), "landing_screen": before.landing_screen})),
            Some(json!({"name": changed_field(), "landing_screen": after.landing_screen})),
        )
        .await?;
    }
    tx.commit().await?;
    Ok(record_response(StatusCode::OK, after.version, &after))
}

/// Deletes a role nobody holds; its pending invitations are revoked. `If-Match` is required.
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/roles/{id}",
    tag = "access",
    operation_id = "roles_delete",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Role"),
        ("If-Match" = String, Header, description = "Version `ETag` of the role")
    ),
    responses((status = 204, description = "Deleted"))
)]
pub async fn roles_delete(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let (mut tx, ws, _) = begin_admin(&state, &ctx).await?;
    let before = load_role(&mut tx, ws, id).await?;
    check_if_match(Concurrency::Optimistic, &headers, before.version, || {
        role_json(&before)
    })?;
    let in_use: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM role_assignments \
         WHERE workspace_id = $1 AND role_id = $2 AND deleted_at IS NULL",
    )
    .bind(ws)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    if in_use > 0 {
        return Err(conflict("/", "role_in_use"));
    }
    sqlx::query(
        "UPDATE invitations SET revoked_at = now(), updated_by = $3 \
         WHERE workspace_id = $1 AND role_id = $2 AND accepted_at IS NULL AND revoked_at IS NULL",
    )
    .bind(ws)
    .bind(id)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE roles SET deleted_at = now(), updated_by = $3 WHERE workspace_id = $1 AND id = $2",
    )
    .bind(ws)
    .bind(id)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await?;
    audit(
        &mut tx,
        &ctx,
        ws,
        AuditEvent::RoleDelete,
        "role",
        id,
        Some(json!({"name": changed_field(), "permissions": before.permissions})),
        None,
    )
    .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

// --- members and assignments ---------------------------------------------------------------

/// One role assignment.
#[derive(Debug, Clone, Serialize, ToSchema, sqlx::FromRow)]
pub struct AssignmentResponse {
    /// Assignment id.
    pub id: Uuid,
    /// The member.
    pub member_id: Uuid,
    /// The role.
    pub role_id: Uuid,
    /// `recipe`, `batch`, `vessel` or `area`; absent for the whole workspace.
    pub scope_kind: Option<String>,
    /// The scoped record.
    pub scope_id: Option<Uuid>,
    /// Creation time.
    pub created_at: DateTime<Utc>,
}

/// A workspace member with assignments.
#[derive(Debug, Serialize, ToSchema)]
pub struct MemberResponse {
    /// Member id.
    pub id: Uuid,
    /// The user.
    pub user_id: Uuid,
    /// `active` or `disabled`.
    pub status: String,
    /// Joined at.
    pub joined_at: DateTime<Utc>,
    /// Record version.
    pub version: i64,
    /// Role assignments.
    pub assignments: Vec<AssignmentResponse>,
}

/// List of members.
#[derive(Debug, Serialize, ToSchema)]
pub struct MemberList {
    /// The members.
    pub items: Vec<MemberResponse>,
}

#[derive(sqlx::FromRow)]
struct MemberRow {
    id: Uuid,
    user_id: Uuid,
    status: String,
    joined_at: DateTime<Utc>,
    version: i64,
}

const ASSIGNMENT_COLS: &str = "id, member_id, role_id, scope_kind, scope_id, created_at";

async fn load_members(
    tx: &mut PgConnection,
    ws: Uuid,
    only: Option<Uuid>,
) -> ApiResult<Vec<MemberResponse>> {
    let rows: Vec<MemberRow> = sqlx::query_as(
        "SELECT id, user_id, status, joined_at, version FROM workspace_members \
         WHERE workspace_id = $1 AND deleted_at IS NULL AND ($2::uuid IS NULL OR id = $2) \
         ORDER BY joined_at, id",
    )
    .bind(ws)
    .bind(only)
    .fetch_all(&mut *tx)
    .await?;
    let assignments: Vec<AssignmentResponse> = sqlx::query_as(sql!(
        "SELECT {ASSIGNMENT_COLS} FROM role_assignments \
         WHERE workspace_id = $1 AND deleted_at IS NULL AND ($2::uuid IS NULL OR member_id = $2) \
         ORDER BY created_at, id"
    ))
    .bind(ws)
    .bind(only)
    .fetch_all(tx)
    .await?;
    Ok(rows
        .into_iter()
        .map(|m| MemberResponse {
            assignments: assignments
                .iter()
                .filter(|a| a.member_id == m.id)
                .cloned()
                .collect(),
            id: m.id,
            user_id: m.user_id,
            status: m.status,
            joined_at: m.joined_at,
            version: m.version,
        })
        .collect())
}

/// Lists the members with their role assignments.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/members",
    tag = "access",
    operation_id = "members_list",
    params(("workspace_id" = Uuid, Path, description = "Workspace")),
    responses((
        status = 200,
        description = "Members",
        body = MemberList,
        example = json!({"items": [{
            "id": "0192f1c07a3b70008000000000000010", "user_id": "0192f1c07a3b70008000000000000011",
            "status": "active", "joined_at": "2026-11-14T10:00:00Z", "version": 1,
            "assignments": [{"id": "0192f1c07a3b70008000000000000012",
                "member_id": "0192f1c07a3b70008000000000000010",
                "role_id": "0192f1c07a3b7000800000000000000e", "scope_kind": null,
                "scope_id": null, "created_at": "2026-11-14T10:00:00Z"}]
        }]})
    ))
)]
pub async fn members_list(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> ApiResult<Json<MemberList>> {
    ctx.require_permission(Permission::UsersManage)?;
    let ws = ws_of(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    let items = load_members(&mut tx, ws, None).await?;
    tx.commit().await?;
    Ok(Json(MemberList { items }))
}

/// Merge-patch body of `PATCH /members/{id}`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct MemberPatch {
    /// `active` or `disabled`.
    pub status: Option<String>,
}

/// Disables or re-enables a member. A disabled member loses access at once.
#[utoipa::path(
    patch,
    path = "/v1/workspaces/{workspace_id}/members/{id}",
    tag = "access",
    operation_id = "members_update",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Member")
    ),
    request_body(content = MemberPatch, content_type = "application/merge-patch+json"),
    responses((
        status = 200,
        description = "The member",
        body = MemberResponse,
        example = json!({
            "id": "0192f1c07a3b70008000000000000010", "user_id": "0192f1c07a3b70008000000000000011",
            "status": "disabled", "joined_at": "2026-11-14T10:00:00Z", "version": 2, "assignments": []
        })
    ))
)]
pub async fn members_update(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    MergePatch(patch): MergePatch<Map<String, Value>>,
) -> ApiResult<Response> {
    let (mut tx, ws, caller) = begin_admin(&state, &ctx).await?;
    let mut issues: Vec<_> = patch
        .keys()
        .filter(|k| k.as_str() != "status")
        .map(|k| FieldIssue::new(format!("/{k}"), "unknown_field"))
        .collect();
    let status = match patch.get("status") {
        Some(Value::String(s)) if matches!(s.as_str(), "active" | "disabled") => Some(s.clone()),
        None => None,
        Some(_) => {
            issues.push(FieldIssue::new("/status", "invalid_choice"));
            None
        }
    };
    if !issues.is_empty() {
        return Err(ApiError::validation(issues));
    }
    let current = load_members(&mut tx, ws, Some(id))
        .await?
        .pop()
        .ok_or_else(not_found)?;
    check_if_match(Concurrency::FieldLww, &headers, current.version, || {
        serde_json::to_value(&current).unwrap_or(Value::Null)
    })?;
    if let Some(s) = status.filter(|s| *s != current.status) {
        if id == caller && s == "disabled" {
            return Err(conflict("/status", "self_lockout"));
        }
        sqlx::query(
            "UPDATE workspace_members SET status = $3, updated_by = $4 \
             WHERE workspace_id = $1 AND id = $2",
        )
        .bind(ws)
        .bind(id)
        .bind(&s)
        .bind(ctx.identity.user_id)
        .execute(&mut *tx)
        .await?;
        bump_authz(&mut tx, ws, ctx.identity.user_id).await?;
        guard(&mut tx, ws, caller).await?;
        audit(
            &mut tx,
            &ctx,
            ws,
            AuditEvent::MemberStatusChange,
            "workspace_member",
            id,
            Some(json!({"status": current.status})),
            Some(json!({"status": s})),
        )
        .await?;
    }
    let member = load_members(&mut tx, ws, Some(id))
        .await?
        .pop()
        .ok_or_else(not_found)?;
    tx.commit().await?;
    Ok(record_response(StatusCode::OK, member.version, &member))
}

/// Removes a member from the workspace together with their assignments.
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/members/{id}",
    tag = "access",
    operation_id = "members_remove",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Member")
    ),
    responses((status = 204, description = "Removed"))
)]
pub async fn members_remove(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Response> {
    let (mut tx, ws, caller) = begin_admin(&state, &ctx).await?;
    let current = load_members(&mut tx, ws, Some(id))
        .await?
        .pop()
        .ok_or_else(not_found)?;
    if id == caller {
        return Err(conflict("/", "self_lockout"));
    }
    let user = ctx.identity.user_id;
    sqlx::query(
        "UPDATE role_assignments SET deleted_at = now(), updated_by = $3 \
         WHERE workspace_id = $1 AND member_id = $2 AND deleted_at IS NULL",
    )
    .bind(ws)
    .bind(id)
    .bind(user)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE workspace_members SET deleted_at = now(), updated_by = $3 \
         WHERE workspace_id = $1 AND id = $2",
    )
    .bind(ws)
    .bind(id)
    .bind(user)
    .execute(&mut *tx)
    .await?;
    bump_authz(&mut tx, ws, user).await?;
    guard(&mut tx, ws, caller).await?;
    audit(
        &mut tx,
        &ctx,
        ws,
        AuditEvent::MemberRemove,
        "workspace_member",
        id,
        Some(json!({"status": current.status, "assignments": current.assignments.len()})),
        None,
    )
    .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Body of `POST /members/{id}/assignments`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NewAssignment {
    /// The role.
    pub role_id: Uuid,
    /// `recipe`, `batch`, `vessel` or `area`; omit for the whole workspace.
    pub scope_kind: Option<String>,
    /// The scoped record (required with `scope_kind`).
    pub scope_id: Option<Uuid>,
}

/// Gives a member a role, for the whole workspace or one recipe, batch, vessel or area.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/members/{id}/assignments",
    tag = "access",
    operation_id = "assignments_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Member"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = NewAssignment,
    responses((
        status = 201,
        description = "Assignment created",
        body = AssignmentResponse,
        example = json!({"id": "0192f1c07a3b70008000000000000012",
            "member_id": "0192f1c07a3b70008000000000000010",
            "role_id": "0192f1c07a3b7000800000000000000e", "scope_kind": "recipe",
            "scope_id": "0192f1c07a3b70008000000000000013", "created_at": "2026-11-14T10:00:00Z"})
    ))
)]
pub async fn assignments_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, member)): Path<(Uuid, Uuid)>,
    ApiJson(req): ApiJson<NewAssignment>,
) -> ApiResult<Response> {
    let (mut tx, ws, _) = begin_admin(&state, &ctx).await?;
    let scope = parse_scope(req.scope_kind.as_deref(), req.scope_id)?;
    let exists: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM workspace_members WHERE workspace_id = $1 AND id = $2 AND deleted_at IS NULL",
    )
    .bind(ws)
    .bind(member)
    .fetch_one(&mut *tx)
    .await?;
    if exists == 0 {
        return Err(not_found());
    }
    live_role(&mut tx, ws, req.role_id).await?;
    if scope.is_some() {
        ensure_scopable(&mut tx, ws, req.role_id).await?;
    }
    let id = Uuid::now_v7();
    let user = ctx.identity.user_id;
    let res = sqlx::query(
        "INSERT INTO role_assignments (id, workspace_id, member_id, role_id, scope_kind, scope_id, \
                                       created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $7)",
    )
    .bind(id)
    .bind(ws)
    .bind(member)
    .bind(req.role_id)
    .bind(scope.map(|s| s.0.as_str()))
    .bind(scope.map(|s| s.1))
    .bind(user)
    .execute(&mut *tx)
    .await;
    if let Err(e) = res {
        return Err(match pg_code(&e).as_deref() {
            Some("23505") => conflict("/role_id", "already_assigned"),
            Some("23503") => invalid("/scope_id", "scope_target_not_found"),
            _ => e.into(),
        });
    }
    bump_authz(&mut tx, ws, user).await?;
    audit(
        &mut tx,
        &ctx,
        ws,
        AuditEvent::RoleAssign,
        "role_assignment",
        id,
        None,
        Some(json!({
            "member_id": member, "role_id": req.role_id,
            "scope_kind": scope.map(|s| s.0.as_str()), "scope_id": scope.map(|s| s.1)
        })),
    )
    .await?;
    let created: AssignmentResponse = sqlx::query_as(sql!(
        "SELECT {ASSIGNMENT_COLS} FROM role_assignments WHERE workspace_id = $1 AND id = $2"
    ))
    .bind(ws)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(created)).into_response())
}

/// Removes a role assignment.
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/members/{id}/assignments/{assignment_id}",
    tag = "access",
    operation_id = "assignments_delete",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Member"),
        ("assignment_id" = Uuid, Path, description = "Assignment")
    ),
    responses((status = 204, description = "Removed"))
)]
pub async fn assignments_delete(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, member, aid)): Path<(Uuid, Uuid, Uuid)>,
) -> ApiResult<Response> {
    let (mut tx, ws, caller) = begin_admin(&state, &ctx).await?;
    let before: AssignmentResponse = sqlx::query_as(sql!(
        "SELECT {ASSIGNMENT_COLS} FROM role_assignments \
         WHERE workspace_id = $1 AND id = $2 AND member_id = $3 AND deleted_at IS NULL"
    ))
    .bind(ws)
    .bind(aid)
    .bind(member)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(not_found)?;
    let user = ctx.identity.user_id;
    sqlx::query("UPDATE role_assignments SET deleted_at = now(), updated_by = $3 WHERE workspace_id = $1 AND id = $2")
        .bind(ws)
        .bind(aid)
        .bind(user)
        .execute(&mut *tx)
        .await?;
    bump_authz(&mut tx, ws, user).await?;
    guard(&mut tx, ws, caller).await?;
    audit(
        &mut tx,
        &ctx,
        ws,
        AuditEvent::RoleUnassign,
        "role_assignment",
        aid,
        Some(json!({
            "member_id": before.member_id, "role_id": before.role_id,
            "scope_kind": before.scope_kind, "scope_id": before.scope_id
        })),
        None,
    )
    .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

// --- invitations ----------------------------------------------------------------------------

/// A pending invitation. The token is only in the mail; the address is not stored.
#[derive(Debug, Serialize, ToSchema, sqlx::FromRow)]
pub struct InvitationResponse {
    /// Invitation id.
    pub id: Uuid,
    /// The role it grants.
    pub role_id: Uuid,
    /// Scope kind, if scoped.
    pub scope_kind: Option<String>,
    /// Scoped record.
    pub scope_id: Option<Uuid>,
    /// Works until.
    pub expires_at: DateTime<Utc>,
    /// Sent at.
    pub created_at: DateTime<Utc>,
}

/// Response of creating an invitation.
#[derive(Debug, Serialize, ToSchema)]
pub struct InvitationCreated {
    /// The invitation.
    #[serde(flatten)]
    pub invitation: InvitationResponse,
    /// Whether the mail was handed to the mail server.
    pub email_sent: bool,
}

/// List of pending invitations.
#[derive(Debug, Serialize, ToSchema)]
pub struct InvitationList {
    /// Pending invitations.
    pub items: Vec<InvitationResponse>,
}

/// Body of `POST /invitations`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NewInvitation {
    /// Where the link is mailed. Not stored; the link is not bound to it.
    pub email: String,
    /// The role to grant.
    pub role_id: Uuid,
    /// `recipe`, `batch`, `vessel` or `area`; omit for the whole workspace.
    pub scope_kind: Option<String>,
    /// The scoped record.
    pub scope_id: Option<Uuid>,
}

const INVITATION_COLS: &str = "id, role_id, scope_kind, scope_id, expires_at, created_at";

fn invitation_mail(to: &str, link: &str, workspace: &str) -> Mail {
    Mail {
        to: to.to_owned(),
        subject: "You have been invited to a brewery".to_owned(),
        text: format!(
            "You have been invited to join \"{workspace}\".\n\n\
             Open this link to accept. It works once and expires in {INVITATION_TTL_DAYS} days.\n\n\
             {link}\n\n\
             If you were not expecting this invitation, ignore this message.\n"
        ),
    }
}

/// Creates the invitation row and mails the link. Shared by invitations and Home shares.
async fn create_invitation(
    state: &AppState,
    svc: &AuthServices,
    ctx: &RequestContext,
    email: &str,
    role_id: Uuid,
    scope: Option<(ScopeKind, Uuid)>,
    must_be_mode: Option<&str>,
) -> ApiResult<Response> {
    let to = normalize_email(email).ok_or_else(|| invalid("/email", "invalid"))?;
    let (mut tx, ws, _) = begin_admin(state, ctx).await?;
    let (mode, workspace_name): (String, String) =
        sqlx::query_as("SELECT mode, name FROM workspaces WHERE id = $1")
            .bind(ws)
            .fetch_one(&mut *tx)
            .await?;
    if let Some(m) = must_be_mode
        && mode != m
    {
        return Err(conflict("/", "home_mode_only"));
    }
    live_role(&mut tx, ws, role_id).await?;
    if scope.is_some() {
        ensure_scopable(&mut tx, ws, role_id).await?;
    }
    let token = new_token();
    let id = Uuid::now_v7();
    let user = ctx.identity.user_id;
    let res = sqlx::query(
        "INSERT INTO invitations (id, workspace_id, token_hash, role_id, scope_kind, scope_id, \
                                  expires_at, created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)",
    )
    .bind(id)
    .bind(ws)
    .bind(token_hash(&token))
    .bind(role_id)
    .bind(scope.map(|s| s.0.as_str()))
    .bind(scope.map(|s| s.1))
    .bind(Utc::now() + Duration::days(INVITATION_TTL_DAYS))
    .bind(user)
    .execute(&mut *tx)
    .await;
    if let Err(e) = res {
        return Err(if pg_code(&e).as_deref() == Some("23503") {
            invalid("/scope_id", "scope_target_not_found")
        } else {
            e.into()
        });
    }
    audit(
        &mut tx,
        ctx,
        ws,
        AuditEvent::InvitationCreate,
        "invitation",
        id,
        None,
        Some(json!({
            "role_id": role_id, "scope_kind": scope.map(|s| s.0.as_str()),
            "scope_id": scope.map(|s| s.1), "email": changed_field()
        })),
    )
    .await?;
    let invitation: InvitationResponse = sqlx::query_as(sql!(
        "SELECT {INVITATION_COLS} FROM invitations WHERE workspace_id = $1 AND id = $2"
    ))
    .bind(ws)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    let link = format!("{}/invite?token={token}", svc.config.web_base_url);
    let email_sent = match svc
        .mailer
        .send(&invitation_mail(&to, &link, &workspace_name))
        .await
    {
        Ok(()) => true,
        Err(e) => {
            tracing::error!(error = %e, "invitation mail not sent");
            false
        }
    };
    Ok((
        StatusCode::CREATED,
        Json(InvitationCreated {
            invitation,
            email_sent,
        }),
    )
        .into_response())
}

/// Invites someone by mail. The link works once, for 7 days.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/invitations",
    tag = "access",
    operation_id = "invitations_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = NewInvitation,
    responses((
        status = 201,
        description = "Invitation created and mailed",
        body = InvitationCreated,
        example = json!({"id": "0192f1c07a3b70008000000000000014",
            "role_id": "0192f1c07a3b7000800000000000000e", "scope_kind": null, "scope_id": null,
            "expires_at": "2026-11-21T10:00:00Z", "created_at": "2026-11-14T10:00:00Z",
            "email_sent": true})
    ))
)]
pub async fn invitations_create(
    State(state): State<AppState>,
    Extension(svc): Svc,
    ctx: RequestContext,
    ApiJson(req): ApiJson<NewInvitation>,
) -> ApiResult<Response> {
    let scope = parse_scope(req.scope_kind.as_deref(), req.scope_id)?;
    create_invitation(&state, &svc, &ctx, &req.email, req.role_id, scope, None).await
}

/// Lists pending invitations.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/invitations",
    tag = "access",
    operation_id = "invitations_list",
    params(("workspace_id" = Uuid, Path, description = "Workspace")),
    responses((
        status = 200,
        description = "Pending invitations",
        body = InvitationList,
        example = json!({"items": []})
    ))
)]
pub async fn invitations_list(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> ApiResult<Json<InvitationList>> {
    ctx.require_permission(Permission::UsersManage)?;
    let ws = ws_of(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    let items = sqlx::query_as(sql!(
        "SELECT {INVITATION_COLS} FROM invitations \
         WHERE workspace_id = $1 AND accepted_at IS NULL AND revoked_at IS NULL \
           AND deleted_at IS NULL AND expires_at > now() ORDER BY created_at, id"
    ))
    .bind(ws)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(InvitationList { items }))
}

/// Revokes a pending invitation.
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/invitations/{id}",
    tag = "access",
    operation_id = "invitations_revoke",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Invitation")
    ),
    responses((status = 204, description = "Revoked"))
)]
pub async fn invitations_revoke(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Response> {
    let (mut tx, ws, _) = begin_admin(&state, &ctx).await?;
    let n = sqlx::query(
        "UPDATE invitations SET revoked_at = now(), updated_by = $3 \
         WHERE workspace_id = $1 AND id = $2 AND accepted_at IS NULL AND revoked_at IS NULL \
           AND deleted_at IS NULL",
    )
    .bind(ws)
    .bind(id)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if n == 0 {
        return Err(not_found());
    }
    audit(
        &mut tx,
        &ctx,
        ws,
        AuditEvent::InvitationRevoke,
        "invitation",
        id,
        None,
        None,
    )
    .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Body of `POST /invitations/accept`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AcceptInvitation {
    /// The token from the mailed link.
    pub token: String,
}

/// Result of accepting.
#[derive(Debug, Serialize, ToSchema)]
pub struct InvitationAccepted {
    /// The workspace joined.
    pub workspace_id: Uuid,
    /// The caller's member id there.
    pub member_id: Uuid,
    /// The role granted.
    pub role_id: Uuid,
    /// Scope kind, if scoped.
    pub scope_kind: Option<String>,
    /// Scoped record.
    pub scope_id: Option<Uuid>,
}

#[derive(sqlx::FromRow)]
struct FoundInvitation {
    id: Uuid,
    workspace_id: Uuid,
    role_id: Uuid,
    scope_kind: Option<String>,
    scope_id: Option<Uuid>,
}

/// Accepts an invitation as the signed-in user: joins the workspace with the invited role.
/// An unknown, used, revoked or expired token is `404`.
#[utoipa::path(
    post,
    path = "/v1/invitations/accept",
    tag = "access",
    operation_id = "invitations_accept",
    request_body = AcceptInvitation,
    responses((
        status = 200,
        description = "Joined",
        body = InvitationAccepted,
        example = json!({"workspace_id": "0192f1c07a3b7000800000000000000c",
            "member_id": "0192f1c07a3b70008000000000000010",
            "role_id": "0192f1c07a3b7000800000000000000e", "scope_kind": null, "scope_id": null})
    ))
)]
pub async fn invitations_accept(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(req): ApiJson<AcceptInvitation>,
) -> ApiResult<Json<InvitationAccepted>> {
    if ctx.identity.actor_kind != db::ActorKind::User || req.token.len() > 256 {
        return Err(if req.token.len() > 256 {
            not_found()
        } else {
            ApiError::new(ErrorCode::Forbidden)
        });
    }
    let found: FoundInvitation = sqlx::query_as(
        "SELECT id, workspace_id, role_id, scope_kind, scope_id FROM find_invitation_by_token_hash($1)",
    )
    .bind(token_hash(&req.token))
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(not_found)?;
    let user = ctx.identity.user_id;
    let ws = found.workspace_id;
    let mut tx = db::begin(&state.db, &db::DbContext::user(user, Some(ws))).await?;
    // The update is the single-use gate: only one concurrent request gets the row.
    let taken = sqlx::query(
        "UPDATE invitations SET accepted_by = $3, accepted_at = now(), updated_by = $3 \
         WHERE workspace_id = $1 AND id = $2 AND accepted_at IS NULL AND revoked_at IS NULL \
           AND deleted_at IS NULL AND expires_at > now()",
    )
    .bind(ws)
    .bind(found.id)
    .bind(user)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    let live: i64 = sqlx::query_scalar(
        "SELECT (SELECT count(*) FROM roles WHERE workspace_id = $1 AND id = $2 AND deleted_at IS NULL) \
              + (SELECT count(*) FROM workspaces WHERE id = $1 AND deleted_at IS NULL)",
    )
    .bind(ws)
    .bind(found.role_id)
    .fetch_one(&mut *tx)
    .await?;
    if taken == 0 || live != 2 {
        return Err(not_found());
    }
    let existing: Option<(Uuid, String, Option<DateTime<Utc>>)> = sqlx::query_as(
        "SELECT id, status, deleted_at FROM workspace_members \
         WHERE workspace_id = $1 AND user_id = $2 FOR UPDATE",
    )
    .bind(ws)
    .bind(user)
    .fetch_optional(&mut *tx)
    .await?;
    let (member_id, joined) = match existing {
        Some((id, _, Some(_))) => {
            sqlx::query(
                "UPDATE workspace_members SET deleted_at = NULL, status = 'active', \
                        joined_at = now(), updated_by = $3 WHERE workspace_id = $1 AND id = $2",
            )
            .bind(ws)
            .bind(id)
            .bind(user)
            .execute(&mut *tx)
            .await?;
            (id, true)
        }
        Some((_, status, None)) if status == "disabled" => {
            return Err(conflict("/", "member_disabled"));
        }
        Some((id, _, None)) => (id, false),
        None => {
            let id = Uuid::now_v7();
            sqlx::query(
                "INSERT INTO workspace_members (id, workspace_id, user_id, created_by, updated_by) \
                 VALUES ($1, $2, $3, $3, $3)",
            )
            .bind(id)
            .bind(ws)
            .bind(user)
            .execute(&mut *tx)
            .await?;
            (id, true)
        }
    };
    let assignment = Uuid::now_v7();
    let res = sqlx::query(
        "INSERT INTO role_assignments (id, workspace_id, member_id, role_id, scope_kind, scope_id, \
                                       created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $7) ON CONFLICT DO NOTHING",
    )
    .bind(assignment)
    .bind(ws)
    .bind(member_id)
    .bind(found.role_id)
    .bind(&found.scope_kind)
    .bind(found.scope_id)
    .bind(user)
    .execute(&mut *tx)
    .await;
    if let Err(e) = res {
        return Err(if pg_code(&e).as_deref() == Some("23503") {
            not_found()
        } else {
            e.into()
        });
    }
    bump_authz(&mut tx, ws, user).await?;
    if joined {
        audit(
            &mut tx,
            &ctx,
            ws,
            AuditEvent::MemberStatusChange,
            "workspace_member",
            member_id,
            None,
            Some(json!({"status": "active", "via": "invitation", "invitation_id": found.id})),
        )
        .await?;
    }
    audit(
        &mut tx,
        &ctx,
        ws,
        AuditEvent::RoleAssign,
        "role_assignment",
        assignment,
        None,
        Some(json!({
            "member_id": member_id, "role_id": found.role_id,
            "scope_kind": found.scope_kind, "scope_id": found.scope_id,
            "invitation_id": found.id
        })),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(InvitationAccepted {
        workspace_id: ws,
        member_id,
        role_id: found.role_id,
        scope_kind: found.scope_kind,
        scope_id: found.scope_id,
    }))
}

// --- Home sharing ---------------------------------------------------------------------------

/// Body of `POST /shares`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NewShare {
    /// `recipe` or `batch`.
    pub kind: String,
    /// The recipe or batch.
    pub id: Uuid,
    /// `editor` or `viewer`.
    pub access: String,
    /// Where the link is mailed.
    pub email: String,
}

/// Shares one recipe or batch with a friend (Home workspaces): a scoped invitation with the
/// editor or viewer preset.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/shares",
    tag = "access",
    operation_id = "shares_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = NewShare,
    responses((
        status = 201,
        description = "Share invitation created and mailed",
        body = InvitationCreated,
        example = json!({"id": "0192f1c07a3b70008000000000000014",
            "role_id": "0192f1c07a3b7000800000000000000e", "scope_kind": "recipe",
            "scope_id": "0192f1c07a3b70008000000000000013",
            "expires_at": "2026-11-21T10:00:00Z", "created_at": "2026-11-14T10:00:00Z",
            "email_sent": true})
    ))
)]
pub async fn shares_create(
    State(state): State<AppState>,
    Extension(svc): Svc,
    ctx: RequestContext,
    ApiJson(req): ApiJson<NewShare>,
) -> ApiResult<Response> {
    let kind = match req.kind.as_str() {
        "recipe" => ScopeKind::Recipe,
        "batch" => ScopeKind::Batch,
        _ => return Err(invalid("/kind", "invalid_choice")),
    };
    let key = match req.access.as_str() {
        "editor" => "share_editor",
        "viewer" => "share_viewer",
        _ => return Err(invalid("/access", "invalid_choice")),
    };
    ctx.require_permission(Permission::UsersManage)?;
    let ws = ws_of(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    let role: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM roles WHERE workspace_id = $1 AND template_key = $2 AND deleted_at IS NULL \
         ORDER BY created_at LIMIT 1",
    )
    .bind(ws)
    .bind(key)
    .fetch_optional(&mut *tx)
    .await?;
    tx.commit().await?;
    let role = role.ok_or_else(|| conflict("/", "home_mode_only"))?;
    create_invitation(
        &state,
        &svc,
        &ctx,
        &req.email,
        role,
        Some((kind, req.id)),
        Some("home"),
    )
    .await
}

// --- registry entry ----------------------------------------------------------------------

/// Operation specs of this module (rows for `crate::openapi`).
mod spec {
    use crate::concurrency::Concurrency;
    use crate::error::ErrorCode;
    use crate::error::ErrorCode::{
        Conflict, Forbidden, IdempotencyInProgress, IdempotencyMismatch, Internal,
        MalformedRequest, NotFound, PreconditionRequired, RateLimited, Unauthenticated,
        UnsupportedMediaType, UpgradeRequired, ValidationFailed, VersionMismatch,
    };
    use crate::openapi::OperationSpec;

    const ACCESS_READ: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        RateLimited,
        UpgradeRequired,
        Internal,
    ];

    const ACCESS_CREATE: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        Conflict,
        IdempotencyInProgress,
        UnsupportedMediaType,
        ValidationFailed,
        IdempotencyMismatch,
        PreconditionRequired,
        RateLimited,
        UpgradeRequired,
        Internal,
    ];

    const ACCESS_UPDATE: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        Conflict,
        VersionMismatch,
        UnsupportedMediaType,
        ValidationFailed,
        PreconditionRequired,
        RateLimited,
        UpgradeRequired,
        Internal,
    ];

    const ACCESS_DELETE: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        Conflict,
        RateLimited,
        UpgradeRequired,
        Internal,
    ];

    const ACCESS_ACCEPT: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        Conflict,
        UnsupportedMediaType,
        RateLimited,
        UpgradeRequired,
        Internal,
    ];

    pub(super) const OPERATIONS: &[OperationSpec] = &[
        OperationSpec {
            operation_id: "roles_list",
            public: false,
            permission: Some("users.manage"),
            concurrency: None,
            errors: ACCESS_READ,
        },
        OperationSpec {
            operation_id: "roles_create",
            public: false,
            permission: Some("users.manage"),
            concurrency: Some(Concurrency::Append),
            errors: ACCESS_CREATE,
        },
        OperationSpec {
            operation_id: "roles_copy",
            public: false,
            permission: Some("users.manage"),
            concurrency: Some(Concurrency::Append),
            errors: ACCESS_CREATE,
        },
        OperationSpec {
            operation_id: "roles_update",
            public: false,
            permission: Some("users.manage"),
            concurrency: Some(Concurrency::Optimistic),
            errors: ACCESS_UPDATE,
        },
        OperationSpec {
            operation_id: "roles_delete",
            public: false,
            permission: Some("users.manage"),
            concurrency: Some(Concurrency::Optimistic),
            errors: ACCESS_UPDATE,
        },
        OperationSpec {
            operation_id: "members_list",
            public: false,
            permission: Some("users.manage"),
            concurrency: None,
            errors: ACCESS_READ,
        },
        OperationSpec {
            operation_id: "members_update",
            public: false,
            permission: Some("users.manage"),
            concurrency: Some(Concurrency::FieldLww),
            errors: ACCESS_UPDATE,
        },
        OperationSpec {
            operation_id: "members_remove",
            public: false,
            permission: Some("users.manage"),
            concurrency: Some(Concurrency::FieldLww),
            errors: ACCESS_DELETE,
        },
        OperationSpec {
            operation_id: "assignments_create",
            public: false,
            permission: Some("users.manage"),
            concurrency: Some(Concurrency::Append),
            errors: ACCESS_CREATE,
        },
        OperationSpec {
            operation_id: "assignments_delete",
            public: false,
            permission: Some("users.manage"),
            concurrency: Some(Concurrency::FieldLww),
            errors: ACCESS_DELETE,
        },
        OperationSpec {
            operation_id: "invitations_create",
            public: false,
            permission: Some("users.manage"),
            concurrency: Some(Concurrency::Append),
            errors: ACCESS_CREATE,
        },
        OperationSpec {
            operation_id: "invitations_list",
            public: false,
            permission: Some("users.manage"),
            concurrency: None,
            errors: ACCESS_READ,
        },
        OperationSpec {
            operation_id: "invitations_revoke",
            public: false,
            permission: Some("users.manage"),
            concurrency: Some(Concurrency::FieldLww),
            errors: ACCESS_DELETE,
        },
        OperationSpec {
            operation_id: "invitations_accept",
            public: false,
            permission: None,
            concurrency: None,
            errors: ACCESS_ACCEPT,
        },
        OperationSpec {
            operation_id: "shares_create",
            public: false,
            permission: Some("users.manage"),
            concurrency: Some(Concurrency::Append),
            errors: ACCESS_CREATE,
        },
    ];
}

/// `OpenAPI` fragment of this module; `crate::openapi` merges it.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(
        roles_list,
        roles_create,
        roles_copy,
        roles_update,
        roles_delete,
        members_list,
        members_update,
        members_remove,
        assignments_create,
        assignments_delete,
        invitations_create,
        invitations_list,
        invitations_revoke,
        invitations_accept,
        shares_create
    ),
    components(schemas(
        RoleResponse,
        RoleList,
        NewRole,
        CopyRole,
        RolePatch,
        AssignmentResponse,
        MemberResponse,
        MemberList,
        MemberPatch,
        NewAssignment,
        InvitationResponse,
        InvitationCreated,
        InvitationList,
        NewInvitation,
        AcceptInvitation,
        InvitationAccepted,
        NewShare
    )),
    tags(
        (name = "access", description = "Roles, members, assignments, invitations and sharing")
    )
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "access",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
