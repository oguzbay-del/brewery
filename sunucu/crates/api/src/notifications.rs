//! Notifications (T2.24): the inbox and its actions, the notification preferences of the caller
//! and the alert rules.
//!
//! The worker creates and delivers notifications (`notifier` crate). The inbox shows only the
//! notifications of the caller. Actions: `approve` (acknowledged), `postpone` (snoozed until a
//! time) and `ignore` (dismissed).

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
};
use chrono::{DateTime, Duration, NaiveTime, Utc};
use notifier::alert::Condition;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{FromRow, PgConnection};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    context::RequestContext,
    error::{ApiError, ApiJson, ApiQuery, ApiResult, ErrorCode},
    library::{id_cursor, issue, not_found, paging, workspace},
    pagination::{ListParams, Page},
};

/// Permission that writes alert rules.
pub const RULE_PERMISSION: &str = "batch.record";
/// Permission that reads alert rules.
pub const READ_PERMISSION: &str = "recipe.view";

/// A notification of the caller.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct Notification {
    /// Identifier.
    pub id: Uuid,
    /// 1 (critical), 2 (normal) or 3 (low).
    pub priority: i16,
    /// Kind (`alert.threshold`, `reminder`, `support.reply`).
    pub kind: String,
    /// Text key of the client catalog.
    pub text_key: String,
    /// Parameters of the text.
    #[schema(value_type = Object)]
    pub params: Value,
    /// Batch.
    pub batch_id: Option<Uuid>,
    /// Batch step.
    pub batch_step_id: Option<Uuid>,
    /// `open`, `snoozed`, `acknowledged`, `dismissed` or `resolved`.
    pub status: String,
    /// End of the postponement.
    pub snoozed_until: Option<DateTime<Utc>>,
    /// Created at.
    pub created_at: DateTime<Utc>,
    /// Record version.
    pub version: i64,
}

const COLUMNS: &str = "id, priority, kind, text_key, params, batch_id, batch_step_id, status, \
                       snoozed_until, created_at, version";

/// Filter of the inbox.
#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct InboxFilter {
    /// `open`, `snoozed`, `acknowledged`, `dismissed` or `resolved`.
    pub status: Option<String>,
}

/// An action on a notification.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ActionBody {
    /// `approve`, `postpone` or `ignore`.
    pub action: String,
    /// `postpone`: until this time (default one hour, at most 7 days).
    pub until: Option<DateTime<Utc>>,
}

/// Notification preferences of the caller in the workspace.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct Preferences {
    /// IANA time zone for quiet hours and the summary.
    pub timezone: String,
    /// Start of the quiet hours (`HH:MM`).
    pub quiet_start: Option<String>,
    /// End of the quiet hours (`HH:MM`).
    pub quiet_end: Option<String>,
    /// Push on.
    pub push_enabled: bool,
    /// E-mail on.
    pub email_enabled: bool,
    /// Daily summary on.
    pub daily_summary: bool,
    /// Time of the summary (`HH:MM`).
    pub summary_time: String,
}

const PREF_COLUMNS: &str = "timezone, to_char(quiet_start, 'HH24:MI') AS quiet_start, \
    to_char(quiet_end, 'HH24:MI') AS quiet_end, push_enabled, email_enabled, daily_summary, \
    to_char(summary_time, 'HH24:MI') AS summary_time";

/// New values of the preferences. Absent fields keep their value.
#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct PreferencesBody {
    /// IANA time zone.
    pub timezone: Option<String>,
    /// Start of the quiet hours; with `quiet_end`.
    pub quiet_start: Option<String>,
    /// End of the quiet hours; with `quiet_start`.
    pub quiet_end: Option<String>,
    /// Remove the quiet hours.
    pub clear_quiet_hours: Option<bool>,
    /// Push on.
    pub push_enabled: Option<bool>,
    /// E-mail on.
    pub email_enabled: Option<bool>,
    /// Daily summary on.
    pub daily_summary: Option<bool>,
    /// Time of the summary.
    pub summary_time: Option<String>,
}

