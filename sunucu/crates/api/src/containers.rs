//! Containers, scan codes, status changes, fills, recall and labels (K2, issue #363).
//!
//! Tables are in migration `0250` (K1). A scan code is a random token. The database keeps only
//! its SHA-256 digest, so the token is shown once: on container create and on label print. A label
//! print issues a new code for each target. Old codes stay valid unless the request asks to revoke
//! them. Every change of a container writes one row in the append-only table `container_event`.
//!
//! Permissions use the existing codes: `recipe.view` reads, `stock.move` writes. The product
//! module is Production (`modules::MODULE_KEY_MAP`).
//!
//! TODO: SSE events (outbox topic `container.changed`) come with the client work.
//! TODO: label text is limited to ASCII (the base 14 PDF fonts have no embedded glyphs).

#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, NaiveDate, Utc};
use db::containers::ContainerStatus;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgConnection};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState, concurrency,
    context::RequestContext,
    error::{ApiError, ApiJson, ApiQuery, ApiResult, ErrorCode, FieldIssue},
    library::{created_response, id_cursor, issue, not_found, paging, sqlstate, workspace},
    pagination::{ListParams, Page},
    recipe_org::conflict,
};

/// Permission that reads containers, scan results and the recall list.
pub const READ_PERMISSION: &str = "recipe.view";
/// Permission that creates, changes, fills and labels containers.
pub const WRITE_PERMISSION: &str = "stock.move";

/// Largest number of targets in one label request.
const LABEL_MAX_TARGETS: usize = 100;
/// Prefix of the text inside a QR code. The scanner reads the token after the last `/`.
const QR_PREFIX: &str = "gristora://scan/";

// --- representations ----------------------------------------------------------------------

/// A reusable container (keg, cask, bottle crate and so on) with its current state.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct Container {
    /// Identifier.
    pub id: Uuid,
    /// Serial number; unique in the workspace.
    pub serial_number: String,
    /// The package type (size).
    pub package_type_id: Uuid,
    /// Name of the package type.
    pub package_type_name: String,
    /// `bottle`, `can`, `keg`, `cask`.
    pub package_kind: String,
    /// Volume of the package type in litres.
    pub volume_l: f64,
    /// `clean`, `filled`, `out`, `dirty`, `maintenance`, `retired`.
    pub status: String,
    /// The packaging run line of the last fill.
    pub last_fill_line_id: Option<Uuid>,
    /// The stock lot of the last fill (for recall).
    pub last_lot_id: Option<Uuid>,
    /// Note of the last `out` status.
    pub out_note: Option<String>,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Updated at.
    pub updated_at: DateTime<Utc>,
    /// Record version (`ETag`).
    pub version: i64,
}

const CONTAINER_SQL: &str = "SELECT c.id, c.serial_number, c.package_type_id, \
    p.name AS package_type_name, p.kind AS package_kind, p.volume_l, c.status, \
    c.last_fill_line_id, c.last_lot_id, c.out_note, c.created_at, c.updated_at, c.version \
    FROM container c JOIN package_types p ON p.id = c.package_type_id WHERE c.deleted_at IS NULL";

/// One row of the history of a container.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct ContainerEvent {
    /// Identifier.
    pub id: Uuid,
    /// `scan`, `status_changed`, `fill`, `note`.
    pub event_type: String,
    /// When it happened.
    pub occurred_at: DateTime<Utc>,
    /// The user.
    pub user_id: Option<Uuid>,
    /// Status before.
    pub old_status: Option<String>,
    /// Status after.
    pub new_status: Option<String>,
    /// Packaging run line (a fill).
    pub last_fill_line_id: Option<Uuid>,
    /// Stock lot (a fill).
    pub last_lot_id: Option<Uuid>,
    /// Note.
    pub note: Option<String>,
}

/// A container with its latest history rows.
#[derive(Debug, Serialize, ToSchema)]
pub struct ContainerDetail {
    /// The container.
    pub container: Container,
    /// The last 50 history rows, newest first.
    pub events: Vec<ContainerEvent>,
}

/// The number of containers in one status.
#[derive(Debug, Serialize, ToSchema)]
pub struct StatusCount {
    /// `clean`, `filled`, `out`, `dirty`, `maintenance`, `retired`.
    pub status: String,
    /// Number of containers.
    pub count: i64,
}

/// A page of containers with the count of each status.
#[derive(Debug, Serialize, ToSchema)]
pub struct ContainerList {
    /// The rows of this page.
    pub items: Vec<Container>,
    /// Cursor of the next page; `null` on the last page.
    pub next_cursor: Option<String>,
    /// All six statuses. The filter `status` does not change the counts.
    pub status_counts: Vec<StatusCount>,
}

/// A created container with its scan token. The token is shown only once.
#[derive(Debug, Serialize, ToSchema)]
pub struct CreatedContainer {
    /// The container.
    pub container: Container,
    /// Opaque scan token. The server keeps only a digest of it.
    pub scan_token: String,
}

/// The answer of a bulk create.
#[derive(Debug, Serialize, ToSchema)]
pub struct CreatedContainers {
    /// The new containers in the order of the request.
    pub items: Vec<CreatedContainer>,
}

/// A tank (vessel) found by a scan.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct VesselSummary {
    /// Identifier.
    pub id: Uuid,
    /// Short code (FV-1).
    pub code: String,
    /// Name.
    pub name: String,
    /// Kind of vessel.
    pub kind: String,
    /// Capacity in litres.
    pub capacity_l: f64,
    /// `dirty`, `cleaning`, `clean`, `sanitized`.
    pub cleaning_status: String,
}

/// A stock lot found by a scan.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct LotSummary {
    /// Identifier.
    pub id: Uuid,
    /// Lot code.
    pub lot_code: String,
    /// The ingredient.
    pub ingredient_id: Uuid,
    /// Name of the ingredient.
    pub ingredient_name: String,
    /// `kg`, `l`, `count`.
    pub unit: String,
    /// Amount now in stock.
    pub quantity: f64,
    /// Best-before day.
    pub expires_on: Option<NaiveDate>,
}

/// The target of a scan code and what the caller may do with it.
#[derive(Debug, Serialize, ToSchema)]
pub struct ScanResolution {
    /// `container`, `vessel` or `lot`.
    pub kind: String,
    /// The target identifier.
    pub id: Uuid,
    /// The scan code row.
    pub scan_code_id: Uuid,
    /// Text for the screen: the serial number, the tank code or the lot code.
    pub label: String,
    /// Set when `kind` is `container`.
    pub container: Option<Container>,
    /// Set when `kind` is `vessel`.
    pub vessel: Option<VesselSummary>,
    /// Set when `kind` is `lot`.
    pub lot: Option<LotSummary>,
    /// Actions the caller may do now: `view`, `change_status`, `fill`, `retire`, `recall`,
    /// `print_label`.
    pub actions: Vec<String>,
}

