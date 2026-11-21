//! A small GitHub REST client for the private support repository.
//!
//! Authentication: a GitHub App installation (`GITHUB_APP_ID`, `GITHUB_APP_INSTALLATION_ID`,
//! `GITHUB_APP_KEY_FILE`) or, for development, a token (`GITHUB_TOKEN`). `GITHUB_API_URL` is
//! `https://api.github.com` by default (the fake GitHub in tests and in compose).

use std::time::{Duration, Instant};

use base64::{Engine, engine::general_purpose::STANDARD, engine::general_purpose::URL_SAFE_NO_PAD};
use reqwest::Method;
use ring::{
    rand::SystemRandom,
    signature::{RSA_PKCS1_SHA256, RsaKeyPair},
};
use serde_json::{Value, json};
use tokio::sync::Mutex;

/// Why a call failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GitHubError {
    /// GitHub does not answer, answers with a server error, or limits the rate. A retry can help.
    #[error("github unavailable: {reason}")]
    Unavailable {
        /// What happened.
        reason: String,
        /// Seconds GitHub asked to wait.
        retry_after: Option<u64>,
    },
    /// GitHub refused the request. A retry does not help.
    #[error("github rejected the request: {0}")]
    Rejected(String),
}

/// GitHub App credentials.
#[derive(Debug)]
pub struct AppAuth {
    app_id: String,
    installation_id: String,
    key: RsaKeyPair,
    cached: Mutex<Option<(Instant, String)>>,
}

/// How the client authenticates.
#[derive(Debug)]
pub enum Auth {
    /// A fixed token.
    Token(String),
    /// A GitHub App installation token.
    App(Box<AppAuth>),
}

/// The client.
#[derive(Debug)]
pub struct GitHub {
    http: reqwest::Client,
    api_url: String,
    repo: String,
    auth: Auth,
    bot_login: String,
}

impl AppAuth {
    /// Reads the PEM text of the App key (PKCS#1 or PKCS#8).
    ///
    /// # Errors
    /// A text that says what is wrong with the key.
    pub fn from_pem(app_id: &str, installation_id: &str, pem: &str) -> Result<Self, String> {
        let pkcs1 = pem.contains("BEGIN RSA PRIVATE KEY");
        let body: String = pem.lines().filter(|l| !l.starts_with("-----")).collect();
        let der = STANDARD
            .decode(body.trim())
            .map_err(|e| format!("GitHub App key is not base64: {e}"))?;
        let key = if pkcs1 {
            RsaKeyPair::from_der(&der)
        } else {
            RsaKeyPair::from_pkcs8(&der)
        }
        .map_err(|e| format!("GitHub App key is not an RSA key: {e}"))?;
        Ok(Self {
            app_id: app_id.to_owned(),
            installation_id: installation_id.to_owned(),
            key,
            cached: Mutex::new(None),
        })
    }

    fn jwt(&self, now_unix: i64) -> Result<String, String> {
        let head = URL_SAFE_NO_PAD.encode(json!({"alg": "RS256", "typ": "JWT"}).to_string());
        let claims = json!({"iat": now_unix - 60, "exp": now_unix + 9 * 60, "iss": self.app_id});
        let body = URL_SAFE_NO_PAD.encode(claims.to_string());
        let input = format!("{head}.{body}");
        let mut sig = vec![0; self.key.public().modulus_len()];
        self.key
            .sign(
                &RSA_PKCS1_SHA256,
                &SystemRandom::new(),
                input.as_bytes(),
                &mut sig,
            )
            .map_err(|e| format!("signing failed: {e}"))?;
        Ok(format!("{input}.{}", URL_SAFE_NO_PAD.encode(sig)))
    }
}

