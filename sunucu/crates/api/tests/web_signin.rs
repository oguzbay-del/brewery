//! Web sign-in with the redirect code flow (T3.1): `GET /v1/auth/{provider}/start` and the
//! callbacks, against the fake OIDC provider (authorize and token endpoints). Needs PostgreSQL
//! (`#[sqlx::test]`) and Redis (127.0.0.1:6379 or `REDIS_URL`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::{collections::HashMap, sync::Arc};

use api::{
    ApiConfig, AppState, auth::AuthServices, auth::email::MemoryMailer, redis_client::Redis,
};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use fake_oidc::{Provider, Running};
use sqlx::PgPool;
use support::{Reply, app_pool, redis_url, send};
use uuid::Uuid;

const APPLE_AUD: &str = "com.example.web";
const GOOGLE_AUD: &str = "google-web.example";
const ORIGIN: &str = "https://app.example.com";

struct World {
    app: Router,
    oidc: Running,
}

async fn world(pool: &PgPool, configured: bool) -> World {
    use ring::{
        rand::SystemRandom,
        signature::{self, EcdsaKeyPair},
    };
    let oidc = fake_oidc::spawn_local().await;
    let pkcs8 = EcdsaKeyPair::generate_pkcs8(
        &signature::ECDSA_P256_SHA256_FIXED_SIGNING,
        &SystemRandom::new(),
    )
    .unwrap();
    let private_key = format!(
        "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----",
        STANDARD.encode(pkcs8.as_ref())
    );
    let mut m: HashMap<String, String> = [
        ("DATABASE_URL", "postgres://unused".to_owned()),
        ("TRUST_FORWARDED_FOR", "true".to_owned()),
        ("APPLE_CLIENT_IDS", APPLE_AUD.to_owned()),
        ("APPLE_ISSUER", oidc.oidc.issuer(Provider::Apple)),
        ("APPLE_JWKS_URL", oidc.oidc.jwks_url(Provider::Apple)),
        ("APPLE_TEAM_ID", "TEAM123".to_owned()),
        ("APPLE_KEY_ID", "KEY456".to_owned()),
        ("APPLE_PRIVATE_KEY", private_key),
        ("PROVIDER_TOKEN_KEY", STANDARD.encode([7_u8; 32])),
        ("GOOGLE_CLIENT_IDS", GOOGLE_AUD.to_owned()),
        ("GOOGLE_ISSUER", oidc.oidc.issuer(Provider::Google)),
        ("GOOGLE_JWKS_URL", oidc.oidc.jwks_url(Provider::Google)),
        ("WEB_BASE_URL", ORIGIN.to_owned()),
        ("CSRF_HMAC_KEY", "test-csrf-key-0123456789".to_owned()),
        ("JWKS_MIN_REFRESH_SECS", "0".to_owned()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v))
    .collect();
    if configured {
        for (k, v) in [
            ("APPLE_WEB_CLIENT_ID", APPLE_AUD.to_owned()),
            (
                "APPLE_AUTHORIZE_URL",
                format!("{}/apple/authorize", oidc.url),
            ),
            ("APPLE_WEB_TOKEN_URL", format!("{}/apple/token", oidc.url)),
            ("GOOGLE_WEB_CLIENT_ID", GOOGLE_AUD.to_owned()),
            ("GOOGLE_CLIENT_SECRET", "google-secret".to_owned()),
            (
                "GOOGLE_AUTHORIZE_URL",
                format!("{}/google/authorize", oidc.url),
            ),
            ("GOOGLE_WEB_TOKEN_URL", format!("{}/google/token", oidc.url)),
        ] {
            m.insert(k.to_owned(), v);
        }
    }
    let cfg = ApiConfig::from_lookup(|n| m.get(n).cloned()).unwrap();
    let db = app_pool(pool).await;
    let redis = Redis::connect_lazy(&redis_url()).unwrap();
    let svc = Arc::new(
        AuthServices::new(
            db.clone(),
            redis.clone(),
            cfg.auth.clone(),
            Arc::new(MemoryMailer::default()),
        )
        .unwrap(),
    );
    let state = AppState::new(db, redis, svc.clone(), cfg).with_auth_services(svc);
    let app = api::build(state.clone(), Router::new());
    World { app, oidc }
}

