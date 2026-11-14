//! Sign-in, sessions and the first workspace (ADR-0009 rules 1 to 4; T2.11).
//!
//! - [`oidc`]: Apple and Google ID token verification.
//! - [`session`]: opaque session tokens, cache, revocation.
//! - [`email`]: addresses and the sign-in mail.
//! - [`handlers`], [`setup`]: the endpoints.
//!
//! [`AuthServices`] ties them together, implements [`Authenticator`] (bearer or cookie session,
//! CSRF, workspace membership and permissions) and builds the routes.

pub mod access;
pub mod config;
pub mod email;
pub mod handlers;
pub mod oidc;
pub mod session;
pub mod setup;
pub mod tokens;

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::Duration,
};

use axum::{
    Extension, Router,
    http::{HeaderMap, Method, header},
    routing::{delete, get, post},
};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub use config::AuthConfig;
use email::Mailer;
use oidc::OidcVerifier;
use session::{ClientKind, SessionStore};

use crate::{
    AppState, authenticated,
    authz::{ScopeKind, ScopedGrant},
    context::{BoxFuture, Identity, Membership},
    error::{ApiError, ApiResult, ErrorCode, FieldIssue},
    idempotent,
    redis_client::Redis,
};
use db::ActorKind;

/// Name of the web session cookie (ADR-0009 rule 3.5).
pub const SESSION_COOKIE: &str = "__Host-session";
/// Header carrying the CSRF token.
pub const CSRF_HEADER: &str = "x-csrf-token";

/// Everything the sign-in endpoints and the authenticator need.
pub struct AuthServices {
    pub(crate) db: PgPool,
    pub(crate) config: AuthConfig,
    pub(crate) apple: Option<OidcVerifier>,
    pub(crate) google: Option<OidcVerifier>,
    pub(crate) mailer: Arc<dyn Mailer>,
    pub(crate) sessions: SessionStore,
    pub(crate) redis: Redis,
}

impl std::fmt::Debug for AuthServices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthServices").finish_non_exhaustive()
    }
}

/// How the request proves the session.
enum Credential<'a> {
    Bearer(&'a str),
    Cookie(&'a str),
}

fn credential(headers: &HeaderMap) -> ApiResult<Option<Credential<'_>>> {
    let bearer = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            let (scheme, token) = v.split_once(' ')?;
            scheme
                .eq_ignore_ascii_case("bearer")
                .then_some(token.trim())
        })
        .filter(|t| !t.is_empty());
    let cookie = headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find_map(|(name, value)| (name == SESSION_COOKIE).then_some(value))
        .filter(|t| !t.is_empty());
    match (bearer, cookie) {
        // Both at once is refused (ADR-0009 rule 3.6).
        (Some(_), Some(_)) => {
            Err(
                ApiError::new(ErrorCode::MalformedRequest).with_errors(vec![FieldIssue::new(
                    "authorization",
                    "ambiguous_credentials",
                )]),
            )
        }
        (Some(t), None) => Ok(Some(Credential::Bearer(t))),
        (None, Some(t)) => Ok(Some(Credential::Cookie(t))),
        (None, None) => Ok(None),
    }
}

impl AuthServices {
    /// Builds the services; the JWKS verifiers exist for the providers that have client ids.
    #[must_use]
    pub fn new(db: PgPool, redis: Redis, config: AuthConfig, mailer: Arc<dyn Mailer>) -> Self {
        let min_refresh = Duration::from_secs(config.jwks_min_refresh_secs);
        let verifier = |s: &Option<config::ProviderSettings>| {
            s.clone().map(|s| OidcVerifier::new(s, min_refresh))
        };
        Self {
            apple: verifier(&config.apple),
            google: verifier(&config.google),
            sessions: SessionStore::new(db.clone(), redis.clone()),
            redis,
            db,
            config,
            mailer,
        }
    }

