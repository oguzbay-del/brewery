use std::time::Duration;

use common::config::secret_from_lookup;

use crate::{StorageError, StorageResult};

/// Default lifetime of presigned requests.
pub const DEFAULT_PRESIGN_TTL_SECS: u64 = 600;
/// Longest accepted lifetime (S3 allows seven days; links here stay short).
pub const MAX_PRESIGN_TTL_SECS: u64 = 3600;

/// S3 settings, read from the environment (`S3_*`; keys also from `*_FILE`).
#[derive(Clone, PartialEq, Eq)]
pub struct S3Config {
    /// Address the server uses (HEAD, DELETE, bucket creation).
    pub endpoint: String,
    /// Address clients use; presigned URLs are signed for it. Defaults to `endpoint`.
    pub public_endpoint: String,
    /// Bucket name.
    pub bucket: String,
    /// Signing region (`auto` or any value for stores that ignore it).
    pub region: String,
    /// Access key (for GCS: an HMAC key).
    pub access_key: String,
    /// Secret key.
    pub secret_key: String,
    /// `endpoint/bucket/key` (true) or `bucket.endpoint/key` (false).
    pub path_style: bool,
    /// Create the bucket at startup when missing (local stack only).
    pub create_bucket: bool,
    /// Lifetime of presigned requests.
    pub presign_ttl: Duration,
}

impl std::fmt::Debug for S3Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("S3Config")
            .field("endpoint", &self.endpoint)
            .field("public_endpoint", &self.public_endpoint)
            .field("bucket", &self.bucket)
            .field("region", &self.region)
            .field("path_style", &self.path_style)
            .field("create_bucket", &self.create_bucket)
            .field("presign_ttl", &self.presign_ttl)
            .finish_non_exhaustive()
    }
}

fn flag(lookup: &dyn Fn(&str) -> Option<String>, name: &str, default: bool) -> bool {
    lookup(name).map_or(default, |v| matches!(v.trim(), "1" | "true"))
}

impl S3Config {
    /// Reads the settings from the process environment.
    ///
    /// # Errors
    /// See [`Self::from_lookup`].
    pub fn from_env() -> StorageResult<Option<Self>> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// Like [`Self::from_env`] with an injectable lookup. `Ok(None)` when `S3_ENDPOINT` is not
    /// set (file endpoints are then unavailable).
    ///
    /// # Errors
    /// [`StorageError::Config`] for a missing bucket or key, or a bad number.
    pub fn from_lookup<F>(lookup: F) -> StorageResult<Option<Self>>
    where
        F: Fn(&str) -> Option<String>,
    {
        let text = |name: &str| {
            lookup(name)
                .map(|v| v.trim().to_owned())
                .filter(|v| !v.is_empty())
        };
        let Some(endpoint) = text("S3_ENDPOINT") else {
            return Ok(None);
        };
        let secret = |name: &'static str, file: &'static str| -> StorageResult<String> {
            secret_from_lookup(&lookup, name, file)
                .map_err(|e| StorageError::Config(e.to_string()))?
                .ok_or_else(|| StorageError::Config(format!("{name} is not set")))
        };
        let ttl = match text("S3_PRESIGN_TTL_SECS") {
            Some(raw) => raw
                .parse::<u64>()
                .ok()
                .filter(|s| (1..=MAX_PRESIGN_TTL_SECS).contains(s))
                .ok_or_else(|| {
                    StorageError::Config(format!(
                        "S3_PRESIGN_TTL_SECS must be 1..={MAX_PRESIGN_TTL_SECS}"
                    ))
                })?,
            None => DEFAULT_PRESIGN_TTL_SECS,
        };
        Ok(Some(Self {
            public_endpoint: text("S3_PUBLIC_ENDPOINT").unwrap_or_else(|| endpoint.clone()),
            endpoint,
            bucket: text("S3_BUCKET")
                .ok_or_else(|| StorageError::Config("S3_BUCKET is not set".to_owned()))?,
            region: text("S3_REGION").unwrap_or_else(|| "us-east-1".to_owned()),
            access_key: secret("S3_ACCESS_KEY", "S3_ACCESS_KEY_FILE")?,
            secret_key: secret("S3_SECRET_KEY", "S3_SECRET_KEY_FILE")?,
            path_style: flag(&lookup, "S3_PATH_STYLE", true),
            create_bucket: flag(&lookup, "S3_CREATE_BUCKET", false),
            presign_ttl: Duration::from_secs(ttl),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup<'a>(extra: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |n| {
            let base = [
                ("S3_ENDPOINT", "http://s3:8333"),
                ("S3_BUCKET", "b"),
                ("S3_ACCESS_KEY", "k"),
                ("S3_SECRET_KEY", "s"),
            ];
            extra
                .iter()
                .chain(base.iter())
                .find(|(k, _)| *k == n)
                .map(|(_, v)| (*v).to_owned())
        }
    }

    #[test]
    fn unset_endpoint_disables_storage() {
        assert!(S3Config::from_lookup(|_| None).unwrap().is_none());
    }

    #[test]
    fn defaults_and_overrides() {
        let c = S3Config::from_lookup(lookup(&[])).unwrap().unwrap();
        assert_eq!(c.public_endpoint, "http://s3:8333");
        assert_eq!(c.region, "us-east-1");
        assert!(c.path_style && !c.create_bucket);
        assert_eq!(c.presign_ttl, Duration::from_mins(10));
        let c = S3Config::from_lookup(lookup(&[
            ("S3_PUBLIC_ENDPOINT", "http://127.0.0.1:1"),
            ("S3_PRESIGN_TTL_SECS", "5"),
            ("S3_CREATE_BUCKET", "true"),
        ]))
        .unwrap()
        .unwrap();
        assert_eq!(c.public_endpoint, "http://127.0.0.1:1");
        assert_eq!(c.presign_ttl, Duration::from_secs(5));
        assert!(c.create_bucket);
    }

    #[test]
    fn bad_values_are_errors() {
        assert!(S3Config::from_lookup(lookup(&[("S3_PRESIGN_TTL_SECS", "0")])).is_err());
        assert!(S3Config::from_lookup(lookup(&[("S3_PRESIGN_TTL_SECS", "99999")])).is_err());
        let no_key = |n: &str| (n != "S3_SECRET_KEY").then(|| lookup(&[])(n)).flatten();
        assert!(S3Config::from_lookup(no_key).is_err());
    }

    #[test]
    fn debug_hides_secrets() {
        let c = S3Config::from_lookup(lookup(&[])).unwrap().unwrap();
        assert!(!format!("{c:?}").contains("secret_key"));
    }
}
