//! Replay safety for record-creating `POST`s (ADR-0008 rule 6).
//!
//! Mount [`layer`] (after the context layer) on the router of `POST` endpoints that create
//! records; read-only `POST`s (tool calculations) stay outside it.
//!
//! - The `Idempotency-Key` header is required (`428` when missing).
//! - The key, a SHA-256 of method, path, query and body, and later the response live 24 hours
//!   in `idempotency_keys`, scoped to the workspace (or, outside a workspace, the user).
//! - Same key and body: the stored response is replayed with `Idempotent-Replayed: true`.
//!   Same key, different request: `422 idempotency_mismatch`. First request still running:
//!   `409 idempotency_in_progress`.
//!
//! Decisions beyond the ADR: only `2xx` responses are stored (a failed attempt releases the key
//! so the client can retry the same request); the handler runs in a detached task so a client
//! disconnect cannot leave a half-finished key; a key still unanswered after two minutes
//! (crashed process) can be taken over by a retry, unless its work already committed: the
//! handler transactions mark the key row (`work_committed_at`, in the same transaction, see
//! `db::begin`). A takeover of a marked key does not run the work again; it answers `409`
//! (F2.1 finding 4).

use std::time::Duration;

use axum::{
    body::{Body, Bytes},
    extract::{Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::Instrument;
use uuid::Uuid;

use crate::{
    AppState,
    config::BODY_LIMIT_BYTES,
    context::RequestContext,
    error::{ApiError, ApiResult, ErrorCode, FieldIssue, REQUEST_ID},
};

/// Request header carrying the client-generated key.
pub static IDEMPOTENCY_KEY: HeaderName = HeaderName::from_static("idempotency-key");
/// Response header on replayed responses.
pub static IDEMPOTENT_REPLAYED: HeaderName = HeaderName::from_static("idempotent-replayed");

/// Longest key the table accepts.
const MAX_KEY_LEN: usize = 255;
/// Handler time limit; past it the key is released and the client gets `503`.
const HANDLER_TIMEOUT: Duration = Duration::from_secs(30);
/// Largest response body that is stored.
const MAX_STORED_BODY: usize = 4 * 1024 * 1024;
/// An unanswered key older than this may be taken over.
const STALE_AFTER_SECS: i32 = 120;
/// Response headers that are stored and replayed.
const REPLAYED_HEADERS: [&str; 4] = ["content-type", "location", "etag", "cache-control"];

#[derive(Serialize, Deserialize)]
struct Stored {
    headers: Vec<(String, String)>,
    body: String,
}

enum Claim {
    Fresh(Uuid),
    Replay {
        status: i32,
        body: Vec<u8>,
    },
    Mismatch,
    InProgress,
    /// The first attempt committed its work, then the process died before the response was
    /// stored. The response is unknown and the work must not run again.
    WorkCommitted,
}

type ExistingRow = (
    Uuid,
    Vec<u8>,
    Option<i32>,
    Option<Vec<u8>>,
    bool,
    bool,
    bool,
);

fn parse_key(headers: &HeaderMap) -> ApiResult<String> {
    let value = headers
        .get(&IDEMPOTENCY_KEY)
        .ok_or_else(|| ApiError::new(ErrorCode::PreconditionRequired))?;
    let key = value
        .to_str()
        .ok()
        .map(str::trim)
        .filter(|k| !k.is_empty() && k.len() <= MAX_KEY_LEN)
        .ok_or_else(|| {
            ApiError::new(ErrorCode::MalformedRequest)
                .with_errors(vec![FieldIssue::new("Idempotency-Key", "invalid_header")])
        })?;
    Ok(key.to_owned())
}

fn request_hash(method: &Method, uri: &axum::http::Uri, body: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(method.as_str().as_bytes());
    hasher.update(b"\n");
    hasher.update(uri.path().as_bytes());
    hasher.update(b"?");
    hasher.update(uri.query().unwrap_or_default().as_bytes());
    hasher.update(b"\n");
    hasher.update(body);
    hasher.finalize().into()
}

fn scope(ctx: &RequestContext) -> (Option<Uuid>, Option<Uuid>) {
    ctx.workspace_id
        .map_or_else(|| (None, Some(ctx.identity.user_id)), |ws| (Some(ws), None))
}

async fn claim(
    state: &AppState,
    ctx: &RequestContext,
    key: &str,
    hash: &[u8; 32],
) -> ApiResult<Claim> {
    let (workspace_id, user_id) = scope(ctx);
    for _ in 0..3 {
        let mut tx = ctx.begin(&state.db).await?;
        let id = Uuid::now_v7();
        let inserted: Option<Uuid> = sqlx::query_scalar(
            "INSERT INTO idempotency_keys (id, workspace_id, user_id, key, request_hash) \
             VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING RETURNING id",
        )
        .bind(id)
        .bind(workspace_id)
        .bind(user_id)
        .bind(key)
        .bind(hash.as_slice())
        .fetch_optional(&mut *tx)
        .await?;
        if let Some(id) = inserted {
            tx.commit().await.map_err(ApiError::from)?;
            return Ok(Claim::Fresh(id));
        }
        let existing: Option<ExistingRow> = sqlx::query_as(
            "SELECT id, request_hash, response_status, response_body, \
                    expires_at < now(), created_at < now() - make_interval(secs => $4), \
                    work_committed_at IS NOT NULL \
             FROM idempotency_keys \
             WHERE key = $1 AND workspace_id IS NOT DISTINCT FROM $2 \
               AND user_id IS NOT DISTINCT FROM $3",
        )
        .bind(key)
        .bind(workspace_id)
        .bind(user_id)
        .bind(f64::from(STALE_AFTER_SECS))
        .fetch_optional(&mut *tx)
        .await?;
        let Some((existing_id, found_hash, status, found_body, expired, abandoned, committed)) =
            existing
        else {
            continue; // deleted between the insert and the read
        };
        if expired {
            sqlx::query("DELETE FROM idempotency_keys WHERE id = $1")
                .bind(existing_id)
                .execute(&mut *tx)
                .await?;
            tx.commit().await.map_err(ApiError::from)?;
            continue;
        }
        if found_hash != hash.as_slice() {
            return Ok(Claim::Mismatch);
        }
        if let Some(status) = status {
            return Ok(Claim::Replay {
                status,
                body: found_body.unwrap_or_default(),
            });
        }
        if abandoned && committed {
            return Ok(Claim::WorkCommitted);
        }
        if abandoned {
            let taken: Option<Uuid> = sqlx::query_scalar(
                "UPDATE idempotency_keys SET created_at = now() \
                 WHERE id = $1 AND response_status IS NULL RETURNING id",
            )
            .bind(existing_id)
            .fetch_optional(&mut *tx)
            .await?;
            tx.commit().await.map_err(ApiError::from)?;
            if let Some(id) = taken {
                return Ok(Claim::Fresh(id));
            }
            continue;
        }
        return Ok(Claim::InProgress);
    }
    Err(ApiError::unavailable(1))
}

async fn release(state: &AppState, ctx: &RequestContext, id: Uuid) {
    let result = async {
        let mut tx = ctx.begin(&state.db).await?;
        sqlx::query("DELETE FROM idempotency_keys WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await.map_err(ApiError::from)
    }
    .await;
    if let Err(e) = result {
        tracing::warn!(
            code = e.code().as_str(),
            "could not release idempotency key"
        );
    }
}

async fn store(
    state: &AppState,
    ctx: &RequestContext,
    id: Uuid,
    status: StatusCode,
    headers: &HeaderMap,
    body: &Bytes,
) -> ApiResult<()> {
    let stored = Stored {
        headers: REPLAYED_HEADERS
            .iter()
            .filter_map(|name| {
                headers
                    .get(*name)
                    .and_then(|v| v.to_str().ok())
                    .map(|v| ((*name).to_owned(), v.to_owned()))
            })
            .collect(),
        body: STANDARD.encode(body),
    };
    let bytes = serde_json::to_vec(&stored).map_err(|_| ApiError::new(ErrorCode::Internal))?;
    let mut tx = ctx.begin(&state.db).await?;
    sqlx::query(
        "UPDATE idempotency_keys SET response_status = $2, response_body = $3 WHERE id = $1",
    )
    .bind(id)
    .bind(i32::from(status.as_u16()))
    .bind(bytes)
    .execute(&mut *tx)
    .await?;
    tx.commit().await.map_err(ApiError::from)
}

fn replay(status: i32, stored: &[u8]) -> Response {
    let Ok(stored) = serde_json::from_slice::<Stored>(stored) else {
        return ApiError::new(ErrorCode::Internal).into_response();
    };
    let Ok(body) = STANDARD.decode(&stored.body) else {
        return ApiError::new(ErrorCode::Internal).into_response();
    };
    let status = u16::try_from(status)
        .ok()
        .and_then(|s| StatusCode::from_u16(s).ok())
        .unwrap_or(StatusCode::OK);
    let mut response = (status, body).into_response();
    for (name, value) in stored.headers {
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(&value),
        ) {
            response.headers_mut().insert(name, value);
        }
    }
    response.headers_mut().insert(
        IDEMPOTENT_REPLAYED.clone(),
        HeaderValue::from_static("true"),
    );
    response
}

/// The middleware (see the module documentation).
pub async fn layer(State(state): State<AppState>, req: Request, next: Next) -> Response {
    match handle(state, req, next).await {
        Ok(response) => response,
        Err(e) => e.into_response(),
    }
}

async fn handle(state: AppState, req: Request, next: Next) -> ApiResult<Response> {
    if matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
        return Ok(next.run(req).await);
    }
    let ctx = req
        .extensions()
        .get::<RequestContext>()
        .cloned()
        .ok_or_else(|| ApiError::new(ErrorCode::Unauthenticated))?;
    let key = parse_key(req.headers())?;
    let (parts, body) = req.into_parts();
    let bytes = axum::body::to_bytes(body, BODY_LIMIT_BYTES)
        .await
        .map_err(|_| ApiError::new(ErrorCode::PayloadTooLarge))?;
    let hash = request_hash(&parts.method, &parts.uri, &bytes);

    let id = match claim(&state, &ctx, &key, &hash).await? {
        Claim::Fresh(id) => id,
        Claim::Replay { status, body } => return Ok(replay(status, &body)),
        Claim::Mismatch => return Err(ApiError::new(ErrorCode::IdempotencyMismatch)),
        Claim::WorkCommitted => {
            return Err(
                ApiError::new(ErrorCode::Conflict).with_errors(vec![FieldIssue::new(
                    "Idempotency-Key",
                    "work_already_committed",
                )]),
            );
        }
        Claim::InProgress => {
            return Err(ApiError::new(ErrorCode::IdempotencyInProgress).with_retry_after(1));
        }
    };

    let req = Request::from_parts(parts, Body::from(bytes));
    let request_id = crate::error::current_request_id().unwrap_or_default();
    let task_state = state.clone();
    let task = tokio::spawn(
        REQUEST_ID
            .scope(request_id, async move {
                let outcome = tokio::time::timeout(
                    HANDLER_TIMEOUT,
                    db::IDEMPOTENCY_KEY.scope(id, next.run(req)),
                )
                .await;
                let Ok(response) = outcome else {
                    release(&task_state, &ctx, id).await;
                    return ApiError::unavailable(1).into_response();
                };
                let (head, body) = response.into_parts();
                let Ok(body) = axum::body::to_bytes(body, MAX_STORED_BODY).await else {
                    release(&task_state, &ctx, id).await;
                    return ApiError::new(ErrorCode::Internal).into_response();
                };
                if head.status.is_success() {
                    if let Err(e) =
                        store(&task_state, &ctx, id, head.status, &head.headers, &body).await
                    {
                        tracing::warn!(
                            code = e.code().as_str(),
                            "could not store idempotent response"
                        );
                    }
                } else {
                    release(&task_state, &ctx, id).await;
                }
                Response::from_parts(head, Body::from(body))
            })
            .instrument(tracing::Span::current()),
    );
    match task.await {
        Ok(response) => Ok(response),
        Err(e) => {
            tracing::error!(error = %e, "handler task failed");
            // `ctx` moved into the task; the key becomes takeable after the stale period.
            Err(ApiError::new(ErrorCode::Internal))
        }
    }
}