    /// The sign-in, `/v1/me` and `/v1/workspaces` routes, ready to pass to [`crate::build`].
    pub fn routes(self: &Arc<Self>, state: &AppState) -> Router<AppState> {
        let public = Router::new()
            .route("/v1/auth/nonce", post(handlers::nonce_create))
            .route("/v1/auth/apple", post(handlers::apple_login))
            .route("/v1/auth/google", post(handlers::google_login))
            .route("/v1/auth/email/start", post(handlers::email_start))
            .route("/v1/auth/email/verify", post(handlers::email_verify));
        let me = authenticated(
            state,
            Router::new()
                .route(
                    "/v1/me/session",
                    get(handlers::session_get).delete(handlers::session_delete),
                )
                .route(
                    "/v1/me/sessions",
                    get(handlers::sessions_list).delete(handlers::sessions_revoke_all),
                )
                .route("/v1/me/sessions/{id}", delete(handlers::session_revoke))
                .route(
                    "/v1/me/identities",
                    get(handlers::identities_list).post(handlers::identity_link),
                )
                .route("/v1/me/identities/{id}", delete(handlers::identity_unlink)),
        );
        let workspaces = authenticated(
            state,
            idempotent(
                state,
                Router::new().route("/v1/workspaces", post(setup::workspace_create)),
            ),
        );
        public
            .merge(me)
            .merge(workspaces)
            .merge(access::routes(state))
            .layer(Extension(self.clone()))
    }

    /// The token the request carries, if any.
    fn token_of(headers: &HeaderMap) -> ApiResult<Option<(String, bool)>> {
        Ok(credential(headers)?.map(|c| match c {
            Credential::Bearer(t) => (t.to_owned(), false),
            Credential::Cookie(t) => (t.to_owned(), true),
        }))
    }

    /// The CSRF token of a session.
    #[must_use]
    pub fn csrf_token(&self, session_id: Uuid) -> String {
        tokens::csrf_token(&self.config.csrf_key, session_id)
    }

    /// `Set-Cookie` value that stores the session token (`max_age_secs` `None`: browser session).
    #[must_use]
    pub fn session_cookie(token: &str, max_age_secs: Option<i64>) -> String {
        let max_age = max_age_secs.map_or_else(String::new, |s| format!("; Max-Age={s}"));
        format!("{SESSION_COOKIE}={token}; HttpOnly; Secure; SameSite=Lax; Path=/{max_age}")
    }

    /// `Set-Cookie` value that removes the session cookie.
    #[must_use]
    pub fn clear_cookie() -> String {
        format!("{SESSION_COOKIE}=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0")
    }
}

fn unauthenticated() -> ApiError {
    ApiError::new(ErrorCode::Unauthenticated)
}

impl crate::Authenticator for AuthServices {
    fn authenticate<'a>(&'a self, headers: &'a HeaderMap) -> BoxFuture<'a, ApiResult<Identity>> {
        Box::pin(async move {
            let (token, _) = Self::token_of(headers)?.ok_or_else(unauthenticated)?;
            if token.len() > 256 {
                return Err(unauthenticated());
            }
            let hash = tokens::token_hash(&token);
            let record = self
                .sessions
                .lookup(&hash)
                .await?
                .ok_or_else(unauthenticated)?;
            let now = Utc::now();
            if !record.is_active(now) {
                return Err(unauthenticated());
            }
            let record = self.sessions.touch(&record, &hash, now).await;
            Ok(Identity {
                user_id: record.user_id,
                session_id: Some(record.id),
                actor_kind: if record.client_kind == ClientKind::SharedDevice {
                    ActorKind::Device
                } else {
                    ActorKind::User
                },
            })
        })
    }

