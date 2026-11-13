//! A fake Apple and Google OIDC provider for tests and the local compose stack (T2.11).
//!
//! It serves a discovery document and the JWKS of each provider and mints ID tokens on request,
//! including deliberately broken ones (wrong signature, `kid`, `iss`, `aud`, time, `alg`), so the
//! API's token checks (ADR-0009 rule 1.3) can be tested end to end. Issuers are
//! `<base>/apple` and `<base>/google`. Not for production: anyone can mint a token.
//!
//! Use it in-process (`FakeOidc::new` and [`FakeOidc::mint`]) or over HTTP (`POST /{provider}/mint`).

use std::{
    sync::{Arc, Mutex, PoisonError},
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rsa::{
    RsaPrivateKey,
    pkcs1v15::SigningKey,
    signature::{SignatureEncoding, Signer},
    traits::PublicKeyParts,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2_010::Sha256;

/// Key size; the API verifier accepts 2048 to 8192 bits.
const KEY_BITS: usize = 2048;

/// The two providers the API signs in with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    /// Sign in with Apple.
    Apple,
    /// Google sign-in.
    Google,
}

impl Provider {
    /// Path segment and issuer suffix.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Apple => "apple",
            Self::Google => "google",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "apple" => Some(Self::Apple),
            "google" => Some(Self::Google),
            _ => None,
        }
    }
}

/// Which key signs a minted token.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyChoice {
    /// The provider's published key.
    #[default]
    Provider,
    /// A different key that is not published (bad signature).
    Wrong,
}

/// What to put in a minted token. Every field has a sensible default; tests override the
/// one that should make the token invalid.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MintSpec {
    /// `sub`.
    pub sub: String,
    /// `email`.
    pub email: Option<String>,
    /// `email_verified` (`true`/`false`; Apple sends the string form, see `email_verified_as_string`).
    pub email_verified: Option<bool>,
    /// Send `email_verified` as the string `"true"`/`"false"` (as Apple does).
    pub email_verified_as_string: bool,
    /// `aud`.
    pub aud: String,
    /// `nonce` claim, sent as given (Apple callers pass the SHA-256 hex of the raw nonce).
    pub nonce: Option<String>,
    /// `name`.
    pub name: Option<String>,
    /// Seconds until `exp` (negative: already expired).
    pub exp_in_secs: i64,
    /// Seconds added to now for `iat` (positive: issued in the future).
    pub iat_offset_secs: i64,
    /// Overrides `iss`.
    pub iss: Option<String>,
    /// Overrides the header `kid`.
    pub kid: Option<String>,
    /// Overrides the header `alg` (the signature stays RS256).
    pub alg: Option<String>,
    /// Which key signs.
    pub signer: KeyChoice,
    /// Leaves the signature empty.
    pub unsigned: bool,
}

impl Default for MintSpec {
    fn default() -> Self {
        Self {
            sub: "fake-user-1".to_owned(),
            email: None,
            email_verified: None,
            email_verified_as_string: false,
            aud: "fake-client".to_owned(),
            nonce: None,
            name: None,
            exp_in_secs: 3600,
            iat_offset_secs: 0,
            iss: None,
            kid: None,
            alg: None,
            signer: KeyChoice::Provider,
            unsigned: false,
        }
    }
}

struct Key {
    kid: String,
    private: RsaPrivateKey,
}

impl Key {
    fn generate(kid: String) -> Self {
        let private = RsaPrivateKey::new(&mut rand::rngs::OsRng, KEY_BITS)
            .unwrap_or_else(|e| panic!("RSA key generation failed: {e}"));
        Self { kid, private }
    }

    fn jwk(&self) -> Value {
        let public = self.private.to_public_key();
        json!({
            "kty": "RSA", "use": "sig", "alg": "RS256", "kid": self.kid,
            "n": URL_SAFE_NO_PAD.encode(public.n().to_bytes_be()),
            "e": URL_SAFE_NO_PAD.encode(public.e().to_bytes_be()),
        })
    }

    fn sign(&self, input: &[u8]) -> Vec<u8> {
        SigningKey::<Sha256>::new(self.private.clone())
            .sign(input)
            .to_vec()
    }
}

struct Keys {
    apple: Key,
    google: Key,
    wrong: Key,
    rotations: u32,
}

/// The provider state: signing keys and the public base URL.
pub struct FakeOidc {
    base_url: String,
    keys: Mutex<Keys>,
}

impl std::fmt::Debug for FakeOidc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FakeOidc")
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

fn b64(value: &Value) -> String {
    URL_SAFE_NO_PAD.encode(value.to_string())
}

