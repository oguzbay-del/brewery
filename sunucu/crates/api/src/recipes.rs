//! Recipes and recipe versions (T2.21, ADR-0007 section 3 "Tarif").
//!
//! A recipe holds the identity. The content lives in versions with a number. A draft changes in
//! place (`If-Match` required). A publish makes the version immutable (status `approved`, enforced
//! by database triggers), runs the calculation and stores the summary in `calc_summary`. A
//! change after the publish starts a new draft that copies the published version.
//!
//! The lines of a version (fermentables, hops, yeasts, miscs, water additions) copy the
//! properties of the library ingredient when the line is written (the snapshot of an
//! `IngredientUse`). The recipe does not change when the library changes. The equipment is copied
//! into `equipment_snapshot` in the same way, and again at the publish.
//!
//! The API has no formulas. Gravities, alcohol, bitterness and colour come from the engine.
//! TODO: `est_cells_ml` and `honey_pct` are simple ratios computed here. Move them into the engine
//! (ADR-0006 gaps) when the engine has the per-fermentable points.

#![allow(clippy::too_many_lines)]

use std::fmt::Write as _;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::{DateTime, Utc};
use db::{AuditEvent, NewAuditEntry};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sqlx::{FromRow, PgConnection};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    concurrency::{self, Concurrency, check_if_match},
    context::RequestContext,
    engine::{CalcScope, CalcTarget, proto},
    error::{ApiError, ApiJson, ApiQuery, ApiResult, ErrorCode, FieldIssue, MergePatch},
    library::{created_response, issue, merge, name_cursor, not_found, paging, workspace},
    pagination::{ListParams, Page},
    recipe_org::{conflict, optional_body, recipe_event},
};

/// Permission that reads recipes.
pub const READ_PERMISSION: &str = "recipe.view";
/// Permission that changes recipes and drafts (and publishes in Home mode).
pub const WRITE_PERMISSION: &str = "recipe.edit";
/// Permission that publishes a version in Business mode.
pub const APPROVE_PERMISSION: &str = "recipe.approve";

const KINDS: &[&str] = &[
    "extract",
    "partial_mash",
    "all_grain",
    "cider",
    "mead",
    "wine",
    "seltzer",
];

/// Columns that a line row sends to the client are all except these.
const LINE_STRIP: &str = "ARRAY['workspace_id','recipe_version_id','created_at','updated_at','deleted_at','created_by','updated_by','version']";

/// Line sections of a version: (JSON key, table, columns without the common ones).
const SECTIONS: &[(&str, &str, &str)] = &[
    (
        "fermentables",
        "recipe_fermentables",
        "ingredient_id, name, amount_kg, usage, grain_type, yield_pct, color_srm, \
         not_fermentable, late_extract_min, max_in_batch_pct, ibu_l_per_kg",
    ),
    (
        "hops",
        "recipe_hops",
        "ingredient_id, name, amount_kg, alpha_pct, form, usage, time_min, whirlpool_temp_c",
    ),
    (
        "yeasts",
        "recipe_yeasts",
        "ingredient_id, name, form, min_attenuation_pct, max_attenuation_pct, tolerance_abv_pct, \
         add_to_secondary, total_cells_billion, viable_cells_billion, packs, viability_pct, \
         use_packages, amount_kg, cells_per_ml_billion, dry_pitch_rate_g_per_hl",
    ),
    (
        "miscs",
        "recipe_miscs",
        "ingredient_id, name, amount, is_liquid, usage, time_min",
    ),
    (
        "water_additions",
        "recipe_water_additions",
        "agent, stage, amount_kg, concentration_pct",
    ),
];

const HEADER_KEYS: &[&str] = &[
    "kind",
    "style_id",
    "equipment_id",
    "batch_volume_l",
    "boil_volume_l",
    "boil_time_min",
    "efficiency_pct",
    "target_og_sg",
    "target_fg_sg",
    "target_ibu",
    "target_srm",
    "target_abv_pct",
    "target_co2_vols",
    "change_note",
    "mash_snapshot",
    "fermentation_snapshot",
    "water_snapshot",
    "options",
];

// --- representations ----------------------------------------------------------------------

/// A recipe (the identity; the content is in the versions).
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct Recipe {
    /// Identifier.
    pub id: Uuid,
    /// Name.
    pub name: String,
    /// Folder.
    pub folder_id: Option<Uuid>,
    /// The owner closed the recipe to changes (Home mode).
    pub is_locked: bool,
    /// Notes.
    pub notes: Option<String>,
    /// The last published version.
    pub approved_version_id: Option<Uuid>,
    /// The open draft.
    pub draft_version_id: Option<Uuid>,
    /// Kind of the published (or else the draft) version.
    pub kind: Option<String>,
    /// Tags of the recipe.
    pub tag_ids: Vec<Uuid>,
    /// The caller marked the recipe as favorite.
    pub is_favorite: bool,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Updated at.
    pub updated_at: DateTime<Utc>,
    /// Deleted at (soft delete).
    pub deleted_at: Option<DateTime<Utc>>,
    /// Record version (`ETag`).
    pub version: i64,
}

/// One ingredient line of a version. All properties are copies (the snapshot).
#[derive(Debug, Default, Clone, Deserialize, ToSchema)]
pub struct FermentableLine {
    /// Keep this line identifier when the lines are replaced.
    pub id: Option<Uuid>,
    /// Library ingredient. Missing properties are copied from it.
    pub ingredient_id: Option<Uuid>,
    /// Name.
    pub name: Option<String>,
    /// Amount, kg.
    pub amount_kg: f64,
    /// `mash` (default), `steep`, `boil`, `whirlpool`, `primary`, `secondary`, `tertiary`, `bottling`, `sparge`.
    pub usage: Option<String>,
    /// `grain`, `extract`, `sugar`, `adjunct`, `dry_extract`, `fruit`, `juice`, `honey`, `acid`.
    pub grain_type: Option<String>,
    /// Yield, percent.
    pub yield_pct: Option<f64>,
    /// Color, SRM.
    pub color_srm: Option<f64>,
    /// The fermentable adds no fermentable extract.
    pub not_fermentable: Option<bool>,
    /// Late extract, minutes before the end of the boil.
    pub late_extract_min: Option<f64>,
    /// Largest share in a batch, percent.
    pub max_in_batch_pct: Option<f64>,
    /// Bitterness an extract adds, IBU L per kg.
    pub ibu_l_per_kg: Option<f64>,
}

/// A hop line.
#[derive(Debug, Default, Clone, Deserialize, ToSchema)]
pub struct HopLine {
    /// Keep this line identifier when the lines are replaced.
    pub id: Option<Uuid>,
    /// Library ingredient.
    pub ingredient_id: Option<Uuid>,
    /// Name.
    pub name: Option<String>,
    /// Amount, kg (liter for extracts).
    pub amount_kg: f64,
    /// Alpha acid, percent.
    pub alpha_pct: Option<f64>,
    /// `pellet` (default), `plug`, `leaf`, `co2_extract`, `isomerized_extract`.
    pub form: Option<String>,
    /// `boil` (default), `dry_hop`, `mash`, `first_wort`, `whirlpool`.
    pub usage: Option<String>,
    /// Time, minutes.
    pub time_min: Option<f64>,
    /// Whirlpool temperature, degrees C.
    pub whirlpool_temp_c: Option<f64>,
}

/// A yeast line.
#[derive(Debug, Default, Clone, Deserialize, ToSchema)]
pub struct YeastLine {
    /// Keep this line identifier when the lines are replaced.
    pub id: Option<Uuid>,
    /// Library ingredient.
    pub ingredient_id: Option<Uuid>,
    /// Name.
    pub name: Option<String>,
    /// `liquid`, `dry`, `slant`, `culture`.
    pub form: Option<String>,
    /// Lowest apparent attenuation, percent.
    pub min_attenuation_pct: Option<f64>,
    /// Highest apparent attenuation, percent.
    pub max_attenuation_pct: Option<f64>,
    /// Alcohol tolerance, percent ABV.
    pub tolerance_abv_pct: Option<f64>,
    /// Added in secondary.
    pub add_to_secondary: Option<bool>,
    /// Cells in one pack, billion.
    pub total_cells_billion: Option<f64>,
    /// Viable cells in one pack, billion.
    pub viable_cells_billion: Option<f64>,
    /// Packs (default 1).
    pub packs: Option<f64>,
    /// Viability, percent.
    pub viability_pct: Option<f64>,
    /// Use the pack counts (default true).
    pub use_packages: Option<bool>,
    /// Amount, kg (dry yeast).
    pub amount_kg: Option<f64>,
    /// Cells per mL, billion.
    pub cells_per_ml_billion: Option<f64>,
    /// Dry pitch rate, g per hL.
    pub dry_pitch_rate_g_per_hl: Option<f64>,
}

/// A misc line.
#[derive(Debug, Default, Clone, Deserialize, ToSchema)]
pub struct MiscLine {
    /// Keep this line identifier when the lines are replaced.
    pub id: Option<Uuid>,
    /// Library ingredient.
    pub ingredient_id: Option<Uuid>,
    /// Name.
    pub name: Option<String>,
    /// Amount (L for a liquid, else kg).
    pub amount: f64,
    /// The misc is a liquid.
    pub is_liquid: Option<bool>,
    /// `boil` (default), `mash`, `primary`, `secondary`, `bottling`, `sparge`, `whirlpool`.
    pub usage: Option<String>,
    /// Time, minutes.
    pub time_min: Option<f64>,
}

/// A water addition line.
#[derive(Debug, Default, Clone, Deserialize, ToSchema)]
pub struct WaterAdditionLine {
    /// Keep this line identifier when the lines are replaced.
    pub id: Option<Uuid>,
    /// Salt or acid (`gypsum`, `calcium_chloride`, `epsom_salt`, `baking_soda`, `chalk`, ...).
    pub agent: String,
    /// `mash` (default), `sparge`, `boil`.
    pub stage: Option<String>,
    /// Amount, kg.
    pub amount_kg: f64,
    /// Concentration of an acid solution, percent.
    pub concentration_pct: Option<f64>,
}

