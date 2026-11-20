//! Ingredient library and style guides (T2.19, ADR-0007 section 3).
//!
//! Shared library rows (no workspace) and workspace rows live in the same tables. Every member
//! reads both. Only workspace rows can change, and only with `recipe.edit`. A shared row answers
//! `403 forbidden` to a write. A row of another workspace is hidden by row level security and
//! answers `404 not_found`.
//!
//! Ingredients have a common part and a part that depends on the kind (`detail`). The detail
//! of a fermentable, hop, yeast or misc is validated here and again by the table constraints.
//! Style guides, styles and the mappings between guides are read only in this task.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sqlx::{FromRow, PgConnection};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    concurrency::{self, Concurrency, check_if_match},
    context::RequestContext,
    error::{ApiError, ApiJson, ApiQuery, ApiResult, ErrorCode, FieldIssue, MergePatch},
    pagination::{Cursor, ListParams, Page, SortSpec},
};

/// Permission that creates, changes and deletes workspace ingredients.
pub const WRITE_PERMISSION: &str = "recipe.edit";

/// Builds SQL from the fixed fragments of this file; values are always bound.
macro_rules! dyn_sql {
    ($($t:tt)*) => {
        sqlx::AssertSqlSafe(format!($($t)*))
    };
}

const KINDS: &[&str] = &["fermentable", "hop", "yeast", "misc"];

// --- representations ----------------------------------------------------------------------

/// An ingredient of the shared library or of the workspace.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct Ingredient {
    /// Identifier.
    pub id: Uuid,
    /// `fermentable`, `hop`, `yeast` or `misc`.
    pub kind: String,
    /// Name.
    pub name: String,
    /// Manufacturer.
    pub manufacturer: Option<String>,
    /// Product code of the manufacturer.
    pub product_code: Option<String>,
    /// Origin (country or region).
    pub origin: Option<String>,
    /// Notes.
    pub notes: Option<String>,
    /// Price per kg, decimal text with up to 4 decimals.
    pub price: Option<String>,
    /// Currency of the price (ISO 4217).
    pub price_currency: Option<String>,
    /// `library` (shared, read only) or `user` (added by the workspace).
    pub source: String,
    /// Fields of the kind: see `FermentableDetail`, `HopDetail`, `YeastDetail`, `MiscDetail`.
    #[schema(value_type = Object)]
    pub detail: Value,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Updated at.
    pub updated_at: DateTime<Utc>,
    /// Deleted at (soft delete).
    pub deleted_at: Option<DateTime<Utc>>,
    /// Record version (`ETag`).
    pub version: i64,
}

/// Detail of a fermentable.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct FermentableDetail {
    /// `grain`, `extract`, `sugar`, `adjunct`, `dry_extract`, `fruit`, `juice`, `honey`, `acid`.
    pub grain_type: String,
    /// Yield, percent of dry basis.
    pub yield_pct: f64,
    /// Color, SRM.
    #[serde(default)]
    pub color_srm: f64,
    /// Moisture, percent.
    pub moisture_pct: Option<f64>,
    /// Diastatic power, Lintner.
    pub diastatic_power_lintner: Option<f64>,
    /// Protein, percent.
    pub protein_pct: Option<f64>,
    /// Largest share in a batch, percent (default 100).
    pub max_in_batch_pct: Option<f64>,
    /// The fermentable adds no fermentable extract.
    #[serde(default)]
    pub not_fermentable: bool,
    /// Bitterness an extract adds, IBU L per kg.
    #[serde(default)]
    pub ibu_l_per_kg: f64,
}

/// Detail of a hop.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct HopDetail {
    /// `pellet` (default), `plug`, `leaf`, `co2_extract`, `isomerized_extract`.
    pub form: Option<String>,
    /// Alpha acid, percent.
    pub alpha_pct: f64,
    /// Beta acid, percent.
    pub beta_pct: Option<f64>,
    /// Hop storage index, percent.
    pub hsi_pct: Option<f64>,
}

/// Detail of a yeast.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct YeastDetail {
    /// `liquid`, `dry`, `slant`, `culture`.
    pub form: String,
    /// Lowest apparent attenuation, percent.
    pub min_attenuation_pct: f64,
    /// Highest apparent attenuation, percent.
    pub max_attenuation_pct: f64,
    /// Alcohol tolerance, percent ABV.
    pub tolerance_abv_pct: Option<f64>,
    /// `low`, `medium`, `high`, `very_high`.
    pub flocculation: Option<String>,
    /// Lowest fermentation temperature, degrees C.
    pub min_temp_c: Option<f64>,
    /// Highest fermentation temperature, degrees C.
    pub max_temp_c: Option<f64>,
    /// Cells in one package, billion.
    pub total_cells_billion: Option<f64>,
    /// Cells in one ml, billion.
    pub cells_per_ml_billion: Option<f64>,
    /// Dry pitch rate, g per hl.
    pub dry_pitch_rate_g_per_hl: Option<f64>,
}

/// Detail of a misc ingredient.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct MiscDetail {
    /// `spice`, `fining`, `herb`, `flavor`, `water_agent`, `other` (default).
    pub misc_kind: Option<String>,
    /// `boil`, `mash`, `primary`, `secondary`, `bottling`, `sparge`, `whirlpool`.
    pub default_usage: Option<String>,
    /// Liquid (measured in L) instead of solid (kg).
    #[serde(default)]
    pub is_liquid: bool,
}

/// Body of `POST .../ingredients`.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct NewIngredient {
    /// `fermentable`, `hop`, `yeast` or `misc`.
    pub kind: String,
    /// Name (1 to 200 characters).
    pub name: String,
    /// Manufacturer (up to 200 characters).
    pub manufacturer: Option<String>,
    /// Product code (up to 100 characters).
    pub product_code: Option<String>,
    /// Origin (up to 100 characters).
    pub origin: Option<String>,
    /// Notes.
    pub notes: Option<String>,
    /// Price per kg, decimal text.
    pub price: Option<String>,
    /// Currency of the price; needed with `price`.
    pub price_currency: Option<String>,
    /// Fields of the kind.
    #[schema(value_type = Object)]
    #[serde(default)]
    pub detail: Value,
}

/// Body of `PATCH .../ingredients/{id}` (JSON Merge Patch). `kind` cannot change.
#[derive(Debug, Deserialize, ToSchema)]
pub struct IngredientPatch {
    /// Name.
    pub name: Option<String>,
    /// Manufacturer; `null` clears it.
    pub manufacturer: Option<String>,
    /// Product code; `null` clears it.
    pub product_code: Option<String>,
    /// Origin; `null` clears it.
    pub origin: Option<String>,
    /// Notes; `null` clears them.
    pub notes: Option<String>,
    /// Price per kg; `null` clears it (with the currency).
    pub price: Option<String>,
    /// Currency of the price.
    pub price_currency: Option<String>,
    /// Fields of the kind, merged one by one.
    #[schema(value_type = Object)]
    pub detail: Option<Value>,
}

/// The fields a client may set, common part and detail.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct IngredientInput {
    name: String,
    manufacturer: Option<String>,
    product_code: Option<String>,
    origin: Option<String>,
    notes: Option<String>,
    price: Option<String>,
    price_currency: Option<String>,
    #[serde(default)]
    detail: Value,
}

enum Detail {
    Fermentable(FermentableDetail),
    Hop(HopDetail),
    Yeast(YeastDetail),
    Misc(MiscDetail),
}

