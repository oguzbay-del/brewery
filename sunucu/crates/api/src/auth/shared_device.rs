//! Shared device mode (ADR-0009 rule 7; T2.12).
//!
//! - Administrators (`devices.manage`, second step for opening and key renewal) register and
//!   revoke devices under `/v1/workspaces/{ws}/shared-devices`.
//! - Members set their own six digit PIN (`PUT /v1/workspaces/{ws}/me/pin`); an administrator
//!   can clear one (`DELETE /v1/workspaces/{ws}/members/{member}/pin`).
//! - The device itself proves itself with its key in `X-Device-Key` and may only call the three
//!   `/v1/shared-devices/{id}/...` endpoints: list the tiles, switch user with a PIN, lock.
//!
//! A switch gives a short lived user session bound to the device (`sessions.shared_device_id`,
//! actor kind `device`) whose idle timeout is the device's; a new switch or a lock closes the
//! previous one. Five failed PINs in a row lock the member for 15 minutes; a device may try 20
//! times a minute.

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use utoipa::ToSchema;
use uuid::Uuid;

use super::{
    AuthServices,
    handlers::SessionIssued,
    pin::{
        DEVICE_ATTEMPTS_PER_MINUTE, LOCK_MINUTES, MAX_FAILED_ATTEMPTS, dummy_verify, hash_pin,
        is_valid_pin, verify_pin,
    },
    session::{ClientKind, SessionRecord, lifetimes},
    step_up::ProtectedAction,
    tokens::{new_token, token_hash},
};
use crate::{
    AppState,
    context::RequestContext,
    error::{ApiError, ApiJson, ApiResult, ErrorCode, FieldIssue},
};

/// Header carrying the device key.
pub const DEVICE_KEY_HEADER: &str = "x-device-key";
/// Permission that manages devices and PINs.
const MANAGE: &str = "devices.manage";

type Svc = Extension<std::sync::Arc<AuthServices>>;

// --- bodies --------------------------------------------------------------------------------

/// Body of `POST /v1/workspaces/{ws}/shared-devices`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SharedDeviceCreate {
    /// Device name (1 to 100 characters).
    pub name: String,
    /// The key the device generated (32 to 128 URL-safe base64 characters). Only its SHA-256
    /// digest is kept.
    pub device_key: String,
    /// Idle lock in minutes (1 to 60, default 5).
    #[serde(default)]
    pub idle_timeout_min: Option<i32>,
}

/// Body of `POST /v1/workspaces/{ws}/shared-devices/{id}/rotate-key`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct DeviceKeyRotate {
    /// The new key (same rules as at registration).
    pub device_key: String,
}

/// A shared device.
#[derive(Debug, Serialize, ToSchema)]
pub struct SharedDeviceSummary {
    /// Device id.
    pub id: Uuid,
    /// Name.
    pub name: String,
    /// Idle lock in minutes.
    pub idle_timeout_min: i32,
    /// The administrator who opened it.
    pub opened_by: Uuid,
    /// Opened at.
    pub created_at: DateTime<Utc>,
    /// Set when revoked.
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow)]
struct DeviceRow {
    id: Uuid,
    name: String,
    idle_timeout_min: i32,
    opened_by: Uuid,
    created_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
}

impl From<DeviceRow> for SharedDeviceSummary {
    fn from(r: DeviceRow) -> Self {
        Self {
            id: r.id,
            name: r.name,
            idle_timeout_min: r.idle_timeout_min,
            opened_by: r.opened_by,
            created_at: r.created_at,
            revoked_at: r.revoked_at,
        }
    }
}

const DEVICE_COLUMNS: &str = "id, name, idle_timeout_min, opened_by, created_at, revoked_at";

/// Body of `PUT /v1/workspaces/{ws}/me/pin`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PinSet {
    /// Six digits.
    pub pin: String,
}

/// A member tile on the device.
#[derive(Debug, Serialize, ToSchema)]
pub struct MemberTile {
    /// Member id (what `switch` takes).
    pub member_id: Uuid,
    /// Name to show.
    pub display_name: String,
}