/// The editable content of a version (create and the merge target of a patch).
#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct VersionInput {
    /// `extract`, `partial_mash`, `all_grain` (default), `cider`, `mead`, `wine`, `seltzer`.
    pub kind: Option<String>,
    /// Style.
    pub style_id: Option<Uuid>,
    /// Equipment. Missing volumes and efficiency come from it.
    pub equipment_id: Option<Uuid>,
    /// Batch volume into the fermenter, L.
    pub batch_volume_l: Option<f64>,
    /// Pre-boil volume, L.
    pub boil_volume_l: Option<f64>,
    /// Boil time, minutes.
    pub boil_time_min: Option<f64>,
    /// Brewhouse efficiency, percent.
    pub efficiency_pct: Option<f64>,
    /// Target OG, SG.
    pub target_og_sg: Option<f64>,
    /// Target FG, SG.
    pub target_fg_sg: Option<f64>,
    /// Target bitterness, IBU.
    pub target_ibu: Option<f64>,
    /// Target color, SRM.
    pub target_srm: Option<f64>,
    /// Target alcohol, percent.
    pub target_abv_pct: Option<f64>,
    /// Target CO2, volumes.
    pub target_co2_vols: Option<f64>,
    /// Note on the change.
    pub change_note: Option<String>,
    /// Mash profile snapshot: `{biab, steps: [{kind, temp_c, infusion_l, time_min}]}`.
    #[schema(value_type = Option<Object>)]
    pub mash_snapshot: Option<Value>,
    /// Fermentation profile snapshot: `{steps: [{name, days, temp_c}]}`.
    #[schema(value_type = Option<Object>)]
    pub fermentation_snapshot: Option<Value>,
    /// Water profile snapshot.
    #[schema(value_type = Option<Object>)]
    pub water_snapshot: Option<Value>,
    /// Calculation options (engine `RecipeOptions` keys and `ibu_equation`).
    #[schema(value_type = Option<Object>)]
    pub options: Option<Value>,
    /// Fermentable lines (replaces all).
    pub fermentables: Option<Vec<FermentableLine>>,
    /// Hop lines (replaces all).
    pub hops: Option<Vec<HopLine>>,
    /// Yeast lines (replaces all).
    pub yeasts: Option<Vec<YeastLine>>,
    /// Misc lines (replaces all).
    pub miscs: Option<Vec<MiscLine>>,
    /// Water addition lines (replaces all).
    pub water_additions: Option<Vec<WaterAdditionLine>>,
}

/// Body of a recipe create.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct NewRecipe {
    /// Name, 1 to 200 characters.
    pub name: String,
    /// Folder.
    pub folder_id: Option<Uuid>,
    /// Notes.
    pub notes: Option<String>,
    /// Tags.
    #[serde(default)]
    pub tag_ids: Vec<Uuid>,
    /// Content of the first draft.
    #[serde(default)]
    pub version: VersionInput,
}

/// Merge patch of a recipe.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RecipePatch {
    /// Name.
    pub name: Option<String>,
    /// Folder (`null` removes it).
    pub folder_id: Option<Uuid>,
    /// Notes.
    pub notes: Option<String>,
    /// Lock the recipe against changes.
    pub is_locked: Option<bool>,
}

/// Body of a new draft.
#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct NewDraft {
    /// Version to copy (default: the last published, else the newest).
    pub from_version_id: Option<Uuid>,
    /// Note on the change.
    pub change_note: Option<String>,
}

/// Body of a publish.
#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PublishBody {
    /// Note on the change.
    pub change_note: Option<String>,
}

/// Body of a scaling preview.
#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ScaleBody {
    /// Multiply all amounts by this factor.
    pub factor: Option<f64>,
    /// Or: the new batch volume, L (the factor is the ratio to the current volume).
    pub batch_volume_l: Option<f64>,
    /// Calculate the scaled recipe with the engine (default true).
    pub calculate: Option<bool>,
}

/// Filter of the recipe list.
#[derive(Debug, Default, Deserialize)]
pub struct RecipeFilter {
    /// Text in the name (case blind).
    pub q: Option<String>,
    /// Folder.
    pub folder_id: Option<Uuid>,
    /// Tag.
    pub tag_id: Option<Uuid>,
    /// Only favorites of the caller.
    pub favorite: Option<bool>,
    /// Kind of the version.
    pub kind: Option<String>,
}

// --- helpers ------------------------------------------------------------------------------

fn constraint_error(e: sqlx::Error) -> ApiError {
    match crate::library::sqlstate(&e).as_deref() {
        Some("23514" | "22003" | "22P02") => issue("/", "constraint_violated"),
        Some("23503") => issue("/", "reference_not_found"),
        Some("23505") => conflict("/", "draft_exists"),
        Some("23000") => conflict("/", "version_immutable"),
        _ => e.into(),
    }
}

fn recipe_cols(user: &str) -> String {
    format!(
        "r.id, r.name, r.folder_id, r.is_locked, r.notes, r.approved_version_id, \
         r.draft_version_id, \
         (SELECT v.kind FROM recipe_versions v \
            WHERE v.id = COALESCE(r.approved_version_id, r.draft_version_id)) AS kind, \
         COALESCE((SELECT array_agg(t.tag_id ORDER BY t.tag_id) FROM recipe_tags t \
            WHERE t.recipe_id = r.id), '{{}}'::uuid[]) AS tag_ids, \
         EXISTS (SELECT 1 FROM favorites f WHERE f.recipe_id = r.id AND f.user_id = {user}) AS is_favorite, \
         r.created_at, r.updated_at, r.deleted_at, r.version"
    )
}

async fn fetch_recipe(tx: &mut PgConnection, user: Uuid, id: Uuid) -> ApiResult<Recipe> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {} FROM recipes r WHERE r.id = $2 AND r.deleted_at IS NULL",
        recipe_cols("$1")
    )))
    .bind(user)
    .bind(id)
    .fetch_optional(tx)
    .await?
    .ok_or_else(not_found)
}

fn version_sql() -> String {
    let mut parts = String::new();
    for (key, table, _) in SECTIONS {
        let _ = write!(
            parts,
            "'{key}', (SELECT COALESCE(jsonb_agg(to_jsonb(l) - {LINE_STRIP} ORDER BY l.position), \
             '[]'::jsonb) FROM {table} l WHERE l.recipe_version_id = v.id), "
        );
    }
    parts.truncate(parts.len() - 2);
    format!(
        "SELECT (to_jsonb(v) - ARRAY['workspace_id','created_by','updated_by','deleted_at']) \
         || jsonb_build_object({parts}) FROM recipe_versions v \
         WHERE v.id = $1 AND v.deleted_at IS NULL"
    )
}

/// The full version as JSON (header, snapshots, lines, `calc_summary`).
pub(crate) async fn version_json(tx: &mut PgConnection, id: Uuid) -> ApiResult<Value> {
    sqlx::query_scalar(sqlx::AssertSqlSafe(version_sql()))
        .bind(id)
        .fetch_optional(tx)
        .await?
        .ok_or_else(not_found)
}

fn num(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn opt_num(v: &Value, key: &str) -> Option<f64> {
    v.get(key).and_then(Value::as_f64)
}

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or_default()
}

fn flag(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn lines<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn with_schema(field: &str, value: Option<Value>) -> ApiResult<Option<Value>> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Object(mut m)) => {
            m.insert("schema_version".to_owned(), json!(1));
            Ok(Some(Value::Object(m)))
        }
        Some(_) => Err(issue(&format!("/{field}"), "invalid_value")),
    }
}

/// Copy of an equipment row for the version (the snapshot).
async fn equipment_snapshot(tx: &mut PgConnection, id: Uuid) -> ApiResult<Value> {
    let row: Option<Value> = sqlx::query_scalar(
        "SELECT to_jsonb(e) - ARRAY['workspace_id','created_at','updated_at','deleted_at', \
            'created_by','updated_by','version'] FROM equipment e \
         WHERE e.id = $1 AND e.deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(tx)
    .await?;
    let mut row = row.ok_or_else(|| issue("/equipment_id", "not_found"))?;
    if let Some(m) = row.as_object_mut() {
        m.insert("schema_version".to_owned(), json!(1));
        m.insert("equipment_id".to_owned(), json!(id));
    }
    Ok(row)
}

/// Header of a version with the defaults filled.
struct Header {
    kind: String,
    style_id: Option<Uuid>,
    equipment_id: Option<Uuid>,
    batch_volume_l: f64,
    boil_volume_l: Option<f64>,
    boil_time_min: f64,
    efficiency_pct: f64,
    targets: [Option<f64>; 6],
    change_note: Option<String>,
    equipment_snapshot: Value,
    mash: Option<Value>,
    fermentation: Option<Value>,
    water: Option<Value>,
    options: Value,
}

async fn resolve_header(
    tx: &mut PgConnection,
    input: &VersionInput,
    previous: Option<(Option<Uuid>, Value)>,
) -> ApiResult<Header> {
    let kind = input.kind.clone().unwrap_or_else(|| "all_grain".to_owned());
    if !KINDS.contains(&kind.as_str()) {
        return Err(issue("/kind", "invalid_value"));
    }
    let equipment_snapshot = match (input.equipment_id, previous) {
        (Some(id), Some((Some(prev), snap))) if prev == id => snap,
        (Some(id), _) => equipment_snapshot(tx, id).await?,
        (None, Some((_, snap)))
            if input.equipment_id.is_none() && snap.get("equipment_id").is_none() =>
        {
            snap
        }
        (None, _) => json!({"schema_version": 1}),
    };
    let from_eq = |key: &str| opt_num(&equipment_snapshot, key);
    let batch_volume_l = input
        .batch_volume_l
        .or_else(|| from_eq("batch_volume_l"))
        .ok_or_else(|| issue("/batch_volume_l", "required"))?;
    let efficiency_pct = input
        .efficiency_pct
        .or_else(|| from_eq("efficiency_pct"))
        .ok_or_else(|| issue("/efficiency_pct", "required"))?;
    Ok(Header {
        kind,
        style_id: input.style_id,
        equipment_id: input.equipment_id,
        batch_volume_l,
        boil_volume_l: input.boil_volume_l.or_else(|| from_eq("boil_volume_l")),
        boil_time_min: input
            .boil_time_min
            .or_else(|| from_eq("boil_time_min"))
            .unwrap_or(60.0),
        efficiency_pct,
        targets: [
            input.target_og_sg,
            input.target_fg_sg,
            input.target_ibu,
            input.target_srm,
            input.target_abv_pct,
            input.target_co2_vols,
        ],
        change_note: input.change_note.clone(),
        equipment_snapshot,
        mash: with_schema("mash_snapshot", input.mash_snapshot.clone())?,
        fermentation: with_schema("fermentation_snapshot", input.fermentation_snapshot.clone())?,
        water: with_schema("water_snapshot", input.water_snapshot.clone())?,
        options: with_schema("options", input.options.clone())?
            .unwrap_or(json!({"schema_version": 1})),
    })
}

/// A library ingredient with its detail (`to_jsonb` of both rows).
async fn library_row(
    tx: &mut PgConnection,
    kind: &str,
    section: &str,
    idx: usize,
    id: Uuid,
) -> ApiResult<Value> {
    let table = match kind {
        "fermentable" => "ingredient_fermentables",
        "hop" => "ingredient_hops",
        "yeast" => "ingredient_yeasts",
        _ => "ingredient_miscs",
    };
    sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "SELECT to_jsonb(i) || to_jsonb(d) FROM ingredients i JOIN {table} d ON d.id = i.id \
         WHERE i.id = $1 AND i.kind = $2 AND i.deleted_at IS NULL"
    )))
    .bind(id)
    .bind(kind)
    .fetch_optional(tx)
    .await?
    .ok_or_else(|| issue(&format!("/{section}/{idx}/ingredient_id"), "not_found"))
}