/// An alert rule.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct AlertRule {
    /// Identifier.
    pub id: Uuid,
    /// Name.
    pub name: String,
    /// Condition (`schema_version`, `measurement`, `op`, `value`, `cooldown_min`).
    #[schema(value_type = Object)]
    pub condition: Value,
    /// Priority of the notification, 1..=3.
    pub priority: i16,
    /// `batch` or `device`; empty for every batch.
    pub scope_kind: Option<String>,
    /// Batch or device.
    pub scope_id: Option<Uuid>,
    /// On.
    pub enabled: bool,
    /// Muted until.
    pub muted_until: Option<DateTime<Utc>>,
    /// Record version.
    pub version: i64,
}

const RULE_COLUMNS: &str =
    "id, name, condition, priority, scope_kind, scope_id, enabled, muted_until, version";

/// A new alert rule.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NewAlertRule {
    /// Name, 1..=200 characters.
    pub name: String,
    /// Condition.
    #[schema(value_type = Object)]
    pub condition: Value,
    /// Priority 1..=3 (default 2).
    pub priority: Option<i16>,
    /// `batch` or `device`.
    pub scope_kind: Option<String>,
    /// Batch or device.
    pub scope_id: Option<Uuid>,
}

/// Changes of an alert rule.
#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct AlertRulePatch {
    /// Name.
    pub name: Option<String>,
    /// Condition.
    #[schema(value_type = Option<Object>)]
    pub condition: Option<Value>,
    /// Priority.
    pub priority: Option<i16>,
    /// On or off.
    pub enabled: Option<bool>,
    /// Mute until; the time must be in the future.
    pub muted_until: Option<DateTime<Utc>>,
}

async fn member_id(tx: &mut PgConnection, ctx: &RequestContext) -> ApiResult<Uuid> {
    sqlx::query_scalar(
        "SELECT id FROM workspace_members WHERE user_id = $1 AND status = 'active'
           AND deleted_at IS NULL",
    )
    .bind(ctx.identity.user_id)
    .fetch_optional(tx)
    .await?
    .ok_or_else(not_found)
}

fn example() -> Value {
    json!({"id": "0192f1c07a3b7000800000000000000b", "priority": 1, "kind": "alert.threshold",
        "text_key": "notification.alert.threshold",
        "params": {"schema_version": 1, "rule_name": "Too warm", "value": 23.4},
        "batch_id": "0192f1c07a3b7000800000000000000c", "batch_step_id": null, "status": "open",
        "snoozed_until": null, "created_at": "2026-11-28T12:00:00Z", "version": 1})
}

// --- inbox --------------------------------------------------------------------------------

/// The inbox of the caller, newest first.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/notifications",
    tag = "notifications",
    operation_id = "notification_list",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("status" = Option<String>, Query, description = "Status filter"),
        ("limit" = Option<u32>, Query, description = "Page size, 1..=200 (default 50)"),
        ("cursor" = Option<String>, Query, description = "`next_cursor` of the previous page")
    ),
    responses((status = 200, description = "A page of notifications", body = Page<Notification>,
        example = json!({"items": [example()], "next_cursor": null})))
)]
pub async fn notification_list(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiQuery(params): ApiQuery<ListParams>,
    ApiQuery(filter): ApiQuery<InboxFilter>,
) -> ApiResult<Json<Page<Notification>>> {
    let paging = paging(&params, &["id"], "-id")?;
    if filter
        .status
        .as_deref()
        .is_some_and(|s| !["open", "snoozed", "acknowledged", "dismissed", "resolved"].contains(&s))
    {
        return Err(issue("status", "invalid_value"));
    }
    let (cmp, dir) = paging.dir();
    let sql = sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM notifications WHERE recipient_member_id = $1 AND deleted_at IS NULL \
           AND ($2::text IS NULL OR status = $2) AND ($3::uuid IS NULL OR id {cmp} $3) \
         ORDER BY id {dir} LIMIT $4"
    ));
    let mut tx = ctx.begin(&state.db).await?;
    let member = member_id(&mut tx, &ctx).await?;
    let rows: Vec<Notification> = sqlx::query_as(sql)
        .bind(member)
        .bind(&filter.status)
        .bind(paging.id)
        .bind(i64::from(paging.limit) + 1)
        .fetch_all(&mut *tx)
        .await?;
    Ok(Json(Page::from_rows(
        rows,
        paging.limit,
        &paging.sort,
        |r| id_cursor(r.id),
    )))
}

