//! Batches (T2.22, ADR-0007 section 3 "Parti ve demleme günü").
//!
//! A batch starts from an `approved` recipe version (also in Home mode; the database trigger
//! `batches_check_version` enforces it). The server writes the steps of the brew day from the
//! recipe. The client records the actual amounts, the step times and the measurements. A
//! measurement carries a client UUID: the same measurement sent two times gives one record.
//! A measurement may name a brew day package (`brew_day.rs`); the server then checks the package.
//!
//! TODO: the actual ABV comes from an `abv_pct` measurement. Calculate it from OG and FG with the
//! engine when the engine has an ABV call for measured values.

#![allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{FromRow, PgConnection};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    brew_day::{
        PackageProblem, check_package, conditioning_days, day_start, fermentation_days, plan_steps,
        schedule,
    },
    concurrency::{self, Concurrency, check_if_match},
    context::RequestContext,
    error::{ApiError, ApiJson, ApiQuery, ApiResult, ErrorCode, FieldIssue, MergePatch},
    library::{created_response, id_cursor, issue, merge, not_found, paging, workspace},
    pagination::{ListParams, Page},
    recipe_org::conflict,
    recipes::version_json,
};

/// Permission that plans batches.
pub const PLAN_PERMISSION: &str = "batch.plan";
/// Permission that records brewing data.
pub const RECORD_PERMISSION: &str = "batch.record";
/// Permission that reads batches.
pub const READ_PERMISSION: &str = "recipe.view";

const STAGES: &[&str] = &[
    "planned",
    "brewing",
    "fermenting",
    "conditioning",
    "packaging",
    "completed",
    "cancelled",
];
const LINE_KINDS: &[&str] = &["fermentable", "hop", "yeast", "misc", "water_addition"];
/// A deviation of an actual amount from this share of the plan (percent) opens a deviation record.
const DEVIATION_LIMIT_PCT: f64 = 5.0;

// --- representations ----------------------------------------------------------------------

/// A batch.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct Batch {
    /// Identifier.
    pub id: Uuid,
    /// Number for people, unique in the workspace (`B-0001`).
    pub code: String,
    /// Name.
    pub name: String,
    /// The recipe version of the batch (published, immutable).
    pub recipe_version_id: Uuid,
    /// The recipe.
    pub recipe_id: Uuid,
    /// Number of the recipe version.
    pub recipe_version_number: i32,
    /// Equipment.
    pub equipment_id: Option<Uuid>,
    /// `planned`, `brewing`, `fermenting`, `conditioning`, `packaging`, `completed`, `cancelled`.
    pub stage: String,
    /// Planned brew day.
    pub planned_date: Option<NaiveDate>,
    /// Brewing started at.
    pub started_at: Option<DateTime<Utc>>,
    /// Completed or cancelled at.
    pub completed_at: Option<DateTime<Utc>>,
    /// Target volume, L.
    pub target_volume_l: Option<f64>,
    /// Actual volume, L.
    pub actual_volume_l: Option<f64>,
    /// Engine version of the recipe calculation.
    pub engine_version: Option<String>,
    /// Notes.
    pub notes: Option<String>,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Updated at.
    pub updated_at: DateTime<Utc>,
    /// Deleted at (soft delete).
    pub deleted_at: Option<DateTime<Utc>>,
    /// Record version (`ETag`).
    pub version: i64,
}

const BATCH_COLUMNS: &str = "b.id, b.code, b.name, b.recipe_version_id, rv.recipe_id, \
    rv.number AS recipe_version_number, b.equipment_id, b.stage, b.planned_date, b.started_at, \
    b.completed_at, b.target_volume_l, b.actual_volume_l, b.engine_version, b.notes, \
    b.created_at, b.updated_at, b.deleted_at, b.version";

const BATCH_FROM: &str = "batches b JOIN recipe_versions rv \
    ON rv.workspace_id = b.workspace_id AND rv.id = b.recipe_version_id";

/// A step of the brew day.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct BatchStep {
    /// Identifier.
    pub id: Uuid,
    /// The batch.
    pub batch_id: Uuid,
    /// Position in the batch.
    pub position: i32,
    /// `prepare`, `mash_step`, `boil`, `cool`, `pitch`, `ferment`, `package`.
    pub kind: String,
    /// Name.
    pub name: String,
    /// Planned start.
    pub planned_start_at: Option<DateTime<Utc>>,
    /// Planned end.
    pub planned_end_at: Option<DateTime<Utc>>,
    /// Actual start.
    pub actual_start_at: Option<DateTime<Utc>>,
    /// Actual end.
    pub actual_end_at: Option<DateTime<Utc>>,
    /// `pending`, `running`, `done`, `skipped`.
    pub status: String,
    /// `timer`, `manual`, `paper`.
    pub input_source: Option<String>,
    /// Record version (`ETag`).
    pub version: i64,
}

const STEP_COLUMNS: &str = "id, batch_id, position, kind, name, planned_start_at, planned_end_at, \
    actual_start_at, actual_end_at, status, input_source, version";

/// An actual amount of an ingredient line, with the plan and the deviation.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct Actual {
    /// Identifier.
    pub id: Uuid,
    /// `fermentable`, `hop`, `yeast`, `misc`, `water_addition` (empty for an unplanned ingredient).
    pub line_kind: Option<String>,
    /// The recipe line.
    pub line_id: Option<Uuid>,
    /// Name of the recipe line.
    pub line_name: Option<String>,
    /// Ingredient that replaces the line, or an unplanned ingredient.
    pub substitute_ingredient_id: Option<Uuid>,
    /// Planned amount of the line (kg; L for a liquid misc).
    pub planned_amount_kg: Option<f64>,
    /// Actual amount.
    pub actual_amount_kg: f64,
    /// Actual minus plan.
    pub deviation_kg: Option<f64>,
    /// Deviation in percent of the plan.
    pub deviation_pct: Option<f64>,
    /// Notes.
    pub notes: Option<String>,
    /// Updated at.
    pub updated_at: DateTime<Utc>,
    /// Record version (`ETag`).
    pub version: i64,
}

#[derive(FromRow)]
struct ActualRow {
    id: Uuid,
    line_kind: Option<String>,
    line_id: Option<Uuid>,
    line_name: Option<String>,
    substitute_ingredient_id: Option<Uuid>,
    planned_amount_kg: Option<f64>,
    actual_amount_kg: f64,
    notes: Option<String>,
    updated_at: DateTime<Utc>,
    version: i64,
}

impl From<ActualRow> for Actual {
    fn from(r: ActualRow) -> Self {
        let deviation_kg = r.planned_amount_kg.map(|p| r.actual_amount_kg - p);
        let deviation_pct = match (deviation_kg, r.planned_amount_kg) {
            (Some(d), Some(p)) if p > 0.0 => Some(d / p * 100.0),
            _ => None,
        };
        Self {
            id: r.id,
            line_kind: r.line_kind,
            line_id: r.line_id,
            line_name: r.line_name,
            substitute_ingredient_id: r.substitute_ingredient_id,
            planned_amount_kg: r.planned_amount_kg,
            actual_amount_kg: r.actual_amount_kg,
            deviation_kg,
            deviation_pct,
            notes: r.notes,
            updated_at: r.updated_at,
            version: r.version,
        }
    }
}