fn required<T>(value: Option<T>, section: &str, idx: usize, field: &str) -> ApiResult<T> {
    value.ok_or_else(|| issue(&format!("/{section}/{idx}/{field}"), "required"))
}

async fn resolve_fermentable(
    tx: &mut PgConnection,
    idx: usize,
    l: &FermentableLine,
) -> ApiResult<Value> {
    let lib = match l.ingredient_id {
        Some(id) => Some(library_row(tx, "fermentable", "fermentables", idx, id).await?),
        None => None,
    };
    let lf = |k: &str| lib.as_ref().and_then(|v| opt_num(v, k));
    let ls = |k: &str| lib.as_ref().map(|v| text(v, k).to_owned());
    Ok(json!({
        "id": l.id.unwrap_or_else(Uuid::now_v7),
        "ingredient_id": l.ingredient_id,
        "name": required(l.name.clone().or_else(|| ls("name")), "fermentables", idx, "name")?,
        "amount_kg": l.amount_kg,
        "usage": l.usage.clone().unwrap_or_else(|| "mash".to_owned()),
        "grain_type": required(l.grain_type.clone().or_else(|| ls("grain_type")), "fermentables", idx, "grain_type")?,
        "yield_pct": required(l.yield_pct.or_else(|| lf("yield_pct")), "fermentables", idx, "yield_pct")?,
        "color_srm": l.color_srm.or_else(|| lf("color_srm")).unwrap_or(0.0),
        "not_fermentable": l.not_fermentable
            .or_else(|| lib.as_ref().and_then(|v| v.get("not_fermentable").and_then(Value::as_bool)))
            .unwrap_or(false),
        "late_extract_min": l.late_extract_min.unwrap_or(0.0),
        "max_in_batch_pct": l.max_in_batch_pct.or_else(|| lf("max_in_batch_pct")).unwrap_or(100.0),
        "ibu_l_per_kg": l.ibu_l_per_kg.or_else(|| lf("ibu_l_per_kg")).unwrap_or(0.0),
    }))
}

async fn resolve_hop(tx: &mut PgConnection, idx: usize, l: &HopLine) -> ApiResult<Value> {
    let lib = match l.ingredient_id {
        Some(id) => Some(library_row(tx, "hop", "hops", idx, id).await?),
        None => None,
    };
    let lf = |k: &str| lib.as_ref().and_then(|v| opt_num(v, k));
    let ls = |k: &str| lib.as_ref().map(|v| text(v, k).to_owned());
    Ok(json!({
        "id": l.id.unwrap_or_else(Uuid::now_v7),
        "ingredient_id": l.ingredient_id,
        "name": required(l.name.clone().or_else(|| ls("name")), "hops", idx, "name")?,
        "amount_kg": l.amount_kg,
        "alpha_pct": required(l.alpha_pct.or_else(|| lf("alpha_pct")), "hops", idx, "alpha_pct")?,
        "form": l.form.clone().or_else(|| ls("form")).unwrap_or_else(|| "pellet".to_owned()),
        "usage": l.usage.clone().unwrap_or_else(|| "boil".to_owned()),
        "time_min": l.time_min.unwrap_or(0.0),
        "whirlpool_temp_c": l.whirlpool_temp_c,
    }))
}

async fn resolve_yeast(tx: &mut PgConnection, idx: usize, l: &YeastLine) -> ApiResult<Value> {
    let lib = match l.ingredient_id {
        Some(id) => Some(library_row(tx, "yeast", "yeasts", idx, id).await?),
        None => None,
    };
    let lf = |k: &str| lib.as_ref().and_then(|v| opt_num(v, k));
    let ls = |k: &str| lib.as_ref().map(|v| text(v, k).to_owned());
    Ok(json!({
        "id": l.id.unwrap_or_else(Uuid::now_v7),
        "ingredient_id": l.ingredient_id,
        "name": required(l.name.clone().or_else(|| ls("name")), "yeasts", idx, "name")?,
        "form": required(l.form.clone().or_else(|| ls("form")), "yeasts", idx, "form")?,
        "min_attenuation_pct": required(l.min_attenuation_pct.or_else(|| lf("min_attenuation_pct")), "yeasts", idx, "min_attenuation_pct")?,
        "max_attenuation_pct": required(l.max_attenuation_pct.or_else(|| lf("max_attenuation_pct")), "yeasts", idx, "max_attenuation_pct")?,
        "tolerance_abv_pct": l.tolerance_abv_pct.or_else(|| lf("tolerance_abv_pct")),
        "add_to_secondary": l.add_to_secondary.unwrap_or(false),
        "total_cells_billion": l.total_cells_billion.or_else(|| lf("total_cells_billion")),
        "viable_cells_billion": l.viable_cells_billion,
        "packs": l.packs.unwrap_or(1.0),
        "viability_pct": l.viability_pct,
        "use_packages": l.use_packages.unwrap_or(true),
        "amount_kg": l.amount_kg.unwrap_or(0.0),
        "cells_per_ml_billion": l.cells_per_ml_billion.or_else(|| lf("cells_per_ml_billion")),
        "dry_pitch_rate_g_per_hl": l.dry_pitch_rate_g_per_hl.or_else(|| lf("dry_pitch_rate_g_per_hl")),
    }))
}

async fn resolve_misc(tx: &mut PgConnection, idx: usize, l: &MiscLine) -> ApiResult<Value> {
    let lib = match l.ingredient_id {
        Some(id) => Some(library_row(tx, "misc", "miscs", idx, id).await?),
        None => None,
    };
    let ls = |k: &str| lib.as_ref().map(|v| text(v, k).to_owned());
    Ok(json!({
        "id": l.id.unwrap_or_else(Uuid::now_v7),
        "ingredient_id": l.ingredient_id,
        "name": required(l.name.clone().or_else(|| ls("name")), "miscs", idx, "name")?,
        "amount": l.amount,
        "is_liquid": l.is_liquid
            .or_else(|| lib.as_ref().and_then(|v| v.get("is_liquid").and_then(Value::as_bool)))
            .unwrap_or(false),
        "usage": l.usage.clone()
            .or_else(|| lib.as_ref().and_then(|v| v.get("default_usage").and_then(Value::as_str).map(str::to_owned)))
            .unwrap_or_else(|| "boil".to_owned()),
        "time_min": l.time_min.unwrap_or(0.0),
    }))
}

/// Resolves the lines of one section (library snapshot) into full line objects.
async fn resolve_section(
    tx: &mut PgConnection,
    key: &str,
    input: &VersionInput,
) -> ApiResult<Option<Vec<Value>>> {
    macro_rules! each {
        ($list:expr, $f:ident) => {{
            let Some(list) = $list.as_ref() else {
                return Ok(None);
            };
            let mut out = Vec::with_capacity(list.len());
            for (i, l) in list.iter().enumerate() {
                out.push($f(tx, i, l).await?);
            }
            Ok(Some(out))
        }};
    }
    match key {
        "fermentables" => each!(input.fermentables, resolve_fermentable),
        "hops" => each!(input.hops, resolve_hop),
        "yeasts" => each!(input.yeasts, resolve_yeast),
        "miscs" => each!(input.miscs, resolve_misc),
        _ => Ok(input.water_additions.as_ref().map(|list| {
            list.iter()
                .map(|l| {
                    json!({
                        "id": l.id.unwrap_or_else(Uuid::now_v7),
                        "agent": l.agent,
                        "stage": l.stage.clone().unwrap_or_else(|| "mash".to_owned()),
                        "amount_kg": l.amount_kg,
                        "concentration_pct": l.concentration_pct,
                    })
                })
                .collect()
        })),
    }
}