/// A container in a recall list.
#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct RecallContainer {
    /// Identifier.
    pub id: Uuid,
    /// Serial number.
    pub serial_number: String,
    /// Name of the package type.
    pub package_type_name: String,
    /// Volume in litres.
    pub volume_l: f64,
    /// Current status.
    pub status: String,
    /// The packaging run line of the fill.
    pub last_fill_line_id: Option<Uuid>,
    /// When the container was filled (the latest `fill` event).
    pub filled_at: Option<DateTime<Utc>>,
    /// Note of the last `out` status.
    pub out_note: Option<String>,
}

/// The result of one code in a batch scan.
#[derive(Debug, Serialize, ToSchema)]
pub struct BatchItemResult {
    /// Position of the code in the request.
    pub index: usize,
    /// The change applied (or the container already had the status).
    pub ok: bool,
    /// `true` when the container already had the status; no event is written.
    pub unchanged: bool,
    /// The container after the change.
    pub container: Option<Container>,
    /// `not_found`, `not_a_container`, `container_retired`.
    pub error: Option<String>,
}

/// The answer of a batch scan.
#[derive(Debug, Serialize, ToSchema)]
pub struct BatchStatusResult {
    /// One row for each code, in the order of the request.
    pub results: Vec<BatchItemResult>,
    /// Number of codes that changed a container.
    pub changed: usize,
    /// Number of codes with an error.
    pub failed: usize,
}

/// The answer of a fill.
#[derive(Debug, Serialize, ToSchema)]
pub struct FillResult {
    /// The filled containers.
    pub items: Vec<Container>,
}

/// A PDF file.
#[derive(Debug, ToSchema)]
#[schema(value_type = String, format = Binary)]
pub struct PdfFile(pub Vec<u8>);

// --- requests -----------------------------------------------------------------------------

/// Filter of the container list.
#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct ContainerFilter {
    /// Only this status.
    pub status: Option<String>,
    /// Only this package type.
    pub package_type_id: Option<Uuid>,
    /// Text in the serial number (case blind).
    pub q: Option<String>,
}

/// Filter of the recall list.
#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct RecallFilter {
    /// Only this status. Default: `filled` and `out` (the containers that can still hold product).
    pub status: Option<String>,
}

/// Bulk create. Give `serial_numbers`, or `prefix` and `count`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateContainers {
    /// The package type of every new container.
    pub package_type_id: Uuid,
    /// Explicit serial numbers (1 to 500, 1 to 100 characters each).
    pub serial_numbers: Option<Vec<String>>,
    /// Prefix of generated serial numbers (`KEG-`).
    pub prefix: Option<String>,
    /// Number of serial numbers to generate (1 to 500).
    pub count: Option<u32>,
    /// First number of the generated range (default 1).
    pub start_at: Option<u32>,
    /// Zero padding of the number (default 4, 1 to 9).
    pub digits: Option<u32>,
}

/// A scan code in the body.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ScanBody {
    /// The scanned text: the bare token or the text of the QR code.
    pub code: String,
}

/// A status change from a scan.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ScanStatusBody {
    /// The scanned text.
    pub code: String,
    /// New status: `clean`, `out`, `dirty`, `maintenance` or `retired`. Use fill for `filled`.
    pub status: String,
    /// Note (up to 2000 characters). It is kept as `out_note` for `out`.
    pub note: Option<String>,
}

/// A status change from many scans.
#[derive(Debug, Deserialize, ToSchema)]
pub struct BatchStatusBody {
    /// The scanned texts (1 to 500).
    pub codes: Vec<String>,
    /// New status. Same values as a single scan.
    pub status: String,
    /// Note for every container.
    pub note: Option<String>,
}

/// Retire request.
#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct RetireBody {
    /// Reason (up to 2000 characters).
    pub note: Option<String>,
}

/// Fill containers from a packaging run line.
#[derive(Debug, Deserialize, ToSchema)]
pub struct FillBody {
    /// The packaging run line.
    pub packaging_run_line_id: Uuid,
    /// The stock lot for recall.
    pub lot_id: Option<Uuid>,
    /// Containers by identifier.
    pub container_ids: Option<Vec<Uuid>>,
    /// Containers by scanned text.
    pub codes: Option<Vec<String>>,
}

/// One target of a label request.
#[derive(Debug, Deserialize, ToSchema)]
pub struct LabelTarget {
    /// `container`, `vessel` or `lot`.
    pub kind: String,
    /// The identifier.
    pub id: Uuid,
}

/// Label request. Each target gets a new scan code.
#[derive(Debug, Deserialize, ToSchema)]
pub struct LabelBody {
    /// 1 to 100 targets. One label (one PDF page) for each.
    pub targets: Vec<LabelTarget>,
    /// Revoke the older codes of each target (default `false`).
    pub revoke_previous: Option<bool>,
}

// --- helpers ------------------------------------------------------------------------------

fn container_example() -> Value {
    json!({
        "id": "0192f1c07a3b7000800000000000001a", "serial_number": "KEG-0001",
        "package_type_id": "0192f1c07a3b70008000000000000010",
        "package_type_name": "Keg 30 L", "package_kind": "keg", "volume_l": 30.0,
        "status": "clean", "last_fill_line_id": null, "last_lot_id": null, "out_note": null,
        "created_at": "2026-11-26T10:00:00Z", "updated_at": "2026-11-26T10:00:00Z", "version": 1
    })
}

fn event_example() -> Value {
    json!({
        "id": "0192f1c07a3b7000800000000000001b", "event_type": "status_changed",
        "occurred_at": "2026-11-26T10:05:00Z", "user_id": "0192f1c07a3b70008000000000000002",
        "old_status": "clean", "new_status": "dirty", "last_fill_line_id": null,
        "last_lot_id": null, "note": null
    })
}

fn counts_example() -> Value {
    json!([
        {"status": "clean", "count": 12}, {"status": "filled", "count": 5},
        {"status": "out", "count": 0}, {"status": "dirty", "count": 3},
        {"status": "maintenance", "count": 0}, {"status": "retired", "count": 1}
    ])
}

fn scan_token() -> String {
    let mut buf = [0u8; 24];
    getrandom::fill(&mut buf).unwrap_or_else(|e| panic!("operating system randomness failed: {e}"));
    URL_SAFE_NO_PAD.encode(buf)
}

/// The token inside the scanned text, and the digest the database stores.
fn code_hash(raw: &str, field: &str) -> ApiResult<Vec<u8>> {
    let raw = raw.trim();
    if raw.is_empty() || raw.chars().count() > 512 {
        return Err(issue(field, "invalid_length"));
    }
    let token = raw
        .rsplit(['/', ':'])
        .find(|part| !part.is_empty())
        .ok_or_else(|| issue(field, "invalid_value"))?;
    Ok(Sha256::digest(token.as_bytes()).to_vec())
}

