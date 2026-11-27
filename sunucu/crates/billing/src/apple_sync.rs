//! Sync of one `StoreKit` 2 transaction (B3): the client sends the signed transaction (JWS) after a
//! purchase or a restore, and the API turns it into a [`SubscriptionUpdate`] that goes through the
//! same path as an App Store notification (`apply::store`, then the worker).
//!
//! The JWS is verified like a notification: ES256, `x5c` chain to the Apple root, Apple marker
//! extensions. The bundle id, the environment and the product list come from the same settings as
//! the webhook (`APPLE_BUNDLE_ID`, `APPLE_ENVIRONMENT`, `APPLE_PRODUCT_IDS`). The product plan
//! comes from `APPLE_PRODUCT_PLANS`.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    PlanKey, Source, SubscriptionUpdate, UPDATE_SCHEMA_VERSION, UpdateKind,
    jws::{JwsError, JwsVerifier},
};

/// Settings of the transaction sync.
#[derive(Debug, Clone)]
pub struct AppleSyncSettings {
    /// JWS verifier ([`JwsVerifier::apple`] in production).
    pub verifier: JwsVerifier,
    /// Accepted bundle id.
    pub bundle_id: String,
    /// Accepted App Store environment (`Production` or `Sandbox`).
    pub environment: String,
    /// Accepted product ids. Empty accepts all.
    pub product_ids: Vec<String>,
    /// Plan of each product. A product that is not here gives no plan.
    pub plans: HashMap<String, PlanKey>,
}

impl AppleSyncSettings {
    /// Reads the settings. `None` without `APPLE_BUNDLE_ID` (the sync is off).
    ///
    /// # Errors
    /// A message for a bad `APPLE_ENVIRONMENT` or `APPLE_PRODUCT_PLANS`.
    pub fn from_lookup<F>(lookup: F) -> Result<Option<Self>, String>
    where
        F: Fn(&str) -> Option<String>,
    {
        let Some(bundle_id) = lookup("APPLE_BUNDLE_ID")
            .map(|v| v.trim().to_owned())
            .filter(|v| !v.is_empty())
        else {
            return Ok(None);
        };
        let environment = match lookup("APPLE_ENVIRONMENT").as_deref().map(str::trim) {
            None | Some("" | "Production") => "Production".to_owned(),
            Some("Sandbox") => "Sandbox".to_owned(),
            Some(other) => {
                return Err(format!(
                    "APPLE_ENVIRONMENT must be Production or Sandbox, got {other:?}"
                ));
            }
        };
        let product_ids = lookup("APPLE_PRODUCT_IDS")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect();
        let plans = crate::parse_product_plans(lookup("APPLE_PRODUCT_PLANS").as_deref())
            .map_err(|e| format!("APPLE_PRODUCT_PLANS: {e}"))?;
        Ok(Some(Self {
            verifier: JwsVerifier::apple(),
            bundle_id,
            environment,
            product_ids,
            plans,
        }))
    }
}