/// Approves, postpones or ignores a notification.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/notifications/{id}/actions",
    tag = "notifications",
    operation_id = "notification_action",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Notification")
    ),
    request_body = ActionBody,
    responses((status = 200, description = "The notification after the action",
        body = Notification, example = json!(example())))
)]
pub async fn notification_action(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((ws, id)): Path<(Uuid, Uuid)>,
    ApiJson(body): ApiJson<ActionBody>,
) -> ApiResult<Json<Notification>> {
    let now = Utc::now();
    let (status, event, until) = match body.action.as_str() {
        "approve" => ("acknowledged", "acknowledged", None),
        "ignore" => ("dismissed", "dismissed", None),
        "postpone" => {
            let until = body.until.unwrap_or_else(|| now + Duration::hours(1));
            if until <= now || until > now + Duration::days(7) {
                return Err(issue("/until", "out_of_range"));
            }
            ("snoozed", "snoozed", Some(until))
        }
        _ => return Err(issue("/action", "invalid_value")),
    };
    let mut tx = ctx.begin(&state.db).await?;
    let member = member_id(&mut tx, &ctx).await?;
    let sql = sqlx::AssertSqlSafe(format!(
        "UPDATE notifications SET status = $3, snoozed_until = $4, updated_by = $5
         WHERE id = $1 AND recipient_member_id = $2 AND deleted_at IS NULL
           AND status IN ('open', 'snoozed') RETURNING {COLUMNS}"
    ));
    let updated: Option<Notification> = sqlx::query_as(sql)
        .bind(id)
        .bind(member)
        .bind(status)
        .bind(until)
        .bind(ctx.identity.user_id)
        .fetch_optional(&mut *tx)
        .await?;
    let Some(updated) = updated else {
        let exists: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM notifications WHERE id = $1 AND recipient_member_id = $2
               AND deleted_at IS NULL",
        )
        .bind(id)
        .bind(member)
        .fetch_optional(&mut *tx)
        .await?;
        return Err(if exists.is_some() {
            ApiError::new(ErrorCode::Conflict)
        } else {
            not_found()
        });
    };
    sqlx::query(
        "INSERT INTO notification_events (id, workspace_id, notification_id, kind, created_by)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::now_v7())
    .bind(ws)
    .bind(id)
    .bind(event)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(updated))
}

// --- preferences --------------------------------------------------------------------------

fn parse_time(field: &str, v: &str) -> ApiResult<NaiveTime> {
    NaiveTime::parse_from_str(v, "%H:%M").map_err(|_| issue(field, "invalid_value"))
}

