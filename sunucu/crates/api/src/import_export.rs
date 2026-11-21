//! Import and export of recipes and data (T2.26, plan "İçe aktarma" and "Dışa aktarma").
//!
//! Import has two steps. `POST /imports` takes the file as the request body (`BeerXML`,
//! `BeerJSON` or a Brewfather JSON backup), parses it with the limits of the `interchange`
//! crate (no DOCTYPE, no entities, size, depth and element limits) and stores the parsed
//! document in `import_sources`. The answer lists what the file holds, with the values the file
//! states (OG, IBU, color). `POST /imports/{id}/apply` creates the recipes (a draft each),
//! creates or matches equipment, matches ingredients by name, calculates each recipe with the
//! engine and returns the source values and the calculated values side by side. Every mapping
//! decision goes to `import_sources.mapping_decisions`.
//!
//! Export is a worker job. `POST /exports` queues a row in `export_jobs`; the worker writes the
//! file to the object store (see `worker::export`). `GET /exports/{id}` gives the status and,
//! when the file is ready, a presigned download link with its expiry time. A user starts one
//! full export (scope `workspace`) in 24 hours. Recipe and batch exports have no limit.
//!
//! TODO: `.bsmx` (`BeerSmith`) import needs a sample file (T2.26).

#![allow(clippy::too_many_lines)]

use std::{collections::HashMap, time::Duration};

use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use interchange::{Document, Recipe};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{FromRow, PgConnection};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    context::RequestContext,
    engine::CalcTarget,
    error::{ApiError, ApiJson, ApiQuery, ApiResult, ErrorCode, FieldIssue},
    library::{issue, not_found, workspace},
    recipe_org::{conflict, optional_body, recipe_event},
    recipes::{NewRecipe, calculate_json, insert_recipe, version_json},
};

/// Permission that imports recipes.
pub const IMPORT_PERMISSION: &str = "recipe.edit";
/// Permission that starts and reads exports.
pub const EXPORT_PERMISSION: &str = "reports.export";
/// Largest import file, bytes.
pub const MAX_IMPORT_BYTES: usize = 8 * 1024 * 1024;
/// How long a finished export stays in the object store.
pub const EXPORT_KEEP: Duration = Duration::from_hours(7 * 24);
/// Lifetime of one presigned download link.
const LINK_TTL: Duration = Duration::from_mins(30);
/// A user starts one full export in this time.
const FULL_EXPORT_INTERVAL_HOURS: i64 = 24;

// --- import -------------------------------------------------------------------------------

/// Query of the import upload.
#[derive(Debug, Default, Deserialize)]
pub struct ImportQuery {
    /// `beerxml`, `beerjson` or `brewfather`. Missing: detected from the content.
    pub format: Option<String>,
    /// Name of the uploaded file (shown later).
    pub file_name: Option<String>,
}

/// Values a file states about a recipe.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SourceFigures {
    /// Original gravity, SG.
    pub og_sg: Option<f64>,
    /// Final gravity, SG.
    pub fg_sg: Option<f64>,
    /// Bitterness, IBU.
    pub ibu: Option<f64>,
    /// Color, SRM.
    pub srm: Option<f64>,
    /// Alcohol, percent.
    pub abv_pct: Option<f64>,
}

/// One recipe in an uploaded file.
#[derive(Debug, Serialize, ToSchema)]
pub struct ImportItem {
    /// Position in the file; use it in the apply body.
    pub index: usize,
    /// Name.
    pub name: String,
    /// Kind (`all_grain`, `extract`, ...).
    pub kind: String,
    /// Style name in the file.
    pub style: Option<String>,
    /// What the file states.
    pub source: SourceFigures,
    /// Name of the equipment in the file.
    pub equipment: Option<String>,
    /// Number of fermentable, hop, yeast and misc lines.
    pub lines: [usize; 4],
    /// Things the parser changed or could not map.
    pub warnings: Vec<String>,
}

