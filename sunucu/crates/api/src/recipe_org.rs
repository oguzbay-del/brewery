//! Folders, tags, favorites and the recipe edit session (T2.21, ADR-0007 section 3).
//!
//! Folders and tags are small workspace lists. A favorite belongs to one user. The edit session
//! is a soft lock: one row for each recipe with the holder and an end time. A lock that ended is
//! free. Another user can take it over on purpose (`take_over`). The lock warns, it does not block
//! writes. Each change goes to the outbox (`recipe.edit_session_started`,
//! `recipe.edit_session_released`) so that SSE clients see it.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, put},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{FromRow, PgConnection};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    concurrency::{self, Concurrency, check_if_match},
    context::RequestContext,
    error::{ApiError, ApiJson, ApiQuery, ApiResult, ErrorCode, FieldIssue, MergePatch},
    library::{created_response, issue, merge, name_cursor, not_found, paging, workspace},
    pagination::{ListParams, Page},
};

/// Permission that changes folders, tags, favorites and edit sessions.
pub const WRITE_PERMISSION: &str = "recipe.edit";
/// Permission that reads them.
pub const READ_PERMISSION: &str = "recipe.view";
/// How long an edit session lasts without a heartbeat, in seconds.
pub const SESSION_TTL_SECS: f64 = 90.0;

/// A recipe folder.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct Folder {
    /// Identifier.
    pub id: Uuid,
    /// Parent folder.
    pub parent_id: Option<Uuid>,
    /// Name.
    pub name: String,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Updated at.
    pub updated_at: DateTime<Utc>,
    /// Record version (`ETag`).
    pub version: i64,
}

/// A tag.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct Tag {
    /// Identifier.
    pub id: Uuid,
    /// Name (unique in the workspace, case blind).
    pub name: String,
    /// Color as `#rrggbb`.
    pub color: Option<String>,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Updated at.
    pub updated_at: DateTime<Utc>,
    /// Record version (`ETag`).
    pub version: i64,
}

/// Body of a folder create.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct NewFolder {
    /// Name, 1 to 200 characters.
    pub name: String,
    /// Parent folder.
    pub parent_id: Option<Uuid>,
}

/// Body of a tag create.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct NewTag {
    /// Name, 1 to 100 characters.
    pub name: String,
    /// Color as `#rrggbb`.
    pub color: Option<String>,
}

/// An edit session (soft lock) of a recipe.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct EditSession {
    /// Identifier.
    pub id: Uuid,
    /// The recipe.
    pub recipe_id: Uuid,
    /// The user who holds the lock.
    pub user_id: Uuid,
    /// Started at.
    pub started_at: DateTime<Utc>,
    /// Last heartbeat.
    pub last_seen_at: DateTime<Utc>,
    /// The lock ends at this time without a heartbeat.
    pub expires_at: DateTime<Utc>,
}

/// Body of an edit session start.
#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct StartSession {
    /// Take the lock from another user (default false).
    #[serde(default)]
    pub take_over: bool,
}

/// Filter of the folder and tag lists.
#[derive(Debug, Default, Deserialize)]
pub struct NameFilter {
    /// Text in the name (case blind).
    pub q: Option<String>,
}

/// Parses an optional JSON body: an empty body gives the default value.
pub(crate) fn optional_body<T: Default + serde::de::DeserializeOwned>(
    bytes: &[u8],
) -> ApiResult<T> {
    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Ok(T::default());
    }
    serde_json::from_slice(bytes).map_err(|_| ApiError::new(ErrorCode::MalformedRequest))
}

pub(crate) fn conflict(field: &str, code: &str) -> ApiError {
    ApiError::new(ErrorCode::Conflict).with_errors(vec![FieldIssue::new(field, code)])
}

fn constraint_error(e: sqlx::Error) -> ApiError {
    match crate::library::sqlstate(&e).as_deref() {
        Some("23514" | "22003" | "22P02") => issue("/", "constraint_violated"),
        Some("23505") => conflict("/name", "already_exists"),
        Some("23503") => issue("/", "reference_not_found"),
        _ => e.into(),
    }
}

fn valid_name(name: &str, max: usize) -> bool {
    let n = name.trim();
    !n.is_empty() && n.chars().count() <= max && !n.chars().any(char::is_control)
}

