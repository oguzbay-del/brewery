//! Equipment, profiles, vessels and settings (T2.20; ADR-0008 rule 5 `field_lww`).
//!
//! - Resources (`fields::RESOURCES`): equipment, mash, fermentation, carbonation and water
//!   profiles, vessels. One generic implementation serves all of them; the field tables drive
//!   validation and the `OpenAPI` schema.
//! - Concurrency is `field_lww`: `PATCH` (JSON Merge Patch) writes only the sent fields, the last
//!   writer wins for each field and `If-Match` is optional. Every change that alters a value
//!   writes one `change_history` row. `previous_value` holds the replaced values of the changed
//!   fields only and `record_version` is the version the change produced. Restoring a row writes
//!   those values back as a new change, so a restore has its own history row.
//! - Delete is soft (`deleted_at`) and goes through the same history, so it can be restored.
//! - Workspace settings (`workspaces.settings`) and user settings (`user_settings`) use the same
//!   rules, with keys instead of columns.
//! - Wizard and calibration proposal: `extras.rs`.

#![allow(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::format_collect
)]

mod extras;
mod fields;
mod spec;

use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use fields::{
    READ_PERMISSIONS, RESOURCES, Resource, SETTING_KEYS, WRITE_PERMISSION, check_object, json_eq,
};
use serde_json::{Map, Value, json};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    AppState, authenticated,
    authz::Permission,
    concurrency::{Concurrency, check_if_match, conditional_get, record_response},
    context::RequestContext,
    error::{ApiError, ApiQuery, ApiResult, ErrorCode, FieldIssue, MergePatch},
    idempotent,
    pagination::{Cursor, ListParams, Page, SortSpec},
};

pub use spec::MODULE;

/// SQL assembled from constants of this module only (never from request data).
macro_rules! sql {
    ($($arg:tt)*) => {
        sqlx::AssertSqlSafe(format!($($arg)*))
    };
}

const WS: &str = "/v1/workspaces/{ws}";
/// Most steps a profile can have.
const MAX_STEPS: usize = 50;

fn not_found() -> ApiError {
    ApiError::new(ErrorCode::NotFound)
}

fn ws_of(ctx: &RequestContext) -> ApiResult<Uuid> {
    ctx.workspace_id.ok_or_else(not_found)
}

fn require_read(ctx: &RequestContext) -> ApiResult<()> {
    if READ_PERMISSIONS.iter().any(|p| ctx.can(*p)) {
        Ok(())
    } else {
        Err(ApiError::new(ErrorCode::Forbidden))
    }
}

fn require_write(ctx: &RequestContext) -> ApiResult<()> {
    ctx.require_permission(WRITE_PERMISSION)
}

fn object(body: Value) -> ApiResult<Map<String, Value>> {
    match body {
        Value::Object(m) => Ok(m),
        _ => Err(ApiError::validation(vec![FieldIssue::new(
            "/",
            "invalid_type",
        )])),
    }
}

/// Maps a constraint error of the database to the error envelope.
fn map_db(r: Option<&Resource>, e: sqlx::Error) -> ApiError {
    let code = e
        .as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .map(std::borrow::Cow::into_owned);
    match code.as_deref() {
        Some("23505") => {
            let field = r
                .and_then(|r| r.unique)
                .map_or_else(|| "/".to_owned(), |f| format!("/{f}"));
            ApiError::new(ErrorCode::Conflict).with_errors(vec![FieldIssue::new(field, "taken")])
        }
        Some("23503") => ApiError::validation(vec![FieldIssue::new("/", "unknown_reference")]),
        Some("23514") => {
            let split = e
                .as_database_error()
                .and_then(sqlx::error::DatabaseError::constraint)
                == Some("equipment_efficiency_split_ck");
            if split {
                ApiError::validation(vec![
                    FieldIssue::new("/conversion_efficiency_pct", "split_incomplete"),
                    FieldIssue::new("/lauter_efficiency_pct", "split_incomplete"),
                ])
            } else {
                ApiError::validation(vec![FieldIssue::new("/", "check_failed")])
            }
        }
        _ => e.into(),
    }
}