/// An uploaded import file.
#[derive(Debug, Serialize, ToSchema)]
pub struct ImportPreview {
    /// Identifier.
    pub id: Uuid,
    /// `beerxml`, `beerjson` or `brewfather`.
    pub format: String,
    /// `parsed` or `applied`.
    pub status: String,
    /// Name of the uploaded file.
    pub file_name: Option<String>,
    /// The recipes of the file.
    pub recipes: Vec<ImportItem>,
    /// Names of the equipment profiles of the file (recipes and stand-alone).
    pub equipment: Vec<String>,
    /// Warnings of the whole file.
    pub warnings: Vec<String>,
    /// The mapping decisions (after the apply).
    #[schema(value_type = Option<Object>)]
    pub mapping_decisions: Option<Value>,
    /// The first recipe the apply created.
    pub recipe_id: Option<Uuid>,
}

#[derive(FromRow)]
struct ImportRow {
    id: Uuid,
    format: String,
    status: String,
    file_name: Option<String>,
    parsed: Option<Value>,
    mapping_decisions: Option<Value>,
    recipe_id: Option<Uuid>,
}

fn figures(r: &Recipe) -> SourceFigures {
    SourceFigures {
        og_sg: r.source.og_sg,
        fg_sg: r.source.fg_sg,
        ibu: r.source.ibu,
        srm: r.source.srm,
        abv_pct: r.source.abv_pct,
    }
}

fn preview(row: ImportRow) -> ApiResult<ImportPreview> {
    let doc: Document = row
        .parsed
        .map(serde_json::from_value)
        .transpose()
        .map_err(|_| ApiError::new(ErrorCode::Internal))?
        .unwrap_or_default();
    let mut equipment: Vec<String> = doc
        .recipes
        .iter()
        .filter_map(|r| r.equipment.as_ref().map(|e| e.name.clone()))
        .chain(doc.equipment.iter().map(|e| e.name.clone()))
        .collect();
    equipment.dedup();
    Ok(ImportPreview {
        id: row.id,
        format: row.format,
        status: row.status,
        file_name: row.file_name,
        recipes: doc
            .recipes
            .iter()
            .enumerate()
            .map(|(index, r)| ImportItem {
                index,
                name: r.name.clone(),
                kind: r.kind.clone(),
                style: r.style.clone(),
                source: figures(r),
                equipment: r.equipment.as_ref().map(|e| e.name.clone()),
                lines: [
                    r.fermentables.len(),
                    r.hops.len(),
                    r.yeasts.len(),
                    r.miscs.len(),
                ],
                warnings: r.warnings.clone(),
            })
            .collect(),
        equipment,
        warnings: doc.warnings,
        mapping_decisions: row.mapping_decisions,
        recipe_id: row.recipe_id,
    })
}

fn detect(body: &[u8]) -> &'static str {
    match body.iter().find(|b| !b.is_ascii_whitespace()) {
        Some(b'<') => "beerxml",
        _ if body.windows(8).take(4096).any(|w| w == b"beerjson") => "beerjson",
        _ => "brewfather",
    }
}

fn parse_error(e: &interchange::Error) -> ApiError {
    use interchange::Error as E;
    match e {
        E::TooLarge => ApiError::new(ErrorCode::PayloadTooLarge),
        E::DoctypeNotAllowed => issue("/body", "xml_doctype_not_allowed"),
        E::EntityNotAllowed => issue("/body", "xml_entity_not_allowed"),
        E::TooComplex => issue("/body", "file_too_complex"),
        E::Xml(_) | E::Json(_) => issue("/body", "invalid_file"),
        E::Format(_) => issue("/body", "unknown_format"),
    }
}

const SELECT_IMPORT: &str = "SELECT id, format, status, file_name, parsed, mapping_decisions, \
    recipe_id FROM import_sources WHERE id = $1 AND deleted_at IS NULL";

