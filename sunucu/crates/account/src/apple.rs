//! Sign in with Apple: the code exchange at the first sign-in and the token revoke at account
//! deletion (ADR-0009 rule 1.5; App Store rule: revoke the token when the account is deleted).
//!
//! The server proves its identity to Apple with a client secret: a JWT signed with the Apple
//! private key (ES256). Reference: <https://developer.apple.com/documentation/sign_in_with_apple>
//! (to be verified against the live service).

use std::{
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::{
    Engine,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use futures_util::future::BoxFuture;
use ring::{
    rand::SystemRandom,
    signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair},
};
use serde_json::{Value, json};

/// Default token endpoint.
pub const TOKEN_URL: &str = "https://appleid.apple.com/auth/token";
/// Default revoke endpoint.
pub const REVOKE_URL: &str = "https://appleid.apple.com/auth/revoke";
const AUDIENCE: &str = "https://appleid.apple.com";

/// Why a call to Apple failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AppleError {
    /// Apple does not answer or answers with a server error. A retry can help.
    #[error("apple unavailable: {0}")]
    Unavailable(String),
    /// Apple refused the request. A retry does not help.
    #[error("apple rejected the request: {0}")]
    Rejected(String),
    /// The settings are not complete.
    #[error("apple settings are not complete: {0}")]
    Config(String),
}

/// The calls the server makes to Apple.
pub trait AppleClient: Send + Sync {
    /// Exchanges the authorization code of the first sign-in. Returns the refresh token.
    fn exchange_code<'a>(&'a self, code: &'a str) -> BoxFuture<'a, Result<String, AppleError>>;
    /// Revokes a refresh token. A token that Apple does not know counts as revoked.
    fn revoke<'a>(&'a self, refresh_token: &'a str) -> BoxFuture<'a, Result<(), AppleError>>;
}

/// The real client.
pub struct HttpApple {
    http: reqwest::Client,
    team_id: String,
    key_id: String,
    client_id: String,
    key: EcdsaKeyPair,
    token_url: String,
    revoke_url: String,
}

impl std::fmt::Debug for HttpApple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpApple")
            .field("client_id", &self.client_id)
            .finish_non_exhaustive()
    }
}

fn b64(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

fn form_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn form_body(pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .map(|(k, v)| format!("{k}={}", form_encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

impl HttpApple {
    /// Builds the client from a PEM private key (`-----BEGIN PRIVATE KEY-----`, PKCS#8).
    ///
    /// # Errors
    /// [`AppleError::Config`] when the key cannot be read.
    pub fn new(
        team_id: &str,
        key_id: &str,
        client_id: &str,
        private_key_pem: &str,
        token_url: &str,
        revoke_url: &str,
    ) -> Result<Self, AppleError> {
        let body: String = private_key_pem
            .lines()
            .filter(|l| !l.starts_with("-----"))
            .collect();
        let der = STANDARD
            .decode(body.trim())
            .map_err(|e| AppleError::Config(format!("private key is not base64: {e}")))?;
        let key = EcdsaKeyPair::from_pkcs8(
            &ECDSA_P256_SHA256_FIXED_SIGNING,
            &der,
            &SystemRandom::new(),
        )
        .map_err(|e| AppleError::Config(format!("private key is not valid: {e}")))?;
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| AppleError::Config(e.to_string()))?;
        Ok(Self {
            http,
            team_id: team_id.to_owned(),
            key_id: key_id.to_owned(),
            client_id: client_id.to_owned(),
            key,
            token_url: token_url.to_owned(),
            revoke_url: revoke_url.to_owned(),
        })
    }

    /// From the environment (see `CLAUDE.md`). `None` when the settings are absent; an invalid
    /// key is logged.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let get = |n: &str| std::env::var(n).ok().filter(|v| !v.trim().is_empty());
        let team = get("APPLE_TEAM_ID")?;
        let key_id = get("APPLE_KEY_ID")?;
        let client = get("APPLE_CLIENT_IDS")?
            .split(',')
            .next()
            .map(|s| s.trim().to_owned())?;
        let pem = get("APPLE_PRIVATE_KEY").or_else(|| {
            get("APPLE_PRIVATE_KEY_FILE").and_then(|p| std::fs::read_to_string(p).ok())
        })?;
        let token_url = get("APPLE_TOKEN_URL").unwrap_or_else(|| TOKEN_URL.to_owned());
        let revoke_url = get("APPLE_REVOKE_URL").unwrap_or_else(|| REVOKE_URL.to_owned());
        match Self::new(&team, &key_id, &client, &pem, &token_url, &revoke_url) {
            Ok(c) => Some(c),
            Err(e) => {
                tracing::error!(error = %e, "Sign in with Apple server settings are invalid");
                None
            }
        }
    }

    fn client_secret(&self) -> Result<String, AppleError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let header = json!({"alg": "ES256", "kid": self.key_id, "typ": "JWT"});
        let claims = json!({"iss": self.team_id, "iat": now, "exp": now + 300,
                            "aud": AUDIENCE, "sub": self.client_id});
        let signing_input = format!("{}.{}", b64(header.to_string().as_bytes()), b64(claims.to_string().as_bytes()));
        let sig = self
            .key
            .sign(&SystemRandom::new(), signing_input.as_bytes())
            .map_err(|_| AppleError::Config("cannot sign the client secret".into()))?;
        Ok(format!("{signing_input}.{}", b64(sig.as_ref())))
    }

    async fn post(&self, url: &str, pairs: &[(&str, &str)]) -> Result<(u16, Value), AppleError> {
        let reply = self
            .http
            .post(url)
            .header("content-type", "application/x-www-form-urlencoded")
            .body(form_body(pairs))
            .send()
            .await
            .map_err(|e| AppleError::Unavailable(e.to_string()))?;
        let status = reply.status().as_u16();
        let text = reply.text().await.unwrap_or_default();
        let body: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
        if status >= 500 || status == 429 {
            return Err(AppleError::Unavailable(format!("status {status}")));
        }
        Ok((status, body))
    }
}

impl AppleClient for HttpApple {
    fn exchange_code<'a>(&'a self, code: &'a str) -> BoxFuture<'a, Result<String, AppleError>> {
        Box::pin(async move {
            let secret = self.client_secret()?;
            let (status, body) = self
                .post(
                    &self.token_url,
                    &[
                        ("client_id", &self.client_id),
                        ("client_secret", &secret),
                        ("code", code),
                        ("grant_type", "authorization_code"),
                    ],
                )
                .await?;
            if status != 200 {
                return Err(AppleError::Rejected(format!(
                    "status {status} {}",
                    body["error"].as_str().unwrap_or_default()
                )));
            }
            body["refresh_token"]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| AppleError::Rejected("no refresh token in the answer".into()))
        })
    }

    fn revoke<'a>(&'a self, refresh_token: &'a str) -> BoxFuture<'a, Result<(), AppleError>> {
        Box::pin(async move {
            let secret = self.client_secret()?;
            let (status, body) = self
                .post(
                    &self.revoke_url,
                    &[
                        ("client_id", &self.client_id),
                        ("client_secret", &secret),
                        ("token", refresh_token),
                        ("token_type_hint", "refresh_token"),
                    ],
                )
                .await?;
            // `invalid_grant`: the token is already revoked or unknown.
            if status == 200 || body["error"].as_str() == Some("invalid_grant") {
                return Ok(());
            }
            Err(AppleError::Rejected(format!(
                "status {status} {}",
                body["error"].as_str().unwrap_or_default()
            )))
        })
    }
}

