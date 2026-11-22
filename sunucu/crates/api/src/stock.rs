//! Stock and shopping list (T2.23, ADR-0007 section 3 "Stok").
//!
//! The ledger (`stock_movements`) is the source of truth. `stock_lots.quantity` is a cache that a
//! database trigger updates in the same transaction; the trigger locks the lot row, so two movements
//! at the same time wait for each other and the total stays correct. A negative stock is not
//! refused: the answer of a movement carries a `negative_stock` warning. A count movement carries
//! the counted amount and the trigger calculates the difference under the lot lock.
//!
//! TODO: SSE events for stock changes (outbox topic `stock.changed`) come with the client work.
//! TODO: a yeast line is deducted only in kg or L. A lot in `count` units (packs) needs a pack
//! amount in the actual record of the batch.

#![allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]

use std::collections::BTreeMap;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{FromRow, PgConnection};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    concurrency::{self, Concurrency, check_if_match},
    context::RequestContext,
    error::{ApiError, ApiJson, ApiQuery, ApiResult, FieldIssue, MergePatch},
    library::{created_response, id_cursor, issue, merge, not_found, paging, sqlstate, workspace},
    pagination::{ListParams, Page},
    recipe_org::conflict,
};

/// Permission that moves stock and edits the shopping list.
pub const MOVE_PERMISSION: &str = "stock.move";
/// Permission that reads stock and the shopping list.
pub const READ_PERMISSION: &str = "recipe.view";
/// Permission that shows costs.
pub const COST_PERMISSION: &str = "cost.view";

const UNITS: &[&str] = &["kg", "l", "count"];
/// Amounts below this are zero.
const EPSILON: f64 = 1e-9;

// --- representations ----------------------------------------------------------------------

/// One lot of an ingredient. In Home mode an ingredient has one lot, `default`.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct StockLot {
    /// Identifier.
    pub id: Uuid,
    /// The ingredient.
    pub ingredient_id: Uuid,
    /// Name of the ingredient.
    pub ingredient_name: String,
    /// `fermentable`, `hop`, `yeast`, `misc`.
    pub ingredient_kind: String,
    /// Lot code (`default` in Home mode).
    pub lot_code: String,
    /// Supplier.
    pub supplier: Option<String>,
    /// Day of the goods receipt.
    pub received_on: Option<NaiveDate>,
    /// Best-before day.
    pub expires_on: Option<NaiveDate>,
    /// `kg`, `l`, `count`.
    pub unit: String,
    /// Cost of one unit. Empty for a caller without `cost.view`.
    pub unit_cost: Option<f64>,
    /// Currency of the cost.
    pub cost_currency: Option<String>,
    /// Amount in the unit of the lot (the cache of the ledger total).
    pub quantity: f64,
    /// Notes.
    pub notes: Option<String>,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Updated at.
    pub updated_at: DateTime<Utc>,
    /// Record version (`ETag`).
    pub version: i64,
}

const LOT_SQL: &str = "SELECT l.id, l.ingredient_id, i.name AS ingredient_name, \
    i.kind AS ingredient_kind, l.lot_code, l.supplier, l.received_on, l.expires_on, l.unit, \
    l.unit_cost::float8 AS unit_cost, l.cost_currency::text AS cost_currency, l.quantity, \
    l.notes, l.created_at, l.updated_at, l.version \
    FROM stock_lots l JOIN ingredients i ON i.id = l.ingredient_id WHERE l.deleted_at IS NULL";

/// One row of the ledger.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct StockMovement {
    /// Identifier.
    pub id: Uuid,
    /// The lot.
    pub lot_id: Uuid,
    /// `receive`, `consume`, `adjust`, `count`.
    pub kind: String,
    /// Signed difference in the unit of the lot.
    pub quantity: f64,
    /// Counted amount (`count` only).
    pub counted_quantity: Option<f64>,
    /// The batch (a deduction).
    pub batch_id: Option<Uuid>,
    /// When it happened.
    pub occurred_at: DateTime<Utc>,
    /// Reason.
    pub reason: Option<String>,
    /// Written at.
    pub created_at: DateTime<Utc>,
}

const MOVEMENT_COLUMNS: &str =
    "id, lot_id, kind, quantity, counted_quantity, batch_id, occurred_at, reason, created_at";

/// A warning that comes with a stock answer.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StockWarning {
    /// `negative_stock`.
    pub kind: String,
    /// The lot (empty in a total of an ingredient).
    pub lot_id: Option<Uuid>,
    /// The ingredient.
    pub ingredient_id: Uuid,
    /// Name of the ingredient.
    pub ingredient_name: String,
    /// The amount that is below zero.
    pub quantity: f64,
}

/// The answer of a stock movement.
#[derive(Debug, Serialize, ToSchema)]
pub struct MovementResult {
    /// The ledger row.
    pub movement: StockMovement,
    /// The lot after the movement.
    pub lot: StockLot,
    /// `negative_stock` when the lot is below zero.
    pub warnings: Vec<StockWarning>,
}

/// The total of one ingredient and unit, calculated from the ledger.
#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct StockTotal {
    /// The ingredient.
    pub ingredient_id: Uuid,
    /// Name of the ingredient.
    pub ingredient_name: String,
    /// `fermentable`, `hop`, `yeast`, `misc`.
    pub ingredient_kind: String,
    /// `kg`, `l`, `count`.
    pub unit: String,
    /// Number of lots.
    pub lots: i64,
    /// Sum of the ledger rows of the lots.
    pub quantity: f64,
    /// Sum of the cached lot amounts (must equal `quantity`).
    pub cached_quantity: f64,
}

/// An item of the shopping list.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct ShoppingItem {
    /// Identifier.
    pub id: Uuid,
    /// The ingredient.
    pub ingredient_id: Uuid,
    /// Name of the ingredient.
    pub ingredient_name: String,
    /// Amount to buy.
    pub quantity: f64,
    /// `kg`, `l`, `count`.
    pub unit: String,
    /// The recipe version that put the item on the list.
    pub source_recipe_version_id: Option<Uuid>,
    /// Bought (ticked off).
    pub checked: bool,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Updated at.
    pub updated_at: DateTime<Utc>,
    /// Record version (`ETag`).
    pub version: i64,
}

const SHOPPING_SQL: &str = "SELECT s.id, s.ingredient_id, i.name AS ingredient_name, s.quantity, \
    s.unit, s.source_recipe_version_id, s.checked, s.created_at, s.updated_at, s.version \
    FROM shopping_list_items s JOIN ingredients i ON i.id = s.ingredient_id \
    WHERE s.deleted_at IS NULL";

