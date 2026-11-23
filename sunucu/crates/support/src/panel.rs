//! Support panel logic (T2.30): grants, the action log, corrections and emergency access.
//!
//! - Content of a workspace is open to support staff only while a `support_grants` row is valid
//!   ([`active_grant`]): not revoked, started, not ended. The customer gives it (72 hours at the
//!   most), or two different staff members open an emergency grant ([`approve_emergency`]).
//! - Every operation writes a `support_actions` row and an audit row ([`record_action`]).
//! - Only `app_worker` writes `subscriptions` and `entitlements`. Staff ask for a correction with
//!   a reason ([`request_correction`]); [`CorrectionHandler`] applies it in the worker.

use db::{
    ActorKind, AuditEvent, NewAuditEntry,
    outbox::{Entity, NewOutboxEvent, enqueue},
};
use events::consumer::{BoxFuture, Event, Handler, HandlerError};
use notifier::create::{NewNotification, create};
use serde_json::{Value, json};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{Result, enter_workspace};

/// Outbox topic: a staff member asked for a correction.
pub const CORRECTION_TOPIC: &str = "support.correction_requested";
/// Entity kind of the correction event.
pub const CORRECTION_ENTITY: &str = "support_correction";
/// Durable name of the correction consumer.
pub const CORRECTION_CONSUMER: &str = "support-corrections";
/// Outbox topic for a later subscription re-verification with the store.
pub const REVERIFY_TOPIC: &str = "subscription.reverify_requested";
/// Hours an emergency grant lasts.
pub const EMERGENCY_HOURS: i64 = 24;
/// Feature key of the paid entitlement (`billing::PAID_FEATURE`).
const PAID_FEATURE: &str = "paid_features";

/// Subject filter of the correction events.
#[must_use]
pub fn correction_filter() -> String {
    format!("{}.*.{CORRECTION_ENTITY}.>", events::SUBJECT_PREFIX)
}

/// A support staff member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Staff {
    /// `staff_users.id`.
    pub id: Uuid,
    /// `support` or `support_admin`.
    pub role: String,
}

/// Finds an active staff member by the e-mail of the company account.
///
/// # Errors
/// Database errors.
pub async fn find_staff(conn: &mut PgConnection, email: &str) -> Result<Option<Staff>> {
    let row: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT id, role FROM staff_users
         WHERE email = lower($1) AND status = 'active' AND deleted_at IS NULL",
    )
    .bind(email)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(row.map(|(id, role)| Staff { id, role }))
}

/// The valid grant of a workspace, if any. Call it in the context of the workspace.
///
/// # Errors
/// Database errors.
pub async fn active_grant(conn: &mut PgConnection, workspace_id: Uuid) -> Result<Option<Uuid>> {
    Ok(sqlx::query_scalar(
        "SELECT id FROM support_grants
         WHERE workspace_id = $1 AND revoked_at IS NULL AND deleted_at IS NULL
           AND starts_at <= now() AND ends_at > now()
         ORDER BY ends_at DESC LIMIT 1",
    )
    .bind(workspace_id)
    .fetch_optional(&mut *conn)
    .await?)
}

/// Writes the `support_actions` row and the audit row of one operation. Call it in the context of
/// the workspace, in a read-write transaction.
///
/// # Errors
/// Database errors.
#[allow(clippy::too_many_arguments)]
pub async fn record_action(
    conn: &mut PgConnection,
    workspace_id: Uuid,
    staff_id: Uuid,
    request_id: Option<Uuid>,
    action: &str,
    record_kind: Option<&str>,
    record_id: Option<Uuid>,
    reason: &str,
) -> Result<Uuid> {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO support_actions
             (id, workspace_id, staff_id, request_id, action, record_kind, record_id, reason)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(id)
    .bind(workspace_id)
    .bind(staff_id)
    .bind(request_id)
    .bind(action)
    .bind(record_kind)
    .bind(record_id)
    .bind(reason)
    .execute(&mut *conn)
    .await?;
    db::append_audit(
        conn,
        workspace_id,
        &NewAuditEntry {
            actor_kind: ActorKind::Support,
            actor_id: Some(staff_id),
            event: AuditEvent::SupportAction,
            record_kind: "support_action".to_owned(),
            record_id: Some(id),
            previous_value: None,
            new_value: Some(json!({"action": action, "record_kind": record_kind,
                "record_id": record_id})),
            reason: Some(reason.to_owned()),
        },
    )
    .await?;
    Ok(id)
}

