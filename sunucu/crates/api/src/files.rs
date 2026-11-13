//! File endpoints: attachments and scanned documents (T2.18, ADR-0008 rule 11.1).
//!
//! Files never pass through the API. Creating a record returns a short lived presigned `PUT`
//! bound to the declared size, type and SHA-256; the client uploads straight to the object store
//! and then calls `complete`, which checks the stored object (size, type, checksum) before the
//! record counts as uploaded. Downloads are short lived presigned `GET`s. Object keys are made
//! by the server (`<workspace>/<kind>/<id>`), never taken from the client.
//!
//! Deleting is soft; the bytes stay in the store until the worker's cleanup removes them (a
//! restore from the trash needs them). A scanned document under legal hold cannot be deleted.

use std::{collections::BTreeMap, sync::Arc, time::Duration};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, Postgres, Transaction, postgres::PgRow};
use storage::{ObjectStore, PresignedRequest, StorageError, UploadLimits};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    concurrency::{self, Concurrency},
    context::RequestContext,
    error::{ApiError, ApiJson, ApiQuery, ApiResult, ErrorCode, FieldIssue},
    pagination::{Cursor, ListParams, Page, SortSpec},
};

/// Largest file (bytes) either endpoint accepts.
pub const MAX_FILE_BYTES: u64 = 25 * 1024 * 1024;
/// Types an attachment may have.
pub const ATTACHMENT_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/heic",
    "image/webp",
    "application/pdf",
    "text/plain",
];
/// Types a scanned document may have.
pub const DOCUMENT_TYPES: &[&str] = &["image/jpeg", "image/png", "image/heic", "application/pdf"];
/// Permission that creates and deletes files.
pub const WRITE_PERMISSION: &str = "batch.record";
/// Permission that sets retention and legal hold on a scanned document.
pub const HOLD_PERMISSION: &str = "workspace.manage";

/// Builds SQL from the constants of [`Spec`] and the fixed fragments in this file; nothing a
/// client sends ever reaches the text (values are bound).
macro_rules! dyn_sql {
    ($($t:tt)*) => {
        sqlx::AssertSqlSafe(format!($($t)*))
    };
}

/// Settings of one file table.
struct Spec {
    table: &'static str,
    /// Columns of the response row.
    columns: &'static str,
    /// Middle part of the object key.
    segment: &'static str,
    /// Text filter column (`$1` of the list query) and uuid filter column (`$2`).
    text_filter: &'static str,
    uuid_filter: &'static str,
}

const ATTACHMENTS: Spec = Spec {
    table: "attachments",
    columns: "id, kind, record_kind, record_id, content_type, size_bytes, \
              encode(sha256, 'hex') AS sha256, caption, uploaded_at, created_at, updated_at, \
              deleted_at, version",
    segment: "attachments",
    text_filter: "record_kind",
    uuid_filter: "record_id",
};

const DOCUMENTS: Spec = Spec {
    table: "scanned_documents",
    columns: "id, batch_id, form_type, content_type, size_bytes, \
              encode(sha256, 'hex') AS sha256, retention_until, legal_hold, uploaded_at, \
              created_at, updated_at, deleted_at, version",
    segment: "documents",
    text_filter: "form_type",
    uuid_filter: "batch_id",
};

// --- representations ----------------------------------------------------------------------

/// A file attached to a batch or tasting note.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct Attachment {
    /// Identifier.
    pub id: Uuid,
    /// `photo`, `note` or `file`.
    pub kind: String,
    /// `batch` or `tasting_note`.
    pub record_kind: String,
    /// The record it belongs to.
    pub record_id: Uuid,
    /// Media type.
    pub content_type: String,
    /// Size in bytes.
    pub size_bytes: i64,
    /// SHA-256, hex.
    pub sha256: String,
    /// Caption.
    pub caption: Option<String>,
    /// Set once the upload was completed; `null` while the upload is pending.
    pub uploaded_at: Option<DateTime<Utc>>,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Updated at.
    pub updated_at: DateTime<Utc>,
    /// Deleted at (soft delete).
    pub deleted_at: Option<DateTime<Utc>>,
    /// Record version (`ETag`).
    pub version: i64,
}

/// A scanned paper form (brew log, quality sheet).
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct ScannedDocument {
    /// Identifier.
    pub id: Uuid,
    /// Batch the form belongs to.
    pub batch_id: Option<Uuid>,
    /// Form type (`brew_log`, ...).
    pub form_type: String,
    /// Media type.
    pub content_type: String,
    /// Size in bytes.
    pub size_bytes: i64,
    /// SHA-256, hex.
    pub sha256: String,
    /// The document may not be deleted before this instant when under legal hold.
    pub retention_until: Option<DateTime<Utc>>,
    /// Legal hold: no deletion before `retention_until` (never when that is empty).
    pub legal_hold: bool,
    /// Set once the upload was completed; `null` while the upload is pending.
    pub uploaded_at: Option<DateTime<Utc>>,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Updated at.
    pub updated_at: DateTime<Utc>,
    /// Deleted at (soft delete).
    pub deleted_at: Option<DateTime<Utc>>,
    /// Record version (`ETag`).
    pub version: i64,
}

