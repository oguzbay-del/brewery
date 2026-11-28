//! Sign-in settings, read from the environment (ADR-0009 rule 11).

use common::{Error, Result, config::secret_from_lookup};

use apple_auth::{Aes256GcmTokenCipher, AppleTokenSettings};

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

/// Settings of the web redirect code flow of one provider (T3.1).
#[derive(Clone, PartialEq, Eq)]
pub struct WebFlowProvider {
    /// Address of the provider authorize page. The browser opens it.
    pub authorize_url: String,
    /// Address of the provider token endpoint. Only the server calls it.
    pub token_url: String,
    /// Client id of the web app (Apple: the service id). It must be in the `*_CLIENT_IDS` list.
    pub client_id: String,
    /// Client secret. Google only; Apple signs a client secret JWT.
    pub client_secret: Option<String>,
}

impl std::fmt::Debug for WebFlowProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebFlowProvider")
            .field("authorize_url", &self.authorize_url)
            .field("token_url", &self.token_url)
            .field("client_id", &self.client_id)
            .field(
                "client_secret",
                &self.client_secret.as_ref().map(|_| "<redacted>"),
            )
            .finish()
    }
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
    /// Apple authorization-code exchange settings; absent until all Apple secrets are set.
    pub apple_token: Option<AppleTokenSettings>,
    /// AES-256-GCM cipher for stored provider refresh tokens.
    pub provider_token_cipher: Option<Aes256GcmTokenCipher>,
    /// Web redirect flow of Apple; `None` while `APPLE_WEB_CLIENT_ID` is unset.
    pub web_apple: Option<WebFlowProvider>,
    /// Web redirect flow of Google; `None` while `GOOGLE_WEB_CLIENT_ID` or the secret is unset.
    pub web_google: Option<WebFlowProvider>,
    /// Public base address of the callback routes (`WEB_AUTH_REDIRECT_BASE_URL`).
    /// The default is `web_base_url`. The `__Host-` cookies tie the flow to this host.
    pub web_redirect_base_url: String,
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
    /// `WebAuthn` relying party id: the domain web and iOS share (ADR-0009 rule 5.1).
    pub webauthn_rp_id: String,
    /// Relying party name shown by the authenticator.
    pub webauthn_rp_name: String,
    /// Origins `WebAuthn` accepts in `clientDataJSON` (web address, `https://<rp id>` for iOS).
    pub webauthn_origins: Vec<String>,
}

/// Host part of `https://host:port/path`.
fn host_of(url: &str) -> String {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let authority = rest.split('/').next().unwrap_or_default();
    let authority = authority.rsplit('@').next().unwrap_or_default();
    authority
        .rsplit_once(':')
        .filter(|(_, port)| port.bytes().all(|b| b.is_ascii_digit()))
        .map_or(authority, |(h, _)| h)
        .to_owned()
}

impl std::fmt::Debug for AuthConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthConfig")
            .field("apple", &self.apple)
            .field("google", &self.google)
            .field("apple_token", &self.apple_token.is_some())
            .field(
                "provider_token_cipher",
                &self.provider_token_cipher.is_some(),
            )
            .field("web_apple", &self.web_apple)
            .field("web_google", &self.web_google)
            .field("web_redirect_base_url", &self.web_redirect_base_url)
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

fn non_empty<F>(lookup: &F, name: &str) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(name)
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

/// Reads the web flow settings of one provider. `secret` is the Google client secret.
fn web_flow<F>(
    lookup: &F,
    prefix: &str,
    audiences: &[String],
    defaults: (&str, &str),
    secret: Option<String>,
) -> Result<Option<WebFlowProvider>>
where
    F: Fn(&str) -> Option<String>,
{
    let Some(client_id) = non_empty(lookup, &format!("{prefix}_WEB_CLIENT_ID")) else {
        return Ok(None);
    };
    if !audiences.contains(&client_id) {
        return Err(Error::InvalidEnv {
            name: if prefix == "APPLE" {
                "APPLE_WEB_CLIENT_ID"
            } else {
                "GOOGLE_WEB_CLIENT_ID"
            },
            reason: format!("must be listed in {prefix}_CLIENT_IDS"),
        });
    }
    Ok(Some(WebFlowProvider {
        authorize_url: non_empty(lookup, &format!("{prefix}_AUTHORIZE_URL"))
            .unwrap_or_else(|| defaults.0.to_owned()),
        token_url: non_empty(lookup, &format!("{prefix}_WEB_TOKEN_URL"))
            .unwrap_or_else(|| defaults.1.to_owned()),
        client_id,
        client_secret: secret,
    }))
}