/// The routes of this module.
pub fn routes(state: &AppState) -> Router<AppState> {
    let mut plain = Router::new();
    let mut creates = Router::new();
    for r in &RESOURCES {
        let base = format!("{WS}/{}", r.path);
        let one = format!("{base}/{{id}}");
        plain = plain
            .route(
                &base,
                get(
                    move |State(s): State<AppState>,
                          ctx: RequestContext,
                          ApiQuery(q): ApiQuery<ListParams>| list(s, ctx, r, q),
                ),
            )
            .route(
                &one,
                get(
                    move |State(s): State<AppState>,
                          ctx: RequestContext,
                          Path((_, id)): Path<(Uuid, Uuid)>,
                          h: HeaderMap| get_one(s, ctx, r, id, h),
                )
                .patch(
                    move |State(s): State<AppState>,
                          ctx: RequestContext,
                          Path((_, id)): Path<(Uuid, Uuid)>,
                          h: HeaderMap,
                          MergePatch(b): MergePatch<Value>| {
                        update(s, ctx, r, id, h, b)
                    },
                )
                .delete(
                    move |State(s): State<AppState>,
                          ctx: RequestContext,
                          Path((_, id)): Path<(Uuid, Uuid)>,
                          h: HeaderMap| remove(s, ctx, r, id, h),
                ),
            )
            .route(
                &format!("{one}/history"),
                get(
                    move |State(s): State<AppState>,
                          ctx: RequestContext,
                          Path((_, id)): Path<(Uuid, Uuid)>,
                          ApiQuery(q): ApiQuery<ListParams>| {
                        history(s, ctx, r, id, q)
                    },
                ),
            )
            .route(
                &format!("{one}/history/{{hid}}/restore"),
                post(
                    move |State(s): State<AppState>,
                          ctx: RequestContext,
                          Path((_, id, hid)): Path<(Uuid, Uuid, Uuid)>| {
                        restore(s, ctx, r, id, hid)
                    },
                ),
            );
        creates = creates.route(
            &base,
            post(
                move |State(s): State<AppState>,
                      ctx: RequestContext,
                      crate::error::ApiJson(b): crate::error::ApiJson<Value>| {
                    create(s, ctx, r, b)
                },
            ),
        );
    }
    plain = plain
        .route(
            &format!("{WS}/settings"),
            get(
                |State(s): State<AppState>, ctx: RequestContext, h: HeaderMap| {
                    settings_get(s, ctx, Scope::Workspace, h)
                },
            )
            .patch(
                |State(s): State<AppState>,
                 ctx: RequestContext,
                 h: HeaderMap,
                 MergePatch(b): MergePatch<Value>| {
                    settings_update(s, ctx, Scope::Workspace, h, b)
                },
            ),
        )
        .route(
            &format!("{WS}/settings/history"),
            get(
                |State(s): State<AppState>,
                 ctx: RequestContext,
                 ApiQuery(q): ApiQuery<ListParams>| {
                    settings_history(s, ctx, Scope::Workspace, q)
                },
            ),
        )
        .route(
            &format!("{WS}/settings/history/{{hid}}/restore"),
            post(
                |State(s): State<AppState>,
                 ctx: RequestContext,
                 Path((_, hid)): Path<(Uuid, Uuid)>| {
                    settings_restore(s, ctx, Scope::Workspace, hid)
                },
            ),
        )
        .route(
            &format!("{WS}/me/settings"),
            get(
                |State(s): State<AppState>, ctx: RequestContext, h: HeaderMap| {
                    settings_get(s, ctx, Scope::User, h)
                },
            )
            .patch(
                |State(s): State<AppState>,
                 ctx: RequestContext,
                 h: HeaderMap,
                 MergePatch(b): MergePatch<Value>| {
                    settings_update(s, ctx, Scope::User, h, b)
                },
            ),
        )
        .route(
            &format!("{WS}/me/settings/history"),
            get(
                |State(s): State<AppState>,
                 ctx: RequestContext,
                 ApiQuery(q): ApiQuery<ListParams>| {
                    settings_history(s, ctx, Scope::User, q)
                },
            ),
        )
        .route(
            &format!("{WS}/me/settings/history/{{hid}}/restore"),
            post(
                |State(s): State<AppState>,
                 ctx: RequestContext,
                 Path((_, hid)): Path<(Uuid, Uuid)>| {
                    settings_restore(s, ctx, Scope::User, hid)
                },
            ),
        );
    let (extra_plain, extra_creates) = extras::routes();
    let creates = idempotent(state, creates.merge(extra_creates));
    authenticated(state, plain.merge(extra_plain).merge(creates))
}

// --- reading --------------------------------------------------------------------------------

async fn load_row(
    tx: &mut PgConnection,
    r: &Resource,
    ws: Uuid,
    id: Uuid,
    deleted_ok: bool,
    lock: bool,
) -> ApiResult<Value> {
    let live = if deleted_ok {
        ""
    } else {
        "AND deleted_at IS NULL"
    };
    let lock = if lock { "FOR UPDATE" } else { "" };
    sqlx::query_scalar::<_, Value>(sql!(
        "SELECT to_jsonb(t) FROM {} t WHERE workspace_id = $1 AND id = $2 {live} {lock}",
        r.table
    ))
    .bind(ws)
    .bind(id)
    .fetch_optional(tx)
    .await?
    .ok_or_else(not_found)
}

