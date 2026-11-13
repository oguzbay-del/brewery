use std::time::Duration;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use reqwest::{Method, StatusCode, Url};
use sha2::{Digest, Sha256};

use crate::{
    BoxFuture, ObjectMeta, ObjectStore, PresignedRequest, S3Config, StorageError, StorageResult,
    UploadLimits,
    sigv4::{self, Canonical, EMPTY_SHA256, UNSIGNED_PAYLOAD},
};

/// Longest key accepted (S3 limit).
const MAX_KEY_BYTES: usize = 1024;

/// S3 and S3 compatible stores (GCS interoperability, `SeaweedFS`, ...).
#[derive(Debug, Clone)]
pub struct S3Store {
    config: S3Config,
    client: reqwest::Client,
}

/// An endpoint split into what URLs and signatures need.
struct Target {
    /// `scheme://host[:port]` as clients connect.
    origin: String,
    /// Value of the signed `host` header.
    host: String,
    /// Encoded path.
    path: String,
}

impl S3Store {
    /// Builds the store.
    ///
    /// # Errors
    /// [`StorageError::Config`] when an endpoint is not a URL or the HTTP client cannot start.
    pub fn new(config: S3Config) -> StorageResult<Self> {
        for endpoint in [&config.endpoint, &config.public_endpoint] {
            Url::parse(endpoint)
                .ok()
                .filter(|u| matches!(u.scheme(), "http" | "https") && u.host_str().is_some())
                .ok_or_else(|| StorageError::Config(format!("bad S3 endpoint: {endpoint}")))?;
        }
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| StorageError::Config(e.to_string()))?;
        Ok(Self { config, client })
    }

    /// The settings.
    #[must_use]
    pub const fn config(&self) -> &S3Config {
        &self.config
    }

    fn target(&self, endpoint: &str, key: Option<&str>) -> StorageResult<Target> {
        let url = Url::parse(endpoint)
            .map_err(|e| StorageError::Config(format!("bad S3 endpoint: {e}")))?;
        let hostname = url
            .host_str()
            .ok_or_else(|| StorageError::Config("S3 endpoint has no host".to_owned()))?;
        let authority = url
            .port()
            .map_or_else(|| hostname.to_owned(), |p| format!("{hostname}:{p}"));
        let bucket = &self.config.bucket;
        let (host, mut path) = if self.config.path_style {
            (authority, format!("/{}", sigv4::encode(bucket, false)))
        } else {
            (format!("{bucket}.{authority}"), String::new())
        };
        match key {
            Some(key) => {
                path.push('/');
                path.push_str(&sigv4::encode(key, true));
            }
            None if path.is_empty() => path.push('/'),
            None => {}
        }
        let origin = format!("{}://{host}", url.scheme());
        Ok(Target { origin, host, path })
    }

    fn check_key(key: &str) -> StorageResult<()> {
        if key.is_empty() || key.len() > MAX_KEY_BYTES || key.starts_with('/') {
            return Err(StorageError::Config(format!("invalid object key: {key:?}")));
        }
        Ok(())
    }

    fn presign(
        &self,
        method: &str,
        key: &str,
        extra_headers: &[(String, String)],
        ttl: Duration,
        now: DateTime<Utc>,
    ) -> StorageResult<PresignedRequest> {
        Self::check_key(key)?;
        let target = self.target(&self.config.public_endpoint, Some(key))?;
        let expires = ttl.as_secs().clamp(1, crate::config::MAX_PRESIGN_TTL_SECS);
        let mut headers = vec![("host".to_owned(), target.host.clone())];
        headers.extend(extra_headers.iter().cloned());
        let signed = Canonical {
            method,
            path: &target.path,
            query: &[],
            headers: &headers,
            payload_hash: UNSIGNED_PAYLOAD,
        }
        .signed_headers();
        let mut query = sigv4::presign_params(
            &self.config.access_key,
            &self.config.region,
            now,
            expires,
            &signed,
        );
        let canonical = Canonical {
            method,
            path: &target.path,
            query: &query,
            headers: &headers,
            payload_hash: UNSIGNED_PAYLOAD,
        };
        let signature = canonical.signature(&self.config.secret_key, &self.config.region, now);
        let query_string = canonical.query_string();
        query.clear();
        let secs = i64::try_from(expires).unwrap_or(i64::MAX);
        Ok(PresignedRequest {
            method: method.to_owned(),
            url: format!(
                "{}{}?{query_string}&X-Amz-Signature={signature}",
                target.origin, target.path
            ),
            headers: extra_headers.to_vec(),
            expires_at: now + chrono::Duration::seconds(secs),
        })
    }

    /// Like [`ObjectStore::presign_put`] with an explicit clock (tests of expiry).
    ///
    /// # Errors
    /// [`StorageError::Config`] for an invalid key or endpoint.
    pub fn presign_put_at(
        &self,
        key: &str,
        limits: &UploadLimits,
        ttl: Duration,
        now: DateTime<Utc>,
    ) -> StorageResult<PresignedRequest> {
        let headers = [
            ("content-length".to_owned(), limits.size_bytes.to_string()),
            ("content-type".to_owned(), limits.content_type.clone()),
            (
                "x-amz-checksum-sha256".to_owned(),
                STANDARD.encode(limits.sha256),
            ),
        ];
        let mut request = self.presign("PUT", key, &headers, ttl, now)?;
        // Clients set `Content-Length` themselves; do not hand it out as a header to send.
        request.headers.retain(|(k, _)| k != "content-length");
        Ok(request)
    }

    /// Like [`ObjectStore::presign_get`] with an explicit clock.
    ///
    /// # Errors
    /// [`StorageError::Config`] for an invalid key or endpoint.
    pub fn presign_get_at(
        &self,
        key: &str,
        ttl: Duration,
        now: DateTime<Utc>,
    ) -> StorageResult<PresignedRequest> {
        self.presign("GET", key, &[], ttl, now)
    }

    /// Sends a header-signed request to the server side endpoint.
    async fn send(
        &self,
        method: Method,
        key: Option<&str>,
        extra_headers: &[(&str, String)],
        body: Vec<u8>,
    ) -> StorageResult<reqwest::Response> {
        if let Some(key) = key {
            Self::check_key(key)?;
        }
        let target = self.target(&self.config.endpoint, key)?;
        let now = Utc::now();
        let payload_hash = if body.is_empty() {
            EMPTY_SHA256.to_owned()
        } else {
            sigv4::sha256_hex(&body)
        };
        let mut headers = vec![
            ("host".to_owned(), target.host.clone()),
            ("x-amz-content-sha256".to_owned(), payload_hash.clone()),
            ("x-amz-date".to_owned(), sigv4::amz_date(now)),
        ];
        headers.extend(
            extra_headers
                .iter()
                .map(|(k, v)| ((*k).to_owned(), v.clone())),
        );
        let authorization = sigv4::authorization(
            &self.config.access_key,
            &self.config.secret_key,
            &self.config.region,
            now,
            &Canonical {
                method: method.as_str(),
                path: &target.path,
                query: &[],
                headers: &headers,
                payload_hash: &payload_hash,
            },
        );
        let mut request = self
            .client
            .request(method, format!("{}{}", target.origin, target.path))
            .header("authorization", authorization);
        for (name, value) in headers.iter().filter(|(k, _)| k != "host") {
            request = request.header(name.as_str(), value.as_str());
        }
        request
            .body(body)
            .send()
            .await
            .map_err(|e| StorageError::Unavailable(e.without_url().to_string()))
    }

    async fn reject(response: reqwest::Response) -> StorageError {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        let body: String = body.chars().take(300).collect();
        if status.is_server_error() {
            StorageError::Unavailable(format!("{status}: {body}"))
        } else {
            StorageError::Rejected {
                status: status.as_u16(),
                body,
            }
        }
    }
}