/// A style guide.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct StyleGuide {
    /// Identifier.
    pub id: Uuid,
    /// `library` or `user`.
    pub source: String,
    /// Name.
    pub name: String,
    /// Year of the edition.
    pub year: Option<i32>,
    /// License note of the data.
    pub license_note: Option<String>,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Updated at.
    pub updated_at: DateTime<Utc>,
    /// Deleted at (soft delete).
    pub deleted_at: Option<DateTime<Utc>>,
    /// Record version (`ETag`).
    pub version: i64,
}

/// A beer style of a guide. Ranges are `*_min` and `*_max`.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct Style {
    /// Identifier.
    pub id: Uuid,
    /// Guide of the style.
    pub guide_id: Uuid,
    /// `library` or `user`.
    pub source: String,
    /// Category name.
    pub category: Option<String>,
    /// Code in the guide.
    pub code: Option<String>,
    /// Name.
    pub name: String,
    /// Original gravity, lowest (SG).
    pub og_min_sg: Option<f64>,
    /// Original gravity, highest (SG).
    pub og_max_sg: Option<f64>,
    /// Final gravity, lowest (SG).
    pub fg_min_sg: Option<f64>,
    /// Final gravity, highest (SG).
    pub fg_max_sg: Option<f64>,
    /// Bitterness, lowest (IBU).
    pub ibu_min: Option<f64>,
    /// Bitterness, highest (IBU).
    pub ibu_max: Option<f64>,
    /// Color, lowest (SRM).
    pub srm_min: Option<f64>,
    /// Color, highest (SRM).
    pub srm_max: Option<f64>,
    /// Alcohol, lowest (percent ABV).
    pub abv_min_pct: Option<f64>,
    /// Alcohol, highest (percent ABV).
    pub abv_max_pct: Option<f64>,
    /// Carbonation, lowest (volumes of CO2).
    pub co2_min_vols: Option<f64>,
    /// Carbonation, highest (volumes of CO2).
    pub co2_max_vols: Option<f64>,
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

/// A mapping from a style of one guide to a style of another guide.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct StyleMapping {
    /// Identifier.
    pub id: Uuid,
    /// `library` or `user`.
    pub source: String,
    /// Style the mapping starts from.
    pub from_style_id: Uuid,
    /// Name of that style.
    pub from_style_name: String,
    /// Guide of that style.
    pub from_guide_id: Uuid,
    /// Style the mapping points to.
    pub to_style_id: Uuid,
    /// Name of that style.
    pub to_style_name: String,
    /// Guide of that style.
    pub to_guide_id: Uuid,
    /// `equivalent` or `closest`.
    pub relation: String,
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

/// Filters of `GET .../ingredients`.
#[derive(Debug, Default, Deserialize)]
pub struct IngredientFilter {
    /// Kind.
    pub kind: Option<String>,
    /// `library` or `user`.
    pub source: Option<String>,
    /// Text in the name (case blind).
    pub q: Option<String>,
}

/// Filters of `GET .../styles`.
#[derive(Debug, Default, Deserialize)]
pub struct StyleFilter {
    /// Guide.
    pub guide_id: Option<Uuid>,
    /// Text in the name or code (case blind).
    pub q: Option<String>,
}

/// Filters of `GET .../style-guides`.
#[derive(Debug, Default, Deserialize)]
pub struct GuideFilter {
    /// `library` or `user`.
    pub source: Option<String>,
}

/// Filters of `GET .../style-mappings`.
#[derive(Debug, Default, Deserialize)]
pub struct MappingFilter {
    /// Mappings that start or end at this style.
    pub style_id: Option<Uuid>,
    /// Mappings that start in this guide.
    pub from_guide_id: Option<Uuid>,
    /// Mappings that end in this guide.
    pub to_guide_id: Option<Uuid>,
}

// --- helpers ------------------------------------------------------------------------------

fn workspace(ctx: &RequestContext) -> ApiResult<Uuid> {
    ctx.workspace_id
        .ok_or_else(|| ApiError::new(ErrorCode::NotFound))
}

fn issue(field: &str, code: &str) -> ApiError {
    ApiError::validation(vec![FieldIssue::new(field, code)])
}

fn not_found() -> ApiError {
    ApiError::new(ErrorCode::NotFound)
}

fn read_only() -> ApiError {
    ApiError::new(ErrorCode::Forbidden).with_errors(vec![FieldIssue::new("/source", "read_only")])
}

fn sqlstate(e: &sqlx::Error) -> Option<String> {
    e.as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .map(|c| c.to_string())
}

/// RFC 7396: merges `patch` into `target`.
fn merge(target: &mut Value, patch: &Value) {
    if let Value::Object(p) = patch {
        if !target.is_object() {
            *target = Value::Object(Map::new());
        }
        if let Value::Object(t) = target {
            for (k, v) in p {
                if v.is_null() {
                    t.remove(k);
                } else {
                    merge(t.entry(k.clone()).or_insert(Value::Null), v);
                }
            }
        }
    } else {
        *target = patch.clone();
    }
}

fn range(issues: &mut Vec<FieldIssue>, field: &str, value: Option<f64>, lo: f64, hi: f64) {
    if let Some(v) = value
        && !(v.is_finite() && (lo..=hi).contains(&v))
    {
        issues.push(
            FieldIssue::new(format!("/detail/{field}"), "out_of_range")
                .with_params(json!({"min": lo, "max": hi})),
        );
    }
}

fn one_of(issues: &mut Vec<FieldIssue>, field: &str, value: Option<&str>, allowed: &[&str]) {
    if let Some(v) = value
        && !allowed.contains(&v)
    {
        issues.push(
            FieldIssue::new(format!("/detail/{field}"), "invalid_value")
                .with_params(json!({"allowed": allowed})),
        );
    }
}

fn text(issues: &mut Vec<FieldIssue>, field: &str, value: Option<&String>, max: usize) {
    if value.is_some_and(|v| v.chars().count() > max) {
        issues.push(FieldIssue::new(field, "too_long").with_params(json!({"max": max})));
    }
}

fn valid_price(price: &str) -> bool {
    let (int, frac) = price.split_once('.').unwrap_or((price, ""));
    !int.is_empty()
        && int.len() <= 15
        && frac.len() <= 4
        && int.bytes().all(|b| b.is_ascii_digit())
        && frac.bytes().all(|b| b.is_ascii_digit())
}

fn parse_detail(kind: &str, raw: &Value) -> Result<Detail, FieldIssue> {
    let raw = if raw.is_null() {
        json!({})
    } else {
        raw.clone()
    };
    let bad = |e: serde_json::Error| {
        FieldIssue::new("/detail", "invalid").with_params(json!({"reason": e.to_string()}))
    };
    match kind {
        "fermentable" => serde_json::from_value(raw).map(Detail::Fermentable),
        "hop" => serde_json::from_value(raw).map(Detail::Hop),
        "yeast" => serde_json::from_value(raw).map(Detail::Yeast),
        _ => serde_json::from_value(raw).map(Detail::Misc),
    }
    .map_err(bad)
}

