//! Paddle Billing webhook adapter (decision O6).
//!
//! The adapter verifies the raw request body before parsing it. It deliberately keeps only
//! subscription state changes; transactions, adjustments and customer events are acknowledged
//! without creating a subscription update.

use std::{collections::HashMap, fmt, sync::Arc};

use chrono::{DateTime, Duration, Utc};
use ring::hmac;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    PlanKey, Source, SubscriptionUpdate, UPDATE_SCHEMA_VERSION, UpdateKind,
    provider::{ProviderError, WebPaymentProvider},
};

const DEFAULT_GRACE_DAYS: i64 = 16;
const DEFAULT_SIGNATURE_TOLERANCE_SECS: i64 = 300;
const SUBSCRIPTION_EVENTS: &[&str] = &[
    "subscription.created",
    "subscription.updated",
    "subscription.activated",
    "subscription.resumed",
    "subscription.past_due",
    "subscription.paused",
    "subscription.canceled",
];

/// Paddle Billing webhook verifier and subscription mapper.
#[derive(Clone)]
pub struct PaddleProvider {
    secret: Arc<[u8]>,
    prices: Arc<HashMap<String, String>>,
    plans: Arc<HashMap<String, PlanKey>>,
    grace_days: i64,
    tolerance_secs: i64,
    clock: Arc<dyn Fn() -> i64 + Send + Sync>,
}

impl fmt::Debug for PaddleProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PaddleProvider")
            .field("prices", &self.prices)
            .field("grace_days", &self.grace_days)
            .field("tolerance_secs", &self.tolerance_secs)
            .finish_non_exhaustive()
    }
}

impl PaddleProvider {
    /// Builds a provider with a system clock.
    #[must_use]
    pub fn new(
        secret: &str,
        prices: HashMap<String, String>,
        grace_days: i64,
        tolerance_secs: i64,
    ) -> Self {
        Self::with_clock(secret, prices, grace_days, tolerance_secs, || {
            Utc::now().timestamp()
        })
    }

    /// Builds a provider with an injectable Unix clock for deterministic tests.
    #[must_use]
    pub fn with_clock<F>(
        secret: &str,
        prices: HashMap<String, String>,
        grace_days: i64,
        tolerance_secs: i64,
        clock: F,
    ) -> Self
    where
        F: Fn() -> i64 + Send + Sync + 'static,
    {
        Self {
            secret: Arc::from(secret.as_bytes()),
            prices: Arc::new(prices),
            plans: Arc::new(HashMap::new()),
            grace_days,
            tolerance_secs,
            clock: Arc::new(clock),
        }
    }

    /// Sets the plan of each price id (`PADDLE_PRICE_HOME`, `PADDLE_PRICE_BUSINESS`). A price
    /// in this map needs no entry in the product map: its product key is the plan key.
    #[must_use]
    pub fn with_plans(mut self, plans: HashMap<String, PlanKey>) -> Self {
        self.plans = Arc::new(plans);
        self
    }

    /// Reads Paddle configuration. Invalid configuration is a startup error; no secret means
    /// that the provider is disabled and the webhook route remains unavailable.
    ///
    /// # Errors
    /// Returns a configuration error for malformed values or secret-file failures.
    pub fn from_env() -> common::Result<Option<Self>> {
        let secret = common::config::secret_from_lookup(
            |name| std::env::var(name).ok(),
            "PADDLE_WEBHOOK_SECRET",
            "PADDLE_WEBHOOK_SECRET_FILE",
        )?;
        let price_map = match std::env::var("PADDLE_PRICE_MAP") {
            Ok(value) => value,
            Err(std::env::VarError::NotPresent) => String::new(),
            Err(error) => return Err(env_error("PADDLE_PRICE_MAP", &error.to_string())),
        };
        let prices = parse_price_map(&price_map)?;
        let mut plans = HashMap::new();
        for (name, plan) in [
            ("PADDLE_PRICE_HOME", PlanKey::Home),
            ("PADDLE_PRICE_BUSINESS", PlanKey::Business),
        ] {
            if let Some(price) = std::env::var(name).ok().filter(|v| !v.trim().is_empty()) {
                plans.insert(price.trim().to_owned(), plan);
            }
        }
        if secret.is_some() && prices.is_empty() && plans.is_empty() {
            return Err(env_error(
                "PADDLE_PRICE_MAP",
                "set at least one price_id=product_key pair or PADDLE_PRICE_HOME",
            ));
        }
        let grace_days = parse_env_i64("PADDLE_GRACE_DAYS", DEFAULT_GRACE_DAYS)?;
        let tolerance_secs = parse_env_i64(
            "PADDLE_SIGNATURE_TOLERANCE_SECS",
            DEFAULT_SIGNATURE_TOLERANCE_SECS,
        )?;
        if grace_days < 0 {
            return Err(env_error("PADDLE_GRACE_DAYS", "must not be negative"));
        }
        if tolerance_secs < 0 {
            return Err(env_error(
                "PADDLE_SIGNATURE_TOLERANCE_SECS",
                "must not be negative",
            ));
        }
        Ok(secret
            .map(|value| Self::new(&value, prices, grace_days, tolerance_secs).with_plans(plans)))
    }

