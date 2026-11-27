//! Interface for a web payment provider (subscriptions outside the App Store).
//!
//! Decision O6 selected Paddle (merchant of record). The Paddle adapter is not written yet: only
//! this trait and [`FakeProvider`] exist. A Paddle adapter reads the `Paddle-Signature` header
//! (timestamp and HMAC of the raw body), checks it in [`WebPaymentProvider::parse_webhook`], maps
//! the subscription events (`subscription.created`, `subscription.updated`,
//! `subscription.canceled`, `transaction.completed`, `adjustment.created` for a refund) to the
//! normalized [`SubscriptionUpdate`] and returns `None` for other events. `ingest` stores the
//! update and the worker applies it, as for the App Store.

use std::sync::Mutex;

use crate::SubscriptionUpdate;

/// Failure of a provider.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProviderError {
    /// The signature is missing or wrong. `ingest` answers 401.
    #[error("invalid signature")]
    InvalidSignature,
    /// The body cannot be parsed. `ingest` answers 400.
    #[error("malformed webhook: {0}")]
    Malformed(String),
}

/// What `ingest` needs from a web payment provider.
pub trait WebPaymentProvider: Send + Sync + 'static {
    /// Verifies the webhook and returns the update, or `None` for an event that changes nothing.
    ///
    /// # Errors
    /// [`ProviderError`] for a bad signature or body.
    fn parse_webhook(
        &self,
        signature: Option<&str>,
        body: &[u8],
    ) -> Result<Option<SubscriptionUpdate>, ProviderError>;
}

/// Fake provider for tests. The signature must equal the shared secret and the body is the JSON
/// of a [`SubscriptionUpdate`] (`{"ignore": true}` is an event without effect).
#[derive(Debug)]
pub struct FakeProvider {
    secret: String,
    seen: Mutex<usize>,
}

impl FakeProvider {
    /// A fake with the given shared secret.
    #[must_use]
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_owned(),
            seen: Mutex::new(0),
        }
    }

    /// How many valid webhooks the fake accepted.
    #[must_use]
    pub fn accepted(&self) -> usize {
        self.seen.lock().map_or(0, |n| *n)
    }
}

impl WebPaymentProvider for FakeProvider {
    fn parse_webhook(
        &self,
        signature: Option<&str>,
        body: &[u8],
    ) -> Result<Option<SubscriptionUpdate>, ProviderError> {
        if signature != Some(self.secret.as_str()) {
            return Err(ProviderError::InvalidSignature);
        }
        let value: serde_json::Value =
            serde_json::from_slice(body).map_err(|e| ProviderError::Malformed(e.to_string()))?;
        if let Ok(mut n) = self.seen.lock() {
            *n += 1;
        }
        if value.get("ignore").and_then(serde_json::Value::as_bool) == Some(true) {
            return Ok(None);
        }
        serde_json::from_value(value)
            .map(Some)
            .map_err(|e| ProviderError::Malformed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Source, UpdateKind};
    use chrono::Utc;

    #[test]
    fn fake_checks_signature_and_parses() {
        let fake = FakeProvider::new("s3cret");
        let update = SubscriptionUpdate {
            schema_version: 1,
            source: Source::Web,
            notification_id: "n1".into(),
            kind: UpdateKind::Active,
            workspace_id: None,
            external_id: "sub_1".into(),
            product_key: "pro".into(),
            period_ends_at: Some(Utc::now()),
            plan_key: None,
            seats: None,
            occurred_at: Utc::now(),
        };
        let body = serde_json::to_vec(&update).unwrap();
        assert_eq!(
            fake.parse_webhook(Some("wrong"), &body),
            Err(ProviderError::InvalidSignature)
        );
        assert_eq!(
            fake.parse_webhook(None, &body),
            Err(ProviderError::InvalidSignature)
        );
        assert_eq!(
            fake.parse_webhook(Some("s3cret"), &body).unwrap(),
            Some(update)
        );
        assert_eq!(
            fake.parse_webhook(Some("s3cret"), br#"{"ignore":true}"#)
                .unwrap(),
            None
        );
        assert!(matches!(
            fake.parse_webhook(Some("s3cret"), b"nope"),
            Err(ProviderError::Malformed(_))
        ));
        assert_eq!(fake.accepted(), 2);
    }
}