/// Uploads a recipe file and parses it (nothing is created yet).
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/imports",
    tag = "import-export",
    operation_id = "import_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("format" = Option<String>, Query, description = "`beerxml`, `beerjson` or `brewfather` (default: detected)"),
        ("file_name" = Option<String>, Query, description = "Name of the file")
    ),
    request_body(content = String, content_type = "application/octet-stream",
        description = "The file (up to 8 MiB)"),
    responses((status = 201, description = "The parsed file", body = ImportPreview))
)]
pub async fn import_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(q): ApiQuery<ImportQuery>,
    body: Bytes,
) -> ApiResult<Response> {
    ctx.require(IMPORT_PERMISSION)?;
    let ws = workspace(&ctx)?;
    if body.is_empty() {
        return Err(issue("/body", "required"));
    }
    let format = match q.format.as_deref() {
        None => detect(&body),
        Some("beerxml") => "beerxml",
        Some("beerjson") => "beerjson",
        Some("brewfather") => "brewfather",
        Some(_) => return Err(issue("/format", "invalid_value")),
    };
    let parsed = match format {
        "beerxml" => interchange::beerxml::parse(&body),
        "beerjson" => interchange::beerjson::parse(&body),
        _ => interchange::brewfather::parse(&body),
    }
    .map_err(|e| parse_error(&e))?;
    let file_name = q.file_name.map(|n| {
        n.chars()
            .filter(|c| !c.is_control())
            .take(255)
            .collect::<String>()
    });
    let id = Uuid::now_v7();
    // The original file is kept as an attachment of the recipes (plan: "kaynak dosya saklanır").
    let key = format!("{ws}/imports/{id}");
    if let Some(store) = &state.store {
        let content_type = if format == "beerxml" {
            "application/xml"
        } else {
            "application/json"
        };
        if let Err(e) = store.put(&key, content_type, body.to_vec()).await {
            tracing::warn!(error = %e, "import source file was not stored");
        }
    }
    let mut tx = ctx.begin(&state.db).await?;
    sqlx::query(
        "INSERT INTO import_sources (id, workspace_id, object_key, format, status, parsed, \
         file_name, created_by, updated_by) VALUES ($1, $2, $3, $4, 'parsed', $5, $6, $7, $7)",
    )
    .bind(id)
    .bind(ws)
    .bind(&key)
    .bind(format)
    .bind(serde_json::to_value(&parsed).map_err(|_| ApiError::new(ErrorCode::Internal))?)
    .bind(&file_name)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await?;
    let row: ImportRow = sqlx::query_as(SELECT_IMPORT)
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    let out = preview(row)?;
    Ok((StatusCode::CREATED, Json(out)).into_response())
}

/// Reads an uploaded import file.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/imports/{id}",
    tag = "import-export",
    operation_id = "import_get",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Import")
    ),
    responses((status = 200, description = "The import", body = ImportPreview))
)]
pub async fn import_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<ImportPreview>> {
    ctx.require(IMPORT_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let row: ImportRow = sqlx::query_as(SELECT_IMPORT)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(not_found)?;
    Ok(Json(preview(row)?))
}

/// What to create from one recipe of the file.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ApplyRecipe {
    /// Index in the file (from the preview).
    pub index: usize,
    /// Use this equipment (default: match the equipment of the file by name, else create it).
    pub equipment_id: Option<Uuid>,
    /// Create the equipment of the file when no match exists (default true).
    pub create_equipment: Option<bool>,
    /// Name for the new recipe (default: the name in the file).
    pub name: Option<String>,
}

/// Body of the apply.
#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ApplyBody {
    /// The recipes to create (default: all).
    pub recipes: Option<Vec<ApplyRecipe>>,
}

/// Values the engine calculated.
#[derive(Debug, Serialize, ToSchema)]
pub struct CalculatedFigures {
    /// Original gravity, SG.
    pub og_sg: Option<f64>,
    /// Final gravity, SG.
    pub fg_sg: Option<f64>,
    /// Bitterness, IBU.
    pub ibu: Option<f64>,
    /// Color, SRM.
    pub color_srm: Option<f64>,
    /// Alcohol, percent.
    pub abv_pct: Option<f64>,
}

/// A recipe the apply created.
#[derive(Debug, Serialize, ToSchema)]
pub struct AppliedRecipe {
    /// Index in the file.
    pub index: usize,
    /// The new recipe.
    pub recipe_id: Uuid,
    /// Its first draft.
    pub version_id: Uuid,
    /// Name.
    pub name: String,
    /// What the file states.
    pub source: SourceFigures,
    /// What the server calculated; `null` when the calculation is not available.
    pub calculated: Option<CalculatedFigures>,
    /// Equipment of the recipe.
    pub equipment_id: Option<Uuid>,
}

