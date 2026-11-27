//! Write side. `ingest` calls [`store`] (role `app_api`: it inserts `external_events` and the
//! outbox row). The worker runs [`NotificationHandler`] (role `app_worker`: it writes
//! `subscriptions` and `entitlements`).
//!
//! A notification takes effect once:
//! - `external_events (source, delivery_id)` is unique, so a repeated delivery is stored once.
//! - The consumer framework records `(consumer, event id)` in `processed_events`.
//! - `external_events.processed_at` marks the event as applied.
//! - A change that is older than `subscriptions.last_verified_at` is skipped (store order).

use chrono::{DateTime, Utc};
use db::outbox::{Entity, NewOutboxEvent, enqueue};
use events::consumer::{BoxFuture, Event, Handler, HandlerError};
use serde_json::{Value, json};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    CHANGED_TOPIC, NOTIFICATION_ENTITY, NOTIFICATION_TOPIC, PAID_FEATURE, PlanKey,
    SubscriptionUpdate, UPDATE_SCHEMA_VERSION,
};

/// Result of [`store`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stored {
    /// New notification: stored, and the worker will apply it.
    Queued(Uuid),
    /// The same notification arrived before. Nothing changed.
    Duplicate,
}

/// Stores a verified update in `external_events` and writes the outbox event, in the caller's
/// transaction.
///
/// # Errors
/// Database errors.
pub async fn store(conn: &mut PgConnection, update: &SubscriptionUpdate) -> db::Result<Stored> {
    let id = Uuid::now_v7();
    let payload = serde_json::to_value(update).map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    let inserted: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO external_events (id, source, delivery_id, payload)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (source, delivery_id) DO NOTHING RETURNING id",
    )
    .bind(id)
    .bind(update.source.event_source())
    .bind(&update.notification_id)
    .bind(payload)
    .fetch_optional(&mut *conn)
    .await?;
    let Some(id) = inserted else {
        return Ok(Stored::Duplicate);
    };
    enqueue(
        conn,
        &NewOutboxEvent {
            topic: NOTIFICATION_TOPIC.to_owned(),
            workspace_id: update.workspace_id,
            entity: Some(Entity {
                kind: NOTIFICATION_ENTITY.to_owned(),
                id,
                version: 1,
            }),
            data: None,
        },
    )
    .await?;
    Ok(Stored::Queued(id))
}

/// The consumer of [`NOTIFICATION_TOPIC`] events.
#[derive(Debug, Clone, Copy, Default)]
pub struct NotificationHandler;

fn classify(e: &sqlx::Error) -> HandlerError {
    // Constraint violations (class 23) never succeed on a retry.
    let permanent = e
        .as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .is_some_and(|c| c.starts_with("23"));
    if permanent {
        HandlerError::Permanent(e.to_string())
    } else {
        HandlerError::Retry(e.to_string())
    }
}

fn dberr(e: impl Into<sqlx::Error>) -> HandlerError {
    classify(&e.into())
}

impl Handler for NotificationHandler {
    fn handle<'a>(
        &'a self,
        event: &'a Event,
        tx: &'a mut PgConnection,
    ) -> BoxFuture<'a, Result<(), HandlerError>> {
        Box::pin(async move {
            if event.topic != NOTIFICATION_TOPIC {
                return Ok(());
            }
            handle_notification(event, tx).await
        })
    }
}

async fn handle_notification(event: &Event, tx: &mut PgConnection) -> Result<(), HandlerError> {
    let external_id = event
        .body
        .pointer("/entity/id")
        .and_then(Value::as_str)
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| HandlerError::Permanent("event has no external event id".to_owned()))?;
    let row: Option<(Value, bool)> = sqlx::query_as(
        "SELECT payload, processed_at IS NOT NULL FROM external_events
         WHERE id = $1 FOR UPDATE",
    )
    .bind(external_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(dberr)?;
    let Some((payload, processed)) = row else {
        return Err(HandlerError::Permanent(format!(
            "external event {external_id} not found"
        )));
    };
    if processed {
        return Ok(());
    }
    let update: SubscriptionUpdate = serde_json::from_value(payload)
        .map_err(|e| HandlerError::Permanent(format!("bad stored update: {e}")))?;
    if update.schema_version != UPDATE_SCHEMA_VERSION {
        return Err(HandlerError::Permanent(format!(
            "unknown update schema {}",
            update.schema_version
        )));
    }
    apply(tx, &update).await?;
    sqlx::query("UPDATE external_events SET processed_at = now() WHERE id = $1")
        .bind(external_id)
        .execute(&mut *tx)
        .await
        .map_err(dberr)?;
    Ok(())
}

/// Stored entitlement values, for the audit row.
type EntitlementRow = (Uuid, Option<String>, Option<i32>, Option<DateTime<Utc>>);

