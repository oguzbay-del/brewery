//! The authorization server endpoints (ADR-0015 rule 2): metadata, `/authorize`, `/token`,
//! `/revoke` (RFC 7009) and `/register` (RFC 7591).
//!
//! These endpoints follow the OAuth RFCs, so their errors use the OAuth JSON format
//! (`error`, `error_description`), not the problem format of `/v1`.

use std::{collections::HashMap, net::SocketAddr};

use axum::{
    Extension, Form, Json,
    extract::{ConnectInfo, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use db::{ActorKind, AuditEvent, DbContext, NewAuditEntry};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use url::Url;
use uuid::Uuid;

use super::{
    ACCESS_PREFIX, ACCESS_TTL_SECS, REFRESH_PREFIX, REFRESH_TTL_SECS, REQUEST_TTL_SECS,
    client::{self, Client, ClientError, Metadata},
    scopes,
};
use crate::{
    AppState,
    auth::tokens::{new_token, token_hash},
};

/// An OAuth error answer.
#[derive(Debug)]
pub struct OAuthError {
    status: StatusCode,
    error: &'static str,
    description: String,
}

impl OAuthError {
    fn new(status: StatusCode, error: &'static str, description: impl Into<String>) -> Self {
        Self {
            status,
            error,
            description: description.into(),
        }
    }

    fn bad(error: &'static str, description: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, error, description)
    }

    fn server(e: &impl std::fmt::Display) -> Self {
        tracing::error!(error = %e, "oauth server error");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "server_error",
            "internal error",
        )
    }
}

impl From<sqlx::Error> for OAuthError {
    fn from(e: sqlx::Error) -> Self {
        Self::server(&e)
    }
}

impl From<ClientError> for OAuthError {
    fn from(e: ClientError) -> Self {
        match e {
            ClientError::Invalid(m) => Self::new(StatusCode::UNAUTHORIZED, "invalid_client", m),
            ClientError::Unavailable(m) => Self::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "temporarily_unavailable",
                m,
            ),
            ClientError::Db(e) => Self::server(&e),
        }
    }
}

fn no_store(mut response: Response) -> Response {
    let h = response.headers_mut();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    response
}

impl IntoResponse for OAuthError {
    fn into_response(self) -> Response {
        no_store(
            (
                self.status,
                Json(json!({"error": self.error, "error_description": self.description})),
            )
                .into_response(),
        )
    }
}

type Params = HashMap<String, String>;

fn param<'a>(p: &'a Params, name: &str) -> Option<&'a str> {
    p.get(name).map(String::as_str).filter(|v| !v.is_empty())
}

// --- metadata ----------------------------------------------------------------------------

/// `GET /.well-known/oauth-authorization-server` (RFC 8414).
#[utoipa::path(
    get,
    path = "/.well-known/oauth-authorization-server",
    tag = "oauth",
    operation_id = "oauth_server_metadata",
    responses((status = 200, description = "Authorization server metadata (RFC 8414)", body = Object, example = json!({
        "issuer": "https://auth.gristora.com", "authorization_endpoint": "https://auth.gristora.com/authorize",
        "token_endpoint": "https://auth.gristora.com/token", "code_challenge_methods_supported": ["S256"]})))
)]
pub async fn server_metadata(State(state): State<AppState>) -> Response {
    let cfg = &state.config.oauth;
    let issuer = cfg.issuer.trim_end_matches('/');
    let doc = json!({
        "issuer": cfg.issuer,
        "authorization_endpoint": format!("{issuer}/authorize"),
        "token_endpoint": format!("{issuer}/token"),
        "revocation_endpoint": format!("{issuer}/revoke"),
        "registration_endpoint": format!("{issuer}/register"),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "code_challenge_methods_supported": ["S256"],
        "client_id_metadata_document_supported": true,
        "authorization_response_iss_parameter_supported": true,
        "scopes_supported": scopes::advertised(),
        "token_endpoint_auth_methods_supported": ["none"],
        "revocation_endpoint_auth_methods_supported": ["none"],
    });
    cached(Json(doc).into_response())
}

fn cached(mut response: Response) -> Response {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=3600"),
    );
    response
}

