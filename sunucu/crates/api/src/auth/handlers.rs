//! Sign-in, session and identity endpoints (ADR-0009 rules 1 to 4).

use std::{net::SocketAddr, sync::Arc};

use axum::{
    Extension, Json,
    extract::{ConnectInfo, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utoipa::ToSchema;
use uuid::Uuid;

use super::{
    AuthServices,
    email::{normalize_email, sign_in_mail},
    oidc::{OidcError, OidcVerifier},
    session::{self, ClientKind, SessionRecord},
    tokens::{hex, new_token, token_hash},
};
use crate::{
    AppState,
    context::RequestContext,
    error::{ApiError, ApiJson, ApiResult, ErrorCode, FieldIssue},
    pagination::Page,
    ratelimit::client_ip_of,
};

/// Lifetime of a nonce row and of the e-mail link (ADR-0009 rules 1.2 and 2.1).
const NONCE_TTL: Duration = Duration::minutes(10);
const EMAIL_LINK_TTL: Duration = Duration::minutes(15);
/// "Signed in within the last 10 minutes" (ADR-0009 rule 4.2).
const RECENT_AUTH: Duration = Duration::minutes(10);
const HOUR_SECS: u64 = 3600;
const EMAIL_PER_ADDRESS_PER_HOUR: u32 = 5;
const EMAIL_PER_IP_PER_HOUR: u32 = 20;

type Svc = Extension<Arc<AuthServices>>;

// --- bodies --------------------------------------------------------------------------------

/// The kind of client signing in; decides how the session travels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClientParam {
    /// The iOS app: the token comes back in the body and goes in `Authorization: Bearer`.
    Ios,
    /// A browser: the token comes back in the `__Host-session` cookie.
    #[default]
    Web,
}

impl From<ClientParam> for ClientKind {
    fn from(c: ClientParam) -> Self {
        match c {
            ClientParam::Ios => Self::Ios,
            ClientParam::Web => Self::Web,
        }
    }
}

/// Response of `POST /v1/auth/nonce`.
#[derive(Debug, Serialize, ToSchema)]
pub struct NonceResponse {
    /// Single-use random value to hand to the provider (Apple: its SHA-256 in hex).
    pub nonce: String,
    /// When it stops being accepted.
    pub expires_at: DateTime<Utc>,
}

/// Body of `POST /v1/auth/apple` and `POST /v1/auth/google`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ProviderLoginRequest {
    /// The provider's ID token (JWT).
    pub id_token: String,
    /// The raw nonce issued by `POST /v1/auth/nonce`.
    pub nonce: String,
    /// Client kind (default `web`).
    #[serde(default)]
    pub client: ClientParam,
    /// Web only: keep the session across browser restarts.
    #[serde(default)]
    pub remember: bool,
    /// Name shown in the session list (at most 100 characters).
    #[serde(default)]
    pub device_name: Option<String>,
    /// Name from the first Apple authorization, which Apple sends only to the app.
    #[serde(default)]
    pub display_name: Option<String>,
}

/// Body of `POST /v1/auth/email/start`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct EmailStartRequest {
    /// The address to send the link to.
    pub email: String,
    /// Which client asks (for the mail text).
    #[serde(default)]
    pub client: ClientParam,
}

/// Response of `POST /v1/auth/email/start`: always the same, whatever the address.
#[derive(Debug, Serialize, ToSchema)]
pub struct EmailStartResponse {
    /// Seconds the link stays valid.
    pub expires_in: i64,
}

/// Body of `POST /v1/auth/email/verify`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct EmailVerifyRequest {
    /// The token from the mailed link.
    pub token: String,
    /// Client kind (default `web`).
    #[serde(default)]
    pub client: ClientParam,
    /// Web only: keep the session across browser restarts.
    #[serde(default)]
    pub remember: bool,
    /// Name shown in the session list.
    #[serde(default)]
    pub device_name: Option<String>,
}

/// The signed-in user.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UserSummary {
    /// User id.
    pub id: Uuid,
    /// Display name (may be empty).
    pub display_name: String,
    /// Verified contact address, if any.
    pub contact_email: Option<String>,
}

/// Sign-in methods of other accounts that use the same verified address (ADR-0009 rule 1.6).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AccountHint {
    /// Providers (`apple`, `google`, `email`) the other account signs in with.
    pub providers: Vec<String>,
}

/// A session that was just issued.
#[derive(Debug, Serialize, ToSchema)]
pub struct SessionIssued {
    /// Session id.
    pub id: Uuid,
    /// Client kind.
    pub client: String,
    /// The bearer token, for `ios` sessions. Shown once. Web sessions use the cookie.
    pub token: Option<String>,
    /// CSRF token to send as `X-CSRF-Token` on writes, for `web` sessions.
    pub csrf_token: Option<String>,
    /// When the session ends at the latest.
    pub expires_at: DateTime<Utc>,
}

/// Response of the sign-in endpoints.
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    /// The user.
    pub user: UserSummary,
    /// The account was created by this sign-in; the client continues with workspace setup.
    pub is_new_user: bool,
    /// Present when the verified address belongs to another account (no automatic merge).
    pub account_hint: Option<AccountHint>,
    /// The new session.
    pub session: SessionIssued,
}