fn parse_status(value: &str, field: &str) -> ApiResult<ContainerStatus> {
    ContainerStatus::parse(value).ok_or_else(|| issue(field, "invalid_value"))
}

fn check_note(note: Option<&str>) -> ApiResult<()> {
    if note.is_some_and(|n| n.chars().count() > 2000) {
        return Err(issue("/note", "invalid_length"));
    }
    Ok(())
}

fn is_not_found(e: &ApiError) -> bool {
    e.problem().code == ErrorCode::NotFound
}

fn constraint_error(e: sqlx::Error) -> ApiError {
    match sqlstate(&e).as_deref() {
        Some("23505") => conflict("/serial_number", "already_exists"),
        // The fill guard trigger of migration 0250 raises `integrity_constraint_violation`.
        Some("23000") => conflict("/status", "not_fillable"),
        Some("23514" | "22003" | "22P02") => issue("/", "constraint_violated"),
        Some("23503") => issue("/", "reference_not_found"),
        _ => e.into(),
    }
}

async fn fetch_container(tx: &mut PgConnection, id: Uuid, lock: bool) -> ApiResult<Container> {
    let lock = if lock { " FOR UPDATE OF c" } else { "" };
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{CONTAINER_SQL} AND c.id = $1{lock}"
    )))
    .bind(id)
    .fetch_optional(tx)
    .await?
    .ok_or_else(not_found)
}

async fn fetch_lot(tx: &mut PgConnection, id: Uuid) -> ApiResult<LotSummary> {
    sqlx::query_as(
        "SELECT l.id, l.lot_code, l.ingredient_id, i.name AS ingredient_name, l.unit, \
             l.quantity, l.expires_on \
         FROM stock_lots l JOIN ingredients i ON i.id = l.ingredient_id \
         WHERE l.id = $1 AND l.deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(tx)
    .await?
    .ok_or_else(not_found)
}

async fn fetch_vessel(tx: &mut PgConnection, id: Uuid) -> ApiResult<VesselSummary> {
    sqlx::query_as(
        "SELECT id, code, name, kind, capacity_l, cleaning_status FROM vessels \
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(tx)
    .await?
    .ok_or_else(not_found)
}

/// What a scan code points to.
#[derive(Debug, FromRow)]
struct ScanTarget {
    id: Uuid,
    container_id: Option<Uuid>,
    vessel_id: Option<Uuid>,
    lot_id: Option<Uuid>,
}

/// Finds an active scan code. A revoked or unknown code gives `None`.
async fn find_code(tx: &mut PgConnection, hash: &[u8]) -> ApiResult<Option<ScanTarget>> {
    Ok(sqlx::query_as(
        "SELECT id, container_id, vessel_id, lot_id FROM scan_code \
         WHERE code_hash = $1 AND revoked_at IS NULL AND deleted_at IS NULL",
    )
    .bind(hash)
    .fetch_optional(tx)
    .await?)
}

struct NewEvent<'a> {
    container: &'a Container,
    event_type: &'a str,
    scan_code_id: Option<Uuid>,
    old_status: Option<&'a str>,
    new_status: Option<&'a str>,
    line: Option<Uuid>,
    lot: Option<Uuid>,
    note: Option<&'a str>,
}

async fn insert_event(
    tx: &mut PgConnection,
    ws: Uuid,
    user: Uuid,
    e: &NewEvent<'_>,
) -> ApiResult<()> {
    sqlx::query(
        "INSERT INTO container_event (id, workspace_id, container_id, container_serial_snapshot, \
             event_type, scan_code_id, user_id, old_status, new_status, last_fill_line_id, \
             last_lot_id, note) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
    )
    .bind(Uuid::now_v7())
    .bind(ws)
    .bind(e.container.id)
    .bind(&e.container.serial_number)
    .bind(e.event_type)
    .bind(e.scan_code_id)
    .bind(user)
    .bind(e.old_status)
    .bind(e.new_status)
    .bind(e.line)
    .bind(e.lot)
    .bind(e.note)
    .execute(tx)
    .await
    .map_err(constraint_error)?;
    Ok(())
}

/// Result of a status change.
enum Changed {
    Yes(Container),
    Same(Container),
}

impl Changed {
    fn into_container(self) -> Container {
        match self {
            Self::Yes(c) | Self::Same(c) => c,
        }
    }
}

/// Changes the status of a container under a row lock. The same status is a no-op, so a retry is
/// safe. A retired container does not change. `filled` is not allowed here (use a fill).
async fn apply_status(
    tx: &mut PgConnection,
    ws: Uuid,
    user: Uuid,
    id: Uuid,
    new: ContainerStatus,
    note: Option<&str>,
    scan_code_id: Option<Uuid>,
) -> ApiResult<Changed> {
    let before = fetch_container(tx, id, true).await?;
    let old =
        ContainerStatus::parse(&before.status).ok_or_else(|| ApiError::new(ErrorCode::Internal))?;
    if old == new {
        return Ok(Changed::Same(before));
    }
    if old == ContainerStatus::Retired {
        return Err(conflict("/status", "container_retired"));
    }
    sqlx::query(
        "UPDATE container SET status = $2, out_note = CASE WHEN $2 = 'out' THEN $3 END, \
             updated_by = $4 WHERE id = $1",
    )
    .bind(id)
    .bind(new.as_str())
    .bind(note)
    .bind(user)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    insert_event(
        tx,
        ws,
        user,
        &NewEvent {
            container: &before,
            event_type: "status_changed",
            scan_code_id,
            old_status: Some(old.as_str()),
            new_status: Some(new.as_str()),
            line: None,
            lot: None,
            note,
        },
    )
    .await?;
    Ok(Changed::Yes(fetch_container(tx, id, false).await?))
}

fn ensure_not_fill(status: ContainerStatus) -> ApiResult<()> {
    if status == ContainerStatus::Filled {
        return Err(issue("/status", "use_fill"));
    }
    Ok(())
}

fn target_column(kind: &str) -> &'static str {
    match kind {
        "container" => "container_id",
        "vessel" => "vessel_id",
        _ => "lot_id",
    }
}

async fn insert_code(
    tx: &mut PgConnection,
    ws: Uuid,
    user: Uuid,
    kind: &str,
    target: Uuid,
) -> ApiResult<(Uuid, String)> {
    let (id, token) = (Uuid::now_v7(), scan_token());
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "INSERT INTO scan_code (id, workspace_id, code_hash, {}, created_by) \
         VALUES ($1, $2, $3, $4, $5)",
        target_column(kind)
    )))
    .bind(id)
    .bind(ws)
    .bind(Sha256::digest(token.as_bytes()).to_vec())
    .bind(target)
    .bind(user)
    .execute(tx)
    .await
    .map_err(constraint_error)?;
    Ok((id, token))
}

