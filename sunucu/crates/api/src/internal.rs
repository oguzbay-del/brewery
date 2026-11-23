//! Support panel endpoints (`/internal`, T2.30).
//!
//! These routes are not on the public API gateway and not in the `OpenAPI` document. The sign-in
//! of staff is the company Google Workspace account with a passkey, behind an OIDC reverse proxy
//! in the cluster. The proxy sets `X-Auth-Request-Email` and the shared token
//! `X-Internal-Proxy-Token` (`INTERNAL_PROXY_TOKEN`). Without a configured token every route
//! answers `503`. A caller without a valid token gets `401`; an e-mail that is not an active
//! staff member gets `403`.
//!
//! Rules:
//! - Every operation needs a `reason` and writes a `support_actions` row and an audit row.
//! - Customer content (`content/*`) is open only while a `support_grants` row is valid. Without
//!   a grant the answer is `403` and the attempt is logged. The content is read in a read-only
//!   transaction with `actor_kind = support`.
//! - Only the worker writes `subscriptions` and `entitlements`: a correction is a request that
//!   the worker applies.

use axum::{
    Json, Router,
    extract::{FromRequestParts, Path, State},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use db::{ActorKind, AuditEvent, DbContext, NewAuditEntry};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgConnection;
use support::panel::{self, Approval, Staff};
use uuid::Uuid;

use crate::{
    AppState,
    error::{ApiError, ApiJson, ApiQuery, ApiResult, ErrorCode, FieldIssue},
};

/// Settings of the support panel routes.
#[derive(Debug, Clone)]
pub struct InternalConfig {
    token: Vec<u8>,
}

impl InternalConfig {
    /// Settings with the shared token of the reverse proxy.
    #[must_use]
    pub fn new(token: &str) -> Self {
        Self {
            token: token.as_bytes().to_vec(),
        }
    }
}

/// The staff member who calls.
#[derive(Debug, Clone)]
pub struct StaffCtx(pub Staff);

impl FromRequestParts<AppState> for StaffCtx {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Some(config) = &state.internal else {
            return Err(ApiError::new(ErrorCode::Unavailable));
        };
        let header = |n: &str| parts.headers.get(n).and_then(|v| v.to_str().ok());
        let token_ok = header("x-internal-proxy-token")
            .is_some_and(|t| Sha256::digest(t.as_bytes()) == Sha256::digest(&config.token));
        if !token_ok {
            return Err(ApiError::new(ErrorCode::Unauthenticated));
        }
        let Some(email) = header("x-auth-request-email") else {
            return Err(ApiError::new(ErrorCode::Unauthenticated));
        };
        let mut conn = state.db.acquire().await?;
        panel::find_staff(&mut conn, email)
            .await
            .map_err(se)?
            .map(StaffCtx)
            .ok_or_else(|| ApiError::new(ErrorCode::Forbidden))
    }
}

fn se(e: support::Error) -> ApiError {
    match e {
        support::Error::Db(e) => e.into(),
        other => ApiError::from(db::Error::from(sqlx::Error::Protocol(other.to_string()))),
    }
}

fn issue(field: &str, code: &str) -> ApiError {
    ApiError::validation(vec![FieldIssue::new(field, code)])
}

fn reason(r: Option<&str>) -> ApiResult<String> {
    let r = r.unwrap_or_default().trim();
    if r.is_empty() || r.chars().count() > 500 {
        return Err(issue("/reason", "length_out_of_range"));
    }
    Ok(r.to_owned())
}

/// A read-write transaction in the context of the workspace (the panel service writes the log and
/// the requests; the audit rows name the staff member). `user` is the customer user
/// for tables that only the owner sees (sessions).
async fn staff_tx(
    state: &AppState,
    ws: Uuid,
    user: Option<Uuid>,
) -> ApiResult<sqlx::Transaction<'static, sqlx::Postgres>> {
    open_tx(state, ws, user, ActorKind::System).await
}