/// A workspace the user belongs to.
#[derive(Debug, Serialize, ToSchema)]
pub struct WorkspaceSummary {
    /// Workspace id.
    pub id: Uuid,
    /// Name.
    pub name: String,
    /// `home` or `business`.
    pub mode: String,
    /// `us` or `uk`.
    pub region: String,
    /// Membership status (`active` or `disabled`).
    pub status: String,
}

/// Response of `GET /v1/me/session`.
#[derive(Debug, Serialize, ToSchema)]
pub struct SessionInfo {
    /// The current session.
    pub session: SessionDetail,
    /// The user.
    pub user: UserSummary,
    /// Workspaces of the user (none yet right after the first sign-in).
    pub workspaces: Vec<WorkspaceSummary>,
}

/// Details of a session.
#[derive(Debug, Serialize, ToSchema)]
pub struct SessionDetail {
    /// Session id.
    pub id: Uuid,
    /// Client kind: `ios`, `web` or `shared_device`.
    pub client: String,
    /// Sign-in time.
    pub created_at: DateTime<Utc>,
    /// Absolute end.
    pub expires_at: DateTime<Utc>,
    /// Time of the last second factor, if any.
    pub step_up_at: Option<DateTime<Utc>>,
    /// CSRF token for cookie sessions.
    pub csrf_token: Option<String>,
}

/// One entry of the session list.
#[derive(Debug, Serialize, ToSchema)]
pub struct SessionItem {
    /// Session id.
    pub id: Uuid,
    /// Client kind.
    pub client: String,
    /// Device name given at sign-in.
    pub device_name: String,
    /// Approximate location (country), if known.
    pub approx_location: Option<String>,
    /// Sign-in time.
    pub created_at: DateTime<Utc>,
    /// Last activity.
    pub last_activity_at: DateTime<Utc>,
    /// Absolute end.
    pub expires_at: DateTime<Utc>,
    /// This is the session of the request.
    pub current: bool,
}

/// Body of `POST /v1/me/identities`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct LinkIdentityRequest {
    /// `apple`, `google` or `email`.
    pub provider: String,
    /// Apple and Google: the fresh ID token.
    #[serde(default)]
    pub id_token: Option<String>,
    /// Apple and Google: the raw nonce from `POST /v1/auth/nonce`.
    #[serde(default)]
    pub nonce: Option<String>,
    /// E-mail: the token from the mailed link.
    #[serde(default)]
    pub token: Option<String>,
}

/// A sign-in method of the user.
#[derive(Debug, Serialize, ToSchema)]
pub struct IdentitySummary {
    /// Identity id.
    pub id: Uuid,
    /// `apple`, `google` or `email`.
    pub provider: String,
    /// The address, for `email` identities.
    pub label: Option<String>,
    /// When it was linked.
    pub linked_at: DateTime<Utc>,
}

/// Response of `POST /v1/me/identities`: the method and the replacement session (the old
/// token stops working; ADR-0009 rule 3.4).
#[derive(Debug, Serialize, ToSchema)]
pub struct LinkResponse {
    /// The linked method.
    pub identity: IdentitySummary,
    /// The new session.
    pub session: SessionIssued,
}

// --- helpers -------------------------------------------------------------------------------

fn oidc_error(e: OidcError) -> ApiError {
    if e == OidcError::JwksUnavailable {
        return ApiError::unavailable(30);
    }
    ApiError::new(ErrorCode::Unauthenticated)
        .with_errors(vec![FieldIssue::new("/id_token", e.code())])
}

fn invalid(field: &str, code: &str) -> ApiError {
    ApiError::validation(vec![FieldIssue::new(field, code)])
}

fn is_unique_violation(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .is_some_and(|c| c == "23505")
}

fn country_of(headers: &HeaderMap) -> Option<String> {
    headers
        .get("cf-ipcountry")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|c| c.len() == 2 && c.bytes().all(|b| b.is_ascii_uppercase()))
        .map(str::to_owned)
}

fn clean_device_name(raw: Option<&str>) -> ApiResult<String> {
    let name: String = raw
        .unwrap_or_default()
        .chars()
        .filter(|c| !c.is_control())
        .collect();
    let name = name.trim().to_owned();
    if name.chars().count() > 100 {
        return Err(invalid("/device_name", "too_long"));
    }
    Ok(name)
}

fn clean_display_name(raw: Option<&str>) -> String {
    let name: String = raw
        .unwrap_or_default()
        .chars()
        .filter(|c| !c.is_control())
        .collect();
    name.trim().chars().take(200).collect()
}

fn problem(code: &str, field: &str) -> ApiError {
    ApiError::new(ErrorCode::Unauthenticated).with_errors(vec![FieldIssue::new(field, code)])
}

/// Who signs in, as the provider (or the e-mail token) states it.
#[derive(Debug, Clone)]
pub(crate) struct IdentityClaim {
    provider: &'static str,
    subject: String,
    email: Option<String>,
    email_verified: bool,
    display_name: String,
}

