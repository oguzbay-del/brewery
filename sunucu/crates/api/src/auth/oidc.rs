//! ID token verification for Apple and Google (ADR-0009 rule 1.3). The one place that checks a
//! provider token: signature against the provider's JWKS, `alg`, `iss`, `aud`, `exp`, `iat`.
//! The nonce is compared by the caller, against the single-use `auth_requests` row.

use std::{
    collections::HashMap,
    sync::Once,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use ring::signature::{RSA_PKCS1_2048_8192_SHA256, RsaPublicKeyComponents};
use serde::Deserialize;
use tokio::sync::RwLock;

use super::config::ProviderSettings;

/// Clock skew allowed on `exp`, `iat` and `nbf` (ADR-0009 rule 1.3).
pub const CLOCK_SKEW_SECS: i64 = 60;
/// A cached JWKS is never used past this age (ADR-0009 rule 1.3).
pub const MAX_JWKS_AGE: Duration = Duration::from_hours(24);
const MAX_TOKEN_LEN: usize = 16 * 1024;
const MAX_JWKS_BYTES: usize = 256 * 1024;
const FETCH_TIMEOUT: Duration = Duration::from_secs(5);

/// Why a token was refused. The `code` goes to the client in `errors[]`; nothing else about
/// the cause is revealed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OidcError {
    /// Not three base64url parts, or JSON that does not parse.
    Malformed,
    /// `alg` is not `RS256` (`none` and symmetric algorithms included).
    UnsupportedAlg,
    /// No `kid`, or no such key even after refreshing the JWKS.
    UnknownKey,
    /// The signature does not match.
    BadSignature,
    /// `iss` is not an accepted issuer.
    BadIssuer,
    /// `aud` holds none of the configured client ids.
    BadAudience,
    /// `exp` has passed.
    Expired,
    /// `iat` or `nbf` lies in the future.
    NotYetValid,
    /// A required claim is missing or empty.
    MissingClaim,
    /// The provider's JWKS cannot be fetched (the caller answers `503`, not `401`).
    JwksUnavailable,
}

impl OidcError {
    /// Stable code for `errors[].code`.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Malformed => "malformed_token",
            Self::UnsupportedAlg => "unsupported_alg",
            Self::UnknownKey => "unknown_key",
            Self::BadSignature => "bad_signature",
            Self::BadIssuer => "bad_issuer",
            Self::BadAudience => "bad_audience",
            Self::Expired => "token_expired",
            Self::NotYetValid => "token_not_yet_valid",
            Self::MissingClaim => "missing_claim",
            Self::JwksUnavailable => "jwks_unavailable",
        }
    }
}

/// The verified claims the sign-in flow uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claims {
    /// `sub`: the provider's permanent user id.
    pub sub: String,
    /// `email`, if present.
    pub email: Option<String>,
    /// `email_verified` (Apple sends a string; both forms are read).
    pub email_verified: bool,
    /// `nonce`, if present.
    pub nonce: Option<String>,
    /// `name`, if present.
    pub name: Option<String>,
}

/// Installs the `ring` crypto provider for rustls once per process (reqwest is built without
/// a provider of its own).
pub fn install_crypto_provider() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        // Err means another provider is already installed; either works.
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

#[derive(Debug, Deserialize)]
struct Header {
    alg: String,
    kid: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Audience {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Deserialize)]
struct RawClaims {
    iss: Option<String>,
    sub: Option<String>,
    aud: Option<Audience>,
    exp: Option<i64>,
    iat: Option<i64>,
    nbf: Option<i64>,
    email: Option<String>,
    email_verified: Option<serde_json::Value>,
    nonce: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JwkSet {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    kty: String,
    kid: Option<String>,
    alg: Option<String>,
    #[serde(rename = "use")]
    use_: Option<String>,
    n: Option<String>,
    e: Option<String>,
}

#[derive(Debug, Clone)]
struct RsaKey {
    n: Vec<u8>,
    e: Vec<u8>,
}

#[derive(Debug, Default)]
struct Cache {
    keys: HashMap<String, RsaKey>,
    fetched_at: Option<Instant>,
    last_attempt: Option<Instant>,
}

/// Verifies the ID tokens of one provider. Cheap to share; the JWKS cache is inside.
#[derive(Debug)]
pub struct OidcVerifier {
    settings: ProviderSettings,
    http: reqwest::Client,
    cache: RwLock<Cache>,
    min_refresh: Duration,
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

fn decode_json<T: for<'de> Deserialize<'de>>(part: &str) -> Result<T, OidcError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(part)
        .map_err(|_| OidcError::Malformed)?;
    serde_json::from_slice(&bytes).map_err(|_| OidcError::Malformed)
}

fn truthy(value: Option<&serde_json::Value>) -> bool {
    match value {
        Some(serde_json::Value::Bool(b)) => *b,
        Some(serde_json::Value::String(s)) => s == "true",
        _ => false,
    }
}

impl OidcVerifier {
    /// A verifier for `settings`. `min_refresh` is the shortest gap between two refreshes that
    /// an unknown `kid` triggers (keeps forged `kid`s from turning into JWKS fetch floods).
    ///
    /// # Panics
    /// When the HTTP client cannot be built (no TLS roots can be loaded).
    #[must_use]
    pub fn new(settings: ProviderSettings, min_refresh: Duration) -> Self {
        install_crypto_provider();
        let http = reqwest::Client::builder()
            .timeout(FETCH_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap_or_else(|e| panic!("cannot build the HTTP client: {e}"));
        Self {
            settings,
            http,
            cache: RwLock::new(Cache::default()),
            min_refresh,
        }
    }

    /// The configured client ids.
    #[must_use]
    pub fn audiences(&self) -> &[String] {
        &self.settings.audiences
    }

    async fn fetch(&self) -> Result<HashMap<String, RsaKey>, OidcError> {
        let response = self
            .http
            .get(&self.settings.jwks_url)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|e| {
                tracing::warn!(error = %e, url = %self.settings.jwks_url, "JWKS fetch failed");
                OidcError::JwksUnavailable
            })?;
        let bytes = response.bytes().await.map_err(|e| {
            tracing::warn!(error = %e, "JWKS body failed");
            OidcError::JwksUnavailable
        })?;
        if bytes.len() > MAX_JWKS_BYTES {
            return Err(OidcError::JwksUnavailable);
        }
        let set: JwkSet = serde_json::from_slice(&bytes).map_err(|_| OidcError::JwksUnavailable)?;
        let mut keys = HashMap::new();
        for jwk in set.keys {
            let usable = jwk.kty == "RSA"
                && jwk.use_.as_deref().is_none_or(|u| u == "sig")
                && jwk.alg.as_deref().is_none_or(|a| a == "RS256");
            if !usable {
                continue;
            }
            if let (Some(kid), Some(n), Some(e)) = (jwk.kid, jwk.n, jwk.e)
                && let (Ok(n), Ok(e)) = (URL_SAFE_NO_PAD.decode(n), URL_SAFE_NO_PAD.decode(e))
            {
                keys.insert(kid, RsaKey { n, e });
            }
        }
        Ok(keys)
    }