/// Body of `POST /v1/shared-devices/{id}/switch`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SwitchRequest {
    /// The member who steps up to the device.
    pub member_id: Uuid,
    /// Their PIN.
    pub pin: String,
}

/// Response of a successful switch.
#[derive(Debug, Serialize, ToSchema)]
pub struct SwitchResponse {
    /// The user session of the member (bound to the device).
    pub session: SessionIssued,
    /// Device session record (adr/0007 `device_sessions`).
    pub device_session_id: Uuid,
    /// Who is signed in.
    pub member: MemberTile,
    /// Idle lock in minutes; the server ends the session after it.
    pub idle_timeout_min: i32,
}

// --- helpers -------------------------------------------------------------------------------

fn invalid(field: &str, code: &str) -> ApiError {
    ApiError::validation(vec![FieldIssue::new(field, code)])
}

fn not_found() -> ApiError {
    ApiError::new(ErrorCode::NotFound)
}

fn not_for_device(ctx: &RequestContext) -> ApiResult<()> {
    if ctx.identity.actor_kind == db::ActorKind::Device {
        return Err(ApiError::new(ErrorCode::Forbidden)
            .with_errors(vec![FieldIssue::new("/", "shared_device_session")]));
    }
    Ok(())
}

fn workspace_of(ctx: &RequestContext) -> ApiResult<Uuid> {
    ctx.workspace_id.ok_or_else(not_found)
}

fn clean_name(raw: &str) -> ApiResult<String> {
    let name: String = raw.chars().filter(|c| !c.is_control()).collect();
    let name = name.trim().to_owned();
    if name.is_empty() || name.chars().count() > 100 {
        return Err(invalid("/name", "invalid"));
    }
    Ok(name)
}

