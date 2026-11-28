//! Web sign-in with the redirect code flow of Apple and Google (T3.1; ADR-0009 rules 1 and 3).
//!
//! The browser never sees a provider script. The flow has two steps:
//!
//! 1. `GET /v1/auth/{provider}/start`: the server makes `state`, `nonce` and (Google) a PKCE
//!    verifier, keeps them in Redis for 10 minutes, ties them to the browser with the cookie
//!    `__Host-oauth_state` and sends the browser to the provider (303).
//! 2. The provider sends the browser back to the callback (Apple: `form_post`, Google: query).
//!    The server takes the state out of Redis one time only, compares it with the cookie,
//!    exchanges the code, verifies the ID token and the nonce, and then does the same account
//!    and session steps as `POST /v1/auth/{provider}` for a `web` session.
//!
//! The callback answers 303 in every case, because the user is in the browser. An error goes to
//! `<web base url>/sign-in?error=<code>`.

use std::borrow::Cow;

use axum::{body::Bytes, extract::RawQuery};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};

use super::{
    ApiError, ApiResult, AuthServices, ClientKind, Device, ErrorCode, Extension, FieldIssue,
    HeaderMap, HeaderValue, Path, Response, StatusCode, Svc, ToSchema, country_of, identity_claim,
    new_token, seal_apple_refresh,
};
use crate::auth::{apple_token::AppleTokenError, config::WebFlowProvider};

/// Name of the cookie that ties the flow to the browser.
pub const STATE_COOKIE: &str = "__Host-oauth_state";
/// Life of the flow in Redis and of the cookie.
const FLOW_TTL_SECS: u64 = 600;
const KEY_PREFIX: &str = "webauth:state:";
const MAX_PARAM_LEN: usize = 4096;
const MAX_RETURN_TO_LEN: usize = 512;
const MAX_TOKEN_RESPONSE: usize = 65_536;
/// Longest display name taken from the Apple `user` form field.
const MAX_DISPLAY_NAME: usize = 100;

/// What the server keeps for one flow.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct FlowState {
    provider: String,
    nonce: String,
    /// PKCE code verifier (Google).
    verifier: Option<String>,
    /// Random value that the cookie `__Host-oauth_state` carries.
    binding: String,
    return_to: String,
    remember: bool,
}

/// Form body of the Apple callback (`response_mode=form_post`). Documentation schema only;
/// the handler reads the raw body.
#[allow(dead_code)]
#[derive(Debug, ToSchema)]
pub struct AppleCallbackForm {
    /// Authorization code.
    pub code: Option<String>,
    /// The `state` from the start step.
    pub state: Option<String>,
    /// Error code of Apple, if the user did not agree.
    pub error: Option<String>,
    /// JSON with the name of the user. Apple sends it only on the first authorization.
    pub user: Option<String>,
}

// --- helpers -------------------------------------------------------------------------------

fn parse_provider(raw: &str) -> Option<&'static str> {
    match raw {
        "apple" => Some("apple"),
        "google" => Some("google"),
        _ => None,
    }
}

/// A return path that cannot leave the web app: one leading `/`, no second `/`, no backslash,
/// no control or non-ASCII character (browsers drop tabs and line breaks inside a URL, so
/// `/\t/evil.example` would turn into `//evil.example`). Anything else gives `/`.
fn safe_return_to(raw: Option<&str>) -> String {
    let Some(path) = raw else {
        return "/".to_owned();
    };
    let ok = path.len() <= MAX_RETURN_TO_LEN
        && path.starts_with('/')
        && !path.starts_with("//")
        && path.bytes().all(|b| b.is_ascii_graphic() && b != b'\\');
    if ok { path.to_owned() } else { "/".to_owned() }
}

fn state_cookie(value: &str) -> String {
    format!(
        "{STATE_COOKIE}={value}; HttpOnly; Secure; SameSite=None; Path=/; Max-Age={FLOW_TTL_SECS}"
    )
}

fn clear_state_cookie() -> String {
    format!("{STATE_COOKIE}=; HttpOnly; Secure; SameSite=None; Path=/; Max-Age=0")
}

fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(axum::http::header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find_map(|(n, v)| (n == name).then_some(v))
        .filter(|v| !v.is_empty())
}