/// The notification preferences of the caller. Defaults when nothing is saved.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/me/notification-preferences",
    tag = "notifications",
    operation_id = "notificationpreference_get",
    params(("workspace_id" = Uuid, Path, description = "Workspace")),
    responses((status = 200, description = "The preferences", body = Preferences,
        example = json!({"timezone": "America/New_York", "quiet_start": "22:00",
            "quiet_end": "07:00", "push_enabled": true, "email_enabled": true,
            "daily_summary": false, "summary_time": "08:00"})))
)]
pub async fn notificationpreference_get(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> ApiResult<Json<Preferences>> {
    let ws = workspace(&ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    Ok(Json(
        read_preferences(&mut tx, ws, ctx.identity.user_id).await?,
    ))
}

async fn read_preferences(tx: &mut PgConnection, ws: Uuid, user: Uuid) -> ApiResult<Preferences> {
    let sql = sqlx::AssertSqlSafe(format!(
        "SELECT {PREF_COLUMNS} FROM notification_preferences
         WHERE workspace_id = $1 AND user_id = $2 AND deleted_at IS NULL"
    ));
    let row: Option<Preferences> = sqlx::query_as(sql)
        .bind(ws)
        .bind(user)
        .fetch_optional(&mut *tx)
        .await?;
    if let Some(row) = row {
        return Ok(row);
    }
    let timezone: String = sqlx::query_scalar("SELECT time_zone FROM workspaces WHERE id = $1")
        .bind(ws)
        .fetch_one(tx)
        .await?;
    Ok(Preferences {
        timezone,
        quiet_start: None,
        quiet_end: None,
        push_enabled: true,
        email_enabled: true,
        daily_summary: false,
        summary_time: "08:00".to_owned(),
    })
}

/// Changes the notification preferences of the caller.
#[utoipa::path(
    put,
    path = "/v1/workspaces/{workspace_id}/me/notification-preferences",
    tag = "notifications",
    operation_id = "notificationpreference_put",
    params(("workspace_id" = Uuid, Path, description = "Workspace")),
    request_body = PreferencesBody,
    responses((status = 200, description = "The preferences after the change", body = Preferences,
        example = json!({"timezone": "America/New_York", "quiet_start": "22:00",
            "quiet_end": "07:00", "push_enabled": true, "email_enabled": true,
            "daily_summary": true, "summary_time": "08:00"})))
)]
pub async fn notificationpreference_put(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<PreferencesBody>,
) -> ApiResult<Json<Preferences>> {
    let ws = workspace(&ctx)?;
    let user = ctx.identity.user_id;
    let mut tx = ctx.begin(&state.db).await?;
    let mut p = read_preferences(&mut tx, ws, user).await?;
    if let Some(tz) = body.timezone {
        let known: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_timezone_names WHERE name = $1)")
                .bind(&tz)
                .fetch_one(&mut *tx)
                .await?;
        if !known {
            return Err(issue("/timezone", "invalid_value"));
        }
        p.timezone = tz;
    }
    if body.clear_quiet_hours == Some(true) {
        p.quiet_start = None;
        p.quiet_end = None;
    }
    match (body.quiet_start, body.quiet_end) {
        (None, None) => {}
        (Some(a), Some(b)) => {
            parse_time("/quiet_start", &a)?;
            parse_time("/quiet_end", &b)?;
            p.quiet_start = Some(a);
            p.quiet_end = Some(b);
        }
        _ => return Err(issue("/quiet_end", "required_together")),
    }
    if let Some(v) = body.push_enabled {
        p.push_enabled = v;
    }
    if let Some(v) = body.email_enabled {
        p.email_enabled = v;
    }
    if let Some(v) = body.daily_summary {
        p.daily_summary = v;
    }
    if let Some(v) = body.summary_time {
        parse_time("/summary_time", &v)?;
        p.summary_time = v;
    }
    let sql = sqlx::AssertSqlSafe(format!(
        "INSERT INTO notification_preferences
             (id, workspace_id, user_id, timezone, quiet_start, quiet_end, push_enabled,
              email_enabled, daily_summary, summary_time, created_by, updated_by)
         VALUES ($1, $2, $3, $4, $5::time, $6::time, $7, $8, $9, $10::time, $3, $3)
         ON CONFLICT (workspace_id, user_id) DO UPDATE SET
             timezone = EXCLUDED.timezone, quiet_start = EXCLUDED.quiet_start,
             quiet_end = EXCLUDED.quiet_end, push_enabled = EXCLUDED.push_enabled,
             email_enabled = EXCLUDED.email_enabled, daily_summary = EXCLUDED.daily_summary,
             summary_time = EXCLUDED.summary_time, updated_by = EXCLUDED.updated_by
         RETURNING {PREF_COLUMNS}"
    ));
    let saved: Preferences = sqlx::query_as(sql)
        .bind(Uuid::now_v7())
        .bind(ws)
        .bind(user)
        .bind(&p.timezone)
        .bind(&p.quiet_start)
        .bind(&p.quiet_end)
        .bind(p.push_enabled)
        .bind(p.email_enabled)
        .bind(p.daily_summary)
        .bind(&p.summary_time)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(saved))
}

// --- alert rules --------------------------------------------------------------------------