const ACTUAL_SQL: &str = "SELECT a.id, a.line_kind, a.line_id, p.name AS line_name, \
    a.substitute_ingredient_id, p.planned AS planned_amount_kg, a.actual_amount_kg, a.notes, \
    a.updated_at, a.version \
    FROM batch_ingredient_actuals a \
    LEFT JOIN LATERAL ( \
        SELECT name, amount_kg AS planned FROM recipe_fermentables WHERE a.line_kind = 'fermentable' AND id = a.line_id \
        UNION ALL SELECT name, amount_kg FROM recipe_hops WHERE a.line_kind = 'hop' AND id = a.line_id \
        UNION ALL SELECT name, amount_kg FROM recipe_yeasts WHERE a.line_kind = 'yeast' AND id = a.line_id \
        UNION ALL SELECT name, amount FROM recipe_miscs WHERE a.line_kind = 'misc' AND id = a.line_id \
        UNION ALL SELECT agent, amount_kg FROM recipe_water_additions WHERE a.line_kind = 'water_addition' AND id = a.line_id \
    ) p ON true \
    WHERE a.batch_id = $1 AND a.deleted_at IS NULL";

/// A measurement.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct Measurement {
    /// Client UUID.
    pub id: Uuid,
    /// The batch.
    pub batch_id: Uuid,
    /// `sg`, `temp_c`, `ph`, `pressure_bar`, `volume_l`, `abv_pct`, `battery_v`, `signal_dbm`.
    pub kind: String,
    /// Value in SI units (SG as specific gravity, temperature in degrees C).
    pub value: f64,
    /// Measured at.
    pub measured_at: DateTime<Utc>,
    /// `manual`, `paper` or `device`.
    pub source: String,
    /// The brew day package of the client.
    pub package_id: Option<Uuid>,
}

/// One measurement of a request.
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct NewMeasurement {
    /// Client UUID. The same ID with the same data is stored once.
    pub id: Uuid,
    /// `sg`, `temp_c`, `ph`, `pressure_bar`, `volume_l`, `abv_pct`.
    pub kind: String,
    /// Value in SI units.
    pub value: f64,
    /// Measured at (RFC 3339).
    pub measured_at: DateTime<Utc>,
    /// `manual` (default) or `paper`.
    pub source: Option<String>,
    /// Brew day package that the client works from.
    pub package_id: Option<Uuid>,
}

/// Body of a measurement request.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct NewMeasurements {
    /// One to 100 measurements.
    pub items: Vec<NewMeasurement>,
}

/// Body of a batch create.
#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct NewBatch {
    /// An `approved` version.
    pub recipe_version_id: Uuid,
    /// Name.
    pub name: Option<String>,
    /// Brew day.
    pub planned_date: Option<NaiveDate>,
    /// The day the beer is ready. The server plans backward and sets `planned_date`.
    pub ready_date: Option<NaiveDate>,
    /// Target volume, L (default: the batch volume of the recipe).
    pub target_volume_l: Option<f64>,
    /// Equipment.
    pub equipment_id: Option<Uuid>,
    /// Notes.
    pub notes: Option<String>,
}

/// Merge patch of a batch.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct BatchPatch {
    /// Name.
    pub name: Option<String>,
    /// Brew day.
    pub planned_date: Option<NaiveDate>,
    /// Target volume, L.
    pub target_volume_l: Option<f64>,
    /// Actual volume, L.
    pub actual_volume_l: Option<f64>,
    /// Equipment.
    pub equipment_id: Option<Uuid>,
    /// Notes.
    pub notes: Option<String>,
}

/// Body of a stage transition.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    /// The next stage, or `cancelled`.
    pub to_stage: String,
}

/// Merge patch of a step.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct StepPatch {
    /// `pending`, `running`, `done`, `skipped`.
    pub status: Option<String>,
    /// Actual start (default: now when the status becomes `running`).
    pub actual_start_at: Option<DateTime<Utc>>,
    /// Actual end (default: now when the status becomes `done`).
    pub actual_end_at: Option<DateTime<Utc>>,
    /// `timer`, `manual`, `paper`.
    pub input_source: Option<String>,
}

/// Body of an actual amount.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ActualBody {
    /// Actual amount, kg (L for a liquid).
    pub actual_amount_kg: f64,
    /// Notes.
    pub notes: Option<String>,
}

/// Body of an unplanned ingredient.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UnplannedActual {
    /// The ingredient from the library.
    pub substitute_ingredient_id: Uuid,
    /// Actual amount, kg.
    pub actual_amount_kg: f64,
    /// Notes.
    pub notes: Option<String>,
}

/// Filter of the batch list.
#[derive(Debug, Default, Deserialize)]
pub struct BatchFilter {
    /// Stage.
    pub stage: Option<String>,
    /// Text in the code or name (case blind).
    pub q: Option<String>,
    /// The recipe.
    pub recipe_id: Option<Uuid>,
}

/// Filter of the measurement list.
#[derive(Debug, Default, Deserialize)]
pub struct MeasurementFilter {
    /// Kind.
    pub kind: Option<String>,
    /// From this time.
    pub from: Option<DateTime<Utc>>,
    /// Until this time.
    pub to: Option<DateTime<Utc>>,
}

/// Query of the calendar.
#[derive(Debug, Deserialize)]
pub struct CalendarQuery {
    /// First day.
    pub from: NaiveDate,
    /// Last day.
    pub to: NaiveDate,
}

/// Body of a backward plan.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct BackwardPlan {
    /// The recipe version that gives the durations.
    pub recipe_version_id: Uuid,
    /// The day the beer is ready.
    pub ready_date: NaiveDate,
}

// --- helpers ------------------------------------------------------------------------------

fn constraint_error(e: sqlx::Error) -> ApiError {
    match crate::library::sqlstate(&e).as_deref() {
        Some("23514" | "22003" | "22P02") => issue("/", "constraint_violated"),
        Some("23503") => issue("/", "reference_not_found"),
        Some("23505") => conflict("/", "already_exists"),
        Some("23000") => conflict("/recipe_version_id", "version_not_approved"),
        _ => e.into(),
    }
}

fn batch_example() -> Value {
    json!({
        "id": "0192f1c07a3b7000800000000000000b", "code": "B-0001", "name": "House IPA",
        "recipe_version_id": "0192f1c07a3b7000800000000000000c",
        "recipe_id": "0192f1c07a3b7000800000000000000d", "recipe_version_number": 1,
        "equipment_id": null, "stage": "planned", "planned_date": "2026-11-28",
        "started_at": null, "completed_at": null, "target_volume_l": 20.0, "actual_volume_l": null,
        "engine_version": "0.1.0", "notes": null, "created_at": "2026-11-20T10:00:00Z",
        "updated_at": "2026-11-20T10:00:00Z", "deleted_at": null, "version": 1
    })
}

async fn fetch_batch(tx: &mut PgConnection, id: Uuid, lock: bool) -> ApiResult<Batch> {
    let lock = if lock { " FOR UPDATE OF b" } else { "" };
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {BATCH_COLUMNS} FROM {BATCH_FROM} WHERE b.id = $1 AND b.deleted_at IS NULL{lock}"
    )))
    .bind(id)
    .fetch_optional(tx)
    .await?
    .ok_or_else(not_found)
}

fn closed(batch: &Batch) -> bool {
    matches!(batch.stage.as_str(), "completed" | "cancelled")
}

async fn batch_event(
    tx: &mut PgConnection,
    ws: Uuid,
    topic: &str,
    batch_id: Uuid,
    version: i64,
    data: Option<Value>,
) -> ApiResult<()> {
    db::outbox::enqueue(
        tx,
        &db::outbox::NewOutboxEvent {
            topic: topic.to_owned(),
            workspace_id: Some(ws),
            entity: Some(db::outbox::Entity {
                kind: "batch".to_owned(),
                id: batch_id,
                version,
            }),
            data,
        },
    )
    .await?;
    Ok(())
}

