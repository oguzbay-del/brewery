//! Support requests of the customer (T2.29): open a request, write messages, edit or retract a
//! message, mark a request as resolved.
//!
//! The record is here; the support team works in GitHub Issues. Each change writes the outbox
//! event `support.sync_requested`, and the worker syncs (`support` crate). If GitHub is not
//! available, the request is open anyway and syncs later. The customer sees only the messages
//! with visibility `customer`; an internal note never shows. A caller sees only the requests that
//! the caller opened. Attachments use the file endpoints with `record_kind` `support_message`.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{FromRow, PgConnection};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    context::RequestContext,
    error::{ApiError, ApiJson, ApiQuery, ApiResult, ErrorCode},
    library::{created_response, id_cursor, issue, not_found, paging, sqlstate, workspace},
    pagination::{ListParams, Page},
};

/// Topics of a request.
pub const TOPICS: &[&str] = &["account", "subscription", "recipes", "device", "other"];
/// Longest message, in characters.
const MAX_BODY: usize = 20_000;

/// A support request.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct SupportRequest {
    /// Identifier.
    pub id: Uuid,
    /// Number for people (`#4182`), the same in all workspaces.
    pub public_number: i64,
    /// `account`, `subscription`, `recipes`, `device` or `other`.
    pub topic: String,
    /// Title.
    pub title: String,
    /// `open`, `answered`, `resolved` or `closed`.
    pub status: String,
    /// Kind of the related record (`recipe_version`, `batch`).
    pub related_record_kind: Option<String>,
    /// The related record.
    pub related_record_id: Option<Uuid>,
    /// The closed request that this one continues.
    pub previous_request_id: Option<Uuid>,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Updated at.
    pub updated_at: DateTime<Utc>,
    /// Record version.
    pub version: i64,
}

const REQUEST_COLUMNS: &str = "id, public_number, topic, title, status, related_record_kind, \
    related_record_id, previous_request_id, created_at, updated_at, version";

/// A message of a request.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct SupportMessage {
    /// Identifier.
    pub id: Uuid,
    /// `customer` or `staff` (the system and the support team show as `staff`).
    pub author: String,
    /// The text; empty after the message was retracted.
    pub body: String,
    /// The message was retracted.
    pub retracted: bool,
    /// The message was edited at this time.
    pub edited_at: Option<DateTime<Utc>>,
    /// Number of attachments.
    pub attachment_count: i64,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Record version.
    pub version: i64,
}

const MESSAGE_COLUMNS: &str = "m.id, CASE WHEN m.author_kind = 'customer' THEN 'customer' \
    ELSE 'staff' END AS author, CASE WHEN m.retracted_at IS NULL THEN m.body ELSE '' END AS body, \
    m.retracted_at IS NOT NULL AS retracted, m.edited_at, \
    (SELECT count(*) FROM attachments a WHERE a.workspace_id = m.workspace_id \
       AND a.record_kind = 'support_message' AND a.record_id = m.id \
       AND a.deleted_at IS NULL) AS attachment_count, m.created_at, m.version";

/// A request with its messages.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RequestDetail {
    /// The request.
    pub request: SupportRequest,
    /// The messages the customer may see, oldest first.
    pub messages: Vec<SupportMessage>,
}

/// A new request.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NewRequest {
    /// Topic.
    pub topic: String,
    /// Title, 1..=200 characters.
    pub title: String,
    /// First message, 1..=20000 characters.
    pub body: String,
    /// Kind of a related record.
    pub related_record_kind: Option<String>,
    /// A related record.
    pub related_record_id: Option<Uuid>,
    /// Diagnostic summary (app version and the like), without personal data.
    #[schema(value_type = Option<Object>)]
    pub diagnostics: Option<Value>,
    /// A closed request that this one continues.
    pub previous_request_id: Option<Uuid>,
}

/// A message text.
#[derive(Debug, Deserialize, ToSchema)]
pub struct MessageBody {
    /// Text, 1..=20000 characters.
    pub body: String,
}