fn rule_example() -> Value {
    json!({"id": "0192f1c07a3b7000800000000000000b", "name": "Too warm",
        "condition": {"schema_version": 1, "measurement": "temp_c", "op": "gt", "value": 22.0,
            "cooldown_min": 60},
        "priority": 1, "scope_kind": null, "scope_id": null, "enabled": true,
        "muted_until": null, "version": 1})
}

fn check_priority(p: i16) -> ApiResult<()> {
    if (1..=3).contains(&p) {
        Ok(())
    } else {
        Err(issue("/priority", "out_of_range"))
    }
}

fn check_condition(v: &Value) -> ApiResult<()> {
    Condition::parse(v)
        .map(|_| ())
        .map_err(|pointer| issue(pointer, "invalid_value"))
}

/// The alert rules of the workspace.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/alert-rules",
    tag = "notifications",
    operation_id = "alertrule_list",
    params(("workspace_id" = Uuid, Path, description = "Workspace")),
    responses((status = 200, description = "The rules", body = Vec<AlertRule>,
        example = json!([rule_example()])))
)]
pub async fn alertrule_list(
    State(state): State<AppState>,
    ctx: RequestContext,
) -> ApiResult<Json<Vec<AlertRule>>> {
    ctx.require(READ_PERMISSION)?;
    let sql = sqlx::AssertSqlSafe(format!(
        "SELECT {RULE_COLUMNS} FROM alert_rules WHERE deleted_at IS NULL ORDER BY id"
    ));
    let mut tx = ctx.begin(&state.db).await?;
    Ok(Json(sqlx::query_as(sql).fetch_all(&mut *tx).await?))
}

/// Creates an alert rule.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/alert-rules",
    tag = "notifications",
    operation_id = "alertrule_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = NewAlertRule,
    responses((status = 201, description = "The rule", body = AlertRule, example = json!(rule_example())))
)]
pub async fn alertrule_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(new): ApiJson<NewAlertRule>,
) -> ApiResult<Response> {
    ctx.require(RULE_PERMISSION)?;
    let ws = workspace(&ctx)?;
    if !(1..=200).contains(&new.name.chars().count()) {
        return Err(issue("/name", "length_out_of_range"));
    }
    check_condition(&new.condition)?;
    let priority = new.priority.unwrap_or(2);
    check_priority(priority)?;
    if new.scope_kind.is_some() != new.scope_id.is_some()
        || new
            .scope_kind
            .as_deref()
            .is_some_and(|k| !["batch", "device"].contains(&k))
    {
        return Err(issue("/scope_kind", "invalid_value"));
    }
    let sql = sqlx::AssertSqlSafe(format!(
        "INSERT INTO alert_rules (id, workspace_id, name, condition, priority, scope_kind,
             scope_id, created_by, updated_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8) RETURNING {RULE_COLUMNS}"
    ));
    let mut tx = ctx.begin(&state.db).await?;
    let rule: AlertRule = sqlx::query_as(sql)
        .bind(Uuid::now_v7())
        .bind(ws)
        .bind(&new.name)
        .bind(&new.condition)
        .bind(priority)
        .bind(&new.scope_kind)
        .bind(new.scope_id)
        .bind(ctx.identity.user_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            if crate::library::sqlstate(&e).as_deref() == Some("23503") {
                issue("/scope_id", "not_found")
            } else {
                e.into()
            }
        })?;
    tx.commit().await?;
    let location = format!("/v1/workspaces/{ws}/alert-rules/{}", rule.id);
    Ok(crate::library::created_response(
        &location,
        rule.version,
        &rule,
    ))
}

