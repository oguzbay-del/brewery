//! Read side: what a workspace may do. Run these on a connection with the workspace context set
//! (`app.workspace_id`); RLS then limits the rows.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::PAID_FEATURE;

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
    /// The first month of the workspace.
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
    /// End of the trial (always set: one month after the workspace was created).
    pub trial_ends_at: DateTime<Utc>,
    /// End of the entitlement that gives access now; `None` for the trial and for an open end.
    pub valid_until: Option<DateTime<Utc>>,
}

/// Computes the access of `workspace_id` at the database time.
///
/// # Errors
/// Database errors. A workspace that the context cannot see gives `sqlx::Error::RowNotFound`.
pub async fn access(conn: &mut PgConnection, workspace_id: Uuid) -> sqlx::Result<Access> {
    let (trial_ends_at, trial_open): (DateTime<Utc>, bool) = sqlx::query_as(
        "SELECT created_at + interval '1 month', created_at + interval '1 month' > now()
         FROM workspaces WHERE id = $1",
    )
    .bind(workspace_id)
    .fetch_one(&mut *conn)
    .await?;
    let paid: Option<(String, Option<DateTime<Utc>>)> = sqlx::query_as(
        "SELECT source, valid_until FROM entitlements
         WHERE workspace_id = $1 AND feature = $2 AND deleted_at IS NULL
           AND valid_from <= now() AND (valid_until IS NULL OR valid_until > now())
         ORDER BY valid_until DESC NULLS FIRST LIMIT 1",
    )
    .bind(workspace_id)
    .bind(PAID_FEATURE)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(match paid {
        Some((source, valid_until)) => Access {
            level: AccessLevel::Full,
            source: match source.as_str() {
                "subscription" => AccessSource::Subscription,
                "promo" => AccessSource::Promo,
                "manual" => AccessSource::Manual,
                _ => AccessSource::Trial,
            },
            trial_ends_at,
            valid_until,
        },
        None if trial_open => Access {
            level: AccessLevel::Full,
            source: AccessSource::Trial,
            trial_ends_at,
            valid_until: None,
        },
        None => Access {
            level: AccessLevel::ReadOnly,
            source: AccessSource::None,
            trial_ends_at,
            valid_until: None,
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
