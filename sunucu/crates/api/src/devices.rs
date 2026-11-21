//! Devices (T2.25): registration, pairing with a batch and the data address key.
//!
//! A device (iSpindel, RAPT Pill, Tilt bridge, other sender) has one data address
//! `/v1/ingest/{key}` on the `ingest` service. The server makes the key, returns it once (on
//! create and on rotate) and keeps only its SHA-256 digest. A rotation replaces the digest, so the
//! old address stops working at once. Pairing sets `batch_id`: the worker writes the readings of
//! the device as measurements of that batch (`devices` crate).
//!
//! The create route has no idempotency layer: a stored replay would keep the key in the
//! idempotency table. A client that lost the answer rotates the key or deletes the device.
//!
//! TODO: pairing with a tank (`vessel_id`) comes with the Business tank endpoints.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use db::{AuditEvent, NewAuditEntry};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{FromRow, PgConnection};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    auth::step_up::ProtectedAction,
    concurrency::{self, Concurrency, check_if_match},
    context::RequestContext,
    error::{ApiError, ApiJson, ApiQuery, ApiResult, FieldIssue, MergePatch},
    library::{created_response, id_cursor, issue, merge, not_found, paging, sqlstate, workspace},
    pagination::{ListParams, Page},
    recipe_org::conflict,
};

/// Permission that registers, pairs, renews and deletes devices.
pub const MANAGE_PERMISSION: &str = "devices.manage";
/// Permission that reads devices.
pub const READ_PERMISSION: &str = "recipe.view";

const KINDS: &[&str] = &["ispindel", "rapt", "tilt", "other"];

/// A device. The key digest never leaves the server.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct Device {
    /// Identifier.
    pub id: Uuid,
    /// `ispindel`, `rapt`, `tilt` or `other`.
    pub kind: String,
    /// Name.
    pub name: String,
    /// The paired batch.
    pub batch_id: Option<Uuid>,
    /// Calibration record, for example `{"schema_version": 1, "gravity_unit": "plato"}`.
    #[schema(value_type = Option<Object>)]
    pub calibration: Option<Value>,
    /// Time of the last accepted reading.
    pub last_data_at: Option<DateTime<Utc>>,
    /// Time the current key was made.
    pub ingest_key_rotated_at: DateTime<Utc>,
    /// Creation time.
    pub created_at: DateTime<Utc>,
    /// Last change.
    pub updated_at: DateTime<Utc>,
    /// Version (`ETag`).
    pub version: i64,
}

/// A device with its data address. Only create and rotate return the key.
#[derive(Debug, Serialize, ToSchema)]
pub struct DeviceWithKey {
    /// The device.
    #[serde(flatten)]
    pub device: Device,
    /// The key. Shown once; keep it secret.
    pub ingest_key: String,
    /// Path of the data address on the ingest service.
    pub ingest_path: String,
    /// Full data address, when the server knows its public URL (`INGEST_PUBLIC_URL`).
    pub ingest_url: Option<String>,
}

/// Body of `POST /v1/workspaces/{ws}/devices`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NewDevice {
    /// `ispindel`, `rapt`, `tilt` or `other`.
    pub kind: String,
    /// Name (1 to 200 characters).
    pub name: String,
    /// Batch to pair with.
    pub batch_id: Option<Uuid>,
    /// Calibration record (object with `schema_version`).
    #[schema(value_type = Option<Object>)]
    pub calibration: Option<Value>,
}

fn device_example() -> Value {
    json!({
        "id": "0192f1c07a3b7000800000000000000e", "kind": "ispindel", "name": "Green iSpindel",
        "batch_id": "0192f1c07a3b7000800000000000000b",
        "calibration": {"schema_version": 1, "gravity_unit": "plato"},
        "last_data_at": "2026-11-21T10:15:00Z", "ingest_key_rotated_at": "2026-11-20T10:00:00Z",
        "created_at": "2026-11-20T10:00:00Z", "updated_at": "2026-11-21T10:15:00Z", "version": 3
    })
}

fn device_with_key_example() -> Value {
    let mut v = device_example();
    v["ingest_key"] = json!("k3Zq1m9XbV7c2nJ0aT5dY8wLpR4sGhUeFiOxNvB6CyA");
    v["ingest_path"] = json!("/v1/ingest/k3Zq1m9XbV7c2nJ0aT5dY8wLpR4sGhUeFiOxNvB6CyA");
    v["ingest_url"] =
        json!("https://ingest.example.com/v1/ingest/k3Zq1m9XbV7c2nJ0aT5dY8wLpR4sGhUeFiOxNvB6CyA");
    v
}