/// An ingredient that stock already covers.
#[derive(Debug, Serialize, ToSchema)]
pub struct CoveredLine {
    /// The ingredient.
    pub ingredient_id: Uuid,
    /// Name of the ingredient.
    pub ingredient_name: String,
    /// Amount the recipe needs.
    pub required: f64,
    /// Amount in stock.
    pub on_hand: f64,
    /// `kg`, `l`, `count`.
    pub unit: String,
}

/// The answer of a shopping list build.
#[derive(Debug, Serialize, ToSchema)]
pub struct ShoppingResult {
    /// The items that were added or renewed.
    pub items: Vec<ShoppingItem>,
    /// The ingredients that stock covers (not on the list).
    pub covered: Vec<CoveredLine>,
    /// Names of recipe lines that have no ingredient record (not on the list).
    pub unlinked: Vec<String>,
}

/// A line that a batch deduction did not move.
#[derive(Debug, Serialize, ToSchema)]
pub struct SkippedLine {
    /// The ingredient (empty for a line without one).
    pub ingredient_id: Option<Uuid>,
    /// The amount that was not deducted.
    pub amount: f64,
    /// `no_ingredient`, `no_lot`, `unit_mismatch`, `already_deducted`.
    pub reason: String,
}

/// The answer of a batch deduction.
#[derive(Debug, Serialize, ToSchema)]
pub struct DeductionResult {
    /// The batch.
    pub batch_id: Uuid,
    /// The ledger rows that this call wrote.
    pub movements: Vec<StockMovement>,
    /// Lines that this call did not move.
    pub skipped: Vec<SkippedLine>,
    /// `negative_stock` for each lot below zero.
    pub warnings: Vec<StockWarning>,
}

/// The totals answer.
#[derive(Debug, Serialize, ToSchema)]
pub struct Totals {
    /// One row for each ingredient and unit.
    pub items: Vec<StockTotal>,
    /// `negative_stock` for each ingredient below zero.
    pub warnings: Vec<StockWarning>,
}

// --- request bodies -----------------------------------------------------------------------

/// Body of a goods receipt.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    /// The ingredient (a library or workspace ingredient).
    pub ingredient_id: Uuid,
    /// Received amount, greater than zero.
    pub quantity: f64,
    /// `kg` (default), `l` or `count`.
    pub unit: Option<String>,
    /// Lot code (default `default`). A known code adds to that lot.
    pub lot_code: Option<String>,
    /// Supplier.
    pub supplier: Option<String>,
    /// Day of the receipt (default today).
    pub received_on: Option<NaiveDate>,
    /// Best-before day.
    pub expires_on: Option<NaiveDate>,
    /// Cost of one unit; needs `cost_currency`.
    pub unit_cost: Option<f64>,
    /// Currency of the cost (ISO 4217).
    pub cost_currency: Option<String>,
    /// Notes of the lot.
    pub notes: Option<String>,
    /// When it happened (default now).
    pub occurred_at: Option<DateTime<Utc>>,
}

/// Body of a movement on a lot.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct MoveBody {
    /// `receive` (amount in), `consume` (amount out), `adjust` (signed correction, needs a
    /// reason) or `count` (stock count).
    pub kind: String,
    /// `receive` and `consume`: the amount, greater than zero. `adjust`: the signed difference.
    pub quantity: Option<f64>,
    /// `count`: the counted amount, zero or more. The server calculates the difference.
    pub counted_quantity: Option<f64>,
    /// Reason.
    pub reason: Option<String>,
    /// When it happened (default now).
    pub occurred_at: Option<DateTime<Utc>>,
    /// The batch that used the amount.
    pub batch_id: Option<Uuid>,
}

/// Body of a lot patch (JSON Merge Patch).
#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct LotPatch {
    /// Supplier.
    pub supplier: Option<String>,
    /// Best-before day.
    pub expires_on: Option<NaiveDate>,
    /// Cost of one unit; needs `cost_currency`.
    pub unit_cost: Option<f64>,
    /// Currency of the cost.
    pub cost_currency: Option<String>,
    /// Notes.
    pub notes: Option<String>,
}

/// Body of a shopping list build from a recipe.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct FromRecipe {
    /// The recipe version.
    pub recipe_version_id: Uuid,
    /// Scale the amounts to this batch volume, L (default: the volume of the recipe).
    pub batch_volume_l: Option<f64>,
    /// Subtract the stock on hand (default true).
    pub subtract_stock: Option<bool>,
}

/// Filter of the lot list.
#[derive(Debug, Default, Deserialize)]
pub struct LotFilter {
    /// The ingredient.
    pub ingredient_id: Option<Uuid>,
    /// Text in the ingredient name, lot code or supplier (case blind).
    pub q: Option<String>,
    /// Only lots below zero.
    pub negative: Option<bool>,
}

/// Filter of the movement list.
#[derive(Debug, Default, Deserialize)]
pub struct MovementFilter {
    /// The lot.
    pub lot_id: Option<Uuid>,
    /// The batch.
    pub batch_id: Option<Uuid>,
    /// `receive`, `consume`, `adjust`, `count`.
    pub kind: Option<String>,
}

/// Filter of the shopping list.
#[derive(Debug, Default, Deserialize)]
pub struct ShoppingFilter {
    /// Only ticked (or only open) items.
    pub checked: Option<bool>,
}

// --- helpers ------------------------------------------------------------------------------

fn constraint_error(e: sqlx::Error) -> ApiError {
    match sqlstate(&e).as_deref() {
        Some("23514" | "22003" | "22P02") => issue("/", "constraint_violated"),
        Some("23503") => issue("/", "reference_not_found"),
        Some("23505") => conflict("/", "already_exists"),
        _ => e.into(),
    }
}

fn lot_example() -> Value {
    json!({
        "id": "0192f1c07a3b7000800000000000000b",
        "ingredient_id": "0192f1c07a3b7000800000000000000c", "ingredient_name": "Pale malt",
        "ingredient_kind": "fermentable", "lot_code": "default", "supplier": null,
        "received_on": "2026-11-20", "expires_on": null, "unit": "kg", "unit_cost": 2.4,
        "cost_currency": "USD", "quantity": 25.0, "notes": null,
        "created_at": "2026-11-20T10:00:00Z", "updated_at": "2026-11-20T10:00:00Z", "version": 1
    })
}

fn movement_example() -> Value {
    json!({
        "id": "0192f1c07a3b7000800000000000000d",
        "lot_id": "0192f1c07a3b7000800000000000000b", "kind": "receive", "quantity": 25.0,
        "counted_quantity": null, "batch_id": null, "occurred_at": "2026-11-20T10:00:00Z",
        "reason": null, "created_at": "2026-11-20T10:00:00Z"
    })
}