/// Writes a correction request and its outbox event. Call it in the context of the workspace.
///
/// # Errors
/// Database errors.
pub async fn request_correction(
    conn: &mut PgConnection,
    workspace_id: Uuid,
    staff_id: Uuid,
    kind: &str,
    params: &Value,
    reason: &str,
) -> Result<Uuid> {
    let id = Uuid::now_v7();
    let mut params = params.clone();
    if let Value::Object(map) = &mut params {
        map.entry("schema_version").or_insert(json!(1));
    }
    sqlx::query(
        "INSERT INTO support_corrections (id, workspace_id, staff_id, kind, params, reason)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(workspace_id)
    .bind(staff_id)
    .bind(kind)
    .bind(&params)
    .bind(reason)
    .execute(&mut *conn)
    .await?;
    enqueue(
        conn,
        &NewOutboxEvent {
            topic: CORRECTION_TOPIC.to_owned(),
            workspace_id: Some(workspace_id),
            entity: Some(Entity {
                kind: CORRECTION_ENTITY.to_owned(),
                id,
                version: 1,
            }),
            data: None,
        },
    )
    .await?;
    Ok(id)
}

/// Applies one correction (role `app_worker`). Call it in the context of the workspace. Returns
/// the text of the result, or `None` when the correction is not pending or waits for a person.
///
/// # Errors
/// Database errors.
#[allow(clippy::too_many_lines)]
pub async fn apply_correction(
    conn: &mut PgConnection,
    workspace_id: Uuid,
    id: Uuid,
) -> Result<Option<String>> {
    let row: Option<(String, Value, String, Uuid, String)> = sqlx::query_as(
        "SELECT kind, params, reason, staff_id, status FROM support_corrections
         WHERE id = $1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *conn)
    .await?;
    let Some((kind, params, reason, staff, status)) = row else {
        return Ok(None);
    };
    if status != "pending" {
        return Ok(None);
    }
    let (result, event) = match kind.as_str() {
        "entitlement_grant" => {
            let until = params
                .get("valid_until")
                .and_then(Value::as_str)
                .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok());
            let entitlement = Uuid::now_v7();
            sqlx::query(
                "INSERT INTO entitlements (id, workspace_id, feature, source, valid_from,
                     valid_until, reason, corrected_by_staff_id)
                 VALUES ($1, $2, $3, 'manual', now(), COALESCE($4, now() + interval '30 days'),
                         $5, $6)",
            )
            .bind(entitlement)
            .bind(workspace_id)
            .bind(PAID_FEATURE)
            .bind(until)
            .bind(&reason)
            .bind(staff)
            .execute(&mut *conn)
            .await?;
            (
                "manual entitlement written".to_owned(),
                Some((
                    AuditEvent::EntitlementManualAdjust,
                    "entitlement",
                    Some(entitlement),
                )),
            )
        }
        "entitlement_revoke" => {
            let n = sqlx::query(
                "UPDATE entitlements SET valid_until = now(), reason = $2,
                     corrected_by_staff_id = $3
                 WHERE feature = $1 AND deleted_at IS NULL AND valid_from < now()
                   AND (valid_until IS NULL OR valid_until > now())",
            )
            .bind(PAID_FEATURE)
            .bind(&reason)
            .bind(staff)
            .execute(&mut *conn)
            .await?
            .rows_affected();
            (
                format!("{n} entitlements ended"),
                Some((AuditEvent::EntitlementManualAdjust, "entitlement", None)),
            )
        }
        "subscription_reverify" => {
            enqueue(
                conn,
                &NewOutboxEvent {
                    topic: REVERIFY_TOPIC.to_owned(),
                    workspace_id: Some(workspace_id),
                    entity: None,
                    data: Some(json!({"correction_id": id})),
                },
            )
            .await?;
            (
                "re-verification queued".to_owned(),
                Some((AuditEvent::SubscriptionManualAdjust, "subscription", None)),
            )
        }
        "deletion_request" => {
            let grace_days = db::account_deletion_grace_days()?;
            let request = db::account_deletion_from_support(conn, id, grace_days).await?;
            (
                format!("account deletion scheduled until {}", request.purge_at),
                Some((
                    AuditEvent::AccountDelete,
                    "account_deletion_request",
                    Some(request.id),
                )),
            )
        }
        _ => return Ok(None),
    };
    if let Some((audit_event, record_kind, record_id)) = event {
        db::append_audit(
            conn,
            workspace_id,
            &NewAuditEntry {
                actor_kind: ActorKind::Support,
                actor_id: Some(staff),
                event: audit_event,
                record_kind: record_kind.to_owned(),
                record_id,
                previous_value: None,
                new_value: Some(json!({"correction_id": id, "kind": kind})),
                reason: Some(reason),
            },
        )
        .await?;
    }
    sqlx::query(
        "UPDATE support_corrections
         SET status = 'applied', result = $2, applied_at = now(),
             params = CASE WHEN kind = 'deletion_request' THEN params - 'user_id' ELSE params END
         WHERE id = $1",
    )
    .bind(id)
    .bind(&result)
    .execute(&mut *conn)
    .await?;
    Ok(Some(result))
}

/// Consumer of `support.correction_requested`.
#[derive(Debug, Clone, Copy, Default)]
pub struct CorrectionHandler;

impl Handler for CorrectionHandler {
    fn handle<'a>(
        &'a self,
        event: &'a Event,
        tx: &'a mut PgConnection,
    ) -> BoxFuture<'a, std::result::Result<(), HandlerError>> {
        Box::pin(async move {
            if event.topic != CORRECTION_TOPIC {
                return Ok(());
            }
            let (Some(ws), Some(id)) = (
                event.workspace_id,
                event
                    .body
                    .pointer("/entity/id")
                    .and_then(Value::as_str)
                    .and_then(|s| Uuid::parse_str(s).ok()),
            ) else {
                return Err(HandlerError::Permanent(
                    "event has no correction".to_owned(),
                ));
            };
            enter_workspace(tx, ws).await?;
            apply_correction(tx, ws, id).await?;
            Ok(())
        })
    }
}