fn resource_doc(cfg: &super::OAuthConfig, resource: &str, read_only_scopes: bool) -> Value {
    let list: Vec<&str> = scopes::SCOPES
        .iter()
        .filter(|s| !read_only_scopes || !s.write)
        .map(|s| s.name)
        .collect();
    json!({
        "resource": resource,
        "authorization_servers": [cfg.issuer],
        "scopes_supported": list,
        "bearer_methods_supported": ["header"],
    })
}

/// `GET /.well-known/oauth-protected-resource/mcp` (RFC 9728). Write scopes are requested when
/// needed, so the document lists the read scopes.
#[utoipa::path(
    get,
    path = "/.well-known/oauth-protected-resource/mcp",
    tag = "oauth",
    operation_id = "oauth_resource_metadata",
    responses((status = 200, description = "Protected resource metadata of MCP (RFC 9728)", body = Object, example = json!({
        "resource": "https://api.gristora.com/mcp", "authorization_servers": ["https://auth.gristora.com"],
        "scopes_supported": ["recipes:read"], "bearer_methods_supported": ["header"]})))
)]
pub async fn mcp_resource_metadata(State(state): State<AppState>) -> Response {
    let cfg = &state.config.oauth;
    cached(Json(resource_doc(cfg, &cfg.mcp_resource, true)).into_response())
}

/// `GET /.well-known/oauth-protected-resource/v1` (RFC 9728), the open API.
#[utoipa::path(
    get,
    path = "/.well-known/oauth-protected-resource/v1",
    tag = "oauth",
    operation_id = "oauth_api_resource_metadata",
    responses((status = 200, description = "Protected resource metadata of the open API (RFC 9728)", body = Object, example = json!({
        "resource": "https://api.gristora.com/v1", "authorization_servers": ["https://auth.gristora.com"],
        "scopes_supported": ["recipes:read"], "bearer_methods_supported": ["header"]})))
)]
pub async fn api_resource_metadata(State(state): State<AppState>) -> Response {
    let cfg = &state.config.oauth;
    cached(Json(resource_doc(cfg, &cfg.api_resource, false)).into_response())
}

// --- authorize ---------------------------------------------------------------------------

/// The redirect address with `pairs` and `iss` added to its query.
pub(super) fn redirect_url(
    state: &AppState,
    redirect_uri: &str,
    pairs: &[(&str, &str)],
) -> Result<String, OAuthError> {
    let mut url =
        Url::parse(redirect_uri).map_err(|_| OAuthError::bad("invalid_request", "redirect_uri"))?;
    {
        let mut q = url.query_pairs_mut();
        for (k, v) in pairs {
            q.append_pair(k, v);
        }
        q.append_pair("iss", &state.config.oauth.issuer);
    }
    Ok(url.to_string())
}

fn redirect_with(
    state: &AppState,
    redirect_uri: &str,
    pairs: &[(&str, &str)],
) -> Result<Redirect, OAuthError> {
    redirect_url(state, redirect_uri, pairs).map(|u| Redirect::to(&u))
}

fn error_redirect(
    state: &AppState,
    redirect_uri: &str,
    oauth_state: Option<&str>,
    error: &str,
    description: &str,
) -> Response {
    let mut pairs = vec![("error", error), ("error_description", description)];
    if let Some(s) = oauth_state {
        pairs.push(("state", s));
    }
    match redirect_with(state, redirect_uri, &pairs) {
        Ok(r) => no_store(r.into_response()),
        Err(e) => e.into_response(),
    }
}