/// Steps of the profiles `ids`, in order.
async fn load_steps(
    tx: &mut PgConnection,
    r: &Resource,
    ws: Uuid,
    ids: &[Uuid],
) -> ApiResult<HashMap<Uuid, Vec<Value>>> {
    let mut out: HashMap<Uuid, Vec<Value>> = HashMap::new();
    let Some(steps) = &r.steps else {
        return Ok(out);
    };
    let rows = sqlx::query_as::<_, (Uuid, Value)>(sql!(
        "SELECT profile_id, to_jsonb(s) FROM {} s \
         WHERE workspace_id = $1 AND profile_id = ANY($2) AND deleted_at IS NULL \
         ORDER BY profile_id, position",
        steps.table
    ))
    .bind(ws)
    .bind(ids)
    .fetch_all(tx)
    .await?;
    for (profile, row) in rows {
        let mut step = Map::new();
        for f in steps.fields {
            step.insert(
                f.name.to_owned(),
                row.get(f.name).cloned().unwrap_or(Value::Null),
            );
        }
        out.entry(profile).or_default().push(Value::Object(step));
    }
    Ok(out)
}

/// The public JSON of a row.
fn project(r: &Resource, row: &Value, steps: Option<Vec<Value>>) -> Value {
    let mut out = Map::new();
    let pick = |k: &str| row.get(k).cloned().unwrap_or(Value::Null);
    out.insert("id".to_owned(), pick("id"));
    for f in r.fields {
        out.insert(f.name.to_owned(), pick(f.name));
    }
    if r.steps.is_some() {
        out.insert("steps".to_owned(), Value::Array(steps.unwrap_or_default()));
    }
    for k in ["version", "created_at", "updated_at", "deleted_at"] {
        out.insert(k.to_owned(), pick(k));
    }
    Value::Object(out)
}

fn version_of(row: &Value) -> i64 {
    row.get("version").and_then(Value::as_i64).unwrap_or(0)
}

async fn record(
    tx: &mut PgConnection,
    r: &Resource,
    ws: Uuid,
    id: Uuid,
    deleted_ok: bool,
) -> ApiResult<(Value, i64)> {
    let row = load_row(tx, r, ws, id, deleted_ok, false).await?;
    let steps = load_steps(tx, r, ws, &[id]).await?.remove(&id);
    Ok((project(r, &row, steps), version_of(&row)))
}

async fn list(
    state: AppState,
    ctx: RequestContext,
    r: &'static Resource,
    params: ListParams,
) -> ApiResult<Json<Page<Value>>> {
    require_read(&ctx)?;
    let ws = ws_of(&ctx)?;
    let sort = SortSpec::parse(params.sort.as_deref(), r.sort, r.sort[0])?;
    let limit = params.limit()?;
    let cursor = params.cursor(&sort)?;
    let (ckey, cid) = cursor.map_or((None, None), |c| {
        (c.key.as_str().map(str::to_owned), Some(c.id))
    });
    let ty = if sort.field == "updated_at" {
        "timestamptz"
    } else {
        "text"
    };
    let (cmp, dir) = if sort.descending {
        ("<", "DESC")
    } else {
        (">", "ASC")
    };
    let col = sort.field;
    let mut tx = ctx.begin(&state.db).await?;
    let rows = sqlx::query_scalar::<_, Value>(sql!(
        "SELECT to_jsonb(t) FROM {} t WHERE workspace_id = $1 \
           AND (deleted_at IS NULL OR $2) \
           AND ($3::timestamptz IS NULL OR updated_at > $3) \
           AND ($4::text IS NULL OR ({col}, id) {cmp} (($4::text)::{ty}, $5::uuid)) \
         ORDER BY {col} {dir}, id {dir} LIMIT $6",
        r.table
    ))
    .bind(ws)
    .bind(params.include_deleted.unwrap_or(false) || params.updated_since.is_some())
    .bind(params.updated_since)
    .bind(ckey)
    .bind(cid)
    .bind(i64::from(limit) + 1)
    .fetch_all(&mut *tx)
    .await?;
    let page = Page::from_rows(rows, limit, &sort, |row| Cursor {
        key: row.get(col).cloned().unwrap_or(Value::Null),
        id: row
            .get("id")
            .and_then(Value::as_str)
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_default(),
    });
    let ids: Vec<Uuid> = page
        .items
        .iter()
        .filter_map(|row| {
            row.get("id")?
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok())
        })
        .collect();
    let mut steps = load_steps(&mut tx, r, ws, &ids).await?;
    tx.commit().await?;
    let items = page
        .items
        .iter()
        .map(|row| {
            let id = row
                .get("id")
                .and_then(Value::as_str)
                .and_then(|s| Uuid::parse_str(s).ok());
            project(r, row, id.and_then(|id| steps.remove(&id)))
        })
        .collect();
    Ok(Json(Page {
        items,
        next_cursor: page.next_cursor,
    }))
}