/// Result of the apply.
#[derive(Debug, Serialize, ToSchema)]
pub struct ApplyResult {
    /// The import.
    pub import_id: Uuid,
    /// The created recipes.
    pub recipes: Vec<AppliedRecipe>,
    /// Equipment created from stand-alone profiles of the file.
    pub equipment_ids: Vec<Uuid>,
    /// The decisions that were saved in the import.
    #[schema(value_type = Object)]
    pub mapping_decisions: Value,
}

async fn find_equipment(tx: &mut PgConnection, name: &str) -> ApiResult<Option<Uuid>> {
    Ok(sqlx::query_scalar(
        "SELECT id FROM equipment WHERE lower(name) = lower($1) AND deleted_at IS NULL \
         ORDER BY created_at LIMIT 1",
    )
    .bind(name)
    .fetch_optional(tx)
    .await?)
}

async fn create_equipment(
    tx: &mut PgConnection,
    ws: Uuid,
    user: Uuid,
    e: &interchange::Equipment,
    fallback_batch: f64,
    fallback_eff: f64,
) -> ApiResult<Uuid> {
    let id = Uuid::now_v7();
    let pos = |v: Option<f64>| v.filter(|x| *x > 0.0);
    let non_neg = |v: Option<f64>| v.filter(|x| *x >= 0.0).unwrap_or(0.0);
    sqlx::query(
        "INSERT INTO equipment (id, workspace_id, name, batch_volume_l, boil_volume_l, \
         boil_time_min, boil_off_l_per_h, trub_loss_l, fermenter_loss_l, efficiency_pct, \
         created_by, updated_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $11)",
    )
    .bind(id)
    .bind(ws)
    .bind(e.name.chars().take(200).collect::<String>())
    .bind(pos(e.batch_volume_l).unwrap_or(fallback_batch))
    .bind(pos(e.boil_volume_l))
    .bind(non_neg(e.boil_time_min.or(Some(60.0))))
    .bind(non_neg(e.boil_off_l_per_h))
    .bind(non_neg(e.trub_loss_l))
    .bind(non_neg(e.fermenter_loss_l))
    .bind(e.efficiency_pct.unwrap_or(fallback_eff).clamp(0.0, 100.0))
    .bind(user)
    .execute(tx)
    .await
    .map_err(|e| {
        crate::library::sqlstate(&e)
            .map_or_else(|| e.into(), |_| issue("/equipment", "constraint_violated"))
    })?;
    Ok(id)
}

/// Looks up the library ingredient with this name. A unique match gives its id.
async fn match_ingredient(
    tx: &mut PgConnection,
    kind: &str,
    name: &str,
) -> ApiResult<(Option<Uuid>, &'static str)> {
    let ids: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM ingredients WHERE kind = $1 AND lower(name) = lower($2) \
         AND deleted_at IS NULL ORDER BY workspace_id IS NULL, created_at LIMIT 2",
    )
    .bind(kind)
    .bind(name)
    .fetch_all(tx)
    .await?;
    Ok(match ids.as_slice() {
        [one] => (Some(*one), "matched"),
        // The first has the workspace's own record before the shared one.
        [first, _] => (Some(*first), "matched_first_of_many"),
        _ => (None, "kept_file_values"),
    })
}

fn calc_figures(summary: &Value) -> CalculatedFigures {
    let f = |k: &str| summary.get(k).and_then(Value::as_f64);
    CalculatedFigures {
        og_sg: f("og_sg"),
        fg_sg: f("fg_sg"),
        ibu: f("ibu"),
        color_srm: f("color_srm"),
        abv_pct: f("abv_pct"),
    }
}