/// Filter of the list.
#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct RequestFilter {
    /// `open`, `answered`, `resolved` or `closed`.
    pub status: Option<String>,
}

fn check_body(field: &str, body: &str) -> ApiResult<()> {
    let n = body.trim().chars().count();
    if n == 0 || body.chars().count() > MAX_BODY {
        return Err(issue(field, "length_out_of_range"));
    }
    Ok(())
}

async fn own_request(tx: &mut PgConnection, ctx: &RequestContext, id: Uuid) -> ApiResult<(String, Uuid)> {
    sqlx::query_as(
        "SELECT status, workspace_id FROM support_requests
         WHERE id = $1 AND created_by = $2 AND deleted_at IS NULL FOR UPDATE",
    )
    .bind(id)
    .bind(ctx.identity.user_id)
    .fetch_optional(tx)
    .await?
    .ok_or_else(not_found)
}

async fn sync(tx: &mut PgConnection, ws: Uuid, request: Uuid) -> ApiResult<()> {
    support::reply::enqueue_sync(tx, ws, request)
        .await
        .map_err(|e| ApiError::from(db::Error::from(sqlx::Error::Protocol(e.to_string()))))
}

fn request_example() -> Value {
    json!({"id": "0192f1c07a3b7000800000000000000b", "public_number": 4182, "topic": "recipes",
        "title": "Cannot publish a version", "status": "open", "related_record_kind": null,
        "related_record_id": null, "previous_request_id": null,
        "created_at": "2026-11-28T12:00:00Z", "updated_at": "2026-11-28T12:00:00Z", "version": 1})
}

fn message_example() -> Value {
    json!({"id": "0192f1c07a3b7000800000000000000c", "author": "customer",
        "body": "The publish button does nothing.", "retracted": false, "edited_at": null,
        "attachment_count": 0, "created_at": "2026-11-28T12:00:00Z", "version": 1})
}

fn detail_example() -> Value {
    json!({"request": request_example(), "messages": [message_example()]})
}

/// Opens a support request with its first message.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/support/requests",
    tag = "support",
    operation_id = "supportrequest_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = NewRequest,
    responses((status = 201, description = "The request with its first message",
        body = RequestDetail, example = json!(detail_example())))
)]
pub async fn supportrequest_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(new): ApiJson<NewRequest>,
) -> ApiResult<Response> {
    let ws = workspace(&ctx)?;
    if !TOPICS.contains(&new.topic.as_str()) {
        return Err(issue("/topic", "invalid_value"));
    }
    if !(1..=200).contains(&new.title.trim().chars().count()) {
        return Err(issue("/title", "length_out_of_range"));
    }
    check_body("/body", &new.body)?;
    if new.related_record_kind.is_some() != new.related_record_id.is_some()
        || new.related_record_kind.as_deref().is_some_and(|k| {
            k.is_empty() || !k.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
        })
    {
        return Err(issue("/related_record_kind", "invalid_value"));
    }
    let diagnostics = match new.diagnostics {
        None => None,
        Some(Value::Object(mut map)) => {
            map.entry("schema_version").or_insert(json!(1));
            Some(Value::Object(map))
        }
        Some(_) => return Err(issue("/diagnostics", "invalid_type")),
    };
    let user = ctx.identity.user_id;
    let mut tx = ctx.begin(&state.db).await?;
    if let Some(prev) = new.previous_request_id {
        let ok: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM support_requests WHERE id = $1 AND created_by = $2 AND status = 'closed'",
        )
        .bind(prev)
        .bind(user)
        .fetch_optional(&mut *tx)
        .await?;
        if ok.is_none() {
            return Err(issue("/previous_request_id", "not_found"));
        }
    }
    let id = Uuid::now_v7();
    let sql = sqlx::AssertSqlSafe(format!(
        "INSERT INTO support_requests (id, workspace_id, topic, title, related_record_kind,
             related_record_id, diagnostics, previous_request_id, created_by, updated_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9) RETURNING {REQUEST_COLUMNS}"
    ));
    let request: SupportRequest = sqlx::query_as(sql)
        .bind(id)
        .bind(ws)
        .bind(&new.topic)
        .bind(new.title.trim())
        .bind(&new.related_record_kind)
        .bind(new.related_record_id)
        .bind(&diagnostics)
        .bind(new.previous_request_id)
        .bind(user)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            if sqlstate(&e).as_deref() == Some("23503") {
                issue("/previous_request_id", "not_found")
            } else {
                e.into()
            }
        })?;
    let message = insert_message(&mut tx, ws, id, user, &new.body).await?;
    sync(&mut tx, ws, id).await?;
    tx.commit().await?;
    let location = format!("/v1/workspaces/{ws}/support/requests/{id}");
    Ok(created_response(
        &location,
        request.version,
        &RequestDetail {
            request,
            messages: vec![message],
        },
    ))
}