fn result_example() -> Value {
    json!({"movement": movement_example(), "lot": lot_example(), "warnings": []})
}

fn shopping_example() -> Value {
    json!({
        "id": "0192f1c07a3b7000800000000000000e",
        "ingredient_id": "0192f1c07a3b7000800000000000000c", "ingredient_name": "Pale malt",
        "quantity": 4.5, "unit": "kg",
        "source_recipe_version_id": "0192f1c07a3b7000800000000000000f", "checked": false,
        "created_at": "2026-11-20T10:00:00Z", "updated_at": "2026-11-20T10:00:00Z", "version": 1
    })
}

fn finite_positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn hide_cost(ctx: &RequestContext, lot: &mut StockLot) {
    if !ctx.has_permission(COST_PERMISSION) {
        lot.unit_cost = None;
        lot.cost_currency = None;
    }
}

async fn fetch_lot(tx: &mut PgConnection, id: Uuid, lock: bool) -> ApiResult<StockLot> {
    let lock = if lock { " FOR UPDATE OF l" } else { "" };
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{LOT_SQL} AND l.id = $1{lock}"
    )))
    .bind(id)
    .fetch_optional(tx)
    .await?
    .ok_or_else(not_found)
}

fn warning_of(lot: &StockLot) -> Option<StockWarning> {
    (lot.quantity < -EPSILON).then(|| StockWarning {
        kind: "negative_stock".to_owned(),
        lot_id: Some(lot.id),
        ingredient_id: lot.ingredient_id,
        ingredient_name: lot.ingredient_name.clone(),
        quantity: lot.quantity,
    })
}

struct NewMovement<'a> {
    kind: &'a str,
    quantity: f64,
    counted_quantity: Option<f64>,
    batch_id: Option<Uuid>,
    occurred_at: Option<DateTime<Utc>>,
    reason: Option<&'a str>,
}

/// Writes one ledger row. The trigger of the table updates the lot under the row lock.
async fn insert_movement(
    tx: &mut PgConnection,
    ws: Uuid,
    user: Uuid,
    lot_id: Uuid,
    m: &NewMovement<'_>,
) -> ApiResult<StockMovement> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "INSERT INTO stock_movements (id, workspace_id, lot_id, kind, quantity, counted_quantity, \
             batch_id, occurred_at, reason, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, COALESCE($8, now()), $9, $10) \
         RETURNING {MOVEMENT_COLUMNS}"
    )))
    .bind(Uuid::now_v7())
    .bind(ws)
    .bind(lot_id)
    .bind(m.kind)
    .bind(m.quantity)
    .bind(m.counted_quantity)
    .bind(m.batch_id)
    .bind(m.occurred_at)
    .bind(m.reason)
    .bind(user)
    .fetch_one(tx)
    .await
    .map_err(|e| match sqlstate(&e).as_deref() {
        Some("23503") => issue("/batch_id", "not_found"),
        _ => constraint_error(e),
    })
}

async fn result_of(
    tx: &mut PgConnection,
    ctx: &RequestContext,
    movement: StockMovement,
) -> ApiResult<MovementResult> {
    let mut lot = fetch_lot(tx, movement.lot_id, false).await?;
    let warnings = warning_of(&lot).into_iter().collect();
    hide_cost(ctx, &mut lot);
    Ok(MovementResult {
        movement,
        lot,
        warnings,
    })
}

fn check_cost(cost: Option<f64>, currency: Option<&str>) -> ApiResult<()> {
    if cost.is_some_and(|c| !c.is_finite() || c < 0.0) {
        return Err(issue("/unit_cost", "out_of_range"));
    }
    if cost.is_some() != currency.is_some() {
        return Err(issue("/cost_currency", "required_with_unit_cost"));
    }
    if currency.is_some_and(|c| c.len() != 3 || !c.chars().all(|ch| ch.is_ascii_uppercase())) {
        return Err(issue("/cost_currency", "invalid_value"));
    }
    Ok(())
}

// --- lot endpoints ------------------------------------------------------------------------

/// Lists the stock lots of the workspace.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/stock/lots",
    tag = "stock",
    operation_id = "stock_lot_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("ingredient_id" = Option<Uuid>, Query, description = "The ingredient"),
        ("q" = Option<String>, Query, description = "Text in the ingredient name, lot code or supplier (case blind)"),
        ("negative" = Option<bool>, Query, description = "Only lots below zero"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page"),
        ("sort" = Option<String>, Query, description = "`id` (creation order, default) or `-id`")
    ),
    responses((status = 200, description = "A page of lots", body = Page<StockLot>,
        example = json!({"items": [lot_example()], "next_cursor": null})))
)]
pub async fn stock_lot_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<LotFilter>,
) -> ApiResult<Json<Page<StockLot>>> {
    ctx.require(READ_PERMISSION)?;
    let paging = paging(&params, &["id"], "-id")?;
    let (cmp, dir) = paging.dir();
    let sql = sqlx::AssertSqlSafe(format!(
        "{LOT_SQL} AND ($1::uuid IS NULL OR l.ingredient_id = $1) \
           AND ($2::text IS NULL OR strpos(lower(i.name || ' ' || l.lot_code || ' ' \
               || coalesce(l.supplier, '')), lower($2)) > 0) \
           AND (NOT coalesce($3, false) OR l.quantity < 0) \
           AND ($4::uuid IS NULL OR l.id {cmp} $4) \
         ORDER BY l.id {dir} LIMIT $5"
    ));
    let mut tx = ctx.begin(&state.db).await?;
    let mut rows: Vec<StockLot> = sqlx::query_as(sql)
        .bind(filter.ingredient_id)
        .bind(&filter.q)
        .bind(filter.negative)
        .bind(paging.id)
        .bind(i64::from(paging.limit) + 1)
        .fetch_all(&mut *tx)
        .await?;
    for lot in &mut rows {
        hide_cost(&ctx, lot);
    }
    Ok(Json(Page::from_rows(
        rows,
        paging.limit,
        &paging.sort,
        |r| id_cursor(r.id),
    )))
}

/// Reads one lot.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/stock/lots/{id}",
    tag = "stock",
    operation_id = "stock_lot_get",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Lot")
    ),
    responses(
        (status = 200, description = "The lot", body = StockLot, example = json!(lot_example())),
        (status = 304, description = "Not modified (`If-None-Match` matched)")
    )
)]
pub async fn stock_lot_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    ctx.require(READ_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let mut lot = fetch_lot(&mut tx, id, false).await?;
    hide_cost(&ctx, &mut lot);
    Ok(concurrency::conditional_get(&headers, lot.version, &lot))
}