/// Dates of the stages when the beer is ready on `ready`: brew day, end of the fermentation, and
/// the ready day (end of the conditioning). The brew day and the pitch are on the same day.
fn backward_dates(version: &Value, ready: NaiveDate) -> Value {
    let ferment = fermentation_days(version).round() as i64;
    let condition = conditioning_days(version).round() as i64;
    let brew = ready - Duration::days(ferment + condition);
    json!({
        "brew_date": brew,
        "stages": [
            {"stage": "brewing", "start": brew, "end": brew},
            {"stage": "fermenting", "start": brew, "end": brew + Duration::days(ferment)},
            {"stage": "conditioning", "start": brew + Duration::days(ferment), "end": ready},
            {"stage": "packaging", "start": ready, "end": ready},
        ],
        "ready_date": ready,
        "fermentation_days": ferment,
        "conditioning_days": condition,
    })
}

// --- batch endpoints ----------------------------------------------------------------------

/// Lists the batches of the workspace.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/batches",
    tag = "batches",
    operation_id = "batch_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("stage" = Option<String>, Query, description = "Stage"),
        ("q" = Option<String>, Query, description = "Text in the code or name (case blind)"),
        ("recipe_id" = Option<Uuid>, Query, description = "The recipe"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page"),
        ("sort" = Option<String>, Query, description = "`id` (creation order, default) or `-id`")
    ),
    responses((status = 200, description = "A page of batches", body = Page<Batch>,
        example = json!({"items": [batch_example()], "next_cursor": null})))
)]
pub async fn batch_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<BatchFilter>,
) -> ApiResult<Json<Page<Batch>>> {
    ctx.require(READ_PERMISSION)?;
    let paging = paging(&params, &["id"], "-id")?;
    if filter
        .stage
        .as_deref()
        .is_some_and(|s| !STAGES.contains(&s))
    {
        return Err(issue("stage", "invalid_value"));
    }
    let (cmp, dir) = paging.dir();
    let sql = sqlx::AssertSqlSafe(format!(
        "SELECT {BATCH_COLUMNS} FROM {BATCH_FROM} WHERE b.deleted_at IS NULL \
           AND ($1::text IS NULL OR b.stage = $1) \
           AND ($2::text IS NULL OR strpos(lower(b.code || ' ' || b.name), lower($2)) > 0) \
           AND ($3::uuid IS NULL OR rv.recipe_id = $3) \
           AND ($4::uuid IS NULL OR b.id {cmp} $4) \
         ORDER BY b.id {dir} LIMIT $5"
    ));
    let mut tx = ctx.begin(&state.db).await?;
    let rows: Vec<Batch> = sqlx::query_as(sql)
        .bind(&filter.stage)
        .bind(&filter.q)
        .bind(filter.recipe_id)
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

/// Starts a batch from an approved recipe version and writes its steps.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/batches",
    tag = "batches",
    operation_id = "batch_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = NewBatch,
    responses((status = 201, description = "The batch in stage `planned`", body = Batch,
        example = json!({"id": "0192f1c07a3b7000800000000000000b", "code": "B-0001",
            "name": "House IPA", "recipe_version_id": "0192f1c07a3b7000800000000000000c",
            "recipe_id": "0192f1c07a3b7000800000000000000d", "recipe_version_number": 1,
            "equipment_id": null, "stage": "planned", "planned_date": "2026-11-28",
            "started_at": null, "completed_at": null, "target_volume_l": 20.0,
            "actual_volume_l": null, "engine_version": "0.1.0", "notes": null,
            "created_at": "2026-11-20T10:00:00Z", "updated_at": "2026-11-20T10:00:00Z",
            "deleted_at": null, "version": 1})))
)]
pub async fn batch_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(new): ApiJson<NewBatch>,
) -> ApiResult<Response> {
    ctx.require(PLAN_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let user = ctx.identity.user_id;
    if new.planned_date.is_some() && new.ready_date.is_some() {
        return Err(issue("/ready_date", "exclusive_with_planned_date"));
    }
    if new.target_volume_l.is_some_and(|v| v.is_nan() || v <= 0.0) {
        return Err(issue("/target_volume_l", "out_of_range"));
    }
    let mut tx = ctx.begin(&state.db).await?;
    let status: Option<(String, Option<String>, Option<Uuid>)> = sqlx::query_as(
        "SELECT status, engine_version, equipment_id FROM recipe_versions \
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(new.recipe_version_id)
    .fetch_optional(&mut *tx)
    .await?;
    let (status, engine_version, version_equipment) =
        status.ok_or_else(|| issue("/recipe_version_id", "not_found"))?;
    if status != "approved" {
        return Err(conflict("/recipe_version_id", "version_not_approved"));
    }
    let v = version_json(&mut tx, new.recipe_version_id).await?;
    let planned_date = match (new.planned_date, new.ready_date) {
        (Some(d), None) => Some(d),
        (None, Some(ready)) => Some(
            ready - Duration::days((fermentation_days(&v) + conditioning_days(&v)).round() as i64),
        ),
        _ => None,
    };
    let name = new
        .name
        .clone()
        .or_else(|| v.get("name").and_then(Value::as_str).map(str::to_owned))
        .unwrap_or_default();
    let name = if name.is_empty() {
        sqlx::query_scalar::<_, String>("SELECT name FROM recipes WHERE id = $1")
            .bind(
                v["recipe_id"]
                    .as_str()
                    .and_then(|s| Uuid::parse_str(s).ok()),
            )
            .fetch_optional(&mut *tx)
            .await?
            .unwrap_or_default()
    } else {
        name
    };
    // The code counts up inside the workspace; the lock keeps two requests apart.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1::text, 22))")
        .bind(ws)
        .execute(&mut *tx)
        .await?;
    let next: i64 = sqlx::query_scalar(
        "SELECT COALESCE(max(substring(code FROM '^B-([0-9]+)$')::bigint), 0) + 1 FROM batches",
    )
    .fetch_one(&mut *tx)
    .await?;
    let code = format!("B-{next:04}");
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO batches (id, workspace_id, code, name, recipe_version_id, equipment_id, \
             planned_date, target_volume_l, engine_version, notes, created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $11)",
    )
    .bind(id)
    .bind(ws)
    .bind(&code)
    .bind(&name)
    .bind(new.recipe_version_id)
    .bind(new.equipment_id.or(version_equipment))
    .bind(planned_date)
    .bind(
        new.target_volume_l
            .or_else(|| v.get("batch_volume_l").and_then(Value::as_f64)),
    )
    .bind(&engine_version)
    .bind(&new.notes)
    .bind(user)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    let steps = plan_steps(&v);
    let times = planned_date.map(|d| schedule(&steps, day_start(d)));
    for (i, s) in steps.iter().enumerate() {
        let (start, end) = times
            .as_ref()
            .and_then(|t| t.get(i))
            .map_or((None, None), |t| (Some(t.0), Some(t.1)));
        sqlx::query(
            "INSERT INTO batch_steps (id, workspace_id, batch_id, position, kind, name, \
                 planned_start_at, planned_end_at, created_by, updated_by) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)",
        )
        .bind(Uuid::now_v7())
        .bind(ws)
        .bind(id)
        .bind(i32::try_from(i).unwrap_or(i32::MAX))
        .bind(s.kind)
        .bind(&s.name)
        .bind(start)
        .bind(end)
        .bind(user)
        .execute(&mut *tx)
        .await
        .map_err(constraint_error)?;
    }
    sqlx::query(
        "INSERT INTO stage_events (id, workspace_id, batch_id, from_stage, to_stage, created_by) \
         VALUES ($1, $2, $3, NULL, 'planned', $4)",
    )
    .bind(Uuid::now_v7())
    .bind(ws)
    .bind(id)
    .bind(user)
    .execute(&mut *tx)
    .await?;
    let row = fetch_batch(&mut tx, id, false).await?;
    batch_event(&mut tx, ws, "batch.created", id, row.version, None).await?;
    tx.commit().await?;
    Ok(created_response(
        &format!("/v1/workspaces/{ws}/batches/{id}"),
        row.version,
        &row,
    ))
}

/// Reads a batch.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/batches/{id}",
    tag = "batches",
    operation_id = "batch_get",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Batch")
    ),
    responses(
        (status = 200, description = "The batch", body = Batch, example = json!(batch_example())),
        (status = 304, description = "Not modified (`If-None-Match` matched)")
    )
)]
pub async fn batch_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    ctx.require(READ_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let row = fetch_batch(&mut tx, id, false).await?;
    Ok(concurrency::conditional_get(&headers, row.version, &row))
}

