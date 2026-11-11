//! Service configuration read from environment variables.

use crate::{Error, Result};

/// Environment variable holding the listen port.
pub const ENV_PORT: &str = "PORT";
/// Environment variable holding the service name.
pub const ENV_SERVICE_NAME: &str = "SERVICE_NAME";

/// Basic configuration every service needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceConfig {
    /// Logical service name, used in logs and health responses.
    pub service_name: String,
    /// TCP port to listen on.
    pub port: u16,
}

impl ServiceConfig {
    /// Reads `SERVICE_NAME` and `PORT` from the process environment,
    /// falling back to the given defaults.
    pub fn from_env(default_name: &str, default_port: u16) -> Result<Self> {
        Self::from_lookup(|name| std::env::var(name).ok(), default_name, default_port)
    }

    /// Like [`Self::from_env`] but with an injectable lookup function (testable without
    /// touching the process environment).
    pub fn from_lookup<F>(lookup: F, default_name: &str, default_port: u16) -> Result<Self>
    where
        F: Fn(&str) -> Option<String>,
    {
        let service_name = lookup(ENV_SERVICE_NAME)
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| default_name.to_owned());
        let port = match lookup(ENV_PORT).filter(|v| !v.trim().is_empty()) {
            Some(raw) => raw.trim().parse::<u16>().map_err(|e| Error::InvalidEnv {
                name: ENV_PORT,
                reason: e.to_string(),
            })?,
            None => default_port,
        };
        Ok(Self { service_name, port })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_apply_when_unset() {
        let cfg = ServiceConfig::from_lookup(|_| None, "api", 8080).unwrap();
        assert_eq!(cfg.service_name, "api");
        assert_eq!(cfg.port, 8080);
    }

    #[test]
    fn values_are_read_from_lookup() {
        let lookup = |name: &str| match name {
            ENV_PORT => Some("9090".to_owned()),
            ENV_SERVICE_NAME => Some("custom".to_owned()),
            _ => None,
        };
        let cfg = ServiceConfig::from_lookup(lookup, "api", 8080).unwrap();
        assert_eq!(cfg.service_name, "custom");
        assert_eq!(cfg.port, 9090);
    }

    #[test]
    fn invalid_port_is_an_error() {
        let lookup = |name: &str| (name == ENV_PORT).then(|| "not-a-port".to_owned());
        let err = ServiceConfig::from_lookup(lookup, "api", 8080).unwrap_err();
        assert!(matches!(err, Error::InvalidEnv { name: ENV_PORT, .. }));
    }
}