const COLUMNS: &str = "id, kind, name, batch_id, calibration, last_data_at, ingest_key_rotated_at, \
     created_at, updated_at, version";

fn with_key(device: Device, key: String) -> DeviceWithKey {
    let ingest_path = format!("/v1/ingest/{key}");
    let ingest_url = std::env::var("INGEST_PUBLIC_URL")
        .ok()
        .filter(|u| !u.is_empty())
        .map(|u| format!("{}{ingest_path}", u.trim_end_matches('/')));
    DeviceWithKey {
        device,
        ingest_key: key,
        ingest_path,
        ingest_url,
    }
}

fn valid_name(name: &str) -> bool {
    let n = name.trim();
    !n.is_empty() && n.chars().count() <= 200 && !n.chars().any(char::is_control)
}

fn valid_calibration(c: &Value) -> bool {
    c.is_object() && c.get("schema_version").is_some()
}

fn constraint_error(e: sqlx::Error) -> ApiError {
    match sqlstate(&e).as_deref() {
        Some("23503") => issue("/batch_id", "reference_not_found"),
        Some("23514" | "22P02") => issue("/", "constraint_violated"),
        _ => e.into(),
    }
}

async fn fetch(tx: &mut PgConnection, id: Uuid, lock: bool) -> ApiResult<Device> {
    let sql = sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM devices WHERE id = $1 AND deleted_at IS NULL{}",
        if lock { " FOR UPDATE" } else { "" }
    ));
    sqlx::query_as(sql)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(not_found)
}

/// A device can pair only with an open batch of the workspace.
async fn check_batch(tx: &mut PgConnection, batch_id: Uuid) -> ApiResult<()> {
    let stage: Option<String> =
        sqlx::query_scalar("SELECT stage FROM batches WHERE id = $1 AND deleted_at IS NULL")
            .bind(batch_id)
            .fetch_optional(&mut *tx)
            .await?;
    match stage.as_deref() {
        None => Err(issue("/batch_id", "reference_not_found")),
        Some("completed" | "cancelled") => Err(conflict("/batch_id", "batch_closed")),
        Some(_) => Ok(()),
    }
}

async fn audit(
    tx: &mut PgConnection,
    ctx: &RequestContext,
    ws: Uuid,
    id: Uuid,
    new: Value,
) -> ApiResult<()> {
    db::append_audit(
        tx,
        ws,
        &NewAuditEntry {
            actor_kind: ctx.identity.actor_kind,
            actor_id: Some(ctx.identity.user_id),
            event: AuditEvent::DeviceKeyRenew,
            record_kind: "device".to_owned(),
            record_id: Some(id),
            previous_value: None,
            new_value: Some(new),
            reason: None,
        },
    )
    .await?;
    Ok(())
}

/// Routes of this module.
pub fn routes(state: &AppState) -> Router<AppState> {
    const WS: &str = "/v1/workspaces/{ws}";
    crate::authenticated(
        state,
        Router::new()
            .route(
                &format!("{WS}/devices"),
                get(device_list).post(device_create),
            )
            .route(
                &format!("{WS}/devices/{{id}}"),
                get(device_get).patch(device_update).delete(device_delete),
            )
            .route(
                &format!("{WS}/devices/{{id}}/rotate-key"),
                post(device_rotate),
            ),
    )
}

/// Lists the devices of the workspace.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/devices",
    tag = "devices",
    operation_id = "device_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page"),
        ("sort" = Option<String>, Query, description = "`id` (creation order, default) or `-id`")
    ),
    responses((status = 200, description = "A page of devices", body = Page<Device>,
        example = json!({"items": [device_example()], "next_cursor": null})))
)]
pub async fn device_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(params): ApiQuery<ListParams>,
) -> ApiResult<Json<Page<Device>>> {
    ctx.require(READ_PERMISSION)?;
    let paging = paging(&params, &["id"], "-id")?;
    let (cmp, dir) = paging.dir();
    let sql = sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM devices WHERE deleted_at IS NULL \
           AND ($1::uuid IS NULL OR id {cmp} $1) ORDER BY id {dir} LIMIT $2"
    ));
    let mut tx = ctx.begin(&state.db).await?;
    let rows: Vec<Device> = sqlx::query_as(sql)
        .bind(paging.id)
        .bind(i64::from(paging.limit) + 1)
        .fetch_all(&mut *tx)
        .await?;
    Ok(Json(Page::from_rows(
        rows,
        paging.limit,
        &paging.sort,
        |r| id_cursor(r.id),
    )))
}