    async fn key_for(&self, kid: &str) -> Result<RsaKey, OidcError> {
        {
            let cache = self.cache.read().await;
            let fresh = cache.fetched_at.is_some_and(|t| t.elapsed() < MAX_JWKS_AGE);
            if fresh && let Some(key) = cache.keys.get(kid) {
                return Ok(key.clone());
            }
        }
        let mut cache = self.cache.write().await;
        // Another task may have refreshed while this one waited for the lock.
        let fresh = cache.fetched_at.is_some_and(|t| t.elapsed() < MAX_JWKS_AGE);
        if fresh && let Some(key) = cache.keys.get(kid) {
            return Ok(key.clone());
        }
        let expired = !fresh && cache.fetched_at.is_some();
        let too_soon = cache
            .last_attempt
            .is_some_and(|t| t.elapsed() < self.min_refresh);
        if too_soon && !expired {
            return Err(OidcError::UnknownKey);
        }
        cache.last_attempt = Some(Instant::now());
        match self.fetch().await {
            Ok(keys) => {
                cache.keys = keys;
                cache.fetched_at = Some(Instant::now());
            }
            Err(e) => {
                if expired {
                    cache.keys.clear();
                    cache.fetched_at = None;
                }
                return Err(e);
            }
        }
        cache.keys.get(kid).cloned().ok_or(OidcError::UnknownKey)
    }

    /// Verifies `token` and returns its claims.
    ///
    /// # Errors
    /// An [`OidcError`] naming the first check that failed.
    pub async fn verify(&self, token: &str) -> Result<Claims, OidcError> {
        if token.len() > MAX_TOKEN_LEN {
            return Err(OidcError::Malformed);
        }
        let mut parts = token.split('.');
        let (Some(h), Some(p), Some(s), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(OidcError::Malformed);
        };
        let header: Header = decode_json(h)?;
        if header.alg != "RS256" {
            return Err(OidcError::UnsupportedAlg);
        }
        let kid = header
            .kid
            .filter(|k| !k.is_empty())
            .ok_or(OidcError::UnknownKey)?;
        let key = self.key_for(&kid).await?;
        let signature = URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|_| OidcError::Malformed)?;
        let signed = &token[..h.len() + 1 + p.len()];
        RsaPublicKeyComponents {
            n: &key.n,
            e: &key.e,
        }
        .verify(&RSA_PKCS1_2048_8192_SHA256, signed.as_bytes(), &signature)
        .map_err(|_| OidcError::BadSignature)?;

        let claims: RawClaims = decode_json(p)?;
        self.check_claims(claims, now_secs())
    }

    fn check_claims(&self, c: RawClaims, now: i64) -> Result<Claims, OidcError> {
        let iss = c.iss.ok_or(OidcError::MissingClaim)?;
        if !self.settings.issuers.contains(&iss) {
            return Err(OidcError::BadIssuer);
        }
        let audiences = match c.aud.ok_or(OidcError::MissingClaim)? {
            Audience::One(a) => vec![a],
            Audience::Many(a) => a,
        };
        if !audiences
            .iter()
            .any(|a| self.settings.audiences.iter().any(|ok| ok == a))
        {
            return Err(OidcError::BadAudience);
        }
        let exp = c.exp.ok_or(OidcError::MissingClaim)?;
        if exp.saturating_add(CLOCK_SKEW_SECS) <= now {
            return Err(OidcError::Expired);
        }
        let iat = c.iat.ok_or(OidcError::MissingClaim)?;
        if iat.saturating_sub(CLOCK_SKEW_SECS) > now
            || c.nbf
                .is_some_and(|n| n.saturating_sub(CLOCK_SKEW_SECS) > now)
        {
            return Err(OidcError::NotYetValid);
        }
        let sub = c
            .sub
            .filter(|s| !s.is_empty() && s.len() <= 255)
            .ok_or(OidcError::MissingClaim)?;
        Ok(Claims {
            sub,
            email: c.email.filter(|e| !e.is_empty()),
            email_verified: truthy(c.email_verified.as_ref()),
            nonce: c.nonce,
            name: c.name.filter(|n| !n.is_empty()),
        })
    }
}