fn same(a: &str, b: &str) -> bool {
    // Compare the digests, so the time does not depend on where the texts differ.
    let (a, b) = (Sha256::digest(a.as_bytes()), Sha256::digest(b.as_bytes()));
    a.iter()
        .zip(b.iter())
        .fold(0_u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

/// A 303 answer with cookies. The answer is never cached.
fn redirect(location: &str, cookies: &[String]) -> ApiResult<Response> {
    let location =
        HeaderValue::from_str(location).map_err(|_| ApiError::new(ErrorCode::Internal))?;
    let mut response = Response::new(axum::body::Body::empty());
    *response.status_mut() = StatusCode::SEE_OTHER;
    let h = response.headers_mut();
    h.insert(axum::http::header::LOCATION, location);
    h.insert(
        axum::http::header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );
    for cookie in cookies {
        let value =
            HeaderValue::from_str(cookie).map_err(|_| ApiError::new(ErrorCode::Internal))?;
        h.append(axum::http::header::SET_COOKIE, value);
    }
    Ok(response)
}

fn not_configured() -> ApiError {
    ApiError::new(ErrorCode::Unavailable)
        .with_errors(vec![FieldIssue::new("provider", "provider_not_configured")])
}

/// The web flow settings of `provider`, if the flow can run.
fn web_provider<'a>(svc: &'a AuthServices, provider: &str) -> Option<&'a WebFlowProvider> {
    let (settings, ready) = if provider == "apple" {
        (
            svc.config.web_apple.as_ref(),
            svc.apple.is_some() && svc.apple_token_client.is_some(),
        )
    } else {
        (svc.config.web_google.as_ref(), svc.google.is_some())
    };
    settings.filter(|_| ready)
}

fn redirect_uri(svc: &AuthServices, provider: &str) -> String {
    format!(
        "{}/v1/auth/{provider}/callback",
        svc.config.web_redirect_base_url
    )
}

fn query_pairs(raw: Option<&str>) -> Vec<(String, String)> {
    url::form_urlencoded::parse(raw.unwrap_or_default().as_bytes())
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect()
}

fn pair<'a>(pairs: &'a [(String, String)], name: &str) -> Option<&'a str> {
    pairs
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.as_str())
        .filter(|v| !v.is_empty() && v.len() <= MAX_PARAM_LEN)
}

// --- step 1: start -------------------------------------------------------------------------

/// Starts a web sign-in with Apple or Google. Answers `303` to the provider authorize page.
///
/// The server keeps `state`, `nonce` and the PKCE verifier (Google) in Redis for 10 minutes and
/// sets the cookie `__Host-oauth_state` (`Secure`, `HttpOnly`, `SameSite=None`). `return_to` is
/// a path in the web app. A value that is not a relative path starts with one `/` gives `/`.
/// Without the provider settings the answer is `503` with the error `provider_not_configured`.
#[utoipa::path(
    get,
    path = "/v1/auth/{provider}/start",
    tag = "auth",
    operation_id = "auth_web_start",
    params(
        ("provider" = String, Path, description = "`apple` or `google`"),
        ("return_to" = Option<String>, Query,
            description = "Path in the web app to open after sign-in (default `/`)"),
        ("remember" = Option<bool>, Query,
            description = "`true`: keep the session across browser restarts")
    ),
    responses((
        status = 303,
        description = "To the provider authorize page. Sets the cookie `__Host-oauth_state`.",
        headers(
            ("Location" = String, description = "Provider authorize address"),
            ("Set-Cookie" = String, description = "`__Host-oauth_state`, 10 minutes")
        )
    ))
)]
pub async fn start(
    Extension(svc): Svc,
    Path(provider): Path<String>,
    RawQuery(query): RawQuery,
) -> ApiResult<Response> {
    let provider = parse_provider(&provider).ok_or_else(|| ApiError::new(ErrorCode::NotFound))?;
    let cfg = web_provider(&svc, provider).ok_or_else(not_configured)?;
    let pairs = query_pairs(query.as_deref());
    let state = new_token();
    let nonce = new_token();
    let binding = new_token();
    let verifier = (provider == "google").then(new_token);
    let record = FlowState {
        provider: provider.to_owned(),
        nonce: nonce.clone(),
        verifier: verifier.clone(),
        binding: binding.clone(),
        return_to: safe_return_to(pair(&pairs, "return_to")),
        remember: pair(&pairs, "remember") == Some("true"),
    };
    let json = serde_json::to_string(&record).map_err(|_| ApiError::new(ErrorCode::Internal))?;
    let key = format!("{KEY_PREFIX}{state}");
    let stored = svc
        .redis
        .run(|mut c| async move {
            redis::cmd("SET")
                .arg(key)
                .arg(json)
                .arg("EX")
                .arg(FLOW_TTL_SECS)
                .query_async::<()>(&mut c)
                .await
        })
        .await;
    if stored.is_none() {
        // No Redis: the state cannot be kept, so the flow cannot run.
        return Err(ApiError::unavailable(30));
    }

    let mut url =
        url::Url::parse(&cfg.authorize_url).map_err(|_| ApiError::new(ErrorCode::Internal))?;
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("response_type", "code");
        q.append_pair("client_id", &cfg.client_id);
        q.append_pair("redirect_uri", &redirect_uri(&svc, provider));
        q.append_pair("state", &state);
        q.append_pair("nonce", &nonce);
        if let Some(verifier) = &verifier {
            q.append_pair("scope", "openid email profile");
            q.append_pair(
                "code_challenge",
                &URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())),
            );
            q.append_pair("code_challenge_method", "S256");
        } else {
            q.append_pair("response_mode", "form_post");
            q.append_pair("scope", "name email");
        }
    }
    redirect(url.as_str(), &[state_cookie(&binding)])
}