async fn insert_message(
    tx: &mut PgConnection,
    ws: Uuid,
    request: Uuid,
    user: Uuid,
    body: &str,
) -> ApiResult<SupportMessage> {
    let sql = sqlx::AssertSqlSafe(format!(
        "WITH m AS (INSERT INTO support_messages
             (id, workspace_id, request_id, author_kind, author_user_id, body, created_by, updated_by)
             VALUES ($1, $2, $3, 'customer', $4, $5, $4, $4) RETURNING *)
         SELECT {MESSAGE_COLUMNS} FROM m"
    ));
    Ok(sqlx::query_as(sql)
        .bind(Uuid::now_v7())
        .bind(ws)
        .bind(request)
        .bind(user)
        .bind(body)
        .fetch_one(tx)
        .await?)
}

/// The requests that the caller opened, newest first.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/support/requests",
    tag = "support",
    operation_id = "supportrequest_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("status" = Option<String>, Query, description = "Status filter"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page")
    ),
    responses((status = 200, description = "A page of requests", body = Page<SupportRequest>,
        example = json!({"items": [request_example()], "next_cursor": null})))
)]
pub async fn supportrequest_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<RequestFilter>,
) -> ApiResult<Json<Page<SupportRequest>>> {
    let paging = paging(&params, &["id"], "-id")?;
    if filter
        .status
        .as_deref()
        .is_some_and(|s| !["open", "answered", "resolved", "closed"].contains(&s))
    {
        return Err(issue("status", "invalid_value"));
    }
    let (cmp, dir) = paging.dir();
    let sql = sqlx::AssertSqlSafe(format!(
        "SELECT {REQUEST_COLUMNS} FROM support_requests
         WHERE created_by = $1 AND deleted_at IS NULL AND ($2::text IS NULL OR status = $2)
           AND ($3::uuid IS NULL OR id {cmp} $3) ORDER BY id {dir} LIMIT $4"
    ));
    let mut tx = ctx.begin(&state.db).await?;
    let rows: Vec<SupportRequest> = sqlx::query_as(sql)
        .bind(ctx.identity.user_id)
        .bind(&filter.status)
        .bind(paging.id)
        .bind(i64::from(paging.limit) + 1)
        .fetch_all(&mut *tx)
        .await?;
    Ok(Json(Page::from_rows(rows, paging.limit, &paging.sort, |r| {
        id_cursor(r.id)
    })))
}