/// Changes the name, plan or volume of a batch. A closed batch answers `409`.
#[utoipa::path(
    patch,
    path = "/v1/workspaces/{workspace_id}/batches/{id}",
    tag = "batches",
    operation_id = "batch_update",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Batch"),
        ("If-Match" = String, Header, description = "Version `ETag` of the batch")
    ),
    request_body(content = BatchPatch, content_type = "application/merge-patch+json"),
    responses((status = 200, description = "The updated batch", body = Batch,
        example = json!(batch_example())))
)]
pub async fn batch_update(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    MergePatch(patch): MergePatch<Value>,
) -> ApiResult<Response> {
    ctx.require(PLAN_PERMISSION)?;
    if !patch.is_object() {
        return Err(issue("/", "invalid"));
    }
    let mut tx = ctx.begin(&state.db).await?;
    let before = fetch_batch(&mut tx, id, true).await?;
    check_if_match(Concurrency::Optimistic, &headers, before.version, || {
        json!(before)
    })?;
    if closed(&before) {
        return Err(conflict("/stage", "batch_closed"));
    }
    let mut merged = json!({
        "name": before.name, "planned_date": before.planned_date,
        "target_volume_l": before.target_volume_l, "actual_volume_l": before.actual_volume_l,
        "equipment_id": before.equipment_id, "notes": before.notes
    });
    merge(&mut merged, &patch);
    let input: BatchPatch = serde_json::from_value(merged).map_err(|e| {
        ApiError::validation(vec![
            FieldIssue::new("/", "invalid").with_params(json!({"reason": e.to_string()})),
        ])
    })?;
    sqlx::query(
        "UPDATE batches SET name = $2, planned_date = $3, target_volume_l = $4, \
             actual_volume_l = $5, equipment_id = $6, notes = $7, updated_by = $8 WHERE id = $1",
    )
    .bind(id)
    .bind(input.name.unwrap_or_default())
    .bind(input.planned_date)
    .bind(input.target_volume_l)
    .bind(input.actual_volume_l)
    .bind(input.equipment_id)
    .bind(&input.notes)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    let row = fetch_batch(&mut tx, id, false).await?;
    tx.commit().await?;
    Ok(concurrency::record_response(
        StatusCode::OK,
        row.version,
        &row,
    ))
}

/// Deletes a batch (soft delete).
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/batches/{id}",
    tag = "batches",
    operation_id = "batch_delete",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Batch"),
        ("If-Match" = String, Header, description = "Version `ETag` of the batch")
    ),
    responses((status = 204, description = "Deleted"))
)]
pub async fn batch_delete(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    ctx.require(PLAN_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let before = fetch_batch(&mut tx, id, true).await?;
    check_if_match(Concurrency::Optimistic, &headers, before.version, || {
        json!(before)
    })?;
    sqlx::query("UPDATE batches SET deleted_at = now(), updated_by = $2 WHERE id = $1")
        .bind(id)
        .bind(ctx.identity.user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

fn next_stage(from: &str) -> Option<&'static str> {
    Some(match from {
        "planned" => "brewing",
        "brewing" => "fermenting",
        "fermenting" => "conditioning",
        "conditioning" => "packaging",
        "packaging" => "completed",
        _ => return None,
    })
}

/// Moves a batch to its next stage, or cancels it. Other moves answer `409`.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/batches/{id}/transitions",
    tag = "batches",
    operation_id = "batch_transition",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Batch"),
        ("If-Match" = String, Header, description = "Version `ETag` of the batch")
    ),
    request_body = Transition,
    responses((status = 200, description = "The batch in its new stage", body = Batch,
        example = json!(batch_example())))
)]
pub async fn batch_transition(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    ApiJson(body): ApiJson<Transition>,
) -> ApiResult<Response> {
    ctx.require(RECORD_PERMISSION)?;
    let ws = workspace(&ctx)?;
    if !STAGES.contains(&body.to_stage.as_str()) {
        return Err(issue("/to_stage", "invalid_value"));
    }
    let mut tx = ctx.begin(&state.db).await?;
    let before = fetch_batch(&mut tx, id, true).await?;
    check_if_match(Concurrency::Optimistic, &headers, before.version, || {
        json!(before)
    })?;
    let allowed = body.to_stage == "cancelled" && !closed(&before)
        || next_stage(&before.stage) == Some(body.to_stage.as_str());
    if !allowed {
        return Err(ApiError::new(ErrorCode::Conflict).with_errors(vec![
            FieldIssue::new("/to_stage", "invalid_transition")
                .with_params(json!({"from": before.stage, "to": body.to_stage})),
        ]));
    }
    sqlx::query(
        "UPDATE batches SET stage = $2, \
             started_at = CASE WHEN $2 = 'brewing' THEN now() ELSE started_at END, \
             completed_at = CASE WHEN $2 IN ('completed', 'cancelled') THEN now() ELSE completed_at END, \
             updated_by = $3 WHERE id = $1",
    )
    .bind(id)
    .bind(&body.to_stage)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    sqlx::query(
        "INSERT INTO stage_events (id, workspace_id, batch_id, from_stage, to_stage, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(Uuid::now_v7())
    .bind(ws)
    .bind(id)
    .bind(&before.stage)
    .bind(&body.to_stage)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await?;
    let row = fetch_batch(&mut tx, id, false).await?;
    batch_event(
        &mut tx,
        ws,
        "batch.stage_changed",
        id,
        row.version,
        Some(json!({"from": before.stage, "to": body.to_stage})),
    )
    .await?;
    tx.commit().await?;
    Ok(concurrency::record_response(
        StatusCode::OK,
        row.version,
        &row,
    ))
}

// --- steps --------------------------------------------------------------------------------

/// Lists the steps of a batch in order.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/batches/{id}/steps",
    tag = "batches",
    operation_id = "batchstep_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Batch")
    ),
    responses((status = 200, description = "The steps", body = Object,
        example = json!({"items": [{"id": "0192f1c07a3b7000800000000000000b",
            "batch_id": "0192f1c07a3b7000800000000000000c", "position": 0, "kind": "prepare",
            "name": "Prepare equipment", "planned_start_at": null, "planned_end_at": null,
            "actual_start_at": null, "actual_end_at": null, "status": "pending",
            "input_source": null, "version": 1}], "next_cursor": null})))
)]
pub async fn batchstep_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<Page<BatchStep>>> {
    ctx.require(READ_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    fetch_batch(&mut tx, id, false).await?;
    let items: Vec<BatchStep> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {STEP_COLUMNS} FROM batch_steps WHERE batch_id = $1 AND deleted_at IS NULL \
         ORDER BY position"
    )))
    .bind(id)
    .fetch_all(&mut *tx)
    .await?;
    Ok(Json(Page {
        items,
        next_cursor: None,
    }))
}