/// Writes an outbox event of a recipe.
pub(crate) async fn recipe_event(
    tx: &mut PgConnection,
    ws: Uuid,
    topic: &str,
    recipe_id: Uuid,
    version: i64,
    data: Option<Value>,
) -> ApiResult<()> {
    db::outbox::enqueue(
        tx,
        &db::outbox::NewOutboxEvent {
            topic: topic.to_owned(),
            workspace_id: Some(ws),
            entity: Some(db::outbox::Entity {
                kind: "recipe".to_owned(),
                id: recipe_id,
                version,
            }),
            data,
        },
    )
    .await?;
    Ok(())
}

// --- folders ------------------------------------------------------------------------------

const FOLDER_COLUMNS: &str = "id, parent_id, name, created_at, updated_at, version";

async fn fetch_folder(tx: &mut PgConnection, id: Uuid) -> ApiResult<Folder> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {FOLDER_COLUMNS} FROM folders WHERE id = $1 AND deleted_at IS NULL"
    )))
    .bind(id)
    .fetch_optional(tx)
    .await?
    .ok_or_else(not_found)
}

/// Lists the folders of the workspace.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/folders",
    tag = "recipes",
    operation_id = "folder_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("q" = Option<String>, Query, description = "Text in the name (case blind)"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page"),
        ("sort" = Option<String>, Query, description = "`name` (default) or `-name`")
    ),
    responses((status = 200, description = "A page of folders", body = Page<Folder>,
        example = json!({"items": [{"id": "0192f1c07a3b7000800000000000000b", "parent_id": null,
            "name": "IPA", "created_at": "2026-11-20T10:00:00Z",
            "updated_at": "2026-11-20T10:00:00Z", "version": 1}], "next_cursor": null})))
)]
pub async fn folder_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<NameFilter>,
) -> ApiResult<Json<Page<Folder>>> {
    ctx.require(READ_PERMISSION)?;
    let paging = paging(&params, &["name"], "name")?;
    let (cmp, dir) = paging.dir();
    let sql = sqlx::AssertSqlSafe(format!(
        "SELECT {FOLDER_COLUMNS} FROM folders \
         WHERE deleted_at IS NULL \
           AND ($1::text IS NULL OR strpos(lower(name), lower($1)) > 0) \
           AND ($2::text IS NULL OR (name, id) {cmp} ($2::text, $3::uuid)) \
         ORDER BY name {dir}, id {dir} LIMIT $4"
    ));
    let mut tx = ctx.begin(&state.db).await?;
    let rows: Vec<Folder> = sqlx::query_as(sql)
        .bind(&filter.q)
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

/// Creates a folder.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/folders",
    tag = "recipes",
    operation_id = "folder_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = NewFolder,
    responses((status = 201, description = "Folder created", body = Folder,
        example = json!({"id": "0192f1c07a3b7000800000000000000b", "parent_id": null,
            "name": "IPA", "created_at": "2026-11-20T10:00:00Z",
            "updated_at": "2026-11-20T10:00:00Z", "version": 1})))
)]
pub async fn folder_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(new): ApiJson<NewFolder>,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    if !valid_name(&new.name, 200) {
        return Err(issue("/name", "invalid_value"));
    }
    let id = Uuid::now_v7();
    let mut tx = ctx.begin(&state.db).await?;
    sqlx::query(
        "INSERT INTO folders (id, workspace_id, parent_id, name, created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $5)",
    )
    .bind(id)
    .bind(ws)
    .bind(new.parent_id)
    .bind(new.name.trim())
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    let row = fetch_folder(&mut tx, id).await?;
    tx.commit().await?;
    Ok(created_response(
        &format!("/v1/workspaces/{ws}/folders/{id}"),
        row.version,
        &row,
    ))
}

