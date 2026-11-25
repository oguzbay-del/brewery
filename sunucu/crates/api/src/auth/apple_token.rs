//! Apple authorization-code exchange (ADR-0009 rule 1.5).

use std::time::Duration;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use ring::{
    rand::SystemRandom,
    signature::{self, EcdsaKeyPair},
};
use serde::Deserialize;

use super::config::AppleTokenSettings;

/// Errors from the Apple authorization-code exchange.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AppleTokenError {
    /// The authorization code is empty or too long.
    #[error("invalid Apple authorization code")]
    InvalidCode,
    /// The verified ID token has no single configured audience.
    #[error("Apple client ID is ambiguous or not configured")]
    InvalidClientId,
    /// The configured private key cannot sign a client secret.
    #[error("Apple client-secret signing failed")]
    ClientSecret,
    /// The Apple token endpoint returned an error or invalid response.
    #[error("Apple token endpoint request failed")]
    Request,
    /// Apple did not return a refresh token.
    #[error("Apple token response has no refresh token")]
    MissingRefreshToken,
}

#[derive(Deserialize)]
struct TokenResponse {
    refresh_token: Option<String>,
}

/// Exchanges Apple authorization codes and returns refresh tokens.
#[derive(Clone)]
pub struct AppleTokenClient {
    settings: AppleTokenSettings,
    http: reqwest::Client,
}

impl std::fmt::Debug for AppleTokenClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppleTokenClient")
            .field("team_id", &self.settings.team_id)
            .field("key_id", &self.settings.key_id)
            .field("private_key", &"<redacted>")
            .field("client_ids", &self.settings.client_ids)
            .field("token_url", &self.settings.token_url)
            .finish_non_exhaustive()
    }
}