async fn get_one(
    state: AppState,
    ctx: RequestContext,
    r: &'static Resource,
    id: Uuid,
    headers: HeaderMap,
) -> ApiResult<Response> {
    require_read(&ctx)?;
    let ws = ws_of(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    let (body, version) = record(&mut tx, r, ws, id, false).await?;
    tx.commit().await?;
    Ok(conditional_get(&headers, version, &body))
}

// --- writing --------------------------------------------------------------------------------

async fn write_history(
    tx: &mut PgConnection,
    ws: Uuid,
    table: &str,
    record: Uuid,
    version: i64,
    previous: &Map<String, Value>,
    user: Uuid,
) -> ApiResult<()> {
    sqlx::query(
        "INSERT INTO change_history \
             (id, workspace_id, table_name, record_id, record_version, previous_value, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(Uuid::now_v7())
    .bind(ws)
    .bind(table)
    .bind(record)
    .bind(version)
    .bind(Value::Object(previous.clone()))
    .bind(user)
    .execute(tx)
    .await?;
    Ok(())
}

/// Column name of a key, from the static field tables (never from request text).
fn static_col(r: &Resource, key: &str) -> Option<&'static str> {
    if key == "deleted_at" {
        return Some("deleted_at");
    }
    r.fields.iter().find(|f| f.name == key).map(|f| f.name)
}

/// Validates the `steps` array of a request.
fn check_steps(r: &Resource, v: &Value, issues: &mut Vec<FieldIssue>) {
    let Some(steps) = &r.steps else { return };
    let Some(list) = v.as_array() else {
        issues.push(FieldIssue::new("/steps", "invalid_type"));
        return;
    };
    if list.len() > MAX_STEPS {
        issues.push(
            FieldIssue::new("/steps", "too_many_items").with_params(json!({"max": MAX_STEPS})),
        );
        return;
    }
    for (i, item) in list.iter().enumerate() {
        match item.as_object() {
            Some(m) => check_object(steps.fields, m, &format!("/steps/{i}"), true, &[], issues),
            None => issues.push(FieldIssue::new(format!("/steps/{i}"), "invalid_type")),
        }
    }
}

fn check_body(r: &Resource, body: &Map<String, Value>, creating: bool) -> ApiResult<()> {
    let mut issues = Vec::new();
    let skip: &[&str] = if r.steps.is_some() { &["steps"] } else { &[] };
    check_object(r.fields, body, "", creating, skip, &mut issues);
    if let Some(v) = body.get("steps") {
        check_steps(r, v, &mut issues);
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::validation(issues))
    }
}

async fn insert_steps(
    tx: &mut PgConnection,
    r: &Resource,
    ws: Uuid,
    profile: Uuid,
    user: Uuid,
    list: &[Value],
) -> ApiResult<()> {
    let Some(steps) = &r.steps else {
        return Ok(());
    };
    for (position, item) in (0_i32..).zip(list) {
        let obj = item.as_object().cloned().unwrap_or_default();
        let cols: Vec<&'static str> = steps
            .fields
            .iter()
            .map(|f| f.name)
            .filter(|n| obj.contains_key(*n))
            .collect();
        let names: String = cols.iter().map(|c| format!(", {c}")).collect();
        let picks: String = cols.iter().map(|c| format!(", x.{c}")).collect();
        sqlx::query(sql!(
            "INSERT INTO {t} (id, workspace_id, profile_id, position, created_by, updated_by{names}) \
             SELECT $1, $2, $3, $4, $5, $5{picks} FROM jsonb_populate_record(NULL::{t}, $6::jsonb) x",
            t = steps.table
        ))
        .bind(Uuid::now_v7())
        .bind(ws)
        .bind(profile)
        .bind(position)
        .bind(user)
        .bind(Value::Object(obj))
        .execute(&mut *tx)
        .await
        .map_err(|e| map_db(Some(r), e))?;
    }
    Ok(())
}