/// Checks the common part and the detail; returns every problem at once.
#[allow(clippy::too_many_lines)] // one flat list of field checks
fn validate(kind: &str, input: &IngredientInput) -> ApiResult<Detail> {
    let mut issues = Vec::new();
    let name = input.name.trim();
    if name.is_empty() || name.chars().count() > 200 {
        issues.push(FieldIssue::new("/name", "invalid").with_params(json!({"min": 1, "max": 200})));
    }
    text(
        &mut issues,
        "/manufacturer",
        input.manufacturer.as_ref(),
        200,
    );
    text(
        &mut issues,
        "/product_code",
        input.product_code.as_ref(),
        100,
    );
    text(&mut issues, "/origin", input.origin.as_ref(), 100);
    match (&input.price, &input.price_currency) {
        (None, None) => {}
        (Some(p), Some(c)) => {
            if !valid_price(p) {
                issues.push(FieldIssue::new("/price", "invalid_value"));
            }
            if c.len() != 3 || !c.bytes().all(|b| b.is_ascii_uppercase()) {
                issues.push(FieldIssue::new("/price_currency", "invalid_value"));
            }
        }
        (Some(_), None) => issues.push(FieldIssue::new("/price_currency", "required")),
        (None, Some(_)) => issues.push(FieldIssue::new("/price", "required")),
    }
    let detail = match parse_detail(kind, &input.detail) {
        Ok(d) => Some(d),
        Err(i) => {
            issues.push(i);
            None
        }
    };
    if let Some(d) = &detail {
        match d {
            Detail::Fermentable(f) => {
                one_of(
                    &mut issues,
                    "grain_type",
                    Some(&f.grain_type),
                    &[
                        "grain",
                        "extract",
                        "sugar",
                        "adjunct",
                        "dry_extract",
                        "fruit",
                        "juice",
                        "honey",
                        "acid",
                    ],
                );
                range(&mut issues, "yield_pct", Some(f.yield_pct), 0.0, 100.0);
                range(&mut issues, "color_srm", Some(f.color_srm), 0.0, 1e4);
                range(&mut issues, "moisture_pct", f.moisture_pct, 0.0, 100.0);
                range(
                    &mut issues,
                    "diastatic_power_lintner",
                    f.diastatic_power_lintner,
                    0.0,
                    1e4,
                );
                range(&mut issues, "protein_pct", f.protein_pct, 0.0, 100.0);
                range(
                    &mut issues,
                    "max_in_batch_pct",
                    f.max_in_batch_pct,
                    0.0,
                    100.0,
                );
                range(&mut issues, "ibu_l_per_kg", Some(f.ibu_l_per_kg), 0.0, 1e6);
            }
            Detail::Hop(h) => {
                one_of(
                    &mut issues,
                    "form",
                    h.form.as_deref(),
                    &[
                        "pellet",
                        "plug",
                        "leaf",
                        "co2_extract",
                        "isomerized_extract",
                    ],
                );
                range(&mut issues, "alpha_pct", Some(h.alpha_pct), 0.0, 100.0);
                range(&mut issues, "beta_pct", h.beta_pct, 0.0, 100.0);
                range(&mut issues, "hsi_pct", h.hsi_pct, 0.0, 100.0);
            }
            Detail::Yeast(y) => {
                one_of(
                    &mut issues,
                    "form",
                    Some(&y.form),
                    &["liquid", "dry", "slant", "culture"],
                );
                one_of(
                    &mut issues,
                    "flocculation",
                    y.flocculation.as_deref(),
                    &["low", "medium", "high", "very_high"],
                );
                range(
                    &mut issues,
                    "min_attenuation_pct",
                    Some(y.min_attenuation_pct),
                    0.0,
                    100.0,
                );
                range(
                    &mut issues,
                    "max_attenuation_pct",
                    Some(y.max_attenuation_pct),
                    0.0,
                    100.0,
                );
                if y.min_attenuation_pct > y.max_attenuation_pct {
                    issues.push(FieldIssue::new("/detail/min_attenuation_pct", "above_max"));
                }
                range(
                    &mut issues,
                    "tolerance_abv_pct",
                    y.tolerance_abv_pct,
                    0.0,
                    100.0,
                );
                range(&mut issues, "min_temp_c", y.min_temp_c, -50.0, 200.0);
                range(&mut issues, "max_temp_c", y.max_temp_c, -50.0, 200.0);
                if let (Some(lo), Some(hi)) = (y.min_temp_c, y.max_temp_c)
                    && lo > hi
                {
                    issues.push(FieldIssue::new("/detail/min_temp_c", "above_max"));
                }
                range(
                    &mut issues,
                    "total_cells_billion",
                    y.total_cells_billion,
                    0.0,
                    1e9,
                );
                range(
                    &mut issues,
                    "cells_per_ml_billion",
                    y.cells_per_ml_billion,
                    0.0,
                    1e6,
                );
                range(
                    &mut issues,
                    "dry_pitch_rate_g_per_hl",
                    y.dry_pitch_rate_g_per_hl,
                    0.0,
                    1e6,
                );
            }
            Detail::Misc(m) => {
                one_of(
                    &mut issues,
                    "misc_kind",
                    m.misc_kind.as_deref(),
                    &["spice", "fining", "herb", "flavor", "water_agent", "other"],
                );
                one_of(
                    &mut issues,
                    "default_usage",
                    m.default_usage.as_deref(),
                    &[
                        "boil",
                        "mash",
                        "primary",
                        "secondary",
                        "bottling",
                        "sparge",
                        "whirlpool",
                    ],
                );
            }
        }
    }
    match (issues.is_empty(), detail) {
        (true, Some(d)) => Ok(d),
        _ => Err(ApiError::validation(issues)),
    }
}

const META_KEYS: &str = "ARRAY['id','workspace_id','created_at','updated_at','deleted_at',\
                         'version','created_by','updated_by']";

fn ingredient_columns() -> String {
    format!(
        "i.id, i.kind, i.name, i.manufacturer, i.product_code, i.origin, i.notes, \
         i.price::text AS price, i.price_currency::text AS price_currency, i.source, \
         COALESCE(CASE i.kind \
             WHEN 'fermentable' THEN (SELECT to_jsonb(d) - {META_KEYS} FROM ingredient_fermentables d WHERE d.id = i.id) \
             WHEN 'hop' THEN (SELECT to_jsonb(d) - {META_KEYS} FROM ingredient_hops d WHERE d.id = i.id) \
             WHEN 'yeast' THEN (SELECT to_jsonb(d) - {META_KEYS} FROM ingredient_yeasts d WHERE d.id = i.id) \
             ELSE (SELECT to_jsonb(d) - {META_KEYS} FROM ingredient_miscs d WHERE d.id = i.id) END, \
             '{{}}'::jsonb) AS detail, \
         i.created_at, i.updated_at, i.deleted_at, i.version"
    )
}

async fn fetch_ingredient(tx: &mut PgConnection, id: Uuid) -> ApiResult<Ingredient> {
    sqlx::query_as(dyn_sql!(
        "SELECT {} FROM ingredients i WHERE i.id = $1 AND i.deleted_at IS NULL",
        ingredient_columns()
    ))
    .bind(id)
    .fetch_optional(tx)
    .await?
    .ok_or_else(not_found)
}

fn ingredient_json(row: &Ingredient) -> Value {
    serde_json::to_value(row).unwrap_or(Value::Null)
}

/// The editable fields of a row as a merge target.
fn input_of(row: &Ingredient) -> Value {
    json!({
        "name": row.name, "manufacturer": row.manufacturer, "product_code": row.product_code,
        "origin": row.origin, "notes": row.notes, "price": row.price,
        "price_currency": row.price_currency, "detail": row.detail,
    })
}

/// Paging of a list: size, sort and the decoded cursor.
struct Paging {
    limit: u32,
    sort: SortSpec,
    key: Option<String>,
    id: Option<Uuid>,
}

