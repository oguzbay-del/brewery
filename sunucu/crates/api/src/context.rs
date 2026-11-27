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
    auth::{shared_device::DEVICE_KEY_HEADER, tokens::token_hash},
    error::{ApiError, ApiResult, ErrorCode},
    modules,
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
    /// Set when an OAuth access token authenticated the call (ADR-0015). Such a call has no
    /// session and never counts as a second step.
    pub oauth: Option<crate::oauth::OAuthBinding>,
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
        Ok(db::begin(pool, &self.db_context()).await?)
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
    if let Some(binding) = &identity.oauth {
        crate::oauth::verify::check_request(state, method, path, workspace_id, binding)?;
    }
    let membership = match workspace_id {
        Some(ws) => state
            .auth
            .membership(&identity, ws)
            .await?
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound))?,
        None => Membership::default(),
    };
    let membership = match &identity.oauth {
        Some(binding) => {
            crate::oauth::verify::gate(state, method, binding).await?;
            crate::oauth::scopes::narrow(membership, &binding.scopes)
        }
        None => membership,
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
    })
}

/// Reads the workspace mode, whether one active `PAID_FEATURE` entitlement exists, and the plan
/// of that entitlement.
pub(crate) async fn workspace_mode_and_paid(
    pool: &PgPool,
    db_context: &DbContext,
    workspace_id: Uuid,
) -> ApiResult<(String, bool, Option<String>)> {
    let mut tx = db::begin(pool, db_context).await?;
    let mode: String = sqlx::query_scalar("SELECT mode FROM workspaces WHERE id = $1")
        .bind(workspace_id)
        .fetch_one(&mut *tx)
        .await?;
    let access = billing::access::access(&mut tx, workspace_id).await?;
    tx.commit().await?;
    let has_paid_feature = matches!(
        access.source,
        billing::access::AccessSource::Trial
            | billing::access::AccessSource::Subscription
            | billing::access::AccessSource::Promo
            | billing::access::AccessSource::Manual
    );
    Ok((mode, has_paid_feature, access.plan_key))
}

async fn module_access(
    pool: &PgPool,
    db_context: &DbContext,
    workspace_id: Uuid,
    module_id: &str,
) -> ApiResult<modules::ModuleAccess> {
    let (mode, has_paid_feature, plan_key) =
        workspace_mode_and_paid(pool, db_context, workspace_id).await?;
    let access =
        modules::module_access_for_plan(module_id, &mode, has_paid_feature, plan_key.as_deref());
    Ok(access)
}

fn allows_method(access: modules::ModuleAccess, method: &Method, path: &str) -> bool {
    match access {
        // Requests that end access pass even when the module is off (Home has no Team
        // module, but old shares and invitations must still be removable).
        modules::ModuleAccess::Disabled => ends_access(method, path),
        modules::ModuleAccess::ReadOnly => {
            method == Method::GET || method == Method::HEAD || ends_access(method, path)
        }
        modules::ModuleAccess::Full => true,
    }
}

/// Requests that only end access. A read-only module must still allow them, so that
/// an expired subscription never keeps a shared terminal open.
fn ends_access(method: &Method, path: &str) -> bool {
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    match segments.as_slice() {
        ["v1", "shared-devices", _, "lock"] => method == Method::POST,
        ["v1", "workspaces", _, "shared-devices" | "invitations", _] => method == Method::DELETE,
        // Remove or disable a member; the handler keeps its admin and self-lockout checks.
        ["v1", "workspaces", _, "members", _] => {
            method == Method::DELETE || method == Method::PATCH
        }
        _ => false,
    }
}

/// A device id never needs percent-encoding. The handler decodes the path segment, so an
/// encoded id would skip [`device_module_layer`]; the layer refuses such a path.
fn is_encoded_device_path(path: &str) -> bool {
    path.starts_with("/v1/shared-devices/") && path.contains('%')
}

/// Parses the device id from a shared-device key route.
fn shared_device_in_path(path: &str) -> Option<Uuid> {
    let mut segments = path.trim_start_matches('/').split('/');
    if segments.next() == Some("v1") && segments.next() == Some("shared-devices") {
        return Uuid::parse_str(segments.next()?).ok();
    }
    None
}