/// Rows the generic operations work on.
trait FileRow: for<'r> FromRow<'r, PgRow> + Serialize + Send + Unpin {
    fn version(&self) -> i64;
    fn id(&self) -> Uuid;
    fn created_at(&self) -> DateTime<Utc>;
}

macro_rules! file_row {
    ($t:ty) => {
        impl FileRow for $t {
            fn version(&self) -> i64 {
                self.version
            }
            fn id(&self) -> Uuid {
                self.id
            }
            fn created_at(&self) -> DateTime<Utc> {
                self.created_at
            }
        }
    };
}
file_row!(Attachment);
file_row!(ScannedDocument);

/// What the client does next: one HTTP request to the object store.
#[derive(Debug, Serialize, ToSchema)]
pub struct UploadInstructions {
    /// `PUT`.
    pub method: String,
    /// Presigned URL (signature in the query string).
    pub url: String,
    /// Headers to send unchanged with the body.
    pub headers: BTreeMap<String, String>,
    /// The URL stops working at this instant.
    pub expires_at: DateTime<Utc>,
}

impl From<PresignedRequest> for UploadInstructions {
    fn from(r: PresignedRequest) -> Self {
        Self {
            method: r.method,
            url: r.url,
            headers: r.headers.into_iter().collect(),
            expires_at: r.expires_at,
        }
    }
}

/// Response of `POST .../attachments`.
#[derive(Debug, Serialize, ToSchema)]
pub struct AttachmentCreated {
    /// The new (pending) attachment.
    pub attachment: Attachment,
    /// How to upload the bytes.
    pub upload: UploadInstructions,
}

/// Response of `POST .../scanned-documents`.
#[derive(Debug, Serialize, ToSchema)]
pub struct ScannedDocumentCreated {
    /// The new (pending) document.
    pub document: ScannedDocument,
    /// How to upload the bytes.
    pub upload: UploadInstructions,
}

/// A short lived download link.
#[derive(Debug, Serialize, ToSchema)]
pub struct DownloadLink {
    /// Presigned `GET` URL.
    pub url: String,
    /// The URL stops working at this instant.
    pub expires_at: DateTime<Utc>,
    /// Media type of the file.
    pub content_type: String,
    /// Size in bytes.
    pub size_bytes: i64,
}

/// Body of `POST .../attachments`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NewAttachment {
    /// `photo`, `note` or `file`.
    pub kind: String,
    /// `batch` or `tasting_note`.
    pub record_kind: String,
    /// The record it belongs to.
    pub record_id: Uuid,
    /// Media type of the file.
    pub content_type: String,
    /// Exact size in bytes.
    pub size_bytes: u64,
    /// SHA-256 of the file, hex (64 characters).
    pub sha256: String,
    /// Caption (up to 500 characters).
    pub caption: Option<String>,
}

/// Body of `POST .../scanned-documents`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NewScannedDocument {
    /// Batch the form belongs to.
    pub batch_id: Option<Uuid>,
    /// Form type, lower case letters, digits and `_`.
    pub form_type: String,
    /// Media type of the file.
    pub content_type: String,
    /// Exact size in bytes.
    pub size_bytes: u64,
    /// SHA-256 of the file, hex (64 characters).
    pub sha256: String,
    /// Retention end; needs `workspace.manage`.
    pub retention_until: Option<DateTime<Utc>>,
    /// Legal hold; needs `workspace.manage`.
    pub legal_hold: Option<bool>,
}

/// Filters of `GET .../attachments`.
#[derive(Debug, Default, Deserialize)]
pub struct AttachmentFilter {
    /// `batch` or `tasting_note`.
    pub record_kind: Option<String>,
    /// The record.
    pub record_id: Option<Uuid>,
}

/// Filters of `GET .../scanned-documents`.
#[derive(Debug, Default, Deserialize)]
pub struct DocumentFilter {
    /// Form type.
    pub form_type: Option<String>,
    /// Batch.
    pub batch_id: Option<Uuid>,
}

// --- shared helpers -----------------------------------------------------------------------

impl From<StorageError> for ApiError {
    fn from(e: StorageError) -> Self {
        match e {
            StorageError::Unavailable(detail) => {
                tracing::error!(%detail, "object store unavailable");
                Self::unavailable(5)
            }
            other => {
                tracing::error!(error = %other, "object store error");
                Self::new(ErrorCode::Internal)
            }
        }
    }
}

fn store(state: &AppState) -> ApiResult<&Arc<dyn ObjectStore>> {
    state.store.as_ref().ok_or_else(|| {
        tracing::warn!("file endpoint called but S3_ENDPOINT is not configured");
        ApiError::unavailable(30)
    })
}

fn ttl(state: &AppState) -> Duration {
    state
        .config
        .storage
        .as_ref()
        .map_or(Duration::from_mins(10), |s| s.presign_ttl)
}

