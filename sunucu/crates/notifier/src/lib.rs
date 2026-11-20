//! Notifications (T2.24): creation, alert rules, delivery (push and e-mail), quiet hours, daily
//! summary and reminders.
//!
//! - [`create::create`] inserts a notification and writes the outbox event `notification.created`.
//! - [`alert::AlertHandler`] is the worker consumer of `batch.measurement_added`. It evaluates
//!   the alert rules of the workspace and creates notifications.
//! - [`deliver::DeliveryHandler`] is the worker consumer of `notification.created`.
//!   [`tick::run_due`] delivers what quiet hours deferred, wakes snoozed notifications, fires
//!   reminders and sends the daily summary.
//!
//! Priority 1 is critical: it ignores quiet hours and goes out as a time-sensitive push.
//! Priority 2 is normal. Priority 3 is low: with the daily summary on, it only appears in the
//! inbox and in the summary.

pub mod alert;
pub mod apns;
pub mod create;
pub mod deliver;
pub mod smtp;
pub mod tick;

use events::consumer::HandlerError;
use sqlx::PgConnection;
use uuid::Uuid;

/// Topic of the outbox event of a new notification.
pub const CREATED_TOPIC: &str = "notification.created";
/// Entity kind of the notification event.
pub const ENTITY: &str = "notification";
/// Durable name of the alert consumer.
pub const ALERT_CONSUMER: &str = "notifier-alerts";
/// Durable name of the delivery consumer.
pub const DELIVERY_CONSUMER: &str = "notifier-delivery";

/// Subject filter of the measurement events (any workspace, entity kind `batch`).
#[must_use]
pub fn alert_filter() -> String {
    format!("{}.*.batch.>", events::SUBJECT_PREFIX)
}

/// Subject filter of the notification events.
#[must_use]
pub fn delivery_filter() -> String {
    format!("{}.*.{ENTITY}.>", events::SUBJECT_PREFIX)
}

/// Errors of this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Database failure.
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    /// Failure of the database helpers.
    #[error(transparent)]
    Core(#[from] db::Error),
    /// A delivery channel failed and a retry can help.
    #[error("delivery: {0}")]
    Delivery(String),
}

/// Result alias.
pub type Result<T> = std::result::Result<T, Error>;

impl From<Error> for HandlerError {
    fn from(e: Error) -> Self {
        // Constraint violations (class 23) never succeed on a retry.
        let permanent = matches!(&e, Error::Db(s) if s
            .as_database_error()
            .and_then(sqlx::error::DatabaseError::code)
            .is_some_and(|c| c.starts_with("23")));
        if permanent {
            Self::Permanent(e.to_string())
        } else {
            Self::Retry(e.to_string())
        }
    }
}

/// Puts the transaction in the context of the system acting inside one workspace.
///
/// # Errors
/// Database errors.
pub async fn enter_workspace(conn: &mut PgConnection, workspace_id: Uuid) -> Result<()> {
    db::set_actor_kind(conn, db::ActorKind::System).await?;
    db::set_workspace_id(conn, workspace_id).await?;
    Ok(())
}