async fn status_counts(
    tx: &mut PgConnection,
    filter: &ContainerFilter,
) -> ApiResult<Vec<StatusCount>> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT c.status, count(*) FROM container c WHERE c.deleted_at IS NULL \
           AND ($1::uuid IS NULL OR c.package_type_id = $1) \
           AND ($2::text IS NULL OR strpos(lower(c.serial_number), lower($2)) > 0) \
         GROUP BY c.status",
    )
    .bind(filter.package_type_id)
    .bind(&filter.q)
    .fetch_all(tx)
    .await?;
    let map: BTreeMap<String, i64> = rows.into_iter().collect();
    Ok(ContainerStatus::ALL
        .iter()
        .map(|s| StatusCount {
            status: s.as_str().to_owned(),
            count: map.get(s.as_str()).copied().unwrap_or(0),
        })
        .collect())
}

fn actions_of(ctx: &RequestContext, status: Option<ContainerStatus>, kind: &str) -> Vec<String> {
    let mut actions = vec!["view".to_owned()];
    let write = ctx.has_permission(WRITE_PERMISSION);
    match kind {
        "container" => {
            if write && status != Some(ContainerStatus::Retired) {
                actions.push("change_status".to_owned());
                if status.is_some_and(|s| !s.blocks_fill() && s != ContainerStatus::Filled) {
                    actions.push("fill".to_owned());
                }
                actions.push("retire".to_owned());
            }
        }
        "lot" => actions.push("recall".to_owned()),
        _ => {}
    }
    if write {
        actions.push("print_label".to_owned());
    }
    actions
}

// --- endpoints ----------------------------------------------------------------------------

/// Resolves a scanned code to its target and the actions the caller may do.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/scan/resolve",
    tag = "containers",
    operation_id = "scan_resolve",
    params(("workspace_id" = Uuid, Path, description = "Workspace")),
    request_body = ScanBody,
    responses((status = 200, description = "The target of the code", body = ScanResolution,
        example = json!({
            "kind": "container", "id": "0192f1c07a3b7000800000000000001a",
            "scan_code_id": "0192f1c07a3b7000800000000000001c", "label": "KEG-0001",
            "container": container_example(), "vessel": null, "lot": null,
            "actions": ["view", "change_status", "fill", "retire", "print_label"]
        })))
)]
pub async fn scan_resolve(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<ScanBody>,
) -> ApiResult<Json<ScanResolution>> {
    ctx.require(READ_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let hash = code_hash(&body.code, "/code")?;
    let mut tx = ctx.begin(&state.db).await?;
    let target = find_code(&mut tx, &hash).await?.ok_or_else(not_found)?;
    let mut out = ScanResolution {
        kind: String::new(),
        id: Uuid::nil(),
        scan_code_id: target.id,
        label: String::new(),
        container: None,
        vessel: None,
        lot: None,
        actions: Vec::new(),
    };
    if let Some(id) = target.container_id {
        let container = fetch_container(&mut tx, id, false).await?;
        if ctx.has_permission(WRITE_PERMISSION) {
            insert_event(
                &mut tx,
                ws,
                ctx.identity.user_id,
                &NewEvent {
                    container: &container,
                    event_type: "scan",
                    scan_code_id: Some(target.id),
                    old_status: None,
                    new_status: None,
                    line: None,
                    lot: None,
                    note: None,
                },
            )
            .await?;
        }
        "container".clone_into(&mut out.kind);
        out.id = id;
        out.label.clone_from(&container.serial_number);
        out.actions = actions_of(&ctx, ContainerStatus::parse(&container.status), "container");
        out.container = Some(container);
    } else if let Some(id) = target.vessel_id {
        let vessel = fetch_vessel(&mut tx, id).await?;
        "vessel".clone_into(&mut out.kind);
        out.id = id;
        out.label.clone_from(&vessel.code);
        out.actions = actions_of(&ctx, None, "vessel");
        out.vessel = Some(vessel);
    } else if let Some(id) = target.lot_id {
        let lot = fetch_lot(&mut tx, id).await?;
        "lot".clone_into(&mut out.kind);
        out.id = id;
        out.label.clone_from(&lot.lot_code);
        out.actions = actions_of(&ctx, None, "lot");
        out.lot = Some(lot);
    } else {
        return Err(not_found());
    }
    tx.commit().await?;
    Ok(Json(out))
}

/// Lists containers with the count of each status.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/containers",
    tag = "containers",
    operation_id = "container_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("status" = Option<String>, Query, description = "Only this status"),
        ("package_type_id" = Option<Uuid>, Query, description = "Only this package type"),
        ("q" = Option<String>, Query, description = "Text in the serial number (case blind)"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page"),
        ("sort" = Option<String>, Query, description = "`id` (creation order) or `-id` (default)")
    ),
    responses((status = 200, description = "A page of containers", body = ContainerList,
        example = json!({"items": [container_example()], "next_cursor": null,
            "status_counts": counts_example()})))
)]
pub async fn container_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<ContainerFilter>,
) -> ApiResult<Json<ContainerList>> {
    ctx.require(READ_PERMISSION)?;
    if let Some(status) = &filter.status {
        parse_status(status, "status")?;
    }
    let paging = paging(&params, &["id"], "-id")?;
    let (cmp, dir) = paging.dir();
    let sql = sqlx::AssertSqlSafe(format!(
        "{CONTAINER_SQL} AND ($1::text IS NULL OR c.status = $1) \
           AND ($2::uuid IS NULL OR c.package_type_id = $2) \
           AND ($3::text IS NULL OR strpos(lower(c.serial_number), lower($3)) > 0) \
           AND ($4::uuid IS NULL OR c.id {cmp} $4) \
         ORDER BY c.id {dir} LIMIT $5"
    ));
    let mut tx = ctx.begin(&state.db).await?;
    let rows: Vec<Container> = sqlx::query_as(sql)
        .bind(&filter.status)
        .bind(filter.package_type_id)
        .bind(&filter.q)
        .bind(paging.id)
        .bind(i64::from(paging.limit) + 1)
        .fetch_all(&mut *tx)
        .await?;
    let counts = status_counts(&mut tx, &filter).await?;
    let page = Page::from_rows(rows, paging.limit, &paging.sort, |r| id_cursor(r.id));
    Ok(Json(ContainerList {
        items: page.items,
        next_cursor: page.next_cursor,
        status_counts: counts,
    }))
}

