//! OAuth clients: Client ID Metadata Documents (CIMD) and dynamic registration (DCR)
//! (ADR-0015 rule 2.3).
//!
//! The CIMD fetch is the one place where the server calls an address that a stranger names.
//! Rules: HTTPS only, port 443 only, no private or local address (checked after DNS, and the
//! connection is pinned to the checked address), no redirect, 5 KiB body limit, 5 second
//! timeout, cache of at most 24 hours.

use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use url::Url;
use uuid::Uuid;

use crate::{AppState, auth::tokens::token_hash};

/// Largest CIMD document.
const MAX_DOCUMENT_BYTES: usize = 5 * 1024;
/// Fetch timeout.
const FETCH_TIMEOUT: Duration = Duration::from_secs(5);
/// Longest cache time of a document (seconds).
const MAX_CACHE_SECS: i64 = 24 * 3600;
/// Cache time when the answer states none.
const DEFAULT_CACHE_SECS: i64 = 3600;

/// A registered client.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Client {
    /// Row id.
    pub id: Uuid,
    /// The `client_id` text.
    pub client_id: String,
    /// `cimd`, `dcr` or `preregistered`.
    pub kind: String,
    /// Display name.
    pub client_name: String,
    /// Logo address.
    pub logo_uri: Option<String>,
    /// Home page of the client.
    pub client_uri: Option<String>,
    /// Exact redirect addresses.
    pub redirect_uris: Vec<String>,
    /// `none` (the only method supported now).
    pub token_endpoint_auth_method: String,
    /// End of the document cache.
    pub metadata_expires_at: Option<DateTime<Utc>>,
}

const COLUMNS: &str = "id, client_id, kind, client_name, logo_uri, client_uri, redirect_uris, \
                       token_endpoint_auth_method, metadata_expires_at";

/// Why a client cannot be used.
#[derive(Debug)]
pub enum ClientError {
    /// Unknown client, or an invalid document.
    Invalid(String),
    /// The document cannot be fetched now (rate limit or network).
    Unavailable(String),
    /// Database failure.
    Db(sqlx::Error),
}

impl From<sqlx::Error> for ClientError {
    fn from(e: sqlx::Error) -> Self {
        Self::Db(e)
    }
}

/// A client by row id.
///
/// # Errors
/// Database errors.
pub async fn by_pk(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Client>> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM oauth_clients WHERE id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Finds the client of a `client_id`: a stored client, or a CIMD document (from the cache or
/// fetched).
///
/// # Errors
/// [`ClientError`].
pub async fn resolve(state: &AppState, client_id: &str, ip: &str) -> Result<Client, ClientError> {
    if client_id.is_empty() || client_id.len() > 2048 {
        return Err(ClientError::Invalid("client_id".into()));
    }
    let stored: Option<Client> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM oauth_clients WHERE client_id = $1"
    )))
    .bind(client_id)
    .fetch_optional(&state.db)
    .await?;
    if let Some(client) = &stored
        && (client.kind != "cimd" || client.metadata_expires_at.is_some_and(|t| t > Utc::now()))
    {
        return Ok(client.clone());
    }
    if !client_id.starts_with("https://") && !state.config.oauth.allow_insecure_cimd {
        // Not a CIMD address and not stored: unknown.
        return stored.ok_or_else(|| ClientError::Invalid("unknown client".into()));
    }
    if !client_id.starts_with("http") {
        return stored.ok_or_else(|| ClientError::Invalid("unknown client".into()));
    }
    fetch_cimd(state, client_id, ip).await
}