fn workspace(ctx: &RequestContext) -> ApiResult<Uuid> {
    ctx.workspace_id
        .ok_or_else(|| ApiError::new(ErrorCode::NotFound))
}

fn issue(field: &str, code: &str) -> ApiError {
    ApiError::validation(vec![FieldIssue::new(field, code)])
}

fn media_type(raw: &str) -> String {
    raw.split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

/// Validates the declared file (type, size, checksum) and returns the SHA-256 bytes.
fn validate_file(
    content_type: &str,
    size_bytes: u64,
    sha256: &str,
    allowed: &[&str],
) -> ApiResult<[u8; 32]> {
    let mut issues = Vec::new();
    if !allowed.contains(&media_type(content_type).as_str())
        || content_type != media_type(content_type)
    {
        issues.push(
            FieldIssue::new("/content_type", "unsupported_type")
                .with_params(json!({"allowed": allowed})),
        );
    }
    if !(1..=MAX_FILE_BYTES).contains(&size_bytes) {
        issues.push(
            FieldIssue::new("/size_bytes", "out_of_range")
                .with_params(json!({"min": 1, "max": MAX_FILE_BYTES})),
        );
    }
    let digest = parse_sha256(sha256);
    if digest.is_none() {
        issues.push(FieldIssue::new("/sha256", "invalid_value"));
    }
    match (issues.is_empty(), digest) {
        (true, Some(d)) => Ok(d),
        _ => Err(ApiError::validation(issues)),
    }
}

fn parse_sha256(hex: &str) -> Option<[u8; 32]> {
    if hex.len() != 64 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let mut out = [0_u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(hex.get(i * 2..i * 2 + 2)?, 16).ok()?;
    }
    Some(out)
}

fn is_sqlstate(e: &sqlx::Error, code: &str) -> bool {
    e.as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .is_some_and(|c| c == code)
}

fn legal_hold_error() -> ApiError {
    ApiError::new(ErrorCode::Conflict)
        .with_errors(vec![FieldIssue::new("/legal_hold", "legal_hold")])
}

fn created_response<T: Serialize>(location: &str, version: i64, body: &T) -> Response {
    let mut response = concurrency::record_response(StatusCode::CREATED, version, body);
    if let Ok(value) = location.parse() {
        response.headers_mut().insert(header::LOCATION, value);
    }
    response
}

async fn fetch<T: FileRow>(
    tx: &mut Transaction<'static, Postgres>,
    spec: &Spec,
    id: Uuid,
) -> ApiResult<T> {
    sqlx::query_as::<_, T>(dyn_sql!(
        "SELECT {} FROM {} WHERE id = $1 AND deleted_at IS NULL",
        spec.columns,
        spec.table
    ))
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| ApiError::new(ErrorCode::NotFound))
}

/// The columns the object checks need.
#[derive(FromRow)]
struct Core {
    object_key: String,
    content_type: String,
    size_bytes: i64,
    sha256: String,
    uploaded_at: Option<DateTime<Utc>>,
}

async fn core(tx: &mut Transaction<'static, Postgres>, spec: &Spec, id: Uuid) -> ApiResult<Core> {
    sqlx::query_as::<_, Core>(dyn_sql!(
        "SELECT object_key, content_type, size_bytes, encode(sha256, 'hex') AS sha256, uploaded_at \
         FROM {} WHERE id = $1 AND deleted_at IS NULL",
        spec.table
    ))
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| ApiError::new(ErrorCode::NotFound))
}

fn object_key(spec: &Spec, workspace_id: Uuid, id: Uuid) -> String {
    format!("{workspace_id}/{}/{id}", spec.segment)
}

fn presign_upload(
    state: &AppState,
    key: &str,
    content_type: &str,
    size_bytes: u64,
    sha256: [u8; 32],
) -> ApiResult<UploadInstructions> {
    let limits = UploadLimits {
        content_type: content_type.to_owned(),
        size_bytes,
        sha256,
    };
    Ok(store(state)?.presign_put(key, &limits, ttl(state))?.into())
}

// --- generic operations -------------------------------------------------------------------

async fn get_one<T: FileRow>(
    state: &AppState,
    ctx: &RequestContext,
    spec: &Spec,
    id: Uuid,
    headers: &HeaderMap,
) -> ApiResult<Response> {
    let mut tx = ctx.begin(&state.db).await?;
    let row: T = fetch(&mut tx, spec, id).await?;
    Ok(concurrency::conditional_get(headers, row.version(), &row))
}

