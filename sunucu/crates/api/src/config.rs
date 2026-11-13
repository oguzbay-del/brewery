//! API configuration, read from the environment (secrets also from `*_FILE`).

use common::{Error, Result, ServiceConfig, config::secret_from_lookup};

use crate::app_version::{AppVersion, MinVersions};

/// JSON body limit (ADR-0008 rule 11.1).
pub const BODY_LIMIT_BYTES: usize = 1024 * 1024;
/// Most items in a bulk request (ADR-0008 rule 11.1).
pub const BULK_MAX_ITEMS: usize = 500;

/// Rate limit settings (ADR-0008 rule 11.2). Windows are one minute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateLimitConfig {
    /// Requests per minute per client IP.
    pub ip_per_minute: u32,
    /// Requests per minute per user.
    pub user_per_minute: u32,
    /// Requests per minute per workspace.
    pub workspace_per_minute: u32,
    /// Take the client IP from the first `X-Forwarded-For` entry (only behind a trusted
    /// proxy; otherwise the header is spoofable).
    pub trust_forwarded_for: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            ip_per_minute: 300,
            user_per_minute: 600,
            workspace_per_minute: 1200,
            trust_forwarded_for: false,
        }
    }
}

/// The API configuration.
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Service name and port.
    pub service: ServiceConfig,
    /// PostgreSQL URL (`app_api` role).
    pub database_url: String,
    /// Redis URL; `None` runs without Redis (rate limit and caches off).
    pub redis_url: Option<String>,
    /// Lowest supported app versions.
    pub min_versions: MinVersions,
    /// Rate limits.
    pub rate: RateLimitConfig,
    /// Object store (`S3_*`); `None` turns the file endpoints off (`503`).
    pub storage: Option<storage::S3Config>,
    /// Sign-in settings (T2.11).
    pub auth: crate::auth::AuthConfig,
}

fn parse_u32(
    lookup: &dyn Fn(&str) -> Option<String>,
    name: &'static str,
    default: u32,
) -> Result<u32> {
    match lookup(name).filter(|v| !v.trim().is_empty()) {
        Some(raw) => raw
            .trim()
            .parse()
            .map_err(|e: std::num::ParseIntError| Error::InvalidEnv {
                name,
                reason: e.to_string(),
            }),
        None => Ok(default),
    }
}

fn parse_version(
    lookup: &dyn Fn(&str) -> Option<String>,
    name: &'static str,
) -> Result<Option<AppVersion>> {
    lookup(name)
        .filter(|v| !v.trim().is_empty())
        .map(|raw| {
            AppVersion::parse(raw.trim()).ok_or_else(|| Error::InvalidEnv {
                name,
                reason: "not a dotted version".to_owned(),
            })
        })
        .transpose()
}

impl ApiConfig {
    /// Reads the configuration from the process environment.
    pub fn from_env() -> Result<Self> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// Like [`Self::from_env`] with an injectable lookup function.
    pub fn from_lookup<F>(lookup: F) -> Result<Self>
    where
        F: Fn(&str) -> Option<String>,
    {
        let service =
            ServiceConfig::from_lookup(&lookup, crate::SERVICE_NAME, crate::DEFAULT_PORT)?;
        let database_url =
            db::database_url_from_lookup(&lookup).map_err(|e| Error::InvalidEnv {
                name: "DATABASE_URL",
                reason: e.to_string(),
            })?;
        let redis_url = secret_from_lookup(&lookup, "REDIS_URL", "REDIS_URL_FILE")?;
        let defaults = RateLimitConfig::default();
        let rate = RateLimitConfig {
            ip_per_minute: parse_u32(&lookup, "RATE_LIMIT_IP_PER_MINUTE", defaults.ip_per_minute)?,
            user_per_minute: parse_u32(
                &lookup,
                "RATE_LIMIT_USER_PER_MINUTE",
                defaults.user_per_minute,
            )?,
            workspace_per_minute: parse_u32(
                &lookup,
                "RATE_LIMIT_WORKSPACE_PER_MINUTE",
                defaults.workspace_per_minute,
            )?,
            trust_forwarded_for: lookup("TRUST_FORWARDED_FOR")
                .is_some_and(|v| matches!(v.trim(), "1" | "true")),
        };
        let min_versions = MinVersions {
            ios: parse_version(&lookup, "MIN_APP_VERSION_IOS")?,
            web: parse_version(&lookup, "MIN_APP_VERSION_WEB")?,
        };
        let storage = storage::S3Config::from_lookup(&lookup).map_err(|e| Error::InvalidEnv {
            name: "S3_ENDPOINT",
            reason: e.to_string(),
        })?;
        Ok(Self {
            service,
            database_url,
            redis_url,
            min_versions,
            rate,
            storage,
            auth: crate::auth::AuthConfig::from_lookup(&lookup)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_overrides() {
        let lookup = |n: &str| match n {
            "DATABASE_URL" => Some("postgres://x/y".to_owned()),
            "MIN_APP_VERSION_IOS" => Some("1.4.0".to_owned()),
            "RATE_LIMIT_IP_PER_MINUTE" => Some("7".to_owned()),
            _ => None,
        };
        let cfg = ApiConfig::from_lookup(lookup).unwrap();
        assert_eq!(cfg.service.port, 8080);
        assert_eq!(cfg.redis_url, None);
        assert_eq!(cfg.rate.ip_per_minute, 7);
        assert_eq!(cfg.rate.user_per_minute, 600);
        assert!(cfg.min_versions.ios.is_some() && cfg.min_versions.web.is_none());
    }

    #[test]
    fn missing_database_url_is_an_error() {
        assert!(ApiConfig::from_lookup(|_| None).is_err());
    }
}