/// Records a step: start, end, status and input source. The newest write wins for each field.
#[utoipa::path(
    patch,
    path = "/v1/workspaces/{workspace_id}/batches/{id}/steps/{step_id}",
    tag = "batches",
    operation_id = "batchstep_update",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Batch"),
        ("step_id" = Uuid, Path, description = "Step"),
        ("If-Match" = Option<String>, Header, description = "Version `ETag` of the step")
    ),
    request_body(content = StepPatch, content_type = "application/merge-patch+json"),
    responses((status = 200, description = "The updated step", body = BatchStep,
        example = json!({"id": "0192f1c07a3b7000800000000000000b",
            "batch_id": "0192f1c07a3b7000800000000000000c", "position": 0, "kind": "prepare",
            "name": "Prepare equipment", "planned_start_at": null, "planned_end_at": null,
            "actual_start_at": "2026-11-28T08:00:00Z", "actual_end_at": null,
            "status": "running", "input_source": "manual", "version": 2})))
)]
pub async fn batchstep_update(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id, step_id)): Path<(Uuid, Uuid, Uuid)>,
    headers: HeaderMap,
    MergePatch(patch): MergePatch<StepPatch>,
) -> ApiResult<Response> {
    ctx.require(RECORD_PERMISSION)?;
    if patch
        .status
        .as_deref()
        .is_some_and(|s| !matches!(s, "pending" | "running" | "done" | "skipped"))
    {
        return Err(issue("/status", "invalid_value"));
    }
    if patch
        .input_source
        .as_deref()
        .is_some_and(|s| !matches!(s, "timer" | "manual" | "paper"))
    {
        return Err(issue("/input_source", "invalid_value"));
    }
    let user = ctx.identity.user_id;
    let mut tx = ctx.begin(&state.db).await?;
    let batch = fetch_batch(&mut tx, id, false).await?;
    if batch.stage == "cancelled" {
        return Err(conflict("/stage", "batch_closed"));
    }
    let before: BatchStep = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {STEP_COLUMNS} FROM batch_steps WHERE id = $1 AND batch_id = $2 \
         AND deleted_at IS NULL FOR UPDATE"
    )))
    .bind(step_id)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(not_found)?;
    check_if_match(Concurrency::FieldLww, &headers, before.version, || {
        json!(before)
    })?;
    let now = Utc::now();
    let status = patch
        .status
        .clone()
        .unwrap_or_else(|| before.status.clone());
    let start = patch
        .actual_start_at
        .or(before.actual_start_at)
        .or_else(|| matches!(status.as_str(), "running" | "done").then_some(now));
    let end = patch
        .actual_end_at
        .or(before.actual_end_at)
        .or_else(|| (status == "done" && patch.status.is_some()).then_some(now));
    sqlx::query(
        "UPDATE batch_steps SET status = $2, actual_start_at = $3, actual_end_at = $4, \
             input_source = COALESCE($5, input_source), \
             closed_by = CASE WHEN $2 IN ('done', 'skipped') THEN $6 ELSE closed_by END, \
             updated_by = $6 WHERE id = $1",
    )
    .bind(step_id)
    .bind(&status)
    .bind(start)
    .bind(end)
    .bind(&patch.input_source)
    .bind(user)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    let row: BatchStep = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {STEP_COLUMNS} FROM batch_steps WHERE id = $1"
    )))
    .bind(step_id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(concurrency::record_response(
        StatusCode::OK,
        row.version,
        &row,
    ))
}

// --- actual amounts -----------------------------------------------------------------------

async fn fetch_actuals(tx: &mut PgConnection, batch_id: Uuid) -> ApiResult<Vec<Actual>> {
    let rows: Vec<ActualRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{ACTUAL_SQL} ORDER BY a.created_at, a.id"
    )))
    .bind(batch_id)
    .fetch_all(tx)
    .await?;
    Ok(rows.into_iter().map(Actual::from).collect())
}

/// Lists the actual amounts of a batch with plan and deviation.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/batches/{id}/actuals",
    tag = "batches",
    operation_id = "batchactual_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Batch")
    ),
    responses((status = 200, description = "The actual amounts", body = Object,
        example = json!({"items": [{"id": "0192f1c07a3b7000800000000000000b",
            "line_kind": "fermentable", "line_id": "0192f1c07a3b7000800000000000000c",
            "line_name": "Pale malt", "substitute_ingredient_id": null,
            "planned_amount_kg": 4.5, "actual_amount_kg": 4.6, "deviation_kg": 0.1,
            "deviation_pct": 2.2, "notes": null, "updated_at": "2026-11-28T09:00:00Z",
            "version": 1}], "next_cursor": null})))
)]
pub async fn batchactual_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<Page<Actual>>> {
    ctx.require(READ_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    fetch_batch(&mut tx, id, false).await?;
    let items = fetch_actuals(&mut tx, id).await?;
    Ok(Json(Page {
        items,
        next_cursor: None,
    }))
}

/// Records the actual amount of a recipe line (create or replace). A deviation of 5 percent or
/// more opens a deviation record (`ingredient_amount`).
#[utoipa::path(
    put,
    path = "/v1/workspaces/{workspace_id}/batches/{id}/actuals/{line_kind}/{line_id}",
    tag = "batches",
    operation_id = "batchactual_put",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Batch"),
        ("line_kind" = String, Path, description = "`fermentable`, `hop`, `yeast`, `misc`, `water_addition`"),
        ("line_id" = Uuid, Path, description = "Line of the recipe version of the batch")
    ),
    request_body = ActualBody,
    responses((status = 200, description = "The actual amount", body = Actual,
        example = json!({"id": "0192f1c07a3b7000800000000000000b", "line_kind": "fermentable",
            "line_id": "0192f1c07a3b7000800000000000000c", "line_name": "Pale malt",
            "substitute_ingredient_id": null, "planned_amount_kg": 4.5,
            "actual_amount_kg": 4.6, "deviation_kg": 0.1, "deviation_pct": 2.2, "notes": null,
            "updated_at": "2026-11-28T09:00:00Z", "version": 1})))
)]
pub async fn batchactual_put(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id, line_kind, line_id)): Path<(Uuid, Uuid, String, Uuid)>,
    ApiJson(body): ApiJson<ActualBody>,
) -> ApiResult<Json<Actual>> {
    ctx.require(RECORD_PERMISSION)?;
    let ws = workspace(&ctx)?;
    if !LINE_KINDS.contains(&line_kind.as_str()) {
        return Err(issue("/line_kind", "invalid_value"));
    }
    if !body.actual_amount_kg.is_finite() || body.actual_amount_kg < 0.0 {
        return Err(issue("/actual_amount_kg", "out_of_range"));
    }
    let user = ctx.identity.user_id;
    let mut tx = ctx.begin(&state.db).await?;
    let batch = fetch_batch(&mut tx, id, false).await?;
    if closed(&batch) {
        return Err(conflict("/stage", "batch_closed"));
    }
    let actual_id: Uuid = sqlx::query_scalar(
        "INSERT INTO batch_ingredient_actuals (id, workspace_id, batch_id, line_kind, line_id, \
             actual_amount_kg, notes, created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8) \
         ON CONFLICT (workspace_id, batch_id, line_kind, line_id) \
             WHERE line_id IS NOT NULL AND deleted_at IS NULL \
         DO UPDATE SET actual_amount_kg = EXCLUDED.actual_amount_kg, notes = EXCLUDED.notes, \
             updated_by = EXCLUDED.updated_by \
         RETURNING id",
    )
    .bind(Uuid::now_v7())
    .bind(ws)
    .bind(id)
    .bind(&line_kind)
    .bind(line_id)
    .bind(body.actual_amount_kg)
    .bind(&body.notes)
    .bind(user)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match crate::library::sqlstate(&e).as_deref() {
        Some("23503") => issue("/line_id", "not_a_line_of_the_recipe_version"),
        _ => constraint_error(e),
    })?;
    let row: ActualRow = sqlx::query_as(sqlx::AssertSqlSafe(format!("{ACTUAL_SQL} AND a.id = $2")))
        .bind(id)
        .bind(actual_id)
        .fetch_one(&mut *tx)
        .await?;
    let actual = Actual::from(row);
    // A new measurement of the deviation replaces the open record of this amount.
    sqlx::query(
        "DELETE FROM deviations WHERE actual_id = $1 AND kind = 'ingredient_amount' AND status = 'open'",
    )
    .bind(actual_id)
    .execute(&mut *tx)
    .await?;
    if let Some(pct) = actual
        .deviation_pct
        .filter(|p| p.abs() >= DEVIATION_LIMIT_PCT)
    {
        let severity = if pct.abs() >= 2.0 * DEVIATION_LIMIT_PCT {
            "minor"
        } else {
            "info"
        };
        sqlx::query(
            "INSERT INTO deviations (id, workspace_id, batch_id, actual_id, kind, description, \
                 severity, created_by, updated_by) \
             VALUES ($1, $2, $3, $4, 'ingredient_amount', $5, $6, $7, $7)",
        )
        .bind(Uuid::now_v7())
        .bind(ws)
        .bind(id)
        .bind(actual_id)
        .bind(format!(
            "{} deviates {:+.1} percent from the plan",
            actual.line_name.clone().unwrap_or_default(),
            pct
        ))
        .bind(severity)
        .bind(user)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(Json(actual))
}