/// Why a transaction was not accepted.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SyncError {
    /// The JWS failed verification.
    #[error(transparent)]
    Jws(#[from] JwsError),
    /// A required field is missing or has a wrong type.
    #[error("missing or bad field: {0}")]
    Field(&'static str),
    /// The transaction is for another app.
    #[error("bundle id mismatch")]
    Bundle,
    /// The transaction is from another App Store environment.
    #[error("environment mismatch")]
    Environment,
    /// The product is not in the allowed list.
    #[error("product not allowed")]
    Product,
    /// `appAccountToken` is not the workspace id.
    #[error("appAccountToken does not match the workspace")]
    Workspace,
}

fn millis(v: &Value, key: &str) -> Option<DateTime<Utc>> {
    v.get(key)
        .and_then(Value::as_i64)
        .and_then(DateTime::from_timestamp_millis)
}

fn text<'a>(v: &'a Value, key: &'static str) -> Result<&'a str, SyncError> {
    v.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or(SyncError::Field(key))
}

/// Verifies `signed_transaction` and builds the update for `workspace_id`. `now_unix` is the
/// current time in Unix seconds.
///
/// # Errors
/// [`SyncError`] for a bad signature or chain, a missing field, another app or environment, a
/// product that is not allowed, or an `appAccountToken` that is not `workspace_id`.
pub fn parse_transaction(
    settings: &AppleSyncSettings,
    signed_transaction: &str,
    now_unix: i64,
    workspace_id: Uuid,
) -> Result<SubscriptionUpdate, SyncError> {
    let tx = settings.verifier.verify(signed_transaction, now_unix)?;
    if tx.get("bundleId").and_then(Value::as_str) != Some(settings.bundle_id.as_str()) {
        return Err(SyncError::Bundle);
    }
    if tx.get("environment").and_then(Value::as_str) != Some(settings.environment.as_str()) {
        return Err(SyncError::Environment);
    }
    let token = tx
        .get("appAccountToken")
        .and_then(Value::as_str)
        .and_then(|s| Uuid::parse_str(s).ok());
    if token != Some(workspace_id) {
        return Err(SyncError::Workspace);
    }
    let product = text(&tx, "productId")?;
    if !settings.product_ids.is_empty() && !settings.product_ids.iter().any(|p| p == product) {
        return Err(SyncError::Product);
    }
    let external_id = text(&tx, "originalTransactionId")?;
    let transaction_id = text(&tx, "transactionId")?;
    let now = DateTime::from_timestamp(now_unix, 0).ok_or(SyncError::Field("clock"))?;
    let occurred_at = millis(&tx, "signedDate").unwrap_or(now);
    let expires = millis(&tx, "expiresDate");
    let kind = if millis(&tx, "revocationDate").is_some() {
        UpdateKind::Refunded
    } else if expires.is_some_and(|e| e > now) {
        UpdateKind::Active
    } else {
        UpdateKind::Expired
    };
    let plan_key = settings.plans.get(product).copied();
    if plan_key.is_none() {
        tracing::warn!(
            product,
            "App Store product has no plan in APPLE_PRODUCT_PLANS"
        );
    }
    Ok(SubscriptionUpdate {
        schema_version: UPDATE_SCHEMA_VERSION,
        source: Source::AppStore,
        // One id for each transaction: a repeated sync of it is stored once.
        notification_id: format!("sync:{transaction_id}"),
        kind,
        workspace_id: Some(workspace_id),
        external_id: external_id.to_owned(),
        product_key: product.to_owned(),
        period_ends_at: expires,
        plan_key,
        seats: None,
        customer_id: None,
        occurred_at,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn settings_need_a_bundle_id_and_a_known_environment() {
        let none = |_: &str| None;
        assert!(AppleSyncSettings::from_lookup(none).unwrap().is_none());
        let with = |env: &'static str| {
            move |name: &str| match name {
                "APPLE_BUNDLE_ID" => Some("app.demleme".to_owned()),
                "APPLE_ENVIRONMENT" => Some(env.to_owned()),
                "APPLE_PRODUCT_PLANS" => Some("a.home=home".to_owned()),
                _ => None,
            }
        };
        let s = AppleSyncSettings::from_lookup(with("Sandbox"))
            .unwrap()
            .unwrap();
        assert_eq!(s.environment, "Sandbox");
        assert_eq!(s.plans.get("a.home"), Some(&PlanKey::Home));
        assert!(AppleSyncSettings::from_lookup(with("sandbox")).is_err());
    }

    #[test]
    fn a_bad_jws_is_rejected() {
        let s = AppleSyncSettings {
            verifier: JwsVerifier::apple(),
            bundle_id: "app.demleme".to_owned(),
            environment: "Production".to_owned(),
            product_ids: Vec::new(),
            plans: HashMap::new(),
        };
        let r = parse_transaction(&s, "not.a.jws", 1_800_000_000, Uuid::nil());
        assert!(matches!(r, Err(SyncError::Jws(_))));
    }
}