// --- step 2: callback ----------------------------------------------------------------------

/// Parameters of a callback.
#[derive(Debug, Default)]
struct CallbackParams {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    user: Option<String>,
}

impl CallbackParams {
    fn from_raw(raw: Option<&[u8]>) -> Self {
        let pairs = query_pairs(raw.map(String::from_utf8_lossy).as_deref());
        let take = |name: &str| pair(&pairs, name).map(str::to_owned);
        Self {
            code: take("code"),
            state: take("state"),
            error: take("error"),
            user: take("user"),
        }
    }
}

/// The name Apple sends in the `user` field (first authorization only).
fn apple_user_name(raw: Option<&str>) -> String {
    let Some(value) = raw.and_then(|r| serde_json::from_str::<serde_json::Value>(r).ok()) else {
        return String::new();
    };
    let part = |k: &str| {
        value["name"][k]
            .as_str()
            .unwrap_or_default()
            .trim()
            .to_owned()
    };
    // The field comes from the browser form, so keep the name short.
    format!("{} {}", part("firstName"), part("lastName"))
        .trim()
        .chars()
        .take(MAX_DISPLAY_NAME)
        .collect()
}

/// The Google token endpoint answer (the part the flow needs).
#[derive(serde::Deserialize)]
struct GoogleTokens {
    id_token: Option<String>,
}

async fn exchange_google(
    svc: &AuthServices,
    cfg: &WebFlowProvider,
    code: &str,
    verifier: &str,
    redirect_uri: &str,
) -> Result<String, &'static str> {
    let secret = cfg
        .client_secret
        .as_deref()
        .ok_or("provider_not_configured")?;
    let body = {
        let mut form = url::form_urlencoded::Serializer::new(String::new());
        form.append_pair("grant_type", "authorization_code");
        form.append_pair("code", code);
        form.append_pair("client_id", &cfg.client_id);
        form.append_pair("client_secret", secret);
        form.append_pair("redirect_uri", redirect_uri);
        form.append_pair("code_verifier", verifier);
        form.finish()
    };
    let response = svc
        .web_http
        .post(&cfg.token_url)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(body)
        .send()
        .await
        .map_err(|_| "provider_unavailable")?;
    let status = response.status();
    if status.is_client_error() {
        // `invalid_grant`: the code is wrong, used or expired.
        return Err("code_invalid");
    }
    if !status.is_success() {
        return Err("provider_unavailable");
    }
    let bytes = response.bytes().await.map_err(|_| "provider_unavailable")?;
    if bytes.len() > MAX_TOKEN_RESPONSE {
        return Err("provider_unavailable");
    }
    serde_json::from_slice::<GoogleTokens>(&bytes)
        .ok()
        .and_then(|t| t.id_token)
        .filter(|t| !t.is_empty())
        .ok_or("provider_unavailable")
}

/// What the code exchange gives: the ID token and, for Apple, the refresh token.
struct Exchanged {
    id_token: String,
    apple_refresh: Option<String>,
}

