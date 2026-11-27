//! OAuth 2.1 authorization server (ADR-0015, M2).
//!
//! - [`server`]: metadata, `/authorize`, `/token`, `/revoke`, `/register`.
//! - [`client`]: client documents (CIMD) and registered clients (DCR).
//! - [`consent`]: the API of the consent screen. [`apps`]: connected apps and the workspace
//!   switch.
//! - [`verify`]: bearer verification and the binding checks. [`scopes`]: scopes and the
//!   operation allow list.
//!
//! Tokens are opaque and stored as SHA-256 digests (tables `oauth_*`, migration `0257`). A
//! token is not a session: it binds to one user, workspace, client and resource, and it never
//! passes a second step.

pub mod apps;
pub mod client;
pub mod consent;
pub mod scopes;
pub mod server;
pub mod verify;

use axum::{
    Router,
    routing::{delete, get, post},
};

pub use verify::OAuthBinding;

use crate::{AppState, authenticated};

/// Prefix of access tokens.
pub const ACCESS_PREFIX: &str = "gso_at_";
/// Prefix of refresh tokens.
pub const REFRESH_PREFIX: &str = "gso_rt_";
/// Access token lifetime (1 hour).
pub const ACCESS_TTL_SECS: i32 = 3600;
/// Refresh token lifetime without use (30 days).
pub const REFRESH_TTL_SECS: i32 = 30 * 24 * 3600;
/// Authorization code lifetime (60 seconds).
pub const CODE_TTL_SECS: i32 = 60;
/// How long a request waits for the consent screen (10 minutes).
pub const REQUEST_TTL_SECS: i32 = 600;

/// Settings of the authorization server.
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    /// The issuer (`OAUTH_ISSUER`).
    pub issuer: String,
    /// Resource address of the open API (`OAUTH_API_RESOURCE`).
    pub api_resource: String,
    /// Resource address of MCP (`OAUTH_MCP_RESOURCE`).
    pub mcp_resource: String,
    /// Address of the consent page (`OAUTH_CONSENT_URL`; default `<WEB_BASE_URL>/oauth/consent`).
    pub consent_url: String,
    /// Test and local setting: fetch client documents from any `http(s)` address, port and
    /// host (`OAUTH_CIMD_ALLOW_INSECURE`). Never set it in production.
    pub allow_insecure_cimd: bool,
}

impl OAuthConfig {
    /// Reads the settings; every value has a default.
    pub fn from_lookup<F>(lookup: &F, web_base_url: &str) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        let get = |name: &str, default: String| {
            lookup(name)
                .map(|v| v.trim().to_owned())
                .filter(|v| !v.is_empty())
                .unwrap_or(default)
        };
        Self {
            issuer: get("OAUTH_ISSUER", "https://auth.gristora.com".to_owned()),
            api_resource: get(
                "OAUTH_API_RESOURCE",
                "https://api.gristora.com/v1".to_owned(),
            ),
            mcp_resource: get(
                "OAUTH_MCP_RESOURCE",
                "https://api.gristora.com/mcp".to_owned(),
            ),
            consent_url: get(
                "OAUTH_CONSENT_URL",
                format!("{}/oauth/consent", web_base_url.trim_end_matches('/')),
            ),
            allow_insecure_cimd: lookup("OAUTH_CIMD_ALLOW_INSECURE")
                .is_some_and(|v| matches!(v.trim(), "1" | "true")),
        }
    }

    /// Whether `resource` is one of the two resources of this server.
    #[must_use]
    pub fn resource_known(&self, resource: &str) -> bool {
        resource == self.api_resource || resource == self.mcp_resource
    }
}