/// The web redirect flow settings: Apple, Google and the public base address of the callbacks.
fn web_settings<F>(
    lookup: &F,
    apple_ids: &[String],
    google_ids: &[String],
    web_base_url: &str,
) -> Result<(Option<WebFlowProvider>, Option<WebFlowProvider>, String)>
where
    F: Fn(&str) -> Option<String>,
{
    let web_apple = web_flow(
        lookup,
        "APPLE",
        apple_ids,
        (
            "https://appleid.apple.com/auth/authorize",
            "https://appleid.apple.com/auth/token",
        ),
        None,
    )?;
    let google_secret =
        secret_from_lookup(lookup, "GOOGLE_CLIENT_SECRET", "GOOGLE_CLIENT_SECRET_FILE")?
            .map(|v| v.trim().to_owned())
            .filter(|v| !v.is_empty());
    // Google needs the secret for the code exchange; without it the flow stays off.
    let web_google = web_flow(
        lookup,
        "GOOGLE",
        google_ids,
        (
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
        ),
        google_secret.clone(),
    )?
    .filter(|_| google_secret.is_some());
    let redirect_base = non_empty(lookup, "WEB_AUTH_REDIRECT_BASE_URL").map_or_else(
        || web_base_url.to_owned(),
        |v| v.trim_end_matches('/').to_owned(),
    );
    Ok((web_apple, web_google, redirect_base))
}

/// `https://app.example.com/some/path` → `https://app.example.com`.
fn origin_of(url: &str) -> String {
    let scheme_end = url.find("://").map_or(0, |i| i + 3);
    let rest = &url[scheme_end..];
    let host_end = rest.find('/').unwrap_or(rest.len());
    url[..scheme_end + host_end].to_owned()
}

fn csrf_key<F>(lookup: &F) -> Result<Vec<u8>>
where
    F: Fn(&str) -> Option<String>,
{
    match secret_from_lookup(lookup, "CSRF_HMAC_KEY", "CSRF_HMAC_KEY_FILE")? {
        Some(key) if key.len() >= 16 => Ok(key.into_bytes()),
        Some(_) => Err(Error::InvalidEnv {
            name: "CSRF_HMAC_KEY",
            reason: "must be at least 16 characters".to_owned(),
        }),
        None => {
            tracing::warn!(
                "CSRF_HMAC_KEY not set; using a random key (CSRF tokens do not survive restarts or work across replicas)"
            );
            Ok(crate::auth::tokens::random_bytes::<32>().to_vec())
        }
    }
}

fn smtp_settings<F>(lookup: &F) -> Result<Option<SmtpSettings>>
where
    F: Fn(&str) -> Option<String>,
{
    let Some(host) = lookup("SMTP_HOST").filter(|v| !v.trim().is_empty()) else {
        return Ok(None);
    };
    let port = lookup("SMTP_PORT")
        .filter(|v| !v.trim().is_empty())
        .map_or(Ok(25), |p| p.trim().parse::<u16>())
        .map_err(|e| Error::InvalidEnv {
            name: "SMTP_PORT",
            reason: e.to_string(),
        })?;
    Ok(Some(SmtpSettings {
        host: host.trim().to_owned(),
        port,
    }))
}