fn apple_error(e: AppleTokenError) -> &'static str {
    match e {
        AppleTokenError::InvalidCode
        | AppleTokenError::InvalidClientId
        | AppleTokenError::MissingIdToken => "code_invalid",
        other => {
            tracing::warn!(error = ?other, "Apple web code exchange failed");
            "provider_unavailable"
        }
    }
}

fn api_error_code(e: &ApiError) -> &'static str {
    if e.code() == ErrorCode::Unavailable {
        "provider_unavailable"
    } else {
        "sign_in_failed"
    }
}

/// Runs the callback. `Ok` is the target address and the session cookie; `Err` is the error code
/// for `/sign-in?error=`.
async fn complete(
    svc: &AuthServices,
    provider: &'static str,
    params: CallbackParams,
    headers: &HeaderMap,
) -> Result<(String, Option<String>), &'static str> {
    let cfg = web_provider(svc, provider).ok_or("provider_not_configured")?;
    let state = params.state.as_deref().ok_or("invalid_state")?;
    // The state works one time only: GETDEL takes it out whatever happens next.
    let key = format!("{KEY_PREFIX}{state}");
    let taken = svc
        .redis
        .run(|mut c| async move {
            redis::cmd("GETDEL")
                .arg(key)
                .query_async::<Option<String>>(&mut c)
                .await
        })
        .await
        .ok_or("provider_unavailable")?;
    let record: FlowState = taken
        .and_then(|json| serde_json::from_str(&json).ok())
        .ok_or("invalid_state")?;
    let bound = cookie_value(headers, STATE_COOKIE).is_some_and(|c| same(c, &record.binding));
    if record.provider != provider || !bound {
        return Err("invalid_state");
    }
    if let Some(error) = params.error.as_deref() {
        return Err(
            if matches!(error, "access_denied" | "user_cancelled_authorize") {
                "access_denied"
            } else {
                "provider_error"
            },
        );
    }
    let code = params.code.as_deref().ok_or("missing_code")?;

    let redirect_uri = redirect_uri(svc, provider);
    let exchanged = if provider == "apple" {
        let client = svc
            .apple_token_client
            .as_ref()
            .ok_or("provider_not_configured")?;
        let tokens = client
            .exchange_web_code(&cfg.token_url, code, &cfg.client_id, &redirect_uri)
            .await
            .map_err(apple_error)?;
        Exchanged {
            id_token: tokens.id_token,
            apple_refresh: tokens.refresh_token,
        }
    } else {
        let verifier = record.verifier.as_deref().ok_or("invalid_state")?;
        Exchanged {
            id_token: exchange_google(svc, cfg, code, verifier, &redirect_uri).await?,
            apple_refresh: None,
        }
    };

    let claims = svc
        .verify_id_token(provider, &exchanged.id_token)
        .await
        .map_err(|e| api_error_code(&e))?;
    // The ID token must be for the web client and must carry the nonce of this flow.
    if claims.audiences.as_slice() != [cfg.client_id.clone()] {
        return Err("sign_in_failed");
    }
    if !claims
        .nonce
        .as_deref()
        .is_some_and(|n| same(n, &record.nonce))
    {
        return Err("nonce_invalid");
    }
    let mut claim = identity_claim(provider, claims);
    if claim.display_name.is_empty() && provider == "apple" {
        claim.display_name = apple_user_name(params.user.as_deref());
    }
    let refresh = exchanged
        .apple_refresh
        .map(|token| seal_apple_refresh(svc, &cfg.client_id, &token))
        .transpose()
        .map_err(|e| api_error_code(&e))?;

    let device = Device {
        kind: ClientKind::Web,
        remember: record.remember,
        name: "Web browser".to_owned(),
        country: country_of(headers),
    };
    let outcome = svc
        .login(&claim, &device, refresh.as_deref())
        .await
        .map_err(|e| api_error_code(&e))?;
    let cookie = AuthServices::cookie_for(&outcome.record, &outcome.token);
    Ok((
        format!("{}{}", svc.config.web_base_url, record.return_to),
        cookie,
    ))
}

/// Answers a callback: the session and a redirect to the app, or a redirect to `/sign-in`.
async fn finish(
    svc: &AuthServices,
    provider: &'static str,
    params: CallbackParams,
    headers: &HeaderMap,
) -> ApiResult<Response> {
    match complete(svc, provider, params, headers).await {
        Ok((location, session_cookie)) => {
            let mut cookies: Vec<String> = session_cookie.into_iter().collect();
            cookies.push(clear_state_cookie());
            redirect(&location, &cookies)
        }
        Err(code) => {
            tracing::info!(provider, code, "web sign-in callback refused");
            let location = format!(
                "{}/sign-in?error={}",
                svc.config.web_base_url,
                urlencode(code)
            );
            redirect(&location, &[clear_state_cookie()])
        }
    }
}