/// Creates the recipes of an uploaded file, with the mapping and the calculated values.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/imports/{id}/apply",
    tag = "import-export",
    operation_id = "import_apply",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Import")
    ),
    request_body = ApplyBody,
    responses((status = 201, description = "Recipes created", body = ApplyResult))
)]
pub async fn import_apply(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    body: Bytes,
) -> ApiResult<Response> {
    ctx.require(IMPORT_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let user = ctx.identity.user_id;
    let body: ApplyBody = optional_body(&body)?;
    let mut tx = ctx.begin(&state.db).await?;
    let row: ImportRow = sqlx::query_as(sqlx::AssertSqlSafe(format!("{SELECT_IMPORT} FOR UPDATE")))
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(not_found)?;
    if row.status == "applied" {
        return Err(conflict("/", "already_applied"));
    }
    let doc: Document = serde_json::from_value(row.parsed.clone().unwrap_or(Value::Null))
        .map_err(|_| ApiError::new(ErrorCode::Internal))?;
    let picks: Vec<ApplyRecipe> = body.recipes.unwrap_or_else(|| {
        (0..doc.recipes.len())
            .map(|index| ApplyRecipe {
                index,
                equipment_id: None,
                create_equipment: None,
                name: None,
            })
            .collect()
    });
    if picks.len() > 100 {
        return Err(issue("/recipes", "too_many"));
    }
    let mut equipment_by_name: HashMap<String, Uuid> = HashMap::new();
    let mut decisions = Vec::new();
    let mut applied = Vec::new();
    let mut first_recipe = None;
    for (n, pick) in picks.iter().enumerate() {
        let mut recipe = doc
            .recipes
            .get(pick.index)
            .cloned()
            .ok_or_else(|| issue(&format!("/recipes/{n}/index"), "not_found"))?;
        let defaults = recipe.fill_defaults();
        // Equipment: the user's choice, a match by name, or a new record.
        let (equipment_id, equipment_how) = match (&pick.equipment_id, &recipe.equipment) {
            (Some(eid), _) => (Some(*eid), "chosen_by_user"),
            (None, Some(e)) if pick.create_equipment != Some(false) => {
                let key = e.name.to_lowercase();
                if let Some(existing) = equipment_by_name.get(&key) {
                    (Some(*existing), "reused_in_import")
                } else if let Some(existing) = find_equipment(&mut tx, &e.name).await? {
                    equipment_by_name.insert(key, existing);
                    (Some(existing), "matched_by_name")
                } else {
                    let created = create_equipment(
                        &mut tx,
                        ws,
                        user,
                        e,
                        recipe.batch_volume_l.unwrap_or(20.0),
                        recipe.efficiency_pct.unwrap_or(70.0),
                    )
                    .await?;
                    equipment_by_name.insert(key, created);
                    (Some(created), "created")
                }
            }
            _ => (None, "none"),
        };
        let eid_text = equipment_id.map(|e| e.to_string());
        let mut version = recipe.version_input(eid_text.as_deref());
        // Ingredient matching by name; a line with no match keeps the values of the file.
        let mut matches = Vec::new();
        for (section, kind) in [
            ("fermentables", "fermentable"),
            ("hops", "hop"),
            ("yeasts", "yeast"),
            ("miscs", "misc"),
        ] {
            let Some(lines) = version.get_mut(section).and_then(Value::as_array_mut) else {
                continue;
            };
            for (i, line) in lines.iter_mut().enumerate() {
                let name = line
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned();
                let (found, how) = match_ingredient(&mut tx, kind, &name).await?;
                if let (Some(found), Some(o)) = (found, line.as_object_mut()) {
                    o.insert("ingredient_id".into(), json!(found));
                }
                matches.push(json!({"section": section, "index": i, "name": name,
                    "ingredient_id": found, "decision": how}));
            }
        }
        let name = pick.name.clone().unwrap_or_else(|| recipe.name.clone());
        let name = name.trim().chars().take(200).collect::<String>();
        if name.is_empty() {
            return Err(issue(&format!("/recipes/{n}/name"), "invalid_value"));
        }
        let new: NewRecipe = serde_json::from_value(json!({
            "name": name, "notes": recipe.notes, "version": version,
        }))
        .map_err(|_| issue(&format!("/recipes/{n}"), "invalid_value"))?;
        let (recipe_id, version_id) = insert_recipe(&mut tx, ws, user, &new, &name).await?;
        recipe_event(&mut tx, ws, "recipe.created", recipe_id, 1, None).await?;
        first_recipe.get_or_insert(recipe_id);
        // The calculation must not break the import: a failure rolls back to the savepoint.
        sqlx::query("SAVEPOINT import_calc")
            .execute(&mut *tx)
            .await?;
        let v = version_json(&mut tx, version_id).await?;
        let calculated = match calculate_json(&state, &mut tx, &ctx, ws, &v, CalcTarget::None).await
        {
            Ok((summary, _)) => {
                sqlx::query("RELEASE SAVEPOINT import_calc")
                    .execute(&mut *tx)
                    .await?;
                Some(calc_figures(&summary))
            }
            Err(e) => {
                tracing::warn!(error = ?e.problem().status, "import calculation failed");
                sqlx::query("ROLLBACK TO SAVEPOINT import_calc")
                    .execute(&mut *tx)
                    .await?;
                None
            }
        };
        decisions.push(json!({
            "index": pick.index,
            "recipe_id": recipe_id,
            "equipment": {"id": equipment_id, "decision": equipment_how},
            "ingredients": matches,
            "defaults_applied": defaults,
            "warnings": recipe.warnings,
            "kept_source_values": {
                "og_sg": recipe.source.og_sg, "fg_sg": recipe.source.fg_sg,
                "ibu": recipe.source.ibu, "srm": recipe.source.srm,
                "abv_pct": recipe.source.abv_pct, "ibu_method": recipe.ibu_method,
                "style": recipe.style,
            },
            "note": "the source values became the targets of the draft",
        }));
        applied.push(AppliedRecipe {
            index: pick.index,
            recipe_id,
            version_id,
            name,
            source: figures(&recipe),
            calculated,
            equipment_id,
        });
    }
    // Stand-alone equipment profiles of the file (for example a Brewfather equipment list).
    let mut equipment_ids = Vec::new();
    let mut equipment_decisions = Vec::new();
    for e in &doc.equipment {
        let key = e.name.to_lowercase();
        if equipment_by_name.contains_key(&key) {
            continue;
        }
        if let Some(existing) = find_equipment(&mut tx, &e.name).await? {
            equipment_decisions
                .push(json!({"name": e.name, "id": existing, "decision": "matched_by_name"}));
            continue;
        }
        let created = create_equipment(&mut tx, ws, user, e, 20.0, 70.0).await?;
        equipment_ids.push(created);
        equipment_decisions.push(json!({"name": e.name, "id": created, "decision": "created"}));
    }
    let mapping = json!({
        "schema_version": 1,
        "format": row.format,
        "recipes": decisions,
        "equipment": equipment_decisions,
    });
    sqlx::query(
        "UPDATE import_sources SET status = 'applied', mapping_decisions = $2, recipe_id = $3, \
         updated_by = $4 WHERE id = $1",
    )
    .bind(id)
    .bind(&mapping)
    .bind(first_recipe)
    .bind(user)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    let result = ApplyResult {
        import_id: id,
        recipes: applied,
        equipment_ids,
        mapping_decisions: mapping,
    };
    Ok((StatusCode::CREATED, Json(result)).into_response())
}

// --- export -------------------------------------------------------------------------------

/// Body of an export request.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct NewExport {
    /// `recipe`, `batch` or `workspace` (the full export).
    pub scope: String,
    /// The recipe or batch (required for those scopes).
    pub scope_id: Option<Uuid>,
    /// Recipe: `beerxml`, `beerjson`, `json`. Batch: `csv`, `json`, `zip`. Workspace: `zip`.
    pub format: String,
    /// Batch only: first measurement time.
    pub from: Option<DateTime<Utc>>,
    /// Batch only: last measurement time.
    pub to: Option<DateTime<Utc>>,
}