/// Renames or moves a folder.
#[utoipa::path(
    patch,
    path = "/v1/workspaces/{workspace_id}/folders/{id}",
    tag = "recipes",
    operation_id = "folder_update",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Folder"),
        ("If-Match" = Option<String>, Header, description = "Version `ETag` of the folder")
    ),
    request_body(content = NewFolder, content_type = "application/merge-patch+json"),
    responses((status = 200, description = "The updated folder", body = Folder,
        example = json!({"id": "0192f1c07a3b7000800000000000000b", "parent_id": null,
            "name": "Hazy IPA", "created_at": "2026-11-20T10:00:00Z",
            "updated_at": "2026-11-20T10:05:00Z", "version": 2})))
)]
pub async fn folder_update(
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
    let mut tx = ctx.begin(&state.db).await?;
    let before = fetch_folder(&mut tx, id).await?;
    check_if_match(Concurrency::FieldLww, &headers, before.version, || {
        json!(before)
    })?;
    let mut merged = json!({"name": before.name, "parent_id": before.parent_id});
    merge(&mut merged, &patch);
    let input: Value = merged;
    let name = input["name"].as_str().unwrap_or_default().to_owned();
    let parent: Option<Uuid> = match &input["parent_id"] {
        Value::Null => None,
        v => Some(
            v.as_str()
                .and_then(|s| Uuid::parse_str(s).ok())
                .ok_or_else(|| issue("/parent_id", "invalid_value"))?,
        ),
    };
    if !valid_name(&name, 200) {
        return Err(issue("/name", "invalid_value"));
    }
    sqlx::query(
        "UPDATE folders SET name = $2, parent_id = $3, updated_by = $4 \
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(name.trim())
    .bind(parent)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    let row = fetch_folder(&mut tx, id).await?;
    tx.commit().await?;
    Ok(concurrency::record_response(
        StatusCode::OK,
        row.version,
        &row,
    ))
}

/// Deletes an empty folder (soft delete). A folder with subfolders or recipes answers `409`.
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/folders/{id}",
    tag = "recipes",
    operation_id = "folder_delete",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Folder"),
        ("If-Match" = Option<String>, Header, description = "Version `ETag` of the folder")
    ),
    responses((status = 204, description = "Deleted"))
)]
pub async fn folder_delete(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let before = fetch_folder(&mut tx, id).await?;
    check_if_match(Concurrency::FieldLww, &headers, before.version, || {
        json!(before)
    })?;
    let used: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM folders WHERE parent_id = $1 AND deleted_at IS NULL) \
             OR EXISTS (SELECT 1 FROM recipes WHERE folder_id = $1 AND deleted_at IS NULL)",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    if used {
        return Err(conflict("/id", "folder_not_empty"));
    }
    sqlx::query("UPDATE folders SET deleted_at = now(), updated_by = $2 WHERE id = $1")
        .bind(id)
        .bind(ctx.identity.user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

// --- tags ---------------------------------------------------------------------------------

const TAG_COLUMNS: &str = "id, name, color, created_at, updated_at, version";

async fn fetch_tag(tx: &mut PgConnection, id: Uuid) -> ApiResult<Tag> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {TAG_COLUMNS} FROM tags WHERE id = $1 AND deleted_at IS NULL"
    )))
    .bind(id)
    .fetch_optional(tx)
    .await?
    .ok_or_else(not_found)
}

/// Lists the tags of the workspace.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/tags",
    tag = "recipes",
    operation_id = "tag_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("q" = Option<String>, Query, description = "Text in the name (case blind)"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page"),
        ("sort" = Option<String>, Query, description = "`name` (default) or `-name`")
    ),
    responses((status = 200, description = "A page of tags", body = Page<Tag>,
        example = json!({"items": [{"id": "0192f1c07a3b7000800000000000000b", "name": "summer",
            "color": "#ffaa00", "created_at": "2026-11-20T10:00:00Z",
            "updated_at": "2026-11-20T10:00:00Z", "version": 1}], "next_cursor": null})))
)]
pub async fn tag_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<NameFilter>,
) -> ApiResult<Json<Page<Tag>>> {
    ctx.require(READ_PERMISSION)?;
    let paging = paging(&params, &["name"], "name")?;
    let (cmp, dir) = paging.dir();
    let sql = sqlx::AssertSqlSafe(format!(
        "SELECT {TAG_COLUMNS} FROM tags \
         WHERE deleted_at IS NULL \
           AND ($1::text IS NULL OR strpos(lower(name), lower($1)) > 0) \
           AND ($2::text IS NULL OR (name, id) {cmp} ($2::text, $3::uuid)) \
         ORDER BY name {dir}, id {dir} LIMIT $4"
    ));
    let mut tx = ctx.begin(&state.db).await?;
    let rows: Vec<Tag> = sqlx::query_as(sql)
        .bind(&filter.q)
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

/// Creates a tag.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/tags",
    tag = "recipes",
    operation_id = "tag_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = NewTag,
    responses((status = 201, description = "Tag created", body = Tag,
        example = json!({"id": "0192f1c07a3b7000800000000000000b", "name": "summer",
            "color": "#ffaa00", "created_at": "2026-11-20T10:00:00Z",
            "updated_at": "2026-11-20T10:00:00Z", "version": 1})))
)]
pub async fn tag_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(new): ApiJson<NewTag>,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    if !valid_name(&new.name, 100) {
        return Err(issue("/name", "invalid_value"));
    }
    let id = Uuid::now_v7();
    let mut tx = ctx.begin(&state.db).await?;
    sqlx::query(
        "INSERT INTO tags (id, workspace_id, name, color, created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $5)",
    )
    .bind(id)
    .bind(ws)
    .bind(new.name.trim())
    .bind(&new.color)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    let row = fetch_tag(&mut tx, id).await?;
    tx.commit().await?;
    Ok(created_response(
        &format!("/v1/workspaces/{ws}/tags/{id}"),
        row.version,
        &row,
    ))
}

