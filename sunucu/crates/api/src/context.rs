//! Request context: who calls, in which workspace, with which permissions (ADR-0007 rule 2.3,
//! ADR-0008 rule 1.2).
//!
//! Authentication itself arrives with T2.11 (ADR-0009): it implements [`Authenticator`]. The
//! layer here resolves the identity, checks workspace membership (`404` for non-members, never
//! `403`; ADR-0008 rule 3.3), applies the user and workspace rate limits and stores a
//! [`RequestContext`] for handlers. Every request then runs in one transaction that
//! [`RequestContext::begin`] opens with `SET LOCAL app.*` already applied.

use std::{collections::BTreeSet, pin::Pin};

use axum::{
    extract::{FromRequestParts, Request, State},
    http::{HeaderMap, Method, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use db::{ActorKind, DbContext};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    AppState,
    error::{ApiError, ApiResult, ErrorCode},
    ratelimit::Slot,
};

/// Boxed future (keeps [`Authenticator`] object safe).
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The authenticated caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// The user acting.
    pub user_id: Uuid,
    /// The session, when there is one (bearer and cookie sessions).
    pub session_id: Option<Uuid>,
    /// Who acts (`user`, `device`, ...).
    pub actor_kind: ActorKind,
}

/// A member's effective permissions inside one workspace.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Membership {
    /// Permission codes held for the whole workspace (`recipe.edit`, `cost.view`, ...).
    pub permissions: BTreeSet<String>,
    /// Permissions held only inside a recipe, batch, vessel or area (ADR-0009 rule 8.3).
    #[serde(default)]
    pub scoped: Vec<crate::authz::ScopedGrant>,
}

/// Filled in by T2.11 (sessions) and T2.13 (permissions); tests use a fake.
pub trait Authenticator: Send + Sync + 'static {
    /// Resolves the caller from the request headers (cookie or bearer token).
    ///
    /// # Errors
    /// `401 unauthenticated` when there is no valid session.
    fn authenticate<'a>(&'a self, headers: &'a HeaderMap) -> BoxFuture<'a, ApiResult<Identity>>;

    /// The caller's membership in `workspace_id`; `None` when not an active member (the API
    /// answers `404`).
    fn membership<'a>(
        &'a self,
        identity: &'a Identity,
        workspace_id: Uuid,
    ) -> BoxFuture<'a, ApiResult<Option<Membership>>>;

    /// Checks the CSRF defence of a cookie-authenticated unsafe request (ADR-0009 rule 3.5).
    /// The default accepts everything (bearer-only authenticators, tests).
    ///
    /// # Errors
    /// `403 forbidden` when the origin or token does not match.
    fn check_csrf<'a>(
        &'a self,
        _method: &'a Method,
        _headers: &'a HeaderMap,
        _identity: &'a Identity,
    ) -> BoxFuture<'a, ApiResult<()>> {
        Box::pin(async { Ok(()) })
    }
}

/// Rejects every request with `401` until real authentication (T2.11) is wired in.
#[derive(Debug, Clone, Copy, Default)]
pub struct DenyAll;

impl Authenticator for DenyAll {
    fn authenticate<'a>(&'a self, _headers: &'a HeaderMap) -> BoxFuture<'a, ApiResult<Identity>> {
        Box::pin(async { Err(ApiError::new(ErrorCode::Unauthenticated)) })
    }

    fn membership<'a>(
        &'a self,
        _identity: &'a Identity,
        _workspace_id: Uuid,
    ) -> BoxFuture<'a, ApiResult<Option<Membership>>> {
        Box::pin(async { Ok(None) })
    }
}

/// Per-request context stored by [`layer`].
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// The caller.
    pub identity: Identity,
    /// Workspace of the path (`/v1/workspaces/{id}/...`), if any.
    pub workspace_id: Option<Uuid>,
    /// Permissions held for the whole workspace (empty outside one).
    pub permissions: BTreeSet<String>,
    /// Scoped grants (see [`crate::authz`]).
    pub scoped: Vec<crate::authz::ScopedGrant>,
    /// The `idempotency_keys` row of this request (set by the idempotency layer): the work
    /// transaction marks it, so the key and the work commit together (F2.1 finding 4).
    pub idempotency_key: Option<Uuid>,
}