/// Clears `is_default` of every other record (its own history row records it).
async fn demote_others(
    tx: &mut PgConnection,
    r: &Resource,
    ws: Uuid,
    keep: Uuid,
    user: Uuid,
) -> ApiResult<()> {
    let ids: Vec<Uuid> = sqlx::query_scalar(sql!(
        "SELECT id FROM {} WHERE workspace_id = $1 AND is_default AND deleted_at IS NULL \
           AND id <> $2 FOR UPDATE",
        r.table
    ))
    .bind(ws)
    .bind(keep)
    .fetch_all(&mut *tx)
    .await?;
    for id in ids {
        let version: i64 = sqlx::query_scalar(sql!(
            "UPDATE {} SET is_default = false, updated_by = $3 \
             WHERE workspace_id = $1 AND id = $2 RETURNING version",
            r.table
        ))
        .bind(ws)
        .bind(id)
        .bind(user)
        .fetch_one(&mut *tx)
        .await?;
        let mut prev = Map::new();
        prev.insert("is_default".to_owned(), json!(true));
        write_history(tx, ws, r.table, id, version, &prev, user).await?;
    }
    Ok(())
}

/// Inserts a validated record and returns its id.
async fn insert_record(
    tx: &mut PgConnection,
    r: &Resource,
    ws: Uuid,
    user: Uuid,
    mut body: Map<String, Value>,
) -> ApiResult<Uuid> {
    let id = Uuid::now_v7();
    if r.has_default {
        if body.get("is_default") == Some(&json!(true)) {
            demote_others(tx, r, ws, id, user).await?;
        } else if !body.contains_key("is_default") {
            let any: bool = sqlx::query_scalar(sql!(
                "SELECT EXISTS (SELECT 1 FROM {} WHERE workspace_id = $1 AND is_default \
                 AND deleted_at IS NULL)",
                r.table
            ))
            .bind(ws)
            .fetch_one(&mut *tx)
            .await?;
            if !any {
                body.insert("is_default".to_owned(), json!(true));
            }
        }
    }
    let cols: Vec<&'static str> = r
        .fields
        .iter()
        .map(|f| f.name)
        .filter(|n| body.contains_key(*n))
        .collect();
    let names: String = cols.iter().map(|c| format!(", {c}")).collect();
    let picks: String = cols.iter().map(|c| format!(", x.{c}")).collect();
    sqlx::query(sql!(
        "INSERT INTO {t} (id, workspace_id, created_by, updated_by{names}) \
         SELECT $1, $2, $3, $3{picks} FROM jsonb_populate_record(NULL::{t}, $4::jsonb) x",
        t = r.table
    ))
    .bind(id)
    .bind(ws)
    .bind(user)
    .bind(Value::Object(body.clone()))
    .execute(&mut *tx)
    .await
    .map_err(|e| map_db(Some(r), e))?;
    if let Some(Value::Array(list)) = body.get("steps") {
        insert_steps(tx, r, ws, id, user, list).await?;
    }
    Ok(id)
}

async fn create(
    state: AppState,
    ctx: RequestContext,
    r: &'static Resource,
    body: Value,
) -> ApiResult<Response> {
    require_write(&ctx)?;
    let ws = ws_of(&ctx)?;
    let body = object(body)?;
    check_body(r, &body, true)?;
    let mut tx = ctx.begin(&state.db).await?;
    let id = insert_record(&mut tx, r, ws, ctx.identity.user_id, body).await?;
    let (rec, version) = record(&mut tx, r, ws, id, false).await?;
    tx.commit().await?;
    Ok(record_response(StatusCode::CREATED, version, &rec))
}

/// The result of [`apply_changes`].
struct Applied {
    record: Value,
    version: i64,
}

