//! Apple client: ES256 client secret and token revocation (ADR-0009 rule 1.5).

use std::time::Duration;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use ring::{
    rand::SystemRandom,
    signature::{self, EcdsaKeyPair},
};

use crate::{payload::ApplePayload, settings::AppleTokenSettings, token_cipher::TokenCipher};

/// Errors from client-secret signing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ClientSecretError {
    /// The configured private key cannot sign a client secret.
    #[error("Apple client-secret signing failed")]
    Signing,
}

/// Answer of one `/auth/revoke` call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevokeOutcome {
    /// Apple answered HTTP 200.
    Revoked,
    /// Apple answered `invalid_client`: this client ID did not issue the token.
    InvalidClient,
    /// Any other answer, or no answer.
    Failed,
}

/// Result of revoking one stored payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevokeResult {
    /// Apple revoked the token.
    Revoked,
    /// The revoke did not succeed. It counts as one failed attempt.
    Failed,
}

/// Signs client secrets and calls Apple endpoints.
#[derive(Clone)]
pub struct AppleClient {
    settings: AppleTokenSettings,
    http: reqwest::Client,
}

impl std::fmt::Debug for AppleClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppleClient")
            .field("settings", &self.settings)
            .finish_non_exhaustive()
    }
}

impl AppleClient {
    /// Creates a client with redirects disabled and a bounded request timeout.
    ///
    /// # Errors
    /// Returns an error if the HTTP client cannot be built.
    pub fn new(settings: AppleTokenSettings) -> Result<Self, reqwest::Error> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(Self { settings, http })
    }

    /// The settings of this client.
    #[must_use]
    pub const fn settings(&self) -> &AppleTokenSettings {
        &self.settings
    }

    /// The HTTP client (redirects off, 10 second timeout).
    #[must_use]
    pub const fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// Signs a short-lived ES256 client secret for `client_id`.
    ///
    /// # Errors
    /// Returns an error when the private key or JSON encoding fails.
    pub fn client_secret(&self, client_id: &str) -> Result<String, ClientSecretError> {
        let rng = SystemRandom::new();
        let key = EcdsaKeyPair::from_pkcs8(
            &signature::ECDSA_P256_SHA256_FIXED_SIGNING,
            &self.settings.private_key,
            &rng,
        )
        .map_err(|_| ClientSecretError::Signing)?;
        let header = URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&serde_json::json!({
                "alg": "ES256",
                "kid": self.settings.key_id,
                "typ": "JWT"
            }))
            .map_err(|_| ClientSecretError::Signing)?,
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
            .map_err(|_| ClientSecretError::Signing)?,
        );
        let signing_input = format!("{header}.{claims}");
        let signature = key
            .sign(&rng, signing_input.as_bytes())
            .map_err(|_| ClientSecretError::Signing)?;
        Ok(format!(
            "{signing_input}.{}",
            URL_SAFE_NO_PAD.encode(signature.as_ref())
        ))
    }

    /// Calls `/auth/revoke` for one refresh token and one client ID.
    pub async fn revoke(&self, refresh_token: &str, client_id: &str) -> RevokeOutcome {
        let Ok(client_secret) = self.client_secret(client_id) else {
            return RevokeOutcome::Failed;
        };
        let body = {
            let mut form = url::form_urlencoded::Serializer::new(String::new());
            form.append_pair("token", refresh_token);
            form.append_pair("token_type_hint", "refresh_token");
            form.append_pair("client_id", client_id);
            form.append_pair("client_secret", &client_secret);
            form.finish()
        };
        let Ok(response) = self
            .http
            .post(&self.settings.revoke_url)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(body)
            .send()
            .await
        else {
            return RevokeOutcome::Failed;
        };
        let status = response.status();
        if status == reqwest::StatusCode::OK {
            return RevokeOutcome::Revoked;
        }
        if status == reqwest::StatusCode::BAD_REQUEST
            && let Ok(bytes) = response.bytes().await
            && bytes.len() <= 65_536
            && let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes)
            && value.get("error").and_then(serde_json::Value::as_str) == Some("invalid_client")
        {
            return RevokeOutcome::InvalidClient;
        }
        RevokeOutcome::Failed
    }

    /// Decrypts a stored payload and revokes its token.
    ///
    /// A version 1 payload uses its own client ID. An old payload tries every configured client
    /// ID in list order and stops at the first HTTP 200. An `invalid_client` answer moves on to
    /// the next ID. When every ID answers `invalid_client`, the result is one failed attempt.
    pub async fn revoke_stored(&self, cipher: &dyn TokenCipher, ciphertext: &[u8]) -> RevokeResult {
        let Ok(plaintext) = cipher.decrypt(ciphertext) else {
            tracing::warn!("stored Apple token cannot be decrypted");
            return RevokeResult::Failed;
        };
        let Ok(payload) = ApplePayload::decode(&plaintext) else {
            tracing::warn!("stored Apple token payload is invalid");
            return RevokeResult::Failed;
        };
        let candidates: Vec<&str> = payload.client_id.as_deref().map_or_else(
            || {
                self.settings
                    .client_ids
                    .iter()
                    .map(String::as_str)
                    .collect()
            },
            |id| vec![id],
        );
        for client_id in candidates {
            match self.revoke(&payload.refresh_token, client_id).await {
                RevokeOutcome::Revoked => return RevokeResult::Revoked,
                RevokeOutcome::InvalidClient => {}
                RevokeOutcome::Failed => return RevokeResult::Failed,
            }
        }
        RevokeResult::Failed
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::sync::{Arc, Mutex};

    use ring::signature::{KeyPair, UnparsedPublicKey};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    use super::*;
    use crate::token_cipher::Aes256GcmTokenCipher;

    fn pkcs8() -> Vec<u8> {
        let rng = SystemRandom::new();
        EcdsaKeyPair::generate_pkcs8(&signature::ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
            .unwrap()
            .as_ref()
            .to_vec()
    }

    fn settings(private_key: Vec<u8>, ids: &[&str], revoke_url: &str) -> AppleTokenSettings {
        AppleTokenSettings {
            team_id: "TEAM123".to_owned(),
            key_id: "KEY456".to_owned(),
            private_key,
            client_ids: ids.iter().map(|s| (*s).to_owned()).collect(),
            token_url: "http://127.0.0.1:1/auth/token".to_owned(),
            revoke_url: revoke_url.to_owned(),
        }
    }

    /// Serves scripted answers and records each request body.
    async fn fake_apple(answers: Vec<(u16, &'static str)>) -> (String, Arc<Mutex<Vec<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/auth/revoke", listener.local_addr().unwrap());
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = Arc::clone(&seen);
        tokio::spawn(async move {
            for (status, body) in answers {
                let Ok((mut socket, _)) = listener.accept().await else {
                    return;
                };
                let mut buf = vec![0_u8; 8192];
                let mut read = 0;
                loop {
                    let n = socket.read(&mut buf[read..]).await.unwrap();
                    read += n;
                    let text = String::from_utf8_lossy(&buf[..read]).into_owned();
                    if let Some((head, rest)) = text.split_once("\r\n\r\n") {
                        let length = head
                            .lines()
                            .find_map(|l| {
                                l.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .map(|v| v.trim().parse::<usize>().unwrap())
                            })
                            .unwrap_or(0);
                        if rest.len() >= length || n == 0 {
                            log.lock().unwrap().push(rest.to_owned());
                            break;
                        }
                    } else if n == 0 {
                        break;
                    }
                }
                let reply = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                socket.write_all(reply.as_bytes()).await.unwrap();
                socket.shutdown().await.unwrap();
            }
        });
        (url, seen)
    }

    fn encrypted(cipher: &Aes256GcmTokenCipher, plaintext: &[u8]) -> Vec<u8> {
        cipher.encrypt(plaintext).unwrap()
    }

    #[test]
    fn client_secret_is_es256_and_uses_the_client_id() {
        let rng = SystemRandom::new();
        let pkcs8 = pkcs8();
        let key =
            EcdsaKeyPair::from_pkcs8(&signature::ECDSA_P256_SHA256_FIXED_SIGNING, &pkcs8, &rng)
                .unwrap();
        let public_key = key.public_key().as_ref().to_vec();
        let client = AppleClient::new(settings(pkcs8, &["com.example.ios"], "http://x/")).unwrap();
        let token = client.client_secret("com.example.ios").unwrap();
        let mut parts = token.split('.');
        let header: serde_json::Value =
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(parts.next().unwrap()).unwrap())
                .unwrap();
        let claims: serde_json::Value =
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(parts.next().unwrap()).unwrap())
                .unwrap();
        let signature = URL_SAFE_NO_PAD.decode(parts.next().unwrap()).unwrap();
        assert_eq!(header["alg"], "ES256");
        assert_eq!(header["kid"], "KEY456");
        assert_eq!(claims["iss"], "TEAM123");
        assert_eq!(claims["sub"], "com.example.ios");
        assert_eq!(claims["aud"], "https://appleid.apple.com");
        UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_FIXED, public_key)
            .verify(token.rsplit_once('.').unwrap().0.as_bytes(), &signature)
            .unwrap();
    }

    #[tokio::test]
    async fn v1_payload_revokes_with_its_own_client_id() {
        let (url, seen) = fake_apple(vec![(200, "{}")]).await;
        let client = AppleClient::new(settings(
            pkcs8(),
            &["com.example.ios", "com.example.web"],
            &url,
        ))
        .unwrap();
        let cipher = Aes256GcmTokenCipher::new([5; 32]);
        let stored = encrypted(
            &cipher,
            &ApplePayload::encode_v1("com.example.web", "r.secret.token").unwrap(),
        );
        assert_eq!(
            client.revoke_stored(&cipher, &stored).await,
            RevokeResult::Revoked
        );
        let bodies = seen.lock().unwrap();
        assert_eq!(bodies.len(), 1);
        assert!(bodies[0].contains("client_id=com.example.web"));
        assert!(bodies[0].contains("token=r.secret.token"));
        assert!(bodies[0].contains("client_secret="));
    }

    #[tokio::test]
    async fn old_payload_with_one_id_uses_that_id() {
        let (url, seen) = fake_apple(vec![(200, "{}")]).await;
        let client = AppleClient::new(settings(pkcs8(), &["com.example.ios"], &url)).unwrap();
        let cipher = Aes256GcmTokenCipher::new([5; 32]);
        let stored = encrypted(&cipher, b"r.old.token");
        assert_eq!(
            client.revoke_stored(&cipher, &stored).await,
            RevokeResult::Revoked
        );
        assert!(seen.lock().unwrap()[0].contains("client_id=com.example.ios"));
    }

    #[tokio::test]
    async fn old_payload_tries_ids_in_order_after_invalid_client() {
        let (url, seen) =
            fake_apple(vec![(400, r#"{"error":"invalid_client"}"#), (200, "{}")]).await;
        let client = AppleClient::new(settings(
            pkcs8(),
            &["com.example.ios", "com.example.web"],
            &url,
        ))
        .unwrap();
        let cipher = Aes256GcmTokenCipher::new([5; 32]);
        let stored = encrypted(&cipher, b"r.old.token");
        assert_eq!(
            client.revoke_stored(&cipher, &stored).await,
            RevokeResult::Revoked
        );
        let bodies = seen.lock().unwrap();
        assert_eq!(bodies.len(), 2);
        assert!(bodies[0].contains("client_id=com.example.ios"));
        assert!(bodies[1].contains("client_id=com.example.web"));
    }

    #[tokio::test]
    async fn other_errors_and_exhausted_ids_fail() {
        let (url, _) = fake_apple(vec![(500, "{}")]).await;
        let client = AppleClient::new(settings(pkcs8(), &["a", "b"], &url)).unwrap();
        let cipher = Aes256GcmTokenCipher::new([5; 32]);
        let stored = encrypted(&cipher, b"r.old.token");
        assert_eq!(
            client.revoke_stored(&cipher, &stored).await,
            RevokeResult::Failed
        );
        let invalid = (400, r#"{"error":"invalid_client"}"#);
        let (url, seen) = fake_apple(vec![invalid, invalid]).await;
        let client = AppleClient::new(settings(pkcs8(), &["a", "b"], &url)).unwrap();
        assert_eq!(
            client.revoke_stored(&cipher, &stored).await,
            RevokeResult::Failed
        );
        assert_eq!(seen.lock().unwrap().len(), 2);
        assert_eq!(
            client
                .revoke_stored(&Aes256GcmTokenCipher::new([6; 32]), &stored)
                .await,
            RevokeResult::Failed
        );
    }
}