impl RequestContext {
    /// The database context for `SET LOCAL`.
    #[must_use]
    pub const fn db_context(&self) -> DbContext {
        DbContext {
            user_id: Some(self.identity.user_id),
            workspace_id: self.workspace_id,
            actor_kind: self.identity.actor_kind,
        }
    }

    /// Whether the caller holds `permission`.
    #[must_use]
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }

    /// Requires `permission`.
    ///
    /// # Errors
    /// `403 forbidden` when the member lacks it.
    pub fn require(&self, permission: &str) -> ApiResult<()> {
        if self.has_permission(permission) {
            Ok(())
        } else {
            Err(ApiError::new(ErrorCode::Forbidden))
        }
    }

    /// Opens the request transaction with the context applied.
    pub async fn begin(&self, pool: &PgPool) -> ApiResult<Transaction<'static, Postgres>> {
        let mut tx = db::begin(pool, &self.db_context()).await?;
        if let Some(key) = self.idempotency_key
            && self.identity.actor_kind != ActorKind::Support
        {
            sqlx::query("UPDATE idempotency_keys SET work_committed_at = now() WHERE id = $1")
                .bind(key)
                .execute(&mut *tx)
                .await?;
        }
        Ok(tx)
    }
}

impl<S: Send + Sync> FromRequestParts<S> for RequestContext {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Self>()
            .cloned()
            .ok_or_else(|| ApiError::new(ErrorCode::Unauthenticated))
    }
}

/// `/v1/workspaces/{id}/...` → the id. `Err` when the segment is not a UUID.
fn workspace_in_path(path: &str) -> Result<Option<Uuid>, ApiError> {
    let mut segments = path.trim_start_matches('/').split('/');
    if segments.next() == Some("v1")
        && segments.next() == Some("workspaces")
        && let Some(raw) = segments.next().filter(|s| !s.is_empty())
    {
        return Uuid::parse_str(raw)
            .map(Some)
            .map_err(|_| ApiError::new(ErrorCode::NotFound));
    }
    Ok(None)
}

async fn resolve(
    state: &AppState,
    method: &Method,
    path: &str,
    headers: &HeaderMap,
    slot: &Slot,
) -> ApiResult<RequestContext> {
    let workspace_id = workspace_in_path(path)?;
    let identity = state.auth.authenticate(headers).await?;
    state.auth.check_csrf(method, headers, &identity).await?;
    let membership = match workspace_id {
        Some(ws) => state
            .auth
            .membership(&identity, ws)
            .await?
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound))?,
        None => Membership::default(),
    };
    state
        .rate
        .check_account(slot, identity.user_id, workspace_id)
        .await?;
    Ok(RequestContext {
        identity,
        workspace_id,
        permissions: membership.permissions,
        scoped: membership.scoped,
        idempotency_key: None,
    })
}

/// Middleware for authenticated routes: stores the [`RequestContext`].
pub async fn layer(State(state): State<AppState>, mut req: Request, next: Next) -> Response {
    let slot = req.extensions().get::<Slot>().cloned().unwrap_or_default();
    let path = req.uri().path().to_owned();
    let method = req.method().clone();
    match resolve(&state, &method, &path, &req.headers().clone(), &slot).await {
        Ok(ctx) => {
            req.extensions_mut().insert(ctx);
            next.run(req).await
        }
        Err(e) => e.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_is_read_from_the_path() {
        let id = Uuid::now_v7();
        assert_eq!(
            workspace_in_path(&format!("/v1/workspaces/{id}/recipes")).unwrap(),
            Some(id)
        );
        assert_eq!(workspace_in_path("/v1/me/identities").unwrap(), None);
        assert_eq!(workspace_in_path("/v1/workspaces").unwrap(), None);
        assert!(workspace_in_path("/v1/workspaces/not-a-uuid/x").is_err());
    }
}