async fn list_rows<T: FileRow>(
    state: &AppState,
    ctx: &RequestContext,
    spec: &Spec,
    params: &ListParams,
    text: Option<&str>,
    id_filter: Option<Uuid>,
) -> ApiResult<Json<Page<T>>> {
    let limit = params.limit()?;
    let sort = SortSpec::parse(params.sort.as_deref(), &["created_at"], "-created_at")?;
    let cursor = params.cursor(&sort)?;
    let (cursor_at, cursor_id) = match cursor {
        Some(c) => {
            let at: DateTime<Utc> =
                c.key.as_str().and_then(|s| s.parse().ok()).ok_or_else(|| {
                    ApiError::new(ErrorCode::MalformedRequest)
                        .with_errors(vec![FieldIssue::new("cursor", "invalid_cursor")])
                })?;
            (Some(at), Some(c.id))
        }
        None => (None, None),
    };
    let (cmp, dir) = if sort.descending {
        ("<", "DESC")
    } else {
        (">", "ASC")
    };
    let sql = format!(
        "SELECT {cols} FROM {table} \
         WHERE uploaded_at IS NOT NULL \
           AND (deleted_at IS NULL OR $3 OR $4::timestamptz IS NOT NULL) \
           AND ($1::text IS NULL OR {tf} = $1) \
           AND ($2::uuid IS NULL OR {uf} = $2) \
           AND ($4::timestamptz IS NULL OR updated_at >= $4) \
           AND ($5::timestamptz IS NULL OR (created_at, id) {cmp} ($5, $6)) \
         ORDER BY created_at {dir}, id {dir} LIMIT $7",
        cols = spec.columns,
        table = spec.table,
        tf = spec.text_filter,
        uf = spec.uuid_filter,
    );
    let mut tx = ctx.begin(&state.db).await?;
    let rows: Vec<T> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
        .bind(text)
        .bind(id_filter)
        .bind(params.include_deleted.unwrap_or(false))
        .bind(params.updated_since)
        .bind(cursor_at)
        .bind(cursor_id)
        .bind(i64::from(limit) + 1)
        .fetch_all(&mut *tx)
        .await?;
    Ok(Json(Page::from_rows(rows, limit, &sort, |r| Cursor {
        key: json!(
            r.created_at()
                .to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
        ),
        id: r.id(),
    })))
}

async fn complete_upload<T: FileRow>(
    state: &AppState,
    ctx: &RequestContext,
    spec: &Spec,
    id: Uuid,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let store = store(state)?;
    let mut tx = ctx.begin(&state.db).await?;
    let row = core(&mut tx, spec, id).await?;
    if row.uploaded_at.is_none() {
        verify_object(store.as_ref(), &row).await?;
        sqlx::query(dyn_sql!(
            "UPDATE {} SET uploaded_at = now(), updated_by = $2 WHERE id = $1 AND uploaded_at IS NULL",
            spec.table
        ))
        .bind(id)
        .bind(ctx.identity.user_id)
        .execute(&mut *tx)
        .await?;
    }
    let record: T = fetch(&mut tx, spec, id).await?;
    tx.commit().await?;
    Ok(concurrency::record_response(
        StatusCode::OK,
        record.version(),
        &record,
    ))
}

/// Checks the stored object against the record. A wrong object is deleted so it cannot be used.
async fn verify_object(store: &dyn ObjectStore, row: &Core) -> ApiResult<()> {
    let declared = u64::try_from(row.size_bytes).unwrap_or(0);
    let Some(meta) = store.head(&row.object_key).await? else {
        return Err(issue("/upload", "not_uploaded"));
    };
    let reject = |code: &'static str| async move {
        if let Err(e) = store.delete(&row.object_key).await {
            tracing::warn!(error = %e, "could not delete the rejected object");
        }
        Err(issue("/upload", code))
    };
    if meta.size_bytes != declared {
        return reject("size_mismatch").await;
    }
    if meta
        .content_type
        .as_deref()
        .is_none_or(|t| media_type(t) != row.content_type)
    {
        return reject("content_type_mismatch").await;
    }
    let expected = parse_sha256(&row.sha256);
    let actual = match meta.sha256 {
        Some(d) => Some(d),
        // The store did not keep the checksum: hash the object.
        None => store
            .digest(&row.object_key, declared)
            .await?
            .map(|(_, d)| d),
    };
    if actual.is_none() || actual != expected {
        return reject("checksum_mismatch").await;
    }
    Ok(())
}

async fn download_link(
    state: &AppState,
    ctx: &RequestContext,
    spec: &Spec,
    id: Uuid,
) -> ApiResult<Response> {
    let store = store(state)?;
    let mut tx = ctx.begin(&state.db).await?;
    let row = core(&mut tx, spec, id).await?;
    if row.uploaded_at.is_none() {
        return Err(ApiError::new(ErrorCode::Conflict)
            .with_errors(vec![FieldIssue::new("/upload", "not_uploaded")]));
    }
    let link = store.presign_get(&row.object_key, ttl(state))?;
    let body = DownloadLink {
        url: link.url,
        expires_at: link.expires_at,
        content_type: row.content_type,
        size_bytes: row.size_bytes,
    };
    let mut response = Json(body).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    Ok(response)
}