    fn membership<'a>(
        &'a self,
        identity: &'a Identity,
        workspace_id: Uuid,
    ) -> BoxFuture<'a, ApiResult<Option<Membership>>> {
        Box::pin(async move {
            let ctx = db::DbContext {
                user_id: Some(identity.user_id),
                workspace_id: Some(workspace_id),
                actor_kind: identity.actor_kind,
            };
            let mut tx = db::begin(&self.db, &ctx).await?;
            let member: Option<(Uuid, i64)> = sqlx::query_as(
                "SELECT m.id, w.authz_version FROM workspace_members m \
                 JOIN workspaces w ON w.id = m.workspace_id AND w.deleted_at IS NULL \
                 WHERE m.workspace_id = $1 AND m.user_id = $2 \
                   AND m.status = 'active' AND m.deleted_at IS NULL",
            )
            .bind(workspace_id)
            .bind(identity.user_id)
            .fetch_optional(&mut *tx)
            .await?;
            let Some((member_id, version)) = member else {
                tx.commit().await?;
                return Ok(None);
            };
            // Effective permissions are cached under (member, authz_version): every role,
            // assignment or membership change bumps the version, so old entries die on their
            // own (ADR-0009 rule 8.4).
            let key = format!("authz:{workspace_id}:{member_id}:{version}");
            let cached = self
                .redis
                .run(|mut c| {
                    let key = key.clone();
                    async move {
                        redis::cmd("GET")
                            .arg(key)
                            .query_async::<Option<String>>(&mut c)
                            .await
                    }
                })
                .await
                .flatten()
                .and_then(|s| serde_json::from_str::<Membership>(&s).ok());
            if let Some(m) = cached {
                tx.commit().await?;
                return Ok(Some(m));
            }
            let rows: Vec<(Option<String>, Option<Uuid>, String)> = sqlx::query_as(
                "SELECT a.scope_kind, a.scope_id, p.permission FROM role_assignments a \
                 JOIN roles r ON r.workspace_id = a.workspace_id AND r.id = a.role_id \
                              AND r.deleted_at IS NULL \
                 JOIN role_permissions p ON p.workspace_id = r.workspace_id AND p.role_id = r.id \
                 WHERE a.workspace_id = $1 AND a.member_id = $2 AND a.deleted_at IS NULL",
            )
            .bind(workspace_id)
            .bind(member_id)
            .fetch_all(&mut *tx)
            .await?;
            tx.commit().await?;
            let mut membership = Membership::default();
            let mut scoped: BTreeMap<(ScopeKind, Uuid), BTreeSet<String>> = BTreeMap::new();
            for (kind, id, permission) in rows {
                match (kind.as_deref().and_then(ScopeKind::parse), id) {
                    (None, _) => {
                        membership.permissions.insert(permission);
                    }
                    (Some(kind), Some(id)) => {
                        scoped.entry((kind, id)).or_default().insert(permission);
                    }
                    (Some(_), None) => {}
                }
            }
            membership.scoped = scoped
                .into_iter()
                .map(|((kind, id), permissions)| ScopedGrant {
                    kind,
                    id,
                    permissions,
                })
                .collect();
            if let Ok(json) = serde_json::to_string(&membership) {
                self.redis
                    .run(|mut c| async move {
                        redis::cmd("SET")
                            .arg(key)
                            .arg(json)
                            .arg("EX")
                            .arg(300)
                            .query_async::<()>(&mut c)
                            .await
                    })
                    .await;
            }
            Ok(Some(membership))
        })
    }

    fn check_csrf<'a>(
        &'a self,
        method: &'a Method,
        headers: &'a HeaderMap,
        identity: &'a Identity,
    ) -> BoxFuture<'a, ApiResult<()>> {
        Box::pin(async move {
            let safe = matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS);
            let cookie_session = matches!(Self::token_of(headers)?, Some((_, true)));
            if safe || !cookie_session {
                return Ok(());
            }
            let forbidden = |field: &str, code: &str| {
                ApiError::new(ErrorCode::Forbidden).with_errors(vec![FieldIssue::new(field, code)])
            };
            let origin_ok = headers
                .get(header::ORIGIN)
                .and_then(|v| v.to_str().ok())
                .is_some_and(|o| self.config.web_origins.iter().any(|w| w == o));
            if !origin_ok {
                return Err(forbidden("origin", "origin_not_allowed"));
            }
            let valid = identity.session_id.is_some_and(|sid| {
                headers
                    .get(CSRF_HEADER)
                    .and_then(|v| v.to_str().ok())
                    .is_some_and(|t| tokens::csrf_valid(&self.config.csrf_key, sid, t))
            });
            if valid {
                Ok(())
            } else {
                Err(forbidden(CSRF_HEADER, "csrf_invalid"))
            }
        })
    }
}