fn private(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_multicast()
                || v4.is_documentation()
                // 0.0.0.0/8, 100.64.0.0/10 (carrier-grade NAT), 192.0.0.0/24,
                // 198.18.0.0/15 (benchmarking) and 240.0.0.0/4 (reserved)
                || o[0] == 0
                || (o[0] == 100 && (o[1] & 0xc0) == 64)
                || (o[0] == 192 && o[1] == 0 && o[2] == 0)
                || (o[0] == 198 && (o[1] & 0xfe) == 18)
                || o[0] >= 240
        }
        IpAddr::V6(v6) => {
            let s = v6.segments();
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                // fc00::/7 unique local, fe80::/10 link local, 2002::/16 (6to4) and
                // 64:ff9b::/96 (NAT64) can carry any IPv4 address
                || s[0] == 0x2002
                || (s[0] == 0x64 && s[1] == 0xff9b && s[2..6].iter().all(|x| *x == 0))
                || (s[0] & 0xfe00) == 0xfc00
                || (s[0] & 0xffc0) == 0xfe80
                || v6.to_ipv4_mapped().is_some_and(|v4| private(IpAddr::V4(v4)))
        }
    }
}

/// Whether `ip` is an address that a CIMD fetch must not call. Exposed for tests.
#[must_use]
pub fn is_private_address(ip: IpAddr) -> bool {
    private(ip)
}

fn max_age(cache_control: Option<&str>) -> i64 {
    let Some(value) = cache_control else {
        return DEFAULT_CACHE_SECS;
    };
    let mut ttl = DEFAULT_CACHE_SECS;
    for part in value.split(',').map(|p| p.trim().to_ascii_lowercase()) {
        if part == "no-store" || part == "no-cache" {
            return 60;
        }
        if let Some(n) = part
            .strip_prefix("max-age=")
            .and_then(|n| n.parse::<i64>().ok())
        {
            ttl = n;
        }
    }
    ttl.clamp(60, MAX_CACHE_SECS)
}

async fn fetch_cimd(state: &AppState, client_id: &str, ip: &str) -> Result<Client, ClientError> {
    let insecure = state.config.oauth.allow_insecure_cimd;
    let url = Url::parse(client_id).map_err(|_| ClientError::Invalid("client_id".into()))?;
    let https_ok = url.scheme() == "https" && url.port().is_none_or(|p| p == 443);
    if !(https_ok || insecure && matches!(url.scheme(), "http" | "https")) {
        return Err(ClientError::Invalid(
            "client_id must be an HTTPS address on port 443".into(),
        ));
    }
    if url.path().len() < 2 || url.fragment().is_some() || !url.username().is_empty() {
        return Err(ClientError::Invalid("client_id needs a path".into()));
    }
    let host = url
        .host_str()
        .ok_or_else(|| ClientError::Invalid("client_id".into()))?
        .to_owned();
    let port = url.port_or_known_default().unwrap_or(443);
    let addrs: Vec<SocketAddr> = tokio::time::timeout(
        FETCH_TIMEOUT,
        tokio::net::lookup_host((host.trim_matches(['[', ']']), port)),
    )
    .await
    .map_err(|_| ClientError::Unavailable("dns timeout".into()))?
    .map_err(|_| ClientError::Unavailable("dns".into()))?
    .collect();
    if addrs.is_empty() || (!insecure && addrs.iter().any(|a| private(a.ip()))) {
        return Err(ClientError::Invalid(
            "client_id address is not allowed".into(),
        ));
    }
    // At most 30 fetches an hour for each caller address (outside the cache).
    if state
        .redis
        .incr_expire(format!("oauth:cimd-ip:{ip}"), 3600)
        .await
        .is_some_and(|n| n > 30)
    {
        return Err(ClientError::Unavailable("rate limited".into()));
    }
    // One fetch per client address and 5 minutes (outside the cache).
    let key = format!("oauth:cimd:{}", hex(&token_hash(client_id)));
    if state
        .redis
        .incr_expire(key, 300)
        .await
        .is_some_and(|n| n > 1)
    {
        return Err(ClientError::Unavailable("rate limited".into()));
    }
    let http = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(FETCH_TIMEOUT)
        .resolve(&host, addrs[0])
        .build()
        .map_err(|_| ClientError::Unavailable("client".into()))?;
    let mut response = http
        .get(url.clone())
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|_| ClientError::Unavailable("fetch failed".into()))?;
    if response.status() != reqwest::StatusCode::OK {
        return Err(ClientError::Invalid("client document not found".into()));
    }
    let ttl = max_age(
        response
            .headers()
            .get("cache-control")
            .and_then(|v| v.to_str().ok()),
    );
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| ClientError::Unavailable("fetch failed".into()))?
    {
        body.extend_from_slice(&chunk);
        if body.len() > MAX_DOCUMENT_BYTES {
            return Err(ClientError::Invalid("client document too large".into()));
        }
    }
    let doc: Value = serde_json::from_slice(&body)
        .map_err(|_| ClientError::Invalid("client document is not JSON".into()))?;
    if doc.get("client_id").and_then(Value::as_str) != Some(client_id) {
        return Err(ClientError::Invalid(
            "client_id in the document does not match its address".into(),
        ));
    }
    let meta = Metadata::from_json(&doc, insecure).map_err(ClientError::Invalid)?;
    let id: Uuid =
        sqlx::query_scalar("SELECT oauth_client_upsert($1, $2, 'cimd', $3, $4, $5, $6, $7, $8)")
            .bind(Uuid::now_v7())
            .bind(client_id)
            .bind(&meta.name)
            .bind(&meta.logo_uri)
            .bind(&meta.client_uri)
            .bind(&meta.redirect_uris)
            .bind("none")
            .bind(Utc::now() + chrono::Duration::seconds(ttl))
            .fetch_one(&state.db)
            .await?;
    by_pk(&state.db, id)
        .await?
        .ok_or_else(|| ClientError::Invalid("client".into()))
}