/// The read-only transaction of a content read: `actor_kind = support` makes `db::begin` set
/// `SET TRANSACTION READ ONLY`. Only call it after a valid grant was found.
async fn support_read_tx(
    state: &AppState,
    ws: Uuid,
) -> ApiResult<sqlx::Transaction<'static, sqlx::Postgres>> {
    open_tx(state, ws, None, ActorKind::Support).await
}

async fn open_tx(
    state: &AppState,
    ws: Uuid,
    user: Option<Uuid>,
    actor_kind: ActorKind,
) -> ApiResult<sqlx::Transaction<'static, sqlx::Postgres>> {
    let mut tx = db::begin(
        &state.db,
        &DbContext {
            user_id: user,
            workspace_id: Some(ws),
            actor_kind,
        },
    )
    .await?;
    let exists: Option<i32> = sqlx::query_scalar("SELECT 1 FROM workspaces WHERE id = $1")
        .bind(ws)
        .fetch_optional(&mut *tx)
        .await?;
    if exists.is_none() {
        return Err(ApiError::new(ErrorCode::NotFound));
    }
    Ok(tx)
}

async fn audit(
    conn: &mut PgConnection,
    ws: Uuid,
    staff: &Staff,
    event: AuditEvent,
    kind: &str,
    id: Option<Uuid>,
    why: &str,
) -> ApiResult<()> {
    db::append_audit(
        conn,
        ws,
        &NewAuditEntry {
            actor_kind: ActorKind::Support,
            actor_id: Some(staff.id),
            event,
            record_kind: kind.to_owned(),
            record_id: id,
            previous_value: None,
            new_value: None,
            reason: Some(why.to_owned()),
        },
    )
    .await?;
    Ok(())
}

/// Query of the read endpoints.
#[derive(Debug, Default, Deserialize)]
pub struct ReasonQuery {
    reason: Option<String>,
}

/// Query of the search.
#[derive(Debug, Default, Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
}

/// Body with only a reason.
#[derive(Debug, Deserialize)]
pub struct ReasonBody {
    reason: Option<String>,
}

// --- search and summary --------------------------------------------------------------------

/// Finds workspaces by pseudonym, workspace id or request number. It returns metadata only.
/// There is no workspace to write the action log to, so the search goes to the service log.
async fn search(
    State(state): State<AppState>,
    StaffCtx(staff): StaffCtx,
    ApiQuery(q): ApiQuery<SearchQuery>,
) -> ApiResult<Json<Value>> {
    let q = q.q.unwrap_or_default();
    let q = q.trim();
    if q.is_empty() || q.chars().count() > 100 {
        return Err(issue("q", "length_out_of_range"));
    }
    tracing::info!(staff = %staff.id, "support search");
    let rows: Vec<(
        Uuid,
        String,
        String,
        String,
        String,
        chrono::DateTime<chrono::Utc>,
    )> = sqlx::query_as("SELECT * FROM support_find_workspace($1)")
        .bind(q)
        .fetch_all(&state.db)
        .await?;
    Ok(Json(json!(
        rows.into_iter()
            .map(|(id, name, mode, region, pseudonym, created_at)| json!({
                "workspace_id": id, "name": name, "mode": mode, "region": region,
                "pseudonym": pseudonym, "created_at": created_at}))
            .collect::<Vec<_>>()
    )))
}