/// Records an ingredient that is not in the recipe.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/batches/{id}/actuals",
    tag = "batches",
    operation_id = "batchactual_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Batch"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = UnplannedActual,
    responses((status = 201, description = "The actual amount", body = Actual,
        example = json!({"id": "0192f1c07a3b7000800000000000000b", "line_kind": null,
            "line_id": null, "line_name": null,
            "substitute_ingredient_id": "0192f1c07a3b7000800000000000000c",
            "planned_amount_kg": null, "actual_amount_kg": 0.1, "deviation_kg": null,
            "deviation_pct": null, "notes": null, "updated_at": "2026-11-28T09:00:00Z",
            "version": 1})))
)]
pub async fn batchactual_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    ApiJson(body): ApiJson<UnplannedActual>,
) -> ApiResult<Response> {
    ctx.require(RECORD_PERMISSION)?;
    let ws = workspace(&ctx)?;
    if !body.actual_amount_kg.is_finite() || body.actual_amount_kg < 0.0 {
        return Err(issue("/actual_amount_kg", "out_of_range"));
    }
    let mut tx = ctx.begin(&state.db).await?;
    let batch = fetch_batch(&mut tx, id, false).await?;
    if closed(&batch) {
        return Err(conflict("/stage", "batch_closed"));
    }
    let actual_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO batch_ingredient_actuals (id, workspace_id, batch_id, substitute_ingredient_id, \
             actual_amount_kg, notes, created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $7)",
    )
    .bind(actual_id)
    .bind(ws)
    .bind(id)
    .bind(body.substitute_ingredient_id)
    .bind(body.actual_amount_kg)
    .bind(&body.notes)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    let row: ActualRow = sqlx::query_as(sqlx::AssertSqlSafe(format!("{ACTUAL_SQL} AND a.id = $2")))
        .bind(id)
        .bind(actual_id)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    let actual = Actual::from(row);
    Ok(created_response(
        &format!("/v1/workspaces/{ws}/batches/{id}/actuals"),
        actual.version,
        &actual,
    ))
}

// --- measurements -------------------------------------------------------------------------

fn measurement_issues(i: usize, m: &NewMeasurement, now: DateTime<Utc>) -> Vec<FieldIssue> {
    let mut out = Vec::new();
    if !m.value.is_finite() {
        out.push(FieldIssue::new(format!("/items/{i}/value"), "out_of_range"));
    }
    if !matches!(
        m.kind.as_str(),
        "sg" | "temp_c" | "ph" | "pressure_bar" | "volume_l" | "abv_pct"
    ) {
        out.push(FieldIssue::new(format!("/items/{i}/kind"), "invalid_value"));
    }
    if !matches!(m.source.as_deref(), None | Some("manual" | "paper")) {
        out.push(FieldIssue::new(
            format!("/items/{i}/source"),
            "invalid_value",
        ));
    }
    if let Err(e) = db::validate_measured_at(m.measured_at, now) {
        let code = match e {
            db::MeasuredAtError::TooOld => "too_old",
            db::MeasuredAtError::InFuture => "in_future",
        };
        out.push(FieldIssue::new(format!("/items/{i}/measured_at"), code));
    }
    out
}

/// Sends measurements. An item with a known `id` is not stored again, so the client can repeat the
/// request after a lost answer. An item that names a package is checked against the package.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/batches/{id}/measurements",
    tag = "batches",
    operation_id = "measurement_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Batch")
    ),
    request_body = NewMeasurements,
    responses((status = 200, description = "Each item is `created` or `duplicate`", body = Object,
        example = json!({"created": 1, "duplicates": 0, "items": [
            {"id": "0192f1c07a3b7000800000000000000b", "status": "created"}]})))
)]
pub async fn measurement_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    ApiJson(body): ApiJson<NewMeasurements>,
) -> ApiResult<Json<Value>> {
    ctx.require(RECORD_PERMISSION)?;
    let ws = workspace(&ctx)?;
    crate::ensure_bulk_len("items", body.items.len())?;
    let now = Utc::now();
    let mut issues: Vec<FieldIssue> = body
        .items
        .iter()
        .enumerate()
        .flat_map(|(i, m)| measurement_issues(i, m, now))
        .collect();
    let mut tx = ctx.begin(&state.db).await?;
    let batch = fetch_batch(&mut tx, id, false).await?;
    if batch.stage == "cancelled" {
        return Err(conflict("/stage", "batch_closed"));
    }
    // Each package is checked once.
    let mut checked: std::collections::HashMap<Uuid, Result<(), PackageProblem>> =
        std::collections::HashMap::new();
    for (i, m) in body.items.iter().enumerate() {
        let Some(pid) = m.package_id else { continue };
        if let std::collections::hash_map::Entry::Vacant(slot) = checked.entry(pid) {
            slot.insert(check_package(&mut tx, ws, id, pid).await?);
        }
        if let Some(Err(problem)) = checked.get(&pid) {
            issues.push(FieldIssue::new(
                format!("/items/{i}/package_id"),
                problem.code(),
            ));
        }
    }
    if !issues.is_empty() {
        return Err(ApiError::validation(issues));
    }
    let mut created = 0_u32;
    let mut results = Vec::with_capacity(body.items.len());
    for m in &body.items {
        let known: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM measurements WHERE id = $1 AND workspace_id = $2)",
        )
        .bind(m.id)
        .bind(ws)
        .fetch_one(&mut *tx)
        .await?;
        let inserted = if known {
            0
        } else {
            sqlx::query(
                "INSERT INTO measurements (id, workspace_id, batch_id, kind, value, measured_at, \
                     source, package_id, created_by) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) ON CONFLICT DO NOTHING",
            )
            .bind(m.id)
            .bind(ws)
            .bind(id)
            .bind(&m.kind)
            .bind(m.value)
            .bind(m.measured_at)
            .bind(m.source.as_deref().unwrap_or("manual"))
            .bind(m.package_id)
            .bind(ctx.identity.user_id)
            .execute(&mut *tx)
            .await
            .map_err(constraint_error)?
            .rows_affected()
        };
        if inserted > 0 {
            created += 1;
        }
        results.push(
            json!({"id": m.id, "status": if inserted > 0 { "created" } else { "duplicate" }}),
        );
    }
    if created > 0 {
        batch_event(
            &mut tx,
            ws,
            "batch.measurement_added",
            id,
            batch.version,
            Some(json!({"count": created})),
        )
        .await?;
    }
    tx.commit().await?;
    let duplicates = u32::try_from(body.items.len()).unwrap_or(u32::MAX) - created;
    Ok(Json(
        json!({"created": created, "duplicates": duplicates, "items": results}),
    ))
}