    fn verify_signature(&self, header: Option<&str>, body: &[u8]) -> bool {
        let Some(header) = header else {
            return false;
        };
        let mut timestamp = None;
        let mut signatures = Vec::new();
        for field in header.split(';').map(str::trim) {
            if let Some(value) = field.strip_prefix("ts=") {
                let Ok(parsed) = value.parse::<i64>() else {
                    return false;
                };
                if timestamp.replace(parsed).is_some() {
                    return false;
                }
            } else if let Some(value) = field.strip_prefix("h1=")
                && let Some(bytes) = decode_hex(value)
            {
                signatures.push(bytes);
            }
        }
        let Some(timestamp) = timestamp else {
            return false;
        };
        if signatures.is_empty() {
            return false;
        }
        let delta = i128::from(timestamp) - i128::from((self.clock)());
        if delta.abs() > i128::from(self.tolerance_secs) {
            return false;
        }
        let mut signed = timestamp.to_string().into_bytes();
        signed.push(b':');
        signed.extend_from_slice(body);
        let key = hmac::Key::new(hmac::HMAC_SHA256, &self.secret);
        signatures
            .iter()
            .any(|signature| hmac::verify(&key, &signed, signature).is_ok())
    }

    fn map_event(&self, envelope: &Value) -> Result<Option<SubscriptionUpdate>, ProviderError> {
        let event_type = required_str(envelope, "event_type")?;
        if !SUBSCRIPTION_EVENTS.contains(&event_type) {
            return Ok(None);
        }
        let data = envelope
            .get("data")
            .and_then(Value::as_object)
            .ok_or_else(|| malformed("missing subscription data"))?;
        let delivery_id = envelope
            .get("event_id")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .or_else(|| {
                envelope
                    .get("notification_id")
                    .and_then(Value::as_str)
                    .filter(|value| !value.is_empty())
            })
            .ok_or_else(|| malformed("missing event_id and notification_id"))?;
        let occurred_at = parse_datetime(required_str(envelope, "occurred_at")?, "occurred_at")?;
        let external_id = data
            .get("id")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| malformed("missing subscription id"))?;
        let status = data
            .get("status")
            .and_then(Value::as_str)
            .ok_or_else(|| malformed("missing subscription status"))?;

        let price_id = data
            .get("items")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(|item| item.pointer("/price/id"))
            .and_then(Value::as_str)
            .ok_or_else(|| malformed("missing subscription price id"))?;
        let plan_key = self.plans.get(price_id).copied();
        let product_key = self
            .prices
            .get(price_id)
            .cloned()
            .or_else(|| plan_key.map(|plan| plan.as_str().to_owned()))
            .ok_or_else(|| malformed("unknown Paddle price id"))?;
        let seats = item_seats(data);

        let workspace_id = envelope
            .pointer("/data/custom_data/workspace_id")
            .and_then(Value::as_str)
            .and_then(|value| Uuid::parse_str(value).ok());

        let (kind, period_ends_at) = match status {
            "active" | "trialing" => {
                let period = envelope
                    .pointer("/data/current_billing_period/ends_at")
                    .and_then(Value::as_str)
                    .ok_or_else(|| malformed("active subscription has no billing period end"))?;
                (
                    UpdateKind::Active,
                    Some(parse_datetime(period, "current_billing_period.ends_at")?),
                )
            }
            "past_due" => {
                let period = envelope
                    .pointer("/data/current_billing_period/ends_at")
                    .and_then(Value::as_str)
                    .ok_or_else(|| malformed("past-due subscription has no billing period end"))?;
                let end = parse_datetime(period, "current_billing_period.ends_at")?
                    + Duration::days(self.grace_days);
                (UpdateKind::Grace, Some(end))
            }
            "paused" => {
                let end =
                    optional_datetime(data.get("paused_at"), "paused_at")?.unwrap_or(occurred_at);
                (UpdateKind::Expired, Some(end))
            }
            "canceled" => {
                let end = optional_datetime(data.get("canceled_at"), "canceled_at")?
                    .unwrap_or(occurred_at);
                (UpdateKind::Expired, Some(end))
            }
            _ => return Err(malformed("unsupported subscription status")),
        };