/// Changes the supplier, best-before day, cost or notes of a lot. The amount changes only with a
/// movement.
#[utoipa::path(
    patch,
    path = "/v1/workspaces/{workspace_id}/stock/lots/{id}",
    tag = "stock",
    operation_id = "stock_lot_update",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Lot"),
        ("If-Match" = String, Header, description = "Version `ETag` of the lot")
    ),
    request_body(content = LotPatch, content_type = "application/merge-patch+json"),
    responses((status = 200, description = "The updated lot", body = StockLot,
        example = json!(lot_example())))
)]
pub async fn stock_lot_update(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    MergePatch(patch): MergePatch<Value>,
) -> ApiResult<Response> {
    ctx.require(MOVE_PERMISSION)?;
    if !patch.is_object() {
        return Err(issue("/", "invalid"));
    }
    let mut tx = ctx.begin(&state.db).await?;
    let before = fetch_lot(&mut tx, id, true).await?;
    check_if_match(Concurrency::Optimistic, &headers, before.version, || {
        json!(before)
    })?;
    let mut merged = json!({
        "supplier": before.supplier, "expires_on": before.expires_on,
        "unit_cost": before.unit_cost, "cost_currency": before.cost_currency,
        "notes": before.notes
    });
    merge(&mut merged, &patch);
    let input: LotPatch = serde_json::from_value(merged).map_err(|e| {
        ApiError::validation(vec![
            FieldIssue::new("/", "invalid").with_params(json!({"reason": e.to_string()})),
        ])
    })?;
    check_cost(input.unit_cost, input.cost_currency.as_deref())?;
    sqlx::query(
        "UPDATE stock_lots SET supplier = $2, expires_on = $3, unit_cost = $4::float8::numeric, \
             cost_currency = $5, notes = $6, updated_by = $7 WHERE id = $1",
    )
    .bind(id)
    .bind(&input.supplier)
    .bind(input.expires_on)
    .bind(input.unit_cost)
    .bind(&input.cost_currency)
    .bind(&input.notes)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    let mut lot = fetch_lot(&mut tx, id, false).await?;
    tx.commit().await?;
    hide_cost(&ctx, &mut lot);
    Ok(concurrency::record_response(
        StatusCode::OK,
        lot.version,
        &lot,
    ))
}

/// Receives goods: finds or creates the lot (ingredient and lot code) and writes a `receive` row.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/stock/receipts",
    tag = "stock",
    operation_id = "stock_receive",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = Receipt,
    responses((status = 201, description = "The movement and the lot", body = MovementResult,
        example = json!(result_example())))
)]
pub async fn stock_receive(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<Receipt>,
) -> ApiResult<Response> {
    ctx.require(MOVE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    if !finite_positive(body.quantity) {
        return Err(issue("/quantity", "out_of_range"));
    }
    let unit = body.unit.clone().unwrap_or_else(|| "kg".to_owned());
    if !UNITS.contains(&unit.as_str()) {
        return Err(issue("/unit", "invalid_value"));
    }
    let code = body
        .lot_code
        .clone()
        .unwrap_or_else(|| "default".to_owned());
    if code.is_empty() || code.chars().count() > 100 {
        return Err(issue("/lot_code", "invalid_length"));
    }
    check_cost(body.unit_cost, body.cost_currency.as_deref())?;
    let user = ctx.identity.user_id;
    let mut tx = ctx.begin(&state.db).await?;
    let (lot_id, lot_unit): (Uuid, String) = sqlx::query_as(
        "INSERT INTO stock_lots (id, workspace_id, ingredient_id, lot_code, supplier, received_on, \
             expires_on, unit, unit_cost, cost_currency, notes, created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, COALESCE($6, CURRENT_DATE), $7, $8, $9::float8::numeric, \
             $10, $11, $12, $12) \
         ON CONFLICT (workspace_id, ingredient_id, lot_code) WHERE deleted_at IS NULL \
         DO UPDATE SET supplier = COALESCE(EXCLUDED.supplier, stock_lots.supplier), \
             received_on = COALESCE($6, stock_lots.received_on), \
             expires_on = COALESCE(EXCLUDED.expires_on, stock_lots.expires_on), \
             unit_cost = COALESCE(EXCLUDED.unit_cost, stock_lots.unit_cost), \
             cost_currency = CASE WHEN EXCLUDED.unit_cost IS NULL THEN stock_lots.cost_currency \
                 ELSE EXCLUDED.cost_currency END, \
             notes = COALESCE(EXCLUDED.notes, stock_lots.notes), \
             updated_by = EXCLUDED.updated_by \
         RETURNING id, unit",
    )
    .bind(Uuid::now_v7())
    .bind(ws)
    .bind(body.ingredient_id)
    .bind(&code)
    .bind(&body.supplier)
    .bind(body.received_on)
    .bind(body.expires_on)
    .bind(&unit)
    .bind(body.unit_cost)
    .bind(&body.cost_currency)
    .bind(&body.notes)
    .bind(user)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match sqlstate(&e).as_deref() {
        Some("23503") => issue("/ingredient_id", "not_found"),
        _ => constraint_error(e),
    })?;
    if lot_unit != unit {
        // The lot exists with another unit. The transaction ends without a commit.
        return Err(conflict("/unit", "unit_mismatch"));
    }
    let movement = insert_movement(
        &mut tx,
        ws,
        user,
        lot_id,
        &NewMovement {
            kind: "receive",
            quantity: body.quantity,
            counted_quantity: None,
            batch_id: None,
            occurred_at: body.occurred_at,
            reason: None,
        },
    )
    .await?;
    let result = result_of(&mut tx, &ctx, movement).await?;
    tx.commit().await?;
    Ok(created_response(
        &format!("/v1/workspaces/{ws}/stock/lots/{lot_id}"),
        result.lot.version,
        &result,
    ))
}