/// Writes `changes` (a merge patch) onto one record: last writer wins for each field. Only
/// values that differ are written; when nothing differs the record stays as it is (no new
/// version, no history row).
async fn apply_changes(
    tx: &mut PgConnection,
    r: &Resource,
    ws: Uuid,
    id: Uuid,
    user: Uuid,
    headers: Option<&HeaderMap>,
    changes: &Map<String, Value>,
    deleted_ok: bool,
) -> ApiResult<Applied> {
    let row = load_row(tx, r, ws, id, deleted_ok, true).await?;
    let steps = load_steps(tx, r, ws, &[id]).await?.remove(&id);
    let current = project(r, &row, steps);
    let version = version_of(&row);
    if let Some(h) = headers {
        check_if_match(Concurrency::FieldLww, h, version, || current.clone())?;
    }
    let mut previous = Map::new();
    let mut diff = Map::new();
    for (key, new) in changes {
        let old = if key == "steps" && r.steps.is_some() {
            current.get("steps").cloned().unwrap_or(Value::Null)
        } else if static_col(r, key).is_some() {
            row.get(key).cloned().unwrap_or(Value::Null)
        } else {
            return Err(ApiError::validation(vec![FieldIssue::new(
                format!("/{key}"),
                "unknown_field",
            )]));
        };
        if !json_eq(&old, new) {
            previous.insert(key.clone(), old);
            diff.insert(key.clone(), new.clone());
        }
    }
    if diff.is_empty() {
        return Ok(Applied {
            record: current,
            version,
        });
    }
    if r.has_default && diff.get("is_default") == Some(&json!(true)) {
        demote_others(tx, r, ws, id, user).await?;
    }
    let sets: String = diff
        .keys()
        .filter_map(|k| static_col(r, k))
        .map(|c| {
            format!(
                "{c} = (jsonb_populate_record(NULL::{t}, $3::jsonb)).{c}, ",
                t = r.table
            )
        })
        .collect();
    let new_version: i64 = sqlx::query_scalar(sql!(
        "UPDATE {} SET {sets}updated_by = $4 WHERE workspace_id = $1 AND id = $2 RETURNING version",
        r.table
    ))
    .bind(ws)
    .bind(id)
    .bind(Value::Object(diff.clone()))
    .bind(user)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| map_db(Some(r), e))?;
    if let Some(Value::Array(list)) = diff.get("steps")
        && let Some(steps) = &r.steps
    {
        sqlx::query(sql!(
            "DELETE FROM {} WHERE workspace_id = $1 AND profile_id = $2",
            steps.table
        ))
        .bind(ws)
        .bind(id)
        .execute(&mut *tx)
        .await?;
        insert_steps(tx, r, ws, id, user, list).await?;
    }
    write_history(tx, ws, r.table, id, new_version, &previous, user).await?;
    let (record, version) = record(tx, r, ws, id, true).await?;
    Ok(Applied { record, version })
}

async fn update(
    state: AppState,
    ctx: RequestContext,
    r: &'static Resource,
    id: Uuid,
    headers: HeaderMap,
    body: Value,
) -> ApiResult<Response> {
    require_write(&ctx)?;
    let ws = ws_of(&ctx)?;
    let body = object(body)?;
    check_body(r, &body, false)?;
    let mut tx = ctx.begin(&state.db).await?;
    let done = apply_changes(
        &mut tx,
        r,
        ws,
        id,
        ctx.identity.user_id,
        Some(&headers),
        &body,
        false,
    )
    .await?;
    tx.commit().await?;
    Ok(record_response(StatusCode::OK, done.version, &done.record))
}

async fn remove(
    state: AppState,
    ctx: RequestContext,
    r: &'static Resource,
    id: Uuid,
    headers: HeaderMap,
) -> ApiResult<Response> {
    require_write(&ctx)?;
    let ws = ws_of(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    let mut changes = Map::new();
    changes.insert("deleted_at".to_owned(), json!(Utc::now().to_rfc3339()));
    if r.has_default {
        changes.insert("is_default".to_owned(), json!(false));
    }
    apply_changes(
        &mut tx,
        r,
        ws,
        id,
        ctx.identity.user_id,
        Some(&headers),
        &changes,
        false,
    )
    .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response_compat())
}

trait IntoResponseCompat {
    fn into_response_compat(self) -> Response;
}

impl IntoResponseCompat for StatusCode {
    fn into_response_compat(self) -> Response {
        axum::response::IntoResponse::into_response(self)
    }
}

// --- history --------------------------------------------------------------------------------

/// One page of the history of a record, newest first.
async fn history_page(
    tx: &mut PgConnection,
    ws: Uuid,
    table: &str,
    record: Option<Uuid>,
    params: &ListParams,
) -> ApiResult<Value> {
    let sort = SortSpec {
        field: "record_version",
        descending: true,
    };
    let limit = params.limit()?;
    let cursor = params.cursor(&sort)?;
    let (ckey, cid) = cursor.map_or((None, None), |c| (c.key.as_i64(), Some(c.id)));
    let rows = match record {
        None => Vec::new(),
        Some(record) => {
            sqlx::query_as::<_, (Uuid, i64, Value, DateTime<Utc>, Option<Uuid>)>(
                "SELECT id, record_version, previous_value, created_at, created_by \
                 FROM change_history \
                 WHERE workspace_id = $1 AND table_name = $2 AND record_id = $3 \
                   AND ($4::bigint IS NULL OR (record_version, id) < ($4, $5::uuid)) \
                 ORDER BY record_version DESC, id DESC LIMIT $6",
            )
            .bind(ws)
            .bind(table)
            .bind(record)
            .bind(ckey)
            .bind(cid)
            .bind(i64::from(limit) + 1)
            .fetch_all(tx)
            .await?
        }
    };
    let page = Page::from_rows(rows, limit, &sort, |row| Cursor {
        key: json!(row.1),
        id: row.0,
    });
    let items: Vec<Value> = page
        .items
        .iter()
        .map(|(id, version, prev, at, by)| {
            json!({
                "id": id, "record_version": version, "previous_value": prev,
                "created_at": at, "created_by": by
            })
        })
        .collect();
    Ok(json!({"items": items, "next_cursor": page.next_cursor}))
}