/// A client for tests and local runs. It does not call Apple.
#[derive(Debug, Default)]
pub struct FakeApple {
    /// Tokens that were revoked, in order.
    pub revoked: Mutex<Vec<String>>,
    /// When `true`, `revoke` fails (to test the retry).
    pub fail_revoke: Mutex<bool>,
}

impl FakeApple {
    /// Revoked tokens so far.
    #[must_use]
    pub fn revoked_tokens(&self) -> Vec<String> {
        self.revoked.lock().map(|g| g.clone()).unwrap_or_default()
    }
}

impl AppleClient for FakeApple {
    fn exchange_code<'a>(&'a self, code: &'a str) -> BoxFuture<'a, Result<String, AppleError>> {
        Box::pin(async move { Ok(format!("fake-refresh-{code}")) })
    }

    fn revoke<'a>(&'a self, refresh_token: &'a str) -> BoxFuture<'a, Result<(), AppleError>> {
        Box::pin(async move {
            if self.fail_revoke.lock().map(|g| *g).unwrap_or(false) {
                return Err(AppleError::Unavailable("fake failure".into()));
            }
            if let Ok(mut g) = self.revoked.lock() {
                g.push(refresh_token.to_owned());
            }
            Ok(())
        })
    }
}

/// The client for this process: the real one when configured, the fake one with
/// `APPLE_LOCAL_MODE=true`, otherwise `None`.
#[must_use]
pub fn client_from_env() -> Option<std::sync::Arc<dyn AppleClient>> {
    if std::env::var("APPLE_LOCAL_MODE").is_ok_and(|v| v == "true") {
        tracing::warn!("APPLE_LOCAL_MODE is on: Apple tokens are not exchanged or revoked");
        return Some(std::sync::Arc::new(FakeApple::default()));
    }
    HttpApple::from_env().map(|c| std::sync::Arc::new(c) as std::sync::Arc<dyn AppleClient>)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn form_values_are_percent_encoded() {
        assert_eq!(form_body(&[("a", "x y&z=1"), ("b", "ok-_.~")]), "a=x%20y%26z%3D1&b=ok-_.~");
    }

    #[tokio::test]
    async fn the_fake_records_revokes() {
        let f = FakeApple::default();
        assert_eq!(f.exchange_code("c1").await.unwrap(), "fake-refresh-c1");
        f.revoke("t").await.unwrap();
        assert_eq!(f.revoked_tokens(), vec!["t".to_owned()]);
        *f.fail_revoke.lock().unwrap() = true;
        assert!(f.revoke("u").await.is_err());
    }

    #[test]
    fn a_bad_key_is_a_config_error() {
        let e = HttpApple::new("t", "k", "c", "not a key", TOKEN_URL, REVOKE_URL).unwrap_err();
        assert!(matches!(e, AppleError::Config(_)));
    }
}