/// Registers a device and returns its key once. The key is part of the data address.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/devices",
    tag = "devices",
    operation_id = "device_create",
    params(("workspace_id" = Uuid, Path, description = "Workspace")),
    request_body = NewDevice,
    responses((status = 201, description = "The device and its key (shown once)", body = DeviceWithKey,
        example = json!(device_with_key_example())))
)]
pub async fn device_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<NewDevice>,
) -> ApiResult<Response> {
    ctx.require(MANAGE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let mut issues = Vec::new();
    if !KINDS.contains(&body.kind.as_str()) {
        issues.push(FieldIssue::new("/kind", "invalid_value"));
    }
    if !valid_name(&body.name) {
        issues.push(FieldIssue::new("/name", "invalid_value"));
    }
    if body
        .calibration
        .as_ref()
        .is_some_and(|c| !valid_calibration(c))
    {
        issues.push(FieldIssue::new("/calibration", "invalid_value"));
    }
    if !issues.is_empty() {
        return Err(ApiError::validation(issues));
    }
    let key = devices::new_key();
    let id = Uuid::now_v7();
    let mut tx = ctx.begin(&state.db).await?;
    if let Some(b) = body.batch_id {
        check_batch(&mut tx, b).await?;
    }
    sqlx::query(
        "INSERT INTO devices (id, workspace_id, kind, name, ingest_key_hash, batch_id, \
             calibration, created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)",
    )
    .bind(id)
    .bind(ws)
    .bind(&body.kind)
    .bind(body.name.trim())
    .bind(devices::key_hash(&key))
    .bind(body.batch_id)
    .bind(&body.calibration)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    let device = fetch(&mut tx, id, false).await?;
    tx.commit().await?;
    let version = device.version;
    Ok(created_response(
        &format!("/v1/workspaces/{ws}/devices/{id}"),
        version,
        &with_key(device, key),
    ))
}

/// Reads a device.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/devices/{id}",
    tag = "devices",
    operation_id = "device_get",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Device")
    ),
    responses(
        (status = 200, description = "The device", body = Device, example = json!(device_example())),
        (status = 304, description = "Not modified (`If-None-Match` matched)")
    )
)]
pub async fn device_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    ctx.require(READ_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let row = fetch(&mut tx, id, false).await?;
    Ok(concurrency::conditional_get(&headers, row.version, &row))
}

/// Changes the name, the calibration or the paired batch (`null` unpairs). `If-Match` is
/// optional: each reading also changes the version.
#[utoipa::path(
    patch,
    path = "/v1/workspaces/{workspace_id}/devices/{id}",
    tag = "devices",
    operation_id = "device_update",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Device")
    ),
    request_body(content = Value, content_type = "application/merge-patch+json"),
    responses((status = 200, description = "The updated device", body = Device,
        example = json!(device_example())))
)]
pub async fn device_update(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    MergePatch(patch): MergePatch<Value>,
) -> ApiResult<Response> {
    ctx.require(MANAGE_PERMISSION)?;
    let Some(fields) = patch.as_object() else {
        return Err(issue("/", "invalid"));
    };
    if let Some(bad) = fields
        .keys()
        .find(|k| !matches!(k.as_str(), "name" | "batch_id" | "calibration"))
    {
        return Err(issue(&format!("/{bad}"), "read_only"));
    }
    let mut tx = ctx.begin(&state.db).await?;
    let before = fetch(&mut tx, id, true).await?;
    check_if_match(Concurrency::FieldLww, &headers, before.version, || {
        json!(before)
    })?;
    let mut merged = json!({
        "name": before.name, "batch_id": before.batch_id, "calibration": before.calibration
    });
    merge(&mut merged, &patch);
    let name = merged["name"].as_str().unwrap_or_default().to_owned();
    let batch_id: Option<Uuid> = match &merged["batch_id"] {
        Value::Null => None,
        v => Some(
            v.as_str()
                .and_then(|s| Uuid::parse_str(s).ok())
                .ok_or_else(|| issue("/batch_id", "invalid_value"))?,
        ),
    };
    let calibration = match &merged["calibration"] {
        Value::Null => None,
        c if valid_calibration(c) => Some(c.clone()),
        _ => return Err(issue("/calibration", "invalid_value")),
    };
    if !valid_name(&name) {
        return Err(issue("/name", "invalid_value"));
    }
    if let Some(b) = batch_id.filter(|b| Some(*b) != before.batch_id) {
        check_batch(&mut tx, b).await?;
    }
    sqlx::query(
        "UPDATE devices SET name = $2, batch_id = $3, calibration = $4, updated_by = $5 \
         WHERE id = $1",
    )
    .bind(id)
    .bind(name.trim())
    .bind(batch_id)
    .bind(&calibration)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    let row = fetch(&mut tx, id, false).await?;
    tx.commit().await?;
    Ok(concurrency::record_response(
        StatusCode::OK,
        row.version,
        &row,
    ))
}