async fn history(
    state: AppState,
    ctx: RequestContext,
    r: &'static Resource,
    id: Uuid,
    params: ListParams,
) -> ApiResult<Json<Value>> {
    require_read(&ctx)?;
    let ws = ws_of(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    load_row(&mut tx, r, ws, id, true, false).await?;
    let page = history_page(&mut tx, ws, r.table, Some(id), &params).await?;
    tx.commit().await?;
    Ok(Json(page))
}

/// The replaced values of a history row of `record`.
async fn history_values(
    tx: &mut PgConnection,
    ws: Uuid,
    table: &str,
    record: Uuid,
    hid: Uuid,
) -> ApiResult<Map<String, Value>> {
    let value: Value = sqlx::query_scalar(
        "SELECT previous_value FROM change_history \
         WHERE workspace_id = $1 AND id = $2 AND table_name = $3 AND record_id = $4",
    )
    .bind(ws)
    .bind(hid)
    .bind(table)
    .bind(record)
    .fetch_optional(tx)
    .await?
    .ok_or_else(not_found)?;
    object(value)
}

async fn restore(
    state: AppState,
    ctx: RequestContext,
    r: &'static Resource,
    id: Uuid,
    hid: Uuid,
) -> ApiResult<Response> {
    require_write(&ctx)?;
    let ws = ws_of(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    let values = history_values(&mut tx, ws, r.table, id, hid).await?;
    let done = apply_changes(
        &mut tx,
        r,
        ws,
        id,
        ctx.identity.user_id,
        None,
        &values,
        true,
    )
    .await?;
    tx.commit().await?;
    Ok(record_response(StatusCode::OK, done.version, &done.record))
}

// --- workspace and user settings -------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    Workspace,
    User,
}

impl Scope {
    const fn table(self) -> &'static str {
        match self {
            Self::Workspace => "workspaces",
            Self::User => "user_settings",
        }
    }

    fn require_write(self, ctx: &RequestContext) -> ApiResult<()> {
        match self {
            Self::Workspace => ctx.require_permission(Permission::WorkspaceManage),
            Self::User => Ok(()),
        }
    }
}

struct Loaded {
    /// Record id for the history (the workspace, or the `user_settings` row); `None` before the
    /// first user write.
    id: Option<Uuid>,
    settings: Map<String, Value>,
    version: i64,
    updated_at: Option<DateTime<Utc>>,
}

impl Loaded {
    fn body(&self) -> Value {
        let mut settings = self.settings.clone();
        settings.remove("schema_version");
        json!({"settings": settings, "version": self.version, "updated_at": self.updated_at})
    }
}

async fn load_settings(
    tx: &mut PgConnection,
    scope: Scope,
    ws: Uuid,
    user: Uuid,
    lock: bool,
) -> ApiResult<Loaded> {
    let lock = if lock { "FOR UPDATE" } else { "" };
    let row = match scope {
        Scope::Workspace => {
            sqlx::query_as::<_, (Uuid, Value, i64, DateTime<Utc>)>(sql!(
                "SELECT id, settings, version, updated_at FROM workspaces WHERE id = $1 {lock}"
            ))
            .bind(ws)
            .fetch_optional(tx)
            .await?
        }
        Scope::User => {
            sqlx::query_as::<_, (Uuid, Value, i64, DateTime<Utc>)>(sql!(
                "SELECT id, settings, version, updated_at FROM user_settings \
             WHERE workspace_id = $1 AND user_id = $2 AND deleted_at IS NULL {lock}"
            ))
            .bind(ws)
            .bind(user)
            .fetch_optional(tx)
            .await?
        }
    };
    match row {
        Some((id, settings, version, at)) => Ok(Loaded {
            id: Some(id),
            settings: object(settings)?,
            version,
            updated_at: Some(at),
        }),
        None if scope == Scope::User => Ok(Loaded {
            id: None,
            settings: Map::new(),
            version: 0,
            updated_at: None,
        }),
        None => Err(not_found()),
    }
}

fn check_settings(body: &Map<String, Value>) -> ApiResult<()> {
    let mut issues = Vec::new();
    check_object(SETTING_KEYS, body, "", false, &[], &mut issues);
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ApiError::validation(issues))
    }
}

