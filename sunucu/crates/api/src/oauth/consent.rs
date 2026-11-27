//! What the consent screen calls, the "Connected apps" endpoints and the workspace switch
//! (ADR-0015 rules 2.7, 3.6, 3.9 and 8). The screen is a client task (M4); this is its API.
//!
//! Consent needs a real sign-in session: an OAuth token cannot consent, list or revoke grants
//! (the allow list in `scopes` has none of these operations).

use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use db::{AuditEvent, NewAuditEntry};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgConnection;
use utoipa::ToSchema;
use uuid::Uuid;

use super::{CODE_TTL_SECS, client, scopes, server::redirect_url};
use crate::{
    AppState,
    auth::tokens::{new_token, token_hash},
    context::{RequestContext, workspace_mode_and_paid},
    error::{ApiError, ApiJson, ApiResult, ErrorCode, FieldIssue},
};

pub(super) fn session_required(ctx: &RequestContext) -> ApiResult<()> {
    // A shared-device session belongs to a terminal, not to a person at a browser.
    if ctx.identity.session_id.is_some()
        && ctx.identity.oauth.is_none()
        && ctx.identity.actor_kind != db::ActorKind::Device
    {
        Ok(())
    } else {
        Err(ApiError::new(ErrorCode::Forbidden))
    }
}

pub(super) fn not_found() -> ApiError {
    ApiError::new(ErrorCode::NotFound)
}

fn invalid(field: &str, code: &str) -> ApiError {
    ApiError::validation(vec![FieldIssue::new(field, code)])
}

/// The client as the consent screen shows it. Every client is "unverified" in the first
/// release (ADR-0015 rule 8.2).
#[derive(Debug, Serialize, ToSchema)]
pub struct ConsentClient {
    /// The `client_id` text.
    pub client_id: String,
    /// Name from the client document.
    pub client_name: String,
    /// Logo address.
    pub logo_uri: Option<String>,
    /// Client home page.
    pub client_uri: Option<String>,
    /// Host of the `client_id` address (empty for a registered client).
    pub client_id_host: String,
    /// Host of the redirect address.
    pub redirect_host: String,
    /// The redirect address is on this computer (extra warning).
    pub localhost_redirect: bool,
    /// Always `false` in the first release.
    pub verified: bool,
}

/// A requested scope.
#[derive(Debug, Serialize, ToSchema)]
pub struct ConsentScope {
    /// The scope.
    pub scope: String,
    /// The scope writes records.
    pub write: bool,
}

/// A workspace the user can pick.
#[derive(Debug, Serialize, ToSchema)]
pub struct ConsentWorkspace {
    /// Workspace id.
    pub id: Uuid,
    /// Name.
    pub name: String,
    /// `home` or `business`.
    pub mode: String,
    /// The workspace can take this grant.
    pub eligible: bool,
    /// Why not: `business_only`, `plan_required` or `mcp_disabled`.
    pub ineligible_reason: Option<String>,
    /// Requested scopes that the role of the user allows.
    pub allowed_scopes: Vec<String>,
    /// Scopes the client already has here (the screen shows only the new ones).
    pub granted_scopes: Vec<String>,
}

/// An authorization request, for the consent screen.
#[derive(Debug, Serialize, ToSchema)]
pub struct ConsentRequestView {
    /// The client.
    pub client: ConsentClient,
    /// The resource (`api` or `mcp` address).
    pub resource: String,
    /// Requested scopes.
    pub requested_scopes: Vec<ConsentScope>,
    /// Workspaces of the user.
    pub workspaces: Vec<ConsentWorkspace>,
}

type RequestRow = (
    Uuid,
    String,
    Vec<String>,
    String,
    String,
    String,
    DateTime<Utc>,
);

struct StoredRequest {
    client_pk: Uuid,
    redirect_uri: String,
    scopes: Vec<String>,
    state: String,
    code_challenge: String,
    resource: String,
}

