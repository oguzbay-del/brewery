//! "Connected apps" endpoints and the workspace MCP switch (ADR-0015 rules 2.7, 3.6, 3.9).
//! A member manages only their own connections. An administrator cannot see or remove the
//! connections of members; the switch closes all MCP connections of the workspace at once.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use db::{AuditEvent, NewAuditEntry};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgConnection;
use utoipa::ToSchema;
use uuid::Uuid;

use super::consent::{audit_grant, not_found, session_required};
use crate::{
    AppState,
    context::RequestContext,
    error::{ApiError, ApiJson, ApiResult, ErrorCode, FieldIssue},
};

/// A client of the caller's account with its grant.
#[derive(Debug, Serialize, ToSchema)]
pub struct ConnectedApp {
    /// Grant id (use it to remove the connection).
    pub id: Uuid,
    /// The `client_id` text.
    pub client_id: String,
    /// Name of the client.
    pub client_name: String,
    /// Logo address.
    pub logo_uri: Option<String>,
    /// Workspace of the grant.
    pub workspace_id: Uuid,
    /// Name of that workspace.
    pub workspace_name: String,
    /// Granted scopes.
    pub scopes: Vec<String>,
    /// The resource: the MCP address or the open API address.
    pub resource: String,
    /// When the user gave consent.
    pub created_at: DateTime<Utc>,
    /// Last use of a token (written at most once a minute).
    pub last_used_at: Option<DateTime<Utc>>,
}

/// The list of connected apps.
#[derive(Debug, Serialize, ToSchema)]
pub struct ConnectedApps {
    /// Active grants of the caller.
    pub items: Vec<ConnectedApp>,
}

type GrantRow = (
    Uuid,
    String,
    String,
    Option<String>,
    Uuid,
    Vec<String>,
    String,
    DateTime<Utc>,
    Option<DateTime<Utc>>,
);