fn hex(bytes: &[u8]) -> String {
    crate::auth::tokens::hex(bytes)
}

/// The validated fields of a client document or registration request.
#[derive(Debug)]
pub struct Metadata {
    /// Display name.
    pub name: String,
    /// Logo address.
    pub logo_uri: Option<String>,
    /// Client home page.
    pub client_uri: Option<String>,
    /// Redirect addresses.
    pub redirect_uris: Vec<String>,
}

impl Metadata {
    /// Reads and validates the common fields (CIMD document and DCR body share them).
    ///
    /// # Errors
    /// A short reason.
    pub fn from_json(doc: &Value, insecure: bool) -> Result<Self, String> {
        let name = doc
            .get("client_name")
            .and_then(Value::as_str)
            .map_or("Unnamed app", str::trim);
        let name = if name.is_empty() { "Unnamed app" } else { name };
        if name.chars().count() > 200 || name.chars().any(char::is_control) {
            return Err("client_name".into());
        }
        let method = doc
            .get("token_endpoint_auth_method")
            .and_then(Value::as_str)
            .unwrap_or("none");
        if method != "none" {
            return Err("only token_endpoint_auth_method none is supported".into());
        }
        for (field, allowed) in [
            ("grant_types", &["authorization_code", "refresh_token"][..]),
            ("response_types", &["code"][..]),
        ] {
            if let Some(list) = doc.get(field) {
                let ok = list.as_array().is_some_and(|a| {
                    a.iter()
                        .all(|v| v.as_str().is_some_and(|s| allowed.contains(&s)))
                });
                if !ok {
                    return Err(field.into());
                }
            }
        }
        let uris = doc
            .get("redirect_uris")
            .and_then(Value::as_array)
            .ok_or("redirect_uris")?;
        if uris.is_empty() || uris.len() > 20 {
            return Err("redirect_uris".into());
        }
        let mut redirect_uris = Vec::new();
        for v in uris {
            let s = v.as_str().ok_or("redirect_uris")?;
            if !valid_redirect_uri(s, insecure) {
                return Err("redirect_uris".into());
            }
            if !redirect_uris.iter().any(|x| x == s) {
                redirect_uris.push(s.to_owned());
            }
        }
        let https_field = |f: &str| -> Result<Option<String>, String> {
            match doc.get(f).and_then(Value::as_str) {
                None => Ok(None),
                Some(s)
                    if s.len() <= 2048 && s.starts_with("https://") && Url::parse(s).is_ok() =>
                {
                    Ok(Some(s.to_owned()))
                }
                Some(_) => Err(f.to_owned()),
            }
        };
        Ok(Self {
            name: name.to_owned(),
            logo_uri: https_field("logo_uri")?,
            client_uri: https_field("client_uri")?,
            redirect_uris,
        })
    }
}

