//! Sign-in settings, read from the environment (ADR-0009 rule 11).

use common::{Error, Result, config::secret_from_lookup};

/// Settings of one OIDC provider (ADR-0009 rule 1.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSettings {
    /// Accepted `iss` values.
    pub issuers: Vec<String>,
    /// Accepted `aud` values (the client ids: iOS bundle id, web service id, ...).
    pub audiences: Vec<String>,
    /// Where the provider publishes its JWKS.
    pub jwks_url: String,
}

/// SMTP relay for the e-mail link (mailpit locally).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmtpSettings {
    /// Host.
    pub host: String,
    /// Port.
    pub port: u16,
}

/// Sign-in configuration.
#[derive(Clone)]
pub struct AuthConfig {
    /// Sign in with Apple; `None` while `APPLE_CLIENT_IDS` is unset.
    pub apple: Option<ProviderSettings>,
    /// Google sign-in; `None` while `GOOGLE_CLIENT_IDS` is unset.
    pub google: Option<ProviderSettings>,
    /// Base address of the web app, used in the e-mail link.
    pub web_base_url: String,
    /// Origins allowed on cookie-authenticated writes (CSRF, ADR-0009 rule 3.5).
    pub web_origins: Vec<String>,
    /// HMAC key of the CSRF token.
    pub csrf_key: Vec<u8>,
    /// SMTP relay; `None` logs the link instead of mailing it (never in production).
    pub smtp: Option<SmtpSettings>,
    /// `From` address of the sign-in mail.
    pub mail_from: String,
    /// Shortest gap between two JWKS refreshes triggered by unknown `kid`s.
    pub jwks_min_refresh_secs: u64,
}

impl std::fmt::Debug for AuthConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthConfig")
            .field("apple", &self.apple)
            .field("google", &self.google)
            .field("web_base_url", &self.web_base_url)
            .field("web_origins", &self.web_origins)
            .field("csrf_key", &"<redacted>")
            .field("smtp", &self.smtp)
            .field("mail_from", &self.mail_from)
            .finish_non_exhaustive()
    }
}

fn list(raw: Option<String>) -> Vec<String> {
    raw.map(|v| {
        v.split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect()
    })
    .unwrap_or_default()
}

fn provider<F>(
    lookup: &F,
    prefix: &str,
    default_issuers: &[&str],
    default_jwks: &str,
) -> Option<ProviderSettings>
where
    F: Fn(&str) -> Option<String>,
{
    let audiences = list(lookup(&format!("{prefix}_CLIENT_IDS")));
    if audiences.is_empty() {
        return None;
    }
    let issuers = match lookup(&format!("{prefix}_ISSUER")).filter(|v| !v.trim().is_empty()) {
        Some(issuer) => vec![issuer.trim().to_owned()],
        None => default_issuers.iter().map(|s| (*s).to_owned()).collect(),
    };
    let jwks_url = lookup(&format!("{prefix}_JWKS_URL"))
        .filter(|v| !v.trim().is_empty())
        .map_or_else(|| default_jwks.to_owned(), |v| v.trim().to_owned());
    Some(ProviderSettings {
        issuers,
        audiences,
        jwks_url,
    })
}

/// `https://app.example.com/some/path` → `https://app.example.com`.
fn origin_of(url: &str) -> String {
    let scheme_end = url.find("://").map_or(0, |i| i + 3);
    let rest = &url[scheme_end..];
    let host_end = rest.find('/').unwrap_or(rest.len());
    url[..scheme_end + host_end].to_owned()
}

impl AuthConfig {
    /// Reads the settings from `lookup`.
    pub fn from_lookup<F>(lookup: &F) -> Result<Self>
    where
        F: Fn(&str) -> Option<String>,
    {
        let web_base_url = lookup("WEB_BASE_URL")
            .filter(|v| !v.trim().is_empty())
            .map_or_else(
                || "http://localhost:3000".to_owned(),
                |v| v.trim().trim_end_matches('/').to_owned(),
            );
        let mut web_origins = list(lookup("WEB_ORIGINS"));
        if web_origins.is_empty() {
            web_origins.push(origin_of(&web_base_url));
        }
        let csrf_key = match secret_from_lookup(lookup, "CSRF_HMAC_KEY", "CSRF_HMAC_KEY_FILE")? {
            Some(key) if key.len() >= 16 => key.into_bytes(),
            Some(_) => {
                return Err(Error::InvalidEnv {
                    name: "CSRF_HMAC_KEY",
                    reason: "must be at least 16 characters".to_owned(),
                });
            }
            None => {
                tracing::warn!(
                    "CSRF_HMAC_KEY not set; using a random key (CSRF tokens do not survive restarts or work across replicas)"
                );
                crate::auth::tokens::random_bytes::<32>().to_vec()
            }
        };
        let smtp = match lookup("SMTP_HOST").filter(|v| !v.trim().is_empty()) {
            Some(host) => {
                let port = lookup("SMTP_PORT")
                    .filter(|v| !v.trim().is_empty())
                    .map_or(Ok(25), |p| p.trim().parse::<u16>())
                    .map_err(|e| Error::InvalidEnv {
                        name: "SMTP_PORT",
                        reason: e.to_string(),
                    })?;
                Some(SmtpSettings {
                    host: host.trim().to_owned(),
                    port,
                })
            }
            None => None,
        };
        let jwks_min_refresh_secs = lookup("JWKS_MIN_REFRESH_SECS")
            .filter(|v| !v.trim().is_empty())
            .map_or(Ok(30), |v| v.trim().parse::<u64>())
            .map_err(|e| Error::InvalidEnv {
                name: "JWKS_MIN_REFRESH_SECS",
                reason: e.to_string(),
            })?;
        Ok(Self {
            apple: provider(
                lookup,
                "APPLE",
                &["https://appleid.apple.com"],
                "https://appleid.apple.com/auth/keys",
            ),
            google: provider(
                lookup,
                "GOOGLE",
                &["https://accounts.google.com", "accounts.google.com"],
                "https://www.googleapis.com/oauth2/v3/certs",
            ),
            web_base_url,
            web_origins,
            csrf_key,
            smtp,
            mail_from: lookup("MAIL_FROM")
                .filter(|v| !v.trim().is_empty())
                .map_or_else(
                    || "no-reply@demleme.local".to_owned(),
                    |v| v.trim().to_owned(),
                ),
            jwks_min_refresh_secs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn providers_need_client_ids_and_take_overrides() {
        let lookup = |n: &str| match n {
            "APPLE_CLIENT_IDS" => Some("com.example.app, web.example".to_owned()),
            "APPLE_ISSUER" => Some("http://fake/apple".to_owned()),
            "WEB_BASE_URL" => Some("https://app.example.com/".to_owned()),
            _ => None,
        };
        let cfg = AuthConfig::from_lookup(&lookup).unwrap();
        let apple = cfg.apple.unwrap();
        assert_eq!(apple.audiences, ["com.example.app", "web.example"]);
        assert_eq!(apple.issuers, ["http://fake/apple"]);
        assert_eq!(apple.jwks_url, "https://appleid.apple.com/auth/keys");
        assert!(cfg.google.is_none());
        assert_eq!(cfg.web_origins, ["https://app.example.com"]);
    }

    #[test]
    fn short_csrf_key_is_rejected() {
        let lookup = |n: &str| (n == "CSRF_HMAC_KEY").then(|| "short".to_owned());
        assert!(AuthConfig::from_lookup(&lookup).is_err());
    }
}