/// A download link.
#[derive(Debug, Serialize, ToSchema)]
pub struct DownloadLink {
    /// Presigned `GET` URL.
    pub url: String,
    /// When this link stops working.
    pub expires_at: DateTime<Utc>,
}

/// An export job.
#[derive(Debug, Serialize, ToSchema)]
pub struct ExportJob {
    /// Identifier.
    pub id: Uuid,
    /// `recipe`, `batch` or `workspace`.
    pub scope: String,
    /// The recipe or batch.
    pub scope_id: Option<Uuid>,
    /// File format.
    pub format: String,
    /// `queued`, `running`, `done`, `failed` or `expired`.
    pub status: String,
    /// Error text of a failed job.
    pub error: Option<String>,
    /// Size of the finished file, bytes.
    pub size_bytes: Option<i64>,
    /// When the finished file is removed.
    pub expires_at: Option<DateTime<Utc>>,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// The download link, when the file is ready.
    pub download: Option<DownloadLink>,
}

#[derive(FromRow)]
struct ExportRow {
    id: Uuid,
    scope: String,
    scope_id: Option<Uuid>,
    format: String,
    status: String,
    error: Option<String>,
    size_bytes: Option<i64>,
    expires_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    result_object_key: Option<String>,
}

const SELECT_EXPORT: &str = "SELECT id, scope, scope_id, format, status, error, size_bytes, \
    expires_at, created_at, result_object_key FROM export_jobs WHERE id = $1 AND deleted_at IS NULL";

