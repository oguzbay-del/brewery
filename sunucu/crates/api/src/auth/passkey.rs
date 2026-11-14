//! Passkeys (`WebAuthn`) and the second step (ADR-0009 rule 5; T2.12).
//!
//! Registration and verification run with `webauthn-rs`, always with user verification
//! required. The ceremony state lives in `webauthn_challenges` (single use, 5 minutes). A
//! passkey row keeps the serialized `webauthn_rs::prelude::Passkey` in `public_key` (it holds the
//! COSE key and flags the library needs for later checks); `credential_id` and `sign_count`
//! stay queryable columns.
//!
//! A successful verification issues a replacement session carrying `step_up_at` (rule 3.4).

use axum::{
    Extension, Json,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use utoipa::ToSchema;
use uuid::Uuid;
use webauthn_rs::prelude::{
    Passkey, PasskeyAuthentication, PasskeyRegistration, PublicKeyCredential,
    RegisterPublicKeyCredential, Url, Webauthn, WebauthnBuilder,
};

use super::{
    AuthServices,
    config::AuthConfig,
    handlers::{SessionIssued, current_record, with_cookie},
    session::ClientKind,
    step_up::ProtectedAction,
};
use crate::{
    context::RequestContext,
    error::{ApiError, ApiJson, ApiResult, ErrorCode, FieldIssue},
};

/// How long a ceremony may take.
const CHALLENGE_TTL: Duration = Duration::minutes(5);
/// Most passkeys one user may keep.
const MAX_PASSKEYS: i64 = 10;

type Svc = Extension<std::sync::Arc<AuthServices>>;

/// Builds the relying party from the configuration; `None` (and a warning) when the RP id or
/// an origin does not fit together, in which case the passkey endpoints answer `503`.
pub(crate) fn build_webauthn(config: &AuthConfig) -> Option<Webauthn> {
    let mut origins = config
        .webauthn_origins
        .iter()
        .filter_map(|o| Url::parse(o).ok());
    let first = origins.next()?;
    let mut builder = WebauthnBuilder::new(&config.webauthn_rp_id, &first)
        .ok()?
        .rp_name(&config.webauthn_rp_name);
    for origin in origins {
        builder = builder.append_allowed_origin(&origin);
    }
    match builder.build() {
        Ok(w) => Some(w),
        Err(e) => {
            tracing::warn!(error = %e, rp_id = %config.webauthn_rp_id, "WebAuthn is not usable with this RP id and origins");
            None
        }
    }
}

// --- bodies --------------------------------------------------------------------------------

/// A registered passkey.
#[derive(Debug, Serialize, ToSchema)]
pub struct PasskeySummary {
    /// Passkey id.
    pub id: Uuid,
    /// Name given at registration.
    pub name: String,
    /// Last successful use.
    pub last_used_at: Option<DateTime<Utc>>,
    /// Registration time.
    pub created_at: DateTime<Utc>,
}

/// A `WebAuthn` ceremony to run on the client.
#[derive(Debug, Serialize, ToSchema)]
pub struct CeremonyStart {
    /// Hand this back to the matching `finish` call.
    pub challenge_id: Uuid,
    /// `WebAuthn` options (`{"publicKey": {...}}`) for `navigator.credentials` or
    /// `ASAuthorizationPlatformPublicKeyCredentialProvider`.
    #[schema(value_type = Object)]
    pub options: serde_json::Value,
    /// When the challenge stops being accepted.
    pub expires_at: DateTime<Utc>,
}

/// Body of `POST /v1/me/passkeys/registration/finish`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegistrationFinish {
    /// From the start call.
    pub challenge_id: Uuid,
    /// The authenticator's answer (`PublicKeyCredential` as JSON).
    #[schema(value_type = Object)]
    pub credential: serde_json::Value,
    /// Name shown in the list (at most 100 characters).
    #[serde(default)]
    pub name: Option<String>,
}

/// Response of a successful registration.
#[derive(Debug, Serialize, ToSchema)]
pub struct RegistrationDone {
    /// The new passkey.
    pub passkey: PasskeySummary,
    /// Replacement session; registering counts as a second step.
    pub session: SessionIssued,
}