fn paging(
    params: &ListParams,
    allowed: &[&'static str],
    default: &'static str,
) -> ApiResult<Paging> {
    let limit = params.limit()?;
    let sort = SortSpec::parse(params.sort.as_deref(), allowed, default)?;
    let (key, id) = match params.cursor(&sort)? {
        Some(c) => {
            let key = c.key.as_str().map(str::to_owned);
            if sort.field == "name" && key.is_none() {
                return Err(ApiError::new(ErrorCode::MalformedRequest)
                    .with_errors(vec![FieldIssue::new("cursor", "invalid_cursor")]));
            }
            (key, Some(c.id))
        }
        None => (None, None),
    };
    Ok(Paging {
        limit,
        sort,
        key,
        id,
    })
}

impl Paging {
    fn dir(&self) -> (&'static str, &'static str) {
        if self.sort.descending {
            ("<", "DESC")
        } else {
            (">", "ASC")
        }
    }
}

fn name_cursor(name: &str, id: Uuid) -> Cursor {
    Cursor {
        key: json!(name),
        id,
    }
}

fn id_cursor(id: Uuid) -> Cursor {
    Cursor {
        key: Value::Null,
        id,
    }
}

fn check_source(source: Option<&str>) -> ApiResult<()> {
    match source {
        None | Some("library" | "user") => Ok(()),
        Some(_) => Err(issue("source", "invalid_value")),
    }
}

fn created_response<T: Serialize>(location: &str, version: i64, body: &T) -> Response {
    let mut response = concurrency::record_response(StatusCode::CREATED, version, body);
    if let Ok(value) = location.parse() {
        response.headers_mut().insert(header::LOCATION, value);
    }
    response
}

// --- ingredient writes --------------------------------------------------------------------

async fn insert_detail(
    tx: &mut PgConnection,
    id: Uuid,
    ws: Uuid,
    user: Uuid,
    detail: &Detail,
) -> Result<(), sqlx::Error> {
    match detail {
        Detail::Fermentable(f) => {
            sqlx::query(
                "INSERT INTO ingredient_fermentables (id, workspace_id, grain_type, yield_pct, \
                     color_srm, moisture_pct, diastatic_power_lintner, protein_pct, \
                     max_in_batch_pct, not_fermentable, ibu_l_per_kg, created_by, updated_by) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, COALESCE($9, 100), $10, $11, $12, $12)",
            )
            .bind(id)
            .bind(ws)
            .bind(&f.grain_type)
            .bind(f.yield_pct)
            .bind(f.color_srm)
            .bind(f.moisture_pct)
            .bind(f.diastatic_power_lintner)
            .bind(f.protein_pct)
            .bind(f.max_in_batch_pct)
            .bind(f.not_fermentable)
            .bind(f.ibu_l_per_kg)
            .bind(user)
            .execute(tx)
            .await?;
        }
        Detail::Hop(h) => {
            sqlx::query(
                "INSERT INTO ingredient_hops (id, workspace_id, form, alpha_pct, beta_pct, hsi_pct, \
                     created_by, updated_by) \
                 VALUES ($1, $2, COALESCE($3, 'pellet'), $4, $5, $6, $7, $7)",
            )
            .bind(id)
            .bind(ws)
            .bind(h.form.as_deref())
            .bind(h.alpha_pct)
            .bind(h.beta_pct)
            .bind(h.hsi_pct)
            .bind(user)
            .execute(tx)
            .await?;
        }
        Detail::Yeast(y) => {
            sqlx::query(
                "INSERT INTO ingredient_yeasts (id, workspace_id, form, min_attenuation_pct, \
                     max_attenuation_pct, tolerance_abv_pct, flocculation, min_temp_c, max_temp_c, \
                     total_cells_billion, cells_per_ml_billion, dry_pitch_rate_g_per_hl, \
                     created_by, updated_by) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $13)",
            )
            .bind(id)
            .bind(ws)
            .bind(&y.form)
            .bind(y.min_attenuation_pct)
            .bind(y.max_attenuation_pct)
            .bind(y.tolerance_abv_pct)
            .bind(y.flocculation.as_deref())
            .bind(y.min_temp_c)
            .bind(y.max_temp_c)
            .bind(y.total_cells_billion)
            .bind(y.cells_per_ml_billion)
            .bind(y.dry_pitch_rate_g_per_hl)
            .bind(user)
            .execute(tx)
            .await?;
        }
        Detail::Misc(m) => {
            sqlx::query(
                "INSERT INTO ingredient_miscs (id, workspace_id, misc_kind, default_usage, \
                     is_liquid, created_by, updated_by) \
                 VALUES ($1, $2, COALESCE($3, 'other'), $4, $5, $6, $6)",
            )
            .bind(id)
            .bind(ws)
            .bind(m.misc_kind.as_deref())
            .bind(m.default_usage.as_deref())
            .bind(m.is_liquid)
            .bind(user)
            .execute(tx)
            .await?;
        }
    }
    Ok(())
}

async fn update_detail(
    tx: &mut PgConnection,
    id: Uuid,
    user: Uuid,
    detail: &Detail,
) -> Result<(), sqlx::Error> {
    match detail {
        Detail::Fermentable(f) => {
            sqlx::query(
                "UPDATE ingredient_fermentables SET grain_type = $2, yield_pct = $3, color_srm = $4, \
                     moisture_pct = $5, diastatic_power_lintner = $6, protein_pct = $7, \
                     max_in_batch_pct = COALESCE($8, 100), not_fermentable = $9, ibu_l_per_kg = $10, \
                     updated_by = $11 WHERE id = $1",
            )
            .bind(id)
            .bind(&f.grain_type)
            .bind(f.yield_pct)
            .bind(f.color_srm)
            .bind(f.moisture_pct)
            .bind(f.diastatic_power_lintner)
            .bind(f.protein_pct)
            .bind(f.max_in_batch_pct)
            .bind(f.not_fermentable)
            .bind(f.ibu_l_per_kg)
            .bind(user)
            .execute(tx)
            .await?;
        }
        Detail::Hop(h) => {
            sqlx::query(
                "UPDATE ingredient_hops SET form = COALESCE($2, 'pellet'), alpha_pct = $3, \
                     beta_pct = $4, hsi_pct = $5, updated_by = $6 WHERE id = $1",
            )
            .bind(id)
            .bind(h.form.as_deref())
            .bind(h.alpha_pct)
            .bind(h.beta_pct)
            .bind(h.hsi_pct)
            .bind(user)
            .execute(tx)
            .await?;
        }
        Detail::Yeast(y) => {
            sqlx::query(
                "UPDATE ingredient_yeasts SET form = $2, min_attenuation_pct = $3, \
                     max_attenuation_pct = $4, tolerance_abv_pct = $5, flocculation = $6, \
                     min_temp_c = $7, max_temp_c = $8, total_cells_billion = $9, \
                     cells_per_ml_billion = $10, dry_pitch_rate_g_per_hl = $11, updated_by = $12 \
                 WHERE id = $1",
            )
            .bind(id)
            .bind(&y.form)
            .bind(y.min_attenuation_pct)
            .bind(y.max_attenuation_pct)
            .bind(y.tolerance_abv_pct)
            .bind(y.flocculation.as_deref())
            .bind(y.min_temp_c)
            .bind(y.max_temp_c)
            .bind(y.total_cells_billion)
            .bind(y.cells_per_ml_billion)
            .bind(y.dry_pitch_rate_g_per_hl)
            .bind(user)
            .execute(tx)
            .await?;
        }
        Detail::Misc(m) => {
            sqlx::query(
                "UPDATE ingredient_miscs SET misc_kind = COALESCE($2, 'other'), default_usage = $3, \
                     is_liquid = $4, updated_by = $5 WHERE id = $1",
            )
            .bind(id)
            .bind(m.misc_kind.as_deref())
            .bind(m.default_usage.as_deref())
            .bind(m.is_liquid)
            .bind(user)
            .execute(tx)
            .await?;
        }
    }
    Ok(())
}

fn constraint_error(e: sqlx::Error) -> ApiError {
    match sqlstate(&e).as_deref() {
        Some("23514" | "22003" | "22P02") => issue("/", "constraint_violated"),
        _ => e.into(),
    }
}

// --- ingredient endpoints -----------------------------------------------------------------

/// Lists the ingredients of the shared library and of the workspace.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/ingredients",
    tag = "library",
    operation_id = "ingredient_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("kind" = Option<String>, Query, description = "`fermentable`, `hop`, `yeast` or `misc`"),
        ("source" = Option<String>, Query, description = "`library` or `user`"),
        ("q" = Option<String>, Query, description = "Text in the name (case blind)"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page"),
        ("sort" = Option<String>, Query, description = "`name` (default) or `-name`"),
        ("updated_since" = Option<String>, Query, description = "Changed since (RFC 3339), deleted ones included"),
        ("include_deleted" = Option<bool>, Query, description = "Include soft-deleted records")
    ),
    responses((
        status = 200,
        description = "A page of ingredients",
        body = Page<Ingredient>,
        example = json!({"items": [{
            "id": "0192f1c07a3b7000800000000000000b", "kind": "hop", "name": "Cascade",
            "manufacturer": null, "product_code": null, "origin": "US", "notes": null,
            "price": null, "price_currency": null, "source": "library",
            "detail": {"form": "pellet", "alpha_pct": 5.5, "beta_pct": 5.0, "hsi_pct": null},
            "created_at": "2026-11-20T10:00:00Z", "updated_at": "2026-11-20T10:00:00Z",
            "deleted_at": null, "version": 1
        }], "next_cursor": null})
    ))
)]
pub async fn ingredient_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<IngredientFilter>,
) -> ApiResult<Json<Page<Ingredient>>> {
    let paging = paging(&params, &["name"], "name")?;
    if filter.kind.as_deref().is_some_and(|k| !KINDS.contains(&k)) {
        return Err(issue("kind", "invalid_value"));
    }
    check_source(filter.source.as_deref())?;
    let (cmp, dir) = paging.dir();
    let sql = dyn_sql!(
        "SELECT {cols} FROM ingredients i \
         WHERE (i.deleted_at IS NULL OR $3 OR $4::timestamptz IS NOT NULL) \
           AND ($1::text IS NULL OR i.kind = $1) \
           AND ($2::text IS NULL OR i.source = $2) \
           AND ($8::text IS NULL OR strpos(lower(i.name), lower($8)) > 0) \
           AND ($4::timestamptz IS NULL OR i.updated_at >= $4) \
           AND ($5::text IS NULL OR (i.name, i.id) {cmp} ($5::text, $6::uuid)) \
         ORDER BY i.name {dir}, i.id {dir} LIMIT $7",
        cols = ingredient_columns()
    );
    let mut tx = ctx.begin(&state.db).await?;
    let rows: Vec<Ingredient> = sqlx::query_as(sql)
        .bind(&filter.kind)
        .bind(&filter.source)
        .bind(params.include_deleted.unwrap_or(false))
        .bind(params.updated_since)
        .bind(&paging.key)
        .bind(paging.id)
        .bind(i64::from(paging.limit) + 1)
        .bind(&filter.q)
        .fetch_all(&mut *tx)
        .await?;
    Ok(Json(Page::from_rows(
        rows,
        paging.limit,
        &paging.sort,
        |r| name_cursor(&r.name, r.id),
    )))
}