fn export_job(row: ExportRow, state: &AppState) -> ApiResult<ExportJob> {
    let now = Utc::now();
    let mut status = row.status.clone();
    if status == "done" && row.expires_at.is_some_and(|t| t <= now) {
        "expired".clone_into(&mut status);
    }
    let download = match (
        status.as_str(),
        &row.result_object_key,
        &state.store,
        row.expires_at,
    ) {
        ("done", Some(key), Some(store), Some(expires)) => {
            let remaining = (expires - now).to_std().unwrap_or(Duration::ZERO);
            let ttl = remaining.min(LINK_TTL);
            let link = store.presign_get(key, ttl).map_err(|e| {
                tracing::error!(error = %e, "presign of an export failed");
                ApiError::new(ErrorCode::Internal)
            })?;
            Some(DownloadLink {
                url: link.url,
                expires_at: link.expires_at,
            })
        }
        _ => None,
    };
    Ok(ExportJob {
        id: row.id,
        scope: row.scope,
        scope_id: row.scope_id,
        format: row.format,
        status,
        error: row.error,
        size_bytes: row.size_bytes,
        expires_at: row.expires_at,
        created_at: row.created_at,
        download,
    })
}

/// Queues an export. The full export (`scope` `workspace`) runs once in 24 hours for a user.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/exports",
    tag = "import-export",
    operation_id = "export_create",
    params(("workspace_id" = Uuid, Path, description = "Workspace")),
    request_body = NewExport,
    responses((status = 202, description = "The queued job", body = ExportJob))
)]
pub async fn export_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(new): ApiJson<NewExport>,
) -> ApiResult<Response> {
    ctx.require(EXPORT_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let user = ctx.identity.user_id;
    let allowed: &[&str] = match new.scope.as_str() {
        "recipe" => &["beerxml", "beerjson", "json"],
        "batch" => &["csv", "json", "zip"],
        "workspace" => &["zip"],
        _ => return Err(issue("/scope", "invalid_value")),
    };
    if !allowed.contains(&new.format.as_str()) {
        return Err(issue("/format", "invalid_value"));
    }
    if new.scope == "workspace" {
        if new.scope_id.is_some() {
            return Err(issue("/scope_id", "not_allowed"));
        }
    } else if new.scope_id.is_none() {
        return Err(issue("/scope_id", "required"));
    }
    if new.scope != "batch" && (new.from.is_some() || new.to.is_some()) {
        return Err(issue("/from", "not_allowed"));
    }
    if let (Some(a), Some(b)) = (new.from, new.to)
        && a > b
    {
        return Err(issue("/from", "invalid_value"));
    }
    let mut tx = ctx.begin(&state.db).await?;
    if let Some(sid) = new.scope_id {
        let table = if new.scope == "recipe" {
            "recipes"
        } else {
            "batches"
        };
        let found: Option<Uuid> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT id FROM {table} WHERE id = $1 AND deleted_at IS NULL"
        )))
        .bind(sid)
        .fetch_optional(&mut *tx)
        .await?;
        if found.is_none() {
            return Err(issue("/scope_id", "not_found"));
        }
    }
    if new.scope == "workspace" {
        // Two requests of one user at the same time must not both pass.
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 26))")
            .bind(format!("full-export:{ws}:{user}"))
            .execute(&mut *tx)
            .await?;
        let last: Option<DateTime<Utc>> = sqlx::query_scalar(
            "SELECT max(created_at) FROM export_jobs WHERE scope = 'workspace' \
             AND created_by = $1 AND status <> 'failed' \
             AND created_at > now() - make_interval(hours => $2::int)",
        )
        .bind(user)
        .bind(i32::try_from(FULL_EXPORT_INTERVAL_HOURS).unwrap_or(24))
        .fetch_one(&mut *tx)
        .await?;
        if let Some(last) = last {
            let retry = (last + chrono::Duration::hours(FULL_EXPORT_INTERVAL_HOURS) - Utc::now())
                .num_seconds()
                .max(1);
            return Err(ApiError::new(ErrorCode::RateLimited)
                .with_errors(vec![FieldIssue::new("/scope", "full_export_once_per_day")])
                .with_retry_after(u64::try_from(retry).unwrap_or(1)));
        }
    }
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO export_jobs (id, workspace_id, scope, scope_id, format, params, created_by, \
         updated_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $7)",
    )
    .bind(id)
    .bind(ws)
    .bind(&new.scope)
    .bind(new.scope_id)
    .bind(&new.format)
    .bind(json!({"from": new.from, "to": new.to}))
    .bind(user)
    .execute(&mut *tx)
    .await?;
    let row: ExportRow = sqlx::query_as(SELECT_EXPORT)
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    let job = export_job(row, &state)?;
    let mut response = (StatusCode::ACCEPTED, Json(job)).into_response();
    if let Ok(v) = format!("/v1/workspaces/{ws}/exports/{id}").parse() {
        response
            .headers_mut()
            .insert(axum::http::header::LOCATION, v);
    }
    Ok(response)
}