/// `GET /authorize`: validates the request, stores it and sends the browser to the consent
/// screen. Only the code flow with PKCE `S256` exists.
#[utoipa::path(
    get,
    path = "/authorize",
    tag = "oauth",
    operation_id = "oauth_authorize",
    params(
        ("response_type" = String, Query, description = "`code`"),
        ("client_id" = String, Query, description = "HTTPS address of the client metadata document, or a registered id"),
        ("redirect_uri" = String, Query, description = "Exact registered redirect address"),
        ("state" = String, Query, description = "Required"),
        ("code_challenge" = String, Query, description = "PKCE S256 challenge"),
        ("code_challenge_method" = String, Query, description = "`S256`"),
        ("resource" = String, Query, description = "MCP or open API address (RFC 8707)"),
        ("scope" = Option<String>, Query, description = "Space-separated scopes"),
    ),
    responses(
        (status = 302, description = "Redirect to the consent page, or to the client with an error", body = String, content_type = "text/plain", example = json!("")),
        (status = 400, description = "Error about the client or redirect address", body = Object, example = json!({"error": "invalid_request", "error_description": "redirect_uri is not registered"})),
    )
)]
pub async fn authorize(State(state): State<AppState>, Query(p): Query<Params>) -> Response {
    // Errors about the client or the redirect address never redirect (RFC 6749 rule 4.1.2.1).
    let Some(client_id) = param(&p, "client_id") else {
        return OAuthError::bad("invalid_request", "client_id is required").into_response();
    };
    let client = match client::resolve(&state, client_id).await {
        Ok(c) => c,
        Err(ClientError::Invalid(m)) => {
            return OAuthError::bad("invalid_client", m).into_response();
        }
        Err(e) => return OAuthError::from(e).into_response(),
    };
    let Some(redirect_uri) = param(&p, "redirect_uri") else {
        return OAuthError::bad("invalid_request", "redirect_uri is required").into_response();
    };
    if !client.redirect_uris.iter().any(|u| u == redirect_uri) {
        return OAuthError::bad("invalid_request", "redirect_uri is not registered")
            .into_response();
    }
    let oauth_state = param(&p, "state");
    let fail = |error: &str, description: &str| {
        error_redirect(&state, redirect_uri, oauth_state, error, description)
    };
    let Some(oauth_state_value) = oauth_state else {
        return fail("invalid_request", "state is required");
    };
    if oauth_state_value.len() > 512 {
        return fail("invalid_request", "state is too long");
    }
    if param(&p, "response_type") != Some("code") {
        return fail(
            "unsupported_response_type",
            "only response_type code exists",
        );
    }
    if param(&p, "code_challenge_method") != Some("S256") {
        return fail("invalid_request", "code_challenge_method must be S256");
    }
    let Some(challenge) = param(&p, "code_challenge").filter(|c| valid_challenge(c)) else {
        return fail(
            "invalid_request",
            "code_challenge is not a valid S256 value",
        );
    };
    let Some(resource) = param(&p, "resource").filter(|r| state.config.oauth.resource_known(r))
    else {
        return fail("invalid_target", "resource is missing or unknown");
    };
    let requested: Vec<String> = match param(&p, "scope") {
        None => scopes::default_scopes()
            .into_iter()
            .map(String::from)
            .collect(),
        Some(raw) => {
            let mut out: Vec<String> = Vec::new();
            for s in raw.split(' ').filter(|s| !s.is_empty()) {
                if s != scopes::OFFLINE_ACCESS && scopes::info(s).is_none() {
                    return fail("invalid_scope", "unknown scope");
                }
                if s != scopes::OFFLINE_ACCESS && !out.iter().any(|x| x == s) {
                    out.push(s.to_owned());
                }
            }
            if out.is_empty() {
                return fail("invalid_scope", "no scope requested");
            }
            out
        }
    };
    let token = new_token();
    let created = sqlx::query("SELECT oauth_request_create($1, $2, $3, $4, $5, $6, $7, $8, $9)")
        .bind(Uuid::now_v7())
        .bind(token_hash(&token))
        .bind(client.id)
        .bind(redirect_uri)
        .bind(&requested)
        .bind(oauth_state_value)
        .bind(challenge)
        .bind(resource)
        .bind(REQUEST_TTL_SECS)
        .execute(&state.db)
        .await;
    if let Err(e) = created {
        return OAuthError::from(e).into_response();
    }
    let consent = format!(
        "{}{}request={token}",
        state.config.oauth.consent_url,
        if state.config.oauth.consent_url.contains('?') {
            '&'
        } else {
            '?'
        }
    );
    no_store(Redirect::to(&consent).into_response())
}

fn valid_challenge(c: &str) -> bool {
    c.len() == 43 && URL_SAFE_NO_PAD.decode(c).is_ok_and(|b| b.len() == 32)
}

// --- token -------------------------------------------------------------------------------

fn valid_verifier(v: &str) -> bool {
    (43..=128).contains(&v.len())
        && v.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~'))
}