/// Account summary without customer content.
async fn summary(
    State(state): State<AppState>,
    StaffCtx(staff): StaffCtx,
    Path(ws): Path<Uuid>,
    ApiQuery(q): ApiQuery<ReasonQuery>,
) -> ApiResult<Json<Value>> {
    let why = reason(q.reason.as_deref())?;
    let mut tx = staff_tx(&state, ws, None).await?;
    panel::record_action(
        &mut tx,
        ws,
        staff.id,
        None,
        "workspace.summary",
        None,
        None,
        &why,
    )
    .await
    .map_err(se)?;
    let (name, mode, region, country, created_at): (
        String,
        String,
        String,
        String,
        chrono::DateTime<chrono::Utc>,
    ) = sqlx::query_as(
        "SELECT name, mode, region, country::text, created_at FROM workspaces WHERE id = $1",
    )
    .bind(ws)
    .fetch_one(&mut *tx)
    .await?;
    let members: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM workspace_members WHERE status = 'active' AND deleted_at IS NULL",
    )
    .fetch_one(&mut *tx)
    .await?;
    let open_requests: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM support_requests WHERE status IN ('open', 'answered')
           AND deleted_at IS NULL",
    )
    .fetch_one(&mut *tx)
    .await?;
    let access = billing::access::access(&mut tx, ws).await?;
    let subscriptions = billing::access::subscriptions(&mut tx, ws).await?;
    let grant = panel::active_grant(&mut tx, ws).await.map_err(se)?;
    tx.commit().await?;
    Ok(Json(json!({
        "workspace": {"id": ws, "name": name, "mode": mode, "region": region,
            "country": country, "created_at": created_at},
        "members": members,
        "open_requests": open_requests,
        "access": {"read_only": matches!(access.level, billing::access::AccessLevel::ReadOnly),
            "valid_until": access.valid_until, "trial_ends_at": access.trial_ends_at},
        "subscriptions": subscriptions.into_iter().map(|s| json!({
            "source": s.source, "product_key": s.product_key, "status": s.status,
            "period_ends_at": s.period_ends_at})).collect::<Vec<_>>(),
        "grant_active": grant.is_some(),
    })))
}

// --- content (needs a grant) ---------------------------------------------------------------

/// Reads customer content. Without a valid grant: `403`, and the attempt is logged.
async fn content(
    State(state): State<AppState>,
    StaffCtx(staff): StaffCtx,
    Path((ws, kind)): Path<(Uuid, String)>,
    ApiQuery(q): ApiQuery<ReasonQuery>,
) -> ApiResult<Json<Value>> {
    let why = reason(q.reason.as_deref())?;
    let sql = match kind.as_str() {
        "batches" => {
            "SELECT id, code, name, stage FROM batches WHERE deleted_at IS NULL \
                      ORDER BY created_at DESC LIMIT 100"
        }
        "recipes" => {
            "SELECT id, name FROM recipes WHERE deleted_at IS NULL \
                      ORDER BY created_at DESC LIMIT 100"
        }
        _ => return Err(ApiError::new(ErrorCode::NotFound)),
    };
    // First transaction: check the grant and write the log. It commits also when access is denied.
    let mut tx = staff_tx(&state, ws, None).await?;
    let grant = panel::active_grant(&mut tx, ws).await.map_err(se)?;
    let action = if grant.is_some() {
        format!("content.read.{kind}")
    } else {
        format!("content.denied.{kind}")
    };
    panel::record_action(
        &mut tx,
        ws,
        staff.id,
        None,
        &action,
        Some("support_grant"),
        grant,
        &why,
    )
    .await
    .map_err(se)?;
    tx.commit().await?;
    if grant.is_none() {
        return Err(ApiError::new(ErrorCode::Forbidden)
            .with_errors(vec![FieldIssue::new("/grant", "support_grant_required")]));
    }
    // Second transaction: read only, as support.
    let mut tx = support_read_tx(&state, ws).await?;
    let rows: Vec<sqlx::postgres::PgRow> = sqlx::query(sqlx::AssertSqlSafe(sql))
        .fetch_all(&mut *tx)
        .await?;
    let items: Vec<Value> = rows
        .iter()
        .map(|r| {
            use sqlx::{Column, Row, TypeInfo};
            let mut o = serde_json::Map::new();
            for c in r.columns() {
                let name = c.name();
                let v = if c.type_info().name() == "UUID" {
                    json!(r.try_get::<Uuid, _>(name).ok())
                } else {
                    json!(r.try_get::<String, _>(name).ok())
                };
                o.insert(name.to_owned(), v);
            }
            Value::Object(o)
        })
        .collect();
    Ok(Json(json!({"items": items})))
}

// --- requests ------------------------------------------------------------------------------

async fn find_request(state: &AppState, number: i64) -> ApiResult<(Uuid, Uuid)> {
    sqlx::query_as("SELECT workspace_id, id FROM support_find_request(NULL, $1)")
        .bind(number)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| ApiError::new(ErrorCode::NotFound))
}