/// Reads an export job. A finished job carries a download link with its expiry time.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/exports/{id}",
    tag = "import-export",
    operation_id = "export_get",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Export job")
    ),
    responses((status = 200, description = "The job", body = ExportJob))
)]
pub async fn export_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<ExportJob>> {
    ctx.require(EXPORT_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let row: ExportRow = sqlx::query_as(SELECT_EXPORT)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(not_found)?;
    Ok(Json(export_job(row, &state)?))
}

// --- routes -------------------------------------------------------------------------------

/// The routes, wrapped with the sign-in layer. The import upload has its own body limit.
pub fn routes(state: &AppState) -> Router<AppState> {
    const WS: &str = "/v1/workspaces/{ws}";
    let upload = Router::new()
        .route(&format!("{WS}/imports"), post(import_create))
        .layer(DefaultBodyLimit::max(MAX_IMPORT_BYTES));
    let others = Router::new()
        .route(&format!("{WS}/imports/{{id}}"), get(import_get))
        .route(&format!("{WS}/imports/{{id}}/apply"), post(import_apply))
        .route(&format!("{WS}/exports"), post(export_create))
        .route(&format!("{WS}/exports/{{id}}"), get(export_get));
    crate::authenticated(state, upload.merge(others))
}

// --- registry entry ----------------------------------------------------------------------

mod spec {
    use crate::concurrency::Concurrency;
    use crate::error::ErrorCode::{
        Conflict, Forbidden, Internal, MalformedRequest, NotFound, PayloadTooLarge, RateLimited,
        Unauthenticated, Unavailable, UnsupportedMediaType, UpgradeRequired, ValidationFailed,
    };
    use crate::openapi::{OperationSpec, file_op};

    const READ: &[crate::error::ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        RateLimited,
        UpgradeRequired,
        Internal,
        Unavailable,
    ];
    const WRITE: &[crate::error::ErrorCode] = &[
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

    pub(super) const OPERATIONS: &[OperationSpec] = &[
        file_op(
            "import_create",
            Some("recipe.edit"),
            Some(Concurrency::Append),
            WRITE,
        ),
        file_op("import_get", Some("recipe.edit"), None, READ),
        file_op(
            "import_apply",
            Some("recipe.edit"),
            Some(Concurrency::Append),
            WRITE,
        ),
        file_op(
            "export_create",
            Some("reports.export"),
            Some(Concurrency::Append),
            WRITE,
        ),
        file_op("export_get", Some("reports.export"), None, READ),
    ];
}

/// `OpenAPI` fragment of this module.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(import_create, import_get, import_apply, export_create, export_get),
    components(schemas(
        SourceFigures,
        ImportItem,
        ImportPreview,
        ApplyRecipe,
        ApplyBody,
        CalculatedFigures,
        AppliedRecipe,
        ApplyResult,
        NewExport,
        DownloadLink,
        ExportJob
    )),
    tags((name = "import-export", description = "Recipe import (BeerXML, BeerJSON, Brewfather) and data export jobs"))
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "import-export",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