/// A redirect address: HTTPS, loopback HTTP, or a private-use scheme of a native app. No
/// fragment, no credentials.
#[must_use]
pub fn valid_redirect_uri(value: &str, insecure: bool) -> bool {
    if value.len() > 2048 {
        return false;
    }
    let Ok(url) = Url::parse(value) else {
        return false;
    };
    if url.fragment().is_some() || !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    match url.scheme() {
        "https" => url.host_str().is_some(),
        "http" => insecure || matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")),
        "javascript" | "data" | "file" | "vbscript" | "about" | "blob" | "ftp" | "ws" | "wss" => {
            false
        }
        _ => true,
    }
}

/// Host text of an address, for the consent screen.
#[must_use]
pub fn host_of(value: &str) -> String {
    Url::parse(value)
        .ok()
        .and_then(|u| u.host_str().map(str::to_owned))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn private_and_local_addresses_are_refused() {
        for ip in [
            "127.0.0.1",
            "10.1.2.3",
            "172.16.0.1",
            "192.168.1.1",
            "169.254.169.254",
            "100.64.0.1",
            "0.0.0.0",
            "::1",
            "fd00::1",
            "fe80::1",
            "::ffff:10.0.0.1",
            "0.1.2.3",
            "198.18.0.1",
            "198.19.255.255",
            "240.0.0.1",
            "255.255.255.255",
            "64:ff9b::7f00:1",
            "2002:7f00:1::1",
        ] {
            assert!(private(ip.parse().unwrap()), "{ip}");
        }
        for ip in [
            "8.8.8.8",
            "198.17.255.255",
            "198.20.0.1",
            "223.0.0.1",
            "2606:4700::1111",
        ] {
            assert!(!private(ip.parse().unwrap()), "{ip}");
        }
    }

    #[test]
    fn redirect_addresses_follow_the_rules() {
        assert!(valid_redirect_uri("https://app.example.com/cb", false));
        assert!(valid_redirect_uri("http://127.0.0.1:8765/cb", false));
        assert!(valid_redirect_uri("cursor://anysphere.cursor/oauth", false));
        assert!(!valid_redirect_uri("http://evil.example.com/cb", false));
        assert!(!valid_redirect_uri("javascript:alert(1)", false));
        assert!(!valid_redirect_uri("https://app.example.com/cb#x", false));
        assert!(!valid_redirect_uri("not a url", false));
    }

    #[test]
    fn metadata_accepts_public_clients_only() {
        let ok = json!({"client_name": "Agent", "redirect_uris": ["https://a.example/cb"]});
        assert!(Metadata::from_json(&ok, false).is_ok());
        let jwt = json!({"redirect_uris": ["https://a.example/cb"],
                         "token_endpoint_auth_method": "private_key_jwt"});
        assert!(Metadata::from_json(&jwt, false).is_err());
        let implicit = json!({"redirect_uris": ["https://a.example/cb"],
                              "response_types": ["token"]});
        assert!(Metadata::from_json(&implicit, false).is_err());
    }

    #[test]
    fn cache_time_is_bounded() {
        assert_eq!(max_age(None), DEFAULT_CACHE_SECS);
        assert_eq!(max_age(Some("max-age=99999999")), MAX_CACHE_SECS);
        assert_eq!(max_age(Some("no-store")), 60);
    }
}