/// Body of `POST /v1/me/step-up/finish`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct StepUpFinish {
    /// From the start call.
    pub challenge_id: Uuid,
    /// The authenticator's answer (`PublicKeyCredential` as JSON).
    #[schema(value_type = Object)]
    pub credential: serde_json::Value,
}

/// Response of a successful second step.
#[derive(Debug, Serialize, ToSchema)]
pub struct StepUpDone {
    /// Time of the second step (valid for 15 minutes).
    pub step_up_at: DateTime<Utc>,
    /// Replacement session; the old token stops working.
    pub session: SessionIssued,
}

// --- helpers -------------------------------------------------------------------------------

fn invalid(field: &str, code: &str) -> ApiError {
    ApiError::validation(vec![FieldIssue::new(field, code)])
}

fn webauthn(svc: &AuthServices) -> ApiResult<&Webauthn> {
    svc.webauthn
        .as_ref()
        .ok_or_else(|| ApiError::unavailable(60))
}

fn not_for_device(ctx: &RequestContext) -> ApiResult<()> {
    if ctx.identity.actor_kind == db::ActorKind::Device {
        return Err(ApiError::new(ErrorCode::Forbidden)
            .with_errors(vec![FieldIssue::new("/", "shared_device_session")]));
    }
    Ok(())
}

