//! Revokes Apple refresh tokens of deleted accounts (T2.31d).
//!
//! The purge job calls [`run`] after it purges the due accounts. A failed revoke is counted in
//! `user_identities.apple_revoke_attempts` and retried on the next tick. The third failed
//! attempt clears the stored ciphertext. A successful revoke clears it at once. The code never
//! logs the token, the ciphertext or the client secret.

use apple_auth::{
    Aes256GcmTokenCipher, AppleClient, AppleTokenSettings, RevokeResult, TokenCipher,
    client_ids_from_lookup,
};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::Result;

const BATCH_LIMIT: i32 = 100;
const MAX_ATTEMPTS: i16 = 3;

/// The Apple client and the cipher that the worker needs for a revoke.
pub struct AppleRevoker {
    client: AppleClient,
    cipher: Box<dyn TokenCipher>,
}

impl std::fmt::Debug for AppleRevoker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppleRevoker")
            .field("client", &self.client)
            .finish_non_exhaustive()
    }
}

impl AppleRevoker {
    /// Creates a revoker from a client and a cipher.
    #[must_use]
    pub fn new(client: AppleClient, cipher: Box<dyn TokenCipher>) -> Self {
        Self { client, cipher }
    }

    /// Reads the `APPLE_*` and `PROVIDER_TOKEN_KEY` settings. Returns `None` when they are not set
    /// or are invalid.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let lookup = |name: &str| std::env::var(name).ok();
        let client_ids = client_ids_from_lookup(&lookup);
        let settings = match AppleTokenSettings::from_lookup(&lookup, &client_ids) {
            Ok(Some(settings)) => settings,
            Ok(None) => {
                tracing::warn!("Apple settings are not set; Apple token revoke is off");
                return None;
            }
            Err(error) => {
                tracing::error!(%error, "Apple settings are invalid; Apple token revoke is off");
                return None;
            }
        };
        let cipher = match Aes256GcmTokenCipher::from_lookup(&lookup, true) {
            Ok(Some(cipher)) => cipher,
            Ok(None) => return None,
            Err(error) => {
                tracing::error!(%error, "PROVIDER_TOKEN_KEY is invalid; Apple token revoke is off");
                return None;
            }
        };
        match AppleClient::new(settings) {
            Ok(client) => Some(Self::new(client, Box::new(cipher))),
            Err(error) => {
                tracing::error!(%error, "Apple client setup failed; Apple token revoke is off");
                None
            }
        }
    }
}

/// Counts of one revoke run.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RevokeReport {
    /// Tokens that Apple revoked.
    pub revoked: u32,
    /// Failed attempts that will be retried.
    pub retry_later: u32,
    /// Tokens that failed the third time. Their ciphertext is cleared and they stay active.
    pub given_up: u32,
    /// Stored tokens that were not tried because the Apple settings are missing.
    pub skipped: u32,
}

impl RevokeReport {
    /// The summary as JSON for the maintenance detail.
    #[must_use]
    pub fn to_json(self) -> Value {
        json!({
            "revoked": self.revoked,
            "retry_later": self.retry_later,
            "given_up": self.given_up,
            "skipped": self.skipped,
        })
    }
}

/// Revokes the stored Apple tokens of deleted accounts.
pub async fn run(pool: &PgPool, apple: Option<&AppleRevoker>) -> Result<RevokeReport> {
    let mut conn = pool.acquire().await.map_err(db::Error::from)?;
    let candidates = db::apple_revoke_pending(&mut conn, BATCH_LIMIT).await?;
    let mut report = RevokeReport::default();
    let Some(apple) = apple else {
        report.skipped = u32::try_from(candidates.len()).unwrap_or(u32::MAX);
        if report.skipped > 0 {
            tracing::warn!(
                skipped = report.skipped,
                "Apple settings are missing; stored Apple tokens are not revoked"
            );
        }
        return Ok(report);
    };
    for candidate in candidates {
        let result = apple
            .client
            .revoke_stored(apple.cipher.as_ref(), &candidate.ciphertext)
            .await;
        match result {
            RevokeResult::Revoked => {
                db::apple_revoke_clear(&mut conn, candidate.identity_id).await?;
                report.revoked += 1;
            }
            RevokeResult::Failed => {
                let attempts = db::apple_revoke_failed(&mut conn, candidate.identity_id)
                    .await?
                    .unwrap_or(candidate.attempts);
                if attempts >= MAX_ATTEMPTS {
                    tracing::error!(
                        identity_id = %candidate.identity_id,
                        "Apple token revoke failed three times; the stored token is cleared"
                    );
                    report.given_up += 1;
                } else {
                    tracing::warn!(
                        identity_id = %candidate.identity_id,
                        attempts,
                        "Apple token revoke failed; will retry"
                    );
                    report.retry_later += 1;
                }
            }
        }
    }
    Ok(report)
}