/// Writes a movement on a lot: `receive`, `consume`, `adjust` or `count`. The answer carries a
/// `negative_stock` warning when the lot is below zero afterwards.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/stock/lots/{id}/movements",
    tag = "stock",
    operation_id = "stock_move",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Lot"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = MoveBody,
    responses((status = 201, description = "The movement and the lot", body = MovementResult,
        example = json!(result_example())))
)]
pub async fn stock_move(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    ApiJson(body): ApiJson<MoveBody>,
) -> ApiResult<Response> {
    ctx.require(MOVE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let reason = body
        .reason
        .as_deref()
        .map(str::trim)
        .filter(|r| !r.is_empty());
    let (quantity, counted) = match body.kind.as_str() {
        "receive" | "consume" => {
            let q = body
                .quantity
                .filter(|q| finite_positive(*q))
                .ok_or_else(|| issue("/quantity", "out_of_range"))?;
            (if body.kind == "consume" { -q } else { q }, None)
        }
        "adjust" => {
            let q = body
                .quantity
                .filter(|q| q.is_finite() && q.abs() > EPSILON)
                .ok_or_else(|| issue("/quantity", "out_of_range"))?;
            if reason.is_none() {
                return Err(issue("/reason", "required"));
            }
            (q, None)
        }
        "count" => {
            let c = body
                .counted_quantity
                .filter(|c| c.is_finite() && *c >= 0.0)
                .ok_or_else(|| issue("/counted_quantity", "out_of_range"))?;
            (0.0, Some(c))
        }
        _ => return Err(issue("/kind", "invalid_value")),
    };
    let mut tx = ctx.begin(&state.db).await?;
    fetch_lot(&mut tx, id, false).await?;
    let movement = insert_movement(
        &mut tx,
        ws,
        ctx.identity.user_id,
        id,
        &NewMovement {
            kind: &body.kind,
            quantity,
            counted_quantity: counted,
            batch_id: body.batch_id,
            occurred_at: body.occurred_at,
            reason,
        },
    )
    .await?;
    let result = result_of(&mut tx, &ctx, movement).await?;
    tx.commit().await?;
    Ok(created_response(
        &format!("/v1/workspaces/{ws}/stock/lots/{id}"),
        result.lot.version,
        &result,
    ))
}

/// Lists the ledger rows.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/stock/movements",
    tag = "stock",
    operation_id = "stock_movement_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("lot_id" = Option<Uuid>, Query, description = "The lot"),
        ("batch_id" = Option<Uuid>, Query, description = "The batch"),
        ("kind" = Option<String>, Query, description = "`receive`, `consume`, `adjust`, `count`"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page"),
        ("sort" = Option<String>, Query, description = "`id` (write order) or `-id` (default)")
    ),
    responses((status = 200, description = "A page of ledger rows", body = Page<StockMovement>,
        example = json!({"items": [movement_example()], "next_cursor": null})))
)]
pub async fn stock_movement_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<MovementFilter>,
) -> ApiResult<Json<Page<StockMovement>>> {
    ctx.require(READ_PERMISSION)?;
    let paging = paging(&params, &["id"], "-id")?;
    if filter
        .kind
        .as_deref()
        .is_some_and(|k| !matches!(k, "receive" | "consume" | "adjust" | "count"))
    {
        return Err(issue("kind", "invalid_value"));
    }
    let (cmp, dir) = paging.dir();
    let sql = sqlx::AssertSqlSafe(format!(
        "SELECT {MOVEMENT_COLUMNS} FROM stock_movements \
         WHERE ($1::uuid IS NULL OR lot_id = $1) AND ($2::uuid IS NULL OR batch_id = $2) \
           AND ($3::text IS NULL OR kind = $3) AND ($4::uuid IS NULL OR id {cmp} $4) \
         ORDER BY id {dir} LIMIT $5"
    ));
    let mut tx = ctx.begin(&state.db).await?;
    let rows: Vec<StockMovement> = sqlx::query_as(sql)
        .bind(filter.lot_id)
        .bind(filter.batch_id)
        .bind(&filter.kind)
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

/// Totals for each ingredient and unit, calculated from the ledger rows. An ingredient below zero
/// gets a `negative_stock` warning.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/stock/totals",
    tag = "stock",
    operation_id = "stock_totals",
    params(("workspace_id" = Uuid, Path, description = "Workspace")),
    responses((status = 200, description = "All totals (no paging)", body = Totals,
        example = json!({"items": [{"ingredient_id": "0192f1c07a3b7000800000000000000c",
            "ingredient_name": "Pale malt", "ingredient_kind": "fermentable", "unit": "kg",
            "lots": 1, "quantity": 25.0, "cached_quantity": 25.0}], "warnings": []})))
)]
pub async fn stock_totals(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> ApiResult<Json<Totals>> {
    ctx.require(READ_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let items: Vec<StockTotal> = sqlx::query_as(
        "SELECT i.id AS ingredient_id, i.name AS ingredient_name, i.kind AS ingredient_kind, \
             t.unit, t.lots, t.quantity, t.cached_quantity \
         FROM ( \
             SELECT l.ingredient_id, l.unit, count(*) AS lots, \
                 COALESCE(sum((SELECT COALESCE(sum(m.quantity), 0) FROM stock_movements m \
                     WHERE m.workspace_id = l.workspace_id AND m.lot_id = l.id)), 0)::float8 \
                     AS quantity, \
                 COALESCE(sum(l.quantity), 0)::float8 AS cached_quantity \
             FROM stock_lots l WHERE l.deleted_at IS NULL GROUP BY l.ingredient_id, l.unit \
         ) t JOIN ingredients i ON i.id = t.ingredient_id \
         ORDER BY i.name, t.unit",
    )
    .fetch_all(&mut *tx)
    .await?;
    let warnings = items
        .iter()
        .filter(|t| t.quantity < -EPSILON)
        .map(|t| StockWarning {
            kind: "negative_stock".to_owned(),
            lot_id: None,
            ingredient_id: t.ingredient_id,
            ingredient_name: t.ingredient_name.clone(),
            quantity: t.quantity,
        })
        .collect();
    Ok(Json(Totals { items, warnings }))
}

// --- deduction for a batch ----------------------------------------------------------------

#[derive(FromRow)]
struct Need {
    ingredient_id: Option<Uuid>,
    lot_id: Option<Uuid>,
    amount: f64,
    unit: String,
}

#[derive(FromRow)]
struct Candidate {
    id: Uuid,
    quantity: f64,
}

/// Deducts the actual amounts of a batch from stock. Each ingredient is taken from the oldest lots
/// first (best-before day, then receipt day); a lot that the actual record names comes first. The
/// call is safe to repeat: it deducts only what the earlier calls did not deduct. A shortage goes
/// to the last lot, which then shows a `negative_stock` warning.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/batches/{id}/stock-deduction",
    tag = "stock",
    operation_id = "stock_deduct",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Batch"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    responses((status = 200, description = "The rows that this call wrote", body = DeductionResult,
        example = json!({"batch_id": "0192f1c07a3b7000800000000000000a",
            "movements": [movement_example()], "skipped": [], "warnings": []})))
)]
pub async fn stock_deduct(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, batch_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<DeductionResult>> {
    ctx.require(MOVE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let user = ctx.identity.user_id;
    let mut tx = ctx.begin(&state.db).await?;
    // The batch lock makes two deductions of one batch wait for each other.
    let batch_stage: String = sqlx::query_scalar(
        "SELECT stage FROM batches WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
    )
    .bind(batch_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(not_found)?;
    if batch_stage == "cancelled" {
        return Err(conflict("/stage", "batch_cancelled"));
    }
    let needs: Vec<Need> = sqlx::query_as(
        "SELECT COALESCE(a.substitute_ingredient_id, p.ingredient_id) AS ingredient_id, \
             a.lot_id, \
             CASE WHEN a.line_kind = 'yeast' AND p.unit = 'count' THEN ceil(p.amount) \
                  ELSE a.actual_amount_kg END AS amount, \
             COALESCE(p.unit, 'kg') AS unit \
         FROM batch_ingredient_actuals a \
         LEFT JOIN LATERAL ( \
             SELECT ingredient_id, 'kg'::text AS unit, amount_kg AS amount \
                 FROM recipe_fermentables WHERE a.line_kind = 'fermentable' AND id = a.line_id \
             UNION ALL SELECT ingredient_id, 'kg', amount_kg \
                 FROM recipe_hops WHERE a.line_kind = 'hop' AND id = a.line_id \
             UNION ALL SELECT ingredient_id, CASE WHEN use_packages THEN 'count' ELSE 'kg' END, \
                 CASE WHEN use_packages THEN packs ELSE amount_kg END \
                 FROM recipe_yeasts WHERE a.line_kind = 'yeast' AND id = a.line_id \
             UNION ALL SELECT ingredient_id, CASE WHEN is_liquid THEN 'l' ELSE 'kg' END, amount \
                 FROM recipe_miscs WHERE a.line_kind = 'misc' AND id = a.line_id \
         ) p ON true \
         WHERE a.batch_id = $1 AND a.deleted_at IS NULL \
           AND (a.actual_amount_kg > 0 OR (a.line_kind = 'yeast' AND p.unit = 'count' AND p.amount > 0)) \
         ORDER BY a.created_at, a.id",
    )

    // One entry for each ingredient: the total amount and the preferred lot.
    let mut skipped = Vec::new();
    let mut per_ingredient: BTreeMap<(Uuid, String), (f64, Option<Uuid>)> = BTreeMap::new();
    for need in needs {
        let Some(ingredient) = need.ingredient_id else {
            skipped.push(SkippedLine {
                ingredient_id: None,
                amount: need.amount,
                reason: "no_ingredient".to_owned(),
            });
            continue;
        };
        let entry = per_ingredient
            .entry((ingredient, need.unit))
            .or_insert((0.0, None));
        entry.0 += need.amount;
        entry.1 = entry.1.or(need.lot_id);
    }

    let mut movements = Vec::new();
    let mut touched: Vec<Uuid> = Vec::new();
    for ((ingredient, unit), (total, preferred)) in per_ingredient {
        let done: f64 = sqlx::query_scalar(
            "SELECT COALESCE(-sum(m.quantity), 0)::float8 FROM stock_movements m \
             JOIN stock_lots l ON l.workspace_id = m.workspace_id AND l.id = m.lot_id \
             WHERE m.batch_id = $1 AND m.kind = 'consume' AND l.ingredient_id = $2 \
               AND (($3 = 'count' AND l.unit = 'count') OR ($3 <> 'count' AND l.unit <> 'count'))",
        )
        .bind(batch_id)
        .bind(ingredient)
        .bind(&unit)
        .fetch_one(&mut *tx)
        .await?;
        let mut remaining = total - done;
        if remaining <= EPSILON {
            skipped.push(SkippedLine {
                ingredient_id: Some(ingredient),
                amount: total,
                reason: "already_deducted".to_owned(),
            });
            continue;
        }
        // Lots in kg or L, oldest first, the preferred lot at the front.
        let mut lots: Vec<Candidate> = sqlx::query_as(
            "SELECT id, quantity FROM stock_lots \
             WHERE ingredient_id = $1 AND deleted_at IS NULL \
               AND (($3 = 'count' AND unit = 'count') OR ($3 <> 'count' AND unit <> 'count')) \
             ORDER BY (id = $2) DESC, expires_on NULLS LAST, received_on NULLS LAST, \
                 created_at, id FOR UPDATE",
        )
        .bind(ingredient)
        .bind(preferred)
        .bind(&unit)
        .fetch_all(&mut *tx)
        .await?;
        if lots.is_empty() {
            let has_count_lot: bool = sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM stock_lots \
                 WHERE ingredient_id = $1 AND deleted_at IS NULL)",
            )
            .bind(ingredient)
            .fetch_one(&mut *tx)
            .await?;
            skipped.push(SkippedLine {
                ingredient_id: Some(ingredient),
                amount: remaining,
                reason: if has_count_lot {
                    "unit_mismatch"
                } else {
                    "no_lot"
                }
                .to_owned(),
            });
            continue;
        }
        // The preferred lot takes the whole amount; otherwise the lots in order, and the shortage
        // goes to the last lot.
        let preferred_first = preferred.is_some_and(|p| lots[0].id == p);
        if preferred_first {
            lots.truncate(1);
        }
        let last = lots.len() - 1;
        for (i, lot) in lots.iter().enumerate() {
            let take = if i == last {
                remaining
            } else {
                lot.quantity.max(0.0).min(remaining)
            };
            if take <= EPSILON {
                continue;
            }
            movements.push(
                insert_movement(
                    &mut tx,
                    ws,
                    user,
                    lot.id,
                    &NewMovement {
                        kind: "consume",
                        quantity: -take,
                        counted_quantity: None,
                        batch_id: Some(batch_id),
                        occurred_at: None,
                        reason: Some("batch deduction"),
                    },
                )
                .await?,
            );
            touched.push(lot.id);
            remaining -= take;
            if remaining <= EPSILON {
                break;
            }
        }
    }

    let mut warnings = Vec::new();
    touched.sort();
    touched.dedup();
    for id in touched {
        if let Some(w) = warning_of(&fetch_lot(&mut tx, id, false).await?) {
            warnings.push(w);
        }
    }
    tx.commit().await?;
    Ok(Json(DeductionResult {
        batch_id,
        movements,
        skipped,
        warnings,
    }))
}

// --- shopping list ------------------------------------------------------------------------

/// Lists the shopping list.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/shopping-list",
    tag = "stock",
    operation_id = "shopping_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("checked" = Option<bool>, Query, description = "Only ticked (or only open) items"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page"),
        ("sort" = Option<String>, Query, description = "`id` (creation order, default) or `-id`")
    ),
    responses((status = 200, description = "A page of items", body = Page<ShoppingItem>,
        example = json!({"items": [shopping_example()], "next_cursor": null})))
)]
pub async fn shopping_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<ShoppingFilter>,
) -> ApiResult<Json<Page<ShoppingItem>>> {
    ctx.require(READ_PERMISSION)?;
    let paging = paging(&params, &["id"], "id")?;
    let (cmp, dir) = paging.dir();
    let sql = sqlx::AssertSqlSafe(format!(
        "{SHOPPING_SQL} AND ($1::boolean IS NULL OR s.checked = $1) \
           AND ($2::uuid IS NULL OR s.id {cmp} $2) ORDER BY s.id {dir} LIMIT $3"
    ));
    let mut tx = ctx.begin(&state.db).await?;
    let rows: Vec<ShoppingItem> = sqlx::query_as(sql)
        .bind(filter.checked)
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

#[derive(FromRow)]
struct RecipeLine {
    ingredient_id: Option<Uuid>,
    name: String,
    unit: String,
    quantity: f64,
}

/// Builds shopping list items from a recipe version: the amounts of the recipe (scaled to the batch
/// volume) minus the stock on hand. An item for the same ingredient, unit and recipe version that
/// is still open gets a new amount, so the call is safe to repeat.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/shopping-list/from-recipe",
    tag = "stock",
    operation_id = "shopping_from_recipe",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = FromRecipe,
    responses((status = 201, description = "The items and the covered lines", body = ShoppingResult,
        example = json!({"items": [shopping_example()], "covered": [], "unlinked": []})))
)]
pub async fn shopping_from_recipe(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<FromRecipe>,
) -> ApiResult<Response> {
    ctx.require(MOVE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let user = ctx.identity.user_id;
    if body.batch_volume_l.is_some_and(|v| !finite_positive(v)) {
        return Err(issue("/batch_volume_l", "out_of_range"));
    }
    let mut tx = ctx.begin(&state.db).await?;
    let recipe_volume: f64 =
        sqlx::query_scalar("SELECT batch_volume_l FROM recipe_versions WHERE id = $1")
            .bind(body.recipe_version_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| issue("/recipe_version_id", "not_found"))?;
    let factor = body.batch_volume_l.map_or(1.0, |v| v / recipe_volume);
    let lines: Vec<RecipeLine> = sqlx::query_as(
        "SELECT ingredient_id, name, 'kg' AS unit, amount_kg AS quantity \
             FROM recipe_fermentables WHERE recipe_version_id = $1 AND deleted_at IS NULL \
         UNION ALL SELECT ingredient_id, name, 'kg', amount_kg \
             FROM recipe_hops WHERE recipe_version_id = $1 AND deleted_at IS NULL \
         UNION ALL SELECT ingredient_id, name, \
                 CASE WHEN use_packages THEN 'count' ELSE 'kg' END, \
                 CASE WHEN use_packages THEN packs ELSE amount_kg END \
             FROM recipe_yeasts WHERE recipe_version_id = $1 AND deleted_at IS NULL \
         UNION ALL SELECT ingredient_id, name, CASE WHEN is_liquid THEN 'l' ELSE 'kg' END, amount \
             FROM recipe_miscs WHERE recipe_version_id = $1 AND deleted_at IS NULL",
    )
    .bind(body.recipe_version_id)
    .fetch_all(&mut *tx)
    .await?;

    let mut unlinked = Vec::new();
    let mut required: BTreeMap<(Uuid, String), f64> = BTreeMap::new();
    for line in lines {
        if line.quantity <= EPSILON {
            continue;
        }
        match line.ingredient_id {
            Some(id) => *required.entry((id, line.unit)).or_insert(0.0) += line.quantity * factor,
            None => unlinked.push(line.name),
        }
    }

    let subtract = body.subtract_stock.unwrap_or(true);
    let mut ids = Vec::new();
    let mut covered = Vec::new();
    for ((ingredient, unit), need) in required {
        let on_hand: f64 = if subtract {
            sqlx::query_scalar(
                "SELECT COALESCE(sum(GREATEST(quantity, 0)), 0)::float8 FROM stock_lots \
                 WHERE ingredient_id = $1 AND unit = $2 AND deleted_at IS NULL",
            )
            .bind(ingredient)
            .bind(&unit)
            .fetch_one(&mut *tx)
            .await?
        } else {
            0.0
        };
        let to_buy = need - on_hand;
        let existing: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM shopping_list_items \
             WHERE ingredient_id = $1 AND unit = $2 AND source_recipe_version_id = $3 \
               AND NOT checked AND deleted_at IS NULL ORDER BY id LIMIT 1 FOR UPDATE",
        )
        .bind(ingredient)
        .bind(&unit)
        .bind(body.recipe_version_id)
        .fetch_optional(&mut *tx)
        .await?;
        if to_buy <= EPSILON {
            if let Some(id) = existing {
                sqlx::query("UPDATE shopping_list_items SET deleted_at = now(), updated_by = $2 WHERE id = $1")
                    .bind(id)
                    .bind(user)
                    .execute(&mut *tx)
                    .await?;
            }
            let name: String = sqlx::query_scalar("SELECT name FROM ingredients WHERE id = $1")
                .bind(ingredient)
                .fetch_one(&mut *tx)
                .await?;
            covered.push(CoveredLine {
                ingredient_id: ingredient,
                ingredient_name: name,
                required: need,
                on_hand,
                unit,
            });
            continue;
        }
        let id = if let Some(id) = existing {
            sqlx::query(
                "UPDATE shopping_list_items SET quantity = $2, updated_by = $3 WHERE id = $1",
            )
            .bind(id)
            .bind(to_buy)
            .bind(user)
            .execute(&mut *tx)
            .await?;
            id
        } else {
            let id = Uuid::now_v7();
            sqlx::query(
                "INSERT INTO shopping_list_items (id, workspace_id, ingredient_id, quantity, unit, \
                     source_recipe_version_id, created_by, updated_by) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $7)",
            )
            .bind(id)
            .bind(ws)
            .bind(ingredient)
            .bind(to_buy)
            .bind(&unit)
            .bind(body.recipe_version_id)
            .bind(user)
            .execute(&mut *tx)
            .await
            .map_err(constraint_error)?;
            id
        };
        ids.push(id);
    }
    let items: Vec<ShoppingItem> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{SHOPPING_SQL} AND s.id = ANY($1) ORDER BY i.name, s.unit"
    )))
    .bind(&ids)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    let result = ShoppingResult {
        items,
        covered,
        unlinked,
    };
    Ok((StatusCode::CREATED, Json(result)).into_response())
}

