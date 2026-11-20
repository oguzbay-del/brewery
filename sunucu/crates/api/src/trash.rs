//! Recently deleted (T2.37): list the soft-deleted recipes, batches, ingredients and equipment
//! of the last 30 days, and restore them.
//!
//! The worker deletes a record for good 30 days after the soft delete
//! (`purge_expired_soft_deletes`, migration 0191). Until then the user can restore it. After
//! that, restore answers `409` (`restore_window_expired`). A record of another workspace is
//! invisible (row level security) and answers `404`.
//!
//! Each kind needs the write permission of its own endpoints: `recipe.edit` for recipes and
//! ingredients, `batch.plan` for batches, `settings.equipment` for equipment. The list shows
//! only the kinds the caller may restore.

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    context::RequestContext,
    error::{ApiError, ApiQuery, ApiResult, ErrorCode, FieldIssue},
    library::{issue, not_found, sqlstate, workspace},
};

/// Days a deleted record can be restored.
pub const RESTORE_DAYS: i64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Recipe,
    Batch,
    Ingredient,
    Equipment,
}

const KINDS: [Kind; 4] = [Kind::Recipe, Kind::Batch, Kind::Ingredient, Kind::Equipment];

impl Kind {
    const fn name(self) -> &'static str {
        match self {
            Self::Recipe => "recipe",
            Self::Batch => "batch",
            Self::Ingredient => "ingredient",
            Self::Equipment => "equipment",
        }
    }
    fn parse(s: &str) -> Option<Self> {
        KINDS.into_iter().find(|k| k.name() == s)
    }
    const fn table(self) -> &'static str {
        match self {
            Self::Recipe => "recipes",
            Self::Batch => "batches",
            Self::Ingredient => "ingredients",
            Self::Equipment => "equipment",
        }
    }
    const fn permission(self) -> &'static str {
        match self {
            Self::Recipe | Self::Ingredient => "recipe.edit",
            Self::Batch => "batch.plan",
            Self::Equipment => "settings.equipment",
        }
    }
    /// Library ingredients are shared and never deleted by a user.
    const fn scope(self) -> &'static str {
        match self {
            Self::Ingredient => {
                " AND workspace_id = (SELECT app_workspace_id()) AND source = 'user'"
            }
            _ => "",
        }
    }
}

/// A record in the trash.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct TrashItem {
    /// `recipe`, `batch`, `ingredient` or `equipment`.
    pub kind: String,
    /// Record id.
    pub id: Uuid,
    /// Name of the record.
    pub name: String,
    /// When the record was deleted.
    pub deleted_at: DateTime<Utc>,
    /// Last instant a restore works.
    pub restorable_until: DateTime<Utc>,
}

/// The trash, newest deletion first.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TrashList {
    /// Deleted records of the last 30 days.
    pub items: Vec<TrashItem>,
}

/// Query of the list.
#[derive(Debug, Default, Clone, Deserialize, ToSchema)]
pub struct TrashQuery {
    /// Only this kind: `recipe`, `batch`, `ingredient` or `equipment`.
    pub kind: Option<String>,
    /// Largest number of items (1..=200, default 50).
    pub limit: Option<u32>,
}

/// Routes of this module.
pub fn routes(state: &AppState) -> Router<AppState> {
    const WS: &str = "/v1/workspaces/{ws}/trash";
    crate::authenticated(
        state,
        Router::new().route(WS, get(trash_list)).route(
            &format!("{WS}/{{kind}}/{{id}}/restore"),
            post(trash_restore),
        ),
    )
}