fn pkce_matches(verifier: &str, challenge: &str) -> bool {
    let computed = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let (a, b) = (computed.as_bytes(), challenge.as_bytes());
    a.len() == b.len() && a.iter().zip(b).fold(0_u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

async fn limited(state: &AppState, key: String, limit: i64, ttl_secs: u64) -> bool {
    state
        .redis
        .incr_expire(key, ttl_secs)
        .await
        .is_some_and(|n| n > limit)
}

fn token_response(access: &str, refresh: &str, scopes: &[String]) -> Response {
    no_store(
        Json(json!({
            "access_token": access,
            "token_type": "Bearer",
            "expires_in": ACCESS_TTL_SECS,
            "refresh_token": refresh,
            "scope": scopes.join(" "),
        }))
        .into_response(),
    )
}

async fn audit(
    state: &AppState,
    workspace_id: Uuid,
    user_id: Uuid,
    event: AuditEvent,
    grant_id: Uuid,
    client: &str,
    value: Value,
) {
    let ctx = DbContext {
        user_id: Some(user_id),
        workspace_id: Some(workspace_id),
        actor_kind: ActorKind::User,
    };
    let result = async {
        let mut tx = db::begin(&state.db, &ctx).await?;
        db::append_audit(
            &mut tx,
            workspace_id,
            &NewAuditEntry {
                actor_kind: ActorKind::User,
                actor_id: Some(user_id),
                event,
                record_kind: "oauth_grant".to_owned(),
                record_id: Some(grant_id),
                previous_value: None,
                new_value: Some(json!({"client_id": client, "detail": value})),
                reason: None,
            },
        )
        .await?;
        tx.commit().await.map_err(db::Error::from)
    }
    .await;
    if let Err(e) = result {
        tracing::error!(error = %e, "oauth audit row failed");
    }
}

/// `POST /token`: authorization code exchange and refresh with rotation.
#[utoipa::path(
    post,
    path = "/token",
    tag = "oauth",
    operation_id = "oauth_token",
    request_body(content = Object, content_type = "application/x-www-form-urlencoded", description = "authorization_code or refresh_token grant"),
    responses(
        (status = 200, description = "Tokens", body = Object, example = json!({
            "access_token": "gso_at_abc", "token_type": "Bearer", "expires_in": 3600,
            "refresh_token": "gso_rt_abc", "scope": "recipes:read"})),
        (status = 400, description = "OAuth error", body = Object, example = json!({"error": "invalid_grant", "error_description": "the code was used before"})),
        (status = 401, description = "Unknown client", body = Object, example = json!({"error": "invalid_client", "error_description": "unknown client"})),
    )
)]
pub async fn token(
    State(state): State<AppState>,
    Form(p): Form<Params>,
) -> Result<Response, OAuthError> {
    let client_id = param(&p, "client_id")
        .ok_or_else(|| OAuthError::bad("invalid_request", "client_id is required"))?;
    let client = client::resolve(&state, client_id).await?;
    if limited(&state, format!("oauth:token:{}", client.id), 30, 60).await {
        return Err(OAuthError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "temporarily_unavailable",
            "too many token requests",
        ));
    }
    match param(&p, "grant_type") {
        Some("authorization_code") => exchange_code(&state, &client, &p).await,
        Some("refresh_token") => refresh(&state, &client, &p).await,
        _ => Err(OAuthError::bad(
            "unsupported_grant_type",
            "use authorization_code or refresh_token",
        )),
    }
}

type CodeRow = (Uuid, Uuid, String, String, Vec<String>, String, bool);
type RefreshRow = (
    String,
    Option<Uuid>,
    Option<Uuid>,
    Option<Uuid>,
    Option<Vec<String>>,
    Option<String>,
);

fn invalid_grant(m: &str) -> OAuthError {
    OAuthError::bad("invalid_grant", m)
}

