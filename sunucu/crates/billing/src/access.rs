//! Read side: what a workspace may do. Run these on a connection with the workspace context set
//! (`app.workspace_id`); RLS then limits the rows.

use chrono::{DateTime, Months, Utc};
use serde::Serialize;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::PAID_FEATURE;

/// Trial length in months for a Home workspace.
pub const TRIAL_MONTHS_HOME: u32 = 3;
/// Trial length in months for a Business workspace.
pub const TRIAL_MONTHS_BUSINESS: u32 = 1;

/// Trial length in months for a workspace mode (`home` or `business`).
/// An unknown mode gets the shorter trial.
#[must_use]
pub fn trial_months(mode: &str) -> u32 {
    if mode == "home" {
        TRIAL_MONTHS_HOME
    } else {
        TRIAL_MONTHS_BUSINESS
    }
}

/// End of the trial: `created_at` plus the trial length of the current mode.
/// A mode change does not restart the trial.
#[must_use]
pub fn trial_end(created_at: DateTime<Utc>, mode: &str) -> DateTime<Utc> {
    created_at
        .checked_add_months(Months::new(trial_months(mode)))
        .unwrap_or(DateTime::<Utc>::MAX_UTC)
}

/// Level of access to the paid features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessLevel {
    /// Reads and writes.
    Full,
    /// Reads only. Export stays available.
    ReadOnly,
}

/// Why the access is what it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessSource {
    /// The trial of the workspace (3 months for Home, 1 month for Business).
    Trial,
    /// A paid subscription.
    Subscription,
    /// A promotion entitlement.
    Promo,
    /// A manual correction by support.
    Manual,
    /// No valid trial and no valid entitlement.
    None,
}

/// A subscription as the API shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SubscriptionInfo {
    /// `app_store` or `web`.
    pub source: String,
    /// Product key.
    pub product_key: String,
    /// `trial`, `active`, `grace`, `expired` or `cancelled`.
    pub status: String,
    /// End of the paid period.
    pub period_ends_at: Option<DateTime<Utc>>,
}

/// The access state of a workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Access {
    /// Level now.
    pub level: AccessLevel,
    /// Reason.
    pub source: AccessSource,
    /// End of the trial (always set: see `trial_end`).
    pub trial_ends_at: DateTime<Utc>,
    /// End of the entitlement that gives access now; `None` for the trial and for an open end.
    pub valid_until: Option<DateTime<Utc>>,
    /// Plan of the entitlement that gives access now (`home` or `business`). `None` for the
    /// trial and for an entitlement without a plan: the module set then follows the mode.
    pub plan_key: Option<String>,
    /// Seat count of that entitlement.
    pub seats: Option<i32>,
}

type PaidRow = (String, Option<DateTime<Utc>>, Option<String>, Option<i32>);

/// Computes the access of `workspace_id` at the database time.
///
/// # Errors
/// Database errors. A workspace that the context cannot see gives `sqlx::Error::RowNotFound`.
pub async fn access(conn: &mut PgConnection, workspace_id: Uuid) -> sqlx::Result<Access> {
    let (created_at, mode, now): (DateTime<Utc>, String, DateTime<Utc>) =
        sqlx::query_as("SELECT created_at, mode, now() FROM workspaces WHERE id = $1")
            .bind(workspace_id)
            .fetch_one(&mut *conn)
            .await?;
    let trial_ends_at = trial_end(created_at, &mode);
    let trial_open = trial_ends_at > now;
    let paid: Option<PaidRow> = sqlx::query_as(
        "SELECT source, valid_until, plan_key, seats FROM entitlements
         WHERE workspace_id = $1 AND feature = $2 AND deleted_at IS NULL
           AND valid_from <= now() AND (valid_until IS NULL OR valid_until > now())
         ORDER BY valid_until DESC NULLS FIRST LIMIT 1",
    )
    .bind(workspace_id)
    .bind(PAID_FEATURE)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(match paid {
        Some((source, valid_until, plan_key, seats)) => Access {
            level: AccessLevel::Full,
            source: match source.as_str() {
                "subscription" => AccessSource::Subscription,
                "promo" => AccessSource::Promo,
                "manual" => AccessSource::Manual,
                _ => AccessSource::Trial,
            },
            trial_ends_at,
            valid_until,
            plan_key,
            seats,
        },
        None if trial_open => Access {
            level: AccessLevel::Full,
            source: AccessSource::Trial,
            trial_ends_at,
            valid_until: None,
            plan_key: None,
            seats: None,
        },
        None => Access {
            level: AccessLevel::ReadOnly,
            source: AccessSource::None,
            trial_ends_at,
            valid_until: None,
            plan_key: None,
            seats: None,
        },
    })
}

/// The subscriptions of the workspace, oldest first.
///
/// # Errors
/// Database errors.
pub async fn subscriptions(
    conn: &mut PgConnection,
    workspace_id: Uuid,
) -> sqlx::Result<Vec<SubscriptionInfo>> {
    let rows: Vec<(String, String, String, Option<DateTime<Utc>>)> = sqlx::query_as(
        "SELECT source, product_key, status, period_ends_at FROM subscriptions
         WHERE workspace_id = $1 AND deleted_at IS NULL ORDER BY created_at, id",
    )
    .bind(workspace_id)
    .fetch_all(conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(source, product_key, status, period_ends_at)| SubscriptionInfo {
                source,
                product_key,
                status,
                period_ends_at,
            },
        )
        .collect())
}