async fn call(
    w: &World,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<(&str, String)>,
) -> Reply {
    let mut b = Request::builder()
        .method(method)
        .uri(path)
        .header("x-forwarded-for", Uuid::now_v7().to_string());
    for (k, v) in headers {
        b = b.header(*k, *v);
    }
    let body = match body {
        Some((ct, text)) => {
            b = b.header(header::CONTENT_TYPE, ct);
            Body::from(text)
        }
        None => Body::empty(),
    };
    send(&w.app, b.body(body).unwrap()).await
}

fn set_cookies(r: &Reply) -> Vec<String> {
    r.headers
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|v| v.to_str().unwrap().to_owned())
        .collect()
}

fn cookie_named(r: &Reply, name: &str) -> Option<String> {
    set_cookies(r)
        .into_iter()
        .find(|c| c.starts_with(&format!("{name}=")))
}

fn cookie_pair(cookie: &str) -> String {
    cookie.split(';').next().unwrap().to_owned()
}

fn query_of(url: &str) -> HashMap<String, String> {
    url::Url::parse(url)
        .unwrap()
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect()
}

/// A started flow: the state, the browser cookie and the provider address.
struct Started {
    state: String,
    cookie: String,
    authorize: String,
}

async fn start(w: &World, provider: &str, query: &str) -> Started {
    let r = call(
        w,
        "GET",
        &format!("/v1/auth/{provider}/start{query}"),
        &[],
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::SEE_OTHER, "{:?}", r.body);
    let authorize = r.header("location").to_owned();
    let cookie = cookie_pair(&cookie_named(&r, "__Host-oauth_state").expect("state cookie"));
    Started {
        state: query_of(&authorize)["state"].clone(),
        cookie,
        authorize,
    }
}

/// Lets the fake provider sign the user in. Gives the callback code.
async fn provider_signs_in(w: &World, s: &Started, hints: &str) -> String {
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let response = http
        .get(format!("{}{hints}", s.authorize))
        .send()
        .await
        .unwrap();
    let _ = w;
    if response.status() == StatusCode::SEE_OTHER {
        let location = response.headers()[header::LOCATION]
            .to_str()
            .unwrap()
            .to_owned();
        assert_eq!(query_of(&location)["state"], s.state);
        return query_of(&location)["code"].clone();
    }
    let html = response.text().await.unwrap();
    let marker = "name=\"code\" value=\"";
    let at = html.find(marker).expect("code field") + marker.len();
    html[at..].split('"').next().unwrap().to_owned()
}

async fn google_callback(w: &World, state: &str, code: &str, cookie: Option<&str>) -> Reply {
    let path = format!("/v1/auth/google/callback?code={code}&state={state}");
    let headers: Vec<(&str, &str)> = cookie.map(|c| ("cookie", c)).into_iter().collect();
    call(w, "GET", &path, &headers, None).await
}

async fn apple_callback(w: &World, form: String, cookie: Option<&str>) -> Reply {
    let headers: Vec<(&str, &str)> = cookie.map(|c| ("cookie", c)).into_iter().collect();
    call(
        w,
        "POST",
        "/v1/auth/apple/callback",
        &headers,
        Some(("application/x-www-form-urlencoded", form)),
    )
    .await
}