/// Reads one container with its latest history.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/containers/{id}",
    tag = "containers",
    operation_id = "container_get",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Container")
    ),
    responses(
        (status = 200, description = "The container", body = ContainerDetail,
            example = json!({"container": container_example(), "events": [event_example()]})),
        (status = 304, description = "Not modified (`If-None-Match` matched)")
    )
)]
pub async fn container_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    ctx.require(READ_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let container = fetch_container(&mut tx, id, false).await?;
    let events: Vec<ContainerEvent> = sqlx::query_as(
        "SELECT id, event_type, occurred_at, user_id, old_status, new_status, last_fill_line_id, \
             last_lot_id, note FROM container_event WHERE container_id = $1 \
         ORDER BY occurred_at DESC, id DESC LIMIT 50",
    )
    .bind(id)
    .fetch_all(&mut *tx)
    .await?;
    let version = container.version;
    Ok(concurrency::conditional_get(
        &headers,
        version,
        &ContainerDetail { container, events },
    ))
}

fn serials_of(body: &CreateContainers) -> ApiResult<Vec<String>> {
    let serials: Vec<String> = match (&body.serial_numbers, &body.prefix, body.count) {
        (Some(list), None, None) => list.iter().map(|s| s.trim().to_owned()).collect(),
        (None, prefix, Some(count)) => {
            let prefix = prefix.as_deref().unwrap_or_default();
            let digits = body.digits.unwrap_or(4);
            if !(1..=9).contains(&digits) {
                return Err(issue("/digits", "out_of_range"));
            }
            let start = u64::from(body.start_at.unwrap_or(1));
            crate::ensure_bulk_len("/count", count as usize)?;
            (0..u64::from(count))
                .map(|i| format!("{prefix}{:0width$}", start + i, width = digits as usize))
                .collect()
        }
        _ => return Err(issue("/serial_numbers", "give_list_or_prefix_and_count")),
    };
    crate::ensure_bulk_len("/serial_numbers", serials.len())?;
    let mut seen = BTreeSet::new();
    for (i, s) in serials.iter().enumerate() {
        if s.is_empty() || s.chars().count() > 100 {
            return Err(issue(&format!("/serial_numbers/{i}"), "invalid_length"));
        }
        if !seen.insert(s.as_str()) {
            return Err(issue(&format!("/serial_numbers/{i}"), "duplicate"));
        }
    }
    Ok(serials)
}

/// Creates many containers with serial numbers. Each container gets a scan code. The tokens are
/// in the answer only.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/containers",
    tag = "containers",
    operation_id = "container_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = CreateContainers,
    responses((status = 201, description = "The new containers and their scan tokens",
        body = CreatedContainers,
        example = json!({"items": [{"container": container_example(),
            "scan_token": "Zk3b0Qm1x7pYtq2D8aVnLw5H9cRjUeSf"}]})))
)]
pub async fn container_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<CreateContainers>,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let user = ctx.identity.user_id;
    let serials = serials_of(&body)?;
    let mut tx = ctx.begin(&state.db).await?;
    let exists: Option<bool> =
        sqlx::query_scalar("SELECT true FROM package_types WHERE id = $1 AND deleted_at IS NULL")
            .bind(body.package_type_id)
            .fetch_optional(&mut *tx)
            .await?;
    if exists.is_none() {
        return Err(issue("/package_type_id", "not_found"));
    }
    let mut items = Vec::with_capacity(serials.len());
    for (i, serial) in serials.iter().enumerate() {
        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO container (id, workspace_id, serial_number, package_type_id, created_by) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(id)
        .bind(ws)
        .bind(serial)
        .bind(body.package_type_id)
        .bind(user)
        .execute(&mut *tx)
        .await
        .map_err(|e| match sqlstate(&e).as_deref() {
            Some("23505") => ApiError::new(ErrorCode::Conflict).with_errors(vec![
                FieldIssue::new(format!("/serial_numbers/{i}"), "already_exists")
                    .with_params(json!({"serial_number": serial})),
            ]),
            _ => constraint_error(e),
        })?;
        let (_, token) = insert_code(&mut tx, ws, user, "container", id).await?;
        items.push(CreatedContainer {
            container: fetch_container(&mut tx, id, false).await?,
            scan_token: token,
        });
    }
    tx.commit().await?;
    Ok(created_response(
        &format!("/v1/workspaces/{ws}/containers"),
        1,
        &CreatedContainers { items },
    ))
}

/// Retires a container for good. A retired container cannot be filled or changed. Repeat is safe.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/containers/{id}/retire",
    tag = "containers",
    operation_id = "container_retire",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Container")
    ),
    request_body = RetireBody,
    responses((status = 200, description = "The retired container", body = Container,
        example = json!(container_example())))
)]
pub async fn container_retire(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    ApiJson(body): ApiJson<RetireBody>,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    check_note(body.note.as_deref())?;
    let mut tx = ctx.begin(&state.db).await?;
    let result = apply_status(
        &mut tx,
        ws,
        ctx.identity.user_id,
        id,
        ContainerStatus::Retired,
        body.note.as_deref(),
        None,
    )
    .await?;
    tx.commit().await?;
    let c = result.into_container();
    Ok(concurrency::record_response(StatusCode::OK, c.version, &c))
}

/// Changes the status of the container that a scanned code points to.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/scan/status",
    tag = "containers",
    operation_id = "scan_status",
    params(("workspace_id" = Uuid, Path, description = "Workspace")),
    request_body = ScanStatusBody,
    responses((status = 200, description = "The container after the change", body = Container,
        example = json!(container_example())))
)]
pub async fn scan_status(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<ScanStatusBody>,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let status = parse_status(&body.status, "/status")?;
    ensure_not_fill(status)?;
    check_note(body.note.as_deref())?;
    let hash = code_hash(&body.code, "/code")?;
    let mut tx = ctx.begin(&state.db).await?;
    let target = find_code(&mut tx, &hash).await?.ok_or_else(not_found)?;
    let id = target
        .container_id
        .ok_or_else(|| issue("/code", "not_a_container"))?;
    let result = apply_status(
        &mut tx,
        ws,
        ctx.identity.user_id,
        id,
        status,
        body.note.as_deref(),
        Some(target.id),
    )
    .await?;
    tx.commit().await?;
    let c = result.into_container();
    Ok(concurrency::record_response(StatusCode::OK, c.version, &c))
}