/// `GET /v1/me/connected-apps`.
#[utoipa::path(
    get,
    path = "/v1/me/connected-apps",
    tag = "oauth",
    operation_id = "connected_app_list",
    responses(
        (status = 200, description = "Active grants of the caller", body = ConnectedApps, example = json!({"items": [{
            "id": "0192f1c07a3b7000800000000000000a", "client_id": "https://agent.example/oauth/client.json",
            "client_name": "Example Agent", "logo_uri": null,
            "workspace_id": "0192f1c07a3b7000800000000000000b", "workspace_name": "Home",
            "scopes": ["recipes:read"], "resource": "https://api.gristora.com/mcp",
            "created_at": "2026-11-27T10:00:00Z", "last_used_at": null}]})),
    )
)]
pub async fn connected_app_list(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> ApiResult<Json<ConnectedApps>> {
    session_required(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    let rows: Vec<GrantRow> = sqlx::query_as(
        "SELECT g.id, c.client_id, c.client_name, c.logo_uri, g.workspace_id, g.scopes,
                g.resource, g.created_at, g.last_used_at
         FROM oauth_grants g JOIN oauth_clients c ON c.id = g.client_id
         WHERE g.revoked_at IS NULL ORDER BY g.created_at DESC",
    )
    .fetch_all(&mut *tx)
    .await?;
    let mut items = Vec::new();
    for (id, client_id, client_name, logo_uri, ws, scopes, resource, created_at, last_used_at) in
        rows
    {
        db::set_workspace_id(&mut tx, ws).await?;
        let name: Option<String> =
            sqlx::query_scalar("SELECT name FROM workspaces WHERE id = $1 AND deleted_at IS NULL")
                .bind(ws)
                .fetch_optional(&mut *tx)
                .await?;
        let Some(workspace_name) = name else { continue };
        items.push(ConnectedApp {
            id,
            client_id,
            client_name,
            logo_uri,
            workspace_id: ws,
            workspace_name,
            scopes,
            resource,
            created_at,
            last_used_at,
        });
    }
    tx.commit().await?;
    Ok(Json(ConnectedApps { items }))
}

async fn audit_revocations(
    tx: &mut PgConnection,
    ctx: &RequestContext,
    rows: &[(Uuid, Uuid, Uuid)],
    by: &str,
) -> ApiResult<()> {
    for (grant, ws, client_pk) in rows {
        let client_id: String =
            sqlx::query_scalar("SELECT client_id FROM oauth_clients WHERE id = $1")
                .bind(client_pk)
                .fetch_one(&mut *tx)
                .await?;
        audit_grant(
            tx,
            ctx,
            *ws,
            AuditEvent::OauthGrantRevoke,
            *grant,
            (*client_pk, &client_id),
            json!({"by": by}),
        )
        .await?;
    }
    Ok(())
}

/// `DELETE /v1/me/connected-apps/{grant_id}`.
#[utoipa::path(
    delete,
    path = "/v1/me/connected-apps/{grant_id}",
    tag = "oauth",
    operation_id = "connected_app_revoke",
    params(("grant_id" = Uuid, Path, description = "Grant id")),
    responses((status = 204, description = "The connection is removed; its tokens stop working"))
)]
pub async fn connected_app_revoke(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path(grant_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    session_required(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    let rows: Vec<(Uuid, Uuid, Uuid)> =
        sqlx::query_as("SELECT * FROM oauth_grant_revoke($1, $2, 'user_revoked')")
            .bind(grant_id)
            .bind(ctx.identity.user_id)
            .fetch_all(&mut *tx)
            .await?;
    if rows.is_empty() {
        return Err(not_found());
    }
    audit_revocations(&mut tx, &ctx, &rows, "user").await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /v1/me/connected-apps`: remove every connection of the caller.
#[utoipa::path(
    delete,
    path = "/v1/me/connected-apps",
    tag = "oauth",
    operation_id = "connected_app_clear",
    responses((status = 204, description = "Every connection is removed"))
)]
pub async fn connected_app_clear(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> ApiResult<StatusCode> {
    session_required(&ctx)?;
    revoke_all_for_user(&state, &ctx).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Closes every grant of the caller and writes the audit rows. Account deletion uses it too.
///
/// # Errors
/// Database errors.
pub async fn revoke_all_for_user(state: &AppState, ctx: &RequestContext) -> ApiResult<usize> {
    let mut tx = ctx.begin(&state.db).await?;
    let n = revoke_all_in_tx(&mut tx, ctx, "user_revoked_all").await?;
    tx.commit().await?;
    Ok(n)
}

/// Like [`revoke_all_for_user`], inside the transaction of the caller (account deletion).
///
/// # Errors
/// Database errors.
pub async fn revoke_all_in_tx(
    tx: &mut PgConnection,
    ctx: &RequestContext,
    reason: &str,
) -> ApiResult<usize> {
    let rows: Vec<(Uuid, Uuid, Uuid)> =
        sqlx::query_as("SELECT * FROM oauth_grant_revoke_all_user($1, $2)")
            .bind(ctx.identity.user_id)
            .bind(reason)
            .fetch_all(&mut *tx)
            .await?;
    audit_revocations(tx, ctx, &rows, "user_all").await?;
    Ok(rows.len())
}

// --- workspace switch --------------------------------------------------------------------

/// The MCP switch of a workspace.
#[derive(Debug, Serialize, ToSchema)]
pub struct McpSwitch {
    /// MCP is on for the workspace.
    pub enabled: bool,
    /// Grants closed by this change (only on a switch to off).
    pub revoked_grants: i64,
}

/// Body of `PUT /v1/workspaces/{ws}/mcp`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct McpSwitchBody {
    /// New state.
    pub enabled: bool,
}

const SWITCH_PERMISSION: &str = "workspace.manage";

async fn business_only(conn: &mut PgConnection, ws: Uuid) -> ApiResult<()> {
    let mode: String = sqlx::query_scalar("SELECT mode FROM workspaces WHERE id = $1")
        .bind(ws)
        .fetch_one(conn)
        .await?;
    if mode == "business" {
        Ok(())
    } else {
        Err(ApiError::new(ErrorCode::Forbidden)
            .with_errors(vec![FieldIssue::new("/", "business_only")]))
    }
}

/// `GET /v1/workspaces/{ws}/mcp`.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/mcp",
    tag = "oauth",
    operation_id = "workspace_mcp_get",
    params(("workspace_id" = Uuid, Path, description = "Workspace id")),
    responses((status = 200, description = "State of the MCP switch", body = McpSwitch,
               example = json!({"enabled": true, "revoked_grants": 0})))
)]
pub async fn workspace_mcp_get(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> ApiResult<Json<McpSwitch>> {
    ctx.require(SWITCH_PERMISSION)?;
    let ws = ctx.workspace_id.ok_or_else(not_found)?;
    let mut tx = ctx.begin(&state.db).await?;
    business_only(&mut tx, ws).await?;
    let enabled: bool = sqlx::query_scalar("SELECT mcp_enabled FROM workspaces WHERE id = $1")
        .bind(ws)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(McpSwitch {
        enabled,
        revoked_grants: 0,
    }))
}

/// `PUT /v1/workspaces/{ws}/mcp`: an administrator of a Business workspace turns MCP on or off.
/// Off closes every MCP grant of the workspace; on does not bring them back.
#[utoipa::path(
    put,
    path = "/v1/workspaces/{workspace_id}/mcp",
    tag = "oauth",
    operation_id = "workspace_mcp_set",
    params(("workspace_id" = Uuid, Path, description = "Workspace id")),
    request_body = McpSwitchBody,
    responses((status = 200, description = "New state of the switch", body = McpSwitch,
               example = json!({"enabled": false, "revoked_grants": 2})))
)]
pub async fn workspace_mcp_set(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<McpSwitchBody>,
) -> ApiResult<Json<McpSwitch>> {
    ctx.require(SWITCH_PERMISSION)?;
    let ws = ctx.workspace_id.ok_or_else(not_found)?;
    let mut tx = ctx.begin(&state.db).await?;
    business_only(&mut tx, ws).await?;
    let before: bool = sqlx::query_scalar("SELECT mcp_enabled FROM workspaces WHERE id = $1")
        .bind(ws)
        .fetch_one(&mut *tx)
        .await?;
    let mut revoked = 0_i64;
    if before != body.enabled {
        sqlx::query("UPDATE workspaces SET mcp_enabled = $2 WHERE id = $1")
            .bind(ws)
            .bind(body.enabled)
            .execute(&mut *tx)
            .await?;
        if !body.enabled {
            let rows: Vec<(Uuid, Uuid, Uuid)> = sqlx::query_as(
                "SELECT * FROM oauth_workspace_revoke_all($1, $2, 'workspace_switch')",
            )
            .bind(ws)
            .bind(&state.config.oauth.mcp_resource)
            .fetch_all(&mut *tx)
            .await?;
            revoked = i64::try_from(rows.len()).unwrap_or(i64::MAX);
        }
        db::append_audit(
            &mut tx,
            ws,
            &NewAuditEntry {
                actor_kind: ctx.identity.actor_kind,
                actor_id: Some(ctx.identity.user_id),
                event: AuditEvent::McpSwitch,
                record_kind: "workspace".to_owned(),
                record_id: Some(ws),
                previous_value: Some(json!({"enabled": before})),
                new_value: Some(json!({"enabled": body.enabled, "revoked_grants": revoked})),
                reason: None,
            },
        )
        .await?;
    }
    tx.commit().await?;
    Ok(Json(McpSwitch {
        enabled: body.enabled,
        revoked_grants: revoked,
    }))
}