async fn settings_get(
    state: AppState,
    ctx: RequestContext,
    scope: Scope,
    headers: HeaderMap,
) -> ApiResult<Response> {
    let ws = ws_of(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    let loaded = load_settings(&mut tx, scope, ws, ctx.identity.user_id, false).await?;
    tx.commit().await?;
    Ok(conditional_get(&headers, loaded.version, &loaded.body()))
}

/// Merges `changes` into the settings (a `null` removes the key); last writer wins per key.
async fn apply_settings(
    tx: &mut PgConnection,
    scope: Scope,
    ws: Uuid,
    user: Uuid,
    headers: Option<&HeaderMap>,
    changes: &Map<String, Value>,
) -> ApiResult<Loaded> {
    let loaded = load_settings(tx, scope, ws, user, true).await?;
    if let Some(h) = headers {
        check_if_match(Concurrency::FieldLww, h, loaded.version, || loaded.body())?;
    }
    let mut previous = Map::new();
    let mut settings = loaded.settings.clone();
    for (key, new) in changes {
        if !SETTING_KEYS.iter().any(|f| f.name == key) {
            return Err(ApiError::validation(vec![FieldIssue::new(
                format!("/{key}"),
                "unknown_field",
            )]));
        }
        let old = settings.get(key).cloned().unwrap_or(Value::Null);
        if json_eq(&old, new) {
            continue;
        }
        previous.insert(key.clone(), old);
        if new.is_null() {
            settings.remove(key);
        } else {
            settings.insert(key.clone(), new.clone());
        }
    }
    if previous.is_empty() {
        return Ok(loaded);
    }
    settings.entry("schema_version").or_insert_with(|| json!(1));
    let value = Value::Object(settings.clone());
    let (id, version, updated_at): (Uuid, i64, DateTime<Utc>) = match (scope, loaded.id) {
        (Scope::Workspace, _) => {
            sqlx::query_as(
                "UPDATE workspaces SET settings = $2, updated_by = $3 WHERE id = $1 \
                 RETURNING id, version, updated_at",
            )
            .bind(ws)
            .bind(&value)
            .bind(user)
            .fetch_one(&mut *tx)
            .await?
        }
        (Scope::User, Some(id)) => {
            sqlx::query_as(
                "UPDATE user_settings SET settings = $2, updated_by = $3 WHERE id = $1 \
                 RETURNING id, version, updated_at",
            )
            .bind(id)
            .bind(&value)
            .bind(user)
            .fetch_one(&mut *tx)
            .await?
        }
        (Scope::User, None) => {
            sqlx::query_as(
                "INSERT INTO user_settings (id, workspace_id, user_id, settings, created_by, updated_by) \
                 VALUES ($1, $2, $3, $4, $3, $3) RETURNING id, version, updated_at",
            )
            .bind(Uuid::now_v7())
            .bind(ws)
            .bind(user)
            .bind(&value)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| map_db(None, e))?
        }
    };
    write_history(tx, ws, scope.table(), id, version, &previous, user).await?;
    Ok(Loaded {
        id: Some(id),
        settings,
        version,
        updated_at: Some(updated_at),
    })
}

async fn settings_update(
    state: AppState,
    ctx: RequestContext,
    scope: Scope,
    headers: HeaderMap,
    body: Value,
) -> ApiResult<Response> {
    scope.require_write(&ctx)?;
    let ws = ws_of(&ctx)?;
    let body = object(body)?;
    check_settings(&body)?;
    let mut tx = ctx.begin(&state.db).await?;
    let done = apply_settings(
        &mut tx,
        scope,
        ws,
        ctx.identity.user_id,
        Some(&headers),
        &body,
    )
    .await?;
    tx.commit().await?;
    Ok(record_response(StatusCode::OK, done.version, &done.body()))
}

async fn settings_history(
    state: AppState,
    ctx: RequestContext,
    scope: Scope,
    params: ListParams,
) -> ApiResult<Json<Value>> {
    let ws = ws_of(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    let loaded = load_settings(&mut tx, scope, ws, ctx.identity.user_id, false).await?;
    let page = history_page(&mut tx, ws, scope.table(), loaded.id, &params).await?;
    tx.commit().await?;
    Ok(Json(page))
}

async fn settings_restore(
    state: AppState,
    ctx: RequestContext,
    scope: Scope,
    hid: Uuid,
) -> ApiResult<Response> {
    scope.require_write(&ctx)?;
    let ws = ws_of(&ctx)?;
    let user = ctx.identity.user_id;
    let mut tx = ctx.begin(&state.db).await?;
    let loaded = load_settings(&mut tx, scope, ws, user, false).await?;
    let record = loaded.id.ok_or_else(not_found)?;
    let values = history_values(&mut tx, ws, scope.table(), record, hid).await?;
    let done = apply_settings(&mut tx, scope, ws, user, None, &values).await?;
    tx.commit().await?;
    Ok(record_response(StatusCode::OK, done.version, &done.body()))
}