async fn soft_delete(
    state: &AppState,
    ctx: &RequestContext,
    spec: &Spec,
    id: Uuid,
    headers: &HeaderMap,
    check_hold: bool,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let (version, hold): (i64, bool) = sqlx::query_as(dyn_sql!(
        "SELECT version, {} FROM {} WHERE id = $1 AND deleted_at IS NULL",
        if check_hold {
            "(legal_hold AND (retention_until IS NULL OR retention_until > now()))"
        } else {
            "false"
        },
        spec.table
    ))
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| ApiError::new(ErrorCode::NotFound))?;
    concurrency::check_if_match(
        Concurrency::FieldLww,
        headers,
        version,
        || json!({"id": id, "version": version}),
    )?;
    if hold {
        return Err(legal_hold_error());
    }
    sqlx::query(dyn_sql!(
        "UPDATE {} SET deleted_at = now(), updated_by = $2 WHERE id = $1",
        spec.table
    ))
    .bind(id)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        if is_sqlstate(&e, "23000") {
            legal_hold_error()
        } else {
            e.into()
        }
    })?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

// --- attachments --------------------------------------------------------------------------

/// Creates an attachment record and returns the upload instructions.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/attachments",
    tag = "files",
    operation_id = "attachment_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = NewAttachment,
    responses((
        status = 201,
        description = "Attachment created; upload the bytes with `upload`, then call `complete`",
        body = AttachmentCreated,
        example = json!({
            "attachment": {
                "id": "0192f1c07a3b7000800000000000000b", "kind": "photo", "record_kind": "batch",
                "record_id": "0192f1c07a3b7000800000000000000c", "content_type": "image/jpeg",
                "size_bytes": 482_113,
                "sha256": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
                "caption": "Krausen on day 2", "uploaded_at": null,
                "created_at": "2026-11-13T10:00:00Z", "updated_at": "2026-11-13T10:00:00Z",
                "deleted_at": null, "version": 1
            },
            "upload": {
                "method": "PUT",
                "url": "https://files.example.com/bucket/ws/attachments/id?X-Amz-Signature=...",
                "headers": {"content-type": "image/jpeg", "x-amz-checksum-sha256": "n4bQgYhMfWWaL+qgxVrQFaO/TxsrC4Is0V1sFbDwCgg="},
                "expires_at": "2026-11-13T10:10:00Z"
            }
        })
    ))
)]
pub async fn attachment_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(new): ApiJson<NewAttachment>,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    store(&state)?;
    let mut issues = Vec::new();
    if !["photo", "note", "file"].contains(&new.kind.as_str()) {
        issues.push(FieldIssue::new("/kind", "invalid_value"));
    }
    let record_table = match new.record_kind.as_str() {
        "batch" => Some("batches"),
        "tasting_note" => Some("tasting_notes"),
        _ => None,
    };
    if record_table.is_none() {
        issues.push(FieldIssue::new("/record_kind", "invalid_value"));
    }
    if new
        .caption
        .as_ref()
        .is_some_and(|c| c.chars().count() > 500)
    {
        issues.push(FieldIssue::new("/caption", "too_long").with_params(json!({"max": 500})));
    }
    let digest = validate_file(
        &new.content_type,
        new.size_bytes,
        &new.sha256,
        ATTACHMENT_TYPES,
    );
    let digest = match (digest, issues.is_empty()) {
        (Ok(d), true) => d,
        (Ok(_), false) => return Err(ApiError::validation(issues)),
        (Err(e), _) => {
            issues.extend(e.errors().iter().cloned());
            return Err(ApiError::validation(issues));
        }
    };
    let Some(record_table) = record_table else {
        return Err(issue("/record_kind", "invalid_value"));
    };
    let id = Uuid::now_v7();
    let key = object_key(&ATTACHMENTS, ws, id);
    let mut tx = ctx.begin(&state.db).await?;
    let target_exists: Option<i32> = sqlx::query_scalar(dyn_sql!(
        "SELECT 1 FROM {record_table} WHERE id = $1 AND deleted_at IS NULL"
    ))
    .bind(new.record_id)
    .fetch_optional(&mut *tx)
    .await?;
    if target_exists.is_none() {
        return Err(issue("/record_id", "not_found"));
    }
    let size = i64::try_from(new.size_bytes).unwrap_or(i64::MAX);
    let attachment: Attachment = sqlx::query_as(dyn_sql!(
        "INSERT INTO attachments (id, workspace_id, kind, record_kind, record_id, object_key, \
             content_type, size_bytes, sha256, caption, created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, decode($9, 'hex'), $10, $11, $11) \
         RETURNING {}",
        ATTACHMENTS.columns
    ))
    .bind(id)
    .bind(ws)
    .bind(&new.kind)
    .bind(&new.record_kind)
    .bind(new.record_id)
    .bind(&key)
    .bind(&new.content_type)
    .bind(size)
    .bind(new.sha256.to_ascii_lowercase())
    .bind(&new.caption)
    .bind(ctx.identity.user_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        if is_sqlstate(&e, "23503") {
            issue("/record_id", "not_found")
        } else {
            e.into()
        }
    })?;
    let upload = presign_upload(&state, &key, &new.content_type, new.size_bytes, digest)?;
    tx.commit().await?;
    let location = format!("/v1/workspaces/{ws}/attachments/{id}");
    Ok(created_response(
        &location,
        attachment.version,
        &AttachmentCreated { attachment, upload },
    ))
}