async fn put_challenge(
    conn: &mut PgConnection,
    user_id: Uuid,
    kind: &str,
    state: serde_json::Value,
) -> sqlx::Result<(Uuid, DateTime<Utc>)> {
    let id = Uuid::now_v7();
    let expires_at = Utc::now() + CHALLENGE_TTL;
    sqlx::query("DELETE FROM webauthn_challenges WHERE expires_at < now()")
        .execute(&mut *conn)
        .await?;
    sqlx::query(
        "INSERT INTO webauthn_challenges (id, user_id, kind, state, expires_at) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(user_id)
    .bind(kind)
    .bind(state)
    .bind(expires_at)
    .execute(conn)
    .await?;
    Ok((id, expires_at))
}

/// Takes (and so uses up) a challenge of the user.
async fn take_challenge(
    conn: &mut PgConnection,
    id: Uuid,
    kind: &str,
) -> ApiResult<serde_json::Value> {
    let row: Option<(serde_json::Value,)> = sqlx::query_as(
        "DELETE FROM webauthn_challenges WHERE id = $1 AND kind = $2 AND expires_at > now() \
         RETURNING state",
    )
    .bind(id)
    .bind(kind)
    .fetch_optional(conn)
    .await?;
    row.map(|r| r.0)
        .ok_or_else(|| invalid("/challenge_id", "challenge_invalid"))
}

#[derive(sqlx::FromRow)]
struct PasskeyRow {
    id: Uuid,
    name: String,
    last_used_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<PasskeyRow> for PasskeySummary {
    fn from(r: PasskeyRow) -> Self {
        Self {
            id: r.id,
            name: r.name,
            last_used_at: r.last_used_at,
            created_at: r.created_at,
        }
    }
}

async fn load_passkeys(conn: &mut PgConnection) -> ApiResult<Vec<(Uuid, Passkey)>> {
    let rows: Vec<(Uuid, Vec<u8>)> =
        sqlx::query_as("SELECT id, public_key FROM passkeys ORDER BY created_at, id")
            .fetch_all(conn)
            .await?;
    let mut out = Vec::with_capacity(rows.len());
    for (id, blob) in rows {
        match serde_json::from_slice::<Passkey>(&blob) {
            Ok(p) => out.push((id, p)),
            Err(e) => tracing::error!(passkey_id = %id, error = %e, "unreadable passkey record"),
        }
    }
    Ok(out)
}

fn state_json<T: Serialize>(state: &T) -> ApiResult<serde_json::Value> {
    serde_json::to_value(state).map_err(|e| {
        tracing::error!(error = %e, "cannot serialise WebAuthn state");
        ApiError::new(ErrorCode::Internal)
    })
}

fn clean_name(raw: Option<&str>) -> ApiResult<String> {
    let name: String = raw
        .unwrap_or_default()
        .chars()
        .filter(|c| !c.is_control())
        .collect();
    let name = name.trim().to_owned();
    if name.chars().count() > 100 {
        return Err(invalid("/name", "too_long"));
    }
    Ok(name)
}

// --- endpoints -----------------------------------------------------------------------------

/// Lists the user's passkeys.
#[utoipa::path(
    get,
    path = "/v1/me/passkeys",
    tag = "me",
    operation_id = "me_passkeys_list",
    responses((
        status = 200,
        description = "Passkeys",
        body = Vec<PasskeySummary>,
        example = json!([{"id": "0192f1c0-7a3b-7000-8000-00000000001a", "name": "iPhone",
                          "last_used_at": null, "created_at": "2026-11-14T10:00:00Z"}])
    ))
)]
pub async fn passkeys_list(
    ctx: RequestContext,
    Extension(svc): Svc,
) -> ApiResult<Json<Vec<PasskeySummary>>> {
    let mut tx = db::begin(&svc.db, &db::DbContext::user(ctx.identity.user_id, None)).await?;
    let rows: Vec<PasskeyRow> = sqlx::query_as(
        "SELECT id, name, last_used_at, created_at FROM passkeys ORDER BY created_at, id",
    )
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

/// Starts a passkey registration. Needs the second step (a passkey check when the user
/// already has one, otherwise a sign-in in the last 10 minutes).
#[utoipa::path(
    post,
    path = "/v1/me/passkeys/registration/start",
    tag = "me",
    operation_id = "me_passkeys_registration_start",
    responses((
        status = 200,
        description = "Ceremony to run on the client",
        body = CeremonyStart,
        example = json!({"challenge_id": "0192f1c0-7a3b-7000-8000-00000000001b",
                         "options": {"publicKey": {"rp": {"id": "app.example.com", "name": "Demleme"}}},
                         "expires_at": "2026-11-14T10:05:00Z"})
    ))
)]
pub async fn registration_start(
    ctx: RequestContext,
    Extension(svc): Svc,
) -> ApiResult<Json<CeremonyStart>> {
    not_for_device(&ctx)?;
    let w = webauthn(&svc)?;
    svc.require_step_up(&ctx, ProtectedAction::PasskeyChange)
        .await?;
    let user_id = ctx.identity.user_id;
    let mut tx = db::begin(&svc.db, &db::DbContext::user(user_id, None)).await?;
    let (display_name, contact_email): (String, Option<String>) =
        sqlx::query_as("SELECT display_name, contact_email FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await?;
    let existing = load_passkeys(&mut tx).await?;
    if i64::try_from(existing.len()).unwrap_or(i64::MAX) >= MAX_PASSKEYS {
        return Err(ApiError::new(ErrorCode::Conflict)
            .with_errors(vec![FieldIssue::new("/", "passkey_limit")]));
    }
    let exclude = existing
        .iter()
        .map(|(_, p)| p.cred_id().clone())
        .collect::<Vec<_>>();
    let user_name = contact_email.unwrap_or_else(|| user_id.to_string());
    let shown = if display_name.is_empty() {
        user_name.clone()
    } else {
        display_name
    };
    let (options, state) = w
        .start_passkey_registration(user_id, &user_name, &shown, Some(exclude))
        .map_err(|e| {
            tracing::error!(error = %e, "passkey registration start failed");
            ApiError::new(ErrorCode::Internal)
        })?;
    let (challenge_id, expires_at) =
        put_challenge(&mut tx, user_id, "registration", state_json(&state)?).await?;
    tx.commit().await?;
    Ok(Json(CeremonyStart {
        challenge_id,
        options: state_json(&options)?,
        expires_at,
    }))
}

/// Finishes a passkey registration. The new session carries the second step.
#[utoipa::path(
    post,
    path = "/v1/me/passkeys/registration/finish",
    tag = "me",
    operation_id = "me_passkeys_registration_finish",
    request_body = RegistrationFinish,
    responses((
        status = 201,
        description = "Passkey registered",
        body = RegistrationDone,
        example = json!({"passkey": {"id": "0192f1c0-7a3b-7000-8000-00000000001a", "name": "iPhone",
                                      "last_used_at": null, "created_at": "2026-11-14T10:00:00Z"},
                         "session": {"id": "0192f1c0-7a3b-7000-8000-00000000001c", "client": "ios",
                                     "token": "opaque", "csrf_token": null,
                                     "expires_at": "2027-02-12T10:00:00Z"}})
    ))
)]
pub async fn registration_finish(
    ctx: RequestContext,
    Extension(svc): Svc,
    ApiJson(req): ApiJson<RegistrationFinish>,
) -> ApiResult<Response> {
    not_for_device(&ctx)?;
    let w = webauthn(&svc)?;
    let name = clean_name(req.name.as_deref())?;
    let user_id = ctx.identity.user_id;
    let credential: RegisterPublicKeyCredential = serde_json::from_value(req.credential)
        .map_err(|_| invalid("/credential", "credential_invalid"))?;
    let mut tx = db::begin(&svc.db, &db::DbContext::user(user_id, None)).await?;
    let state: PasskeyRegistration =
        serde_json::from_value(take_challenge(&mut tx, req.challenge_id, "registration").await?)
            .map_err(|_| invalid("/challenge_id", "challenge_invalid"))?;
    // The challenge is used up even when the check fails.
    let passkey = match w.finish_passkey_registration(&credential, &state) {
        Ok(p) => p,
        Err(e) => {
            tx.commit().await?;
            tracing::info!(user_id = %user_id, error = %e, "passkey registration refused");
            return Err(invalid("/credential", "registration_invalid"));
        }
    };
    let blob = serde_json::to_vec(&passkey).map_err(|_| ApiError::new(ErrorCode::Internal))?;
    let inserted: Result<PasskeyRow, sqlx::Error> = sqlx::query_as(
        "INSERT INTO passkeys (id, user_id, credential_id, public_key, name) \
         VALUES ($1, $2, $3, $4, $5) RETURNING id, name, last_used_at, created_at",
    )
    .bind(Uuid::now_v7())
    .bind(user_id)
    .bind(passkey.cred_id().as_ref() as &[u8])
    .bind(blob)
    .bind(&name)
    .fetch_one(&mut *tx)
    .await;
    let row = match inserted {
        Ok(r) => r,
        Err(e)
            if e.as_database_error()
                .and_then(sqlx::error::DatabaseError::code)
                .is_some_and(|c| c == "23505") =>
        {
            return Err(invalid("/credential", "credential_in_use"));
        }
        Err(e) => return Err(e.into()),
    };
    tx.commit().await?;
    tracing::info!(user_id = %user_id, "passkey registered");
    let (token, record, cookie) = svc.step_up_session(&ctx).await?;
    let body = RegistrationDone {
        passkey: row.into(),
        session: svc.issued(&record, Some(token)),
    };
    let mut response = with_cookie(Json(body).into_response(), cookie.as_deref());
    *response.status_mut() = StatusCode::CREATED;
    Ok(response)
}