/// A request with all messages, internal notes included.
async fn request_get(
    State(state): State<AppState>,
    StaffCtx(staff): StaffCtx,
    Path(number): Path<i64>,
    ApiQuery(q): ApiQuery<ReasonQuery>,
) -> ApiResult<Json<Value>> {
    let why = reason(q.reason.as_deref())?;
    let (ws, id) = find_request(&state, number).await?;
    let mut tx = staff_tx(&state, ws, None).await?;
    panel::record_action(
        &mut tx,
        ws,
        staff.id,
        Some(id),
        "request.read",
        Some("support_request"),
        Some(id),
        &why,
    )
    .await
    .map_err(se)?;
    let (topic, title, status, issue_no): (String, String, String, Option<i32>) = sqlx::query_as(
        "SELECT topic, title, status, github_issue_number FROM support_requests WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    let messages: Vec<(
        Uuid,
        String,
        String,
        String,
        bool,
        chrono::DateTime<chrono::Utc>,
    )> = sqlx::query_as(
        "SELECT id, author_kind, visibility, body, retracted_at IS NOT NULL, created_at
             FROM support_messages WHERE request_id = $1 AND deleted_at IS NULL
             ORDER BY created_at, id",
    )
    .bind(id)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(json!({
        "workspace_id": ws, "id": id, "public_number": number, "topic": topic, "title": title,
        "status": status, "github_issue_number": issue_no,
        "messages": messages.into_iter().map(|(id, author, visibility, body, retracted, at)| json!({
            "id": id, "author_kind": author, "visibility": visibility, "body": body,
            "retracted": retracted, "created_at": at})).collect::<Vec<_>>(),
    })))
}

/// Body of a staff message.
#[derive(Debug, Deserialize)]
pub struct StaffMessage {
    body: String,
    #[serde(default)]
    internal: bool,
    reason: Option<String>,
}

/// A message from the panel. A reply reaches the customer (inbox, push, e-mail); an internal
/// note does not.
async fn request_message(
    State(state): State<AppState>,
    StaffCtx(staff): StaffCtx,
    Path(number): Path<i64>,
    ApiJson(m): ApiJson<StaffMessage>,
) -> ApiResult<Response> {
    let why = reason(m.reason.as_deref())?;
    if m.body.trim().is_empty() || m.body.chars().count() > 20_000 {
        return Err(issue("/body", "length_out_of_range"));
    }
    let (ws, id) = find_request(&state, number).await?;
    let mut tx = staff_tx(&state, ws, None).await?;
    let message = support::reply::staff_message(&mut tx, ws, id, staff.id, &m.body, m.internal)
        .await
        .map_err(se)?;
    let action = if m.internal {
        "request.note"
    } else {
        "request.reply"
    };
    panel::record_action(
        &mut tx,
        ws,
        staff.id,
        Some(id),
        action,
        Some("support_message"),
        Some(message),
        &why,
    )
    .await
    .map_err(se)?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(json!({"id": message}))).into_response())
}

// --- corrections, sessions, devices --------------------------------------------------------

/// Body of a correction request.
#[derive(Debug, Deserialize)]
pub struct CorrectionBody {
    kind: String,
    valid_until: Option<chrono::DateTime<chrono::Utc>>,
    user_id: Option<Uuid>,
    reason: Option<String>,
}