/// Opens an emergency access request (the first of the two approvals). Call it in the context of
/// the workspace.
///
/// # Errors
/// Database errors.
pub async fn open_emergency(
    conn: &mut PgConnection,
    workspace_id: Uuid,
    staff_id: Uuid,
    reason: &str,
) -> Result<Uuid> {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO support_emergency_requests (id, workspace_id, requested_by_staff_id, reason)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(workspace_id)
    .bind(staff_id)
    .bind(reason)
    .execute(&mut *conn)
    .await?;
    Ok(id)
}

/// What [`approve_emergency`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Approval {
    /// The grant is open.
    Granted(Uuid),
    /// Another approval came first.
    AlreadyDecided,
    /// The approver is the person who asked.
    SameApprover,
    /// No such request in the workspace.
    NotFound,
}

/// The second approval of an emergency request. It opens a grant (24 hours) and tells the
/// customer (inbox, push, e-mail). Call it in the context of the workspace.
///
/// # Errors
/// Database errors.
#[allow(clippy::too_many_lines)]
pub async fn approve_emergency(
    conn: &mut PgConnection,
    workspace_id: Uuid,
    request_id: Uuid,
    approver: Uuid,
) -> Result<Approval> {
    let row: Option<(Uuid, String, String)> = sqlx::query_as(
        "SELECT requested_by_staff_id, status, reason FROM support_emergency_requests
         WHERE id = $1 FOR UPDATE",
    )
    .bind(request_id)
    .fetch_optional(&mut *conn)
    .await?;
    let Some((requester, status, reason)) = row else {
        return Ok(Approval::NotFound);
    };
    if status != "pending" {
        return Ok(Approval::AlreadyDecided);
    }
    if requester == approver {
        return Ok(Approval::SameApprover);
    }
    let grant = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO support_grants (id, workspace_id, starts_at, ends_at, emergency,
             approver_1_staff_id, approver_2_staff_id, customer_notified_at)
         VALUES ($1, $2, now(), now() + make_interval(hours => $3), true, $4, $5, now())",
    )
    .bind(grant)
    .bind(workspace_id)
    .bind(i32::try_from(EMERGENCY_HOURS).unwrap_or(24))
    .bind(requester)
    .bind(approver)
    .execute(&mut *conn)
    .await?;
    sqlx::query(
        "UPDATE support_emergency_requests
         SET status = 'approved', approved_by_staff_id = $2, grant_id = $3 WHERE id = $1",
    )
    .bind(request_id)
    .bind(approver)
    .bind(grant)
    .execute(&mut *conn)
    .await?;
    record_action(
        conn,
        workspace_id,
        approver,
        None,
        "emergency_access.approve",
        Some("support_grant"),
        Some(grant),
        &reason,
    )
    .await?;
    db::append_audit(
        conn,
        workspace_id,
        &NewAuditEntry {
            actor_kind: ActorKind::Support,
            actor_id: Some(approver),
            event: AuditEvent::SupportGrantChange,
            record_kind: "support_grant".to_owned(),
            record_id: Some(grant),
            previous_value: None,
            new_value: Some(json!({"emergency": true, "requested_by": requester})),
            reason: Some(reason),
        },
    )
    .await?;

    // The customer learns afterwards: members who hold `support.grant`, else every member.
    let mut members: Vec<Uuid> = sqlx::query_scalar(
        "SELECT DISTINCT m.id FROM workspace_members m
         JOIN role_assignments ra ON ra.workspace_id = m.workspace_id AND ra.member_id = m.id
              AND ra.deleted_at IS NULL
         JOIN role_permissions rp ON rp.workspace_id = ra.workspace_id AND rp.role_id = ra.role_id
         WHERE m.workspace_id = $1 AND m.status = 'active' AND m.deleted_at IS NULL
           AND rp.permission = 'support.grant'",
    )
    .bind(workspace_id)
    .fetch_all(&mut *conn)
    .await?;
    if members.is_empty() {
        members = sqlx::query_scalar(
            "SELECT id FROM workspace_members
             WHERE workspace_id = $1 AND status = 'active' AND deleted_at IS NULL",
        )
        .bind(workspace_id)
        .fetch_all(&mut *conn)
        .await?;
    }
    for member in members {
        create(
            conn,
            &NewNotification {
                workspace_id,
                recipient_member_id: member,
                priority: 1,
                kind: "support.emergency_access".to_owned(),
                text_key: "notification.support.emergency_access".to_owned(),
                params: json!({"grant_id": grant}),
                batch_id: None,
                batch_step_id: None,
                dedupe_key: Some(format!("emergency:{grant}:{member}")),
            },
        )
        .await?;
    }
    Ok(Approval::Granted(grant))
}