/// Renames a tag or changes its color.
#[utoipa::path(
    patch,
    path = "/v1/workspaces/{workspace_id}/tags/{id}",
    tag = "recipes",
    operation_id = "tag_update",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Tag"),
        ("If-Match" = Option<String>, Header, description = "Version `ETag` of the tag")
    ),
    request_body(content = NewTag, content_type = "application/merge-patch+json"),
    responses((status = 200, description = "The updated tag", body = Tag,
        example = json!({"id": "0192f1c07a3b7000800000000000000b", "name": "winter",
            "color": null, "created_at": "2026-11-20T10:00:00Z",
            "updated_at": "2026-11-20T10:05:00Z", "version": 2})))
)]
pub async fn tag_update(
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
    let mut tx = ctx.begin(&state.db).await?;
    let before = fetch_tag(&mut tx, id).await?;
    check_if_match(Concurrency::FieldLww, &headers, before.version, || {
        json!(before)
    })?;
    let mut merged = json!({"name": before.name, "color": before.color});
    merge(&mut merged, &patch);
    let name = merged["name"].as_str().unwrap_or_default().to_owned();
    let color = merged["color"].as_str().map(str::to_owned);
    if !valid_name(&name, 100) {
        return Err(issue("/name", "invalid_value"));
    }
    sqlx::query(
        "UPDATE tags SET name = $2, color = $3, updated_by = $4 \
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(name.trim())
    .bind(&color)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await
    .map_err(constraint_error)?;
    let row = fetch_tag(&mut tx, id).await?;
    tx.commit().await?;
    Ok(concurrency::record_response(
        StatusCode::OK,
        row.version,
        &row,
    ))
}

/// Deletes a tag (soft delete) and removes it from the recipes.
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/tags/{id}",
    tag = "recipes",
    operation_id = "tag_delete",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Tag"),
        ("If-Match" = Option<String>, Header, description = "Version `ETag` of the tag")
    ),
    responses((status = 204, description = "Deleted"))
)]
pub async fn tag_delete(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let before = fetch_tag(&mut tx, id).await?;
    check_if_match(Concurrency::FieldLww, &headers, before.version, || {
        json!(before)
    })?;
    sqlx::query("DELETE FROM recipe_tags WHERE tag_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM batch_tags WHERE tag_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE tags SET deleted_at = now(), updated_by = $2 WHERE id = $1")
        .bind(id)
        .bind(ctx.identity.user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

// --- recipe tags and favorites --------------------------------------------------------------

async fn recipe_exists(tx: &mut PgConnection, id: Uuid) -> ApiResult<()> {
    let found: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM recipes WHERE id = $1 AND deleted_at IS NULL)",
    )
    .bind(id)
    .fetch_one(tx)
    .await?;
    if found { Ok(()) } else { Err(not_found()) }
}

