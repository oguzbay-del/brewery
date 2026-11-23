//! HTTP-only end-to-end helpers for the local Compose stack.
//!
//! Tests are skipped when E2E_BASE_URL is not set, so the ordinary workspace test suite
//! does not require a running Compose environment.

use std::{env, error::Error};

use reqwest::{Client, Method, RequestBuilder, Response};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub type TestResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/// A configured HTTP client for the Compose API and its fake external services.
#[derive(Debug, Clone)]
pub struct Context {
    pub client: Client,
    pub base_url: String,
    pub fake_oidc_url: String,
    pub mailpit_url: String,
}

/// An authenticated API user.
#[derive(Debug, Clone)]
pub struct User {
    pub token: String,
    pub email: String,
}

impl Context {
    /// Returns None when E2E_BASE_URL is absent; this is the normal workspace-test mode.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let base_url = env::var("E2E_BASE_URL").ok()?;
        if base_url.trim().is_empty() {
            return None;
        }
        Some(Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_owned(),
            fake_oidc_url: env::var("E2E_FAKE_OIDC_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:9000".to_owned())
                .trim_end_matches('/')
                .to_owned(),
            mailpit_url: env::var("E2E_MAILPIT_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8025".to_owned())
                .trim_end_matches('/')
                .to_owned(),
        })
    }

    /// Creates a fresh Apple-backed user through the public fake-OIDC and API HTTP endpoints.
    pub async fn login(&self) -> TestResult<User> {
        let email = format!("e2e-{}@example.test", Uuid::new_v4());
        self.login_with_email(&email).await
    }

    /// Creates a fresh Apple-backed user with the requested verified email.
    pub async fn login_with_email(&self, email: &str) -> TestResult<User> {
        let nonce_response = self
            .client
            .post(format!("{}/v1/auth/nonce", self.base_url))
            .send()
            .await?
            .error_for_status()?;
        let nonce_body: Value = nonce_response.json().await?;
        let nonce = nonce_body["nonce"]
            .as_str()
            .ok_or("nonce response did not contain nonce")?;
        let digest = Sha256::digest(nonce.as_bytes());
        let nonce_hash = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        let subject = format!("e2e-{}", Uuid::new_v4());

        let minted: Value = self
            .client
            .post(format!("{}/apple/mint", self.fake_oidc_url))
            .json(&json!({
                "sub": subject,
                "aud": "com.demleme.app.dev",
                "nonce": nonce_hash,
                "email": email,
                "email_verified": true,
                "email_verified_as_string": true,
                "name": "GRISTORA E2E"
            }))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let id_token = minted["id_token"]
            .as_str()
            .ok_or("fake OIDC response did not contain id_token")?;

        let login: Value = self
            .client
            .post(format!("{}/v1/auth/apple", self.base_url))
            .json(&json!({
                "id_token": id_token,
                "nonce": nonce,
                "client": "ios",
                "display_name": "GRISTORA E2E"
            }))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let token = login["session"]["token"]
            .as_str()
            .ok_or("login response did not contain a session token")?
            .to_owned();

        Ok(User {
            token,
            email: email.to_owned(),
        })
    }

    /// Builds an API request with optional bearer authentication.
    #[must_use]
    pub fn request(&self, method: Method, path: &str, user: Option<&User>) -> RequestBuilder {
        let request = self.client.request(
            method,
            format!("{}/{}", self.base_url, path.trim_start_matches('/')),
        );
        if let Some(user) = user {
            request.bearer_auth(&user.token)
        } else {
            request
        }
    }

    /// Sends a JSON POST with the required idempotency header.
    pub async fn post_json(
        &self,
        path: &str,
        user: Option<&User>,
        body: &Value,
    ) -> TestResult<Response> {
        Ok(self
            .request(Method::POST, path, user)
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(body)
            .send()
            .await?)
    }

    /// Creates a workspace and returns its response body.
    pub async fn create_workspace(&self, user: &User, mode: &str, name: &str) -> TestResult<Value> {
        Ok(self
            .post_json(
                "/v1/workspaces",
                Some(user),
                &json!({
                    "name": name,
                    "mode": mode,
                    "region": "us",
                    "country": "US",
                    "time_zone": "America/New_York"
                }),
            )
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Creates a recipe and returns its response body and headers.
    pub async fn create_recipe(
        &self,
        user: &User,
        workspace_id: &str,
        name: &str,
    ) -> TestResult<Response> {
        self.post_json(
            &format!("/v1/workspaces/{workspace_id}/recipes"),
            Some(user),
            &json!({"name": name}),
        )
        .await
    }

    /// Finds the one-time invitation token in the local Mailpit inbox.
    pub async fn invitation_token(&self, email: &str) -> TestResult<String> {
        let list: Value = self
            .client
            .get(format!("{}/api/v1/messages?limit=50", self.mailpit_url))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let messages = list["messages"]
            .as_array()
            .or_else(|| list.as_array())
            .ok_or("Mailpit response did not contain a messages array")?;

        for message in messages {
            let id = message["ID"]
                .as_str()
                .or_else(|| message["id"].as_str());
            let Some(id) = id else { continue };
            let detail: Value = self
                .client
                .get(format!("{}/api/v1/message/{id}", self.mailpit_url))
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            let content = detail.to_string();
            if !content.contains(email) {
                continue;
            }
            if let Some(token) = token_after(&content, "token=") {
                return Ok(token);
            }
            if let Some(token) = token_after(&content, "/accept/") {
                return Ok(token);
            }
        }
        Err("no invitation token for the invited email was found in Mailpit".into())
    }
}

fn token_after(content: &str, marker: &str) -> Option<String> {
    let start = content.find(marker)? + marker.len();
    let token = content[start..]
        .split(|c: char| matches!(c, '&' | '"' | '\'' | '<' | '>' | ' ' | ')' | '\\'))
        .next()?
        .trim();
    if token.is_empty() { None } else { Some(token.to_owned()) }
}
