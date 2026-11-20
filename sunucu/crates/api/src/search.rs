//! Global search (T2.27, ADR-0007 section "Arama").
//!
//! One query over recipes, batches, ingredients and stock lots with the PostgreSQL full-text
//! columns (`search`, configuration `simple`). Only rows of the workspace come back: each branch
//! filters on `workspace_id`, and row level security applies on top. Library ingredients have no
//! workspace; every workspace sees them. A word matches as a prefix (`zyth` finds `Zythum`). The
//! whole text also runs as a web search query, so a code such as `B-0001` matches exactly.

use axum::{Json, Router, extract::State, routing::get};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    context::RequestContext,
    error::{ApiQuery, ApiResult},
    library::{issue, workspace},
};

/// Permission that reads the searched records.
pub const READ_PERMISSION: &str = "recipe.view";

const TYPES: &[&str] = &["recipe", "batch", "ingredient", "stock"];
const DEFAULT_LIMIT: u32 = 20;
const MAX_LIMIT: u32 = 50;
const MAX_TOKENS: usize = 8;

/// A search hit.
#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct SearchHit {
    /// `recipe`, `batch`, `ingredient` or `stock`.
    #[serde(rename = "type")]
    #[sqlx(rename = "kind")]
    pub kind: String,
    /// Identifier of the record.
    pub id: Uuid,
    /// Name of the record (for stock: the ingredient).
    pub title: String,
    /// Second line: batch code, ingredient kind, lot code.
    pub subtitle: Option<String>,
    /// Rank, higher is better.
    pub rank: f32,
}

/// The search answer.
#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResults {
    /// The hits, best first. No paging: use `limit`.
    pub items: Vec<SearchHit>,
}

/// Query of the search.
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    /// The text.
    pub q: Option<String>,
    /// Comma separated: `recipe`, `batch`, `ingredient`, `stock` (default all).
    pub types: Option<String>,
    /// Number of hits, 1..=50 (default 20).
    pub limit: Option<u32>,
}

/// The prefix query of the words in `text`: `ab:* & cd:*`. Empty when the text has no word.
fn prefix_query(text: &str) -> String {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .take(MAX_TOKENS)
        .map(|w| format!("{}:*", w.to_lowercase()))
        .collect::<Vec<_>>()
        .join(" & ")
}

/// Searches recipes, batches, ingredients and stock of the workspace.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/search",
    tag = "search",
    operation_id = "search",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("q" = String, Query, description = "Text, 1 to 200 characters"),
        ("types" = Option<String>, Query, description = "Comma separated: `recipe`, `batch`, `ingredient`, `stock` (default all)"),
        ("limit" = Option<u32>, Query, description = "Number of hits, 1..=50 (default 20)")
    ),
    responses((status = 200, description = "The hits, best first", body = SearchResults,
        example = json!({"items": [{"type": "recipe", "id": "0192f1c07a3b7000800000000000000b",
            "title": "House IPA", "subtitle": null, "rank": 0.06}]})))
)]
pub async fn search(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(query): ApiQuery<SearchQuery>,
) -> ApiResult<Json<SearchResults>> {
    ctx.require(READ_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let text = query.q.as_deref().map(str::trim).unwrap_or_default();
    if text.is_empty() {
        return Err(issue("q", "required"));
    }
    if text.chars().count() > 200 {
        return Err(issue("q", "invalid_length"));
    }
    let types: Vec<String> = match query.types.as_deref() {
        None | Some("") => TYPES.iter().map(|t| (*t).to_owned()).collect(),
        Some(list) => {
            let mut out = Vec::new();
            for t in list.split(',').map(str::trim) {
                if !TYPES.contains(&t) {
                    return Err(issue("types", "invalid_value"));
                }
                out.push(t.to_owned());
            }
            out
        }
    };
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT);
    if !(1..=MAX_LIMIT).contains(&limit) {
        return Err(crate::error::ApiError::validation(vec![
            crate::error::FieldIssue::new("limit", "out_of_range")
                .with_params(json!({"min": 1, "max": MAX_LIMIT})),
        ]));
    }
    let prefix = prefix_query(text);
    if prefix.is_empty() {
        return Ok(Json(SearchResults { items: Vec::new() }));
    }
    let mut tx = ctx.begin(&state.db).await?;
    let items: Vec<SearchHit> = sqlx::query_as(
        "WITH q AS (SELECT websearch_to_tsquery('simple', $1) || to_tsquery('simple', $2) AS t) \
         SELECT kind, id, title, subtitle, rank FROM ( \
             SELECT 'recipe' AS kind, r.id, r.name AS title, NULL::text AS subtitle, \
                 ts_rank(r.search, q.t) AS rank \
             FROM recipes r, q \
             WHERE 'recipe' = ANY($4) AND r.workspace_id = $3 AND r.deleted_at IS NULL \
               AND r.search @@ q.t \
             UNION ALL \
             SELECT 'batch', b.id, CASE WHEN b.name = '' THEN b.code ELSE b.name END, b.code, \
                 ts_rank(b.search, q.t) \
             FROM batches b, q \
             WHERE 'batch' = ANY($4) AND b.workspace_id = $3 AND b.deleted_at IS NULL \
               AND b.search @@ q.t \
             UNION ALL \
             SELECT 'ingredient', i.id, i.name, i.kind, ts_rank(i.search, q.t) \
             FROM ingredients i, q \
             WHERE 'ingredient' = ANY($4) AND (i.workspace_id = $3 OR i.workspace_id IS NULL) \
               AND i.deleted_at IS NULL AND i.search @@ q.t \
             UNION ALL \
             SELECT 'stock', l.id, i.name, l.lot_code, \
                 GREATEST(ts_rank(l.search, q.t), ts_rank(i.search, q.t)) \
             FROM stock_lots l \
             JOIN ingredients i ON i.id = l.ingredient_id, q \
             WHERE 'stock' = ANY($4) AND l.workspace_id = $3 AND l.deleted_at IS NULL \
               AND (l.search @@ q.t OR i.search @@ q.t) \
         ) h ORDER BY rank DESC, title, id LIMIT $5",
    )
    .bind(text)
    .bind(&prefix)
    .bind(ws)
    .bind(&types)
    .bind(i64::from(limit))
    .fetch_all(&mut *tx)
    .await?;
    Ok(Json(SearchResults { items }))
}

/// The search route, wrapped with the sign-in layer.
pub fn routes(state: &AppState) -> Router<AppState> {
    crate::authenticated(
        state,
        Router::new().route("/v1/workspaces/{ws}/search", get(search)),
    )
}

// --- registry entry ----------------------------------------------------------------------

mod spec {
    use crate::error::ErrorCode::{
        Forbidden, Internal, MalformedRequest, NotFound, RateLimited, Unauthenticated, Unavailable,
        UpgradeRequired, ValidationFailed,
    };
    use crate::openapi::{OperationSpec, file_op};

    pub(super) const OPERATIONS: &[OperationSpec] = &[file_op(
        "search",
        Some("recipe.view"),
        None,
        &[
            MalformedRequest,
            Unauthenticated,
            Forbidden,
            NotFound,
            ValidationFailed,
            RateLimited,
            UpgradeRequired,
            Internal,
            Unavailable,
        ],
    )];
}

/// `OpenAPI` fragment of this module.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(search),
    components(schemas(SearchHit, SearchResults)),
    tags((name = "search", description = "Global search over recipes, batches, ingredients and stock"))
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "search",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