/// Removes a passkey. Needs the second step.
#[utoipa::path(
    delete,
    path = "/v1/me/passkeys/{id}",
    tag = "me",
    operation_id = "me_passkeys_delete",
    params(("id" = Uuid, Path, description = "Passkey id")),
    responses((status = 204, description = "Passkey removed"))
)]
pub async fn passkey_delete(
    ctx: RequestContext,
    Extension(svc): Svc,
    Path(id): Path<Uuid>,
) -> ApiResult<Response> {
    not_for_device(&ctx)?;
    svc.require_step_up(&ctx, ProtectedAction::PasskeyChange)
        .await?;
    let mut tx = db::begin(&svc.db, &db::DbContext::user(ctx.identity.user_id, None)).await?;
    let done = sqlx::query("DELETE FROM passkeys WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    if done.rows_affected() == 0 {
        return Err(ApiError::new(ErrorCode::NotFound));
    }
    tracing::info!(user_id = %ctx.identity.user_id, passkey_id = %id, "passkey removed");
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Starts the second step with a passkey.
#[utoipa::path(
    post,
    path = "/v1/me/step-up/start",
    tag = "me",
    operation_id = "me_stepup_start",
    responses((
        status = 200,
        description = "Ceremony to run on the client",
        body = CeremonyStart,
        example = json!({"challenge_id": "0192f1c0-7a3b-7000-8000-00000000001d",
                         "options": {"publicKey": {"rpId": "app.example.com"}},
                         "expires_at": "2026-11-14T10:05:00Z"})
    ))
)]
pub async fn step_up_start(
    ctx: RequestContext,
    Extension(svc): Svc,
) -> ApiResult<Json<CeremonyStart>> {
    not_for_device(&ctx)?;
    let w = webauthn(&svc)?;
    let user_id = ctx.identity.user_id;
    let mut tx = db::begin(&svc.db, &db::DbContext::user(user_id, None)).await?;
    let keys: Vec<Passkey> = load_passkeys(&mut tx)
        .await?
        .into_iter()
        .map(|p| p.1)
        .collect();
    if keys.is_empty() {
        return Err(ApiError::new(ErrorCode::Conflict)
            .with_errors(vec![FieldIssue::new("/", "no_passkey")]));
    }
    let (options, state) = w.start_passkey_authentication(&keys).map_err(|e| {
        tracing::error!(error = %e, "passkey authentication start failed");
        ApiError::new(ErrorCode::Internal)
    })?;
    let (challenge_id, expires_at) =
        put_challenge(&mut tx, user_id, "authentication", state_json(&state)?).await?;
    tx.commit().await?;
    Ok(Json(CeremonyStart {
        challenge_id,
        options: state_json(&options)?,
        expires_at,
    }))
}

