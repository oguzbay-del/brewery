//! The periodic work of the worker: deferred deliveries, snoozed notifications, reminders and the
//! daily summary. `notify_due_work` lists the due rows across workspaces; each row runs in its
//! own transaction in the context of its workspace.

use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{PgConnection, PgPool};
use tokio::sync::watch;
use uuid::Uuid;

use crate::{
    Result,
    create::{self, NewNotification},
    deliver::{Deliverer, deliver_one},
    enter_workspace,
};

/// How often the worker asks for due work.
pub const INTERVAL: Duration = Duration::from_secs(30);

/// Runs every due item once. Returns the number of items handled. A failing item is logged and
/// stays due.
///
/// # Errors
/// Database errors of the listing.
pub async fn run_due(pool: &PgPool, d: &Deliverer, now: DateTime<Utc>) -> Result<usize> {
    let due: Vec<(String, Uuid, Uuid)> =
        sqlx::query_as("SELECT kind, workspace_id, id FROM notify_due_work($1, 100)")
            .bind(now)
            .fetch_all(pool)
            .await?;
    let mut handled = 0;
    for (kind, ws, id) in due {
        match run_item(pool, d, &kind, ws, id, now).await {
            Ok(()) => handled += 1,
            Err(e) => tracing::warn!(%kind, %id, error = %e, "notification work failed"),
        }
    }
    Ok(handled)
}

async fn run_item(
    pool: &PgPool,
    d: &Deliverer,
    kind: &str,
    ws: Uuid,
    id: Uuid,
    now: DateTime<Utc>,
) -> Result<()> {
    let mut tx = pool.begin().await?;
    enter_workspace(&mut tx, ws).await?;
    match kind {
        "deliver" => {
            deliver_one(&mut tx, d, ws, id, now).await?;
        }
        "wake" => {
            sqlx::query(
                "UPDATE notifications
                 SET status = 'open', snoozed_until = NULL, delivered_at = NULL, deliver_after = $2
                 WHERE id = $1 AND status = 'snoozed' AND snoozed_until <= $2",
            )
            .bind(id)
            .bind(now)
            .execute(&mut *tx)
            .await?;
            deliver_one(&mut tx, d, ws, id, now).await?;
        }
        "reminder" => fire_reminder(&mut tx, ws, id).await?,
        "summary" => send_summary(&mut tx, d, id, now).await?,
        _ => {}
    }
    tx.commit().await?;
    Ok(())
}

async fn fire_reminder(conn: &mut PgConnection, ws: Uuid, id: Uuid) -> Result<()> {
    let row: Option<(Option<Uuid>, Option<Uuid>, Option<Uuid>, String)> = sqlx::query_as(
        "UPDATE reminders SET status = 'fired' WHERE id = $1 AND status = 'scheduled'
         RETURNING recipient_member_id, batch_id, batch_step_id, rule",
    )
    .bind(id)
    .fetch_optional(&mut *conn)
    .await?;
    let Some((member, batch_id, batch_step_id, rule)) = row else {
        return Ok(());
    };
    let members = match member {
        Some(m) => vec![m],
        None => create::recipients(conn, ws).await?,
    };
    for m in members {
        create::create(
            conn,
            &NewNotification {
                workspace_id: ws,
                recipient_member_id: m,
                priority: 2,
                kind: "reminder".to_owned(),
                text_key: "notification.reminder".to_owned(),
                params: json!({"rule": rule}),
                batch_id,
                batch_step_id,
                dedupe_key: Some(format!("reminder:{id}:{m}")),
            },
        )
        .await?;
    }
    Ok(())
}

async fn send_summary(
    conn: &mut PgConnection,
    d: &Deliverer,
    preference: Uuid,
    now: DateTime<Utc>,
) -> Result<()> {
    let (user_id,): (Uuid,) = sqlx::query_as(
        "SELECT user_id FROM notification_preferences WHERE id = $1 AND daily_summary",
    )
    .bind(preference)
    .fetch_one(&mut *conn)
    .await?;
    db::set_user_id(conn, user_id).await?;
    let (open, critical): (i64, i64) = sqlx::query_as(
        "SELECT count(*), count(*) FILTER (WHERE n.priority = 1)
         FROM notifications n
         JOIN workspace_members m ON m.workspace_id = n.workspace_id AND m.id = n.recipient_member_id
         WHERE m.user_id = $1 AND n.status IN ('open', 'snoozed') AND n.deleted_at IS NULL
           AND n.created_at > $2 - interval '24 hours'",
    )
    .bind(user_id)
    .bind(now)
    .fetch_one(&mut *conn)
    .await?;
    if open > 0 && d.can_mail() {
        let to: Option<Option<String>> = sqlx::query_scalar(
            "SELECT contact_email FROM users
             WHERE id = $1 AND contact_email_verified_at IS NOT NULL",
        )
        .bind(user_id)
        .fetch_optional(&mut *conn)
        .await?;
        if let Some(Some(to)) = to {
            let text = format!(
                "You have {open} open notifications from the last 24 hours ({critical} critical).\n\
                 Open the app to see them.\n"
            );
            d.mail(&to, "Your daily summary", &text)
                .await
                .map_err(crate::Error::Delivery)?;
        }
    }
    // Local date of the recipient, so the summary goes out once a day.
    sqlx::query(
        "UPDATE notification_preferences SET last_summary_on = ($2 AT TIME ZONE timezone)::date
         WHERE id = $1",
    )
    .bind(preference)
    .bind(now)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Runs [`run_due`] every [`INTERVAL`] until `stop` turns `true`.
pub async fn run(pool: PgPool, d: std::sync::Arc<Deliverer>, mut stop: watch::Receiver<bool>) {
    loop {
        if let Err(e) = run_due(&pool, &d, Utc::now()).await {
            tracing::warn!(error = %e, "notification tick failed");
        }
        tokio::select! {
            r = stop.changed() => if r.is_err() || *stop.borrow() { return; },
            () = tokio::time::sleep(INTERVAL) => {}
        }
    }
}