async fn detail(tx: &mut PgConnection, ctx: &RequestContext, id: Uuid) -> ApiResult<RequestDetail> {
    let sql = sqlx::AssertSqlSafe(format!(
        "SELECT {REQUEST_COLUMNS} FROM support_requests
         WHERE id = $1 AND created_by = $2 AND deleted_at IS NULL"
    ));
    let request: SupportRequest = sqlx::query_as(sql)
        .bind(id)
        .bind(ctx.identity.user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(not_found)?;
    let sql = sqlx::AssertSqlSafe(format!(
        "SELECT {MESSAGE_COLUMNS} FROM support_messages m
         WHERE m.request_id = $1 AND m.visibility = 'customer' AND m.deleted_at IS NULL
         ORDER BY m.created_at, m.id"
    ));
    let messages = sqlx::query_as(sql).bind(id).fetch_all(tx).await?;
    Ok(RequestDetail { request, messages })
}

/// A request with its messages.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/support/requests/{id}",
    tag = "support",
    operation_id = "supportrequest_get",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Request")
    ),
    responses((status = 200, description = "The request", body = RequestDetail,
        example = json!(detail_example())))
)]
pub async fn supportrequest_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<RequestDetail>> {
    let mut tx = ctx.begin(&state.db).await?;
    Ok(Json(detail(&mut tx, &ctx, id).await?))
}

/// The customer marks a request as resolved. The Issue closes.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/support/requests/{id}/resolve",
    tag = "support",
    operation_id = "supportrequest_resolve",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Request")
    ),
    responses((status = 200, description = "The request", body = RequestDetail,
        example = json!(detail_example())))
)]
pub async fn supportrequest_resolve(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<RequestDetail>> {
    let mut tx = ctx.begin(&state.db).await?;
    let (status, _) = own_request(&mut tx, &ctx, id).await?;
    if !["open", "answered"].contains(&status.as_str()) {
        return Err(ApiError::new(ErrorCode::Conflict));
    }
    sqlx::query(
        "UPDATE support_requests SET status = 'resolved', resolved_at = now(), updated_by = $2
         WHERE id = $1",
    )
    .bind(id)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await?;
    sync(&mut tx, ws, id).await?;
    let d = detail(&mut tx, &ctx, id).await?;
    tx.commit().await?;
    Ok(Json(d))
}

/// Adds a message. A request that was answered or resolved opens again. A closed request takes
/// no message: the customer opens a new request that continues the old one.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/support/requests/{id}/messages",
    tag = "support",
    operation_id = "supportmessage_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Request"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = MessageBody,
    responses((status = 201, description = "The message", body = SupportMessage,
        example = json!(message_example())))
)]
pub async fn supportmessage_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((ws, id)): Path<(Uuid, Uuid)>,
    ApiJson(new): ApiJson<MessageBody>,
) -> ApiResult<Response> {
    check_body("/body", &new.body)?;
    let mut tx = ctx.begin(&state.db).await?;
    let (status, _) = own_request(&mut tx, &ctx, id).await?;
    if status == "closed" {
        return Err(ApiError::new(ErrorCode::Conflict));
    }
    let message = insert_message(&mut tx, ws, id, ctx.identity.user_id, &new.body).await?;
    sqlx::query(
        "UPDATE support_requests SET status = 'open', resolved_at = NULL, updated_by = $2
         WHERE id = $1 AND status IN ('answered', 'resolved')",
    )
    .bind(id)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await?;
    sync(&mut tx, ws, id).await?;
    tx.commit().await?;
    let location = format!("/v1/workspaces/{ws}/support/requests/{id}");
    Ok(created_response(&location, message.version, &message))
}

async fn own_message(
    tx: &mut PgConnection,
    ctx: &RequestContext,
    request: Uuid,
    message: Uuid,
) -> ApiResult<()> {
    let ok: Option<i32> = sqlx::query_scalar(
        "SELECT 1 FROM support_messages WHERE id = $1 AND request_id = $2
           AND author_kind = 'customer' AND author_user_id = $3 AND retracted_at IS NULL
           AND deleted_at IS NULL FOR UPDATE",
    )
    .bind(message)
    .bind(request)
    .bind(ctx.identity.user_id)
    .fetch_optional(tx)
    .await?;
    ok.map(|_| ()).ok_or_else(not_found)
}