/// The device a session is created for.
#[derive(Debug, Clone)]
pub(crate) struct Device {
    kind: ClientKind,
    remember: bool,
    name: String,
    country: Option<String>,
}

struct LoginOutcome {
    user: UserSummary,
    is_new_user: bool,
    hint: Vec<String>,
    token: String,
    record: SessionRecord,
}

impl AuthServices {
    pub(super) fn issued(&self, record: &SessionRecord, token: Option<String>) -> SessionIssued {
        let web = record.client_kind == ClientKind::Web;
        SessionIssued {
            id: record.id,
            client: record.client_kind.as_str().to_owned(),
            token: if web { None } else { token },
            csrf_token: web.then(|| self.csrf_token(record.id)),
            expires_at: record.expires_at,
        }
    }

    /// `Set-Cookie` for a web session; `None` for the iOS app.
    pub(super) fn cookie_for(record: &SessionRecord, token: &str) -> Option<String> {
        (record.client_kind == ClientKind::Web).then(|| {
            let max_age = record
                .remember
                .then(|| (record.expires_at - Utc::now()).num_seconds().max(0));
            Self::session_cookie(token, max_age)
        })
    }

    async fn consume_nonce(&self, raw: &str) -> ApiResult<bool> {
        if raw.is_empty() || raw.len() > 128 {
            return Ok(false);
        }
        let row: Option<(String,)> =
            sqlx::query_as("SELECT nonce FROM auth_request_consume($1, NULL)")
                .bind(raw)
                .fetch_optional(&self.db)
                .await?;
        Ok(row.is_some())
    }

    /// Verifies an Apple or Google token and the single-use nonce.
    async fn verify_provider(
        &self,
        provider: &'static str,
        id_token: &str,
        nonce: &str,
    ) -> ApiResult<IdentityClaim> {
        let verifier: Option<&OidcVerifier> = match provider {
            "apple" => self.apple.as_ref(),
            _ => self.google.as_ref(),
        };
        let verifier = verifier.ok_or_else(|| ApiError::new(ErrorCode::NotFound))?;
        let claims = verifier.verify(id_token).await.map_err(oidc_error)?;
        // The nonce row is used up whatever the outcome, so a token cannot be replayed.
        let known = self.consume_nonce(nonce).await?;
        // Apple copies the nonce the client gave it, which is the SHA-256 (hex) of the raw one.
        let expected = if provider == "apple" {
            hex(&Sha256::digest(nonce.as_bytes()))
        } else {
            nonce.to_owned()
        };
        if !known || claims.nonce.as_deref() != Some(expected.as_str()) {
            return Err(problem("nonce_invalid", "/nonce"));
        }
        let email = claims.email.as_deref().and_then(normalize_email);
        Ok(IdentityClaim {
            provider,
            subject: claims.sub,
            email_verified: claims.email_verified && email.is_some(),
            email,
            display_name: claims.name.unwrap_or_default(),
        })
    }

    /// Consumes an e-mail link token (single use, 15 minutes) into an identity claim.
    async fn verify_email_token(&self, token: &str) -> ApiResult<IdentityClaim> {
        if token.is_empty() || token.len() > 128 {
            return Err(problem("invalid_or_expired", "/token"));
        }
        let row: Option<(String,)> =
            sqlx::query_as("SELECT email FROM email_login_token_consume($1)")
                .bind(token_hash(token))
                .fetch_optional(&self.db)
                .await?;
        let (email,) = row.ok_or_else(|| problem("invalid_or_expired", "/token"))?;
        Ok(IdentityClaim {
            provider: "email",
            subject: email.clone(),
            email: Some(email),
            email_verified: true,
            display_name: String::new(),
        })
    }

    async fn login(&self, claim: &IdentityClaim, device: &Device) -> ApiResult<LoginOutcome> {
        for _ in 0..2 {
            let existing: Option<(Uuid,)> =
                sqlx::query_as("SELECT user_id FROM find_user_by_identity($1, $2)")
                    .bind(claim.provider)
                    .bind(&claim.subject)
                    .fetch_optional(&self.db)
                    .await?;
            if let Some((user_id,)) = existing {
                return self.login_existing(user_id, device).await;
            }
            if let Some(outcome) = self.create_user(claim, device).await? {
                return Ok(outcome);
            }
            // Lost a race with a parallel first sign-in of the same identity: look it up again.
        }
        Err(ApiError::new(ErrorCode::Internal))
    }