/// Puts a tag on a recipe. The call is safe to repeat.
#[utoipa::path(
    put,
    path = "/v1/workspaces/{workspace_id}/recipes/{id}/tags/{tag_id}",
    tag = "recipes",
    operation_id = "recipetag_put",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Recipe"),
        ("tag_id" = Uuid, Path, description = "Tag")
    ),
    responses((status = 204, description = "The recipe has the tag"))
)]
pub async fn recipetag_put(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id, tag_id)): Path<(Uuid, Uuid, Uuid)>,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    recipe_exists(&mut tx, id).await?;
    let tag_ok: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM tags WHERE id = $1 AND deleted_at IS NULL)",
    )
    .bind(tag_id)
    .fetch_one(&mut *tx)
    .await?;
    if !tag_ok {
        return Err(not_found());
    }
    sqlx::query(
        "INSERT INTO recipe_tags (workspace_id, recipe_id, tag_id, created_by) \
         VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
    )
    .bind(ws)
    .bind(id)
    .bind(tag_id)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Removes a tag from a recipe. The call is safe to repeat.
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/recipes/{id}/tags/{tag_id}",
    tag = "recipes",
    operation_id = "recipetag_delete",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Recipe"),
        ("tag_id" = Uuid, Path, description = "Tag")
    ),
    responses((status = 204, description = "The recipe has no such tag"))
)]
pub async fn recipetag_delete(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id, tag_id)): Path<(Uuid, Uuid, Uuid)>,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    recipe_exists(&mut tx, id).await?;
    sqlx::query("DELETE FROM recipe_tags WHERE recipe_id = $1 AND tag_id = $2")
        .bind(id)
        .bind(tag_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Marks a recipe as favorite for the caller. The call is safe to repeat.
#[utoipa::path(
    put,
    path = "/v1/workspaces/{workspace_id}/recipes/{id}/favorite",
    tag = "recipes",
    operation_id = "recipefavorite_put",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Recipe")
    ),
    responses((status = 204, description = "The recipe is a favorite"))
)]
pub async fn recipefavorite_put(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Response> {
    ctx.require(READ_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    recipe_exists(&mut tx, id).await?;
    sqlx::query(
        "INSERT INTO favorites (id, workspace_id, user_id, recipe_id) \
         VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
    )
    .bind(Uuid::now_v7())
    .bind(ws)
    .bind(ctx.identity.user_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Removes the favorite mark of the caller. The call is safe to repeat.
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/recipes/{id}/favorite",
    tag = "recipes",
    operation_id = "recipefavorite_delete",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Recipe")
    ),
    responses((status = 204, description = "The recipe is not a favorite"))
)]
pub async fn recipefavorite_delete(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Response> {
    ctx.require(READ_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    sqlx::query("DELETE FROM favorites WHERE recipe_id = $1 AND user_id = $2")
        .bind(id)
        .bind(ctx.identity.user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

// --- edit session ---------------------------------------------------------------------------

const SESSION_COLUMNS: &str = "id, recipe_id, user_id, started_at, last_seen_at, expires_at";

async fn active_session(tx: &mut PgConnection, recipe_id: Uuid) -> ApiResult<Option<EditSession>> {
    Ok(sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {SESSION_COLUMNS} FROM edit_sessions \
         WHERE recipe_id = $1 AND expires_at > now() FOR UPDATE"
    )))
    .bind(recipe_id)
    .fetch_optional(tx)
    .await?)
}

async fn recipe_version(tx: &mut PgConnection, id: Uuid) -> ApiResult<i64> {
    sqlx::query_scalar("SELECT version FROM recipes WHERE id = $1 AND deleted_at IS NULL")
        .bind(id)
        .fetch_optional(tx)
        .await?
        .ok_or_else(not_found)
}

/// Reads the edit session (soft lock) of a recipe. The answer is `null` when nobody holds it.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/recipes/{id}/edit-session",
    tag = "recipes",
    operation_id = "editsession_get",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Recipe")
    ),
    responses((status = 200, description = "The active session or `null`", body = Option<EditSession>,
        example = json!({"id": "0192f1c07a3b7000800000000000000b",
            "recipe_id": "0192f1c07a3b7000800000000000000c",
            "user_id": "0192f1c07a3b7000800000000000000d",
            "started_at": "2026-11-20T10:00:00Z", "last_seen_at": "2026-11-20T10:00:30Z",
            "expires_at": "2026-11-20T10:02:00Z"})))
)]
pub async fn editsession_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<Option<EditSession>>> {
    ctx.require(READ_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    recipe_version(&mut tx, id).await?;
    let session = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {SESSION_COLUMNS} FROM edit_sessions WHERE recipe_id = $1 AND expires_at > now()"
    )))
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;
    Ok(Json(session))
}