/// Lists the measurements of a batch in time order.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/batches/{id}/measurements",
    tag = "batches",
    operation_id = "measurement_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Batch"),
        ("kind" = Option<String>, Query, description = "Kind"),
        ("from" = Option<String>, Query, description = "From this time (RFC 3339)"),
        ("to" = Option<String>, Query, description = "Until this time (RFC 3339)"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page"),
        ("sort" = Option<String>, Query, description = "`measured_at` (default) or `-measured_at`")
    ),
    responses((status = 200, description = "A page of measurements", body = Page<Measurement>,
        example = json!({"items": [{"id": "0192f1c07a3b7000800000000000000b",
            "batch_id": "0192f1c07a3b7000800000000000000c", "kind": "sg", "value": 1.052,
            "measured_at": "2026-11-28T12:00:00Z", "source": "manual", "package_id": null}],
            "next_cursor": null})))
)]
pub async fn measurement_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<MeasurementFilter>,
) -> ApiResult<Json<Page<Measurement>>> {
    ctx.require(READ_PERMISSION)?;
    let paging = paging(&params, &["measured_at"], "measured_at")?;
    let (cmp, dir) = paging.dir();
    let sql = sqlx::AssertSqlSafe(format!(
        "SELECT id, batch_id, kind, value, measured_at, source, package_id FROM measurements \
         WHERE batch_id = $1 AND ($2::text IS NULL OR kind = $2) \
           AND ($3::timestamptz IS NULL OR measured_at >= $3) \
           AND ($4::timestamptz IS NULL OR measured_at <= $4) \
           AND ($5::text IS NULL OR (measured_at, id) {cmp} ($5::timestamptz, $6::uuid)) \
         ORDER BY measured_at {dir}, id {dir} LIMIT $7"
    ));
    let mut tx = ctx.begin(&state.db).await?;
    fetch_batch(&mut tx, id, false).await?;
    let rows: Vec<Measurement> = sqlx::query_as(sql)
        .bind(id)
        .bind(&filter.kind)
        .bind(filter.from)
        .bind(filter.to)
        .bind(&paging.key)
        .bind(paging.id)
        .bind(i64::from(paging.limit) + 1)
        .fetch_all(&mut *tx)
        .await?;
    Ok(Json(Page::from_rows(
        rows,
        paging.limit,
        &paging.sort,
        |r| crate::pagination::Cursor {
            key: json!(
                r.measured_at
                    .to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
            ),
            id: r.id,
        },
    )))
}

// --- summary, calendar --------------------------------------------------------------------

fn diff(actual: Option<f64>, plan: Option<f64>) -> Option<f64> {
    Some(actual? - plan?)
}

/// Plan against actual for a batch: gravities, volume, alcohol, ingredient deviations, step times
/// and the stage history.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/batches/{id}/summary",
    tag = "batches",
    operation_id = "batch_summary",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Batch")
    ),
    responses((status = 200, description = "The summary", body = Object,
        example = json!({"batch": {"id": "0192f1c07a3b7000800000000000000b", "code": "B-0001"},
            "plan": {"og_sg": 1.055, "fg_sg": 1.012, "abv_pct": 5.6, "volume_l": 20.0},
            "actual": {"og_sg": 1.057, "fg_sg": 1.011, "abv_pct": null, "volume_l": 19.5},
            "difference": {"og_sg": 0.002, "fg_sg": -0.001, "abv_pct": null, "volume_l": -0.5},
            "ingredients": [], "steps": {"total": 7, "done": 1, "items": []},
            "stage_events": []})))
)]
pub async fn batch_summary(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<Value>> {
    ctx.require(READ_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let batch = fetch_batch(&mut tx, id, false).await?;
    let v = version_json(&mut tx, batch.recipe_version_id).await?;
    let plan_summary = v.get("calc_summary").cloned().unwrap_or(Value::Null);
    let plan = |k: &str| plan_summary.get(k).and_then(Value::as_f64);
    let plan_volume = batch
        .target_volume_l
        .or_else(|| v.get("batch_volume_l").and_then(Value::as_f64));
    // The first gravity reading is the actual OG, the last one the actual FG.
    let first_sg: Option<f64> = sqlx::query_scalar(
        "SELECT value FROM measurements WHERE batch_id = $1 AND kind = 'sg' \
         ORDER BY measured_at, id LIMIT 1",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;
    let last_of = |kind: &'static str| {
        sqlx::query_scalar::<_, f64>(
            "SELECT value FROM measurements WHERE batch_id = $1 AND kind = $2 \
             ORDER BY measured_at DESC, id DESC LIMIT 1",
        )
        .bind(id)
        .bind(kind)
    };
    let last_sg = last_of("sg").fetch_optional(&mut *tx).await?;
    let abv = last_of("abv_pct").fetch_optional(&mut *tx).await?;
    let measured_volume = last_of("volume_l").fetch_optional(&mut *tx).await?;
    let actual_volume = batch.actual_volume_l.or(measured_volume);
    let actual_fg = if last_sg == first_sg { None } else { last_sg };
    let ingredients = fetch_actuals(&mut tx, id).await?;
    let steps: Vec<BatchStep> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {STEP_COLUMNS} FROM batch_steps WHERE batch_id = $1 AND deleted_at IS NULL ORDER BY position"
    )))
    .bind(id)
    .fetch_all(&mut *tx)
    .await?;
    let events: Value = sqlx::query_scalar(
        "SELECT COALESCE(jsonb_agg(jsonb_build_object('from_stage', from_stage, 'to_stage', to_stage, \
             'occurred_at', occurred_at) ORDER BY occurred_at, id), '[]'::jsonb) \
         FROM stage_events WHERE batch_id = $1",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    let seconds =
        |a: Option<DateTime<Utc>>, b: Option<DateTime<Utc>>| Some((b? - a?).num_seconds());
    let step_items: Vec<Value> = steps
        .iter()
        .map(|s| {
            let planned = seconds(s.planned_start_at, s.planned_end_at);
            let actual = seconds(s.actual_start_at, s.actual_end_at);
            json!({
                "id": s.id, "kind": s.kind, "name": s.name, "status": s.status,
                "planned_seconds": planned, "actual_seconds": actual,
                "difference_seconds": diff(actual.map(|x| x as f64), planned.map(|x| x as f64)),
            })
        })
        .collect();
    tx.commit().await?;
    Ok(Json(json!({
        "batch": batch,
        "plan": {"og_sg": plan("og_sg"), "fg_sg": plan("fg_sg"), "abv_pct": plan("abv_pct"),
                 "ibu": plan("ibu"), "color_srm": plan("color_srm"), "volume_l": plan_volume},
        "actual": {"og_sg": first_sg, "fg_sg": actual_fg, "abv_pct": abv, "volume_l": actual_volume},
        "difference": {
            "og_sg": diff(first_sg, plan("og_sg")), "fg_sg": diff(actual_fg, plan("fg_sg")),
            "abv_pct": diff(abv, plan("abv_pct")), "volume_l": diff(actual_volume, plan_volume),
        },
        "ingredients": ingredients,
        "steps": {
            "total": steps.len(),
            "done": steps.iter().filter(|s| s.status == "done").count(),
            "items": step_items,
        },
        "stage_events": events,
    })))
}