/// 32 to 128 characters of the URL-safe base64 alphabet.
fn valid_device_key(key: &str) -> bool {
    (32..=128).contains(&key.len())
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn unique_violation(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .is_some_and(|c| c == "23505")
}

/// A device as the key resolves it.
struct DeviceAuth {
    id: Uuid,
    workspace_id: Uuid,
    idle_timeout_min: i32,
}

/// Resolves the device of `X-Device-Key`; the key must belong to `path_id`.
async fn device_auth(
    svc: &AuthServices,
    headers: &HeaderMap,
    path_id: Uuid,
) -> ApiResult<DeviceAuth> {
    let key = headers
        .get(DEVICE_KEY_HEADER)
        .and_then(|v| v.to_str().ok())
        .filter(|k| !k.is_empty() && k.len() <= 256)
        .ok_or_else(|| {
            ApiError::new(ErrorCode::Unauthenticated).with_errors(vec![FieldIssue::new(
                DEVICE_KEY_HEADER,
                "device_key_invalid",
            )])
        })?;
    let row: Option<(Uuid, Uuid, i32)> = sqlx::query_as(
        "SELECT id, workspace_id, idle_timeout_min FROM find_shared_device_by_key_hash($1)",
    )
    .bind(token_hash(key))
    .fetch_optional(&svc.db)
    .await?;
    let (id, workspace_id, idle_timeout_min) = row.ok_or_else(|| {
        ApiError::new(ErrorCode::Unauthenticated).with_errors(vec![FieldIssue::new(
            DEVICE_KEY_HEADER,
            "device_key_invalid",
        )])
    })?;
    if id != path_id {
        return Err(not_found());
    }
    Ok(DeviceAuth {
        id,
        workspace_id,
        idle_timeout_min,
    })
}

/// Closes every session of the device and the open device sessions; the caller commits.
async fn close_device_sessions(
    svc: &AuthServices,
    conn: &mut PgConnection,
    device_id: Uuid,
) -> ApiResult<Vec<Vec<u8>>> {
    let hashes: Vec<(Vec<u8>,)> =
        sqlx::query_as("SELECT token_hash FROM close_shared_device_sessions($1)")
            .bind(device_id)
            .fetch_all(&mut *conn)
            .await?;
    sqlx::query(
        "UPDATE device_sessions SET ended_at = now() \
         WHERE shared_device_id = $1 AND ended_at IS NULL",
    )
    .bind(device_id)
    .execute(&mut *conn)
    .await?;
    let _ = svc;
    Ok(hashes.into_iter().map(|h| h.0).collect())
}

async fn forget_all(svc: &AuthServices, hashes: &[Vec<u8>]) {
    for h in hashes {
        svc.sessions.forget(h).await;
    }
}

// --- administrator endpoints ---------------------------------------------------------------

/// Opens shared device mode for a device. Needs `devices.manage` and the second step.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws}/shared-devices",
    tag = "shared-devices",
    operation_id = "shareddevices_create",
    params(("ws" = Uuid, Path, description = "Workspace id")),
    request_body = SharedDeviceCreate,
    responses((
        status = 201,
        description = "Device registered",
        body = SharedDeviceSummary,
        example = json!({"id": "0192f1c0-7a3b-7000-8000-00000000002a", "name": "Brewhouse iPad",
                         "idle_timeout_min": 5, "opened_by": "0192f1c0-7a3b-7000-8000-00000000000a",
                         "created_at": "2026-11-14T10:00:00Z", "revoked_at": null})
    ))
)]
pub async fn device_create(
    ctx: RequestContext,
    Extension(svc): Svc,
    ApiJson(req): ApiJson<SharedDeviceCreate>,
) -> ApiResult<(StatusCode, Json<SharedDeviceSummary>)> {
    ctx.require(MANAGE)?;
    not_for_device(&ctx)?;
    svc.require_step_up(&ctx, ProtectedAction::SharedDeviceOpen)
        .await?;
    let name = clean_name(&req.name)?;
    let idle = req.idle_timeout_min.unwrap_or(5);
    if !(1..=60).contains(&idle) {
        return Err(invalid("/idle_timeout_min", "out_of_range"));
    }
    if !valid_device_key(&req.device_key) {
        return Err(invalid("/device_key", "invalid"));
    }
    let workspace_id = workspace_of(&ctx)?;
    let user_id = ctx.identity.user_id;
    let mut tx = ctx.begin(&svc.db).await?;
    let inserted: Result<DeviceRow, sqlx::Error> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "INSERT INTO shared_devices (id, workspace_id, name, opened_by, device_key_hash, \
                                     idle_timeout_min, created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $4, $4) RETURNING {DEVICE_COLUMNS}"
    )))
    .bind(Uuid::now_v7())
    .bind(workspace_id)
    .bind(&name)
    .bind(user_id)
    .bind(token_hash(&req.device_key))
    .bind(idle)
    .fetch_one(&mut *tx)
    .await;
    let row = match inserted {
        Ok(r) => r,
        Err(e) if unique_violation(&e) => {
            return Err(ApiError::new(ErrorCode::Conflict)
                .with_errors(vec![FieldIssue::new("/device_key", "device_key_in_use")]));
        }
        Err(e) => return Err(e.into()),
    };
    tx.commit().await?;
    tracing::info!(workspace_id = %workspace_id, device_id = %row.id, "shared device opened");
    Ok((StatusCode::CREATED, Json(row.into())))
}

/// Lists the workspace's shared devices.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws}/shared-devices",
    tag = "shared-devices",
    operation_id = "shareddevices_list",
    params(("ws" = Uuid, Path, description = "Workspace id")),
    responses((
        status = 200,
        description = "Devices",
        body = Vec<SharedDeviceSummary>,
        example = json!([{"id": "0192f1c0-7a3b-7000-8000-00000000002a", "name": "Brewhouse iPad",
                          "idle_timeout_min": 5, "opened_by": "0192f1c0-7a3b-7000-8000-00000000000a",
                          "created_at": "2026-11-14T10:00:00Z", "revoked_at": null}])
    ))
)]
pub async fn device_list(
    ctx: RequestContext,
    Extension(svc): Svc,
) -> ApiResult<Json<Vec<SharedDeviceSummary>>> {
    ctx.require(MANAGE)?;
    let mut tx = ctx.begin(&svc.db).await?;
    let rows: Vec<DeviceRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {DEVICE_COLUMNS} FROM shared_devices WHERE deleted_at IS NULL \
         ORDER BY created_at, id LIMIT 200"
    )))
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