/// Reads one ingredient with its detail.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/ingredients/{id}",
    tag = "library",
    operation_id = "ingredient_get",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Ingredient")
    ),
    responses(
        (
            status = 200,
            description = "The ingredient",
            body = Ingredient,
            example = json!({
                "id": "0192f1c07a3b7000800000000000000b", "kind": "yeast", "name": "Sample Ale Yeast",
                "manufacturer": "Sample Lab", "product_code": null, "origin": null, "notes": null,
                "price": "4.5000", "price_currency": "USD", "source": "user",
                "detail": {"form": "dry", "min_attenuation_pct": 73.0, "max_attenuation_pct": 80.0,
                           "flocculation": "medium", "min_temp_c": 15.0, "max_temp_c": 24.0},
                "created_at": "2026-11-20T10:00:00Z", "updated_at": "2026-11-20T10:00:00Z",
                "deleted_at": null, "version": 1
            })
        ),
        (status = 304, description = "Not modified (`If-None-Match` matched)")
    )
)]
pub async fn ingredient_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let mut tx = ctx.begin(&state.db).await?;
    let row = fetch_ingredient(&mut tx, id).await?;
    Ok(concurrency::conditional_get(&headers, row.version, &row))
}

/// Adds an ingredient to the workspace.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/ingredients",
    tag = "library",
    operation_id = "ingredient_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = NewIngredient,
    responses((
        status = 201,
        description = "Ingredient created (`source` is `user`)",
        body = Ingredient,
        example = json!({
            "id": "0192f1c07a3b7000800000000000000b", "kind": "misc", "name": "Orange peel",
            "manufacturer": null, "product_code": null, "origin": null, "notes": null,
            "price": null, "price_currency": null, "source": "user",
            "detail": {"misc_kind": "spice", "default_usage": "boil", "is_liquid": false},
            "created_at": "2026-11-20T10:00:00Z", "updated_at": "2026-11-20T10:00:00Z",
            "deleted_at": null, "version": 1
        })
    ))
)]
pub async fn ingredient_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(new): ApiJson<NewIngredient>,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    if !KINDS.contains(&new.kind.as_str()) {
        return Err(issue("/kind", "invalid_value"));
    }
    let kind = new.kind;
    let input = IngredientInput {
        name: new.name,
        manufacturer: new.manufacturer,
        product_code: new.product_code,
        origin: new.origin,
        notes: new.notes,
        price: new.price,
        price_currency: new.price_currency,
        detail: new.detail,
    };
    let detail = validate(&kind, &input)?;
    let id = Uuid::now_v7();
    let user = ctx.identity.user_id;
    let mut tx = ctx.begin(&state.db).await?;
    sqlx::query(
        "INSERT INTO ingredients (id, workspace_id, kind, name, manufacturer, product_code, origin, \
             notes, price, price_currency, source, created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::text::numeric, $10::text, 'user', $11, $11)",
    )
    .bind(id)
    .bind(ws)
    .bind(&kind)
    .bind(input.name.trim())
    .bind(&input.manufacturer)
    .bind(&input.product_code)
    .bind(&input.origin)
    .bind(&input.notes)
    .bind(&input.price)
    .bind(&input.price_currency)
    .bind(user)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    insert_detail(&mut tx, id, ws, user, &detail)
        .await
        .map_err(constraint_error)?;
    let row = fetch_ingredient(&mut tx, id).await?;
    tx.commit().await?;
    let location = format!("/v1/workspaces/{ws}/ingredients/{id}");
    Ok(created_response(&location, row.version, &row))
}