/// Replaces the lines of one section of a draft.
async fn write_section(
    tx: &mut PgConnection,
    ws: Uuid,
    user: Uuid,
    version_id: Uuid,
    key: &str,
    rows: &[Value],
) -> ApiResult<()> {
    let (_, table, cols) = SECTIONS
        .iter()
        .find(|(k, _, _)| *k == key)
        .ok_or_else(|| ApiError::new(ErrorCode::Internal))?;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "DELETE FROM {table} WHERE recipe_version_id = $1"
    )))
    .bind(version_id)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    for (position, row) in rows.iter().enumerate() {
        let position = i32::try_from(position).unwrap_or(i32::MAX);
        let id = row
            .get("id")
            .and_then(Value::as_str)
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(Uuid::now_v7);
        let ingredient = row
            .get("ingredient_id")
            .and_then(Value::as_str)
            .and_then(|s| Uuid::parse_str(s).ok());
        let q = |sql: String| sqlx::AssertSqlSafe(sql);
        let head = format!(
            "INSERT INTO {table} (id, workspace_id, recipe_version_id, position, {cols}, created_by, updated_by) VALUES "
        );
        let result = match key {
            "fermentables" => {
                sqlx::query(q(format!(
                    "{head}($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$16)"
                )))
                .bind(id)
                .bind(ws)
                .bind(version_id)
                .bind(position)
                .bind(ingredient)
                .bind(text(row, "name"))
                .bind(num(row, "amount_kg"))
                .bind(text(row, "usage"))
                .bind(text(row, "grain_type"))
                .bind(num(row, "yield_pct"))
                .bind(num(row, "color_srm"))
                .bind(flag(row, "not_fermentable"))
                .bind(num(row, "late_extract_min"))
                .bind(num(row, "max_in_batch_pct"))
                .bind(num(row, "ibu_l_per_kg"))
                .bind(user)
                .execute(&mut *tx)
                .await
            }
            "hops" => {
                sqlx::query(q(format!(
                    "{head}($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$13)"
                )))
                .bind(id)
                .bind(ws)
                .bind(version_id)
                .bind(position)
                .bind(ingredient)
                .bind(text(row, "name"))
                .bind(num(row, "amount_kg"))
                .bind(num(row, "alpha_pct"))
                .bind(text(row, "form"))
                .bind(text(row, "usage"))
                .bind(num(row, "time_min"))
                .bind(opt_num(row, "whirlpool_temp_c"))
                .bind(user)
                .execute(&mut *tx)
                .await
            }
            "yeasts" => sqlx::query(q(format!(
                "{head}($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$20)"
            )))
            .bind(id)
            .bind(ws)
            .bind(version_id)
            .bind(position)
            .bind(ingredient)
            .bind(text(row, "name"))
            .bind(text(row, "form"))
            .bind(num(row, "min_attenuation_pct"))
            .bind(num(row, "max_attenuation_pct"))
            .bind(opt_num(row, "tolerance_abv_pct"))
            .bind(flag(row, "add_to_secondary"))
            .bind(opt_num(row, "total_cells_billion"))
            .bind(opt_num(row, "viable_cells_billion"))
            .bind(num(row, "packs"))
            .bind(opt_num(row, "viability_pct"))
            .bind(
                row.get("use_packages")
                    .and_then(Value::as_bool)
                    .unwrap_or(true),
            )
            .bind(num(row, "amount_kg"))
            .bind(opt_num(row, "cells_per_ml_billion"))
            .bind(opt_num(row, "dry_pitch_rate_g_per_hl"))
            .bind(user)
            .execute(&mut *tx)
            .await,
            "miscs" => {
                sqlx::query(q(format!("{head}($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$11)")))
                    .bind(id)
                    .bind(ws)
                    .bind(version_id)
                    .bind(position)
                    .bind(ingredient)
                    .bind(text(row, "name"))
                    .bind(num(row, "amount"))
                    .bind(flag(row, "is_liquid"))
                    .bind(text(row, "usage"))
                    .bind(num(row, "time_min"))
                    .bind(user)
                    .execute(&mut *tx)
                    .await
            }
            _ => {
                sqlx::query(q(format!("{head}($1,$2,$3,$4,$5,$6,$7,$8,$9,$9)")))
                    .bind(id)
                    .bind(ws)
                    .bind(version_id)
                    .bind(position)
                    .bind(text(row, "agent"))
                    .bind(text(row, "stage"))
                    .bind(num(row, "amount_kg"))
                    .bind(opt_num(row, "concentration_pct"))
                    .bind(user)
                    .execute(&mut *tx)
                    .await
            }
        };
        result.map_err(constraint_error)?;
    }
    Ok(())
}

// --- engine request and summary -----------------------------------------------------------

fn build_request(v: &Value) -> proto::CalculateRecipeRequest {
    use proto::{
        FermentableKind as FK, FermentableUse as FU, HopForm, HopUse, MiscUse, RecipeKind,
    };
    let kind = match text(v, "kind") {
        "extract" => RecipeKind::Extract,
        "partial_mash" => RecipeKind::PartialMash,
        "cider" => RecipeKind::Cider,
        "mead" => RecipeKind::Mead,
        "wine" => RecipeKind::Wine,
        "seltzer" => RecipeKind::Seltzer,
        _ => RecipeKind::AllGrain,
    };
    let eq = v.get("equipment_snapshot").cloned().unwrap_or(Value::Null);
    let equipment = proto::Equipment {
        trub_loss_l: num(&eq, "trub_loss_l"),
        pre_trub_loss_l: num(&eq, "pre_trub_loss_l"),
        altitude_m: num(&eq, "altitude_m"),
        hop_util_pct: opt_num(&eq, "hop_utilization_pct").unwrap_or(100.0),
        whirlpool_carry: flag(&eq, "whirlpool_carry"),
        whirlpool_time_min: num(&eq, "whirlpool_time_min"),
    };
    let fermentables: Vec<proto::Fermentable> = lines(v, "fermentables")
        .iter()
        .map(|l| proto::Fermentable {
            name: text(l, "name").to_owned(),
            amount_kg: num(l, "amount_kg"),
            yield_pct: num(l, "yield_pct"),
            color_srm: num(l, "color_srm"),
            kind: match text(l, "grain_type") {
                "extract" => FK::Extract,
                "sugar" => FK::Sugar,
                "adjunct" => FK::Adjunct,
                "dry_extract" => FK::DryExtract,
                "fruit" => FK::Fruit,
                "juice" => FK::Juice,
                "honey" => FK::Honey,
                "acid" => FK::Acid,
                _ => FK::Grain,
            } as i32,
            usage: match text(l, "usage") {
                "steep" => FU::Steep,
                "boil" => FU::Boil,
                "whirlpool" => FU::Whirlpool,
                "primary" => FU::Primary,
                "secondary" => FU::Secondary,
                "tertiary" => FU::Tertiary,
                "bottling" => FU::Bottling,
                "sparge" => FU::Sparge,
                _ => FU::Mash,
            } as i32,
            not_fermentable: flag(l, "not_fermentable"),
            late_extract_min: num(l, "late_extract_min"),
            max_in_batch_pct: num(l, "max_in_batch_pct"),
            ibu_l_per_kg: num(l, "ibu_l_per_kg"),
        })
        .collect();
    let hops = lines(v, "hops")
        .iter()
        .map(|l| {
            let form = match text(l, "form") {
                "plug" => HopForm::Plug,
                "leaf" => HopForm::Leaf,
                "co2_extract" => HopForm::Co2Extract,
                "isomerized_extract" => HopForm::IsomerizedExtract,
                _ => HopForm::Pellet,
            };
            // The engine takes grams (mL for extracts); the line holds kg (L for extracts).
            proto::Hop {
                amount: num(l, "amount_kg") * 1000.0,
                alpha_pct: num(l, "alpha_pct"),
                form: form as i32,
                usage: match text(l, "usage") {
                    "dry_hop" => HopUse::DryHop,
                    "mash" => HopUse::Mash,
                    "first_wort" => HopUse::FirstWort,
                    "whirlpool" => HopUse::Whirlpool,
                    _ => HopUse::Boil,
                } as i32,
                time_min: num(l, "time_min"),
                whirlpool_temp_c: num(l, "whirlpool_temp_c"),
            }
        })
        .collect();
    let yeasts = lines(v, "yeasts")
        .iter()
        .map(|l| proto::RecipeYeast {
            min_attenuation_pct: num(l, "min_attenuation_pct"),
            max_attenuation_pct: num(l, "max_attenuation_pct"),
            tolerance_abv_pct: num(l, "tolerance_abv_pct"),
            add_to_secondary: flag(l, "add_to_secondary"),
        })
        .collect();
    let miscs = lines(v, "miscs")
        .iter()
        .map(|l| proto::Misc {
            usage: match text(l, "usage") {
                "mash" => MiscUse::Mash,
                "primary" => MiscUse::Primary,
                "secondary" => MiscUse::Secondary,
                "bottling" => MiscUse::Bottling,
                "sparge" => MiscUse::Sparge,
                "whirlpool" => MiscUse::Whirlpool,
                _ => MiscUse::Boil,
            } as i32,
            time_min: num(l, "time_min"),
        })
        .collect();
    let mash_src = v.get("mash_snapshot").cloned().unwrap_or(Value::Null);
    let grain_kg: f64 = lines(v, "fermentables")
        .iter()
        .filter(|l| text(l, "grain_type") == "grain" && text(l, "usage") == "mash")
        .map(|l| num(l, "amount_kg"))
        .sum();
    let steps = lines(&mash_src, "steps")
        .iter()
        .map(|s| proto::MashStep {
            kind: match text(s, "kind") {
                "decoction" => proto::MashStepKind::Decoction,
                "temperature" => proto::MashStepKind::Temperature,
                _ => proto::MashStepKind::Infusion,
            } as i32,
            temp_c: num(s, "temp_c"),
            infusion_l: num(s, "infusion_l"),
        })
        .collect();
    let o = v.get("options").cloned().unwrap_or(Value::Null);
    let options = proto::RecipeOptions {
        steep_efficiency_pct: opt_num(&o, "steep_efficiency_pct"),
        mash_fg_adjust: o.get("mash_fg_adjust").and_then(Value::as_bool),
        mash_fg_adj_temp_c: opt_num(&o, "mash_fg_adj_temp_c"),
        mash_fg_adj_slope_pct_per_c: opt_num(&o, "mash_fg_adj_slope_pct_per_c"),
        hop_first_wort_adj_pct: opt_num(&o, "hop_first_wort_adj_pct"),
        hop_type_adj: o.get("hop_type_adj").and_then(Value::as_bool),
        hop_leaf_adj_pct: opt_num(&o, "hop_leaf_adj_pct"),
        hop_plug_adj_pct: opt_num(&o, "hop_plug_adj_pct"),
        hop_pellet_adj_pct: opt_num(&o, "hop_pellet_adj_pct"),
        hop_mash_adj_pct: opt_num(&o, "hop_mash_adj_pct"),
    };
    let ibu_equation = match text(&o, "ibu_equation") {
        "rager" => proto::IbuEquation::Rager,
        "garetz" => proto::IbuEquation::Garetz,
        _ => proto::IbuEquation::Tinseth,
    };
    let batch = num(v, "batch_volume_l");
    proto::CalculateRecipeRequest {
        kind: kind as i32,
        batch_volume_l: batch,
        boil_volume_l: opt_num(v, "boil_volume_l").unwrap_or(batch),
        boil_time_min: num(v, "boil_time_min"),
        efficiency_pct: num(v, "efficiency_pct"),
        equipment: Some(equipment),
        fermentables,
        hops,
        yeasts,
        miscs,
        mash: Some(proto::Mash {
            grain_weight_kg: grain_kg,
            biab: flag(&mash_src, "biab"),
            steps,
        }),
        options: Some(options),
        ibu_equation: ibu_equation as i32,
        // From the equipment snapshot (T2.40). Both set: the engine uses their product. One set
        // alone: the engine rejects the request (the database allows only both or none).
        conversion_efficiency_pct: opt_num(&eq, "conversion_efficiency_pct"),
        lauter_efficiency_pct: opt_num(&eq, "lauter_efficiency_pct"),
    }
}