/// Confirms that the bytes were uploaded; the stored object is checked first.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/attachments/{id}/complete",
    tag = "files",
    operation_id = "attachment_complete",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Attachment")
    ),
    responses((
        status = 200,
        description = "Upload verified (size, type and checksum); safe to repeat",
        body = Attachment,
        example = json!({
            "id": "0192f1c07a3b7000800000000000000b", "kind": "photo", "record_kind": "batch",
            "record_id": "0192f1c07a3b7000800000000000000c", "content_type": "image/jpeg",
            "size_bytes": 482_113,
            "sha256": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
            "caption": null, "uploaded_at": "2026-11-13T10:01:00Z",
            "created_at": "2026-11-13T10:00:00Z", "updated_at": "2026-11-13T10:01:00Z",
            "deleted_at": null, "version": 2
        })
    ))
)]
pub async fn attachment_complete(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Response> {
    complete_upload::<Attachment>(&state, &ctx, &ATTACHMENTS, id).await
}

/// Lists completed attachments.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/attachments",
    tag = "files",
    operation_id = "attachment_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("record_kind" = Option<String>, Query, description = "`batch` or `tasting_note`"),
        ("record_id" = Option<Uuid>, Query, description = "Record the files belong to"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page"),
        ("sort" = Option<String>, Query, description = "`created_at` or `-created_at` (default)"),
        ("updated_since" = Option<String>, Query, description = "Changed since (RFC 3339), deleted ones included"),
        ("include_deleted" = Option<bool>, Query, description = "Include soft-deleted records")
    ),
    responses((
        status = 200,
        description = "A page of attachments",
        body = Page<Attachment>,
        example = json!({"items": [], "next_cursor": null})
    ))
)]
pub async fn attachment_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<AttachmentFilter>,
) -> ApiResult<Json<Page<Attachment>>> {
    list_rows(
        &state,
        &ctx,
        &ATTACHMENTS,
        &params,
        filter.record_kind.as_deref(),
        filter.record_id,
    )
    .await
}

/// Reads one attachment.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/attachments/{id}",
    tag = "files",
    operation_id = "attachment_get",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Attachment")
    ),
    responses(
        (
            status = 200,
            description = "The attachment (also while the upload is pending)",
            body = Attachment,
            example = json!({
                "id": "0192f1c07a3b7000800000000000000b", "kind": "photo", "record_kind": "batch",
                "record_id": "0192f1c07a3b7000800000000000000c", "content_type": "image/jpeg",
                "size_bytes": 482_113,
                "sha256": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
                "caption": null, "uploaded_at": "2026-11-13T10:01:00Z",
                "created_at": "2026-11-13T10:00:00Z", "updated_at": "2026-11-13T10:01:00Z",
                "deleted_at": null, "version": 2
            })
        ),
        (status = 304, description = "Not modified (`If-None-Match` matched)")
    )
)]
pub async fn attachment_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    get_one::<Attachment>(&state, &ctx, &ATTACHMENTS, id, &headers).await
}

/// Gives a short lived link to download the file.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/attachments/{id}/download",
    tag = "files",
    operation_id = "attachment_download",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Attachment")
    ),
    responses((
        status = 200,
        description = "Presigned download link",
        body = DownloadLink,
        example = json!({
            "url": "https://files.example.com/bucket/ws/attachments/id?X-Amz-Signature=...",
            "expires_at": "2026-11-13T10:10:00Z", "content_type": "image/jpeg", "size_bytes": 482_113
        })
    ))
)]
pub async fn attachment_download(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Response> {
    download_link(&state, &ctx, &ATTACHMENTS, id).await
}

/// Deletes an attachment (soft delete).
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/attachments/{id}",
    tag = "files",
    operation_id = "attachment_delete",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Attachment")
    ),
    responses((status = 204, description = "Deleted"))
)]
pub async fn attachment_delete(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    soft_delete(&state, &ctx, &ATTACHMENTS, id, &headers, false).await
}

// --- scanned documents --------------------------------------------------------------------