        Ok(Some(SubscriptionUpdate {
            schema_version: UPDATE_SCHEMA_VERSION,
            source: Source::Web,
            // The project decision uses event_id as the stable deduplication key.
            notification_id: delivery_id.to_owned(),
            kind,
            workspace_id,
            external_id: external_id.to_owned(),
            product_key,
            period_ends_at,
            plan_key,
            seats,
            customer_id: customer_id(data),
            occurred_at,
        }))
    }
}

impl WebPaymentProvider for PaddleProvider {
    fn parse_webhook(
        &self,
        signature: Option<&str>,
        body: &[u8],
    ) -> Result<Option<SubscriptionUpdate>, ProviderError> {
        if !self.verify_signature(signature, body) {
            return Err(ProviderError::InvalidSignature);
        }
        let envelope: Value =
            serde_json::from_slice(body).map_err(|_| malformed("body is not valid JSON"))?;
        self.map_event(&envelope)
    }
}

/// Seat count: the quantity of the first item (at least 1).
fn item_seats(data: &serde_json::Map<String, Value>) -> Option<u32> {
    data.get("items")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("quantity"))
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
        .filter(|n| *n >= 1)
}

fn customer_id(data: &serde_json::Map<String, Value>) -> Option<String> {
    data.get("customer_id")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
}

fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str, ProviderError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| malformed("missing required envelope field"))
}

fn parse_datetime(value: &str, field: &str) -> Result<DateTime<Utc>, ProviderError> {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.with_timezone(&Utc))
        .map_err(|_| {
            malformed(match field {
                "occurred_at" => "invalid occurred_at timestamp",
                "current_billing_period.ends_at" => "invalid billing period timestamp",
                _ => "invalid subscription timestamp",
            })
        })
}

fn optional_datetime(
    value: Option<&Value>,
    field: &str,
) -> Result<Option<DateTime<Utc>>, ProviderError> {
    value
        .and_then(Value::as_str)
        .map(|raw| parse_datetime(raw, field))
        .transpose()
}

fn malformed(reason: &str) -> ProviderError {
    ProviderError::Malformed(reason.to_owned())
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if value.is_empty() || !value.len().is_multiple_of(2) || !value.is_ascii() {
        return None;
    }
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).ok())
        .collect()
}

fn env_error(name: &'static str, reason: &str) -> common::Error {
    common::Error::InvalidEnv {
        name,
        reason: reason.to_owned(),
    }
}

fn parse_env_i64(name: &'static str, default: i64) -> common::Result<i64> {
    match std::env::var(name) {
        Ok(value) => value
            .trim()
            .parse()
            .map_err(|error: std::num::ParseIntError| env_error(name, &error.to_string())),
        Err(std::env::VarError::NotPresent) => Ok(default),
        Err(error) => Err(env_error(name, &error.to_string())),
    }
}