/// Changes an ingredient of the workspace. A shared library ingredient answers `403`.
#[utoipa::path(
    patch,
    path = "/v1/workspaces/{workspace_id}/ingredients/{id}",
    tag = "library",
    operation_id = "ingredient_update",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Ingredient"),
        ("If-Match" = String, Header, description = "Version `ETag` of the ingredient")
    ),
    request_body(content = IngredientPatch, content_type = "application/merge-patch+json"),
    responses((
        status = 200,
        description = "The updated ingredient",
        body = Ingredient,
        example = json!({
            "id": "0192f1c07a3b7000800000000000000b", "kind": "hop", "name": "Cascade (own crop)",
            "manufacturer": null, "product_code": null, "origin": "US", "notes": null,
            "price": null, "price_currency": null, "source": "user",
            "detail": {"form": "pellet", "alpha_pct": 6.1, "beta_pct": null, "hsi_pct": null},
            "created_at": "2026-11-20T10:00:00Z", "updated_at": "2026-11-20T10:05:00Z",
            "deleted_at": null, "version": 2
        })
    ))
)]
pub async fn ingredient_update(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    MergePatch(patch): MergePatch<Value>,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    if !patch.is_object() {
        return Err(issue("/", "invalid"));
    }
    let user = ctx.identity.user_id;
    let mut tx = ctx.begin(&state.db).await?;
    let before = fetch_ingredient(&mut tx, id).await?;
    if before.source != "user" {
        return Err(read_only());
    }
    check_if_match(Concurrency::Optimistic, &headers, before.version, || {
        ingredient_json(&before)
    })?;
    let mut merged = input_of(&before);
    merge(&mut merged, &patch);
    let input: IngredientInput = serde_json::from_value(merged).map_err(|e| {
        ApiError::validation(vec![
            FieldIssue::new("/", "invalid").with_params(json!({"reason": e.to_string()})),
        ])
    })?;
    let detail = validate(&before.kind, &input)?;
    let done = sqlx::query(
        "UPDATE ingredients SET name = $2, manufacturer = $3, product_code = $4, origin = $5, \
             notes = $6, price = $7::text::numeric, price_currency = $8::text, updated_by = $9 \
         WHERE id = $1 AND version = $10 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(input.name.trim())
    .bind(&input.manufacturer)
    .bind(&input.product_code)
    .bind(&input.origin)
    .bind(&input.notes)
    .bind(&input.price)
    .bind(&input.price_currency)
    .bind(user)
    .bind(before.version)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    if done.rows_affected() == 0 {
        let current = fetch_ingredient(&mut tx, id).await?;
        return Err(ApiError::version_mismatch(ingredient_json(&current)));
    }
    update_detail(&mut tx, id, user, &detail)
        .await
        .map_err(constraint_error)?;
    let row = fetch_ingredient(&mut tx, id).await?;
    tx.commit().await?;
    Ok(concurrency::record_response(
        StatusCode::OK,
        row.version,
        &row,
    ))
}

/// Deletes an ingredient of the workspace (soft delete). A shared library ingredient answers `403`.
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/ingredients/{id}",
    tag = "library",
    operation_id = "ingredient_delete",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Ingredient"),
        ("If-Match" = String, Header, description = "Version `ETag` of the ingredient")
    ),
    responses((status = 204, description = "Deleted"))
)]
pub async fn ingredient_delete(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let before = fetch_ingredient(&mut tx, id).await?;
    if before.source != "user" {
        return Err(read_only());
    }
    check_if_match(Concurrency::Optimistic, &headers, before.version, || {
        ingredient_json(&before)
    })?;
    let done = sqlx::query(
        "UPDATE ingredients SET deleted_at = now(), updated_by = $2 \
         WHERE id = $1 AND version = $3 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(ctx.identity.user_id)
    .bind(before.version)
    .execute(&mut *tx)
    .await?;
    if done.rows_affected() == 0 {
        let current = fetch_ingredient(&mut tx, id).await?;
        return Err(ApiError::version_mismatch(ingredient_json(&current)));
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

// --- style guides -------------------------------------------------------------------------

const GUIDE_COLUMNS: &str = "g.id, CASE WHEN g.workspace_id IS NULL THEN 'library' ELSE 'user' END AS source, \
     g.name, g.year, g.license_note, g.created_at, g.updated_at, g.deleted_at, g.version";

const STYLE_COLUMNS: &str = "s.id, s.guide_id, CASE WHEN s.workspace_id IS NULL THEN 'library' ELSE 'user' END AS source, \
     s.category, s.code, s.name, s.og_min_sg, s.og_max_sg, s.fg_min_sg, s.fg_max_sg, \
     s.ibu_min, s.ibu_max, s.srm_min, s.srm_max, s.abv_min_pct, s.abv_max_pct, \
     s.co2_min_vols, s.co2_max_vols, s.notes, s.created_at, s.updated_at, s.deleted_at, s.version";

const MAPPING_COLUMNS: &str = "m.id, CASE WHEN m.workspace_id IS NULL THEN 'library' ELSE 'user' END AS source, \
     m.from_style_id, f.name AS from_style_name, f.guide_id AS from_guide_id, \
     m.to_style_id, t.name AS to_style_name, t.guide_id AS to_guide_id, \
     m.relation, m.notes, m.created_at, m.updated_at, m.deleted_at, m.version";

const MAPPING_FROM: &str = "style_mappings m \
     JOIN styles f ON f.id = m.from_style_id JOIN styles t ON t.id = m.to_style_id";

async fn fetch_guide(tx: &mut PgConnection, id: Uuid) -> ApiResult<StyleGuide> {
    sqlx::query_as(dyn_sql!(
        "SELECT {GUIDE_COLUMNS} FROM style_guides g WHERE g.id = $1 AND g.deleted_at IS NULL"
    ))
    .bind(id)
    .fetch_optional(tx)
    .await?
    .ok_or_else(not_found)
}

/// Lists the style guides.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/style-guides",
    tag = "library",
    operation_id = "style_guide_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("source" = Option<String>, Query, description = "`library` or `user`"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page"),
        ("sort" = Option<String>, Query, description = "`name` (default) or `-name`"),
        ("updated_since" = Option<String>, Query, description = "Changed since (RFC 3339), deleted ones included"),
        ("include_deleted" = Option<bool>, Query, description = "Include soft-deleted records")
    ),
    responses((
        status = 200,
        description = "A page of style guides",
        body = Page<StyleGuide>,
        example = json!({"items": [{
            "id": "0192f1c07a3b7000800000000000000b", "source": "library",
            "name": "Sample Guide A", "year": 2021, "license_note": "SAMPLE DATA",
            "created_at": "2026-11-20T10:00:00Z", "updated_at": "2026-11-20T10:00:00Z",
            "deleted_at": null, "version": 1
        }], "next_cursor": null})
    ))
)]
pub async fn style_guide_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<GuideFilter>,
) -> ApiResult<Json<Page<StyleGuide>>> {
    let paging = paging(&params, &["name"], "name")?;
    check_source(filter.source.as_deref())?;
    let (cmp, dir) = paging.dir();
    let sql = dyn_sql!(
        "SELECT {GUIDE_COLUMNS} FROM style_guides g \
         WHERE (g.deleted_at IS NULL OR $2 OR $3::timestamptz IS NOT NULL) \
           AND ($1::text IS NULL OR (CASE WHEN g.workspace_id IS NULL THEN 'library' ELSE 'user' END) = $1) \
           AND ($3::timestamptz IS NULL OR g.updated_at >= $3) \
           AND ($4::text IS NULL OR (g.name, g.id) {cmp} ($4::text, $5::uuid)) \
         ORDER BY g.name {dir}, g.id {dir} LIMIT $6"
    );
    let mut tx = ctx.begin(&state.db).await?;
    let rows: Vec<StyleGuide> = sqlx::query_as(sql)
        .bind(&filter.source)
        .bind(params.include_deleted.unwrap_or(false))
        .bind(params.updated_since)
        .bind(&paging.key)
        .bind(paging.id)
        .bind(i64::from(paging.limit) + 1)
        .fetch_all(&mut *tx)
        .await?;
    Ok(Json(Page::from_rows(
        rows,
        paging.limit,
        &paging.sort,
        |r| name_cursor(&r.name, r.id),
    )))
}