    async fn login_existing(&self, user_id: Uuid, device: &Device) -> ApiResult<LoginOutcome> {
        let mut tx = db::begin(&self.db, &db::DbContext::user(user_id, None)).await?;
        let user: Option<(Uuid, String, Option<String>)> = sqlx::query_as(
            "SELECT id, display_name, contact_email FROM users WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?;
        let (id, display_name, contact_email) =
            user.ok_or_else(|| ApiError::new(ErrorCode::Unauthenticated))?;
        let (token, record) = session::insert(
            &mut tx,
            user_id,
            device.kind,
            device.remember,
            &device.name,
            device.country.as_deref(),
        )
        .await?;
        tx.commit().await?;
        Ok(LoginOutcome {
            user: UserSummary {
                id,
                display_name,
                contact_email,
            },
            is_new_user: false,
            hint: Vec::new(),
            token,
            record,
        })
    }

    /// Creates user, identity and session in one transaction. `None` when the identity appeared
    /// meanwhile (unique violation).
    async fn create_user(
        &self,
        claim: &IdentityClaim,
        device: &Device,
    ) -> ApiResult<Option<LoginOutcome>> {
        let hint: Vec<String> = match (&claim.email, claim.email_verified) {
            (Some(email), true) => {
                let rows: Vec<(String,)> = sqlx::query_as(
                    "SELECT DISTINCT provider FROM find_accounts_by_email($1) ORDER BY provider",
                )
                .bind(email)
                .fetch_all(&self.db)
                .await?;
                rows.into_iter().map(|r| r.0).collect()
            }
            _ => Vec::new(),
        };
        let user_id = Uuid::now_v7();
        let contact_email = claim.email.clone().filter(|_| claim.email_verified);
        let display_name = clean_display_name(Some(&claim.display_name));
        let mut tx = db::begin(&self.db, &db::DbContext::user(user_id, None)).await?;
        let inserted = async {
            sqlx::query(
                "INSERT INTO users (id, display_name, contact_email, contact_email_verified_at) \
                 VALUES ($1, $2, $3, CASE WHEN $3::text IS NULL THEN NULL ELSE now() END)",
            )
            .bind(user_id)
            .bind(&display_name)
            .bind(&contact_email)
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                "INSERT INTO user_identities (id, user_id, provider, subject) VALUES ($1, $2, $3, $4)",
            )
            .bind(Uuid::now_v7())
            .bind(user_id)
            .bind(claim.provider)
            .bind(&claim.subject)
            .execute(&mut *tx)
            .await?;
            session::insert(
                &mut tx,
                user_id,
                device.kind,
                device.remember,
                &device.name,
                device.country.as_deref(),
            )
            .await
        }
        .await;
        match inserted {
            Ok((token, record)) => {
                tx.commit().await?;
                Ok(Some(LoginOutcome {
                    user: UserSummary {
                        id: user_id,
                        display_name,
                        contact_email,
                    },
                    is_new_user: true,
                    hint,
                    token,
                    record,
                }))
            }
            Err(e) if is_unique_violation(&e) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn login_response(&self, o: LoginOutcome) -> Response {
        let cookie = Self::cookie_for(&o.record, &o.token);
        let body = LoginResponse {
            user: o.user,
            is_new_user: o.is_new_user,
            account_hint: (!o.hint.is_empty()).then_some(AccountHint { providers: o.hint }),
            session: self.issued(&o.record, Some(o.token)),
        };
        with_cookie(Json(body).into_response(), cookie.as_deref())
    }
}

pub(super) fn with_cookie(mut response: Response, cookie: Option<&str>) -> Response {
    let h = response.headers_mut();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    if let Some(cookie) = cookie.and_then(|c| HeaderValue::from_str(c).ok()) {
        h.append(header::SET_COOKIE, cookie);
    }
    response
}

fn device_of(
    client: ClientParam,
    remember: bool,
    device_name: Option<&str>,
    headers: &HeaderMap,
) -> ApiResult<Device> {
    Ok(Device {
        kind: client.into(),
        remember: remember && client == ClientParam::Web,
        name: clean_device_name(device_name)?,
        country: country_of(headers),
    })
}

// --- sign-in endpoints ---------------------------------------------------------------------

/// Opens a sign-in attempt: a single-use nonce (ADR-0009 rule 1.2).
#[utoipa::path(
    post,
    path = "/v1/auth/nonce",
    tag = "auth",
    operation_id = "auth_nonce_create",
    responses((
        status = 201,
        description = "A nonce valid for 10 minutes",
        body = NonceResponse,
        example = json!({"nonce": "q8o0m2s1V6mQ0fQmB1x8m7Tt0cV3oO6bV4k0fF0yJ1A", "expires_at": "2026-11-13T10:10:00Z"})
    ))
)]
pub async fn nonce_create(Extension(svc): Svc) -> ApiResult<(StatusCode, Json<NonceResponse>)> {
    let nonce = new_token();
    let expires_at = Utc::now() + NONCE_TTL;
    sqlx::query("SELECT auth_request_create($1, $2, NULL, NULL, $3)")
        .bind(Uuid::now_v7())
        .bind(&nonce)
        .bind(expires_at)
        .execute(&svc.db)
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(NonceResponse { nonce, expires_at }),
    ))
}

async fn provider_login(
    svc: &Arc<AuthServices>,
    provider: &'static str,
    req: ProviderLoginRequest,
    headers: &HeaderMap,
) -> ApiResult<Response> {
    let device = device_of(
        req.client,
        req.remember,
        req.device_name.as_deref(),
        headers,
    )?;
    let mut claim = svc
        .verify_provider(provider, &req.id_token, &req.nonce)
        .await?;
    if claim.display_name.is_empty() {
        claim.display_name = req.display_name.unwrap_or_default();
    }
    Ok(svc.login_response(svc.login(&claim, &device).await?))
}