/// Lists the records deleted in the last 30 days that the caller may restore.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/trash",
    tag = "trash",
    operation_id = "trash_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("kind" = Option<String>, Query, description = "Only this kind"),
        ("limit" = Option<u32>, Query, description = "Largest number of items (1..=200)")
    ),
    responses((status = 200, description = "Deleted records", body = TrashList,
        example = json!({"items": [{"kind": "recipe",
            "id": "0192f1c07a3b7000800000000000000a", "name": "House IPA",
            "deleted_at": "2026-11-20T10:00:00Z", "restorable_until": "2026-12-20T10:00:00Z"}]}))
    )
)]
pub async fn trash_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(query): ApiQuery<TrashQuery>,
) -> ApiResult<Json<TrashList>> {
    workspace(&ctx)?;
    let only = match query.kind.as_deref() {
        None => None,
        Some(s) => Some(Kind::parse(s).ok_or_else(|| issue("kind", "invalid_value"))?),
    };
    let limit = query.limit.unwrap_or(50);
    if !(1..=200).contains(&limit) {
        return Err(ApiError::validation(vec![
            FieldIssue::new("limit", "out_of_range").with_params(json!({"min": 1, "max": 200})),
        ]));
    }
    let parts: Vec<String> = KINDS
        .into_iter()
        .filter(|k| only.is_none_or(|o| o == *k) && ctx.has_permission(k.permission()))
        .map(|k| {
            format!(
                "SELECT '{kind}'::text AS kind, id, name, deleted_at, \
                    deleted_at + make_interval(days => {RESTORE_DAYS}) AS restorable_until \
                 FROM {table} WHERE deleted_at > now() - make_interval(days => {RESTORE_DAYS}){scope}",
                kind = k.name(),
                table = k.table(),
                scope = k.scope()
            )
        })
        .collect();
    if parts.is_empty() {
        return Ok(Json(TrashList { items: Vec::new() }));
    }
    let sql = sqlx::AssertSqlSafe(format!(
        "SELECT * FROM ({}) t ORDER BY deleted_at DESC, id LIMIT $1",
        parts.join(" UNION ALL ")
    ));
    let mut tx = ctx.begin(&state.db).await?;
    let items: Vec<TrashItem> = sqlx::query_as(sql)
        .bind(i64::from(limit))
        .fetch_all(&mut *tx)
        .await?;
    Ok(Json(TrashList { items }))
}

/// Restores a deleted record within 30 days of its deletion.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/trash/{kind}/{id}/restore",
    tag = "trash",
    operation_id = "trash_restore",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("kind" = String, Path, description = "`recipe`, `batch`, `ingredient` or `equipment`"),
        ("id" = Uuid, Path, description = "Record")
    ),
    responses((status = 200, description = "The restored record", body = TrashItem,
        example = json!({"kind": "recipe", "id": "0192f1c07a3b7000800000000000000a",
            "name": "House IPA", "deleted_at": "2026-11-20T10:00:00Z",
            "restorable_until": "2026-12-20T10:00:00Z"}))
    )
)]
pub async fn trash_restore(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, kind, id)): Path<(Uuid, String, Uuid)>,
) -> ApiResult<Json<TrashItem>> {
    workspace(&ctx)?;
    let kind = Kind::parse(&kind).ok_or_else(not_found)?;
    ctx.require(kind.permission())?;
    let mut tx = ctx.begin(&state.db).await?;
    // Not deleted, of another workspace, or already gone for good: not found.
    let (name, deleted_at): (String, DateTime<Utc>) = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT name, deleted_at FROM {} WHERE id = $1 AND deleted_at IS NOT NULL{} FOR UPDATE",
        kind.table(),
        kind.scope()
    )))
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(not_found)?;
    let until = deleted_at + Duration::days(RESTORE_DAYS);
    if until <= Utc::now() {
        return Err(ApiError::new(ErrorCode::Conflict)
            .with_errors(vec![FieldIssue::new("/id", "restore_window_expired")]));
    }
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "UPDATE {} SET deleted_at = NULL, updated_by = $2 WHERE id = $1",
        kind.table()
    )))
    .bind(id)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| match sqlstate(&e).as_deref() {
        Some("23505") => {
            ApiError::new(ErrorCode::Conflict).with_errors(vec![FieldIssue::new("/", "taken")])
        }
        _ => e.into(),
    })?;
    tx.commit().await?;
    Ok(Json(TrashItem {
        kind: kind.name().to_owned(),
        id,
        name,
        deleted_at,
        restorable_until: until,
    }))
}

// --- registry entry ----------------------------------------------------------------------

mod spec {
    use crate::error::ErrorCode::{
        Conflict, Forbidden, Internal, MalformedRequest, NotFound, RateLimited, Unauthenticated,
        Unavailable, UpgradeRequired, ValidationFailed,
    };
    use crate::openapi::{OperationSpec, file_op};

    pub(super) const OPERATIONS: &[OperationSpec] = &[
        file_op(
            "trash_list",
            None,
            None,
            &[
                MalformedRequest,
                Unauthenticated,
                NotFound,
                ValidationFailed,
                RateLimited,
                UpgradeRequired,
                Internal,
                Unavailable,
            ],
        ),
        file_op(
            "trash_restore",
            None,
            None,
            &[
                MalformedRequest,
                Unauthenticated,
                Forbidden,
                NotFound,
                Conflict,
                RateLimited,
                UpgradeRequired,
                Internal,
                Unavailable,
            ],
        ),
    ];
}

/// `OpenAPI` fragment of this module.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(trash_list, trash_restore),
    components(schemas(TrashItem, TrashList)),
    tags((name = "trash", description = "Recently deleted records (30 days) and their restore"))
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "trash",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