/// Finishes the second step: writes `step_up_at` and replaces the session token.
#[utoipa::path(
    post,
    path = "/v1/me/step-up/finish",
    tag = "me",
    operation_id = "me_stepup_finish",
    request_body = StepUpFinish,
    responses((
        status = 200,
        description = "Second step done",
        body = StepUpDone,
        example = json!({"step_up_at": "2026-11-14T10:01:00Z",
                         "session": {"id": "0192f1c0-7a3b-7000-8000-00000000001e", "client": "web",
                                     "token": null, "csrf_token": "csrf",
                                     "expires_at": "2026-11-14T22:01:00Z"}})
    ))
)]
pub async fn step_up_finish(
    ctx: RequestContext,
    Extension(svc): Svc,
    ApiJson(req): ApiJson<StepUpFinish>,
) -> ApiResult<Response> {
    not_for_device(&ctx)?;
    let w = webauthn(&svc)?;
    let user_id = ctx.identity.user_id;
    let credential: PublicKeyCredential = serde_json::from_value(req.credential)
        .map_err(|_| invalid("/credential", "credential_invalid"))?;
    let mut tx = db::begin(&svc.db, &db::DbContext::user(user_id, None)).await?;
    let state: PasskeyAuthentication =
        serde_json::from_value(take_challenge(&mut tx, req.challenge_id, "authentication").await?)
            .map_err(|_| invalid("/challenge_id", "challenge_invalid"))?;
    let result = match w.finish_passkey_authentication(&credential, &state) {
        Ok(r) => r,
        Err(e) => {
            tx.commit().await?;
            tracing::info!(user_id = %user_id, error = %e, "passkey check refused");
            return Err(invalid("/credential", "passkey_invalid"));
        }
    };
    let mut keys = load_passkeys(&mut tx).await?;
    let Some((row_id, key)) = keys
        .iter_mut()
        .find(|(_, k)| k.cred_id() == result.cred_id())
    else {
        tx.commit().await?;
        return Err(invalid("/credential", "passkey_invalid"));
    };
    key.update_credential(&result);
    let blob = serde_json::to_vec(key).map_err(|_| ApiError::new(ErrorCode::Internal))?;
    sqlx::query(
        "UPDATE passkeys SET public_key = $2, sign_count = $3, last_used_at = now() WHERE id = $1",
    )
    .bind(*row_id)
    .bind(blob)
    .bind(i64::from(result.counter()))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    let (token, record, cookie) = svc.step_up_session(&ctx).await?;
    let body = StepUpDone {
        step_up_at: record.step_up_at.unwrap_or_else(Utc::now),
        session: svc.issued(&record, Some(token)),
    };
    Ok(with_cookie(Json(body).into_response(), cookie.as_deref()))
}

impl AuthServices {
    /// Replaces the caller's session by one stamped with the second step.
    async fn step_up_session(
        &self,
        ctx: &RequestContext,
    ) -> ApiResult<(String, super::session::SessionRecord, Option<String>)> {
        let old = current_record(self, ctx).await?;
        if old.client_kind == ClientKind::SharedDevice {
            return Err(ApiError::new(ErrorCode::Forbidden)
                .with_errors(vec![FieldIssue::new("/", "shared_device_session")]));
        }
        let (token, record) = self.sessions.rotate_with(&old, Some(Utc::now())).await?;
        let cookie = Self::cookie_for(&record, &token);
        Ok((token, record, cookie))
    }
}