/// Reads one style guide.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/style-guides/{id}",
    tag = "library",
    operation_id = "style_guide_get",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Style guide")
    ),
    responses(
        (
            status = 200,
            description = "The style guide",
            body = StyleGuide,
            example = json!({
                "id": "0192f1c07a3b7000800000000000000b", "source": "library",
                "name": "Sample Guide A", "year": 2021, "license_note": "SAMPLE DATA",
                "created_at": "2026-11-20T10:00:00Z", "updated_at": "2026-11-20T10:00:00Z",
                "deleted_at": null, "version": 1
            })
        ),
        (status = 304, description = "Not modified (`If-None-Match` matched)")
    )
)]
pub async fn style_guide_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let mut tx = ctx.begin(&state.db).await?;
    let row = fetch_guide(&mut tx, id).await?;
    Ok(concurrency::conditional_get(&headers, row.version, &row))
}

async fn list_styles(
    state: &AppState,
    ctx: &RequestContext,
    params: &ListParams,
    guide: Option<Uuid>,
    q: Option<&str>,
    check_guide: bool,
) -> ApiResult<Json<Page<Style>>> {
    let paging = paging(params, &["name"], "name")?;
    let (cmp, dir) = paging.dir();
    let sql = dyn_sql!(
        "SELECT {STYLE_COLUMNS} FROM styles s \
         WHERE (s.deleted_at IS NULL OR $3 OR $4::timestamptz IS NOT NULL) \
           AND ($1::uuid IS NULL OR s.guide_id = $1) \
           AND ($2::text IS NULL OR strpos(lower(s.name || ' ' || coalesce(s.code, '')), lower($2)) > 0) \
           AND ($4::timestamptz IS NULL OR s.updated_at >= $4) \
           AND ($5::text IS NULL OR (s.name, s.id) {cmp} ($5::text, $6::uuid)) \
         ORDER BY s.name {dir}, s.id {dir} LIMIT $7"
    );
    let mut tx = ctx.begin(&state.db).await?;
    if let (true, Some(g)) = (check_guide, guide) {
        fetch_guide(&mut tx, g).await?;
    }
    let rows: Vec<Style> = sqlx::query_as(sql)
        .bind(guide)
        .bind(q)
        .bind(params.include_deleted.unwrap_or(false))
        .bind(params.updated_since)
        .bind(&paging.key)
        .bind(paging.id)
        .bind(i64::from(paging.limit) + 1)
        .fetch_all(&mut *tx)
        .await?;
    Ok(Json(Page::from_rows(
        rows,
        paging.limit,
        &paging.sort,
        |r| name_cursor(&r.name, r.id),
    )))
}

/// Lists the styles of one guide.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/style-guides/{id}/styles",
    tag = "library",
    operation_id = "style_guide_styles",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Style guide"),
        ("q" = Option<String>, Query, description = "Text in the name or code (case blind)"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page"),
        ("sort" = Option<String>, Query, description = "`name` (default) or `-name`"),
        ("updated_since" = Option<String>, Query, description = "Changed since (RFC 3339), deleted ones included"),
        ("include_deleted" = Option<bool>, Query, description = "Include soft-deleted records")
    ),
    responses((
        status = 200,
        description = "A page of styles of the guide",
        body = Page<Style>,
        example = json!({"items": [json!({
            "id": "0192f1c07a3b7000800000000000000c", "guide_id": "0192f1c07a3b7000800000000000000b",
            "source": "library", "category": "Pale Ale", "code": "A1", "name": "Sample Pale Ale",
            "og_min_sg": 1.045, "og_max_sg": 1.060, "fg_min_sg": 1.010, "fg_max_sg": 1.015,
            "ibu_min": 30.0, "ibu_max": 50.0, "srm_min": 5.0, "srm_max": 10.0,
            "abv_min_pct": 4.5, "abv_max_pct": 6.2, "co2_min_vols": 2.2, "co2_max_vols": 2.8,
            "notes": "SAMPLE DATA", "created_at": "2026-11-20T10:00:00Z",
            "updated_at": "2026-11-20T10:00:00Z", "deleted_at": null, "version": 1
        })], "next_cursor": null})
    ))
)]
pub async fn style_guide_styles(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<StyleFilter>,
) -> ApiResult<Json<Page<Style>>> {
    list_styles(&state, &ctx, &params, Some(id), filter.q.as_deref(), true).await
}

/// Lists styles of all guides.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/styles",
    tag = "library",
    operation_id = "style_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("guide_id" = Option<Uuid>, Query, description = "Only the styles of this guide"),
        ("q" = Option<String>, Query, description = "Text in the name or code (case blind)"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page"),
        ("sort" = Option<String>, Query, description = "`name` (default) or `-name`"),
        ("updated_since" = Option<String>, Query, description = "Changed since (RFC 3339), deleted ones included"),
        ("include_deleted" = Option<bool>, Query, description = "Include soft-deleted records")
    ),
    responses((
        status = 200,
        description = "A page of styles",
        body = Page<Style>,
        example = json!({"items": [json!({
            "id": "0192f1c07a3b7000800000000000000c", "guide_id": "0192f1c07a3b7000800000000000000b",
            "source": "library", "category": "Pale Ale", "code": "A1", "name": "Sample Pale Ale",
            "og_min_sg": 1.045, "og_max_sg": 1.060, "fg_min_sg": 1.010, "fg_max_sg": 1.015,
            "ibu_min": 30.0, "ibu_max": 50.0, "srm_min": 5.0, "srm_max": 10.0,
            "abv_min_pct": 4.5, "abv_max_pct": 6.2, "co2_min_vols": 2.2, "co2_max_vols": 2.8,
            "notes": "SAMPLE DATA", "created_at": "2026-11-20T10:00:00Z",
            "updated_at": "2026-11-20T10:00:00Z", "deleted_at": null, "version": 1
        })], "next_cursor": null})
    ))
)]
pub async fn style_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<StyleFilter>,
) -> ApiResult<Json<Page<Style>>> {
    list_styles(
        &state,
        &ctx,
        &params,
        filter.guide_id,
        filter.q.as_deref(),
        false,
    )
    .await
}

/// Reads one style.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/styles/{id}",
    tag = "library",
    operation_id = "style_get",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Style")
    ),
    responses(
        (status = 200, description = "The style", body = Style, example = json!({
            "id": "0192f1c07a3b7000800000000000000c", "guide_id": "0192f1c07a3b7000800000000000000b",
            "source": "library", "category": "Pale Ale", "code": "A1", "name": "Sample Pale Ale",
            "og_min_sg": 1.045, "og_max_sg": 1.060, "fg_min_sg": 1.010, "fg_max_sg": 1.015,
            "ibu_min": 30.0, "ibu_max": 50.0, "srm_min": 5.0, "srm_max": 10.0,
            "abv_min_pct": 4.5, "abv_max_pct": 6.2, "co2_min_vols": 2.2, "co2_max_vols": 2.8,
            "notes": "SAMPLE DATA", "created_at": "2026-11-20T10:00:00Z",
            "updated_at": "2026-11-20T10:00:00Z", "deleted_at": null, "version": 1
        })),
        (status = 304, description = "Not modified (`If-None-Match` matched)")
    )
)]
pub async fn style_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let mut tx = ctx.begin(&state.db).await?;
    let row: Style = sqlx::query_as(dyn_sql!(
        "SELECT {STYLE_COLUMNS} FROM styles s WHERE s.id = $1 AND s.deleted_at IS NULL"
    ))
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(not_found)?;
    Ok(concurrency::conditional_get(&headers, row.version, &row))
}

