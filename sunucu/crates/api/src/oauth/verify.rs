//! Bearer verification of OAuth access tokens and the per-request binding checks
//! (ADR-0015 rules 2.4, 3.1 to 3.5, 3.8, 3.9 and 7).
//!
//! A token binds to one user, workspace, client and resource. The lookup reads the token
//! record on every request (no cache), so a revoked token fails at once. A token never counts
//! as a session: it has no session id, so no second step can pass with it.

use std::collections::BTreeSet;

use axum::http::Method;
use chrono::{DateTime, Utc};
use db::{ActorKind, DbContext};
use sqlx::PgPool;
use uuid::Uuid;

use super::{ACCESS_PREFIX, scopes};
use crate::{
    AppState,
    auth::tokens::token_hash,
    context::{Identity, workspace_mode_and_paid},
    error::{ApiError, ApiResult, ErrorCode, FieldIssue},
};

/// What an access token is bound to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuthBinding {
    /// The grant (consent).
    pub grant_id: Uuid,
    /// Row id of the client.
    pub client_pk: Uuid,
    /// The `client_id` text of the client.
    pub client_id: String,
    /// The one workspace of the token.
    pub workspace_id: Uuid,
    /// Granted scopes.
    pub scopes: BTreeSet<String>,
    /// The resource (audience) of the token.
    pub resource: String,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct Lookup {
    token_id: Uuid,
    kind: String,
    grant_id: Uuid,
    user_id: Uuid,
    workspace_id: Uuid,
    client_pk: Uuid,
    client_id: String,
    client_name: String,
    scopes: Vec<String>,
    resource: String,
    expires_at: DateTime<Utc>,
    usable: bool,
}

/// Whether a bearer token has the shape of an OAuth access token.
#[must_use]
pub fn is_access_token(token: &str) -> bool {
    token.starts_with(ACCESS_PREFIX)
}

fn unauthenticated() -> ApiError {
    ApiError::new(ErrorCode::Unauthenticated)
}

/// Resolves an access token to the caller.
///
/// # Errors
/// `401 unauthenticated` for an unknown, revoked or expired token, a refresh token, or a token
/// of a closed grant.
pub async fn authenticate(pool: &PgPool, token: &str) -> ApiResult<Identity> {
    let row: Option<Lookup> = sqlx::query_as("SELECT * FROM oauth_token_lookup($1)")
        .bind(token_hash(token))
        .fetch_optional(pool)
        .await?;
    let row = row
        .filter(|r| r.usable && r.kind == "access")
        .ok_or_else(unauthenticated)?;
    Ok(Identity {
        user_id: row.user_id,
        session_id: None,
        actor_kind: ActorKind::User,
        oauth: Some(OAuthBinding {
            grant_id: row.grant_id,
            client_pk: row.client_pk,
            client_id: row.client_id,
            workspace_id: row.workspace_id,
            scopes: row.scopes.into_iter().collect(),
            resource: row.resource,
        }),
    })
}

/// Checks a request of an OAuth token against its binding. Order of answers: wrong resource
/// `401`, other workspace `404`, operation or scope not allowed `403`.
///
/// # Errors
/// See above.
pub fn check_request(
    state: &AppState,
    method: &Method,
    path: &str,
    workspace_in_path: Option<Uuid>,
    binding: &OAuthBinding,
) -> ApiResult<()> {
    // A token of another resource (for example an MCP token on `/v1`) is not valid here.
    if binding.resource != state.config.oauth.api_resource {
        return Err(unauthenticated());
    }
    match workspace_in_path {
        Some(ws) if ws != binding.workspace_id => return Err(ApiError::new(ErrorCode::NotFound)),
        Some(_) => {}
        None => return Err(ApiError::new(ErrorCode::Forbidden)),
    }
    let needed = scopes::operation_for(method, path)
        .and_then(scopes::scope_of_operation)
        .ok_or_else(|| ApiError::new(ErrorCode::Forbidden))?;
    if binding.scopes.contains(needed) {
        Ok(())
    } else {
        Err(ApiError::new(ErrorCode::Forbidden)
            .with_errors(vec![FieldIssue::new("scope", "insufficient_scope")]))
    }
}

/// Workspace gates of an OAuth token (ADR-0015 rules 3.8, 3.9, 7):
/// the open API needs a Business workspace; MCP in a Home workspace needs the paid plan
/// (trial included) and the workspace switch must be on; a write call needs Redis (fail
/// closed).
///
/// # Errors
/// `403 entitlement_required`, `403 forbidden` (`mcp_disabled`), `503` without Redis on a write.
pub async fn gate(state: &AppState, method: &Method, binding: &OAuthBinding) -> ApiResult<()> {
    let ctx = DbContext {
        user_id: None,
        workspace_id: Some(binding.workspace_id),
        actor_kind: ActorKind::User,
    };
    let (mode, paid, _) = workspace_mode_and_paid(&state.db, &ctx, binding.workspace_id).await?;
    if binding.resource == state.config.oauth.api_resource {
        if mode != "business" {
            return Err(ApiError::new(ErrorCode::EntitlementRequired));
        }
    } else {
        if mode == "home" && !paid {
            return Err(ApiError::new(ErrorCode::EntitlementRequired));
        }
        if !mcp_enabled(&state.db, &ctx, binding.workspace_id).await? {
            return Err(ApiError::new(ErrorCode::Forbidden)
                .with_errors(vec![FieldIssue::new("/", "mcp_disabled")]));
        }
    }
    let safe = matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS);
    if !safe && !state.redis.ping().await {
        tracing::warn!("redis unavailable; OAuth write refused (fail closed)");
        return Err(ApiError::unavailable(5));
    }
    Ok(())
}

/// The MCP switch of a workspace.
///
/// # Errors
/// Database errors.
pub async fn mcp_enabled(pool: &PgPool, ctx: &DbContext, workspace_id: Uuid) -> ApiResult<bool> {
    let mut tx = db::begin(pool, ctx).await?;
    let on: Option<bool> = sqlx::query_scalar("SELECT mcp_enabled FROM workspaces WHERE id = $1")
        .bind(workspace_id)
        .fetch_optional(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(on.unwrap_or(false))
}