fn jwks_min_refresh_secs<F>(lookup: &F) -> Result<u64>
where
    F: Fn(&str) -> Option<String>,
{
    lookup("JWKS_MIN_REFRESH_SECS")
        .filter(|v| !v.trim().is_empty())
        .map_or(Ok(30), |v| v.trim().parse::<u64>())
        .map_err(|e| Error::InvalidEnv {
            name: "JWKS_MIN_REFRESH_SECS",
            reason: e.to_string(),
        })
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
        let csrf_key = csrf_key(lookup)?;
        let smtp = smtp_settings(lookup)?;
        let jwks_min_refresh_secs = jwks_min_refresh_secs(lookup)?;
        let webauthn_rp_id = lookup("WEBAUTHN_RP_ID")
            .filter(|v| !v.trim().is_empty())
            .map_or_else(|| host_of(&web_base_url), |v| v.trim().to_lowercase());
        let mut webauthn_origins = list(lookup("WEBAUTHN_ORIGINS"));
        if webauthn_origins.is_empty() {
            webauthn_origins.clone_from(&web_origins);
            let ios = format!("https://{webauthn_rp_id}");
            if !webauthn_origins.contains(&ios) {
                webauthn_origins.push(ios);
            }
        }
        let apple = provider(
            lookup,
            "APPLE",
            &["https://appleid.apple.com"],
            "https://appleid.apple.com/auth/keys",
        );
        let google = provider(
            lookup,
            "GOOGLE",
            &["https://accounts.google.com", "accounts.google.com"],
            "https://www.googleapis.com/oauth2/v3/certs",
        );
        let apple_ids = apple
            .as_ref()
            .map(|p| p.audiences.clone())
            .unwrap_or_default();
        let (web_apple, web_google, web_redirect_base_url) = web_settings(
            lookup,
            &apple_ids,
            google
                .as_ref()
                .map(|p| p.audiences.as_slice())
                .unwrap_or_default(),
            &web_base_url,
        )?;
        let apple_token = AppleTokenSettings::from_lookup(lookup, &apple_ids)?;
        let provider_token_cipher =
            Aes256GcmTokenCipher::from_lookup(lookup, apple_token.is_some())?;

        Ok(Self {
            apple,
            google,
            apple_token,
            provider_token_cipher,
            web_apple,
            web_google,
            web_redirect_base_url,
            web_base_url,
            web_origins,
            csrf_key,
            smtp,
            mail_from: lookup("MAIL_FROM")
                .filter(|v| !v.trim().is_empty())
                .map_or_else(
                    || "GRISTORA <no-reply@gristora.local>".to_owned(),
                    |v| v.trim().to_owned(),
                ),
            jwks_min_refresh_secs,
            webauthn_rp_id,
            webauthn_rp_name: lookup("WEBAUTHN_RP_NAME")
                .filter(|v| !v.trim().is_empty())
                .map_or_else(|| "GRISTORA".to_owned(), |v| v.trim().to_owned()),
            webauthn_origins,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    #[test]
    fn apple_token_settings_debug_redacts_private_key() {
        let settings = AppleTokenSettings {
            team_id: "TEAM123".to_owned(),
            key_id: "KEY456".to_owned(),
            private_key: b"private-key-material".to_vec(),
            client_ids: vec!["com.example.ios".to_owned()],
            token_url: "https://appleid.apple.com/auth/token".to_owned(),
            revoke_url: "https://appleid.apple.com/auth/revoke".to_owned(),
        };
        let debug = format!("{settings:?}");
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("private-key-material"));
    }

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
    fn web_flow_needs_a_listed_client_id_and_google_needs_its_secret() {
        let base = |extra: &'static [(&'static str, &'static str)]| {
            move |n: &str| {
                extra
                    .iter()
                    .find(|(k, _)| *k == n)
                    .map(|(_, v)| (*v).to_owned())
                    .or_else(|| match n {
                        "GOOGLE_CLIENT_IDS" => Some("g-web".to_owned()),
                        "CSRF_HMAC_KEY" => Some("test-csrf-key-0123456789".to_owned()),
                        _ => None,
                    })
            }
        };
        let off = AuthConfig::from_lookup(&base(&[])).unwrap();
        assert!(off.web_google.is_none() && off.web_apple.is_none());
        // The client id is set but the secret is missing: the flow stays off.
        let no_secret = AuthConfig::from_lookup(&base(&[("GOOGLE_WEB_CLIENT_ID", "g-web")]));
        assert!(no_secret.unwrap().web_google.is_none());
        let on = AuthConfig::from_lookup(&base(&[
            ("GOOGLE_WEB_CLIENT_ID", "g-web"),
            ("GOOGLE_CLIENT_SECRET", "s3cret"),
        ]))
        .unwrap();
        let g = on.web_google.unwrap();
        assert_eq!(g.token_url, "https://oauth2.googleapis.com/token");
        assert!(!format!("{g:?}").contains("s3cret"));
        assert_eq!(on.web_redirect_base_url, "http://localhost:3000");
        // A client id outside the audience list is a configuration error.
        assert!(
            AuthConfig::from_lookup(&base(&[
                ("GOOGLE_WEB_CLIENT_ID", "other"),
                ("GOOGLE_CLIENT_SECRET", "s3cret"),
            ]))
            .is_err()
        );
    }

    #[test]
    fn webauthn_defaults_come_from_the_web_address() {
        let lookup =
            |n: &str| (n == "WEB_BASE_URL").then(|| "https://app.example.com:8443/x".to_owned());
        let cfg = AuthConfig::from_lookup(&lookup).unwrap();
        assert_eq!(cfg.webauthn_rp_id, "app.example.com");
        assert_eq!(
            cfg.webauthn_origins,
            ["https://app.example.com:8443", "https://app.example.com"]
        );
    }

    #[test]
    fn short_csrf_key_is_rejected() {
        let lookup = |n: &str| (n == "CSRF_HMAC_KEY").then(|| "short".to_owned());
        assert!(AuthConfig::from_lookup(&lookup).is_err());
    }
    #[test]
    fn apple_token_settings_decode_private_key_and_require_cipher_key() {
        use ring::{
            rand::SystemRandom,
            signature::{self, EcdsaKeyPair},
        };

        let rng = SystemRandom::new();
        let pkcs8 = EcdsaKeyPair::generate_pkcs8(&signature::ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
            .unwrap();
        let pem = format!(
            "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----",
            STANDARD.encode(pkcs8.as_ref())
        );
        let cipher_key = STANDARD.encode([9_u8; 32]);
        let lookup = |name: &str| match name {
            "APPLE_CLIENT_IDS" => Some("com.example.ios,com.example.web".to_owned()),
            "APPLE_TEAM_ID" => Some("TEAM123".to_owned()),
            "APPLE_KEY_ID" => Some("KEY456".to_owned()),
            "APPLE_PRIVATE_KEY" => Some(pem.clone()),
            "PROVIDER_TOKEN_KEY" => Some(cipher_key.clone()),
            "WEB_BASE_URL" => Some("https://app.example.com".to_owned()),
            "CSRF_HMAC_KEY" => Some("test-csrf-key-0123456789".to_owned()),
            _ => None,
        };
        let config = AuthConfig::from_lookup(&lookup).unwrap();
        let apple = config.apple_token.unwrap();
        assert_eq!(apple.team_id, "TEAM123");
        assert_eq!(apple.key_id, "KEY456");
        assert_eq!(apple.client_ids, ["com.example.ios", "com.example.web"]);
        assert_eq!(apple.token_url, "https://appleid.apple.com/auth/token");
        assert_eq!(apple.revoke_url, "https://appleid.apple.com/auth/revoke");
        assert!(config.provider_token_cipher.is_some());
    }

    #[test]
    fn apple_token_settings_reject_partial_secrets_and_missing_cipher_key() {
        let partial = |name: &str| match name {
            "APPLE_CLIENT_IDS" => Some("com.example.ios".to_owned()),
            "APPLE_TEAM_ID" => Some("TEAM123".to_owned()),
            _ => None,
        };
        assert!(AuthConfig::from_lookup(&partial).is_err());

        let no_cipher = |name: &str| match name {
            "APPLE_CLIENT_IDS" => Some("com.example.ios".to_owned()),
            "APPLE_TEAM_ID" => Some("TEAM123".to_owned()),
            "APPLE_KEY_ID" => Some("KEY456".to_owned()),
            "APPLE_PRIVATE_KEY" => {
                Some("-----BEGIN PRIVATE KEY-----\nAA==\n-----END PRIVATE KEY-----".to_owned())
            }
            "CSRF_HMAC_KEY" => Some("test-csrf-key-0123456789".to_owned()),
            _ => None,
        };
        assert!(AuthConfig::from_lookup(&no_cipher).is_err());
    }
}
