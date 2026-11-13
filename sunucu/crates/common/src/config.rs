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

/// Reads a setting that may be a secret (ADR-0009 rule 11.1): from variable `name` or, if
/// unset, from the file named by `<name>_FILE` (mounted secret). Both set is an error;
/// neither gives `Ok(None)`. `file_name` must be `<name>_FILE`.
pub fn secret_from_lookup<F>(
    lookup: F,
    name: &'static str,
    file_name: &'static str,
) -> Result<Option<String>>
where
    F: Fn(&str) -> Option<String>,
{
    let direct = lookup(name).filter(|v| !v.trim().is_empty());
    let file = lookup(file_name).filter(|v| !v.trim().is_empty());
    match (direct, file) {
        (Some(_), Some(_)) => Err(Error::InvalidEnv {
            name,
            reason: format!("{name} and {file_name} are both set; set exactly one"),
        }),
        (Some(value), None) => Ok(Some(value.trim().to_owned())),
        (None, Some(path)) => {
            let content = std::fs::read_to_string(&path).map_err(|e| Error::InvalidEnv {
                name: file_name,
                reason: format!("cannot read {path}: {e}"),
            })?;
            let value = content.trim();
            if value.is_empty() {
                return Err(Error::InvalidEnv {
                    name: file_name,
                    reason: format!("{path} is empty"),
                });
            }
            Ok(Some(value.to_owned()))
        }
        (None, None) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_is_read_from_variable_file_or_nothing() {
        let var = |n: &str| (n == "X").then(|| " v \n".to_owned());
        assert_eq!(
            secret_from_lookup(var, "X", "X_FILE").unwrap().as_deref(),
            Some("v")
        );
        assert_eq!(secret_from_lookup(|_| None, "X", "X_FILE").unwrap(), None);
        let both = |n: &str| Some(n.to_owned());
        assert!(secret_from_lookup(both, "X", "X_FILE").is_err());
        let missing = |n: &str| (n == "X_FILE").then(|| "/nonexistent/secret".to_owned());
        assert!(secret_from_lookup(missing, "X", "X_FILE").is_err());
    }

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