fn assert_error_redirect(r: &Reply, code: &str) {
    assert_eq!(r.status, StatusCode::SEE_OTHER, "{:?}", r.body);
    assert_eq!(
        r.header("location"),
        format!("{ORIGIN}/sign-in?error={code}")
    );
    assert!(cookie_named(r, "__Host-session").is_none());
    assert!(
        cookie_named(r, "__Host-oauth_state")
            .unwrap()
            .contains("Max-Age=0")
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn start_sets_the_cookie_and_redirects_to_the_provider(pool: PgPool) {
    let w = world(&pool, true).await;
    let r = call(&w, "GET", "/v1/auth/google/start", &[], None).await;
    assert_eq!(r.status, StatusCode::SEE_OTHER);
    let cookie = cookie_named(&r, "__Host-oauth_state").unwrap();
    for part in [
        "HttpOnly",
        "Secure",
        "SameSite=None",
        "Path=/",
        "Max-Age=600",
    ] {
        assert!(cookie.contains(part), "{cookie}");
    }
    let q = query_of(r.header("location"));
    assert!(r.header("location").starts_with(&w.oidc.url));
    assert_eq!(q["client_id"], GOOGLE_AUD);
    assert_eq!(
        q["redirect_uri"],
        format!("{ORIGIN}/v1/auth/google/callback")
    );
    assert_eq!(q["code_challenge_method"], "S256");
    assert!(q["state"].len() >= 22 && q["nonce"].len() >= 22);
    assert!(q.contains_key("code_challenge"));
    // The per-IP rate limit covers the new path.
    assert!(!r.header("ratelimit").is_empty());

    let a = call(&w, "GET", "/v1/auth/apple/start", &[], None).await;
    let q = query_of(a.header("location"));
    assert_eq!(q["response_mode"], "form_post");
    assert_eq!(q["scope"], "name email");
    assert!(!q.contains_key("code_challenge"));

    let unknown = call(&w, "GET", "/v1/auth/github/start", &[], None).await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn without_settings_the_endpoints_answer_503(pool: PgPool) {
    let w = world(&pool, false).await;
    for provider in ["apple", "google"] {
        let r = call(&w, "GET", &format!("/v1/auth/{provider}/start"), &[], None).await;
        assert_eq!(r.status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(r.body["errors"][0]["code"], "provider_not_configured");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_good_google_callback_signs_in_and_redirects(pool: PgPool) {
    let w = world(&pool, true).await;
    let s = start(&w, "google", "?return_to=/recipes%3Fx%3D1&remember=true").await;
    let code = provider_signs_in(&w, &s, "&fake_sub=g-1&fake_email=sam%40example.com").await;
    let r = google_callback(&w, &s.state, &code, Some(&s.cookie)).await;
    assert_eq!(
        r.status,
        StatusCode::SEE_OTHER,
        "{:?}",
        r.header("location")
    );
    assert_eq!(r.header("location"), format!("{ORIGIN}/recipes?x=1"));
    let session = cookie_named(&r, "__Host-session").expect("session cookie");
    assert!(session.contains("HttpOnly") && session.contains("Secure"));
    assert!(session.contains("Max-Age="), "remember=true: {session}");
    assert!(
        cookie_named(&r, "__Host-oauth_state")
            .unwrap()
            .contains("Max-Age=0")
    );
    let n: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM user_identities WHERE provider = 'google' AND subject = 'g-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, 1);
    // The cookie is a real session.
    let me = call(
        &w,
        "GET",
        "/v1/me/session",
        &[("cookie", &cookie_pair(&session))],
        None,
    )
    .await;
    assert_eq!(me.status, StatusCode::OK, "{:?}", me.body);
    assert_eq!(me.body["session"]["client"], "web");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_good_apple_callback_signs_in_and_keeps_the_name(pool: PgPool) {
    let w = world(&pool, true).await;
    let s = start(&w, "apple", "?return_to=/settings").await;
    let code = provider_signs_in(&w, &s, "&fake_sub=a-1&fake_email=sam%40example.com").await;
    let user = r#"{"name":{"firstName":"Sam","lastName":"Brewer"}}"#;
    let form = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("code", &code)
        .append_pair("state", &s.state)
        .append_pair("user", user)
        .finish();
    let r = apple_callback(&w, form, Some(&s.cookie)).await;
    assert_eq!(
        r.status,
        StatusCode::SEE_OTHER,
        "{:?}",
        r.header("location")
    );
    assert_eq!(r.header("location"), format!("{ORIGIN}/settings"));
    assert!(cookie_named(&r, "__Host-session").is_some());
    let (name, enc): (String, Option<Vec<u8>>) = sqlx::query_as(
        "SELECT u.display_name, i.provider_refresh_token_enc FROM users u \
         JOIN user_identities i ON i.user_id = u.id WHERE i.provider = 'apple' AND i.subject = 'a-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(name, "Sam Brewer");
    assert!(enc.is_some(), "the Apple refresh token is stored encrypted");
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_open_redirect_target_goes_to_the_root(pool: PgPool) {
    let w = world(&pool, true).await;
    for bad in ["//evil.example", "https://evil.example/", "/%5Cevil"] {
        let s = start(
            &w,
            "google",
            &format!("?return_to={}", bad.replace('/', "%2F")),
        )
        .await;
        let code = provider_signs_in(&w, &s, "").await;
        let r = google_callback(&w, &s.state, &code, Some(&s.cookie)).await;
        let want = format!("{ORIGIN}/");
        assert_eq!(r.header("location"), want, "{bad}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn bad_missing_used_and_unbound_states_are_refused(pool: PgPool) {
    let w = world(&pool, true).await;
    // A state the server never made.
    let s = start(&w, "google", "").await;
    let code = provider_signs_in(&w, &s, "").await;
    let r = google_callback(&w, "not-a-state", &code, Some(&s.cookie)).await;
    assert_error_redirect(&r, "invalid_state");
    // No state at all.
    let r = call(&w, "GET", "/v1/auth/google/callback?code=x", &[], None).await;
    assert_error_redirect(&r, "invalid_state");

    // No cookie: the flow is not tied to this browser. The state is used up.
    let r = google_callback(&w, &s.state, &code, None).await;
    assert_error_redirect(&r, "invalid_state");
    let r = google_callback(&w, &s.state, &code, Some(&s.cookie)).await;
    assert_error_redirect(&r, "invalid_state");

    // A cookie of another flow.
    let s1 = start(&w, "google", "").await;
    let s2 = start(&w, "google", "").await;
    let code1 = provider_signs_in(&w, &s1, "").await;
    let r = google_callback(&w, &s1.state, &code1, Some(&s2.cookie)).await;
    assert_error_redirect(&r, "invalid_state");

    // A used state: the first call works, the second gives the error.
    let s3 = start(&w, "google", "").await;
    let code3 = provider_signs_in(&w, &s3, "").await;
    let first = google_callback(&w, &s3.state, &code3, Some(&s3.cookie)).await;
    assert!(cookie_named(&first, "__Host-session").is_some());
    let again = google_callback(&w, &s3.state, &code3, Some(&s3.cookie)).await;
    assert_error_redirect(&again, "invalid_state");

    // A state of the other provider.
    let sa = start(&w, "apple", "").await;
    let codea = provider_signs_in(&w, &sa, "").await;
    let r = google_callback(&w, &sa.state, &codea, Some(&sa.cookie)).await;
    assert_error_redirect(&r, "invalid_state");

    // An expired state (Redis dropped it).
    let s4 = start(&w, "google", "").await;
    let code4 = provider_signs_in(&w, &s4, "").await;
    let redis = redis::Client::open(redis_url()).unwrap();
    let mut conn = redis.get_multiplexed_async_connection().await.unwrap();
    redis::cmd("DEL")
        .arg(format!("webauth:state:{}", s4.state))
        .query_async::<()>(&mut conn)
        .await
        .unwrap();
    let r = google_callback(&w, &s4.state, &code4, Some(&s4.cookie)).await;
    assert_error_redirect(&r, "invalid_state");
}

#[sqlx::test(migrations = "../../migrations")]
async fn provider_errors_and_bad_codes_go_back_to_sign_in(pool: PgPool) {
    let w = world(&pool, true).await;
    let s = start(&w, "google", "").await;
    let path = format!(
        "/v1/auth/google/callback?error=access_denied&state={}",
        s.state
    );
    let r = call(&w, "GET", &path, &[("cookie", &s.cookie)], None).await;
    assert_error_redirect(&r, "access_denied");

    let s = start(&w, "google", "").await;
    let r = google_callback(&w, &s.state, "forged-code", Some(&s.cookie)).await;
    assert_error_redirect(&r, "code_invalid");

    // The PKCE verifier is checked by the provider: a code issued for another challenge fails.
    let s = start(&w, "google", "").await;
    let other = w.oidc.oidc.issue_code(
        GOOGLE_AUD,
        None,
        Some("some-other-challenge".to_owned()),
        fake_oidc::MintSpec::default(),
    );
    let r = google_callback(&w, &s.state, &other, Some(&s.cookie)).await;
    assert_error_redirect(&r, "code_invalid");

    // An ID token with the wrong nonce.
    let s = start(&w, "google", "").await;
    let forged = w.oidc.oidc.issue_code(
        GOOGLE_AUD,
        Some("not-the-nonce".to_owned()),
        None,
        fake_oidc::MintSpec::default(),
    );
    let r = google_callback(&w, &s.state, &forged, Some(&s.cookie)).await;
    assert_error_redirect(&r, "nonce_invalid");
}