async fn load_request(conn: &mut PgConnection, request: &str) -> ApiResult<StoredRequest> {
    let row: Option<RequestRow> = sqlx::query_as("SELECT * FROM oauth_request_get($1)")
        .bind(token_hash(request))
        .fetch_optional(conn)
        .await?;
    let (client_pk, redirect_uri, scopes, state, code_challenge, resource, _exp) =
        row.ok_or_else(not_found)?;
    Ok(StoredRequest {
        client_pk,
        redirect_uri,
        scopes,
        state,
        code_challenge,
        resource,
    })
}

async fn workspace_options(
    state: &AppState,
    ctx: &RequestContext,
    stored: &StoredRequest,
) -> ApiResult<Vec<ConsentWorkspace>> {
    let mut tx = ctx.begin(&state.db).await?;
    let ids: Vec<Uuid> = sqlx::query_scalar(
        "SELECT workspace_id FROM workspace_members
         WHERE user_id = $1 AND status = 'active' AND deleted_at IS NULL
         ORDER BY workspace_id",
    )
    .bind(ctx.identity.user_id)
    .fetch_all(&mut *tx)
    .await?;
    let mut rows = Vec::new();
    for ws in ids {
        db::set_workspace_id(&mut tx, ws).await?;
        let info: Option<(String, String, bool)> = sqlx::query_as(
            "SELECT name, mode, mcp_enabled FROM workspaces WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(ws)
        .fetch_optional(&mut *tx)
        .await?;
        let granted: Option<Vec<String>> = sqlx::query_scalar(
            "SELECT scopes FROM oauth_grants
             WHERE workspace_id = $1 AND client_id = $2 AND resource = $3 AND revoked_at IS NULL",
        )
        .bind(ws)
        .bind(stored.client_pk)
        .bind(&stored.resource)
        .fetch_optional(&mut *tx)
        .await?;
        if let Some((name, mode, mcp_on)) = info {
            rows.push((ws, name, mode, mcp_on, granted.unwrap_or_default()));
        }
    }
    tx.commit().await?;
    let mut out = Vec::new();
    for (id, name, mode, mcp_on, granted_scopes) in rows {
        let Some(membership) = state.auth.membership(&ctx.identity, id).await? else {
            continue;
        };
        let is_api = stored.resource == state.config.oauth.api_resource;
        let reason = if is_api {
            (mode != "business").then_some("business_only")
        } else {
            let db_ctx = ctx.db_context();
            let (_, paid, _) = workspace_mode_and_paid(
                &state.db,
                &db::DbContext {
                    workspace_id: Some(id),
                    ..db_ctx
                },
                id,
            )
            .await?;
            if mode == "home" && !paid {
                Some("plan_required")
            } else if !mcp_on {
                Some("mcp_disabled")
            } else {
                None
            }
        };
        let usable = scopes::usable_by(&membership.permissions);
        out.push(ConsentWorkspace {
            id,
            name,
            mode,
            eligible: reason.is_none(),
            ineligible_reason: reason.map(String::from),
            allowed_scopes: stored
                .scopes
                .iter()
                .filter(|s| usable.contains(&s.as_str()))
                .cloned()
                .collect(),
            granted_scopes,
        });
    }
    Ok(out)
}

/// `GET /v1/me/oauth/requests/{request}`.
#[utoipa::path(
    get,
    path = "/v1/me/oauth/requests/{request}",
    tag = "oauth",
    operation_id = "oauth_request_get",
    params(("request" = String, Path, description = "Short-lived request id from the consent address")),
    responses(
        (status = 200, description = "The request for the consent screen", body = ConsentRequestView, example = json!({
            "client": {"client_id": "https://agent.example/oauth/client.json", "client_name": "Example Agent",
                       "logo_uri": null, "client_uri": null, "client_id_host": "agent.example",
                       "redirect_host": "agent.example", "localhost_redirect": false, "verified": false},
            "resource": "https://api.gristora.com/mcp",
            "requested_scopes": [{"scope": "recipes:read", "write": false}],
            "workspaces": [{"id": "0192f1c07a3b7000800000000000000a", "name": "Home", "mode": "home",
                            "eligible": true, "ineligible_reason": null,
                            "allowed_scopes": ["recipes:read"], "granted_scopes": []}]
        })),
    )
)]
pub async fn request_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path(request): Path<String>,
) -> ApiResult<Json<ConsentRequestView>> {
    session_required(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    let stored = load_request(&mut tx, &request).await?;
    tx.commit().await?;
    let client = client::by_pk(&state.db, stored.client_pk)
        .await?
        .ok_or_else(not_found)?;
    let redirect_host = client::host_of(&stored.redirect_uri);
    let view = ConsentClient {
        client_id_host: if client.kind == "cimd" {
            client::host_of(&client.client_id)
        } else {
            String::new()
        },
        localhost_redirect: matches!(redirect_host.as_str(), "localhost" | "127.0.0.1" | "[::1]"),
        redirect_host,
        client_id: client.client_id,
        client_name: client.client_name,
        logo_uri: client.logo_uri,
        client_uri: client.client_uri,
        verified: false,
    };
    let workspaces = workspace_options(&state, &ctx, &stored).await?;
    Ok(Json(ConsentRequestView {
        client: view,
        resource: stored.resource,
        requested_scopes: stored
            .scopes
            .iter()
            .map(|s| ConsentScope {
                scope: s.clone(),
                write: scopes::info(s).is_some_and(|i| i.write),
            })
            .collect(),
        workspaces,
    }))
}

/// Body of the decision.
#[derive(Debug, Deserialize, ToSchema)]
pub struct Decision {
    /// `approve` or `deny`.
    pub decision: String,
    /// The one workspace (required for `approve`).
    pub workspace_id: Option<Uuid>,
    /// Scopes to give; they may be fewer than requested. All requested scopes when absent.
    pub scopes: Option<Vec<String>>,
}

/// Where the screen sends the browser next.
#[derive(Debug, Serialize, ToSchema)]
pub struct DecisionResult {
    /// The redirect address of the client with `code` (or `error`), `state` and `iss`.
    pub redirect_to: String,
}

pub(super) async fn audit_grant(
    conn: &mut PgConnection,
    ctx: &RequestContext,
    ws: Uuid,
    event: AuditEvent,
    grant_id: Uuid,
    (client_pk, client_id): (Uuid, &str),
    detail: serde_json::Value,
) -> ApiResult<()> {
    db::set_workspace_id(conn, ws).await?;
    db::set_audit_channel(conn, "app", Some(client_pk), Some(grant_id), None).await?;
    db::append_audit(
        conn,
        ws,
        &NewAuditEntry {
            actor_kind: ctx.identity.actor_kind,
            actor_id: Some(ctx.identity.user_id),
            event,
            record_kind: "oauth_grant".to_owned(),
            record_id: Some(grant_id),
            previous_value: None,
            new_value: Some(json!({"client_id": client_id, "detail": detail})),
            reason: None,
        },
    )
    .await?;
    db::set_audit_channel(conn, "app", None, None, None).await?;
    Ok(())
}

/// `POST /v1/me/oauth/requests/{request}/decision`.
#[utoipa::path(
    post,
    path = "/v1/me/oauth/requests/{request}/decision",
    tag = "oauth",
    operation_id = "oauth_request_decide",
    params(("request" = String, Path, description = "Request id from the consent address")),
    request_body = Decision,
    responses(
        (status = 200, description = "Redirect address for the browser", body = DecisionResult,
         example = json!({"redirect_to": "https://agent.example/cb?code=abc&state=xyz&iss=https%3A%2F%2Fauth.gristora.com"})),
    )
)]
#[allow(clippy::too_many_lines)]
pub async fn request_decide(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path(request): Path<String>,
    ApiJson(body): ApiJson<Decision>,
) -> ApiResult<Json<DecisionResult>> {
    session_required(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    let stored = load_request(&mut tx, &request).await?;
    let client = client::by_pk(&state.db, stored.client_pk)
        .await?
        .ok_or_else(not_found)?;
    let req_hash = token_hash(&request);
    if body.decision == "deny" {
        sqlx::query("SELECT * FROM oauth_request_consume($1)")
            .bind(&req_hash)
            .fetch_optional(&mut *tx)
            .await?;
        tx.commit().await?;
        let to = redirect_url(
            &state,
            &stored.redirect_uri,
            &[("error", "access_denied"), ("state", &stored.state)],
        )
        .map_err(|_| ApiError::new(ErrorCode::Internal))?;
        return Ok(Json(DecisionResult { redirect_to: to }));
    }
    if body.decision != "approve" {
        return Err(invalid("/decision", "invalid_value"));
    }
    tx.commit().await?;
    let ws = body
        .workspace_id
        .ok_or_else(|| invalid("/workspace_id", "required"))?;
    let options = workspace_options(&state, &ctx, &stored).await?;
    let option = options
        .iter()
        .find(|o| o.id == ws)
        .ok_or_else(|| invalid("/workspace_id", "not_a_member"))?;
    if let Some(reason) = &option.ineligible_reason {
        return Err(match reason.as_str() {
            "plan_required" | "business_only" => ApiError::new(ErrorCode::EntitlementRequired),
            _ => ApiError::new(ErrorCode::Forbidden)
                .with_errors(vec![FieldIssue::new("/workspace_id", reason.clone())]),
        });
    }
    let chosen: Vec<String> = body.scopes.unwrap_or_else(|| stored.scopes.clone());
    if chosen.is_empty() {
        return Err(invalid("/scopes", "too_few_items"));
    }
    for s in &chosen {
        if !stored.scopes.contains(s) {
            return Err(invalid("/scopes", "not_requested"));
        }
        if !option.allowed_scopes.contains(s) {
            return Err(invalid("/scopes", "not_permitted_by_role"));
        }
    }
    let code = new_token();
    let mut tx = ctx.begin(&state.db).await?;
    let consumed: Option<(Uuid,)> =
        sqlx::query_as("SELECT client_pk FROM oauth_request_consume($1)")
            .bind(&req_hash)
            .fetch_optional(&mut *tx)
            .await?;
    if consumed.is_none() {
        return Err(not_found());
    }
    let (grant_id, granted, widened): (Uuid, Vec<String>, bool) =
        sqlx::query_as("SELECT * FROM oauth_grant_upsert($1, $2, $3, $4, $5, $6)")
            .bind(Uuid::now_v7())
            .bind(ctx.identity.user_id)
            .bind(ws)
            .bind(stored.client_pk)
            .bind(&chosen)
            .bind(&stored.resource)
            .fetch_one(&mut *tx)
            .await?;
    // The code carries the scopes of this consent; a refresh later cannot widen them.
    sqlx::query("SELECT oauth_code_create($1, $2, $3, $4, $5, $6, $7, $8, $9)")
        .bind(Uuid::now_v7())
        .bind(token_hash(&code))
        .bind(grant_id)
        .bind(stored.client_pk)
        .bind(&stored.redirect_uri)
        .bind(&stored.code_challenge)
        .bind(&granted)
        .bind(&stored.resource)
        .bind(CODE_TTL_SECS)
        .execute(&mut *tx)
        .await?;
    audit_grant(
        &mut tx,
        &ctx,
        ws,
        AuditEvent::OauthGrant,
        grant_id,
        (stored.client_pk, &client.client_id),
        json!({"scopes": granted, "widened": widened, "resource": stored.resource}),
    )
    .await?;
    tx.commit().await?;
    let to = redirect_url(
        &state,
        &stored.redirect_uri,
        &[("code", &code), ("state", &stored.state)],
    )
    .map_err(|_| ApiError::new(ErrorCode::Internal))?;
    Ok(Json(DecisionResult { redirect_to: to }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Identity;
    use std::collections::BTreeSet;

    fn ctx(actor_kind: db::ActorKind, session: bool) -> RequestContext {
        RequestContext {
            identity: Identity {
                user_id: Uuid::nil(),
                session_id: session.then(Uuid::nil),
                actor_kind,
                oauth: None,
            },
            workspace_id: None,
            permissions: BTreeSet::new(),
            scoped: Vec::new(),
        }
    }

    #[test]
    fn only_a_person_session_can_consent() {
        assert!(session_required(&ctx(db::ActorKind::User, true)).is_ok());
        assert!(session_required(&ctx(db::ActorKind::Device, true)).is_err());
        assert!(session_required(&ctx(db::ActorKind::User, false)).is_err());
    }
}
