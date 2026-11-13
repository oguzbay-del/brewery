//! First workspace of a user: `POST /v1/workspaces` (ADR-0009 rule 1.6: the account exists
//! after the first sign-in; the workspace is opened by the setup screen with mode and region).

use axum::{Json, http::StatusCode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::tokens::{hex, random_bytes};
use crate::{
    context::RequestContext,
    error::{ApiError, ApiJson, ApiResult, FieldIssue},
};

/// Every permission code (ADR-0009 rule 8.1). The administrator role of a new workspace holds
/// all of them; the other preset roles are seeded by T2.13.
pub const ALL_PERMISSIONS: [&str; 17] = [
    "recipe.view",
    "recipe.edit",
    "recipe.approve",
    "batch.plan",
    "batch.record",
    "batch.correct_closed",
    "quality.record",
    "release.approve",
    "stock.move",
    "cost.view",
    "reports.export",
    "settings.equipment",
    "users.manage",
    "workspace.manage",
    "billing.manage",
    "support.grant",
    "devices.manage",
];

/// Most workspaces one user may belong to.
const MAX_WORKSPACES_PER_USER: i64 = 20;

/// Body of `POST /v1/workspaces`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct WorkspaceCreateRequest {
    /// Name (1 to 200 characters).
    pub name: String,
    /// `home` or `business`.
    pub mode: String,
    /// Data region: `us` or `uk`.
    pub region: String,
    /// ISO 3166-1 alpha-2 country, upper case.
    pub country: String,
    /// IANA time zone name.
    pub time_zone: String,
}

/// A workspace.
#[derive(Debug, Serialize, ToSchema)]
pub struct WorkspaceResponse {
    /// Workspace id.
    pub id: Uuid,
    /// Name.
    pub name: String,
    /// `home` or `business`.
    pub mode: String,
    /// `us` or `uk`.
    pub region: String,
    /// Country.
    pub country: String,
    /// Time zone.
    pub time_zone: String,
    /// Creation time.
    pub created_at: DateTime<Utc>,
}

fn validate(req: &WorkspaceCreateRequest) -> Vec<FieldIssue> {
    let mut issues = Vec::new();
    let name = req.name.trim();
    if name.is_empty() || name.chars().count() > 200 || name.chars().any(char::is_control) {
        issues.push(FieldIssue::new("/name", "invalid"));
    }
    if !matches!(req.mode.as_str(), "home" | "business") {
        issues.push(FieldIssue::new("/mode", "invalid_choice"));
    }
    if !matches!(req.region.as_str(), "us" | "uk") {
        issues.push(FieldIssue::new("/region", "invalid_choice"));
    }
    if req.country.len() != 2 || !req.country.bytes().all(|b| b.is_ascii_uppercase()) {
        issues.push(FieldIssue::new("/country", "invalid"));
    }
    let tz = &req.time_zone;
    if tz.is_empty()
        || tz.len() > 64
        || !tz
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'/' | b'+' | b'-'))
    {
        issues.push(FieldIssue::new("/time_zone", "invalid"));
    }
    issues
}

/// Opens a workspace; the caller becomes its first member with the administrator role.
#[utoipa::path(
    post,
    path = "/v1/workspaces",
    tag = "workspaces",
    operation_id = "workspaces_create",
    request_body = WorkspaceCreateRequest,
    responses((
        status = 201,
        description = "Workspace created",
        body = WorkspaceResponse,
        example = json!({
            "id": "0192f1c0-7a3b-7000-8000-00000000000c", "name": "Home brewery", "mode": "home",
            "region": "us", "country": "US", "time_zone": "America/New_York",
            "created_at": "2026-11-13T10:00:00Z"
        })
    ))
)]
pub async fn workspace_create(
    ctx: RequestContext,
    axum::extract::State(state): axum::extract::State<crate::AppState>,
    ApiJson(req): ApiJson<WorkspaceCreateRequest>,
) -> ApiResult<(StatusCode, Json<WorkspaceResponse>)> {
    let issues = validate(&req);
    if !issues.is_empty() {
        return Err(ApiError::validation(issues));
    }
    let user_id = ctx.identity.user_id;
    let (workspace_id, member_id, role_id) = (Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7());
    let name = req.name.trim().to_owned();

    // The workspace id goes into the context first: the row policies check it on insert.
    let mut tx = db::begin(&state.db, &db::DbContext::user(user_id, Some(workspace_id))).await?;
    let memberships: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM workspace_members WHERE user_id = $1 AND deleted_at IS NULL",
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;
    if memberships >= MAX_WORKSPACES_PER_USER {
        return Err(ApiError::new(crate::ErrorCode::Conflict)
            .with_errors(vec![FieldIssue::new("/", "workspace_limit")]));
    }
    let created_at: DateTime<Utc> = sqlx::query_scalar(
        "INSERT INTO workspaces (id, name, mode, region, country, time_zone, support_pseudonym, \
                                 created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8) RETURNING created_at",
    )
    .bind(workspace_id)
    .bind(&name)
    .bind(&req.mode)
    .bind(&req.region)
    .bind(&req.country)
    .bind(&req.time_zone)
    .bind(format!("ws-{}", hex(&random_bytes::<8>())))
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO workspace_members (id, workspace_id, user_id, created_by, updated_by) \
         VALUES ($1, $2, $3, $3, $3)",
    )
    .bind(member_id)
    .bind(workspace_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO roles (id, workspace_id, name, template_key, created_by, updated_by) \
         VALUES ($1, $2, 'Administrator', 'admin', $3, $3)",
    )
    .bind(role_id)
    .bind(workspace_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO role_permissions (workspace_id, role_id, permission, created_by) \
         SELECT $1, $2, p, $3 FROM unnest($4::text[]) AS p",
    )
    .bind(workspace_id)
    .bind(role_id)
    .bind(user_id)
    .bind(ALL_PERMISSIONS.map(str::to_owned).to_vec())
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO role_assignments (id, workspace_id, member_id, role_id, created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $5)",
    )
    .bind(Uuid::now_v7())
    .bind(workspace_id)
    .bind(member_id)
    .bind(role_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    tracing::info!(user_id = %user_id, workspace_id = %workspace_id, "workspace created");

    Ok((
        StatusCode::CREATED,
        Json(WorkspaceResponse {
            id: workspace_id,
            name,
            mode: req.mode,
            region: req.region,
            country: req.country,
            time_zone: req.time_zone,
            created_at,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req() -> WorkspaceCreateRequest {
        WorkspaceCreateRequest {
            name: "Brewery".into(),
            mode: "home".into(),
            region: "uk".into(),
            country: "GB".into(),
            time_zone: "Europe/London".into(),
        }
    }

    #[test]
    fn valid_request_has_no_issues() {
        assert!(validate(&req()).is_empty());
    }

    #[test]
    fn every_bad_field_is_reported() {
        let bad = WorkspaceCreateRequest {
            name: " ".into(),
            mode: "x".into(),
            region: "eu".into(),
            country: "gb".into(),
            time_zone: "no spaces".into(),
        };
        let fields: Vec<_> = validate(&bad).into_iter().map(|i| i.field).collect();
        assert_eq!(
            fields,
            ["/name", "/mode", "/region", "/country", "/time_zone"]
        );
    }

    #[test]
    fn permission_list_matches_the_database_check() {
        assert_eq!(ALL_PERMISSIONS.len(), 17);
        let unique: std::collections::HashSet<_> = ALL_PERMISSIONS.iter().collect();
        assert_eq!(unique.len(), 17);
    }
}