/// Edits a message of the caller. The comment in the Issue changes too.
#[utoipa::path(
    patch,
    path = "/v1/workspaces/{workspace_id}/support/requests/{id}/messages/{message_id}",
    tag = "support",
    operation_id = "supportmessage_update",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Request"),
        ("message_id" = Uuid, Path, description = "Message")
    ),
    request_body = MessageBody,
    responses((status = 200, description = "The message", body = SupportMessage,
        example = json!(message_example())))
)]
pub async fn supportmessage_update(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((ws, id, message)): Path<(Uuid, Uuid, Uuid)>,
    ApiJson(new): ApiJson<MessageBody>,
) -> ApiResult<Json<SupportMessage>> {
    check_body("/body", &new.body)?;
    let mut tx = ctx.begin(&state.db).await?;
    own_message(&mut tx, &ctx, id, message).await?;
    let sql = sqlx::AssertSqlSafe(format!(
        "WITH m AS (UPDATE support_messages SET body = $2, edited_at = now(), updated_by = $3
             WHERE id = $1 RETURNING *)
         SELECT {MESSAGE_COLUMNS} FROM m"
    ));
    let updated: SupportMessage = sqlx::query_as(sql)
        .bind(message)
        .bind(&new.body)
        .bind(ctx.identity.user_id)
        .fetch_one(&mut *tx)
        .await?;
    sync(&mut tx, ws, id).await?;
    tx.commit().await?;
    Ok(Json(updated))
}

/// Retracts a message of the caller. The comment in the Issue is deleted.
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/support/requests/{id}/messages/{message_id}",
    tag = "support",
    operation_id = "supportmessage_delete",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Request"),
        ("message_id" = Uuid, Path, description = "Message")
    ),
    responses((status = 204, description = "Retracted"))
)]
pub async fn supportmessage_delete(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((ws, id, message)): Path<(Uuid, Uuid, Uuid)>,
) -> ApiResult<Response> {
    let mut tx = ctx.begin(&state.db).await?;
    own_message(&mut tx, &ctx, id, message).await?;
    sqlx::query("UPDATE support_messages SET retracted_at = now(), updated_by = $2 WHERE id = $1")
        .bind(message)
        .bind(ctx.identity.user_id)
        .execute(&mut *tx)
        .await?;
    sync(&mut tx, ws, id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Routes of this module.
pub fn routes(state: &AppState) -> Router<AppState> {
    const BASE: &str = "/v1/workspaces/{ws}/support/requests";
    let creates = crate::idempotent(
        state,
        Router::new()
            .route(BASE, post(supportrequest_create))
            .route(&format!("{BASE}/{{id}}/messages"), post(supportmessage_create)),
    );
    let others = Router::new()
        .route(BASE, get(supportrequest_list))
        .route(&format!("{BASE}/{{id}}"), get(supportrequest_get))
        .route(
            &format!("{BASE}/{{id}}/resolve"),
            post(supportrequest_resolve),
        )
        .route(
            &format!("{BASE}/{{id}}/messages/{{message_id}}"),
            patch(supportmessage_update).delete(supportmessage_delete),
        );
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
            "supportrequest_create",
            None,
            Some(Concurrency::Append),
            CREATE,
        ),
        file_op("supportrequest_list", None, None, READ),
        file_op("supportrequest_get", None, None, READ),
        file_op(
            "supportrequest_resolve",
            None,
            Some(Concurrency::FieldLww),
            WRITE,
        ),
        file_op(
            "supportmessage_create",
            None,
            Some(Concurrency::Append),
            CREATE,
        ),
        file_op(
            "supportmessage_update",
            None,
            Some(Concurrency::FieldLww),
            WRITE,
        ),
        file_op(
            "supportmessage_delete",
            None,
            Some(Concurrency::FieldLww),
            WRITE,
        ),
    ];
}

/// `OpenAPI` fragment of this module.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(
        supportrequest_create,
        supportrequest_list,
        supportrequest_get,
        supportrequest_resolve,
        supportmessage_create,
        supportmessage_update,
        supportmessage_delete
    ),
    components(schemas(
        SupportRequest,
        SupportMessage,
        RequestDetail,
        NewRequest,
        MessageBody
    )),
    tags((name = "support", description = "Support requests of the customer"))
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "support",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