fn parse_price_map(raw: &str) -> common::Result<HashMap<String, String>> {
    let mut prices = HashMap::new();
    if raw.trim().is_empty() {
        return Ok(prices);
    }
    for entry in raw.split(',') {
        let Some((price, product)) = entry.split_once('=') else {
            return Err(env_error(
                "PADDLE_PRICE_MAP",
                "expected price_id=product_key pairs",
            ));
        };
        let price = price.trim();
        let product = product.trim();
        if price.is_empty() || product.is_empty() || prices.contains_key(price) {
            return Err(env_error(
                "PADDLE_PRICE_MAP",
                "entries must be non-empty and price ids must be unique",
            ));
        }
        prices.insert(price.to_owned(), product.to_owned());
    }
    Ok(prices)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use ring::hmac::{self, Key};
    use std::fmt::Write as _;

    const NOW: i64 = 1_800_000_000;
    const WORKSPACE: &str = "123e4567-e89b-12d3-a456-426614174000";

    fn provider() -> PaddleProvider {
        PaddleProvider::with_clock(
            "test-secret",
            HashMap::from([("pri_home".to_owned(), "home_pro".to_owned())]),
            16,
            300,
            || NOW,
        )
    }

    fn payload(event_type: &str, status: &str) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "notification_id": "ntf_retry",
            "event_id": "evt_stable",
            "event_type": event_type,
            "occurred_at": "2026-12-15T12:00:00Z",
            "data": {
                "id": "sub_test",
                "customer_id": "ctm_1",
                "status": status,
                "custom_data": {"workspace_id": WORKSPACE},
                "items": [{"price": {"id": "pri_home"}}],
                "current_billing_period": {"ends_at": "2026-12-31T00:00:00Z"},
                "paused_at": "2026-12-20T00:00:00Z",
                "canceled_at": "2026-12-21T00:00:00Z"
            }
        }))
        .unwrap_or_default()
    }

    fn sign(body: &[u8], timestamp: i64, secret: &[u8]) -> String {
        let mut signed = timestamp.to_string().into_bytes();
        signed.push(b':');
        signed.extend_from_slice(body);
        let tag = hmac::sign(&Key::new(hmac::HMAC_SHA256, secret), &signed);
        let hex = tag.as_ref().iter().fold(String::new(), |mut output, byte| {
            let _ = write!(&mut output, "{byte:02x}");
            output
        });
        format!("ts={timestamp};h1={hex}")
    }

    #[test]
    fn verifies_signature_and_raw_body() {
        let provider = provider();
        let body = payload("subscription.activated", "active");
        let header = sign(&body, NOW, b"test-secret");
        assert!(provider.parse_webhook(Some(&header), &body).is_ok());
        assert_eq!(
            provider.parse_webhook(Some("ts=1800000000;h1=00"), &body),
            Err(ProviderError::InvalidSignature)
        );
        assert_eq!(
            provider.parse_webhook(Some(&header), br#"{"changed":true}"#),
            Err(ProviderError::InvalidSignature)
        );
        assert_eq!(
            provider.parse_webhook(None, &body),
            Err(ProviderError::InvalidSignature)
        );
    }

    #[test]
    fn accepts_any_valid_h1_and_rejects_stale_or_future_timestamps() {
        let provider = provider();
        let body = payload("subscription.activated", "active");
        let good = sign(&body, NOW, b"test-secret");
        let header = format!(
            "ts={NOW};h1=00;{}",
            good.split_once(';').map_or("", |(_, h)| h)
        );
        assert!(provider.parse_webhook(Some(&header), &body).is_ok());
        let stale = sign(&body, NOW - 301, b"test-secret");
        assert_eq!(
            provider.parse_webhook(Some(&stale), &body),
            Err(ProviderError::InvalidSignature)
        );
        let future = sign(&body, NOW + 301, b"test-secret");
        assert_eq!(
            provider.parse_webhook(Some(&future), &body),
            Err(ProviderError::InvalidSignature)
        );
    }

    #[test]
    fn maps_active_trialing_grace_paused_and_canceled() {
        let provider = provider();
        let active = payload("subscription.activated", "active");
        let update = provider
            .parse_webhook(Some(&sign(&active, NOW, b"test-secret")), &active)
            .ok()
            .flatten()
            .unwrap();
        assert_eq!(update.kind, UpdateKind::Active);
        assert_eq!(update.notification_id, "evt_stable");
        assert_eq!(update.workspace_id, Uuid::parse_str(WORKSPACE).ok());

        let period_end = "2026-12-31T00:00:00Z";
        assert_eq!(update.period_ends_at, Some(period_end.parse().unwrap()));

        for (status, kind, end) in [
            ("trialing", UpdateKind::Active, period_end),
            ("past_due", UpdateKind::Grace, "2027-01-16T00:00:00Z"),
            ("paused", UpdateKind::Expired, "2026-12-20T00:00:00Z"),
            ("canceled", UpdateKind::Expired, "2026-12-21T00:00:00Z"),
        ] {
            let body = payload("subscription.updated", status);
            let update = provider
                .parse_webhook(Some(&sign(&body, NOW, b"test-secret")), &body)
                .ok()
                .flatten()
                .unwrap();
            assert_eq!(update.kind, kind);
            assert_eq!(update.period_ends_at, Some(end.parse().unwrap()));
        }
    }

    #[test]
    fn cancellation_without_canceled_at_uses_occurrence_time() {
        let mut event: Value =
            serde_json::from_slice(&payload("subscription.updated", "canceled")).unwrap();
        event["data"].as_object_mut().unwrap().remove("canceled_at");
        let body = serde_json::to_vec(&event).unwrap();
        let update = provider()
            .parse_webhook(Some(&sign(&body, NOW, b"test-secret")), &body)
            .unwrap()
            .unwrap();
        assert_eq!(update.kind, UpdateKind::Expired);
        assert_eq!(update.period_ends_at, Some(update.occurred_at));
    }

    #[test]
    fn scheduled_cancellation_remains_active_and_workspace_can_be_missing() {
        let provider = provider();
        let mut value: Value = serde_json::from_slice(&payload("subscription.updated", "active"))
            .unwrap_or(Value::Null);
        value["data"]["scheduled_change"] = serde_json::json!({"action":"cancel"});
        value["data"]["custom_data"] = Value::Null;
        let body = serde_json::to_vec(&value).unwrap_or_default();
        let update = provider
            .parse_webhook(Some(&sign(&body, NOW, b"test-secret")), &body)
            .ok()
            .flatten()
            .unwrap();
        assert_eq!(update.kind, UpdateKind::Active);
        assert_eq!(update.workspace_id, None);
    }

    #[test]
    fn ignores_other_events_and_rejects_unknown_prices_and_missing_period() {
        let provider = provider();
        let ignored = payload("transaction.completed", "active");
        assert_eq!(
            provider.parse_webhook(Some(&sign(&ignored, NOW, b"test-secret")), &ignored),
            Ok(None)
        );

        let mut value: Value = serde_json::from_slice(&payload("subscription.updated", "active"))
            .unwrap_or(Value::Null);
        value["data"]["items"][0]["price"]["id"] = Value::String("pri_unknown".to_owned());
        let body = serde_json::to_vec(&value).unwrap_or_default();
        assert!(matches!(
            provider.parse_webhook(Some(&sign(&body, NOW, b"test-secret")), &body),
            Err(ProviderError::Malformed(_))
        ));

        value["data"]["items"][0]["price"]["id"] = Value::String("pri_home".to_owned());
        value["data"]["current_billing_period"] = Value::Null;
        let body = serde_json::to_vec(&value).unwrap_or_default();
        assert!(matches!(
            provider.parse_webhook(Some(&sign(&body, NOW, b"test-secret")), &body),
            Err(ProviderError::Malformed(_))
        ));
    }

    #[test]
    fn falls_back_to_notification_id_only_when_event_id_is_missing() {
        let provider = provider();
        let mut value: Value = serde_json::from_slice(&payload("subscription.activated", "active"))
            .unwrap_or(Value::Null);
        value.as_object_mut().unwrap().remove("event_id");
        let body = serde_json::to_vec(&value).unwrap_or_default();
        let update = provider
            .parse_webhook(Some(&sign(&body, NOW, b"test-secret")), &body)
            .ok()
            .flatten()
            .unwrap();
        assert_eq!(update.notification_id, "ntf_retry");
    }

    #[test]
    fn official_activation_fixture_maps_event_id() {
        let body = include_bytes!("../tests/fixtures/paddle/subscription_activated.json");
        let provider = PaddleProvider::with_clock(
            "test-secret",
            HashMap::from([("pri_example_home".to_owned(), "home_pro".to_owned())]),
            16,
            300,
            || NOW,
        );
        let header = sign(body, NOW, b"test-secret");
        let update = provider
            .parse_webhook(Some(&header), body)
            .ok()
            .flatten()
            .unwrap();
        assert_eq!(update.notification_id, "evt_01j00000000000000000000001");
    }

    #[test]
    fn malformed_price_maps_are_rejected() {
        assert!(parse_price_map("pri_a=home,broken").is_err());
        assert!(parse_price_map("pri_a=home,pri_a=business").is_err());
        assert!(parse_price_map("pri_a=home,pri_b=business").is_ok_and(|map| map.len() == 2));
    }

    #[test]
    fn maps_plan_from_price_and_seats_from_quantity() {
        let provider = PaddleProvider::with_clock("test-secret", HashMap::new(), 16, 300, || NOW)
            .with_plans(HashMap::from([("pri_home".to_owned(), PlanKey::Business)]));
        let mut value: serde_json::Value =
            serde_json::from_slice(&payload("subscription.updated", "active")).unwrap();
        value["data"]["items"][0]["quantity"] = serde_json::json!(7);
        let body = serde_json::to_vec(&value).unwrap();
        let header = sign(&body, NOW, b"test-secret");
        let update = provider
            .parse_webhook(Some(&header), &body)
            .ok()
            .flatten()
            .unwrap();
        assert_eq!(update.plan_key, Some(PlanKey::Business));
        assert_eq!(update.seats, Some(7));
        assert_eq!(update.customer_id.as_deref(), Some("ctm_1"));
        assert_eq!(update.product_key, "business");
    }
}