/// Asks the worker for a correction: manual entitlement, ending an entitlement, subscription
/// re-verification, or a deletion request (recorded, a person decides).
async fn correction_create(
    State(state): State<AppState>,
    StaffCtx(staff): StaffCtx,
    Path(ws): Path<Uuid>,
    ApiJson(b): ApiJson<CorrectionBody>,
) -> ApiResult<Response> {
    let why = reason(b.reason.as_deref())?;
    let kind = match b.kind.as_str() {
        "entitlement_grant"
        | "entitlement_revoke"
        | "subscription_reverify"
        | "deletion_request" => b.kind.as_str(),
        _ => return Err(issue("/kind", "invalid_value")),
    };
    if b.valid_until.is_some_and(|t| t <= chrono::Utc::now()) {
        return Err(issue("/valid_until", "must_be_in_the_future"));
    }
    if kind == "deletion_request" && staff.role != "support_admin" {
        return Err(ApiError::new(ErrorCode::Forbidden));
    }
    if kind != "deletion_request" && b.user_id.is_some() {
        return Err(issue("/user_id", "not_allowed"));
    }
    let mut tx = staff_tx(&state, ws, None).await?;
    let params = if kind == "deletion_request" {
        let user_id = b.user_id.ok_or_else(|| issue("/user_id", "required"))?;
        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM workspace_members
                WHERE workspace_id = $1 AND user_id = $2
                  AND status = 'active' AND deleted_at IS NULL
            )",
        )
        .bind(ws)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;
        if !is_member {
            return Err(issue("/user_id", "not_a_workspace_member"));
        }
        json!({"user_id": user_id})
    } else {
        json!({"valid_until": b.valid_until})
    };
    let id = panel::request_correction(
        &mut tx,
        ws,
        staff.id,
        kind,
        &params,
        &why,
    )
    .await
    .map_err(se)?;
    panel::record_action(
        &mut tx,
        ws,
        staff.id,
        None,
        &format!("correction.{kind}"),
        Some("support_correction"),
        Some(id),
        &why,
    )
    .await
    .map_err(se)?;
    tx.commit().await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({"id": id, "status": "pending"})),
    )
        .into_response())
}

/// The corrections of a workspace.
async fn correction_list(
    State(state): State<AppState>,
    StaffCtx(staff): StaffCtx,
    Path(ws): Path<Uuid>,
    ApiQuery(q): ApiQuery<ReasonQuery>,
) -> ApiResult<Json<Value>> {
    let why = reason(q.reason.as_deref())?;
    let mut tx = staff_tx(&state, ws, None).await?;
    panel::record_action(
        &mut tx,
        ws,
        staff.id,
        None,
        "correction.list",
        None,
        None,
        &why,
    )
    .await
    .map_err(se)?;
    let rows: Vec<(Uuid, String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, kind, status, reason, result FROM support_corrections ORDER BY created_at DESC",
    )
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(json!(
        rows.into_iter()
            .map(
                |(id, kind, status, reason, result)| json!({"id": id, "kind": kind,
            "status": status, "reason": reason, "result": result})
            )
            .collect::<Vec<_>>()
    )))
}

/// Closes every session of a user of the workspace.
async fn sessions_revoke(
    State(state): State<AppState>,
    StaffCtx(staff): StaffCtx,
    Path((ws, user)): Path<(Uuid, Uuid)>,
    ApiJson(b): ApiJson<ReasonBody>,
) -> ApiResult<Json<Value>> {
    let why = reason(b.reason.as_deref())?;
    let mut tx = staff_tx(&state, ws, Some(user)).await?;
    let member: Option<i32> =
        sqlx::query_scalar("SELECT 1 FROM workspace_members WHERE user_id = $1")
            .bind(user)
            .fetch_optional(&mut *tx)
            .await?;
    if member.is_none() {
        return Err(ApiError::new(ErrorCode::NotFound));
    }
    let n = sqlx::query(
        "UPDATE sessions SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(user)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    panel::record_action(
        &mut tx,
        ws,
        staff.id,
        None,
        "session.revoke",
        Some("user"),
        Some(user),
        &why,
    )
    .await
    .map_err(se)?;
    audit(
        &mut tx,
        ws,
        &staff,
        AuditEvent::SessionRevoke,
        "user",
        Some(user),
        &why,
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"revoked": n})))
}