/// The routes. The authorization server endpoints need no sign-in; the consent, connected apps
/// and switch endpoints do.
pub fn routes(state: &AppState) -> Router<AppState> {
    let public = Router::new()
        .route(
            "/.well-known/oauth-authorization-server",
            get(server::server_metadata),
        )
        .route(
            "/.well-known/oauth-protected-resource/mcp",
            get(server::mcp_resource_metadata),
        )
        .route(
            "/.well-known/oauth-protected-resource/v1",
            get(server::api_resource_metadata),
        )
        .route("/authorize", get(server::authorize))
        .route("/token", post(server::token))
        .route("/revoke", post(server::revoke))
        .route("/register", post(server::register));
    let signed_in = authenticated(
        state,
        Router::new()
            .route("/v1/me/oauth/requests/{request}", get(consent::request_get))
            .route(
                "/v1/me/oauth/requests/{request}/decision",
                post(consent::request_decide),
            )
            .route("/v1/me/connected-apps", get(apps::connected_app_list))
            .route("/v1/me/connected-apps", delete(apps::connected_app_clear))
            .route(
                "/v1/me/connected-apps/{grant_id}",
                delete(apps::connected_app_revoke),
            )
            .route(
                "/v1/workspaces/{workspace_id}/mcp",
                get(apps::workspace_mcp_get).put(apps::workspace_mcp_set),
            ),
    );
    public.merge(signed_in)
}

mod spec {
    use crate::error::ErrorCode;
    use crate::error::ErrorCode::{
        Forbidden, Internal, NotFound, RateLimited, Unauthenticated, Unavailable, ValidationFailed,
    };
    use crate::openapi::{OperationSpec, file_op, op};

    const META: &[ErrorCode] = &[Internal];
    const PUBLIC: &[ErrorCode] = &[RateLimited, Internal, Unavailable];
    const ME_READ: &[ErrorCode] = &[Unauthenticated, Forbidden, NotFound, RateLimited, Internal];
    const ME_WRITE: &[ErrorCode] = &[
        Unauthenticated,
        Forbidden,
        NotFound,
        ValidationFailed,
        ErrorCode::EntitlementRequired,
        RateLimited,
        Internal,
    ];
    const SWITCH: &[ErrorCode] = &[Unauthenticated, Forbidden, NotFound, RateLimited, Internal];

    pub(super) const OPERATIONS: &[OperationSpec] = &[
        op("oauth_server_metadata", true, None, META),
        op("oauth_resource_metadata", true, None, META),
        op("oauth_api_resource_metadata", true, None, META),
        op("oauth_authorize", true, None, PUBLIC),
        op("oauth_token", true, None, PUBLIC),
        op("oauth_revoke", true, None, PUBLIC),
        op("oauth_register", true, None, PUBLIC),
        file_op("oauth_request_get", None, None, ME_READ),
        file_op("oauth_request_decide", None, None, ME_WRITE),
        file_op("connected_app_list", None, None, ME_READ),
        file_op("connected_app_revoke", None, None, ME_READ),
        file_op("connected_app_clear", None, None, ME_READ),
        file_op("workspace_mcp_get", Some("workspace.manage"), None, SWITCH),
        file_op("workspace_mcp_set", Some("workspace.manage"), None, SWITCH),
    ];
}

#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(
        server::server_metadata,
        server::mcp_resource_metadata,
        server::api_resource_metadata,
        server::authorize,
        server::token,
        server::revoke,
        server::register,
        consent::request_get,
        consent::request_decide,
        apps::connected_app_list,
        apps::connected_app_revoke,
        apps::connected_app_clear,
        apps::workspace_mcp_get,
        apps::workspace_mcp_set,
    ),
    components(schemas(
        consent::ConsentRequestView,
        consent::ConsentClient,
        consent::ConsentScope,
        consent::ConsentWorkspace,
        consent::Decision,
        consent::DecisionResult,
        apps::ConnectedApp,
        apps::ConnectedApps,
        apps::McpSwitch,
        apps::McpSwitchBody,
    )),
    tags((name = "oauth", description = "OAuth 2.1 authorization server, consent, connected apps (ADR-0015)"))
)]
struct ModuleDoc;

/// Registry entry.
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "oauth",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