/// Changes an alert rule.
#[utoipa::path(
    patch,
    path = "/v1/workspaces/{workspace_id}/alert-rules/{id}",
    tag = "notifications",
    operation_id = "alertrule_update",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Rule")
    ),
    request_body = AlertRulePatch,
    responses((status = 200, description = "The rule", body = AlertRule, example = json!(rule_example())))
)]
pub async fn alertrule_update(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
    ApiJson(patch): ApiJson<AlertRulePatch>,
) -> ApiResult<Json<AlertRule>> {
    ctx.require(RULE_PERMISSION)?;
    if let Some(name) = &patch.name
        && !(1..=200).contains(&name.chars().count())
    {
        return Err(issue("/name", "length_out_of_range"));
    }
    if let Some(c) = &patch.condition {
        check_condition(c)?;
    }
    if let Some(p) = patch.priority {
        check_priority(p)?;
    }
    if patch.muted_until.is_some_and(|t| t <= Utc::now()) {
        return Err(issue("/muted_until", "must_be_in_the_future"));
    }
    let sql = sqlx::AssertSqlSafe(format!(
        "UPDATE alert_rules SET name = COALESCE($2, name), condition = COALESCE($3, condition),
             priority = COALESCE($4, priority), enabled = COALESCE($5, enabled),
             muted_until = COALESCE($6, muted_until), updated_by = $7
         WHERE id = $1 AND deleted_at IS NULL RETURNING {RULE_COLUMNS}"
    ));
    let mut tx = ctx.begin(&state.db).await?;
    let rule: AlertRule = sqlx::query_as(sql)
        .bind(id)
        .bind(&patch.name)
        .bind(&patch.condition)
        .bind(patch.priority)
        .bind(patch.enabled)
        .bind(patch.muted_until)
        .bind(ctx.identity.user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(not_found)?;
    tx.commit().await?;
    Ok(Json(rule))
}

/// Deletes an alert rule.
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{workspace_id}/alert-rules/{id}",
    tag = "notifications",
    operation_id = "alertrule_delete",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Rule")
    ),
    responses((status = 204, description = "Deleted"))
)]
pub async fn alertrule_delete(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Response> {
    ctx.require(RULE_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let n = sqlx::query(
        "UPDATE alert_rules SET deleted_at = now(), updated_by = $2
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if n == 0 {
        return Err(not_found());
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Routes of this module.
pub fn routes(state: &AppState) -> Router<AppState> {
    const WS: &str = "/v1/workspaces/{ws}";
    let creates = crate::idempotent(
        state,
        Router::new().route(&format!("{WS}/alert-rules"), post(alertrule_create)),
    );
    let others = Router::new()
        .route(&format!("{WS}/notifications"), get(notification_list))
        .route(
            &format!("{WS}/notifications/{{id}}/actions"),
            post(notification_action),
        )
        .route(
            &format!("{WS}/me/notification-preferences"),
            get(notificationpreference_get).put(notificationpreference_put),
        )
        .route(&format!("{WS}/alert-rules"), get(alertrule_list))
        .route(
            &format!("{WS}/alert-rules/{{id}}"),
            patch(alertrule_update).delete(alertrule_delete),
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
        file_op("notification_list", None, None, READ),
        file_op(
            "notification_action",
            None,
            Some(Concurrency::FieldLww),
            WRITE,
        ),
        file_op("notificationpreference_get", None, None, READ),
        file_op(
            "notificationpreference_put",
            None,
            Some(Concurrency::FieldLww),
            WRITE,
        ),
        file_op("alertrule_list", Some("recipe.view"), None, READ),
        file_op(
            "alertrule_create",
            Some("batch.record"),
            Some(Concurrency::Append),
            CREATE,
        ),
        file_op(
            "alertrule_update",
            Some("batch.record"),
            Some(Concurrency::FieldLww),
            WRITE,
        ),
        file_op(
            "alertrule_delete",
            Some("batch.record"),
            Some(Concurrency::FieldLww),
            WRITE,
        ),
    ];
}

/// `OpenAPI` fragment of this module.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(
        notification_list,
        notification_action,
        notificationpreference_get,
        notificationpreference_put,
        alertrule_list,
        alertrule_create,
        alertrule_update,
        alertrule_delete
    ),
    components(schemas(
        Notification,
        ActionBody,
        Preferences,
        PreferencesBody,
        AlertRule,
        NewAlertRule,
        AlertRulePatch
    )),
    tags((name = "notifications", description = "Notification inbox, preferences and alert rules"))
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "notifications",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