fn header_text(response: &reqwest::Response, name: &str) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
}

impl ObjectStore for S3Store {
    fn presign_put(
        &self,
        key: &str,
        limits: &UploadLimits,
        ttl: Duration,
    ) -> StorageResult<PresignedRequest> {
        self.presign_put_at(key, limits, ttl, Utc::now())
    }

    fn presign_get(&self, key: &str, ttl: Duration) -> StorageResult<PresignedRequest> {
        self.presign_get_at(key, ttl, Utc::now())
    }

    fn head<'a>(&'a self, key: &'a str) -> BoxFuture<'a, StorageResult<Option<ObjectMeta>>> {
        Box::pin(async move {
            let response = self
                .send(
                    Method::HEAD,
                    Some(key),
                    &[("x-amz-checksum-mode", "ENABLED".to_owned())],
                    Vec::new(),
                )
                .await?;
            match response.status() {
                StatusCode::NOT_FOUND => Ok(None),
                s if s.is_success() => {
                    let size_bytes = header_text(&response, "content-length")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(0);
                    let sha256 = header_text(&response, "x-amz-checksum-sha256")
                        .and_then(|v| STANDARD.decode(v).ok())
                        .and_then(|v| <[u8; 32]>::try_from(v).ok());
                    Ok(Some(ObjectMeta {
                        size_bytes,
                        content_type: header_text(&response, "content-type"),
                        sha256,
                    }))
                }
                _ => Err(Self::reject(response).await),
            }
        })
    }