/// Changes the status of many containers from many scans. Each code has its own result. A bad
/// code does not stop the others.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/scan/status-batch",
    tag = "containers",
    operation_id = "scan_status_batch",
    params(("workspace_id" = Uuid, Path, description = "Workspace")),
    request_body = BatchStatusBody,
    responses((status = 200, description = "One result for each code", body = BatchStatusResult,
        example = json!({"results": [
            {"index": 0, "ok": true, "unchanged": false, "container": container_example(),
             "error": null},
            {"index": 1, "ok": false, "unchanged": false, "container": null,
             "error": "not_found"}],
            "changed": 1, "failed": 1})))
)]
pub async fn scan_status_batch(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<BatchStatusBody>,
) -> ApiResult<Json<BatchStatusResult>> {
    ctx.require(WRITE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let status = parse_status(&body.status, "/status")?;
    ensure_not_fill(status)?;
    check_note(body.note.as_deref())?;
    crate::ensure_bulk_len("/codes", body.codes.len())?;
    let hashes = body
        .codes
        .iter()
        .enumerate()
        .map(|(i, c)| code_hash(c, &format!("/codes/{i}")))
        .collect::<ApiResult<Vec<_>>>()?;
    let mut tx = ctx.begin(&state.db).await?;
    let mut results = Vec::with_capacity(hashes.len());
    for (index, hash) in hashes.iter().enumerate() {
        let failed = |error: &str| BatchItemResult {
            index,
            ok: false,
            unchanged: false,
            container: None,
            error: Some(error.to_owned()),
        };
        let Some(target) = find_code(&mut tx, hash).await? else {
            results.push(failed("not_found"));
            continue;
        };
        let Some(id) = target.container_id else {
            results.push(failed("not_a_container"));
            continue;
        };
        match apply_status(
            &mut tx,
            ws,
            ctx.identity.user_id,
            id,
            status,
            body.note.as_deref(),
            Some(target.id),
        )
        .await
        {
            Ok(changed) => {
                let unchanged = matches!(changed, Changed::Same(_));
                results.push(BatchItemResult {
                    index,
                    ok: true,
                    unchanged,
                    container: Some(changed.into_container()),
                    error: None,
                });
            }
            Err(e) if e.errors().iter().any(|i| i.code == "container_retired") => {
                results.push(failed("container_retired"));
            }
            Err(e) if is_not_found(&e) => results.push(failed("not_found")),
            Err(e) => return Err(e),
        }
    }
    tx.commit().await?;
    let changed = results.iter().filter(|r| r.ok && !r.unchanged).count();
    let failed = results.iter().filter(|r| !r.ok).count();
    Ok(Json(BatchStatusResult {
        results,
        changed,
        failed,
    }))
}

#[derive(Debug, FromRow)]
struct LineRow {
    package_type_id: Uuid,
    quantity: i32,
}

/// Fills containers from a packaging run line. The whole request fails with `409` when one
/// container is dirty, in maintenance, retired, already filled or of another package type.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/containers/fill",
    tag = "containers",
    operation_id = "container_fill",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = FillBody,
    responses((status = 200, description = "The filled containers", body = FillResult,
        example = json!({"items": [container_example()]})))
)]
pub async fn container_fill(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<FillBody>,
) -> ApiResult<Json<FillResult>> {
    ctx.require(WRITE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let user = ctx.identity.user_id;
    let mut ids: Vec<Uuid> = body.container_ids.clone().unwrap_or_default();
    let mut tx = ctx.begin(&state.db).await?;
    for (i, raw) in body.codes.iter().flatten().enumerate() {
        let hash = code_hash(raw, &format!("/codes/{i}"))?;
        let target = find_code(&mut tx, &hash).await?.ok_or_else(not_found)?;
        ids.push(
            target
                .container_id
                .ok_or_else(|| issue(&format!("/codes/{i}"), "not_a_container"))?,
        );
    }
    let mut seen = BTreeSet::new();
    ids.retain(|id| seen.insert(*id));
    crate::ensure_bulk_len("/container_ids", ids.len())?;
    let line: LineRow = sqlx::query_as(
        "SELECT package_type_id, quantity FROM packaging_run_lines \
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(body.packaging_run_line_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| issue("/packaging_run_line_id", "not_found"))?;
    if let Some(lot) = body.lot_id {
        fetch_lot(&mut tx, lot)
            .await
            .map_err(|_| issue("/lot_id", "not_found"))?;
    }
    let rows: Vec<Container> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{CONTAINER_SQL} AND c.id = ANY($1) ORDER BY c.id FOR UPDATE OF c"
    )))
    .bind(&ids)
    .fetch_all(&mut *tx)
    .await?;
    if rows.len() != ids.len() {
        return Err(not_found());
    }
    let mut problems = Vec::new();
    for c in &rows {
        let status = ContainerStatus::parse(&c.status);
        let code = if status.is_none_or(ContainerStatus::blocks_fill) {
            Some("not_fillable")
        } else if status == Some(ContainerStatus::Filled) {
            Some("already_filled")
        } else if c.package_type_id != line.package_type_id {
            Some("package_type_mismatch")
        } else {
            None
        };
        if let Some(code) = code {
            problems.push(FieldIssue::new("/container_ids", code).with_params(json!({
                "container_id": c.id, "serial_number": c.serial_number, "status": c.status
            })));
        }
    }
    if !problems.is_empty() {
        return Err(ApiError::new(ErrorCode::Conflict).with_errors(problems));
    }
    let already: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM container WHERE last_fill_line_id = $1 \
           AND status IN ('filled', 'out') AND deleted_at IS NULL",
    )
    .bind(body.packaging_run_line_id)
    .fetch_one(&mut *tx)
    .await?;
    if already + i64::try_from(ids.len()).unwrap_or(i64::MAX) > i64::from(line.quantity) {
        return Err(
            conflict("/container_ids", "line_quantity_exceeded").with_current(json!({
                "line_quantity": line.quantity, "already_filled": already
            })),
        );
    }
    for c in &rows {
        sqlx::query(
            "UPDATE container SET status = 'filled', last_fill_line_id = $2, last_lot_id = $3, \
                 out_note = NULL, updated_by = $4 WHERE id = $1",
        )
        .bind(c.id)
        .bind(body.packaging_run_line_id)
        .bind(body.lot_id)
        .bind(user)
        .execute(&mut *tx)
        .await
        .map_err(constraint_error)?;
        insert_event(
            &mut tx,
            ws,
            user,
            &NewEvent {
                container: c,
                event_type: "fill",
                scan_code_id: None,
                old_status: Some(&c.status),
                new_status: Some("filled"),
                line: Some(body.packaging_run_line_id),
                lot: body.lot_id,
                note: None,
            },
        )
        .await?;
    }
    let mut items = Vec::with_capacity(ids.len());
    for id in &ids {
        items.push(fetch_container(&mut tx, *id, false).await?);
    }
    tx.commit().await?;
    Ok(Json(FillResult { items }))
}