/// Million viable cells per mL of wort (`est_cells_ml`: cells in billions times 1000 over the
/// volume in mL). TODO: move into the engine.
fn est_cells_ml(v: &Value) -> f64 {
    let billions: f64 = lines(v, "yeasts")
        .iter()
        .map(|y| {
            let per_pack = opt_num(y, "viable_cells_billion").unwrap_or_else(|| {
                opt_num(y, "total_cells_billion").unwrap_or(0.0)
                    * opt_num(y, "viability_pct").unwrap_or(100.0)
                    / 100.0
            });
            per_pack * opt_num(y, "packs").unwrap_or(1.0)
        })
        .sum();
    let volume_ml = num(v, "batch_volume_l") * 1000.0;
    billions * 1000.0 / volume_ml.max(0.1)
}

/// Share of the fermentable extract that comes from honey, percent. The weight of a fermentable
/// is its amount times its yield. TODO: use the OG points of the engine.
fn honey_pct(v: &Value) -> Option<f64> {
    let points = |l: &Value| num(l, "amount_kg") * num(l, "yield_pct");
    let all: f64 = lines(v, "fermentables")
        .iter()
        .filter(|l| !flag(l, "not_fermentable"))
        .map(points)
        .sum();
    let honey: f64 = lines(v, "fermentables")
        .iter()
        .filter(|l| text(l, "grain_type") == "honey" && !flag(l, "not_fermentable"))
        .map(points)
        .sum();
    (all > 0.0).then(|| honey / all * 100.0)
}

fn summary_json(s: &proto::RecipeSummary, engine_version: &str, v: &Value) -> Value {
    json!({
        "schema_version": 1,
        "engine_version": engine_version,
        "og_sg": s.og_sg, "fg_sg": s.fg_sg, "virtual_og_sg": s.virtual_og_sg,
        "pre_boil_sg": s.pre_boil_sg, "abv_pct": s.abv_pct, "abw_pct": s.abw_pct,
        "ibu": s.ibu, "color_srm": s.color_srm, "color_ebc": s.color_ebc,
        "mash_efficiency_pct": s.mash_efficiency_pct,
        "energy_kcal_per_100ml": s.energy_kcal_per_100ml,
        "carbohydrates_g_per_100ml": s.carbohydrates_g_per_100ml,
        "est_cells_ml": est_cells_ml(v),
        "honey_pct": honey_pct(v),
    })
}

/// Calculates a version JSON with the engine. Returns the summary and where it came from.
pub(crate) async fn calculate_json(
    state: &AppState,
    tx: &mut PgConnection,
    ctx: &RequestContext,
    ws: Uuid,
    v: &Value,
    target: CalcTarget,
) -> ApiResult<(Value, &'static str)> {
    let request = build_request(v);
    let mut scope = CalcScope::new(ws, Some(ctx.identity.user_id));
    scope.target = target;
    let done = state
        .engine
        .calculate(tx, &scope, "recipe_summary", request, |ch, r| async move {
            proto::recipe_service_client::RecipeServiceClient::new(ch)
                .calculate(r)
                .await
        })
        .await?;
    let source = match done.source {
        crate::engine::Source::Redis => "redis",
        crate::engine::Source::Database => "database",
        crate::engine::Source::Engine => "engine",
    };
    Ok((summary_json(&done.output, &done.engine_version, v), source))
}

/// The version JSON, plus the fields of `patch` that differ from it (for the `412` body).
fn with_conflicts(mut current: Value, patch: &Value) -> Value {
    let fields: Vec<String> = patch
        .as_object()
        .map(|p| {
            p.iter()
                .filter(|(k, v)| current.get(k.as_str()) != Some(*v))
                .map(|(k, _)| k.clone())
                .collect()
        })
        .unwrap_or_default();
    if let Some(m) = current.as_object_mut() {
        m.insert("conflicting_fields".to_owned(), json!(fields));
    }
    current
}

async fn recipe_row_lock(tx: &mut PgConnection, id: Uuid) -> ApiResult<(bool, Option<Uuid>)> {
    sqlx::query_as(
        "SELECT is_locked, draft_version_id FROM recipes \
         WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(tx)
    .await?
    .ok_or_else(not_found)
}

async fn insert_header(
    tx: &mut PgConnection,
    ws: Uuid,
    user: Uuid,
    id: Uuid,
    recipe_id: Uuid,
    number: i32,
    h: &Header,
) -> ApiResult<()> {
    sqlx::query(
        "INSERT INTO recipe_versions (id, workspace_id, recipe_id, number, kind, style_id, \
             equipment_id, batch_volume_l, boil_volume_l, boil_time_min, efficiency_pct, \
             target_og_sg, target_fg_sg, target_ibu, target_srm, target_abv_pct, target_co2_vols, \
             change_note, equipment_snapshot, mash_snapshot, fermentation_snapshot, water_snapshot, \
             options, created_by, updated_by) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$24)",
    )
    .bind(id)
    .bind(ws)
    .bind(recipe_id)
    .bind(number)
    .bind(&h.kind)
    .bind(h.style_id)
    .bind(h.equipment_id)
    .bind(h.batch_volume_l)
    .bind(h.boil_volume_l)
    .bind(h.boil_time_min)
    .bind(h.efficiency_pct)
    .bind(h.targets[0])
    .bind(h.targets[1])
    .bind(h.targets[2])
    .bind(h.targets[3])
    .bind(h.targets[4])
    .bind(h.targets[5])
    .bind(&h.change_note)
    .bind(&h.equipment_snapshot)
    .bind(&h.mash)
    .bind(&h.fermentation)
    .bind(&h.water)
    .bind(&h.options)
    .bind(user)
    .execute(tx)
    .await
    .map_err(constraint_error)?;
    Ok(())
}

// --- recipe endpoints ---------------------------------------------------------------------

macro_rules! recipe_example {
    () => {
        json!({
        "id": "0192f1c07a3b7000800000000000000b", "name": "House IPA", "folder_id": null,
        "is_locked": false, "notes": null, "approved_version_id": null,
        "draft_version_id": "0192f1c07a3b7000800000000000000c", "kind": "all_grain",
        "tag_ids": [], "is_favorite": false, "created_at": "2026-11-20T10:00:00Z",
        "updated_at": "2026-11-20T10:00:00Z", "deleted_at": null, "version": 1
        })
    };
}

macro_rules! version_example {
    () => {
        json!({
        "id": "0192f1c07a3b7000800000000000000c", "recipe_id": "0192f1c07a3b7000800000000000000b",
        "number": 1, "status": "draft", "kind": "all_grain", "batch_volume_l": 20.0,
        "boil_time_min": 60.0, "efficiency_pct": 72.0, "fermentables": [], "hops": [],
        "yeasts": [], "miscs": [], "water_additions": [], "calc_summary": null, "version": 1
        })
    };
}

/// Lists the recipes of the workspace.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/recipes",
    tag = "recipes",
    operation_id = "recipe_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("q" = Option<String>, Query, description = "Text in the name (case blind)"),
        ("folder_id" = Option<Uuid>, Query, description = "Folder"),
        ("tag_id" = Option<Uuid>, Query, description = "Tag"),
        ("favorite" = Option<bool>, Query, description = "Only favorites of the caller"),
        ("kind" = Option<String>, Query, description = "Kind of the version (`mead`, `cider`, ...)"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page"),
        ("sort" = Option<String>, Query, description = "`name` (default) or `-name`")
    ),
    responses((status = 200, description = "A page of recipes", body = Page<Recipe>,
        example = json!({"items": [recipe_example!()], "next_cursor": null})))
)]
pub async fn recipe_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<RecipeFilter>,
) -> ApiResult<Json<Page<Recipe>>> {
    ctx.require(READ_PERMISSION)?;
    let paging = paging(&params, &["name"], "name")?;
    if filter.kind.as_deref().is_some_and(|k| !KINDS.contains(&k)) {
        return Err(issue("kind", "invalid_value"));
    }
    let (cmp, dir) = paging.dir();
    let sql = sqlx::AssertSqlSafe(format!(
        "SELECT {cols} FROM recipes r WHERE r.deleted_at IS NULL \
           AND ($2::text IS NULL OR strpos(lower(r.name), lower($2)) > 0) \
           AND ($3::uuid IS NULL OR r.folder_id = $3) \
           AND ($4::uuid IS NULL OR EXISTS (SELECT 1 FROM recipe_tags t \
                 WHERE t.recipe_id = r.id AND t.tag_id = $4)) \
           AND ($5::bool IS NOT TRUE OR EXISTS (SELECT 1 FROM favorites f \
                 WHERE f.recipe_id = r.id AND f.user_id = $1)) \
           AND ($6::text IS NULL OR EXISTS (SELECT 1 FROM recipe_versions v \
                 WHERE v.id = COALESCE(r.approved_version_id, r.draft_version_id) AND v.kind = $6)) \
           AND ($7::text IS NULL OR (r.name, r.id) {cmp} ($7::text, $8::uuid)) \
         ORDER BY r.name {dir}, r.id {dir} LIMIT $9",
        cols = recipe_cols("$1")
    ));
    let mut tx = ctx.begin(&state.db).await?;
    let rows: Vec<Recipe> = sqlx::query_as(sql)
        .bind(ctx.identity.user_id)
        .bind(&filter.q)
        .bind(filter.folder_id)
        .bind(filter.tag_id)
        .bind(filter.favorite)
        .bind(&filter.kind)
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

/// Creates a recipe with its first draft.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/recipes",
    tag = "recipes",
    operation_id = "recipe_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = NewRecipe,
    responses((status = 201, description = "Recipe created with draft number 1", body = Recipe,
        example = json!(recipe_example!())))
)]
pub async fn recipe_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(new): ApiJson<NewRecipe>,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let name = new.name.trim().to_owned();
    if name.is_empty() || name.chars().count() > 200 {
        return Err(issue("/name", "invalid_value"));
    }
    let user = ctx.identity.user_id;
    let id = Uuid::now_v7();
    let version_id = Uuid::now_v7();
    let mut tx = ctx.begin(&state.db).await?;
    sqlx::query(
        "INSERT INTO recipes (id, workspace_id, name, folder_id, notes, created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $6)",
    )
    .bind(id)
    .bind(ws)
    .bind(&name)
    .bind(new.folder_id)
    .bind(&new.notes)
    .bind(user)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    let header = resolve_header(&mut tx, &new.version, None).await?;
    insert_header(&mut tx, ws, user, version_id, id, 1, &header).await?;
    for (key, _, _) in SECTIONS {
        if let Some(rows) = resolve_section(&mut tx, key, &new.version).await? {
            write_section(&mut tx, ws, user, version_id, key, &rows).await?;
        }
    }
    sqlx::query("UPDATE recipes SET draft_version_id = $2 WHERE id = $1")
        .bind(id)
        .bind(version_id)
        .execute(&mut *tx)
        .await
        .map_err(constraint_error)?;
    for tag_id in &new.tag_ids {
        sqlx::query(
            "INSERT INTO recipe_tags (workspace_id, recipe_id, tag_id, created_by) \
             VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
        )
        .bind(ws)
        .bind(id)
        .bind(tag_id)
        .bind(user)
        .execute(&mut *tx)
        .await
        .map_err(constraint_error)?;
    }
    let row = fetch_recipe(&mut tx, user, id).await?;
    recipe_event(&mut tx, ws, "recipe.created", id, row.version, None).await?;
    tx.commit().await?;
    Ok(created_response(
        &format!("/v1/workspaces/{ws}/recipes/{id}"),
        row.version,
        &row,
    ))
}