/// Signs in with an Apple ID token.
#[utoipa::path(
    post,
    path = "/v1/auth/apple",
    tag = "auth",
    operation_id = "auth_apple_login",
    request_body = ProviderLoginRequest,
    responses((
        status = 200,
        description = "Signed in; iOS gets the token in the body, web gets the cookie",
        body = LoginResponse,
        example = json!({
            "user": {"id": "0192f1c0-7a3b-7000-8000-00000000000a", "display_name": "", "contact_email": null},
            "is_new_user": true,
            "account_hint": null,
            "session": {"id": "0192f1c0-7a3b-7000-8000-00000000000b", "client": "ios",
                        "token": "Zk3x9rR0m1c0n0pq0V4m7Xb1u2y8wQ5n6J7h8G9f0Aa",
                        "csrf_token": null, "expires_at": "2027-02-11T10:00:00Z"}
        })
    ))
)]
pub async fn apple_login(
    Extension(svc): Svc,
    headers: HeaderMap,
    ApiJson(req): ApiJson<ProviderLoginRequest>,
) -> ApiResult<Response> {
    provider_login(&svc, "apple", req, &headers).await
}

/// Signs in with a Google ID token.
#[utoipa::path(
    post,
    path = "/v1/auth/google",
    tag = "auth",
    operation_id = "auth_google_login",
    request_body = ProviderLoginRequest,
    responses((
        status = 200,
        description = "Signed in; iOS gets the token in the body, web gets the cookie",
        body = LoginResponse,
        example = json!({
            "user": {"id": "0192f1c0-7a3b-7000-8000-00000000000a", "display_name": "Sam Brewer", "contact_email": "sam@example.com"},
            "is_new_user": false,
            "account_hint": null,
            "session": {"id": "0192f1c0-7a3b-7000-8000-00000000000b", "client": "web", "token": null,
                        "csrf_token": "n0m7k4V0s2k6e1x9Q8d3b5aA7cC0uUu1wWw2yYy3zZz",
                        "expires_at": "2026-11-13T22:00:00Z"}
        })
    ))
)]
pub async fn google_login(
    Extension(svc): Svc,
    headers: HeaderMap,
    ApiJson(req): ApiJson<ProviderLoginRequest>,
) -> ApiResult<Response> {
    provider_login(&svc, "google", req, &headers).await
}

/// Mails a sign-in link. Always `202`; whether the address is known never shows.
#[utoipa::path(
    post,
    path = "/v1/auth/email/start",
    tag = "auth",
    operation_id = "auth_email_start",
    request_body = EmailStartRequest,
    responses((
        status = 202,
        description = "Accepted (a mail is sent when the address is valid)",
        body = EmailStartResponse,
        example = json!({"expires_in": 900})
    ))
)]
pub async fn email_start(
    Extension(svc): Svc,
    State(state): State<AppState>,
    headers: HeaderMap,
    connect: Option<Extension<ConnectInfo<SocketAddr>>>,
    ApiJson(req): ApiJson<EmailStartRequest>,
) -> ApiResult<(StatusCode, Json<EmailStartResponse>)> {
    let email = normalize_email(&req.email).ok_or_else(|| invalid("/email", "invalid_email"))?;
    let ip = client_ip_of(
        &headers,
        connect.map(|c| (c.0).0),
        state.rate.config().trust_forwarded_for,
    );
    state
        .rate
        .check_window("email_ip", &ip, EMAIL_PER_IP_PER_HOUR, HOUR_SECS)
        .await?;
    let address_key = hex(&Sha256::digest(email.as_bytes()));
    state
        .rate
        .check_window(
            "email_addr",
            &address_key,
            EMAIL_PER_ADDRESS_PER_HOUR,
            HOUR_SECS,
        )
        .await?;

    let token = new_token();
    sqlx::query("SELECT email_login_token_create($1, $2, $3, $4)")
        .bind(Uuid::now_v7())
        .bind(token_hash(&token))
        .bind(&email)
        .bind(Utc::now() + EMAIL_LINK_TTL)
        .execute(&svc.db)
        .await?;
    let link = format!("{}/auth/email?token={token}", svc.config.web_base_url);
    let client = match req.client {
        ClientParam::Ios => "iOS",
        ClientParam::Web => "web browser",
    };
    let mail = sign_in_mail(&email, &link, client, country_of(&headers).as_deref());
    if let Err(e) = svc.mailer.send(&mail).await {
        // The caller must not learn anything either way; the failure is for the operators.
        tracing::error!(error = %e, "sign-in mail not sent");
    }
    Ok((
        StatusCode::ACCEPTED,
        Json(EmailStartResponse {
            expires_in: EMAIL_LINK_TTL.num_seconds(),
        }),
    ))
}