async fn write_entitlement(
    tx: &mut PgConnection,
    workspace_id: Uuid,
    subscription_id: Uuid,
    update: &SubscriptionUpdate,
    until: DateTime<Utc>,
) -> Result<(), HandlerError> {
    // The entitlement takes the plan and seats of the subscription row (a change without a
    // plan keeps the stored one).
    let previous: Option<EntitlementRow> = sqlx::query_as(
        "SELECT id, plan_key, seats, valid_until FROM entitlements
             WHERE workspace_id = $1 AND subscription_id = $2 AND feature = $3
               AND deleted_at IS NULL",
    )
    .bind(workspace_id)
    .bind(subscription_id)
    .bind(PAID_FEATURE)
    .fetch_optional(&mut *tx)
    .await
    .map_err(dberr)?;
    let (entitlement_id, new_plan, new_seats): (Uuid, Option<String>, Option<i32>) =
        sqlx::query_as(
            "INSERT INTO entitlements
                 (id, workspace_id, feature, source, subscription_id, valid_from, valid_until,
                  plan_key, seats)
             SELECT $1, $2, $3, 'subscription', $4, LEAST(now(), $5 - interval '1 second'), $5,
                    s.plan_key, s.seats
             FROM subscriptions s WHERE s.id = $4
             ON CONFLICT (workspace_id, subscription_id, feature)
                 WHERE subscription_id IS NOT NULL AND deleted_at IS NULL
             DO UPDATE SET valid_until = EXCLUDED.valid_until,
                           plan_key = EXCLUDED.plan_key,
                           seats = EXCLUDED.seats,
                           valid_from = LEAST(entitlements.valid_from,
                                              EXCLUDED.valid_until - interval '1 second')
             RETURNING id, plan_key, seats",
        )
        .bind(Uuid::now_v7())
        .bind(workspace_id)
        .bind(PAID_FEATURE)
        .bind(subscription_id)
        .bind(until)
        .fetch_one(&mut *tx)
        .await
        .map_err(dberr)?;
    db::append_audit(
        tx,
        workspace_id,
        &db::NewAuditEntry {
            actor_kind: db::ActorKind::System,
            actor_id: None,
            event: db::AuditEvent::EntitlementChange,
            record_kind: "entitlement".to_owned(),
            record_id: Some(entitlement_id),
            previous_value: previous.map(|(_, plan, seats, valid_until)| {
                json!({ "plan_key": plan, "seats": seats, "valid_until": valid_until })
            }),
            new_value: Some(json!({
                "plan_key": new_plan,
                "seats": new_seats,
                "valid_until": until,
                "status": update.kind.status(),
            })),
            reason: None,
        },
    )
    .await
    .map_err(|e| HandlerError::Retry(e.to_string()))?;

    Ok(())
}

/// Applies one update: upserts the subscription and its entitlement and writes the
/// `subscription.changed` event. Returns `false` when the update is older than the stored state.
///
/// # Errors
/// [`HandlerError::Permanent`] for an update that cannot apply (no workspace, unknown
/// workspace); [`HandlerError::Retry`] for a transient database error.
pub async fn apply(
    tx: &mut PgConnection,
    update: &SubscriptionUpdate,
) -> Result<bool, HandlerError> {
    let workspace_id = update.workspace_id.ok_or_else(|| {
        HandlerError::Permanent("notification has no workspace (appAccountToken)".to_owned())
    })?;
    let until = update
        .entitlement_until()
        .ok_or_else(|| HandlerError::Permanent("update has no period end".to_owned()))?;
    db::set_actor_kind(tx, db::ActorKind::System)
        .await
        .map_err(|e| HandlerError::Retry(e.to_string()))?;
    db::set_workspace_id(tx, workspace_id)
        .await
        .map_err(|e| HandlerError::Retry(e.to_string()))?;
    let exists: Option<i32> = sqlx::query_scalar("SELECT 1 FROM workspaces WHERE id = $1")
        .bind(workspace_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(dberr)?;
    if exists.is_none() {
        return Err(HandlerError::Permanent(format!(
            "workspace {workspace_id} not found"
        )));
    }

    let plan = update.plan_key.map(PlanKey::as_str);
    let seats = update.seats.map(|n| i32::try_from(n).unwrap_or(i32::MAX));
    let upserted: Option<(Uuid, i64)> = sqlx::query_as(
        "INSERT INTO subscriptions
             (id, workspace_id, source, external_id, product_key, status, period_ends_at,
              last_verified_at, plan_key, seats)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (source, external_id) DO UPDATE
         SET product_key = EXCLUDED.product_key,
             plan_key = COALESCE(EXCLUDED.plan_key, subscriptions.plan_key),
             seats = COALESCE(EXCLUDED.seats, subscriptions.seats),
             status = EXCLUDED.status,
             period_ends_at = EXCLUDED.period_ends_at,
             last_verified_at = EXCLUDED.last_verified_at
         WHERE subscriptions.workspace_id = EXCLUDED.workspace_id
           AND (subscriptions.last_verified_at IS NULL
                OR subscriptions.last_verified_at <= EXCLUDED.last_verified_at)
         RETURNING id, version",
    )
    .bind(Uuid::now_v7())
    .bind(workspace_id)
    .bind(update.source.subscription_source())
    .bind(&update.external_id)
    .bind(&update.product_key)
    .bind(update.kind.status())
    .bind(update.period_ends_at)
    .bind(update.occurred_at)
    .bind(plan)
    .bind(seats)
    .fetch_optional(&mut *tx)
    .await
    .map_err(dberr)?;
    let Some((subscription_id, version)) = upserted else {
        tracing::info!(
            external_id = %update.external_id,
            "subscription update skipped: older than the stored state"
        );
        return Ok(false);
    };

    write_entitlement(tx, workspace_id, subscription_id, update, until).await?;

    enqueue(
        tx,
        &NewOutboxEvent {
            topic: CHANGED_TOPIC.to_owned(),
            workspace_id: Some(workspace_id),
            entity: Some(Entity {
                kind: "subscription".to_owned(),
                id: subscription_id,
                version,
            }),
            data: Some(json!({ "status": update.kind.status() })),
        },
    )
    .await
    .map_err(|e| HandlerError::Retry(e.to_string()))?;
    Ok(true)
}