/// Creates a scanned document record and returns the upload instructions.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/scanned-documents",
    tag = "files",
    operation_id = "scanned_document_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = NewScannedDocument,
    responses((
        status = 201,
        description = "Document created; upload the bytes with `upload`, then call `complete`",
        body = ScannedDocumentCreated,
        example = json!({
            "document": {
                "id": "0192f1c07a3b7000800000000000000d", "batch_id": "0192f1c07a3b7000800000000000000c",
                "form_type": "brew_log", "content_type": "application/pdf", "size_bytes": 90_211,
                "sha256": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
                "retention_until": null, "legal_hold": false, "uploaded_at": null,
                "created_at": "2026-11-13T10:00:00Z", "updated_at": "2026-11-13T10:00:00Z",
                "deleted_at": null, "version": 1
            },
            "upload": {
                "method": "PUT",
                "url": "https://files.example.com/bucket/ws/documents/id?X-Amz-Signature=...",
                "headers": {"content-type": "application/pdf", "x-amz-checksum-sha256": "n4bQgYhMfWWaL+qgxVrQFaO/TxsrC4Is0V1sFbDwCgg="},
                "expires_at": "2026-11-13T10:10:00Z"
            }
        })
    ))
)]
pub async fn scanned_document_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(new): ApiJson<NewScannedDocument>,
) -> ApiResult<Response> {
    ctx.require(WRITE_PERMISSION)?;
    if new.legal_hold == Some(true) || new.retention_until.is_some() {
        ctx.require(HOLD_PERMISSION)?;
    }
    let ws = workspace(&ctx)?;
    store(&state)?;
    let mut issues = Vec::new();
    let form_ok = (1..=50).contains(&new.form_type.len())
        && new.form_type.starts_with(|c: char| c.is_ascii_lowercase())
        && new
            .form_type
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    if !form_ok {
        issues.push(FieldIssue::new("/form_type", "invalid_value"));
    }
    let digest = validate_file(
        &new.content_type,
        new.size_bytes,
        &new.sha256,
        DOCUMENT_TYPES,
    );
    let digest = match (digest, issues.is_empty()) {
        (Ok(d), true) => d,
        (Ok(_), false) => return Err(ApiError::validation(issues)),
        (Err(e), _) => {
            issues.extend(e.errors().iter().cloned());
            return Err(ApiError::validation(issues));
        }
    };
    let id = Uuid::now_v7();
    let key = object_key(&DOCUMENTS, ws, id);
    let mut tx = ctx.begin(&state.db).await?;
    if let Some(batch_id) = new.batch_id {
        let exists: Option<i32> =
            sqlx::query_scalar("SELECT 1 FROM batches WHERE id = $1 AND deleted_at IS NULL")
                .bind(batch_id)
                .fetch_optional(&mut *tx)
                .await?;
        if exists.is_none() {
            return Err(issue("/batch_id", "not_found"));
        }
    }
    let size = i64::try_from(new.size_bytes).unwrap_or(i64::MAX);
    let document: ScannedDocument = sqlx::query_as(dyn_sql!(
        "INSERT INTO scanned_documents (id, workspace_id, batch_id, form_type, scanned_by, \
             object_key, content_type, size_bytes, sha256, retention_until, legal_hold, \
             created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, decode($9, 'hex'), $10, $11, $5, $5) \
         RETURNING {}",
        DOCUMENTS.columns
    ))
    .bind(id)
    .bind(ws)
    .bind(new.batch_id)
    .bind(&new.form_type)
    .bind(ctx.identity.user_id)
    .bind(&key)
    .bind(&new.content_type)
    .bind(size)
    .bind(new.sha256.to_ascii_lowercase())
    .bind(new.retention_until)
    .bind(new.legal_hold.unwrap_or(false))
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        if is_sqlstate(&e, "23503") {
            issue("/batch_id", "not_found")
        } else {
            e.into()
        }
    })?;
    let upload = presign_upload(&state, &key, &new.content_type, new.size_bytes, digest)?;
    tx.commit().await?;
    let location = format!("/v1/workspaces/{ws}/scanned-documents/{id}");
    Ok(created_response(
        &location,
        document.version,
        &ScannedDocumentCreated { document, upload },
    ))
}

/// Confirms that the bytes were uploaded; the stored object is checked first.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/scanned-documents/{id}/complete",
    tag = "files",
    operation_id = "scanned_document_complete",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Scanned document")
    ),
    responses((
        status = 200,
        description = "Upload verified (size, type and checksum); safe to repeat",
        body = ScannedDocument,
        example = json!({
            "id": "0192f1c07a3b7000800000000000000d", "batch_id": null, "form_type": "brew_log",
            "content_type": "application/pdf", "size_bytes": 90_211,
            "sha256": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
            "retention_until": null, "legal_hold": false, "uploaded_at": "2026-11-13T10:01:00Z",
            "created_at": "2026-11-13T10:00:00Z", "updated_at": "2026-11-13T10:01:00Z",
            "deleted_at": null, "version": 2
        })
    ))
)]
pub async fn scanned_document_complete(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Response> {
    complete_upload::<ScannedDocument>(&state, &ctx, &DOCUMENTS, id).await
}

/// Lists completed scanned documents.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/scanned-documents",
    tag = "files",
    operation_id = "scanned_document_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("form_type" = Option<String>, Query, description = "Form type"),
        ("batch_id" = Option<Uuid>, Query, description = "Batch"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page"),
        ("sort" = Option<String>, Query, description = "`created_at` or `-created_at` (default)"),
        ("updated_since" = Option<String>, Query, description = "Changed since (RFC 3339), deleted ones included"),
        ("include_deleted" = Option<bool>, Query, description = "Include soft-deleted records")
    ),
    responses((
        status = 200,
        description = "A page of scanned documents",
        body = Page<ScannedDocument>,
        example = json!({"items": [], "next_cursor": null})
    ))
)]
pub async fn scanned_document_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<DocumentFilter>,
) -> ApiResult<Json<Page<ScannedDocument>>> {
    list_rows(
        &state,
        &ctx,
        &DOCUMENTS,
        &params,
        filter.form_type.as_deref(),
        filter.batch_id,
    )
    .await
}