/// Starts an edit session (takes the soft lock). Another user who holds it gets `409` with the
/// holder in `current`, unless `take_over` is true.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/recipes/{id}/edit-session",
    tag = "recipes",
    operation_id = "editsession_start",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Recipe")
    ),
    request_body = StartSession,
    responses((status = 200, description = "The session of the caller", body = EditSession,
        example = json!({"id": "0192f1c07a3b7000800000000000000b",
            "recipe_id": "0192f1c07a3b7000800000000000000c",
            "user_id": "0192f1c07a3b7000800000000000000d",
            "started_at": "2026-11-20T10:00:00Z", "last_seen_at": "2026-11-20T10:00:00Z",
            "expires_at": "2026-11-20T10:01:30Z"})))
)]
pub async fn editsession_start(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    body: axum::body::Bytes,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let take_over = optional_body::<StartSession>(&body)?.take_over;
    let user = ctx.identity.user_id;
    let mut tx = ctx.begin(&state.db).await?;
    let version = recipe_version(&mut tx, id).await?;
    // Serialize the starters of one recipe: lock the recipe row.
    sqlx::query("SELECT 1 FROM recipes WHERE id = $1 FOR UPDATE")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    if let Some(held) = active_session(&mut tx, id).await?
        && held.user_id != user
        && !take_over
    {
        return Err(ApiError::new(ErrorCode::Conflict)
            .with_errors(vec![FieldIssue::new("/", "edit_session_held")])
            .with_current(json!(held)));
    }
    let session: EditSession = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "INSERT INTO edit_sessions (id, workspace_id, recipe_id, user_id, expires_at) \
         VALUES ($1, $2, $3, $4, now() + make_interval(secs => $5)) \
         ON CONFLICT (workspace_id, recipe_id) DO UPDATE SET \
             user_id = EXCLUDED.user_id, \
             started_at = CASE WHEN edit_sessions.user_id = EXCLUDED.user_id \
                                AND edit_sessions.expires_at > now() \
                               THEN edit_sessions.started_at ELSE now() END, \
             last_seen_at = now(), expires_at = EXCLUDED.expires_at \
         RETURNING {SESSION_COLUMNS}"
    )))
    .bind(Uuid::now_v7())
    .bind(ws)
    .bind(id)
    .bind(user)
    .bind(SESSION_TTL_SECS)
    .fetch_one(&mut *tx)
    .await?;
    recipe_event(
        &mut tx,
        ws,
        "recipe.edit_session_started",
        id,
        version,
        Some(json!({"user_id": user})),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(session).into_response())
}

/// Extends the edit session of the caller (heartbeat). Use it about every 30 seconds.
#[utoipa::path(
    put,
    path = "/v1/workspaces/{workspace_id}/recipes/{id}/edit-session",
    tag = "recipes",
    operation_id = "editsession_renew",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Recipe")
    ),
    responses((status = 200, description = "The extended session", body = EditSession,
        example = json!({"id": "0192f1c07a3b7000800000000000000b",
            "recipe_id": "0192f1c07a3b7000800000000000000c",
            "user_id": "0192f1c07a3b7000800000000000000d",
            "started_at": "2026-11-20T10:00:00Z", "last_seen_at": "2026-11-20T10:00:30Z",
            "expires_at": "2026-11-20T10:02:00Z"})))
)]
pub async fn editsession_renew(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<EditSession>> {
    ctx.require(WRITE_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    recipe_version(&mut tx, id).await?;
    let held = active_session(&mut tx, id).await?.ok_or_else(not_found)?;
    if held.user_id != ctx.identity.user_id {
        return Err(ApiError::new(ErrorCode::Conflict)
            .with_errors(vec![FieldIssue::new("/", "edit_session_held")])
            .with_current(json!(held)));
    }
    let session: EditSession = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "UPDATE edit_sessions SET last_seen_at = now(), \
             expires_at = now() + make_interval(secs => $2) \
         WHERE recipe_id = $1 RETURNING {SESSION_COLUMNS}"
    )))
    .bind(id)
    .bind(SESSION_TTL_SECS)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(session))
}