async fn exchange_code(
    state: &AppState,
    client: &Client,
    p: &Params,
) -> Result<Response, OAuthError> {
    let code = param(p, "code").ok_or_else(|| OAuthError::bad("invalid_request", "code"))?;
    let verifier = param(p, "code_verifier")
        .filter(|v| valid_verifier(v))
        .ok_or_else(|| OAuthError::bad("invalid_request", "code_verifier"))?;
    let redirect_uri = param(p, "redirect_uri")
        .ok_or_else(|| OAuthError::bad("invalid_request", "redirect_uri"))?;
    let mut tx = state.db.begin().await?;
    let row: Option<CodeRow> = sqlx::query_as("SELECT * FROM oauth_code_consume($1)")
        .bind(token_hash(code))
        .fetch_optional(&mut *tx)
        .await?;
    let Some((grant_id, client_pk, stored_redirect, challenge, granted, resource, replayed)) = row
    else {
        return Err(invalid_grant("the code is unknown or expired"));
    };
    if replayed {
        // The grant is revoked inside the function; keep that change.
        tx.commit().await?;
        return Err(invalid_grant("the code was used before"));
    }
    if client_pk != client.id
        || stored_redirect != redirect_uri
        || !pkce_matches(verifier, &challenge)
    {
        // The code is spent either way (the consume above is committed).
        tx.commit().await?;
        return Err(invalid_grant(
            "client, redirect_uri or code_verifier does not match",
        ));
    }
    if param(p, "resource").is_some_and(|r| r != resource) {
        tx.commit().await?;
        return Err(OAuthError::bad(
            "invalid_target",
            "resource does not match the code",
        ));
    }
    let (access, refresh) = (
        format!("{ACCESS_PREFIX}{}", new_token()),
        format!("{REFRESH_PREFIX}{}", new_token()),
    );
    let issued: bool = sqlx::query_scalar("SELECT oauth_token_issue($1, $2, $3, $4, $5, $6)")
        .bind(grant_id)
        .bind(&granted)
        .bind(token_hash(&access))
        .bind(token_hash(&refresh))
        .bind(ACCESS_TTL_SECS)
        .bind(REFRESH_TTL_SECS)
        .fetch_one(&mut *tx)
        .await?;
    if !issued {
        tx.commit().await?;
        return Err(invalid_grant("the grant is closed"));
    }
    tx.commit().await?;
    Ok(token_response(&access, &refresh, &granted))
}

async fn refresh(state: &AppState, client: &Client, p: &Params) -> Result<Response, OAuthError> {
    let old = param(p, "refresh_token")
        .ok_or_else(|| OAuthError::bad("invalid_request", "refresh_token"))?;
    let requested: Option<Vec<String>> = param(p, "scope").map(|s| {
        s.split(' ')
            .filter(|x| !x.is_empty() && *x != scopes::OFFLINE_ACCESS)
            .map(String::from)
            .collect()
    });
    let (access, new_refresh) = (
        format!("{ACCESS_PREFIX}{}", new_token()),
        format!("{REFRESH_PREFIX}{}", new_token()),
    );
    let row: RefreshRow =
        sqlx::query_as("SELECT * FROM oauth_refresh_rotate($1, $2, $3, $4, $5, $6, $7)")
            .bind(token_hash(old))
            .bind(client.id)
            .bind(requested)
            .bind(token_hash(&access))
            .bind(token_hash(&new_refresh))
            .bind(ACCESS_TTL_SECS)
            .bind(REFRESH_TTL_SECS)
            .fetch_one(&state.db)
            .await?;
    match row {
        (status, Some(grant), Some(user), Some(ws), Some(granted), _) if status == "ok" => {
            let _ = (grant, user, ws);
            Ok(token_response(&access, &new_refresh, &granted))
        }
        (status, Some(grant), Some(user), Some(ws), _, _) if status == "reuse" => {
            audit(
                state,
                ws,
                user,
                AuditEvent::OauthRefreshReuse,
                grant,
                &client.client_id,
                json!({"reason": "refresh_reuse"}),
            )
            .await;
            Err(invalid_grant(
                "the refresh token was used before; the grant is revoked",
            ))
        }
        (status, ..) if status == "invalid_scope" => Err(OAuthError::bad(
            "invalid_scope",
            "a refresh cannot widen the scope",
        )),
        _ => Err(invalid_grant("the refresh token is not valid")),
    }
}

// --- revoke ------------------------------------------------------------------------------

