//! Notifications for billing (B7): the trial ends in 7 days and in 1 day, a payment failed, the
//! subscription ended.
//!
//! - [`BillingNotificationHandler`] is the worker consumer of `subscription.changed`. Status
//!   `grace` is a failed payment; status `expired` or `cancelled` is the end.
//! - [`run_trial_reminders`] is a periodic job (hourly). It finds the workspaces whose trial is
//!   in the last 7 days or in the last day and that have no valid entitlement.
//!
//! Each notification has a dedupe key, so a repeated event or a rerun of the job creates
//! nothing new. The recipients are the active members with the permission `billing.manage`.
//! The texts are keys of the client catalog; the parameters carry the values.

use chrono::{DateTime, Duration, Utc};
use events::consumer::{BoxFuture, Event, Handler, HandlerError};
use notifier::create::{NewNotification, create};
use serde_json::{Value, json};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::{
    CHANGED_TOPIC,
    access::{self, TRIAL_MONTHS_BUSINESS, TRIAL_MONTHS_HOME, trial_end},
};

/// Durable name of the consumer of `subscription.changed`.
pub const CONSUMER_NAME: &str = "billing-notifications";
/// Permission of the members who get billing notifications.
pub const RECIPIENT_PERMISSION: &str = "billing.manage";
/// Days before the end of the trial that get a notification.
pub const TRIAL_REMINDER_DAYS: [i64; 2] = [7, 1];

/// NATS subject filter of the subscription events (any workspace, entity kind `subscription`).
#[must_use]
pub fn changed_filter() -> String {
    format!("{}.*.subscription.>", events::SUBJECT_PREFIX)
}

fn notifier_error(e: notifier::Error) -> HandlerError {
    HandlerError::from(e)
}

/// Active members that hold [`RECIPIENT_PERMISSION`].
async fn recipients(conn: &mut PgConnection, workspace_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT DISTINCT m.id
         FROM workspace_members m
         JOIN role_assignments ra ON ra.workspace_id = m.workspace_id AND ra.member_id = m.id
              AND ra.deleted_at IS NULL
         JOIN role_permissions rp ON rp.workspace_id = ra.workspace_id AND rp.role_id = ra.role_id
         WHERE m.workspace_id = $1 AND m.status = 'active' AND m.deleted_at IS NULL
           AND rp.permission = $2",
    )
    .bind(workspace_id)
    .bind(RECIPIENT_PERMISSION)
    .fetch_all(conn)
    .await
}

/// Creates one notification for each recipient. The dedupe key is `<key>:<member>`.
async fn notify_all(
    conn: &mut PgConnection,
    workspace_id: Uuid,
    kind: &str,
    params: &Value,
    key: &str,
) -> Result<usize, notifier::Error> {
    let mut created = 0;
    for member in recipients(conn, workspace_id).await? {
        let made = create(
            conn,
            &NewNotification {
                workspace_id,
                recipient_member_id: member,
                priority: 2,
                kind: kind.to_owned(),
                text_key: format!("notification.{kind}"),
                params: params.clone(),
                batch_id: None,
                batch_step_id: None,
                dedupe_key: Some(format!("{key}:{member}")),
            },
        )
        .await?;
        created += usize::from(made.is_some());
    }
    Ok(created)
}

/// Consumer of `subscription.changed`: a failed payment and the end of a subscription.
#[derive(Debug, Clone, Copy, Default)]
pub struct BillingNotificationHandler;

