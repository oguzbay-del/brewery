//! Apple authorization-code exchange (ADR-0009 rule 1.5).

use apple_auth::{AppleClient, AppleTokenSettings};
use serde::Deserialize;

use super::oidc::{OidcError, OidcVerifier};

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
    /// Apple did not return an ID token.
    #[error("Apple token response has no ID token")]
    MissingIdToken,
    /// The response ID token could not be verified.
    #[error("Apple token response ID token is invalid")]
    InvalidIdToken,
    /// The response ID token belongs to a different identity or client.
    #[error("Apple token response identity does not match the sign-in identity")]
    IdentityMismatch,
    /// The Apple JWKS endpoint could not be reached.
    #[error("Apple ID token verification is temporarily unavailable")]
    VerificationUnavailable,
}

#[derive(Deserialize)]
struct TokenResponse {
    refresh_token: Option<String>,
    id_token: Option<String>,
}

/// Tokens of one web code exchange.
#[derive(Debug)]
pub struct WebTokens {
    /// The ID token. The caller verifies it.
    pub id_token: String,
    /// The refresh token, if Apple sent one.
    pub refresh_token: Option<String>,
}

/// Exchanges Apple authorization codes and returns refresh tokens.
#[derive(Clone, Debug)]
pub struct AppleTokenClient {
    inner: AppleClient,
}

impl AppleTokenClient {
    /// Creates a client with redirects disabled and a bounded request timeout.
    ///
    /// # Errors
    /// Returns an error if the HTTP client cannot be built.
    pub fn new(settings: AppleTokenSettings) -> Result<Self, reqwest::Error> {
        Ok(Self {
            inner: AppleClient::new(settings)?,
        })
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
        if !self
            .inner
            .settings()
            .client_ids
            .iter()
            .any(|id| id == client_id)
        {
            return Err(AppleTokenError::InvalidClientId);
        }
        Ok(client_id)
    }

    /// Exchanges the code of the web redirect flow (T3.1). The caller verifies the ID token and
    /// its nonce. `client_id` is the web service id; `redirect_uri` is the one sent to Apple.
    ///
    /// # Errors
    /// Returns an error when the code, client ID, signing key or Apple response is invalid.
    pub async fn exchange_web_code(
        &self,
        token_url: &str,
        code: &str,
        client_id: &str,
        redirect_uri: &str,
    ) -> Result<WebTokens, AppleTokenError> {
        if code.is_empty() || code.len() > 4096 {
            return Err(AppleTokenError::InvalidCode);
        }
        if !self
            .inner
            .settings()
            .client_ids
            .iter()
            .any(|id| id == client_id)
        {
            return Err(AppleTokenError::InvalidClientId);
        }
        let client_secret = self
            .inner
            .client_secret(client_id)
            .map_err(|_| AppleTokenError::ClientSecret)?;
        let body = {
            let mut form = url::form_urlencoded::Serializer::new(String::new());
            form.append_pair("grant_type", "authorization_code");
            form.append_pair("code", code);
            form.append_pair("client_id", client_id);
            form.append_pair("client_secret", &client_secret);
            form.append_pair("redirect_uri", redirect_uri);
            form.finish()
        };
        let response = self
            .inner
            .http()
            .post(token_url)
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
        Ok(WebTokens {
            id_token: response
                .id_token
                .filter(|token| !token.is_empty())
                .ok_or(AppleTokenError::MissingIdToken)?,
            refresh_token: response.refresh_token.filter(|token| !token.is_empty()),
        })
    }

    /// Exchanges a single-use authorization code for its refresh token.
    ///
    /// # Errors
    /// Returns an error when the code, client ID, signing key or Apple response is invalid.
    pub async fn exchange_code(
        &self,
        code: &str,
        client_id: &str,
        verifier: &OidcVerifier,
        expected_subject: &str,
    ) -> Result<String, AppleTokenError> {
        if code.is_empty() || code.len() > 4096 {
            return Err(AppleTokenError::InvalidCode);
        }
        if !self
            .inner
            .settings()
            .client_ids
            .iter()
            .any(|id| id == client_id)
        {
            return Err(AppleTokenError::InvalidClientId);
        }
        let client_secret = self
            .inner
            .client_secret(client_id)
            .map_err(|_| AppleTokenError::ClientSecret)?;
        let body = {
            let mut form = url::form_urlencoded::Serializer::new(String::new());
            form.append_pair("grant_type", "authorization_code");
            form.append_pair("code", code);
            form.append_pair("client_id", client_id);
            form.append_pair("client_secret", &client_secret);
            form.finish()
        };
        let response = self
            .inner
            .http()
            .post(&self.inner.settings().token_url)
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
        let id_token = response
            .id_token
            .filter(|token| !token.is_empty())
            .ok_or(AppleTokenError::MissingIdToken)?;
        let claims = verifier.verify(&id_token).await.map_err(|error| {
            if error == OidcError::JwksUnavailable {
                AppleTokenError::VerificationUnavailable
            } else {
                AppleTokenError::InvalidIdToken
            }
        })?;
        if claims.sub != expected_subject
            || claims.audiences.len() != 1
            || claims
                .audiences
                .first()
                .is_none_or(|audience| audience != client_id)
        {
            return Err(AppleTokenError::IdentityMismatch);
        }
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
    use ring::{
        rand::SystemRandom,
        signature::{self, EcdsaKeyPair},
    };

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
    fn audience_selection_rejects_missing_or_ambiguous_client_ids() {
        let rng = SystemRandom::new();
        let pkcs8 = EcdsaKeyPair::generate_pkcs8(&signature::ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
            .unwrap();
        let client = AppleTokenClient::new(settings(pkcs8.as_ref().to_vec())).unwrap();
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