async fn fetch_item(tx: &mut PgConnection, id: Uuid, lock: bool) -> ApiResult<ShoppingItem> {
    let lock = if lock { " FOR UPDATE OF s" } else { "" };
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{SHOPPING_SQL} AND s.id = $1{lock}"
    )))
    .bind(id)
    .fetch_optional(tx)
    .await?
    .ok_or_else(not_found)
}

/// Ticks off an item or changes its amount.
#[utoipa::path(
    patch,
    path = "/v1/workspaces/{workspace_id}/shopping-list/{id}",
    tag = "stock",
    operation_id = "shopping_update",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Item"),
        ("If-Match" = String, Header, description = "Version `ETag` of the item")
    ),
    request_body(content = Value, content_type = "application/merge-patch+json",
        description = "`checked` (boolean) and `quantity` (number, zero or more)"),
    responses((status = 200, description = "The updated item", body = ShoppingItem,
        example = json!(shopping_example())))
)]
pub async fn shopping_update(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    MergePatch(patch): MergePatch<Value>,
) -> ApiResult<Response> {
    ctx.require(MOVE_PERMISSION)?;
    let Some(fields) = patch.as_object() else {
        return Err(issue("/", "invalid"));
    };
    if let Some(unknown) = fields
        .keys()
        .find(|k| !matches!(k.as_str(), "checked" | "quantity"))
    {
        return Err(issue(&format!("/{unknown}"), "unknown_field"));
    }
    let checked = match fields.get("checked") {
        None => None,
        Some(Value::Bool(b)) => Some(*b),
        Some(_) => return Err(issue("/checked", "invalid_value")),
    };
    let quantity = match fields.get("quantity") {
        None => None,
        Some(v) => Some(
            v.as_f64()
                .filter(|q| q.is_finite() && *q >= 0.0)
                .ok_or_else(|| issue("/quantity", "out_of_range"))?,
        ),
    };
    let mut tx = ctx.begin(&state.db).await?;
    let before = fetch_item(&mut tx, id, true).await?;
    check_if_match(Concurrency::Optimistic, &headers, before.version, || {
        json!(before)
    })?;
    sqlx::query(
        "UPDATE shopping_list_items SET checked = COALESCE($2, checked), \
             quantity = COALESCE($3, quantity), updated_by = $4 WHERE id = $1",
    )
    .bind(id)
    .bind(checked)
    .bind(quantity)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    let row = fetch_item(&mut tx, id, false).await?;
    tx.commit().await?;
    Ok(concurrency::record_response(
        StatusCode::OK,
        row.version,
        &row,
    ))
}