    fn digest<'a>(
        &'a self,
        key: &'a str,
        max_bytes: u64,
    ) -> BoxFuture<'a, StorageResult<Option<crate::Digest>>> {
        Box::pin(async move {
            let mut response = self.send(Method::GET, Some(key), &[], Vec::new()).await?;
            if response.status() == StatusCode::NOT_FOUND {
                return Ok(None);
            }
            if !response.status().is_success() {
                return Err(Self::reject(response).await);
            }
            let mut hasher = Sha256::new();
            let mut total: u64 = 0;
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|e| StorageError::Unavailable(e.without_url().to_string()))?
            {
                total += chunk.len() as u64;
                if total > max_bytes {
                    return Err(StorageError::TooLarge(max_bytes));
                }
                hasher.update(&chunk);
            }
            Ok(Some((total, hasher.finalize().into())))
        })
    }

    fn put<'a>(
        &'a self,
        key: &'a str,
        content_type: &'a str,
        body: Vec<u8>,
    ) -> BoxFuture<'a, StorageResult<()>> {
        Box::pin(async move {
            let response = self
                .send(
                    Method::PUT,
                    Some(key),
                    &[("content-type", content_type.to_owned())],
                    body,
                )
                .await?;
            if response.status().is_success() {
                Ok(())
            } else {
                Err(Self::reject(response).await)
            }
        })
    }

    fn delete<'a>(&'a self, key: &'a str) -> BoxFuture<'a, StorageResult<()>> {
        Box::pin(async move {
            let response = self
                .send(Method::DELETE, Some(key), &[], Vec::new())
                .await?;
            if response.status().is_success() || response.status() == StatusCode::NOT_FOUND {
                Ok(())
            } else {
                Err(Self::reject(response).await)
            }
        })
    }

    fn ensure_bucket(&self) -> BoxFuture<'_, StorageResult<()>> {
        Box::pin(async move {
            let response = self.send(Method::PUT, None, &[], Vec::new()).await?;
            let status = response.status();
            if status.is_success() || status == StatusCode::CONFLICT {
                // 409 BucketAlreadyOwnedByYou: nothing to do.
                return Ok(());
            }
            Err(Self::reject(response).await)
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;

    use super::*;

    fn store(path_style: bool) -> S3Store {
        S3Store::new(S3Config {
            endpoint: "http://objectstore:8333".to_owned(),
            public_endpoint: "http://127.0.0.1:8333".to_owned(),
            bucket: "files".to_owned(),
            region: "us-east-1".to_owned(),
            access_key: "AK".to_owned(),
            secret_key: "SK".to_owned(),
            path_style,
            create_bucket: false,
            presign_ttl: Duration::from_mins(10),
        })
        .unwrap()
    }

    fn limits() -> UploadLimits {
        UploadLimits {
            content_type: "image/png".to_owned(),
            size_bytes: 12,
            sha256: [7; 32],
        }
    }

    #[test]
    fn put_url_is_bound_to_the_limits_and_the_public_host() {
        let now = Utc.with_ymd_and_hms(2026, 11, 13, 10, 0, 0).unwrap();
        let r = store(true)
            .presign_put_at("ws/a b", &limits(), Duration::from_mins(1), now)
            .unwrap();
        assert!(
            r.url
                .starts_with("http://127.0.0.1:8333/files/ws/a%20b?X-Amz-Algorithm=")
        );
        assert!(r.url.contains(
            "X-Amz-SignedHeaders=content-length%3Bcontent-type%3Bhost%3Bx-amz-checksum-sha256"
        ));
        assert!(r.url.contains("X-Amz-Expires=60"));
        assert_eq!(r.expires_at, now + chrono::Duration::seconds(60));
        assert_eq!(r.method, "PUT");
        assert!(
            r.headers
                .iter()
                .any(|(k, v)| k == "content-type" && v == "image/png")
        );
        assert!(r.headers.iter().all(|(k, _)| k != "content-length"));
        // A different size gives a different signature.
        let mut other = limits();
        other.size_bytes = 13;
        let r2 = store(true)
            .presign_put_at("ws/a b", &other, Duration::from_mins(1), now)
            .unwrap();
        assert_ne!(r.url, r2.url);
    }

    #[test]
    fn virtual_hosted_style_puts_the_bucket_in_the_host() {
        let now = Utc.with_ymd_and_hms(2026, 11, 13, 10, 0, 0).unwrap();
        let r = store(false)
            .presign_get_at("k", Duration::from_mins(1), now)
            .unwrap();
        assert!(r.url.starts_with("http://files.127.0.0.1:8333/k?"));
    }

    #[test]
    fn bad_keys_and_endpoints_are_rejected() {
        let now = Utc::now();
        assert!(
            store(true)
                .presign_get_at("", Duration::from_secs(1), now)
                .is_err()
        );
        assert!(
            store(true)
                .presign_get_at("/x", Duration::from_secs(1), now)
                .is_err()
        );
        let mut cfg = store(true).config().clone();
        cfg.endpoint = "not a url".to_owned();
        assert!(S3Store::new(cfg).is_err());
    }
}