/// Revokes a device: its key stops working and its sessions close.
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws}/shared-devices/{id}",
    tag = "shared-devices",
    operation_id = "shareddevices_revoke",
    params(("ws" = Uuid, Path, description = "Workspace id"), ("id" = Uuid, Path, description = "Device id")),
    responses((status = 204, description = "Device revoked"))
)]
pub async fn device_revoke(
    ctx: RequestContext,
    Extension(svc): Svc,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Response> {
    ctx.require(MANAGE)?;
    let mut tx = ctx.begin(&svc.db).await?;
    let done = sqlx::query(
        "UPDATE shared_devices SET revoked_at = now(), updated_by = $2 \
         WHERE id = $1 AND revoked_at IS NULL AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await?;
    if done.rows_affected() == 0 {
        return Err(not_found());
    }
    let hashes = close_device_sessions(&svc, &mut tx, id).await?;
    tx.commit().await?;
    forget_all(&svc, &hashes).await;
    tracing::info!(device_id = %id, "shared device revoked");
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Replaces the device key. Needs `devices.manage` and the second step; the device's open
/// session closes.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws}/shared-devices/{id}/rotate-key",
    tag = "shared-devices",
    operation_id = "shareddevices_rotatekey",
    params(("ws" = Uuid, Path, description = "Workspace id"), ("id" = Uuid, Path, description = "Device id")),
    request_body = DeviceKeyRotate,
    responses((status = 204, description = "Key replaced"))
)]
pub async fn device_rotate_key(
    ctx: RequestContext,
    Extension(svc): Svc,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    ApiJson(req): ApiJson<DeviceKeyRotate>,
) -> ApiResult<Response> {
    ctx.require(MANAGE)?;
    not_for_device(&ctx)?;
    svc.require_step_up(&ctx, ProtectedAction::DeviceKeyRotate)
        .await?;
    if !valid_device_key(&req.device_key) {
        return Err(invalid("/device_key", "invalid"));
    }
    let mut tx = ctx.begin(&svc.db).await?;
    let updated = sqlx::query(
        "UPDATE shared_devices SET device_key_hash = $2, updated_by = $3 \
         WHERE id = $1 AND revoked_at IS NULL AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(token_hash(&req.device_key))
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await;
    match updated {
        Ok(r) if r.rows_affected() == 0 => return Err(not_found()),
        Ok(_) => {}
        Err(e) if unique_violation(&e) => {
            return Err(ApiError::new(ErrorCode::Conflict)
                .with_errors(vec![FieldIssue::new("/device_key", "device_key_in_use")]));
        }
        Err(e) => return Err(e.into()),
    }
    let hashes = close_device_sessions(&svc, &mut tx, id).await?;
    tx.commit().await?;
    forget_all(&svc, &hashes).await;
    tracing::info!(device_id = %id, "shared device key rotated");
    Ok(StatusCode::NO_CONTENT.into_response())
}

// --- PIN endpoints -------------------------------------------------------------------------

/// Sets (or replaces) the caller's own PIN. Replacing clears a lock.
#[utoipa::path(
    put,
    path = "/v1/workspaces/{ws}/me/pin",
    tag = "shared-devices",
    operation_id = "shareddevices_pinset",
    params(("ws" = Uuid, Path, description = "Workspace id")),
    request_body = PinSet,
    responses((status = 204, description = "PIN set"))
)]
pub async fn pin_set(
    ctx: RequestContext,
    Extension(svc): Svc,
    ApiJson(req): ApiJson<PinSet>,
) -> ApiResult<Response> {
    not_for_device(&ctx)?;
    if !is_valid_pin(&req.pin) {
        return Err(invalid("/pin", "invalid_pin"));
    }
    let workspace_id = workspace_of(&ctx)?;
    let hash = hash_pin(&req.pin).await?;
    let user_id = ctx.identity.user_id;
    let mut tx = ctx.begin(&svc.db).await?;
    let member: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM workspace_members \
         WHERE workspace_id = $1 AND user_id = $2 AND status = 'active' AND deleted_at IS NULL",
    )
    .bind(workspace_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;
    let (member_id,) = member.ok_or_else(not_found)?;
    sqlx::query(
        "INSERT INTO member_pins (id, workspace_id, member_id, pin_hash, created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $5) \
         ON CONFLICT (workspace_id, member_id) DO UPDATE \
         SET pin_hash = EXCLUDED.pin_hash, failed_attempts = 0, locked_until = NULL, \
             deleted_at = NULL, updated_by = EXCLUDED.updated_by",
    )
    .bind(Uuid::now_v7())
    .bind(workspace_id)
    .bind(member_id)
    .bind(hash)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    tracing::info!(user_id = %user_id, "member PIN set");
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Clears a member's PIN (forgotten PIN, lock): the member sets a new one.
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws}/members/{member}/pin",
    tag = "shared-devices",
    operation_id = "shareddevices_pinclear",
    params(("ws" = Uuid, Path, description = "Workspace id"), ("member" = Uuid, Path, description = "Member id")),
    responses((status = 204, description = "PIN cleared"))
)]
pub async fn pin_clear(
    ctx: RequestContext,
    Extension(svc): Svc,
    Path((_ws, member)): Path<(Uuid, Uuid)>,
) -> ApiResult<Response> {
    ctx.require(MANAGE)?;
    let mut tx = ctx.begin(&svc.db).await?;
    let done = sqlx::query("DELETE FROM member_pins WHERE member_id = $1")
        .bind(member)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    if done.rows_affected() == 0 {
        return Err(not_found());
    }
    Ok(StatusCode::NO_CONTENT.into_response())
}