/// Recall query: the containers filled from a stock lot.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/recall/lots/{lot_id}/containers",
    tag = "containers",
    operation_id = "recall_containers",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("lot_id" = Uuid, Path, description = "Stock lot"),
        ("status" = Option<String>, Query,
            description = "Only this status (default: `filled` and `out`)"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page"),
        ("sort" = Option<String>, Query, description = "`id` (creation order, default) or `-id`")
    ),
    responses((status = 200, description = "A page of containers", body = Page<RecallContainer>,
        example = json!({"items": [{
            "id": "0192f1c07a3b7000800000000000001a", "serial_number": "KEG-0001",
            "package_type_name": "Keg 30 L", "volume_l": 30.0, "status": "filled",
            "last_fill_line_id": "0192f1c07a3b7000800000000000001d",
            "filled_at": "2026-11-26T11:00:00Z", "out_note": null}],
            "next_cursor": null})))
)]
pub async fn recall_containers(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, lot_id)): Path<(Uuid, Uuid)>,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<RecallFilter>,
) -> ApiResult<Json<Page<RecallContainer>>> {
    ctx.require(READ_PERMISSION)?;
    if let Some(status) = &filter.status {
        parse_status(status, "status")?;
    }
    let paging = paging(&params, &["id"], "id")?;
    let (cmp, dir) = paging.dir();
    let sql = sqlx::AssertSqlSafe(format!(
        "SELECT c.id, c.serial_number, p.name AS package_type_name, p.volume_l, c.status, \
             c.last_fill_line_id, c.out_note, \
             (SELECT max(e.occurred_at) FROM container_event e \
               WHERE e.container_id = c.id AND e.event_type = 'fill') AS filled_at \
         FROM container c JOIN package_types p ON p.id = c.package_type_id \
         WHERE c.deleted_at IS NULL AND c.last_lot_id = $1 \
           AND (CASE WHEN $2::text IS NULL THEN c.status IN ('filled', 'out') \
                     ELSE c.status = $2 END) \
           AND ($3::uuid IS NULL OR c.id {cmp} $3) \
         ORDER BY c.id {dir} LIMIT $4"
    ));
    let mut tx = ctx.begin(&state.db).await?;
    fetch_lot(&mut tx, lot_id).await?;
    let rows: Vec<RecallContainer> = sqlx::query_as(sql)
        .bind(lot_id)
        .bind(&filter.status)
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

// --- labels -------------------------------------------------------------------------------

/// The text of one label.
struct LabelText {
    caption: &'static str,
    title: String,
    lines: Vec<String>,
}

fn trim_number(value: f64) -> String {
    let text = format!("{value:.2}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

async fn label_text(tx: &mut PgConnection, kind: &str, id: Uuid) -> ApiResult<LabelText> {
    match kind {
        "container" => {
            let c = fetch_container(tx, id, false).await?;
            Ok(LabelText {
                caption: "CONTAINER",
                title: c.serial_number,
                lines: vec![
                    format!("{} L", trim_number(c.volume_l)),
                    format!("{} ({})", c.package_type_name, c.package_kind),
                ],
            })
        }
        "vessel" => {
            let v = fetch_vessel(tx, id).await?;
            Ok(LabelText {
                caption: "TANK",
                title: v.code,
                lines: vec![format!("{} L", trim_number(v.capacity_l)), v.name, v.kind],
            })
        }
        "lot" => {
            let l = fetch_lot(tx, id).await?;
            let mut lines = vec![l.ingredient_name];
            if let Some(day) = l.expires_on {
                lines.push(format!("Best before {day}"));
            }
            Ok(LabelText {
                caption: "LOT",
                title: l.lot_code,
                lines,
            })
        }
        _ => Err(issue("/targets", "invalid_value")),
    }
}

/// Folds text to ASCII for the base PDF fonts. Other characters become `?`.
fn ascii(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| {
            if c.is_ascii() && !c.is_ascii_control() {
                c as u8
            } else {
                b'?'
            }
        })
        .collect()
}

/// Draws one page of 90 x 50 mm: the QR code on the left, the text on the right.
#[allow(clippy::cast_precision_loss)]
fn label_page(qr: &qrcode::QrCode, text: &LabelText) -> pdf_writer::Content {
    use pdf_writer::{Content, Name, Str};
    const H: f32 = 141.7;
    const QUIET: usize = 2;
    let width = qr.width();
    let side = 113.4; // 40 mm
    let module = side / (width + 2 * QUIET) as f32;
    let (x0, y0) = (14.0, (H - side) / 2.0);
    let mut c = Content::new();
    c.set_fill_rgb(0.0, 0.0, 0.0);
    let colors = qr.to_colors();
    for row in 0..width {
        for col in 0..width {
            if colors[row * width + col] == qrcode::Color::Dark {
                let x = x0 + (col + QUIET) as f32 * module;
                let y = y0 + side - (row + QUIET + 1) as f32 * module;
                c.rect(x, y, module, module);
            }
        }
    }
    c.fill_nonzero();
    let tx = x0 + side + 12.0;
    let avail = 255.1 - tx - 10.0;
    let title = ascii(&text.title);
    let font_size = (avail / (title.len().max(1) as f32 * 0.6)).clamp(8.0, 20.0);
    c.begin_text();
    c.set_font(Name(b"F1"), 7.0).next_line(tx, H - 34.0);
    c.show(Str(text.caption.as_bytes()));
    c.set_font(Name(b"F2"), font_size)
        .next_line(0.0, -(font_size + 4.0));
    c.show(Str(&title));
    c.set_font(Name(b"F1"), 10.0).next_line(0.0, -16.0);
    for (i, line) in text.lines.iter().enumerate() {
        if i > 0 {
            c.next_line(0.0, -13.0);
        }
        c.show(Str(&ascii(line)));
    }
    c.end_text();
    c
}

fn label_pdf(labels: &[(String, LabelText)]) -> ApiResult<Vec<u8>> {
    use pdf_writer::{Finish, Name, Pdf, Rect, Ref};
    let (catalog, tree, regular, bold) = (Ref::new(1), Ref::new(2), Ref::new(3), Ref::new(4));
    let mut pdf = Pdf::new();
    pdf.catalog(catalog).pages(tree);
    let slot = |i: usize| i32::try_from(i * 2).unwrap_or(0);
    let page_ids: Vec<Ref> = (0..labels.len()).map(|i| Ref::new(5 + slot(i))).collect();
    pdf.pages(tree)
        .kids(page_ids.iter().copied())
        .count(i32::try_from(labels.len()).unwrap_or(0));
    pdf.type1_font(regular).base_font(Name(b"Helvetica"));
    pdf.type1_font(bold).base_font(Name(b"Helvetica-Bold"));
    for (i, (payload, text)) in labels.iter().enumerate() {
        let qr = qrcode::QrCode::with_error_correction_level(payload, qrcode::EcLevel::M)
            .map_err(|_| ApiError::new(ErrorCode::Internal))?;
        let content_id = Ref::new(6 + slot(i));
        let mut page = pdf.page(page_ids[i]);
        page.media_box(Rect::new(0.0, 0.0, 255.1, 141.7));
        page.parent(tree);
        page.contents(content_id);
        let mut resources = page.resources();
        let mut fonts = resources.fonts();
        fonts.pair(Name(b"F1"), regular);
        fonts.pair(Name(b"F2"), bold);
        fonts.finish();
        resources.finish();
        page.finish();
        let content = label_page(&qr, text);
        pdf.stream(content_id, &content.finish());
    }
    Ok(pdf.finish())
}

/// Prints labels. Each target gets a new scan code whose token is in the QR code. The answer is
/// one PDF with one page for each target. Every call issues new codes.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/labels",
    tag = "containers",
    operation_id = "label_print",
    params(("workspace_id" = Uuid, Path, description = "Workspace")),
    request_body = LabelBody,
    responses((status = 200, description = "A PDF file, one page for each target",
        body = PdfFile, content_type = "application/pdf", example = json!("%PDF-1.7")))
)]
pub async fn label_print(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<LabelBody>,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let user = ctx.identity.user_id;
    if body.targets.is_empty() || body.targets.len() > LABEL_MAX_TARGETS {
        return Err(ApiError::validation(vec![
            FieldIssue::new("/targets", "too_many_items")
                .with_params(json!({"min": 1, "max": LABEL_MAX_TARGETS})),
        ]));
    }
    let mut tx = ctx.begin(&state.db).await?;
    let mut labels = Vec::with_capacity(body.targets.len());
    for (i, t) in body.targets.iter().enumerate() {
        let text = label_text(&mut tx, &t.kind, t.id).await.map_err(|e| {
            if is_not_found(&e) {
                issue(&format!("/targets/{i}/id"), "not_found")
            } else {
                e
            }
        })?;
        let (code_id, token) = insert_code(&mut tx, ws, user, &t.kind, t.id).await?;
        if body.revoke_previous == Some(true) {
            sqlx::query(sqlx::AssertSqlSafe(format!(
                "UPDATE scan_code SET revoked_at = now(), updated_by = $3 \
                 WHERE {} = $1 AND id <> $2 AND revoked_at IS NULL",
                target_column(&t.kind)
            )))
            .bind(t.id)
            .bind(code_id)
            .bind(user)
            .execute(&mut *tx)
            .await?;
        }
        labels.push((format!("{QR_PREFIX}{token}"), text));
    }
    let pdf = label_pdf(&labels)?;
    tx.commit().await?;
    let mut response = (StatusCode::OK, pdf).into_response();
    let h = response.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    h.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("inline; filename=\"labels.pdf\""),
    );
    h.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    Ok(response)
}