impl FakeOidc {
    /// Creates the provider (generates three RSA keys) for the given public base URL, without
    /// a trailing slash. The issuers are `<base_url>/apple` and `<base_url>/google`.
    #[must_use]
    pub fn new(base_url: &str) -> Arc<Self> {
        Arc::new(Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            keys: Mutex::new(Keys {
                apple: Key::generate("apple-1".to_owned()),
                google: Key::generate("google-1".to_owned()),
                wrong: Key::generate("wrong-1".to_owned()),
                rotations: 1,
            }),
        })
    }

    /// The issuer of `provider`.
    #[must_use]
    pub fn issuer(&self, provider: Provider) -> String {
        format!("{}/{}", self.base_url, provider.as_str())
    }

    /// The JWKS address of `provider`.
    #[must_use]
    pub fn jwks_url(&self, provider: Provider) -> String {
        format!("{}/jwks", self.issuer(provider))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Keys> {
        self.keys.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The key id currently published for `provider`.
    #[must_use]
    pub fn kid(&self, provider: Provider) -> String {
        let keys = self.lock();
        match provider {
            Provider::Apple => keys.apple.kid.clone(),
            Provider::Google => keys.google.kid.clone(),
        }
    }

    /// The JWKS document of `provider`.
    #[must_use]
    pub fn jwks(&self, provider: Provider) -> Value {
        let keys = self.lock();
        let key = match provider {
            Provider::Apple => &keys.apple,
            Provider::Google => &keys.google,
        };
        json!({"keys": [key.jwk()]})
    }

    /// Replaces the published key of `provider` with a new one (new `kid`); tokens signed by the
    /// old key stop verifying once a client has refreshed its JWKS.
    pub fn rotate(&self, provider: Provider) -> String {
        let mut keys = self.lock();
        keys.rotations += 1;
        let kid = format!("{}-{}", provider.as_str(), keys.rotations);
        let key = Key::generate(kid.clone());
        match provider {
            Provider::Apple => keys.apple = key,
            Provider::Google => keys.google = key,
        }
        kid
    }

    /// Mints an ID token as `spec` describes.
    #[must_use]
    pub fn mint(&self, provider: Provider, spec: &MintSpec) -> String {
        let keys = self.lock();
        let published = match provider {
            Provider::Apple => &keys.apple,
            Provider::Google => &keys.google,
        };
        let key = match spec.signer {
            KeyChoice::Provider => published,
            KeyChoice::Wrong => &keys.wrong,
        };
        let now = now_secs();
        let header = json!({
            "alg": spec.alg.as_deref().unwrap_or("RS256"),
            "kid": spec.kid.clone().unwrap_or_else(|| published.kid.clone()),
            "typ": "JWT",
        });
        let mut claims = json!({
            "iss": spec.iss.clone().unwrap_or_else(|| self.issuer(provider)),
            "sub": spec.sub,
            "aud": spec.aud,
            "iat": now + spec.iat_offset_secs,
            "exp": now + spec.exp_in_secs,
        });
        if let Some(email) = &spec.email {
            claims["email"] = json!(email);
        }
        if let Some(verified) = spec.email_verified {
            claims["email_verified"] = if spec.email_verified_as_string {
                json!(verified.to_string())
            } else {
                json!(verified)
            };
        }
        if let Some(nonce) = &spec.nonce {
            claims["nonce"] = json!(nonce);
        }
        if let Some(name) = &spec.name {
            claims["name"] = json!(name);
        }
        let input = format!("{}.{}", b64(&header), b64(&claims));
        let signature = if spec.unsigned {
            Vec::new()
        } else {
            key.sign(input.as_bytes())
        };
        format!("{input}.{}", URL_SAFE_NO_PAD.encode(signature))
    }

    /// The HTTP router (`/{provider}/jwks`, `/{provider}/mint`, ...).
    pub fn router(self: &Arc<Self>) -> Router {
        Router::new()
            .route("/healthz", get(|| async { "ok" }))
            .route(
                "/{provider}/.well-known/openid-configuration",
                get(discovery),
            )
            .route("/{provider}/jwks", get(jwks))
            .route("/{provider}/mint", post(mint))
            .route("/{provider}/rotate", post(rotate))
            .with_state(self.clone())
    }
}

type Shared = State<Arc<FakeOidc>>;

fn provider_of(name: &str) -> Result<Provider, StatusCode> {
    Provider::parse(name).ok_or(StatusCode::NOT_FOUND)
}

async fn discovery(State(s): Shared, Path(p): Path<String>) -> Result<Json<Value>, StatusCode> {
    let provider = provider_of(&p)?;
    Ok(Json(json!({
        "issuer": s.issuer(provider),
        "jwks_uri": s.jwks_url(provider),
        "id_token_signing_alg_values_supported": ["RS256"],
    })))
}

async fn jwks(State(s): Shared, Path(p): Path<String>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(s.jwks(provider_of(&p)?)))
}

async fn mint(
    State(s): Shared,
    Path(p): Path<String>,
    Json(spec): Json<MintSpec>,
) -> Result<Json<Value>, StatusCode> {
    let provider = provider_of(&p)?;
    Ok(Json(json!({
        "id_token": s.mint(provider, &spec),
        "issuer": s.issuer(provider),
    })))
}

async fn rotate(State(s): Shared, Path(p): Path<String>) -> Result<Json<Value>, StatusCode> {
    let provider = provider_of(&p)?;
    Ok(Json(json!({"kid": s.rotate(provider)})))
}

/// A running in-process server.
#[derive(Debug)]
pub struct Running {
    /// Base URL (`http://127.0.0.1:<port>`).
    pub url: String,
    /// The provider, for minting tokens in the test itself.
    pub oidc: Arc<FakeOidc>,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for Running {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// Starts the fake provider on a free local port. The keys are generated on a blocking thread.
///
/// # Panics
/// When the port cannot be bound or key generation panics.
pub async fn spawn_local() -> Running {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap_or_else(|e| panic!("bind failed: {e}"));
    let port = listener
        .local_addr()
        .unwrap_or_else(|e| panic!("local_addr failed: {e}"))
        .port();
    let url = format!("http://127.0.0.1:{port}");
    let base = url.clone();
    let oidc = tokio::task::spawn_blocking(move || FakeOidc::new(&base))
        .await
        .unwrap_or_else(|e| panic!("key generation failed: {e}"));
    let app = oidc.router();
    let task = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    Running { url, oidc, task }
}