/// Removes an item from the list (soft delete).
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/shopping-list/{id}",
    tag = "stock",
    operation_id = "shopping_delete",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Item"),
        ("If-Match" = String, Header, description = "Version `ETag` of the item")
    ),
    responses((status = 204, description = "Deleted"))
)]
pub async fn shopping_delete(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    ctx.require(MOVE_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let before = fetch_item(&mut tx, id, true).await?;
    check_if_match(Concurrency::Optimistic, &headers, before.version, || {
        json!(before)
    })?;
    sqlx::query("UPDATE shopping_list_items SET deleted_at = now(), updated_by = $2 WHERE id = $1")
        .bind(id)
        .bind(ctx.identity.user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

// --- routes -------------------------------------------------------------------------------

/// The stock routes, wrapped with the sign-in and idempotency layers.
pub fn routes(state: &AppState) -> Router<AppState> {
    const WS: &str = "/v1/workspaces/{ws}";
    let creates = crate::idempotent(
        state,
        Router::new()
            .route(&format!("{WS}/stock/receipts"), post(stock_receive))
            .route(
                &format!("{WS}/stock/lots/{{id}}/movements"),
                post(stock_move),
            )
            .route(
                &format!("{WS}/batches/{{id}}/stock-deduction"),
                post(stock_deduct),
            )
            .route(
                &format!("{WS}/shopping-list/from-recipe"),
                post(shopping_from_recipe),
            ),
    );
    let others = Router::new()
        .route(&format!("{WS}/stock/lots"), get(stock_lot_list))
        .route(
            &format!("{WS}/stock/lots/{{id}}"),
            get(stock_lot_get).patch(stock_lot_update),
        )
        .route(&format!("{WS}/stock/movements"), get(stock_movement_list))
        .route(&format!("{WS}/stock/totals"), get(stock_totals))
        .route(&format!("{WS}/shopping-list"), get(shopping_list))
        .route(
            &format!("{WS}/shopping-list/{{id}}"),
            axum::routing::patch(shopping_update).delete(shopping_delete),
        );
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
        file_op("stock_lot_list", Some("recipe.view"), None, LIST),
        file_op("stock_lot_get", Some("recipe.view"), None, READ),
        file_op(
            "stock_lot_update",
            Some("stock.move"),
            Some(Concurrency::Optimistic),
            UPDATE,
        ),
        file_op(
            "stock_receive",
            Some("stock.move"),
            Some(Concurrency::Append),
            CREATE,
        ),
        file_op(
            "stock_move",
            Some("stock.move"),
            Some(Concurrency::Append),
            CREATE,
        ),
        file_op("stock_movement_list", Some("recipe.view"), None, LIST),
        file_op("stock_totals", Some("recipe.view"), None, READ),
        file_op(
            "stock_deduct",
            Some("stock.move"),
            Some(Concurrency::Append),
            CREATE,
        ),
        file_op("shopping_list", Some("recipe.view"), None, LIST),
        file_op(
            "shopping_from_recipe",
            Some("stock.move"),
            Some(Concurrency::Append),
            CREATE,
        ),
        file_op(
            "shopping_update",
            Some("stock.move"),
            Some(Concurrency::Optimistic),
            UPDATE,
        ),
        file_op(
            "shopping_delete",
            Some("stock.move"),
            Some(Concurrency::Optimistic),
            UPDATE,
        ),
    ];
}

/// `OpenAPI` fragment of this module.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(
        stock_lot_list,
        stock_lot_get,
        stock_lot_update,
        stock_receive,
        stock_move,
        stock_movement_list,
        stock_totals,
        stock_deduct,
        shopping_list,
        shopping_from_recipe,
        shopping_update,
        shopping_delete
    ),
    components(schemas(
        StockLot,
        StockMovement,
        StockWarning,
        MovementResult,
        StockTotal,
        Totals,
        ShoppingItem,
        ShoppingResult,
        CoveredLine,
        SkippedLine,
        DeductionResult,
        Receipt,
        MoveBody,
        LotPatch,
        FromRecipe
    )),
    tags((name = "stock", description = "Stock lots, the movement ledger, goods receipt, count, deduction for a batch and the shopping list"))
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "stock",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