// --- routes -------------------------------------------------------------------------------

/// The routes of the module.
pub fn routes(state: &AppState) -> Router<AppState> {
    const WS: &str = "/v1/workspaces/{ws}";
    let creates = crate::idempotent(
        state,
        Router::new()
            .route(&format!("{WS}/containers"), post(container_create))
            .route(&format!("{WS}/containers/fill"), post(container_fill)),
    );
    let others = Router::new()
        .route(&format!("{WS}/containers"), get(container_list))
        .route(&format!("{WS}/containers/{{id}}"), get(container_get))
        .route(
            &format!("{WS}/containers/{{id}}/retire"),
            post(container_retire),
        )
        .route(&format!("{WS}/scan/resolve"), post(scan_resolve))
        .route(&format!("{WS}/scan/status"), post(scan_status))
        .route(&format!("{WS}/scan/status-batch"), post(scan_status_batch))
        .route(
            &format!("{WS}/recall/lots/{{lot_id}}/containers"),
            get(recall_containers),
        )
        .route(&format!("{WS}/labels"), post(label_print));
    crate::authenticated(state, creates.merge(others))
}

// --- registry entry ----------------------------------------------------------------------

mod spec {
    use crate::concurrency::Concurrency;
    use crate::error::ErrorCode;
    use crate::error::ErrorCode::{
        Conflict, Forbidden, IdempotencyInProgress, IdempotencyMismatch, Internal,
        MalformedRequest, NotFound, PayloadTooLarge, RateLimited, Unauthenticated, Unavailable,
        UnsupportedMediaType, UpgradeRequired, ValidationFailed,
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
        PayloadTooLarge,
        UnsupportedMediaType,
        ValidationFailed,
        RateLimited,
        UpgradeRequired,
        Internal,
        Unavailable,
    ];
    const CREATE: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        Conflict,
        IdempotencyInProgress,
        PayloadTooLarge,
        UnsupportedMediaType,
        ValidationFailed,
        IdempotencyMismatch,
        RateLimited,
        UpgradeRequired,
        Internal,
        Unavailable,
    ];

    pub(super) const OPERATIONS: &[OperationSpec] = &[
        file_op(
            "scan_resolve",
            Some("recipe.view"),
            Some(Concurrency::Append),
            WRITE,
        ),
        file_op("container_list", Some("recipe.view"), None, READ),
        file_op("container_get", Some("recipe.view"), None, READ),
        file_op(
            "container_create",
            Some("stock.move"),
            Some(Concurrency::Append),
            CREATE,
        ),
        file_op(
            "container_retire",
            Some("stock.move"),
            Some(Concurrency::Append),
            WRITE,
        ),
        file_op(
            "scan_status",
            Some("stock.move"),
            Some(Concurrency::Append),
            WRITE,
        ),
        file_op(
            "scan_status_batch",
            Some("stock.move"),
            Some(Concurrency::Append),
            WRITE,
        ),
        file_op(
            "container_fill",
            Some("stock.move"),
            Some(Concurrency::Append),
            CREATE,
        ),
        file_op("recall_containers", Some("recipe.view"), None, READ),
        file_op(
            "label_print",
            Some("stock.move"),
            Some(Concurrency::Append),
            WRITE,
        ),
    ];
}

/// `OpenAPI` fragment of this module.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(
        scan_resolve,
        container_list,
        container_get,
        container_create,
        container_retire,
        scan_status,
        scan_status_batch,
        container_fill,
        recall_containers,
        label_print
    ),
    components(schemas(
        Container,
        ContainerEvent,
        ContainerDetail,
        StatusCount,
        ContainerList,
        CreatedContainer,
        CreatedContainers,
        VesselSummary,
        LotSummary,
        ScanResolution,
        RecallContainer,
        BatchItemResult,
        BatchStatusResult,
        FillResult,
        PdfFile,
        ContainerFilter,
        RecallFilter,
        CreateContainers,
        ScanBody,
        ScanStatusBody,
        BatchStatusBody,
        RetireBody,
        FillBody,
        LabelTarget,
        LabelBody
    )),
    tags((name = "containers", description = "Containers, scan codes, status changes, fills, recall and labels"))
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "containers",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