/// Replaces the data key of a device with a random one. The old key stops working. The new key
/// is not shown to staff: the customer sets a new one in the app.
async fn device_rotate(
    State(state): State<AppState>,
    StaffCtx(staff): StaffCtx,
    Path((ws, device)): Path<(Uuid, Uuid)>,
    ApiJson(b): ApiJson<ReasonBody>,
) -> ApiResult<Response> {
    let why = reason(b.reason.as_deref())?;
    // The new key is dropped: staff never see it. The customer shows a new key in the app.
    let hash = devices::key_hash(&devices::new_key());
    let mut tx = staff_tx(&state, ws, None).await?;
    let n = sqlx::query(
        "UPDATE devices SET ingest_key_hash = $2, ingest_key_rotated_at = now()
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(device)
    .bind(hash)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if n == 0 {
        return Err(ApiError::new(ErrorCode::NotFound));
    }
    panel::record_action(
        &mut tx,
        ws,
        staff.id,
        None,
        "device.rotate_key",
        Some("device"),
        Some(device),
        &why,
    )
    .await
    .map_err(se)?;
    audit(
        &mut tx,
        ws,
        &staff,
        AuditEvent::DeviceKeyRenew,
        "device",
        Some(device),
        &why,
    )
    .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

// --- emergency access ----------------------------------------------------------------------

/// First approval: a staff member asks for emergency access to a workspace.
async fn emergency_open(
    State(state): State<AppState>,
    StaffCtx(staff): StaffCtx,
    Path(ws): Path<Uuid>,
    ApiJson(b): ApiJson<ReasonBody>,
) -> ApiResult<Response> {
    let why = reason(b.reason.as_deref())?;
    let mut tx = staff_tx(&state, ws, None).await?;
    let id = panel::open_emergency(&mut tx, ws, staff.id, &why)
        .await
        .map_err(se)?;
    panel::record_action(
        &mut tx,
        ws,
        staff.id,
        None,
        "emergency_access.request",
        Some("support_emergency_request"),
        Some(id),
        &why,
    )
    .await
    .map_err(se)?;
    tx.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"id": id, "status": "pending"})),
    )
        .into_response())
}

/// Second approval, by a different `support_admin`. It opens a 24 hour grant and tells the
/// customer.
async fn emergency_approve(
    State(state): State<AppState>,
    StaffCtx(staff): StaffCtx,
    Path((ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<Value>> {
    if staff.role != "support_admin" {
        return Err(ApiError::new(ErrorCode::Forbidden));
    }
    let mut tx = staff_tx(&state, ws, None).await?;
    match panel::approve_emergency(&mut tx, ws, id, staff.id)
        .await
        .map_err(se)?
    {
        Approval::Granted(grant) => {
            tx.commit().await?;
            Ok(Json(json!({"grant_id": grant})))
        }
        Approval::SameApprover => {
            Err(
                ApiError::new(ErrorCode::Forbidden).with_errors(vec![FieldIssue::new(
                    "/approver",
                    "must_differ_from_requester",
                )]),
            )
        }
        Approval::AlreadyDecided => Err(ApiError::new(ErrorCode::Conflict)),
        Approval::NotFound => Err(ApiError::new(ErrorCode::NotFound)),
    }
}

/// Routes of this module.
pub fn routes(_state: &AppState) -> Router<AppState> {
    const WS: &str = "/internal/v1/workspaces/{ws}";
    Router::new()
        .route("/internal/v1/workspaces", get(search))
        .route(&format!("{WS}/summary"), get(summary))
        .route(&format!("{WS}/content/{{kind}}"), get(content))
        .route("/internal/v1/requests/{number}", get(request_get))
        .route(
            "/internal/v1/requests/{number}/messages",
            post(request_message),
        )
        .route(
            &format!("{WS}/corrections"),
            post(correction_create).get(correction_list),
        )
        .route(
            &format!("{WS}/users/{{user}}/sessions/revoke"),
            post(sessions_revoke),
        )
        .route(
            &format!("{WS}/devices/{{device}}/rotate-key"),
            post(device_rotate),
        )
        .route(&format!("{WS}/emergency-access"), post(emergency_open))
        .route(
            &format!("{WS}/emergency-access/{{id}}/approve"),
            post(emergency_approve),
        )
}

/// The support panel routes are not in the public `OpenAPI` document.
#[derive(Debug, utoipa::OpenApi)]
#[openapi()]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "internal",
    routes,
    operations: &[],
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