/// The calendar: batches with a planned brew day in a date range, and their steps.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/calendar",
    tag = "batches",
    operation_id = "calendar_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("from" = String, Query, description = "First day (YYYY-MM-DD)"),
        ("to" = String, Query, description = "Last day (YYYY-MM-DD, at most 366 days after `from`)")
    ),
    responses((status = 200, description = "Batches of the range with their steps", body = Object,
        example = json!({"items": [{"batch_id": "0192f1c07a3b7000800000000000000b",
            "code": "B-0001", "name": "House IPA", "stage": "planned",
            "planned_date": "2026-11-28", "steps": []}]})))
)]
pub async fn calendar_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(q): ApiQuery<CalendarQuery>,
) -> ApiResult<Json<Value>> {
    ctx.require(READ_PERMISSION)?;
    if q.to < q.from || (q.to - q.from).num_days() > 366 {
        return Err(issue("to", "out_of_range"));
    }
    let mut tx = ctx.begin(&state.db).await?;
    let items: Value = sqlx::query_scalar(
        "SELECT COALESCE(jsonb_agg(jsonb_build_object('batch_id', b.id, 'code', b.code, \
             'name', b.name, 'stage', b.stage, 'planned_date', b.planned_date, \
             'steps', (SELECT COALESCE(jsonb_agg(jsonb_build_object('id', s.id, 'kind', s.kind, \
                 'name', s.name, 'planned_start_at', s.planned_start_at, \
                 'planned_end_at', s.planned_end_at, 'status', s.status) ORDER BY s.position), \
                 '[]'::jsonb) FROM batch_steps s WHERE s.batch_id = b.id AND s.deleted_at IS NULL)) \
             ORDER BY b.planned_date, b.id), '[]'::jsonb) \
         FROM batches b WHERE b.deleted_at IS NULL AND b.stage <> 'cancelled' \
           AND b.planned_date BETWEEN $1 AND $2",
    )
    .bind(q.from)
    .bind(q.to)
    .fetch_one(&mut *tx)
    .await?;
    Ok(Json(json!({"items": items})))
}

/// Plans backward from the day the beer is ready: brew day, end of the fermentation and of the
/// conditioning. The durations come from the fermentation profile of the recipe version.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/calendar/backward-plan",
    tag = "batches",
    operation_id = "calendar_plan",
    params(("workspace_id" = Uuid, Path, description = "Workspace")),
    request_body = BackwardPlan,
    responses((status = 200, description = "The dates of the stages", body = Object,
        example = json!({"brew_date": "2026-10-30", "ready_date": "2026-11-27",
            "fermentation_days": 14, "conditioning_days": 14, "stages": []})))
)]
pub async fn calendar_plan(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<BackwardPlan>,
) -> ApiResult<Json<Value>> {
    ctx.require(READ_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let v = version_json(&mut tx, body.recipe_version_id)
        .await
        .map_err(|_| issue("/recipe_version_id", "not_found"))?;
    Ok(Json(backward_dates(&v, body.ready_date)))
}

// --- routes -------------------------------------------------------------------------------

/// The batch routes, wrapped with the sign-in and idempotency layers.
pub fn routes(state: &AppState) -> Router<AppState> {
    const WS: &str = "/v1/workspaces/{ws}";
    let creates = crate::idempotent(
        state,
        Router::new()
            .route(&format!("{WS}/batches"), post(batch_create))
            .route(
                &format!("{WS}/batches/{{id}}/actuals"),
                post(batchactual_create),
            ),
    );
    let others = Router::new()
        .route(&format!("{WS}/batches"), get(batch_list))
        .route(
            &format!("{WS}/batches/{{id}}"),
            get(batch_get).patch(batch_update).delete(batch_delete),
        )
        .route(
            &format!("{WS}/batches/{{id}}/transitions"),
            post(batch_transition),
        )
        .route(&format!("{WS}/batches/{{id}}/steps"), get(batchstep_list))
        .route(
            &format!("{WS}/batches/{{id}}/steps/{{step_id}}"),
            axum::routing::patch(batchstep_update),
        )
        .route(
            &format!("{WS}/batches/{{id}}/actuals"),
            get(batchactual_list),
        )
        .route(
            &format!("{WS}/batches/{{id}}/actuals/{{line_kind}}/{{line_id}}"),
            put(batchactual_put),
        )
        .route(
            &format!("{WS}/batches/{{id}}/measurements"),
            get(measurement_list).post(measurement_create),
        )
        .route(&format!("{WS}/batches/{{id}}/summary"), get(batch_summary))
        .route(&format!("{WS}/calendar"), get(calendar_list))
        .route(&format!("{WS}/calendar/backward-plan"), post(calendar_plan));
    crate::authenticated(state, creates.merge(others))
}

// --- registry entry ----------------------------------------------------------------------

mod spec {
    use crate::concurrency::Concurrency;
    use crate::error::ErrorCode;
    use crate::error::ErrorCode::{
        Conflict, Forbidden, IdempotencyInProgress, IdempotencyMismatch, Internal,
        MalformedRequest, NotFound, PayloadTooLarge, PreconditionRequired, RateLimited,
        Unauthenticated, Unavailable, UnsupportedMediaType, UpgradeRequired, ValidationFailed,
        VersionMismatch,
    };
    use crate::openapi::{OperationSpec, file_op};

    const LIST: &[ErrorCode] = &[
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
    const READ: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
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
    const UPDATE: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        Conflict,
        VersionMismatch,
        PayloadTooLarge,
        UnsupportedMediaType,
        ValidationFailed,
        PreconditionRequired,
        RateLimited,
        UpgradeRequired,
        Internal,
        Unavailable,
    ];

    pub(super) const OPERATIONS: &[OperationSpec] = &[
        file_op("batch_list", Some("recipe.view"), None, LIST),
        file_op(
            "batch_create",
            Some("batch.plan"),
            Some(Concurrency::Append),
            CREATE,
        ),
        file_op("batch_get", Some("recipe.view"), None, READ),
        file_op(
            "batch_update",
            Some("batch.plan"),
            Some(Concurrency::Optimistic),
            UPDATE,
        ),
        file_op(
            "batch_delete",
            Some("batch.plan"),
            Some(Concurrency::Optimistic),
            UPDATE,
        ),
        file_op(
            "batch_transition",
            Some("batch.record"),
            Some(Concurrency::Optimistic),
            UPDATE,
        ),
        file_op("batchstep_list", Some("recipe.view"), None, READ),
        file_op(
            "batchstep_update",
            Some("batch.record"),
            Some(Concurrency::FieldLww),
            UPDATE,
        ),
        file_op("batchactual_list", Some("recipe.view"), None, READ),
        file_op(
            "batchactual_put",
            Some("batch.record"),
            Some(Concurrency::FieldLww),
            WRITE,
        ),
        file_op(
            "batchactual_create",
            Some("batch.record"),
            Some(Concurrency::Append),
            CREATE,
        ),
        file_op(
            "measurement_create",
            Some("batch.record"),
            Some(Concurrency::Append),
            WRITE,
        ),
        file_op("measurement_list", Some("recipe.view"), None, LIST),
        file_op("batch_summary", Some("recipe.view"), None, READ),
        file_op("calendar_list", Some("recipe.view"), None, LIST),
        file_op("calendar_plan", Some("recipe.view"), None, WRITE),
    ];
}

/// `OpenAPI` fragment of this module.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(
        batch_list,
        batch_create,
        batch_get,
        batch_update,
        batch_delete,
        batch_transition,
        batchstep_list,
        batchstep_update,
        batchactual_list,
        batchactual_put,
        batchactual_create,
        measurement_create,
        measurement_list,
        batch_summary,
        calendar_list,
        calendar_plan
    ),
    components(schemas(
        Batch,
        BatchStep,
        Actual,
        Measurement,
        NewMeasurement,
        NewMeasurements,
        NewBatch,
        BatchPatch,
        Transition,
        StepPatch,
        ActualBody,
        UnplannedActual,
        BackwardPlan
    )),
    tags((name = "batches", description = "Batches, steps, actual amounts, measurements, the brew day package and the calendar"))
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "batches",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