/// Exchanges the mailed token for a session. The token works once.
#[utoipa::path(
    post,
    path = "/v1/auth/email/verify",
    tag = "auth",
    operation_id = "auth_email_verify",
    request_body = EmailVerifyRequest,
    responses((
        status = 200,
        description = "Signed in",
        body = LoginResponse,
        example = json!({
            "user": {"id": "0192f1c0-7a3b-7000-8000-00000000000a", "display_name": "", "contact_email": "sam@example.com"},
            "is_new_user": true,
            "account_hint": {"providers": ["apple"]},
            "session": {"id": "0192f1c0-7a3b-7000-8000-00000000000b", "client": "ios",
                        "token": "Zk3x9rR0m1c0n0pq0V4m7Xb1u2y8wQ5n6J7h8G9f0Aa",
                        "csrf_token": null, "expires_at": "2027-02-11T10:00:00Z"}
        })
    ))
)]
pub async fn email_verify(
    Extension(svc): Svc,
    headers: HeaderMap,
    ApiJson(req): ApiJson<EmailVerifyRequest>,
) -> ApiResult<Response> {
    let device = device_of(
        req.client,
        req.remember,
        req.device_name.as_deref(),
        &headers,
    )?;
    let claim = svc.verify_email_token(&req.token).await?;
    Ok(svc.login_response(svc.login(&claim, &device).await?))
}

// --- session endpoints ---------------------------------------------------------------------

fn session_id_of(ctx: &RequestContext) -> ApiResult<Uuid> {
    ctx.identity
        .session_id
        .ok_or_else(|| ApiError::new(ErrorCode::Unauthenticated))
}

pub(super) async fn current_record(
    svc: &AuthServices,
    ctx: &RequestContext,
) -> ApiResult<SessionRecord> {
    svc.sessions
        .get(ctx.identity.user_id, session_id_of(ctx)?)
        .await?
        .ok_or_else(|| ApiError::new(ErrorCode::Unauthenticated))
}

/// The current session, the user and the user's workspaces.
#[utoipa::path(
    get,
    path = "/v1/me/session",
    tag = "me",
    operation_id = "me_session_get",
    responses((
        status = 200,
        description = "Current session",
        body = SessionInfo,
        example = json!({
            "session": {"id": "0192f1c0-7a3b-7000-8000-00000000000b", "client": "web",
                        "created_at": "2026-11-13T10:00:00Z", "expires_at": "2026-11-13T22:00:00Z",
                        "step_up_at": null, "csrf_token": "n0m7k4V0s2k6e1x9Q8d3b5aA7cC0uUu1wWw2yYy3zZz"},
            "user": {"id": "0192f1c0-7a3b-7000-8000-00000000000a", "display_name": "Sam Brewer", "contact_email": "sam@example.com"},
            "workspaces": [{"id": "0192f1c0-7a3b-7000-8000-00000000000c", "name": "Home brewery",
                            "mode": "home", "region": "us", "status": "active"}]
        })
    ))
)]
pub async fn session_get(Extension(svc): Svc, ctx: RequestContext) -> ApiResult<Json<SessionInfo>> {
    let record = current_record(&svc, &ctx).await?;
    let user_id = ctx.identity.user_id;
    let mut tx = db::begin(&svc.db, &db::DbContext::user(user_id, None)).await?;
    let user: Option<(Uuid, String, Option<String>)> = sqlx::query_as(
        "SELECT id, display_name, contact_email FROM users WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;
    let memberships: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT workspace_id, status FROM workspace_members \
         WHERE user_id = $1 AND deleted_at IS NULL ORDER BY joined_at, id LIMIT 50",
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    let (id, display_name, contact_email) =
        user.ok_or_else(|| ApiError::new(ErrorCode::Unauthenticated))?;

    let mut workspaces = Vec::new();
    for (workspace_id, status) in memberships {
        let mut tx = db::begin(&svc.db, &db::DbContext::user(user_id, Some(workspace_id))).await?;
        let row: Option<(String, String, String)> = sqlx::query_as(
            "SELECT name, mode, region FROM workspaces WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(workspace_id)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        if let Some((name, mode, region)) = row {
            workspaces.push(WorkspaceSummary {
                id: workspace_id,
                name,
                mode,
                region,
                status,
            });
        }
    }
    let csrf_token = (record.client_kind == ClientKind::Web).then(|| svc.csrf_token(record.id));
    Ok(Json(SessionInfo {
        session: SessionDetail {
            id: record.id,
            client: record.client_kind.as_str().to_owned(),
            created_at: record.created_at,
            expires_at: record.expires_at,
            step_up_at: record.step_up_at,
            csrf_token,
        },
        user: UserSummary {
            id,
            display_name,
            contact_email,
        },
        workspaces,
    }))
}

fn closed(clear_cookie: bool) -> Response {
    with_cookie(
        StatusCode::NO_CONTENT.into_response(),
        clear_cookie.then(AuthServices::clear_cookie).as_deref(),
    )
}

/// Signs out of the current session.
#[utoipa::path(
    delete,
    path = "/v1/me/session",
    tag = "me",
    operation_id = "me_session_delete",
    responses((status = 204, description = "Signed out"))
)]
pub async fn session_delete(Extension(svc): Svc, ctx: RequestContext) -> ApiResult<Response> {
    svc.sessions
        .revoke(ctx.identity.user_id, session_id_of(&ctx)?)
        .await?;
    tracing::info!(user_id = %ctx.identity.user_id, "session closed (sign-out)");
    Ok(closed(true))
}

