//! Apple token-endpoint settings and their environment parser (ADR-0009 rule 1.5).

use base64::{Engine as _, engine::general_purpose::STANDARD};
use common::{Error, Result, config::secret_from_lookup};

/// Apple token-endpoint settings.
#[derive(Clone)]
pub struct AppleTokenSettings {
    /// Apple developer team identifier.
    pub team_id: String,
    /// Apple key identifier.
    pub key_id: String,
    /// PKCS#8 DER bytes of the Apple .p8 private key.
    pub private_key: Vec<u8>,
    /// Accepted Apple client IDs, also used to select the code's audience.
    pub client_ids: Vec<String>,
    /// Apple token endpoint (a local mock URL in compose).
    pub token_url: String,
    /// Apple revoke endpoint (a local mock URL in compose).
    pub revoke_url: String,
}

impl std::fmt::Debug for AppleTokenSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppleTokenSettings")
            .field("team_id", &self.team_id)
            .field("key_id", &self.key_id)
            .field("private_key", &"<redacted>")
            .field("client_ids", &self.client_ids)
            .field("token_url", &self.token_url)
            .field("revoke_url", &self.revoke_url)
            .finish()
    }
}

/// Splits a comma-separated list such as `APPLE_CLIENT_IDS`.
#[must_use]
pub fn client_ids_from_lookup<F>(lookup: &F) -> Vec<String>
where
    F: Fn(&str) -> Option<String>,
{
    lookup("APPLE_CLIENT_IDS")
        .map(|v| {
            v.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// Decodes a PEM-encoded PKCS#8 private key.
fn decode_private_key(raw: &str) -> Result<Vec<u8>> {
    let encoded: String = raw
        .lines()
        .filter(|line| !line.trim_start().starts_with("-----"))
        .map(str::trim)
        .collect();
    STANDARD.decode(encoded).map_err(|_| Error::InvalidEnv {
        name: "APPLE_PRIVATE_KEY",
        reason: "invalid PKCS#8 base64".to_owned(),
    })
}

fn endpoint_url(raw: Option<String>, name: &'static str, default: &'static str) -> Result<String> {
    let value = raw
        .filter(|v| !v.trim().is_empty())
        .map_or_else(|| default.to_owned(), |v| v.trim().to_owned());
    let parsed = url::Url::parse(&value).map_err(|e| Error::InvalidEnv {
        name,
        reason: format!("invalid URL: {e}"),
    })?;
    let local_host = matches!(
        parsed.host_str(),
        Some("localhost" | "127.0.0.1" | "apple-mock" | "::1" | "[::1]")
    );
    if parsed.scheme() != "https" && !(parsed.scheme() == "http" && local_host) {
        return Err(Error::InvalidEnv {
            name,
            reason: "must use HTTPS except for the local Apple mock".to_owned(),
        });
    }
    if !parsed.username().is_empty() || parsed.password().is_some() || parsed.fragment().is_some() {
        return Err(Error::InvalidEnv {
            name,
            reason: "must not contain user information or a fragment".to_owned(),
        });
    }
    Ok(value)
}

impl AppleTokenSettings {
    /// Reads the settings from `lookup`. `client_ids` are the accepted Apple client IDs.
    ///
    /// Returns `None` when no Apple secret is set.
    ///
    /// # Errors
    /// Returns an error when the secrets are partial or a value is invalid.
    pub fn from_lookup<F>(lookup: &F, client_ids: &[String]) -> Result<Option<Self>>
    where
        F: Fn(&str) -> Option<String>,
    {
        let team_id = lookup("APPLE_TEAM_ID").filter(|v| !v.trim().is_empty());
        let key_id = lookup("APPLE_KEY_ID").filter(|v| !v.trim().is_empty());
        let private_key =
            secret_from_lookup(lookup, "APPLE_PRIVATE_KEY", "APPLE_PRIVATE_KEY_FILE")?;
        match (team_id, key_id, private_key) {
            (None, None, None) => Ok(None),
            (Some(team_id), Some(key_id), Some(private_key)) => {
                if client_ids.is_empty() {
                    return Err(Error::InvalidEnv {
                        name: "APPLE_CLIENT_IDS",
                        reason: "must be set when Apple token exchange is configured".to_owned(),
                    });
                }
                let private_key = decode_private_key(&private_key)?;
                if private_key.is_empty() {
                    return Err(Error::InvalidEnv {
                        name: "APPLE_PRIVATE_KEY",
                        reason: "must contain a PKCS#8 key".to_owned(),
                    });
                }
                Ok(Some(Self {
                    team_id: team_id.trim().to_owned(),
                    key_id: key_id.trim().to_owned(),
                    private_key,
                    client_ids: client_ids.to_vec(),
                    token_url: endpoint_url(
                        lookup("APPLE_TOKEN_URL"),
                        "APPLE_TOKEN_URL",
                        "https://appleid.apple.com/auth/token",
                    )?,
                    revoke_url: endpoint_url(
                        lookup("APPLE_REVOKE_URL"),
                        "APPLE_REVOKE_URL",
                        "https://appleid.apple.com/auth/revoke",
                    )?,
                }))
            }
            _ => Err(Error::InvalidEnv {
                name: "APPLE_TEAM_ID",
                reason: "APPLE_TEAM_ID, APPLE_KEY_ID and APPLE_PRIVATE_KEY must be set together"
                    .to_owned(),
            }),
        }
    }
}