/// Releases the edit session of the caller. The call is safe to repeat.
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/recipes/{id}/edit-session",
    tag = "recipes",
    operation_id = "editsession_release",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Recipe")
    ),
    responses((status = 204, description = "The caller holds no lock"))
)]
pub async fn editsession_release(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    let version = recipe_version(&mut tx, id).await?;
    let done = sqlx::query("DELETE FROM edit_sessions WHERE recipe_id = $1 AND user_id = $2")
        .bind(id)
        .bind(ctx.identity.user_id)
        .execute(&mut *tx)
        .await?;
    if done.rows_affected() > 0 {
        recipe_event(
            &mut tx,
            ws,
            "recipe.edit_session_released",
            id,
            version,
            Some(json!({"user_id": ctx.identity.user_id})),
        )
        .await?;
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

// --- routes -------------------------------------------------------------------------------

/// The routes of this module, wrapped with the sign-in and idempotency layers.
pub fn routes(state: &AppState) -> Router<AppState> {
    const WS: &str = "/v1/workspaces/{ws}";
    let creates = crate::idempotent(
        state,
        Router::new()
            .route(&format!("{WS}/folders"), axum::routing::post(folder_create))
            .route(&format!("{WS}/tags"), axum::routing::post(tag_create)),
    );
    let others = Router::new()
        .route(&format!("{WS}/folders"), get(folder_list))
        .route(
            &format!("{WS}/folders/{{id}}"),
            axum::routing::patch(folder_update).delete(folder_delete),
        )
        .route(&format!("{WS}/tags"), get(tag_list))
        .route(
            &format!("{WS}/tags/{{id}}"),
            axum::routing::patch(tag_update).delete(tag_delete),
        )
        .route(
            &format!("{WS}/recipes/{{id}}/tags/{{tag_id}}"),
            put(recipetag_put).delete(recipetag_delete),
        )
        .route(
            &format!("{WS}/recipes/{{id}}/favorite"),
            put(recipefavorite_put).delete(recipefavorite_delete),
        )
        .route(
            &format!("{WS}/recipes/{{id}}/edit-session"),
            get(editsession_get)
                .post(editsession_start)
                .put(editsession_renew)
                .delete(editsession_release),
        );
    crate::authenticated(state, creates.merge(others))
}

// --- registry entry ----------------------------------------------------------------------

/// Operation specs of this module (rows for `crate::openapi`).
mod spec {
    use crate::concurrency::Concurrency;
    use crate::error::ErrorCode;
    use crate::error::ErrorCode::{
        Conflict, Forbidden, IdempotencyInProgress, IdempotencyMismatch, Internal,
        MalformedRequest, NotFound, PayloadTooLarge, RateLimited, Unauthenticated, Unavailable,
        UnsupportedMediaType, UpgradeRequired, ValidationFailed, VersionMismatch,
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
        RateLimited,
        UpgradeRequired,
        Internal,
        Unavailable,
    ];
    const ACTION: &[ErrorCode] = &[
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
        file_op("folder_list", Some("recipe.view"), None, LIST),
        file_op(
            "folder_create",
            Some("recipe.edit"),
            Some(Concurrency::Append),
            CREATE,
        ),
        file_op(
            "folder_update",
            Some("recipe.edit"),
            Some(Concurrency::FieldLww),
            UPDATE,
        ),
        file_op(
            "folder_delete",
            Some("recipe.edit"),
            Some(Concurrency::FieldLww),
            UPDATE,
        ),
        file_op("tag_list", Some("recipe.view"), None, LIST),
        file_op(
            "tag_create",
            Some("recipe.edit"),
            Some(Concurrency::Append),
            CREATE,
        ),
        file_op(
            "tag_update",
            Some("recipe.edit"),
            Some(Concurrency::FieldLww),
            UPDATE,
        ),
        file_op(
            "tag_delete",
            Some("recipe.edit"),
            Some(Concurrency::FieldLww),
            UPDATE,
        ),
        file_op("recipetag_put", Some("recipe.edit"), None, ACTION),
        file_op("recipetag_delete", Some("recipe.edit"), None, ACTION),
        file_op("recipefavorite_put", Some("recipe.view"), None, ACTION),
        file_op("recipefavorite_delete", Some("recipe.view"), None, ACTION),
        file_op("editsession_get", Some("recipe.view"), None, LIST),
        file_op("editsession_start", Some("recipe.edit"), None, ACTION),
        file_op("editsession_renew", Some("recipe.edit"), None, ACTION),
        file_op("editsession_release", Some("recipe.edit"), None, ACTION),
    ];
}

/// `OpenAPI` fragment of this module; `crate::openapi` merges it.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(
        folder_list,
        folder_create,
        folder_update,
        folder_delete,
        tag_list,
        tag_create,
        tag_update,
        tag_delete,
        recipetag_put,
        recipetag_delete,
        recipefavorite_put,
        recipefavorite_delete,
        editsession_get,
        editsession_start,
        editsession_renew,
        editsession_release
    ),
    components(schemas(Folder, Tag, NewFolder, NewTag, EditSession, StartSession)),
    tags((name = "recipes", description = "Recipes, versions, folders, tags, favorites and the edit session"))
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "recipe_org",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