/// Lists the mappings between the styles of two guides.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/style-mappings",
    tag = "library",
    operation_id = "style_mapping_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("style_id" = Option<Uuid>, Query, description = "Mappings that start or end at this style"),
        ("from_guide_id" = Option<Uuid>, Query, description = "Mappings that start in this guide"),
        ("to_guide_id" = Option<Uuid>, Query, description = "Mappings that end in this guide"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page"),
        ("sort" = Option<String>, Query, description = "`id` (default) or `-id`"),
        ("updated_since" = Option<String>, Query, description = "Changed since (RFC 3339), deleted ones included"),
        ("include_deleted" = Option<bool>, Query, description = "Include soft-deleted records")
    ),
    responses((
        status = 200,
        description = "A page of style mappings",
        body = Page<StyleMapping>,
        example = json!({"items": [{
            "id": "0192f1c07a3b7000800000000000000d", "source": "library",
            "from_style_id": "0192f1c07a3b7000800000000000000c", "from_style_name": "Sample Pale Ale",
            "from_guide_id": "0192f1c07a3b7000800000000000000b",
            "to_style_id": "0192f1c07a3b7000800000000000000e", "to_style_name": "Sample American Ale",
            "to_guide_id": "0192f1c07a3b7000800000000000000f",
            "relation": "closest", "notes": null, "created_at": "2026-11-20T10:00:00Z",
            "updated_at": "2026-11-20T10:00:00Z", "deleted_at": null, "version": 1
        }], "next_cursor": null})
    ))
)]
pub async fn style_mapping_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<MappingFilter>,
) -> ApiResult<Json<Page<StyleMapping>>> {
    let paging = paging(&params, &["id"], "id")?;
    let (cmp, dir) = paging.dir();
    let sql = dyn_sql!(
        "SELECT {MAPPING_COLUMNS} FROM {MAPPING_FROM} \
         WHERE (m.deleted_at IS NULL OR $4 OR $5::timestamptz IS NOT NULL) \
           AND ($1::uuid IS NULL OR m.from_style_id = $1 OR m.to_style_id = $1) \
           AND ($2::uuid IS NULL OR f.guide_id = $2) \
           AND ($3::uuid IS NULL OR t.guide_id = $3) \
           AND ($5::timestamptz IS NULL OR m.updated_at >= $5) \
           AND ($6::uuid IS NULL OR m.id {cmp} $6) \
         ORDER BY m.id {dir} LIMIT $7"
    );
    let mut tx = ctx.begin(&state.db).await?;
    let rows: Vec<StyleMapping> = sqlx::query_as(sql)
        .bind(filter.style_id)
        .bind(filter.from_guide_id)
        .bind(filter.to_guide_id)
        .bind(params.include_deleted.unwrap_or(false))
        .bind(params.updated_since)
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

// --- routes -------------------------------------------------------------------------------

/// The library routes, wrapped with the sign-in and idempotency layers.
pub fn routes(state: &AppState) -> Router<AppState> {
    const WS: &str = "/v1/workspaces/{ws}";
    let creates = crate::idempotent(
        state,
        Router::new().route(
            &format!("{WS}/ingredients"),
            axum::routing::post(ingredient_create),
        ),
    );
    let others = Router::new()
        .route(&format!("{WS}/ingredients"), get(ingredient_list))
        .route(
            &format!("{WS}/ingredients/{{id}}"),
            get(ingredient_get)
                .patch(ingredient_update)
                .delete(ingredient_delete),
        )
        .route(&format!("{WS}/style-guides"), get(style_guide_list))
        .route(&format!("{WS}/style-guides/{{id}}"), get(style_guide_get))
        .route(
            &format!("{WS}/style-guides/{{id}}/styles"),
            get(style_guide_styles),
        )
        .route(&format!("{WS}/styles"), get(style_list))
        .route(&format!("{WS}/styles/{{id}}"), get(style_get))
        .route(&format!("{WS}/style-mappings"), get(style_mapping_list));
    crate::authenticated(state, creates.merge(others))
}

// --- registry entry ----------------------------------------------------------------------

/// Operation specs of this module (rows for `crate::openapi`).
mod spec {
    use crate::concurrency::Concurrency;
    use crate::error::ErrorCode;
    use crate::error::ErrorCode::{
        Forbidden, IdempotencyInProgress, IdempotencyMismatch, Internal, MalformedRequest,
        NotFound, PayloadTooLarge, PreconditionRequired, RateLimited, Unauthenticated, Unavailable,
        UnsupportedMediaType, UpgradeRequired, ValidationFailed, VersionMismatch,
    };
    use crate::openapi::{OperationSpec, file_op};

    const LIST: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
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
        IdempotencyInProgress,
        PayloadTooLarge,
        UnsupportedMediaType,
        ValidationFailed,
        IdempotencyMismatch,
        PreconditionRequired,
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

    const DELETE: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        VersionMismatch,
        PreconditionRequired,
        RateLimited,
        UpgradeRequired,
        Internal,
        Unavailable,
    ];

    pub(super) const OPERATIONS: &[OperationSpec] = &[
        file_op("ingredient_list", None, None, LIST),
        file_op("ingredient_get", None, None, READ),
        file_op(
            "ingredient_create",
            Some(super::WRITE_PERMISSION),
            Some(Concurrency::Append),
            CREATE,
        ),
        file_op(
            "ingredient_update",
            Some(super::WRITE_PERMISSION),
            Some(Concurrency::Optimistic),
            UPDATE,
        ),
        file_op(
            "ingredient_delete",
            Some(super::WRITE_PERMISSION),
            Some(Concurrency::Optimistic),
            DELETE,
        ),
        file_op("style_guide_list", None, None, LIST),
        file_op("style_guide_get", None, None, READ),
        file_op("style_guide_styles", None, None, LIST),
        file_op("style_list", None, None, LIST),
        file_op("style_get", None, None, READ),
        file_op("style_mapping_list", None, None, LIST),
    ];
}

/// `OpenAPI` fragment of this module; `crate::openapi` merges it.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(
        ingredient_list,
        ingredient_get,
        ingredient_create,
        ingredient_update,
        ingredient_delete,
        style_guide_list,
        style_guide_get,
        style_guide_styles,
        style_list,
        style_get,
        style_mapping_list
    ),
    components(schemas(
        Ingredient,
        NewIngredient,
        IngredientPatch,
        FermentableDetail,
        HopDetail,
        YeastDetail,
        MiscDetail,
        StyleGuide,
        Style,
        StyleMapping
    )),
    tags(
        (name = "library", description = "Ingredient library, style guides, styles and mappings between guides")
    )
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "library",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_follows_rfc_7396() {
        let mut target = json!({"a": 1, "detail": {"x": 1, "y": 2}});
        merge(&mut target, &json!({"a": null, "detail": {"y": 3, "z": 4}}));
        assert_eq!(target, json!({"detail": {"x": 1, "y": 3, "z": 4}}));
    }

    #[test]
    fn price_format() {
        assert!(valid_price("12"));
        assert!(valid_price("12.5000"));
        assert!(!valid_price("12.50001"));
        assert!(!valid_price("-1"));
        assert!(!valid_price(".5"));
        assert!(!valid_price("1e3"));
    }
}