/// Reads a recipe.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/recipes/{id}",
    tag = "recipes",
    operation_id = "recipe_get",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Recipe")
    ),
    responses(
        (status = 200, description = "The recipe", body = Recipe, example = json!(recipe_example!())),
        (status = 304, description = "Not modified (`If-None-Match` matched)")
    )
)]
pub async fn recipe_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    ctx.require(READ_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let row = fetch_recipe(&mut tx, ctx.identity.user_id, id).await?;
    Ok(concurrency::conditional_get(&headers, row.version, &row))
}

/// Changes the name, folder, notes or lock of a recipe.
#[utoipa::path(
    patch,
    path = "/v1/workspaces/{workspace_id}/recipes/{id}",
    tag = "recipes",
    operation_id = "recipe_update",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Recipe"),
        ("If-Match" = String, Header, description = "Version `ETag` of the recipe")
    ),
    request_body(content = RecipePatch, content_type = "application/merge-patch+json"),
    responses((status = 200, description = "The updated recipe", body = Recipe,
        example = json!(recipe_example!())))
)]
pub async fn recipe_update(
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
    let before = fetch_recipe(&mut tx, user, id).await?;
    check_if_match(Concurrency::Optimistic, &headers, before.version, || {
        with_conflicts(json!(before), &patch)
    })?;
    let mut merged = json!({
        "name": before.name, "folder_id": before.folder_id,
        "notes": before.notes, "is_locked": before.is_locked
    });
    merge(&mut merged, &patch);
    let input: RecipePatch = serde_json::from_value(merged).map_err(|e| {
        ApiError::validation(vec![
            FieldIssue::new("/", "invalid").with_params(json!({"reason": e.to_string()})),
        ])
    })?;
    let name = input.name.unwrap_or_default().trim().to_owned();
    if name.is_empty() || name.chars().count() > 200 {
        return Err(issue("/name", "invalid_value"));
    }
    sqlx::query(
        "UPDATE recipes SET name = $2, folder_id = $3, notes = $4, is_locked = $5, updated_by = $6 \
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(&name)
    .bind(input.folder_id)
    .bind(&input.notes)
    .bind(input.is_locked.unwrap_or(false))
    .bind(user)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    let row = fetch_recipe(&mut tx, user, id).await?;
    tx.commit().await?;
    Ok(concurrency::record_response(
        StatusCode::OK,
        row.version,
        &row,
    ))
}

/// Deletes a recipe (soft delete). Batches that use its versions keep their data.
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/recipes/{id}",
    tag = "recipes",
    operation_id = "recipe_delete",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Recipe"),
        ("If-Match" = String, Header, description = "Version `ETag` of the recipe")
    ),
    responses((status = 204, description = "Deleted"))
)]
pub async fn recipe_delete(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let user = ctx.identity.user_id;
    let mut tx = ctx.begin(&state.db).await?;
    let before = fetch_recipe(&mut tx, user, id).await?;
    check_if_match(Concurrency::Optimistic, &headers, before.version, || {
        json!(before)
    })?;
    sqlx::query("UPDATE recipes SET deleted_at = now(), updated_by = $2 WHERE id = $1")
        .bind(id)
        .bind(user)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

// --- version endpoints --------------------------------------------------------------------

/// Lists the versions of a recipe, newest first (header only, no lines).
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/recipes/{id}/versions",
    tag = "recipes",
    operation_id = "recipe_version_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Recipe")
    ),
    responses((status = 200, description = "The versions", body = Object,
        example = json!({"items": [{"id": "0192f1c07a3b7000800000000000000c", "number": 1,
            "status": "approved", "kind": "all_grain", "change_note": null,
            "approved_at": "2026-11-20T10:00:00Z", "version": 2}], "next_cursor": null})))
)]
pub async fn recipe_version_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<Value>> {
    ctx.require(READ_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    fetch_recipe(&mut tx, ctx.identity.user_id, id).await?;
    let items: Value = sqlx::query_scalar(
        "SELECT COALESCE(jsonb_agg(jsonb_build_object('id', v.id, 'recipe_id', v.recipe_id, \
             'number', v.number, 'status', v.status, 'kind', v.kind, \
             'batch_volume_l', v.batch_volume_l, 'change_note', v.change_note, \
             'approved_at', v.approved_at, 'created_at', v.created_at, 'version', v.version) \
             ORDER BY v.number DESC), '[]'::jsonb) \
         FROM recipe_versions v WHERE v.recipe_id = $1 AND v.deleted_at IS NULL",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    Ok(Json(json!({"items": items, "next_cursor": null})))
}

/// Starts a new draft that copies a published version.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/recipes/{id}/versions",
    tag = "recipes",
    operation_id = "recipe_version_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Recipe"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = NewDraft,
    responses((status = 201, description = "The new draft", body = Object, example = json!(version_example!())))
)]
pub async fn recipe_version_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    body: axum::body::Bytes,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let input: NewDraft = optional_body(&body)?;
    let user = ctx.identity.user_id;
    let mut tx = ctx.begin(&state.db).await?;
    let (locked, draft) = recipe_row_lock(&mut tx, id).await?;
    if locked {
        return Err(conflict("/", "recipe_locked"));
    }
    if draft.is_some() {
        return Err(conflict("/", "draft_exists"));
    }
    let source: Option<Uuid> = match input.from_version_id {
        Some(v) => sqlx::query_scalar(
            "SELECT id FROM recipe_versions WHERE id = $1 AND recipe_id = $2 AND deleted_at IS NULL",
        )
        .bind(v)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?,
        None => sqlx::query_scalar(
            "SELECT id FROM recipe_versions WHERE recipe_id = $1 AND deleted_at IS NULL \
             ORDER BY (status = 'approved') DESC, number DESC LIMIT 1",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?,
    };
    let source = source.ok_or_else(|| issue("/from_version_id", "not_found"))?;
    let new_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO recipe_versions (id, workspace_id, recipe_id, number, kind, style_id, \
             equipment_id, batch_volume_l, boil_volume_l, boil_time_min, efficiency_pct, \
             target_og_sg, target_fg_sg, target_ibu, target_srm, target_abv_pct, target_co2_vols, \
             change_note, equipment_snapshot, mash_snapshot, fermentation_snapshot, water_snapshot, \
             options, created_by, updated_by) \
         SELECT $2, workspace_id, recipe_id, \
             (SELECT max(number) + 1 FROM recipe_versions WHERE recipe_id = $3), kind, style_id, \
             equipment_id, batch_volume_l, boil_volume_l, boil_time_min, efficiency_pct, \
             target_og_sg, target_fg_sg, target_ibu, target_srm, target_abv_pct, target_co2_vols, \
             $4, equipment_snapshot, mash_snapshot, fermentation_snapshot, water_snapshot, \
             options, $5, $5 FROM recipe_versions WHERE id = $1",
    )
    .bind(source)
    .bind(new_id)
    .bind(id)
    .bind(&input.change_note)
    .bind(user)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    for (_, table, cols) in SECTIONS {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "INSERT INTO {table} (id, workspace_id, recipe_version_id, position, {cols}, created_by, updated_by) \
             SELECT gen_random_uuid(), workspace_id, $2, position, {cols}, $3, $3 \
             FROM {table} WHERE recipe_version_id = $1"
        )))
        .bind(source)
        .bind(new_id)
        .bind(user)
        .execute(&mut *tx)
        .await
        .map_err(constraint_error)?;
    }
    sqlx::query("UPDATE recipes SET draft_version_id = $2, updated_by = $3 WHERE id = $1")
        .bind(id)
        .bind(new_id)
        .bind(user)
        .execute(&mut *tx)
        .await
        .map_err(constraint_error)?;
    let version = version_json(&mut tx, new_id).await?;
    recipe_event(&mut tx, ws, "recipe.draft_started", id, 0, None).await?;
    tx.commit().await?;
    let row_version = version["version"].as_i64().unwrap_or(1);
    Ok(created_response(
        &format!("/v1/workspaces/{ws}/recipe-versions/{new_id}"),
        row_version,
        &version,
    ))
}

/// Reads a version with its lines, snapshots and (when published) the calculation summary.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/recipe-versions/{id}",
    tag = "recipes",
    operation_id = "recipe_version_get",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Recipe version")
    ),
    responses(
        (status = 200, description = "The version", body = Object, example = json!(version_example!())),
        (status = 304, description = "Not modified (`If-None-Match` matched)")
    )
)]
pub async fn recipe_version_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    ctx.require(READ_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let v = version_json(&mut tx, id).await?;
    let version = v["version"].as_i64().unwrap_or(1);
    Ok(concurrency::conditional_get(&headers, version, &v))
}