impl AppleTokenClient {
    /// Creates a client with redirects disabled and a bounded request timeout.
    #[must_use]
    pub fn new(settings: AppleTokenSettings) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { settings, http }
    }

    /// Selects the one configured client ID present in the verified ID token audience.
    ///
    /// # Errors
    /// Returns an error when no unique configured audience can be selected.
    pub fn client_id_for_audiences<'a>(
        &self,
        audiences: &'a [String],
    ) -> Result<&'a str, AppleTokenError> {
        if audiences.len() != 1 {
            return Err(AppleTokenError::InvalidClientId);
        }
        let client_id = audiences.first().ok_or(AppleTokenError::InvalidClientId)?;
        if !self.settings.client_ids.iter().any(|id| id == client_id) {
            return Err(AppleTokenError::InvalidClientId);
        }
        Ok(client_id)
    }

    fn client_secret(&self, client_id: &str) -> Result<String, AppleTokenError> {
        let rng = SystemRandom::new();
        let key = EcdsaKeyPair::from_pkcs8(
            &signature::ECDSA_P256_SHA256_FIXED_SIGNING,
            &self.settings.private_key,
            &rng,
        )
        .map_err(|_| AppleTokenError::ClientSecret)?;
        let header = URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&serde_json::json!({
                "alg": "ES256",
                "kid": self.settings.key_id,
                "typ": "JWT"
            }))
            .map_err(|_| AppleTokenError::ClientSecret)?,
        );
        let now = Utc::now().timestamp();
        let claims = URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&serde_json::json!({
                "iss": self.settings.team_id,
                "iat": now,
                "exp": now + 300,
                "aud": "https://appleid.apple.com",
                "sub": client_id
            }))
            .map_err(|_| AppleTokenError::ClientSecret)?,
        );
        let signing_input = format!("{header}.{claims}");
        let signature = key
            .sign(&rng, signing_input.as_bytes())
            .map_err(|_| AppleTokenError::ClientSecret)?;
        Ok(format!(
            "{signing_input}.{}",
            URL_SAFE_NO_PAD.encode(signature.as_ref())
        ))
    }

    /// Exchanges a single-use authorization code for its refresh token.
    ///
    /// # Errors
    /// Returns an error when the code, client ID, signing key or Apple response is invalid.
    pub async fn exchange_code(
        &self,
        code: &str,
        client_id: &str,
    ) -> Result<String, AppleTokenError> {
        if code.is_empty() || code.len() > 4096 {
            return Err(AppleTokenError::InvalidCode);
        }
        if !self.settings.client_ids.iter().any(|id| id == client_id) {
            return Err(AppleTokenError::InvalidClientId);
        }
        let client_secret = self.client_secret(client_id)?;
        let mut form = url::form_urlencoded::Serializer::new(String::new());
        form.append_pair("grant_type", "authorization_code");
        form.append_pair("code", code);
        form.append_pair("client_id", client_id);
        form.append_pair("client_secret", &client_secret);
        let body = form.finish();
        let response = self
            .http
            .post(&self.settings.token_url)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(body)
            .send()
            .await
            .map_err(|_| AppleTokenError::Request)?
            .error_for_status()
            .map_err(|_| AppleTokenError::Request)?;
        let bytes = response
            .bytes()
            .await
            .map_err(|_| AppleTokenError::Request)?;
        if bytes.len() > 65_536 {
            return Err(AppleTokenError::Request);
        }
        let response: TokenResponse =
            serde_json::from_slice(&bytes).map_err(|_| AppleTokenError::Request)?;
        response
            .refresh_token
            .filter(|token| !token.is_empty())
            .ok_or(AppleTokenError::MissingRefreshToken)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use ring::signature::{KeyPair, UnparsedPublicKey};

    fn settings(private_key: Vec<u8>) -> AppleTokenSettings {
        AppleTokenSettings {
            team_id: "TEAM123".to_owned(),
            key_id: "KEY456".to_owned(),
            private_key,
            client_ids: vec!["com.example.ios".to_owned(), "com.example.web".to_owned()],
            token_url: "http://127.0.0.1:8080/auth/token".to_owned(),
            revoke_url: "http://127.0.0.1:8080/auth/revoke".to_owned(),
        }
    }

    #[test]
    fn client_secret_is_es256_and_uses_the_verified_audience() {
        let rng = SystemRandom::new();
        let pkcs8 = EcdsaKeyPair::generate_pkcs8(&signature::ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
            .unwrap();
        let key = EcdsaKeyPair::from_pkcs8(
            &signature::ECDSA_P256_SHA256_FIXED_SIGNING,
            pkcs8.as_ref(),
            &rng,
        )
        .unwrap();
        let public_key = key.public_key().as_ref().to_vec();
        let client = AppleTokenClient::new(settings(pkcs8.as_ref().to_vec()));
        let audiences = ["com.example.ios".to_owned()];
        let client_id = client.client_id_for_audiences(&audiences).unwrap();
        assert_eq!(client_id, "com.example.ios");

        let token = client.client_secret(client_id).unwrap();
        let mut parts = token.split('.');
        let header = URL_SAFE_NO_PAD.decode(parts.next().unwrap()).unwrap();
        let claims = URL_SAFE_NO_PAD.decode(parts.next().unwrap()).unwrap();
        let signature = URL_SAFE_NO_PAD.decode(parts.next().unwrap()).unwrap();
        assert!(parts.next().is_none());
        let header: serde_json::Value = serde_json::from_slice(&header).unwrap();
        let claims: serde_json::Value = serde_json::from_slice(&claims).unwrap();
        assert_eq!(header["alg"], "ES256");
        assert_eq!(header["kid"], "KEY456");
        assert_eq!(claims["iss"], "TEAM123");
        assert_eq!(claims["sub"], "com.example.ios");
        assert_eq!(claims["aud"], "https://appleid.apple.com");
        assert_eq!(
            claims["exp"].as_i64().unwrap() - claims["iat"].as_i64().unwrap(),
            300
        );
        let signing_input = token.rsplit_once('.').unwrap().0;
        UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_FIXED, public_key)
            .verify(signing_input.as_bytes(), &signature)
            .unwrap();
    }

    #[test]
    fn audience_selection_rejects_missing_or_ambiguous_client_ids() {
        let rng = SystemRandom::new();
        let pkcs8 = EcdsaKeyPair::generate_pkcs8(&signature::ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
            .unwrap();
        let client = AppleTokenClient::new(settings(pkcs8.as_ref().to_vec()));
        assert_eq!(
            client.client_id_for_audiences(&["unknown".to_owned()]),
            Err(AppleTokenError::InvalidClientId)
        );
        assert_eq!(
            client.client_id_for_audiences(&[
                "com.example.ios".to_owned(),
                "com.example.web".to_owned()
            ]),
            Err(AppleTokenError::InvalidClientId)
        );
    }
}