impl GitHub {
    /// A client for `repo` (`owner/name`).
    #[must_use]
    pub fn new(api_url: &str, repo: &str, auth: Auth, bot_login: &str) -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .user_agent("demleme-support")
                .build()
                .unwrap_or_default(),
            api_url: api_url.trim_end_matches('/').to_owned(),
            repo: repo.to_owned(),
            auth,
            bot_login: bot_login.to_ascii_lowercase(),
        }
    }

    /// Reads the settings from the environment. `None` when `GITHUB_REPO` or the credentials are
    /// missing.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let var = |n: &str| std::env::var(n).ok().filter(|v| !v.is_empty());
        let repo = var("GITHUB_REPO")?;
        let auth = if let Some(token) = var("GITHUB_TOKEN") {
            Auth::Token(token)
        } else {
            let (app, inst, file) = (
                var("GITHUB_APP_ID")?,
                var("GITHUB_APP_INSTALLATION_ID")?,
                var("GITHUB_APP_KEY_FILE")?,
            );
            let pem = std::fs::read_to_string(file).ok()?;
            match AppAuth::from_pem(&app, &inst, &pem) {
                Ok(a) => Auth::App(Box::new(a)),
                Err(e) => {
                    tracing::error!(error = %e, "GitHub App key unusable; support sync is off");
                    return None;
                }
            }
        };
        Some(Self::new(
            &var("GITHUB_API_URL").unwrap_or_else(|| "https://api.github.com".to_owned()),
            &repo,
            auth,
            &var("GITHUB_BOT_LOGIN").unwrap_or_else(|| "demleme-support[bot]".to_owned()),
        ))
    }

    /// Login of the App; its own comments and events are ignored (lower case).
    #[must_use]
    pub fn bot_login(&self) -> &str {
        &self.bot_login
    }

    async fn token(&self) -> Result<String, GitHubError> {
        match &self.auth {
            Auth::Token(t) => Ok(t.clone()),
            Auth::App(app) => {
                let mut guard = app.cached.lock().await;
                if let Some((at, token)) = guard.as_ref()
                    && at.elapsed() < Duration::from_mins(50)
                {
                    return Ok(token.clone());
                }
                let jwt = app
                    .jwt(chrono::Utc::now().timestamp())
                    .map_err(GitHubError::Rejected)?;
                let url = format!(
                    "{}/app/installations/{}/access_tokens",
                    self.api_url, app.installation_id
                );
                let reply = self.send(Method::POST, &url, &jwt, None).await?;
                let token = reply["token"]
                    .as_str()
                    .ok_or_else(|| GitHubError::Rejected("no installation token".to_owned()))?
                    .to_owned();
                *guard = Some((Instant::now(), token.clone()));
                Ok(token)
            }
        }
    }

    async fn send(
        &self,
        method: Method,
        url: &str,
        token: &str,
        body: Option<&Value>,
    ) -> Result<Value, GitHubError> {
        let mut request = self
            .http
            .request(method, url)
            .bearer_auth(token)
            .header("accept", "application/vnd.github+json")
            .header("x-github-api-version", "2022-11-28");
        if let Some(b) = body {
            request = request.json(b);
        }
        let response = request.send().await.map_err(|e| GitHubError::Unavailable {
            reason: e.to_string(),
            retry_after: None,
        })?;
        let status = response.status();
        let header = |n: &str| {
            response
                .headers()
                .get(n)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
        };
        let retry_after = header("retry-after").and_then(|v| v.parse::<u64>().ok());
        let exhausted = header("x-ratelimit-remaining").as_deref() == Some("0");
        let reset = header("x-ratelimit-reset").and_then(|v| v.parse::<i64>().ok());
        let text = response.text().await.unwrap_or_default();
        if status.is_success() {
            return Ok(serde_json::from_str(&text).unwrap_or(Value::Null));
        }
        let limited = status.as_u16() == 429
            || (status.as_u16() == 403 && (retry_after.is_some() || exhausted));
        if limited || status.is_server_error() {
            let wait = retry_after.or_else(|| {
                reset.map(|r| u64::try_from(r - chrono::Utc::now().timestamp()).unwrap_or(0))
            });
            return Err(GitHubError::Unavailable {
                reason: format!("{status}"),
                retry_after: wait,
            });
        }
        Err(GitHubError::Rejected(format!(
            "{status} {}",
            text.chars().take(200).collect::<String>()
        )))
    }

    async fn call(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value, GitHubError> {
        let token = self.token().await?;
        let url = format!("{}/repos/{}{path}", self.api_url, self.repo);
        self.send(method, &url, &token, body.as_ref()).await
    }

    /// Opens an Issue. Returns its number.
    ///
    /// # Errors
    /// See [`GitHubError`].
    pub async fn create_issue(
        &self,
        title: &str,
        body: &str,
        labels: &[String],
    ) -> Result<i64, GitHubError> {
        let reply = self
            .call(
                Method::POST,
                "/issues",
                Some(json!({"title": title, "body": body, "labels": labels})),
            )
            .await?;
        reply["number"]
            .as_i64()
            .ok_or_else(|| GitHubError::Rejected("no issue number".to_owned()))
    }

    /// Adds a comment. Returns its id.
    ///
    /// # Errors
    /// See [`GitHubError`].
    pub async fn add_comment(&self, issue: i64, body: &str) -> Result<i64, GitHubError> {
        let reply = self
            .call(
                Method::POST,
                &format!("/issues/{issue}/comments"),
                Some(json!({"body": body})),
            )
            .await?;
        reply["id"]
            .as_i64()
            .ok_or_else(|| GitHubError::Rejected("no comment id".to_owned()))
    }

    /// Changes a comment. A missing comment counts as done.
    ///
    /// # Errors
    /// See [`GitHubError`].
    pub async fn edit_comment(&self, comment: i64, body: &str) -> Result<(), GitHubError> {
        match self
            .call(
                Method::PATCH,
                &format!("/issues/comments/{comment}"),
                Some(json!({"body": body})),
            )
            .await
        {
            Err(GitHubError::Rejected(r)) if r.starts_with("404") => Ok(()),
            other => other.map(|_| ()),
        }
    }

    /// Deletes a comment. A missing comment counts as done.
    ///
    /// # Errors
    /// See [`GitHubError`].
    pub async fn delete_comment(&self, comment: i64) -> Result<(), GitHubError> {
        match self
            .call(Method::DELETE, &format!("/issues/comments/{comment}"), None)
            .await
        {
            Err(GitHubError::Rejected(r)) if r.starts_with("404") => Ok(()),
            other => other.map(|_| ()),
        }
    }

    /// Removes the personal content of an Issue (account deletion, T2.31): deletes every
    /// comment, blanks the title and the body, closes the Issue. A missing Issue counts as done.
    /// GitHub has no REST call that deletes an Issue; the GraphQL `deleteIssue` mutation needs
    /// an administrator right (to be verified).
    ///
    /// # Errors
    /// See [`GitHubError`].
    pub async fn scrub_issue(&self, issue: i64) -> Result<(), GitHubError> {
        let gone = |r: &GitHubError| matches!(r, GitHubError::Rejected(m) if m.starts_with("404"));
        for _ in 0..100 {
            let list = match self
                .call(
                    Method::GET,
                    &format!("/issues/{issue}/comments?per_page=100"),
                    None,
                )
                .await
            {
                Ok(v) => v,
                Err(e) if gone(&e) => return Ok(()),
                Err(e) => return Err(e),
            };
            let ids: Vec<i64> = list
                .as_array()
                .map(|a| a.iter().filter_map(|c| c["id"].as_i64()).collect())
                .unwrap_or_default();
            if ids.is_empty() {
                break;
            }
            for id in ids {
                self.delete_comment(id).await?;
            }
        }
        match self
            .call(
                Method::PATCH,
                &format!("/issues/{issue}"),
                Some(json!({"title": "Deleted request", "state": "closed",
                            "body": "Deleted at the request of the account owner."})),
            )
            .await
        {
            Err(e) if gone(&e) => Ok(()),
            other => other.map(|_| ()),
        }
    }

    /// Sets the state and the labels of an Issue.
    ///
    /// # Errors
    /// See [`GitHubError`].
    pub async fn update_issue(
        &self,
        issue: i64,
        closed: bool,
        labels: &[String],
    ) -> Result<(), GitHubError> {
        self.call(
            Method::PATCH,
            &format!("/issues/{issue}"),
            Some(json!({"state": if closed { "closed" } else { "open" }, "labels": labels})),
        )
        .await
        .map(|_| ())
    }
}

/// A boxed future, as the trait below needs.
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

/// Removes the personal content of Issues (account deletion, T2.31). [`GitHub`] implements
/// it; tests use a fake.
pub trait IssueScrubber: Send + Sync {
    /// See [`GitHub::scrub_issue`].
    fn scrub(&self, issue: i64) -> BoxFuture<'_, Result<(), GitHubError>>;
}

impl IssueScrubber for GitHub {
    fn scrub(&self, issue: i64) -> BoxFuture<'_, Result<(), GitHubError>> {
        Box::pin(self.scrub_issue(issue))
    }
}