/// Changes a draft. Arrays replace the lines of their section. A stale `If-Match` answers `412`
/// with the current version and `conflicting_fields` (the fields of the patch that differ).
#[utoipa::path(
    patch,
    path = "/v1/workspaces/{workspace_id}/recipe-versions/{id}",
    tag = "recipes",
    operation_id = "recipe_version_update",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Recipe version"),
        ("If-Match" = String, Header, description = "Version `ETag` of the draft")
    ),
    request_body(content = VersionInput, content_type = "application/merge-patch+json"),
    responses((status = 200, description = "The updated draft", body = Object, example = json!(version_example!())))
)]
pub async fn recipe_version_update(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    MergePatch(patch): MergePatch<Value>,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    if !patch.is_object() {
        return Err(issue("/", "invalid"));
    }
    let user = ctx.identity.user_id;
    let mut tx = ctx.begin(&state.db).await?;
    let (status, recipe_id): (String, Uuid) = sqlx::query_as(
        "SELECT status, recipe_id FROM recipe_versions \
         WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(not_found)?;
    let current = version_json(&mut tx, id).await?;
    if status != "draft" {
        return Err(conflict("/status", "version_immutable"));
    }
    let (locked, _) = recipe_row_lock(&mut tx, recipe_id).await?;
    if locked {
        return Err(conflict("/", "recipe_locked"));
    }
    let row_version = current["version"].as_i64().unwrap_or(0);
    check_if_match(Concurrency::Optimistic, &headers, row_version, || {
        with_conflicts(current.clone(), &patch)
    })?;
    let mut base = Map::new();
    for key in HEADER_KEYS {
        if let Some(v) = current.get(*key) {
            base.insert((*key).to_owned(), v.clone());
        }
    }
    for (key, _, _) in SECTIONS {
        if let Some(v) = current.get(*key) {
            base.insert((*key).to_owned(), v.clone());
        }
    }
    let mut merged = Value::Object(base);
    merge(&mut merged, &patch);
    let input: VersionInput = serde_json::from_value(merged).map_err(|e| {
        ApiError::validation(vec![
            FieldIssue::new("/", "invalid").with_params(json!({"reason": e.to_string()})),
        ])
    })?;
    let previous = Some((
        current
            .get("equipment_id")
            .and_then(Value::as_str)
            .and_then(|s| Uuid::parse_str(s).ok()),
        current
            .get("equipment_snapshot")
            .cloned()
            .unwrap_or(json!({"schema_version": 1})),
    ));
    let h = resolve_header(&mut tx, &input, previous).await?;
    sqlx::query(
        "UPDATE recipe_versions SET kind = $2, style_id = $3, equipment_id = $4, \
             batch_volume_l = $5, boil_volume_l = $6, boil_time_min = $7, efficiency_pct = $8, \
             target_og_sg = $9, target_fg_sg = $10, target_ibu = $11, target_srm = $12, \
             target_abv_pct = $13, target_co2_vols = $14, change_note = $15, \
             equipment_snapshot = $16, mash_snapshot = $17, fermentation_snapshot = $18, \
             water_snapshot = $19, options = $20, updated_by = $21 WHERE id = $1",
    )
    .bind(id)
    .bind(&h.kind)
    .bind(h.style_id)
    .bind(h.equipment_id)
    .bind(h.batch_volume_l)
    .bind(h.boil_volume_l)
    .bind(h.boil_time_min)
    .bind(h.efficiency_pct)
    .bind(h.targets[0])
    .bind(h.targets[1])
    .bind(h.targets[2])
    .bind(h.targets[3])
    .bind(h.targets[4])
    .bind(h.targets[5])
    .bind(&h.change_note)
    .bind(&h.equipment_snapshot)
    .bind(&h.mash)
    .bind(&h.fermentation)
    .bind(&h.water)
    .bind(&h.options)
    .bind(user)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    for (key, _, _) in SECTIONS {
        if patch.get(*key).is_some()
            && let Some(rows) = resolve_section(&mut tx, key, &input).await?
        {
            write_section(&mut tx, ws, user, id, key, &rows).await?;
        }
    }
    let v = version_json(&mut tx, id).await?;
    recipe_event(&mut tx, ws, "recipe.updated", recipe_id, 0, None).await?;
    tx.commit().await?;
    let version = v["version"].as_i64().unwrap_or(1);
    Ok(concurrency::record_response(StatusCode::OK, version, &v))
}

/// Calculates a version with the engine. A published version returns its stored summary.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/recipe-versions/{id}/calculate",
    tag = "recipes",
    operation_id = "recipe_version_calculate",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Recipe version")
    ),
    responses((status = 200, description = "The calculation summary", body = Object,
        example = json!({"schema_version": 1, "engine_version": "0.1.0", "og_sg": 1.060,
            "fg_sg": 1.012, "abv_pct": 6.3, "ibu": 40.0, "color_srm": 8.0, "est_cells_ml": 0.0,
            "honey_pct": null, "source": "engine"})))
)]
pub async fn recipe_version_calculate(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<Value>> {
    ctx.require(READ_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    let v = version_json(&mut tx, id).await?;
    if let Some(stored) = v.get("calc_summary").filter(|s| s.is_object()) {
        let mut out = stored.clone();
        out["source"] = json!("stored");
        return Ok(Json(out));
    }
    let (mut summary, source) =
        calculate_json(&state, &mut tx, &ctx, ws, &v, CalcTarget::None).await?;
    summary["source"] = json!(source);
    tx.commit().await?;
    Ok(Json(summary))
}

/// Publishes a draft: refreshes the equipment snapshot, calculates, stores the summary and makes
/// the version immutable. Home mode needs `recipe.edit`; Business mode needs `recipe.approve`.
/// TODO: the review steps of Business mode (submit, review, reject) come in stage 6.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/recipe-versions/{id}/publish",
    tag = "recipes",
    operation_id = "recipe_version_publish",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Recipe version"),
        ("If-Match" = String, Header, description = "Version `ETag` of the draft"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = PublishBody,
    responses((status = 200, description = "The published version", body = Object,
        example = json!(version_example!())))
)]
pub async fn recipe_version_publish(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> ApiResult<Response> {
    let ws = workspace(&ctx)?;
    let input: PublishBody = optional_body(&body)?;
    let user = ctx.identity.user_id;
    let mut tx = ctx.begin(&state.db).await?;
    let mode: String = sqlx::query_scalar("SELECT mode FROM workspaces WHERE id = $1")
        .bind(ws)
        .fetch_one(&mut *tx)
        .await?;
    ctx.require(if mode == "business" {
        APPROVE_PERMISSION
    } else {
        WRITE_PERMISSION
    })?;
    let (status, recipe_id, equipment_id): (String, Uuid, Option<Uuid>) = sqlx::query_as(
        "SELECT status, recipe_id, equipment_id FROM recipe_versions \
         WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(not_found)?;
    let current = version_json(&mut tx, id).await?;
    let row_version = current["version"].as_i64().unwrap_or(0);
    check_if_match(Concurrency::Optimistic, &headers, row_version, || {
        current.clone()
    })?;
    if status != "draft" {
        return Err(conflict("/status", "version_immutable"));
    }
    if lines(&current, "fermentables").is_empty() {
        return Err(ApiError::validation(vec![FieldIssue::new(
            "/fermentables",
            "too_few_items",
        )]));
    }
    recipe_row_lock(&mut tx, recipe_id).await?;
    // The equipment snapshot is refreshed at the publish.
    let mut v = current.clone();
    if let Some(eq) = equipment_id {
        let snapshot = equipment_snapshot(&mut tx, eq).await?;
        v["equipment_snapshot"] = snapshot;
    }
    let (summary, _) =
        calculate_json(&state, &mut tx, &ctx, ws, &v, CalcTarget::RecipeVersion(id)).await?;
    let engine_version = summary["engine_version"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    sqlx::query(
        "UPDATE recipe_versions SET status = 'approved', approved_by = $2, approved_at = now(), \
             submitted_by = $2, submitted_at = now(), engine_version = $3, calculated_at = now(), \
             calc_summary = $4, equipment_snapshot = $5, \
             change_note = COALESCE($6, change_note), updated_by = $2 WHERE id = $1",
    )
    .bind(id)
    .bind(user)
    .bind(&engine_version)
    .bind(&summary)
    .bind(&v["equipment_snapshot"])
    .bind(&input.change_note)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    sqlx::query(
        "UPDATE recipes SET approved_version_id = $2, draft_version_id = NULL, updated_by = $3 \
         WHERE id = $1",
    )
    .bind(recipe_id)
    .bind(id)
    .bind(user)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    db::append_audit(
        &mut tx,
        ws,
        &NewAuditEntry {
            actor_kind: ctx.identity.actor_kind,
            actor_id: Some(user),
            event: AuditEvent::RecipeApprove,
            record_kind: "recipe_version".to_owned(),
            record_id: Some(id),
            previous_value: None,
            new_value: Some(json!({"recipe_id": recipe_id, "number": current["number"]})),
            reason: None,
        },
    )
    .await?;
    let out = version_json(&mut tx, id).await?;
    recipe_event(
        &mut tx,
        ws,
        "recipe.published",
        recipe_id,
        0,
        Some(json!({"version_id": id})),
    )
    .await?;
    tx.commit().await?;
    let version = out["version"].as_i64().unwrap_or(1);
    Ok(concurrency::record_response(StatusCode::OK, version, &out))
}

fn scale_lines(v: &Value, key: &str, fields: &[&str], factor: f64) -> Value {
    let rows: Vec<Value> = lines(v, key)
        .iter()
        .map(|l| {
            let mut l = l.clone();
            if let Some(m) = l.as_object_mut() {
                m.remove("id");
                for f in fields {
                    if let Some(x) = m.get(*f).and_then(Value::as_f64) {
                        m.insert((*f).to_owned(), json!(x * factor));
                    }
                }
            }
            l
        })
        .collect();
    Value::Array(rows)
}

/// Previews a recipe scaled by a factor or to a new batch volume. Nothing is saved: send the
/// lines in a patch to a draft to keep them.
/// TODO: rounding of small amounts comes with the scaling screen (stage 3).
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/recipe-versions/{id}/scale",
    tag = "recipes",
    operation_id = "recipe_version_scale",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Recipe version")
    ),
    request_body = ScaleBody,
    responses((status = 200, description = "The scaled lines and the calculation", body = Object,
        example = json!({"factor": 2.0, "batch_volume_l": 40.0, "boil_volume_l": 50.0,
            "fermentables": [], "hops": [], "yeasts": [], "miscs": [], "water_additions": [],
            "summary": null})))
)]
pub async fn recipe_version_scale(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    ApiJson(body): ApiJson<ScaleBody>,
) -> ApiResult<Json<Value>> {
    ctx.require(READ_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    let v = version_json(&mut tx, id).await?;
    let base = num(&v, "batch_volume_l");
    let factor = match (body.factor, body.batch_volume_l) {
        (Some(f), None) => f,
        (None, Some(volume)) if base > 0.0 => volume / base,
        _ => return Err(issue("/factor", "exactly_one_of_factor_or_batch_volume_l")),
    };
    if !factor.is_finite() || factor <= 0.0 || factor > 1000.0 {
        return Err(issue("/factor", "out_of_range"));
    }
    let mut scaled = v.clone();
    scaled["batch_volume_l"] = json!(base * factor);
    if let Some(b) = opt_num(&v, "boil_volume_l") {
        scaled["boil_volume_l"] = json!(b * factor);
    }
    scaled["fermentables"] = scale_lines(&v, "fermentables", &["amount_kg"], factor);
    scaled["hops"] = scale_lines(&v, "hops", &["amount_kg"], factor);
    scaled["yeasts"] = scale_lines(
        &v,
        "yeasts",
        &[
            "amount_kg",
            "total_cells_billion",
            "viable_cells_billion",
            "packs",
        ],
        factor,
    );
    scaled["miscs"] = scale_lines(&v, "miscs", &["amount"], factor);
    scaled["water_additions"] = scale_lines(&v, "water_additions", &["amount_kg"], factor);
    let summary = if body.calculate.unwrap_or(true) {
        Some(
            calculate_json(&state, &mut tx, &ctx, ws, &scaled, CalcTarget::None)
                .await?
                .0,
        )
    } else {
        None
    };
    tx.commit().await?;
    Ok(Json(json!({
        "factor": factor,
        "batch_volume_l": scaled["batch_volume_l"],
        "boil_volume_l": scaled["boil_volume_l"],
        "fermentables": scaled["fermentables"],
        "hops": scaled["hops"],
        "yeasts": scaled["yeasts"],
        "miscs": scaled["miscs"],
        "water_additions": scaled["water_additions"],
        "summary": summary,
    })))
}

fn field_diff(a: &Value, b: &Value, keys: &[&str]) -> Vec<Value> {
    keys.iter()
        .filter(|k| a.get(**k) != b.get(**k))
        .map(|k| json!({"field": k, "a": a.get(*k), "b": b.get(*k)}))
        .collect()
}

fn line_key(section: &str, l: &Value) -> String {
    if section == "water_additions" {
        format!("{}/{}", text(l, "agent"), text(l, "stage"))
    } else {
        text(l, "name").to_owned()
    }
}

fn keyed<'a>(section: &str, rows: &'a [Value]) -> Vec<(String, &'a Value)> {
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    rows.iter()
        .map(|l| {
            let base = line_key(section, l);
            let n = seen.entry(base.clone()).or_insert(0);
            *n += 1;
            (format!("{base}#{n}"), l)
        })
        .collect()
}

fn diff_section(section: &str, a: &Value, b: &Value) -> Value {
    let ka = keyed(section, lines(a, section));
    let kb = keyed(section, lines(b, section));
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();
    for (key, la) in &ka {
        match kb.iter().find(|(k, _)| k == key) {
            None => removed.push((*la).clone()),
            Some((_, lb)) => {
                let mut names: Vec<&String> = la
                    .as_object()
                    .into_iter()
                    .chain(lb.as_object())
                    .flat_map(Map::keys)
                    .filter(|k| !matches!(k.as_str(), "id" | "position"))
                    .collect();
                names.sort();
                names.dedup();
                let fields: Vec<Value> = names
                    .into_iter()
                    .filter(|k| la.get(k.as_str()) != lb.get(k.as_str()))
                    .map(|k| json!({"field": k, "a": la.get(k.as_str()), "b": lb.get(k.as_str())}))
                    .collect();
                if !fields.is_empty() {
                    changed.push(json!({"key": key, "fields": fields}));
                }
            }
        }
    }
    for (key, lb) in &kb {
        if !ka.iter().any(|(k, _)| k == key) {
            added.push((*lb).clone());
        }
    }
    json!({"added": added, "removed": removed, "changed": changed})
}

/// Compares two versions (of the same recipe or of two recipes): header fields, lines by name,
/// and the calculation summaries.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/recipe-versions/{id}/compare/{other_id}",
    tag = "recipes",
    operation_id = "recipe_version_compare",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "First recipe version (`a`)"),
        ("other_id" = Uuid, Path, description = "Second recipe version (`b`)")
    ),
    responses((status = 200, description = "The differences", body = Object,
        example = json!({"a": {"id": "0192f1c07a3b7000800000000000000c", "recipe_id": "0192f1c07a3b7000800000000000000b", "number": 1},
            "b": {"id": "0192f1c07a3b7000800000000000000d", "recipe_id": "0192f1c07a3b7000800000000000000b", "number": 2},
            "fields": [], "lines": {"fermentables": {"added": [], "removed": [], "changed": []}},
            "summary": []})))
)]
pub async fn recipe_version_compare(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id, other)): Path<(Uuid, Uuid, Uuid)>,
) -> ApiResult<Json<Value>> {
    ctx.require(READ_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let a = version_json(&mut tx, id).await?;
    let b = version_json(&mut tx, other).await?;
    let brief = |v: &Value| json!({"id": v["id"], "recipe_id": v["recipe_id"], "number": v["number"], "status": v["status"]});
    let mut lines_diff = Map::new();
    for (key, _, _) in SECTIONS {
        lines_diff.insert((*key).to_owned(), diff_section(key, &a, &b));
    }
    let summary_keys = [
        "og_sg",
        "fg_sg",
        "abv_pct",
        "ibu",
        "color_srm",
        "est_cells_ml",
        "honey_pct",
    ];
    let sa = a.get("calc_summary").cloned().unwrap_or(Value::Null);
    let sb = b.get("calc_summary").cloned().unwrap_or(Value::Null);
    Ok(Json(json!({
        "a": brief(&a),
        "b": brief(&b),
        "fields": field_diff(&a, &b, HEADER_KEYS),
        "lines": lines_diff,
        "summary": field_diff(&sa, &sb, &summary_keys),
    })))
}

// --- routes -------------------------------------------------------------------------------

/// The recipe routes, wrapped with the sign-in and idempotency layers.
pub fn routes(state: &AppState) -> Router<AppState> {
    const WS: &str = "/v1/workspaces/{ws}";
    let creates = crate::idempotent(
        state,
        Router::new()
            .route(&format!("{WS}/recipes"), axum::routing::post(recipe_create))
            .route(
                &format!("{WS}/recipes/{{id}}/versions"),
                axum::routing::post(recipe_version_create),
            )
            .route(
                &format!("{WS}/recipe-versions/{{id}}/publish"),
                axum::routing::post(recipe_version_publish),
            ),
    );
    let others = Router::new()
        .route(&format!("{WS}/recipes"), get(recipe_list))
        .route(
            &format!("{WS}/recipes/{{id}}"),
            get(recipe_get).patch(recipe_update).delete(recipe_delete),
        )
        .route(
            &format!("{WS}/recipes/{{id}}/versions"),
            get(recipe_version_list),
        )
        .route(
            &format!("{WS}/recipe-versions/{{id}}"),
            get(recipe_version_get).patch(recipe_version_update),
        )
        .route(
            &format!("{WS}/recipe-versions/{{id}}/calculate"),
            axum::routing::post(recipe_version_calculate),
        )
        .route(
            &format!("{WS}/recipe-versions/{{id}}/scale"),
            axum::routing::post(recipe_version_scale),
        )
        .route(
            &format!("{WS}/recipe-versions/{{id}}/compare/{{other_id}}"),
            get(recipe_version_compare),
        );
    crate::authenticated(state, creates.merge(others))
}

// --- registry entry ----------------------------------------------------------------------

/// Operation specs of this module (rows for `crate::openapi`).
mod spec {
    use crate::concurrency::Concurrency;
    use crate::error::ErrorCode;
    use crate::error::ErrorCode::{
        CalculationFailed, Conflict, Forbidden, IdempotencyInProgress, IdempotencyMismatch,
        Internal, MalformedRequest, NotFound, PayloadTooLarge, PreconditionRequired, RateLimited,
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
    const PUBLISH: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        Conflict,
        IdempotencyInProgress,
        VersionMismatch,
        PayloadTooLarge,
        UnsupportedMediaType,
        ValidationFailed,
        IdempotencyMismatch,
        CalculationFailed,
        PreconditionRequired,
        RateLimited,
        UpgradeRequired,
        Internal,
        Unavailable,
    ];
    const CALC: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        PayloadTooLarge,
        UnsupportedMediaType,
        ValidationFailed,
        CalculationFailed,
        RateLimited,
        UpgradeRequired,
        Internal,
        Unavailable,
    ];

    pub(super) const OPERATIONS: &[OperationSpec] = &[
        file_op("recipe_list", Some("recipe.view"), None, LIST),
        file_op(
            "recipe_create",
            Some("recipe.edit"),
            Some(Concurrency::Append),
            CREATE,
        ),
        file_op("recipe_get", Some("recipe.view"), None, READ),
        file_op(
            "recipe_update",
            Some("recipe.edit"),
            Some(Concurrency::Optimistic),
            UPDATE,
        ),
        file_op(
            "recipe_delete",
            Some("recipe.edit"),
            Some(Concurrency::Optimistic),
            UPDATE,
        ),
        file_op("recipe_version_list", Some("recipe.view"), None, READ),
        file_op(
            "recipe_version_create",
            Some("recipe.edit"),
            Some(Concurrency::Append),
            CREATE,
        ),
        file_op("recipe_version_get", Some("recipe.view"), None, READ),
        file_op(
            "recipe_version_update",
            Some("recipe.edit"),
            Some(Concurrency::Optimistic),
            UPDATE,
        ),
        file_op("recipe_version_calculate", Some("recipe.view"), None, CALC),
        file_op(
            "recipe_version_publish",
            Some("recipe.edit"),
            Some(Concurrency::Optimistic),
            PUBLISH,
        ),
        file_op("recipe_version_scale", Some("recipe.view"), None, CALC),
        file_op("recipe_version_compare", Some("recipe.view"), None, READ),
    ];
}

/// `OpenAPI` fragment of this module; `crate::openapi` merges it.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(
        recipe_list,
        recipe_create,
        recipe_get,
        recipe_update,
        recipe_delete,
        recipe_version_list,
        recipe_version_create,
        recipe_version_get,
        recipe_version_update,
        recipe_version_calculate,
        recipe_version_publish,
        recipe_version_scale,
        recipe_version_compare
    ),
    components(schemas(
        Recipe,
        NewRecipe,
        RecipePatch,
        VersionInput,
        FermentableLine,
        HopLine,
        YeastLine,
        MiscLine,
        WaterAdditionLine,
        NewDraft,
        PublishBody,
        ScaleBody
    ))
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "recipes",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