// --- device endpoints ----------------------------------------------------------------------

/// The tiles of the device: members who have a PIN.
#[utoipa::path(
    get,
    path = "/v1/shared-devices/{id}/members",
    tag = "shared-devices",
    operation_id = "shareddevices_members",
    params(
        ("id" = Uuid, Path, description = "Device id"),
        ("X-Device-Key" = String, Header, description = "The device key")
    ),
    responses((
        status = 200,
        description = "Members",
        body = Vec<MemberTile>,
        example = json!([{"member_id": "0192f1c0-7a3b-7000-8000-00000000003a", "display_name": "Sam Brewer"}])
    ))
)]
pub async fn device_members(
    Extension(svc): Svc,
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<MemberTile>>> {
    let device = device_auth(&svc, &headers, id).await?;
    let ctx = db::DbContext {
        user_id: None,
        workspace_id: Some(device.workspace_id),
        actor_kind: db::ActorKind::Device,
    };
    let mut tx = db::begin(&svc.db, &ctx).await?;
    let rows: Vec<(Uuid, String)> =
        sqlx::query_as("SELECT member_id, display_name FROM shared_device_member_tiles($1)")
            .bind(device.workspace_id)
            .fetch_all(&mut *tx)
            .await?;
    tx.commit().await?;
    Ok(Json(
        rows.into_iter()
            .map(|(member_id, display_name)| MemberTile {
                member_id,
                display_name,
            })
            .collect(),
    ))
}

fn pin_invalid() -> ApiError {
    ApiError::new(ErrorCode::Unauthenticated)
        .with_errors(vec![FieldIssue::new("/pin", "pin_invalid")])
}

fn pin_locked(secs: i64) -> ApiError {
    ApiError::new(ErrorCode::RateLimited)
        .with_errors(vec![FieldIssue::new("/pin", "pin_locked")])
        .with_retry_after(u64::try_from(secs.max(1)).unwrap_or(1))
}

/// Checks the PIN of a member; counts failures and locks. Returns the member's user id.
async fn check_pin(
    svc: &AuthServices,
    device: &DeviceAuth,
    member_id: Uuid,
    pin: &str,
) -> ApiResult<Uuid> {
    let ctx = db::DbContext {
        user_id: None,
        workspace_id: Some(device.workspace_id),
        actor_kind: db::ActorKind::Device,
    };
    let mut tx = db::begin(&svc.db, &ctx).await?;
    let member: Option<(Uuid,)> = sqlx::query_as(
        "SELECT user_id FROM workspace_members \
         WHERE workspace_id = $1 AND id = $2 AND status = 'active' AND deleted_at IS NULL",
    )
    .bind(device.workspace_id)
    .bind(member_id)
    .fetch_optional(&mut *tx)
    .await?;
    // Serialises attempts of one member, so parallel guesses cannot dodge the counter.
    let row: Option<(String, i32, Option<DateTime<Utc>>)> = if member.is_some() {
        sqlx::query_as(
            "SELECT pin_hash, failed_attempts, locked_until FROM member_pins \
             WHERE workspace_id = $1 AND member_id = $2 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(device.workspace_id)
        .bind(member_id)
        .fetch_optional(&mut *tx)
        .await?
    } else {
        None
    };
    let (Some((user_id,)), Some((hash, failed, locked_until))) = (member, row) else {
        tx.commit().await?;
        dummy_verify(pin).await;
        return Err(pin_invalid());
    };
    let now = Utc::now();
    if let Some(until) = locked_until.filter(|u| *u > now) {
        tx.commit().await?;
        return Err(pin_locked((until - now).num_seconds()));
    }
    if verify_pin(pin, &hash).await {
        sqlx::query(
            "UPDATE member_pins SET failed_attempts = 0, locked_until = NULL \
             WHERE workspace_id = $1 AND member_id = $2",
        )
        .bind(device.workspace_id)
        .bind(member_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        return Ok(user_id);
    }
    // An expired lock leaves the counter at zero so a fresh run of failures starts over.
    let failed = failed + 1;
    let lock = failed >= MAX_FAILED_ATTEMPTS;
    sqlx::query(
        "UPDATE member_pins SET failed_attempts = $3, locked_until = $4 \
         WHERE workspace_id = $1 AND member_id = $2",
    )
    .bind(device.workspace_id)
    .bind(member_id)
    .bind(if lock { 0 } else { failed })
    .bind(lock.then(|| now + Duration::minutes(LOCK_MINUTES)))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    if lock {
        tracing::warn!(member_id = %member_id, device_id = %device.id, "member PIN locked");
        return Err(pin_locked(LOCK_MINUTES * 60));
    }
    Err(pin_invalid())
}

/// A member steps up to the device: PIN in, user session out.
#[utoipa::path(
    post,
    path = "/v1/shared-devices/{id}/switch",
    tag = "shared-devices",
    operation_id = "shareddevices_switch",
    params(
        ("id" = Uuid, Path, description = "Device id"),
        ("X-Device-Key" = String, Header, description = "The device key")
    ),
    request_body = SwitchRequest,
    responses((
        status = 200,
        description = "Member signed in on the device",
        body = SwitchResponse,
        example = json!({"session": {"id": "0192f1c0-7a3b-7000-8000-00000000004a", "client": "shared_device",
                                     "token": "opaque", "csrf_token": null,
                                     "expires_at": "2026-11-14T22:00:00Z"},
                         "device_session_id": "0192f1c0-7a3b-7000-8000-00000000004b",
                         "member": {"member_id": "0192f1c0-7a3b-7000-8000-00000000003a", "display_name": "Sam Brewer"},
                         "idle_timeout_min": 5})
    ))
)]
pub async fn device_switch(
    Extension(svc): Svc,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    ApiJson(req): ApiJson<SwitchRequest>,
) -> ApiResult<Json<SwitchResponse>> {
    let device = device_auth(&svc, &headers, id).await?;
    state
        .rate
        .check_window(
            "device_pin",
            &id.to_string(),
            DEVICE_ATTEMPTS_PER_MINUTE,
            60,
        )
        .await?;
    if !is_valid_pin(&req.pin) {
        return Err(invalid("/pin", "invalid_pin"));
    }
    let user_id = check_pin(&svc, &device, req.member_id, &req.pin).await?;

    let ctx = db::DbContext {
        user_id: Some(user_id),
        workspace_id: Some(device.workspace_id),
        actor_kind: db::ActorKind::Device,
    };
    let mut tx = db::begin(&svc.db, &ctx).await?;
    let (display_name, device_name): (String, String) = sqlx::query_as(
        "SELECT (SELECT display_name FROM users WHERE id = $1), \
                (SELECT name FROM shared_devices WHERE id = $2)",
    )
    .bind(user_id)
    .bind(device.id)
    .fetch_one(&mut *tx)
    .await?;
    let closed = close_device_sessions(&svc, &mut tx, device.id).await?;
    let (token, record) = insert_device_session(&mut tx, user_id, &device, &device_name).await?;
    let device_session_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO device_sessions (id, workspace_id, shared_device_id, member_id, session_id, \
                                      created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $6)",
    )
    .bind(device_session_id)
    .bind(device.workspace_id)
    .bind(device.id)
    .bind(req.member_id)
    .bind(record.id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    forget_all(&svc, &closed).await;
    tracing::info!(device_id = %device.id, member_id = %req.member_id, "shared device user switched");
    Ok(Json(SwitchResponse {
        session: svc.issued(&record, Some(token)),
        device_session_id,
        member: MemberTile {
            member_id: req.member_id,
            display_name,
        },
        idle_timeout_min: device.idle_timeout_min,
    }))
}

async fn insert_device_session(
    conn: &mut PgConnection,
    user_id: Uuid,
    device: &DeviceAuth,
    device_name: &str,
) -> sqlx::Result<(String, SessionRecord)> {
    let token = new_token();
    let (_, max) = lifetimes(ClientKind::SharedDevice, false);
    let now = Utc::now();
    let idle_secs = i64::from(device.idle_timeout_min) * 60;
    let record = SessionRecord {
        id: Uuid::now_v7(),
        user_id,
        client_kind: ClientKind::SharedDevice,
        shared_device_id: Some(device.id),
        remember: false,
        created_at: now,
        last_activity_at: now,
        expires_at: now + max,
        step_up_at: None,
        revoked_at: None,
        idle_timeout_secs: Some(idle_secs),
    };
    sqlx::query(
        "INSERT INTO sessions (id, user_id, token_hash, client_kind, shared_device_id, device_name, \
                               last_activity_at, expires_at, created_at, idle_timeout_secs) \
         VALUES ($1, $2, $3, 'shared_device', $4, $5, $6, $7, $6, $8)",
    )
    .bind(record.id)
    .bind(user_id)
    .bind(token_hash(&token))
    .bind(device.id)
    .bind(device_name)
    .bind(now)
    .bind(record.expires_at)
    .bind(i32::try_from(idle_secs).unwrap_or(300))
    .execute(conn)
    .await?;
    Ok((token, record))
}

/// Locks the device: the signed-in member's session ends at once.
#[utoipa::path(
    post,
    path = "/v1/shared-devices/{id}/lock",
    tag = "shared-devices",
    operation_id = "shareddevices_lock",
    params(
        ("id" = Uuid, Path, description = "Device id"),
        ("X-Device-Key" = String, Header, description = "The device key")
    ),
    responses((status = 204, description = "Device locked"))
)]
pub async fn device_lock(
    Extension(svc): Svc,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let device = device_auth(&svc, &headers, id).await?;
    let ctx = db::DbContext {
        user_id: None,
        workspace_id: Some(device.workspace_id),
        actor_kind: db::ActorKind::Device,
    };
    let mut tx = db::begin(&svc.db, &ctx).await?;
    let hashes = close_device_sessions(&svc, &mut tx, device.id).await?;
    tx.commit().await?;
    forget_all(&svc, &hashes).await;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Shared device administration, member PINs and the routes the device calls with its key.
pub fn routes(state: &AppState) -> Router<AppState> {
    // Without the auth services (see `AppState::with_auth_services`) these routes are off.
    if state.auth_services.is_none() {
        return Router::new();
    }
    // Shared device administration and PINs (ADR-0009 rule 7).
    let devices_admin = crate::authenticated(
        state,
        crate::idempotent(
            state,
            Router::new().route("/v1/workspaces/{ws}/shared-devices", post(device_create)),
        )
        .route("/v1/workspaces/{ws}/shared-devices", get(device_list))
        .route(
            "/v1/workspaces/{ws}/shared-devices/{id}",
            delete(device_revoke),
        )
        .route(
            "/v1/workspaces/{ws}/shared-devices/{id}/rotate-key",
            post(device_rotate_key),
        )
        .route("/v1/workspaces/{ws}/me/pin", put(pin_set))
        .route(
            "/v1/workspaces/{ws}/members/{member}/pin",
            delete(pin_clear),
        ),
    );
    // The device proves itself with its key (`X-Device-Key`), not with a session.
    let device_key = Router::new()
        .route("/v1/shared-devices/{id}/members", get(device_members))
        .route("/v1/shared-devices/{id}/switch", post(device_switch))
        .route("/v1/shared-devices/{id}/lock", post(device_lock));
    devices_admin.merge(device_key)
}

// --- registry entry ----------------------------------------------------------------------

/// Operation specs of this module (rows for `crate::openapi`).
mod spec {
    use crate::error::ErrorCode;
    use crate::error::ErrorCode::{
        Conflict, Forbidden, IdempotencyInProgress, IdempotencyMismatch, Internal,
        MalformedRequest, NotFound, PreconditionRequired, RateLimited, StepUpRequired,
        Unauthenticated, UnsupportedMediaType, UpgradeRequired, ValidationFailed,
    };
    use crate::openapi::{OperationSpec, op};

    const DEVICE_ADMIN_ERRORS: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        StepUpRequired,
        NotFound,
        Conflict,
        UnsupportedMediaType,
        ValidationFailed,
        UpgradeRequired,
        RateLimited,
        Internal,
    ];

    const DEVICE_CREATE_ERRORS: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        StepUpRequired,
        NotFound,
        Conflict,
        IdempotencyInProgress,
        UnsupportedMediaType,
        ValidationFailed,
        IdempotencyMismatch,
        PreconditionRequired,
        UpgradeRequired,
        RateLimited,
        Internal,
    ];

    /// Endpoints the device calls with its key: no session, so `401` means a bad key.
    const DEVICE_KEY_ERRORS: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        NotFound,
        UnsupportedMediaType,
        ValidationFailed,
        UpgradeRequired,
        RateLimited,
        Internal,
    ];

    pub(super) const OPERATIONS: &[OperationSpec] = &[
        op(
            "shareddevices_create",
            false,
            Some("devices.manage"),
            DEVICE_CREATE_ERRORS,
        ),
        op(
            "shareddevices_list",
            false,
            Some("devices.manage"),
            DEVICE_ADMIN_ERRORS,
        ),
        op(
            "shareddevices_revoke",
            false,
            Some("devices.manage"),
            DEVICE_ADMIN_ERRORS,
        ),
        op(
            "shareddevices_rotatekey",
            false,
            Some("devices.manage"),
            DEVICE_ADMIN_ERRORS,
        ),
        op("shareddevices_pinset", false, None, DEVICE_ADMIN_ERRORS),
        op(
            "shareddevices_pinclear",
            false,
            Some("devices.manage"),
            DEVICE_ADMIN_ERRORS,
        ),
        op("shareddevices_members", true, None, DEVICE_KEY_ERRORS),
        op("shareddevices_switch", true, None, DEVICE_KEY_ERRORS),
        op("shareddevices_lock", true, None, DEVICE_KEY_ERRORS),
    ];
}

/// `OpenAPI` fragment of this module; `crate::openapi` merges it.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(
        device_create,
        device_list,
        device_revoke,
        device_rotate_key,
        pin_set,
        pin_clear,
        device_members,
        device_switch,
        device_lock
    ),
    components(schemas(
        DeviceKeyRotate,
        MemberTile,
        PinSet,
        SharedDeviceCreate,
        SharedDeviceSummary,
        SwitchRequest,
        SwitchResponse
    )),
    tags(
        (name = "shared-devices", description = "Shared device mode: devices, member PINs, user switching")
    )
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "shared-devices",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_keys_are_long_url_safe_strings() {
        assert!(valid_device_key(&"a".repeat(43)));
        assert!(!valid_device_key(&"a".repeat(31)));
        assert!(!valid_device_key(&"a".repeat(129)));
        assert!(!valid_device_key(&format!("{}+", "a".repeat(40))));
    }

    #[test]
    fn pin_module_limits_match_the_adr() {
        assert_eq!(MAX_FAILED_ATTEMPTS, 5);
        assert_eq!(LOCK_MINUTES, 15);
        assert_eq!(DEVICE_ATTEMPTS_PER_MINUTE, 20);
    }
}