/// The user's open sessions.
#[utoipa::path(
    get,
    path = "/v1/me/sessions",
    tag = "me",
    operation_id = "me_sessions_list",
    responses((
        status = 200,
        description = "Open sessions, newest first",
        body = Page<SessionItem>,
        example = json!({"items": [{
            "id": "0192f1c0-7a3b-7000-8000-00000000000b", "client": "ios", "device_name": "Sam's iPad",
            "approx_location": "US", "created_at": "2026-11-13T10:00:00Z",
            "last_activity_at": "2026-11-13T10:05:00Z", "expires_at": "2027-02-11T10:00:00Z", "current": true
        }], "next_cursor": null})
    ))
)]
pub async fn sessions_list(
    Extension(svc): Svc,
    ctx: RequestContext,
) -> ApiResult<Json<Page<SessionItem>>> {
    let current = session_id_of(&ctx)?;
    let items = svc
        .sessions
        .list(ctx.identity.user_id, Utc::now())
        .await?
        .into_iter()
        .map(|i| SessionItem {
            id: i.record.id,
            client: i.record.client_kind.as_str().to_owned(),
            device_name: i.device_name,
            approx_location: i.approx_location,
            created_at: i.record.created_at,
            last_activity_at: i.record.last_activity_at,
            expires_at: i.record.expires_at,
            current: i.record.id == current,
        })
        .collect();
    Ok(Json(Page {
        items,
        next_cursor: None,
    }))
}

/// Signs out everywhere, this session included.
#[utoipa::path(
    delete,
    path = "/v1/me/sessions",
    tag = "me",
    operation_id = "me_sessions_revoke_all",
    responses((status = 204, description = "Every session closed"))
)]
pub async fn sessions_revoke_all(Extension(svc): Svc, ctx: RequestContext) -> ApiResult<Response> {
    let n = svc.sessions.revoke_all(ctx.identity.user_id, None).await?;
    tracing::info!(user_id = %ctx.identity.user_id, closed = n, "all sessions closed");
    Ok(closed(true))
}

/// Closes one of the user's sessions.
#[utoipa::path(
    delete,
    path = "/v1/me/sessions/{id}",
    tag = "me",
    operation_id = "me_session_revoke",
    params(("id" = Uuid, Path, description = "Session id")),
    responses((status = 204, description = "Session closed"))
)]
pub async fn session_revoke(
    Extension(svc): Svc,
    ctx: RequestContext,
    Path(id): Path<Uuid>,
) -> ApiResult<Response> {
    if !svc.sessions.revoke(ctx.identity.user_id, id).await? {
        return Err(ApiError::new(ErrorCode::NotFound));
    }
    tracing::info!(user_id = %ctx.identity.user_id, session_id = %id, "session closed");
    Ok(closed(session_id_of(&ctx)? == id))
}

// --- identities ----------------------------------------------------------------------------

#[derive(sqlx::FromRow)]
struct IdentityRow {
    id: Uuid,
    provider: String,
    subject: String,
    linked_at: DateTime<Utc>,
}

impl From<IdentityRow> for IdentitySummary {
    fn from(r: IdentityRow) -> Self {
        Self {
            id: r.id,
            label: (r.provider == "email").then_some(r.subject),
            provider: r.provider,
            linked_at: r.linked_at,
        }
    }
}

/// The user's sign-in methods.
#[utoipa::path(
    get,
    path = "/v1/me/identities",
    tag = "me",
    operation_id = "me_identities_list",
    responses((
        status = 200,
        description = "Linked sign-in methods",
        body = Page<IdentitySummary>,
        example = json!({"items": [{"id": "0192f1c0-7a3b-7000-8000-00000000000d", "provider": "apple",
                                    "label": null, "linked_at": "2026-11-13T10:00:00Z"}], "next_cursor": null})
    ))
)]
pub async fn identities_list(
    Extension(svc): Svc,
    ctx: RequestContext,
) -> ApiResult<Json<Page<IdentitySummary>>> {
    let mut tx = db::begin(&svc.db, &db::DbContext::user(ctx.identity.user_id, None)).await?;
    let rows: Vec<IdentityRow> = sqlx::query_as(
        "SELECT id, provider, subject, linked_at FROM user_identities ORDER BY linked_at, id",
    )
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(Page {
        items: rows.into_iter().map(Into::into).collect(),
        next_cursor: None,
    }))
}

fn identity_in_use() -> ApiError {
    ApiError::new(ErrorCode::Conflict)
        .with_errors(vec![FieldIssue::new("/identity", "identity_in_use")])
}