/// Module gate for shared-device-key routes, which have no workspace id in their URL.
pub async fn device_module_layer(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path().to_owned();
    if is_encoded_device_path(&path) {
        return ApiError::new(ErrorCode::NotFound).into_response();
    }
    let Some(device_id) = shared_device_in_path(&path) else {
        return next.run(req).await;
    };
    let Some(module_id) = modules::module_for_path(&path) else {
        return next.run(req).await;
    };
    let Some(services) = state.auth_services.as_ref() else {
        return next.run(req).await;
    };
    let Some(key) = req
        .headers()
        .get(DEVICE_KEY_HEADER)
        .and_then(|value| value.to_str().ok())
    else {
        return next.run(req).await;
    };
    let device = sqlx::query_as::<_, (Uuid, Uuid)>(
        "SELECT id, workspace_id FROM find_shared_device_by_key_hash($1)",
    )
    .bind(token_hash(key))
    .fetch_optional(&services.db)
    .await;
    let (found_id, workspace_id) = match device {
        Ok(Some(row)) => row,
        Ok(None) => return next.run(req).await,
        Err(error) => return ApiError::from(error).into_response(),
    };
    if found_id != device_id {
        return next.run(req).await;
    }
    let db_context = DbContext {
        user_id: None,
        workspace_id: Some(workspace_id),
        actor_kind: ActorKind::Device,
    };
    match module_access(&services.db, &db_context, workspace_id, module_id).await {
        Ok(access) if allows_method(access, req.method(), &path) => next.run(req).await,
        Ok(_) => ApiError::new(ErrorCode::ModuleDisabled).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Middleware for authenticated routes: stores the [`RequestContext`].
pub async fn layer(State(state): State<AppState>, mut req: Request, next: Next) -> Response {
    let slot = req.extensions().get::<Slot>().cloned().unwrap_or_default();
    let path = req.uri().path().to_owned();
    let method = req.method().clone();
    match resolve(&state, &method, &path, &req.headers().clone(), &slot).await {
        Ok(ctx) => {
            if let Some(workspace_id) = ctx.workspace_id
                && !modules::is_export_path(&path)
                && let Some(module_id) = modules::module_for_path(&path)
                && !modules::is_free_module(module_id)
            {
                match module_access(&state.db, &ctx.db_context(), workspace_id, module_id).await {
                    Ok(access) if allows_method(access, &method, &path) => {}
                    Ok(_) => {
                        return ApiError::new(ErrorCode::ModuleDisabled).into_response();
                    }
                    Err(error) => return error.into_response(),
                }
            }
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
    fn only_requests_that_end_access_pass_a_read_only_module() {
        let device = Uuid::now_v7();
        let ws = Uuid::now_v7();
        let lock = format!("/v1/shared-devices/{device}/lock");
        let remove = format!("/v1/workspaces/{ws}/shared-devices/{device}");
        let switch = format!("/v1/shared-devices/{device}/switch");
        assert!(ends_access(&Method::POST, &lock));
        assert!(ends_access(&Method::DELETE, &remove));
        assert!(!ends_access(&Method::POST, &switch));
        assert!(!ends_access(&Method::PATCH, &remove));
        let member = format!("/v1/workspaces/{ws}/members/{device}");
        assert!(ends_access(&Method::DELETE, &member));
        assert!(ends_access(&Method::PATCH, &member));
        assert!(!ends_access(
            &Method::POST,
            &format!("/v1/workspaces/{ws}/members")
        ));
        let read_only = modules::ModuleAccess::ReadOnly;
        assert!(allows_method(read_only, &Method::POST, &lock));
        assert!(!allows_method(read_only, &Method::POST, &switch));
        let disabled = modules::ModuleAccess::Disabled;
        assert!(allows_method(disabled, &Method::POST, &lock));
        assert!(!allows_method(disabled, &Method::POST, &switch));
        let invitation = format!("/v1/workspaces/{ws}/invitations/{device}");
        assert!(ends_access(&Method::DELETE, &invitation));
        assert!(!ends_access(
            &Method::POST,
            &format!("/v1/workspaces/{ws}/invitations")
        ));
    }

    #[test]
    fn an_encoded_device_id_is_refused() {
        let device = Uuid::now_v7().to_string();
        let encoded = format!("%{:02X}{}", device.as_bytes()[0], &device[1..]);
        let encoded_path = format!("/v1/shared-devices/{encoded}/switch");
        assert!(shared_device_in_path(&encoded_path).is_none());
        assert!(is_encoded_device_path(&encoded_path));
        assert!(!is_encoded_device_path(&format!(
            "/v1/shared-devices/{device}/switch"
        )));
    }

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
