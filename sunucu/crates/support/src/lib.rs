//! Support requests (T2.29, plan "Destek talepleri: GitHub Issues").
//!
//! The record of a request is in the database (`support_requests`, `support_messages`). The
//! support team works in GitHub Issues in a private repository, and the two sides stay in sync:
//!
//! - [`sync`] sends requests, customer messages, edits, deletes and status changes to GitHub
//!   (outbox event `support.sync_requested`, retry through `scheduled_jobs` when GitHub is not
//!   available).
//! - [`webhook`] verifies and stores GitHub webhooks (`ingest`) and applies them (`worker`):
//!   `/yanit` comments, internal notes, edits, deletes, closing and reopening of Issues.
//! - [`reply`] writes a staff message from the support panel.
//!
//! GitHub sees only the pseudonym of the workspace and the request number. An internal note never
//! goes to the customer.

pub mod github;
pub mod panel;
pub mod reply;
pub mod sync;
pub mod webhook;

use events::consumer::HandlerError;
use sqlx::PgConnection;
use uuid::Uuid;

/// Outbox topic: a request needs a GitHub sync.
pub const SYNC_TOPIC: &str = "support.sync_requested";
/// Outbox topic: a verified GitHub webhook is stored in `external_events`.
pub const GITHUB_EVENT_TOPIC: &str = "support.github_event";
/// Entity kind of the sync event.
pub const REQUEST_ENTITY: &str = "support_request";
/// Entity kind of the webhook event.
pub const EVENT_ENTITY: &str = "external_event";
/// Durable name of the sync consumer.
pub const SYNC_CONSUMER: &str = "support-sync";
/// Durable name of the webhook consumer.
pub const GITHUB_CONSUMER: &str = "support-github";
/// `scheduled_jobs.kind` of the sync retry.
pub const JOB_KIND: &str = "support.github_sync";
/// Start of the hidden marker in each comment the application writes. The webhook ignores a
/// comment with the marker.
pub const MARKER: &str = "<!-- demleme:msg:";
/// The label of a request that waits for the customer.
pub const WAITING_LABEL: &str = "bekliyor:musteri";

/// Subject filter of the sync events.
#[must_use]
pub fn sync_filter() -> String {
    format!("{}.*.{REQUEST_ENTITY}.>", events::SUBJECT_PREFIX)
}

/// Subject filter of the webhook events (they have no workspace).
#[must_use]
pub fn github_filter() -> String {
    format!("{}._.{EVENT_ENTITY}.>", events::SUBJECT_PREFIX)
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
    /// Failure of the notification helpers.
    #[error(transparent)]
    Notify(#[from] notifier::Error),
}

/// Result alias.
pub type Result<T> = std::result::Result<T, Error>;

impl From<Error> for HandlerError {
    fn from(e: Error) -> Self {
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