/// Links another sign-in method to the account. The session is replaced.
#[utoipa::path(
    post,
    path = "/v1/me/identities",
    tag = "me",
    operation_id = "me_identities_link",
    request_body = LinkIdentityRequest,
    responses((
        status = 200,
        description = "Linked; the response carries the replacement session",
        body = LinkResponse,
        example = json!({
            "identity": {"id": "0192f1c0-7a3b-7000-8000-00000000000d", "provider": "google",
                         "label": null, "linked_at": "2026-11-13T10:00:00Z"},
            "session": {"id": "0192f1c0-7a3b-7000-8000-00000000000e", "client": "ios",
                        "token": "Zk3x9rR0m1c0n0pq0V4m7Xb1u2y8wQ5n6J7h8G9f0Aa",
                        "csrf_token": null, "expires_at": "2027-02-11T10:00:00Z"}
        })
    ))
)]
pub async fn identity_link(
    Extension(svc): Svc,
    ctx: RequestContext,
    ApiJson(req): ApiJson<LinkIdentityRequest>,
) -> ApiResult<Response> {
    let claim = match req.provider.as_str() {
        p @ ("apple" | "google") => {
            let provider = if p == "apple" { "apple" } else { "google" };
            let (Some(id_token), Some(nonce)) = (&req.id_token, &req.nonce) else {
                return Err(invalid("/id_token", "required"));
            };
            svc.verify_provider(provider, id_token, nonce).await?
        }
        "email" => {
            let token = req
                .token
                .as_deref()
                .ok_or_else(|| invalid("/token", "required"))?;
            svc.verify_email_token(token).await?
        }
        _ => return Err(invalid("/provider", "unsupported_provider")),
    };
    let user_id = ctx.identity.user_id;
    let existing: Option<(Uuid, Uuid)> =
        sqlx::query_as("SELECT user_id, identity_id FROM find_user_by_identity($1, $2)")
            .bind(claim.provider)
            .bind(&claim.subject)
            .fetch_optional(&svc.db)
            .await?;
    if existing.is_some_and(|(owner, _)| owner != user_id) {
        return Err(identity_in_use());
    }
    let mut tx = db::begin(&svc.db, &db::DbContext::user(user_id, None)).await?;
    if existing.is_none() {
        let inserted = sqlx::query(
            "INSERT INTO user_identities (id, user_id, provider, subject) VALUES ($1, $2, $3, $4)",
        )
        .bind(Uuid::now_v7())
        .bind(user_id)
        .bind(claim.provider)
        .bind(&claim.subject)
        .execute(&mut *tx)
        .await;
        match inserted {
            Ok(_) => {}
            Err(e) if is_unique_violation(&e) => return Err(identity_in_use()),
            Err(e) => return Err(e.into()),
        }
    }
    let row: IdentityRow = sqlx::query_as(
        "SELECT id, provider, subject, linked_at FROM user_identities WHERE provider = $1 AND subject = $2",
    )
    .bind(claim.provider)
    .bind(&claim.subject)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    tracing::info!(user_id = %user_id, provider = claim.provider, "identity linked");

    // A new token after linking (session fixation, ADR-0009 rule 3.4).
    let old = current_record(&svc, &ctx).await?;
    let (token, record) = svc.sessions.rotate(&old).await?;
    let cookie = AuthServices::cookie_for(&record, &token);
    let body = LinkResponse {
        identity: row.into(),
        session: svc.issued(&record, Some(token)),
    };
    Ok(with_cookie(Json(body).into_response(), cookie.as_deref()))
}

/// Removes a sign-in method. The last one stays; a sign-in in the last 10 minutes is needed.
#[utoipa::path(
    delete,
    path = "/v1/me/identities/{id}",
    tag = "me",
    operation_id = "me_identities_unlink",
    params(("id" = Uuid, Path, description = "Identity id")),
    responses((status = 204, description = "Method removed"))
)]
pub async fn identity_unlink(
    Extension(svc): Svc,
    ctx: RequestContext,
    Path(id): Path<Uuid>,
) -> ApiResult<Response> {
    let record = current_record(&svc, &ctx).await?;
    let fresh = Utc::now() - RECENT_AUTH;
    if record.created_at < fresh && record.step_up_at.is_none_or(|t| t < fresh) {
        return Err(ApiError::new(ErrorCode::StepUpRequired));
    }
    let mut tx = db::begin(&svc.db, &db::DbContext::user(ctx.identity.user_id, None)).await?;
    // Lock the user's identity rows so two parallel removals cannot both leave none.
    let ids: Vec<(Uuid,)> = sqlx::query_as("SELECT id FROM user_identities ORDER BY id FOR UPDATE")
        .fetch_all(&mut *tx)
        .await?;
    if !ids.iter().any(|r| r.0 == id) {
        return Err(ApiError::new(ErrorCode::NotFound));
    }
    if ids.len() <= 1 {
        return Err(ApiError::new(ErrorCode::Conflict)
            .with_errors(vec![FieldIssue::new("/id", "last_identity")]));
    }
    sqlx::query("DELETE FROM user_identities WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    tracing::info!(user_id = %ctx.identity.user_id, identity_id = %id, "identity removed");
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_name_is_cleaned_and_limited() {
        assert_eq!(clean_device_name(Some("  iPad\n ")).unwrap(), "iPad");
        assert!(clean_device_name(Some(&"x".repeat(101))).is_err());
        assert_eq!(clean_device_name(None).unwrap(), "");
    }

    #[test]
    fn country_comes_only_from_a_two_letter_header() {
        let mut h = HeaderMap::new();
        h.insert("cf-ipcountry", HeaderValue::from_static("US"));
        assert_eq!(country_of(&h).as_deref(), Some("US"));
        h.insert("cf-ipcountry", HeaderValue::from_static("usa"));
        assert_eq!(country_of(&h), None);
    }
}