fn urlencode(value: &str) -> Cow<'_, str> {
    // The codes are fixed ASCII words; this keeps the address valid if one ever is not.
    Cow::Owned(url::form_urlencoded::byte_serialize(value.as_bytes()).collect())
}

/// Callback of Apple (`response_mode=form_post`). Answers `303` in every case.
///
/// The user is in the browser, so an error does not give a JSON body: it gives `303` to
/// `<web base url>/sign-in?error=<code>`. Codes: `invalid_state` (state missing, wrong, used,
/// expired, or not tied to this browser), `access_denied`, `provider_error`, `missing_code`,
/// `code_invalid`, `nonce_invalid`, `sign_in_failed`, `provider_unavailable`,
/// `provider_not_configured`. A good sign-in sets `__Host-session`, clears
/// `__Host-oauth_state` and answers `303` to the `return_to` path.
#[utoipa::path(
    post,
    path = "/v1/auth/apple/callback",
    tag = "auth",
    operation_id = "auth_apple_callback",
    request_body(
        content = AppleCallbackForm,
        content_type = "application/x-www-form-urlencoded"
    ),
    responses((
        status = 303,
        description = "To the web app (signed in) or to `/sign-in?error=<code>`",
        headers(
            ("Location" = String, description = "Web app address"),
            ("Set-Cookie" = String, description = "`__Host-session` (on success) and the cleared `__Host-oauth_state`")
        )
    ))
)]
pub async fn apple_callback(
    Extension(svc): Svc,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Response> {
    finish(
        &svc,
        "apple",
        CallbackParams::from_raw(Some(&body)),
        &headers,
    )
    .await
}

/// Callback of Google (code in the query). Answers `303` in every case.
///
/// The answers are the same as for `POST /v1/auth/apple/callback`.
#[utoipa::path(
    get,
    path = "/v1/auth/google/callback",
    tag = "auth",
    operation_id = "auth_google_callback",
    params(
        ("code" = Option<String>, Query, description = "Authorization code"),
        ("state" = Option<String>, Query, description = "The `state` from the start step"),
        ("error" = Option<String>, Query, description = "Error code of Google")
    ),
    responses((
        status = 303,
        description = "To the web app (signed in) or to `/sign-in?error=<code>`",
        headers(
            ("Location" = String, description = "Web app address"),
            ("Set-Cookie" = String, description = "`__Host-session` (on success) and the cleared `__Host-oauth_state`")
        )
    ))
)]
pub async fn google_callback(
    Extension(svc): Svc,
    headers: HeaderMap,
    RawQuery(query): RawQuery,
) -> ApiResult<Response> {
    finish(
        &svc,
        "google",
        CallbackParams::from_raw(query.as_deref().map(str::as_bytes)),
        &headers,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn return_to_cannot_leave_the_app() {
        for bad in [
            "//evil.example",
            "https://evil.example/",
            "evil.example",
            "/\\evil.example",
            "/\t/evil.example",
            "/a\nb",
            "/caf\u{e9}",
            "",
        ] {
            assert_eq!(safe_return_to(Some(bad)), "/", "{bad:?}");
        }
        assert_eq!(safe_return_to(None), "/");
        assert_eq!(safe_return_to(Some("/recipes/1?x=2#y")), "/recipes/1?x=2#y");
        assert_eq!(safe_return_to(Some(&format!("/{}", "a".repeat(600)))), "/");
    }

    #[test]
    fn apple_name_is_read_from_the_user_field() {
        let raw = r#"{"name":{"firstName":"Sam","lastName":"Brewer"},"email":"a@b.c"}"#;
        assert_eq!(apple_user_name(Some(raw)), "Sam Brewer");
        assert_eq!(apple_user_name(Some("not json")), "");
        assert_eq!(apple_user_name(None), "");
        let long = format!(r#"{{"name":{{"firstName":"{}"}}}}"#, "a".repeat(300));
        assert_eq!(apple_user_name(Some(&long)).chars().count(), MAX_DISPLAY_NAME);
    }
}