/// `POST /revoke` (RFC 7009). The answer is `200` for any token, known or not.
#[utoipa::path(
    post,
    path = "/revoke",
    tag = "oauth",
    operation_id = "oauth_revoke",
    request_body(content = Object, content_type = "application/x-www-form-urlencoded", description = "token and client_id (RFC 7009)"),
    responses(
        (status = 200, description = "Revoked, or the token was not known", body = String, content_type = "text/plain", example = json!("")),
        (status = 400, description = "OAuth error", body = Object, example = json!({"error": "invalid_request", "error_description": "token"})),
        (status = 401, description = "Unknown client", body = Object, example = json!({"error": "invalid_client", "error_description": "unknown client"})),
    )
)]
pub async fn revoke(
    State(state): State<AppState>,
    Form(p): Form<Params>,
) -> Result<Response, OAuthError> {
    let client_id = param(&p, "client_id")
        .ok_or_else(|| OAuthError::bad("invalid_request", "client_id is required"))?;
    let client = client::resolve(&state, client_id).await?;
    let token = param(&p, "token").ok_or_else(|| OAuthError::bad("invalid_request", "token"))?;
    let rows: Vec<(Uuid, Uuid, Uuid, bool)> =
        sqlx::query_as("SELECT * FROM oauth_token_revoke($1, $2)")
            .bind(token_hash(token))
            .bind(client.id)
            .fetch_all(&state.db)
            .await?;
    for (grant, ws, user, whole) in rows {
        if whole {
            audit(
                &state,
                ws,
                user,
                AuditEvent::OauthGrantRevoke,
                grant,
                &client.client_id,
                json!({"by": "client"}),
            )
            .await;
        }
    }
    Ok(no_store(StatusCode::OK.into_response()))
}

// --- register ----------------------------------------------------------------------------

/// `POST /register` (RFC 7591) for clients that cannot use a metadata document. Only public
/// clients register; registration gives no access by itself.
#[utoipa::path(
    post,
    path = "/register",
    tag = "oauth",
    operation_id = "oauth_register",
    request_body(content = Object, content_type = "application/json", description = "Client metadata (RFC 7591); public clients only"),
    responses(
        (status = 201, description = "Registered client", body = Object, example = json!({
            "client_id": "dcr_0192f1c07a3b7000800000000000000a", "client_name": "Agent",
            "redirect_uris": ["https://agent.example/cb"], "grant_types": ["authorization_code", "refresh_token"],
            "response_types": ["code"], "token_endpoint_auth_method": "none", "client_id_issued_at": 1_795_000_000})),
        (status = 400, description = "Invalid client metadata", body = Object, example = json!({"error": "invalid_client_metadata", "error_description": "redirect_uris"})),
    )
)]
pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    connect: Option<Extension<ConnectInfo<SocketAddr>>>,
    body: Result<Json<Value>, axum::extract::rejection::JsonRejection>,
) -> Result<Response, OAuthError> {
    let ip = crate::ratelimit::client_ip_of(
        &headers,
        connect.map(|c| c.0.0),
        state.config.rate.trust_forwarded_for,
    );
    if limited(&state, format!("oauth:dcr:{ip}"), 10, 3600).await {
        return Err(OAuthError::new(
            StatusCode::TOO_MANY_REQUESTS,
            "temporarily_unavailable",
            "too many registrations",
        ));
    }
    let Json(doc) =
        body.map_err(|_| OAuthError::bad("invalid_client_metadata", "body is not JSON"))?;
    let meta = Metadata::from_json(&doc, state.config.oauth.allow_insecure_cimd)
        .map_err(|m| OAuthError::bad("invalid_client_metadata", m))?;
    let client_id = format!("dcr_{}", Uuid::new_v4().simple());
    let id: Uuid = sqlx::query_scalar(
        "SELECT oauth_client_upsert($1, $2, 'dcr', $3, $4, $5, $6, 'none', NULL)",
    )
    .bind(Uuid::now_v7())
    .bind(&client_id)
    .bind(&meta.name)
    .bind(&meta.logo_uri)
    .bind(&meta.client_uri)
    .bind(&meta.redirect_uris)
    .fetch_one(&state.db)
    .await?;
    let _ = id;
    Ok(no_store(
        (
            StatusCode::CREATED,
            Json(json!({
                "client_id": client_id,
                "client_name": meta.name,
                "redirect_uris": meta.redirect_uris,
                "grant_types": ["authorization_code", "refresh_token"],
                "response_types": ["code"],
                "token_endpoint_auth_method": "none",
                "client_id_issued_at": chrono::Utc::now().timestamp(),
            })),
        )
            .into_response(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_s256_matches_the_rfc_example() {
        // RFC 7636 appendix B.
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
        assert!(valid_verifier(verifier));
        assert!(valid_challenge(challenge));
        assert!(pkce_matches(verifier, challenge));
        assert!(!pkce_matches("x".repeat(43).as_str(), challenge));
    }

    #[test]
    fn verifier_shape_is_checked() {
        assert!(!valid_verifier("short"));
        assert!(!valid_verifier(&"a".repeat(129)));
        assert!(!valid_verifier(&format!("{}!", "a".repeat(43))));
    }
}