/// Reads one scanned document.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/scanned-documents/{id}",
    tag = "files",
    operation_id = "scanned_document_get",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Scanned document")
    ),
    responses(
        (
            status = 200,
            description = "The document (also while the upload is pending)",
            body = ScannedDocument,
            example = json!({
                "id": "0192f1c07a3b7000800000000000000d", "batch_id": null, "form_type": "brew_log",
                "content_type": "application/pdf", "size_bytes": 90_211,
                "sha256": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
                "retention_until": null, "legal_hold": false, "uploaded_at": "2026-11-13T10:01:00Z",
                "created_at": "2026-11-13T10:00:00Z", "updated_at": "2026-11-13T10:01:00Z",
                "deleted_at": null, "version": 2
            })
        ),
        (status = 304, description = "Not modified (`If-None-Match` matched)")
    )
)]
pub async fn scanned_document_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    get_one::<ScannedDocument>(&state, &ctx, &DOCUMENTS, id, &headers).await
}

/// Gives a short lived link to download the file.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/scanned-documents/{id}/download",
    tag = "files",
    operation_id = "scanned_document_download",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Scanned document")
    ),
    responses((
        status = 200,
        description = "Presigned download link",
        body = DownloadLink,
        example = json!({
            "url": "https://files.example.com/bucket/ws/documents/id?X-Amz-Signature=...",
            "expires_at": "2026-11-13T10:10:00Z", "content_type": "application/pdf", "size_bytes": 90_211
        })
    ))
)]
pub async fn scanned_document_download(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Response> {
    download_link(&state, &ctx, &DOCUMENTS, id).await
}

/// Deletes a scanned document (soft delete); refused under legal hold.
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/scanned-documents/{id}",
    tag = "files",
    operation_id = "scanned_document_delete",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Scanned document")
    ),
    responses((status = 204, description = "Deleted"))
)]
pub async fn scanned_document_delete(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> ApiResult<Response> {
    soft_delete(&state, &ctx, &DOCUMENTS, id, &headers, true).await
}

// --- routes -------------------------------------------------------------------------------

/// The file routes, wrapped with the sign-in and idempotency layers; pass the result to
/// [`crate::build`].
pub fn routes(state: &AppState) -> Router<AppState> {
    let writes = crate::idempotent(
        state,
        Router::new()
            .route("/v1/workspaces/{ws}/attachments", post(attachment_create))
            .route(
                "/v1/workspaces/{ws}/scanned-documents",
                post(scanned_document_create),
            ),
    );
    let others = Router::new()
        .route("/v1/workspaces/{ws}/attachments", get(attachment_list))
        .route(
            "/v1/workspaces/{ws}/attachments/{id}",
            get(attachment_get).delete(attachment_delete),
        )
        .route(
            "/v1/workspaces/{ws}/attachments/{id}/complete",
            post(attachment_complete),
        )
        .route(
            "/v1/workspaces/{ws}/attachments/{id}/download",
            get(attachment_download),
        )
        .route(
            "/v1/workspaces/{ws}/scanned-documents",
            get(scanned_document_list),
        )
        .route(
            "/v1/workspaces/{ws}/scanned-documents/{id}",
            get(scanned_document_get).delete(scanned_document_delete),
        )
        .route(
            "/v1/workspaces/{ws}/scanned-documents/{id}/complete",
            post(scanned_document_complete),
        )
        .route(
            "/v1/workspaces/{ws}/scanned-documents/{id}/download",
            get(scanned_document_download),
        );
    crate::authenticated(state, writes.merge(others))
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

    #[test]
    fn sha256_parsing() {
        assert!(parse_sha256(HASH).is_some());
        assert!(parse_sha256(&HASH.to_ascii_uppercase()).is_some());
        assert!(parse_sha256(&HASH[1..]).is_none());
        assert!(parse_sha256(&HASH.replace('9', "g")).is_none());
    }

    #[test]
    fn file_validation_reports_every_problem() {
        assert!(validate_file("image/png", 10, HASH, ATTACHMENT_TYPES).is_ok());
        let err = validate_file("text/html", 0, "zz", ATTACHMENT_TYPES).unwrap_err();
        let fields: Vec<&str> = err.errors().iter().map(|e| e.field.as_str()).collect();
        assert_eq!(fields, ["/content_type", "/size_bytes", "/sha256"]);
        assert!(validate_file("image/png", MAX_FILE_BYTES + 1, HASH, DOCUMENT_TYPES).is_err());
        // Parameters on the type would break the signed header comparison.
        assert!(validate_file("image/png; charset=x", 10, HASH, DOCUMENT_TYPES).is_err());
        assert!(validate_file("text/plain", 10, HASH, DOCUMENT_TYPES).is_err());
    }
}