/// Deletes a device (soft delete). Its data address stops working at once.
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/devices/{id}",
    tag = "devices",
    operation_id = "device_delete",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Device")
    ),
    responses((status = 204, description = "Deleted"))
)]
pub async fn device_delete(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Response> {
    ctx.require(MANAGE_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    fetch(&mut tx, id, true).await?;
    sqlx::query("UPDATE devices SET deleted_at = now(), updated_by = $2 WHERE id = $1")
        .bind(id)
        .bind(ctx.identity.user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Makes a new key and returns it once. The old key stops working at once. Needs the second
/// step when the auth services run.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/devices/{id}/rotate-key",
    tag = "devices",
    operation_id = "device_rotatekey",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Device")
    ),
    responses((status = 200, description = "The device and the new key (shown once)", body = DeviceWithKey,
        example = json!(device_with_key_example())))
)]
pub async fn device_rotate(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Response> {
    ctx.require(MANAGE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    if let Some(svc) = &state.auth_services {
        svc.require_step_up(&ctx, ProtectedAction::DeviceKeyRotate)
            .await?;
    }
    let key = devices::new_key();
    let mut tx = ctx.begin(&state.db).await?;
    fetch(&mut tx, id, true).await?;
    sqlx::query(
        "UPDATE devices SET ingest_key_hash = $2, ingest_key_rotated_at = now(), updated_by = $3 \
         WHERE id = $1",
    )
    .bind(id)
    .bind(devices::key_hash(&key))
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await?;
    audit(&mut tx, &ctx, ws, id, json!({"kind": "ingest_device"})).await?;
    let device = fetch(&mut tx, id, false).await?;
    tx.commit().await?;
    let version = device.version;
    Ok(concurrency::record_response(
        StatusCode::OK,
        version,
        &with_key(device, key),
    ))
}

// --- registry entry ----------------------------------------------------------------------

mod spec {
    use crate::concurrency::Concurrency;
    use crate::error::ErrorCode;
    use crate::error::ErrorCode::{
        Conflict, Forbidden, Internal, MalformedRequest, NotFound, PayloadTooLarge, RateLimited,
        StepUpRequired, Unauthenticated, Unavailable, UnsupportedMediaType, UpgradeRequired,
        ValidationFailed, VersionMismatch,
    };
    use crate::openapi::{OperationSpec, file_op};

    const READ: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        ValidationFailed,
        RateLimited,
        UpgradeRequired,
        Internal,
        Unavailable,
    ];
    const WRITE: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        Conflict,
        VersionMismatch,
        PayloadTooLarge,
        UnsupportedMediaType,
        ValidationFailed,
        StepUpRequired,
        RateLimited,
        UpgradeRequired,
        Internal,
        Unavailable,
    ];

    pub(super) const OPERATIONS: &[OperationSpec] = &[
        file_op("device_list", Some("recipe.view"), None, READ),
        file_op("device_create", Some("devices.manage"), None, WRITE),
        file_op("device_get", Some("recipe.view"), None, READ),
        file_op(
            "device_update",
            Some("devices.manage"),
            Some(Concurrency::FieldLww),
            WRITE,
        ),
        file_op("device_delete", Some("devices.manage"), None, WRITE),
        file_op("device_rotatekey", Some("devices.manage"), None, WRITE),
    ];
}

/// `OpenAPI` fragment of this module.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(
        device_list,
        device_create,
        device_get,
        device_update,
        device_delete,
        device_rotate
    ),
    components(schemas(Device, DeviceWithKey, NewDevice)),
    tags((name = "devices", description = "Devices: registration, pairing and the data address key"))
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "devices",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
