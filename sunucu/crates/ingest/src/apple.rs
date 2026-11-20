//! App Store Server Notifications v2: from the signed payload to a [`SubscriptionUpdate`].
//!
//! The outer JWS (`signedPayload`) and the inner `signedTransactionInfo` and `signedRenewalInfo`
//! are all verified with [`JwsVerifier`]. The client sets `appAccountToken` to the workspace id
//! at purchase; the update carries it as `workspace_id`.

use billing::{Source, SubscriptionUpdate, UPDATE_SCHEMA_VERSION, UpdateKind};
use chrono::{DateTime, Duration, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::jws::{JwsError, JwsVerifier};

/// Grace period used when the renewal info has no `gracePeriodExpiresDate`.
const DEFAULT_GRACE_DAYS: i64 = 16;

/// Why a notification was not accepted.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AppleError {
    /// A JWS failed verification.
    #[error(transparent)]
    Jws(#[from] JwsError),
    /// A required field is missing or has a wrong type.
    #[error("missing or bad field: {0}")]
    Field(&'static str),
    /// The notification is for another app.
    #[error("bundle id mismatch")]
    Bundle,
}

/// Result of parsing a notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parsed {
    /// A change to store.
    Update(Box<SubscriptionUpdate>),
    /// A valid notification that changes nothing (`TEST`, renewal status change, ...).
    Ignored {
        /// The `notificationType`.
        notification_type: String,
    },
}

fn millis(v: &Value, key: &'static str) -> Option<DateTime<Utc>> {
    v.get(key)
        .and_then(Value::as_i64)
        .and_then(DateTime::from_timestamp_millis)
}

fn text<'a>(v: &'a Value, key: &'static str) -> Result<&'a str, AppleError> {
    v.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or(AppleError::Field(key))
}

/// Verifies and parses a `signedPayload`.
///
/// `bundle_id` limits the accepted app (`None` accepts any app: only for tests and sandbox).
///
/// # Errors
/// [`AppleError`] for a bad signature or chain, a missing field or another bundle.
pub fn parse_notification(
    verifier: &JwsVerifier,
    signed_payload: &str,
    now_unix: i64,
    bundle_id: Option<&str>,
) -> Result<Parsed, AppleError> {
    let outer = verifier.verify(signed_payload, now_unix)?;
    let notification_type = text(&outer, "notificationType")?.to_owned();
    let subtype = outer.get("subtype").and_then(Value::as_str);
    let notification_id = text(&outer, "notificationUUID")?.to_owned();
    let occurred_at = millis(&outer, "signedDate").ok_or(AppleError::Field("signedDate"))?;

    let kind = match (notification_type.as_str(), subtype) {
        ("SUBSCRIBED" | "DID_RENEW" | "OFFER_REDEEMED" | "RENEWAL_EXTENDED", _) => {
            UpdateKind::Active
        }
        ("DID_FAIL_TO_RENEW", Some("GRACE_PERIOD")) => UpdateKind::Grace,
        ("DID_FAIL_TO_RENEW" | "EXPIRED" | "GRACE_PERIOD_EXPIRED", _) => UpdateKind::Expired,
        ("REFUND" | "REVOKE", _) => UpdateKind::Refunded,
        _ => return Ok(Parsed::Ignored { notification_type }),
    };

    let data = outer.get("data").ok_or(AppleError::Field("data"))?;
    if let Some(expected) = bundle_id
        && data.get("bundleId").and_then(Value::as_str) != Some(expected)
    {
        return Err(AppleError::Bundle);
    }
    let tx = verifier.verify(text(data, "signedTransactionInfo")?, now_unix)?;
    let renewal = match data.get("signedRenewalInfo").and_then(Value::as_str) {
        Some(jws) => Some(verifier.verify(jws, now_unix)?),
        None => None,
    };

    let expires = millis(&tx, "expiresDate");
    let period_ends_at = if kind == UpdateKind::Grace {
        renewal
            .as_ref()
            .and_then(|r| millis(r, "gracePeriodExpiresDate"))
            .or_else(|| expires.map(|e| e + Duration::days(DEFAULT_GRACE_DAYS)))
    } else {
        expires
    };
    let workspace_id = tx
        .get("appAccountToken")
        .and_then(Value::as_str)
        .and_then(|s| Uuid::parse_str(s).ok());
    Ok(Parsed::Update(Box::new(SubscriptionUpdate {
        schema_version: UPDATE_SCHEMA_VERSION,
        source: Source::AppStore,
        notification_id,
        kind,
        workspace_id,
        external_id: text(&tx, "originalTransactionId")?.to_owned(),
        product_key: text(&tx, "productId")?.to_owned(),
        period_ends_at,
        occurred_at,
    })))
}
