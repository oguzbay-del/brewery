//! Subscription and entitlement (T2.28).
//!
//! - [`SubscriptionUpdate`] is the verified, normalized change that `ingest` stores for a store
//!   or payment notification. It carries no secrets and no customer content.
//! - [`apply::NotificationHandler`] is the `worker` consumer. Only `app_worker` writes
//!   `subscriptions` and `entitlements`.
//! - [`access`] reads the state. A workspace has full access during the trial (one month from
//!   the creation of the workspace) and while a paid entitlement is valid. After that the paid
//!   features are read-only. Export is never blocked.
//! - [`provider::WebPaymentProvider`] is the interface for a web payment provider (decision O6).

pub mod access;
pub mod apply;
pub mod paddle;
pub mod provider;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Feature key of the entitlement row that unlocks every paid feature.
pub const PAID_FEATURE: &str = "paid_features";
/// Topic of the outbox event that `ingest` writes for a stored notification.
pub const NOTIFICATION_TOPIC: &str = "subscription.notification_received";
/// Topic of the outbox event that the worker writes after a change.
pub const CHANGED_TOPIC: &str = "subscription.changed";
/// Entity kind of the notification event.
pub const NOTIFICATION_ENTITY: &str = "external_event";
/// Durable name of the worker consumer.
pub const CONSUMER_NAME: &str = "subscription-notifications";
/// NATS subject filter of the notification events (any workspace, entity kind
/// [`NOTIFICATION_ENTITY`]).
#[must_use]
pub fn notification_filter() -> String {
    format!("{}.*.{NOTIFICATION_ENTITY}.>", events::SUBJECT_PREFIX)
}
/// Schema version of the stored [`SubscriptionUpdate`].
pub const UPDATE_SCHEMA_VERSION: u32 = 1;

/// Where a subscription is billed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// App Store (`external_events.source = 'app_store'`).
    AppStore,
    /// Web payment provider (`external_events.source = 'web_payment'`).
    Web,
}

impl Source {
    /// Value of `subscriptions.source`.
    #[must_use]
    pub const fn subscription_source(self) -> &'static str {
        match self {
            Self::AppStore => "app_store",
            Self::Web => "web",
        }
    }

    /// Value of `external_events.source`.
    #[must_use]
    pub const fn event_source(self) -> &'static str {
        match self {
            Self::AppStore => "app_store",
            Self::Web => "web_payment",
        }
    }
}

/// What a notification changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateKind {
    /// New purchase or renewal: paid until `period_ends_at`.
    Active,
    /// Billing failed, the store retries: access continues until `period_ends_at`.
    Grace,
    /// The subscription ended: access ends at `period_ends_at` (or at the notification time).
    Expired,
    /// Refund or revoke: access ends at the notification time.
    Refunded,
}

impl UpdateKind {
    /// Value of `subscriptions.status`.
    #[must_use]
    pub const fn status(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Grace => "grace",
            Self::Expired => "expired",
            Self::Refunded => "cancelled",
        }
    }
}

/// A verified change of one subscription.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionUpdate {
    /// Schema version ([`UPDATE_SCHEMA_VERSION`]).
    pub schema_version: u32,
    /// Billing source.
    pub source: Source,
    /// Id of the notification (the delivery id of the external event).
    pub notification_id: String,
    /// Kind of change.
    pub kind: UpdateKind,
    /// Workspace of the subscription. The client sets it at purchase (App Store
    /// `appAccountToken`).
    pub workspace_id: Option<Uuid>,
    /// Id of the subscription at the source (App Store `originalTransactionId`).
    pub external_id: String,
    /// Product key (App Store `productId`).
    pub product_key: String,
    /// End of the paid period, or of the grace period for [`UpdateKind::Grace`].
    pub period_ends_at: Option<DateTime<Utc>>,
    /// Time the source signed the notification. The worker ignores an older change.
    pub occurred_at: DateTime<Utc>,
}

impl SubscriptionUpdate {
    /// End of the entitlement after this change. `None` means the change is invalid.
    #[must_use]
    pub fn entitlement_until(&self) -> Option<DateTime<Utc>> {
        match self.kind {
            UpdateKind::Active | UpdateKind::Grace => self.period_ends_at,
            UpdateKind::Expired => Some(
                self.period_ends_at
                    .map_or(self.occurred_at, |p| p.min(self.occurred_at)),
            ),
            UpdateKind::Refunded => Some(self.occurred_at),
        }
    }
}