impl Handler for BillingNotificationHandler {
    fn handle<'a>(
        &'a self,
        event: &'a Event,
        tx: &'a mut PgConnection,
    ) -> BoxFuture<'a, Result<(), HandlerError>> {
        Box::pin(async move {
            if event.topic != CHANGED_TOPIC {
                return Ok(());
            }
            let (Some(ws), Some(subscription), Some(status)) = (
                event.workspace_id,
                event
                    .body
                    .pointer("/entity/id")
                    .and_then(Value::as_str)
                    .and_then(|s| Uuid::parse_str(s).ok()),
                event.body.pointer("/data/status").and_then(Value::as_str),
            ) else {
                return Err(HandlerError::Permanent(
                    "event has no subscription or status".to_owned(),
                ));
            };
            let kind = match status {
                "grace" => "billing.payment_failed",
                "expired" | "cancelled" => "billing.ended",
                _ => return Ok(()),
            };
            // The row version makes one key for each change: a repeated event creates nothing,
            // a later failed payment (a new version) creates a new notification.
            let version = event
                .body
                .pointer("/entity/version")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            notifier::enter_workspace(tx, ws)
                .await
                .map_err(notifier_error)?;
            notify_all(
                tx,
                ws,
                kind,
                &json!({ "subscription_id": subscription }),
                &format!("{kind}:{subscription}:{version}"),
            )
            .await
            .map_err(notifier_error)?;
            Ok(())
        })
    }
}

/// Which reminder a trial needs at `now`: 7 from 7 days before the end until 1 day before, then
/// 1 until the end. `None` outside the last 7 days.
#[must_use]
pub fn reminder_days(trial_ends_at: DateTime<Utc>, now: DateTime<Utc>) -> Option<i64> {
    if now >= trial_ends_at || now < trial_ends_at - Duration::days(TRIAL_REMINDER_DAYS[0]) {
        None
    } else if now >= trial_ends_at - Duration::days(TRIAL_REMINDER_DAYS[1]) {
        Some(TRIAL_REMINDER_DAYS[1])
    } else {
        Some(TRIAL_REMINDER_DAYS[0])
    }
}

/// Creates the trial reminders that are due at `now`. Returns the number of notifications
/// created. A failing workspace is logged and does not stop the others.
///
/// # Errors
/// Database errors of the listing.
pub async fn run_trial_reminders(pool: &PgPool, now: DateTime<Utc>) -> sqlx::Result<usize> {
    let candidates: Vec<(Uuid, DateTime<Utc>, String)> = sqlx::query_as(
        "SELECT workspace_id, created_at, mode FROM billing_trial_candidates($1, $2, $3, 1000)",
    )
    .bind(now)
    .bind(i32::try_from(TRIAL_MONTHS_HOME).unwrap_or(i32::MAX))
    .bind(i32::try_from(TRIAL_MONTHS_BUSINESS).unwrap_or(i32::MAX))
    .fetch_all(pool)
    .await?;
    let mut created = 0;
    for (ws, created_at, mode) in candidates {
        let ends = trial_end(created_at, &mode);
        let Some(days) = reminder_days(ends, now) else {
            continue;
        };
        match remind_one(pool, ws, ends, days).await {
            Ok(n) => created += n,
            Err(e) => tracing::warn!(%ws, error = %e, "trial reminder failed"),
        }
    }
    Ok(created)
}

async fn remind_one(
    pool: &PgPool,
    ws: Uuid,
    ends: DateTime<Utc>,
    days: i64,
) -> Result<usize, notifier::Error> {
    let mut tx = pool.begin().await?;
    notifier::enter_workspace(&mut tx, ws).await?;
    // A workspace with a valid entitlement has nothing to be reminded of.
    let level = access::access(&mut tx, ws).await?;
    if level.source != access::AccessSource::Trial {
        return Ok(0);
    }
    let created = notify_all(
        &mut tx,
        ws,
        "billing.trial_ending",
        &json!({ "days": days, "trial_ends_at": ends }),
        &format!("billing.trial_ending:{days}"),
    )
    .await?;
    tx.commit().await?;
    Ok(created)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn reminder_windows() {
        let end = Utc.with_ymd_and_hms(2026, 12, 20, 9, 0, 0).unwrap();
        let at = |d: i64, h: i64| end - Duration::days(d) - Duration::hours(h);
        assert_eq!(reminder_days(end, at(8, 0)), None);
        assert_eq!(reminder_days(end, at(7, 0)), Some(7));
        assert_eq!(reminder_days(end, at(3, 0)), Some(7));
        assert_eq!(reminder_days(end, at(1, 1)), Some(7));
        assert_eq!(reminder_days(end, at(1, 0)), Some(1));
        assert_eq!(reminder_days(end, at(0, 1)), Some(1));
        assert_eq!(reminder_days(end, end), None);
    }
}
