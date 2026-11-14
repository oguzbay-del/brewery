//! End to end tests of T2.11 (ADR-0009 rules 1 to 4): sign-in with fake Apple and Google tokens,
//! the e-mail link, sessions, CSRF, identity linking and the first workspace. Needs PostgreSQL
//! (`#[sqlx::test]`), the compose Redis (127.0.0.1:6379) and, for one test, mailpit (SMTP 1025,
//! UI 8025).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::items_after_statements, clippy::semicolon_if_nothing_returned)]

mod support;

use std::{collections::HashMap, sync::Arc};

use api::{
    ApiConfig, AppState, Authenticator, RequestContext,
    auth::{
        AuthServices,
        email::{Mail, MemoryMailer, SmtpMailer},
        tokens::hex,
    },
    redis_client::Redis,
};
use axum::{
    Json, Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::get,
};
use fake_oidc::{KeyChoice, MintSpec, Provider, Running};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use support::{RedisProxy, Reply, app_pool, redis_url, send};
use uuid::Uuid;

const APPLE_AUD: &str = "com.example.brewery";
const GOOGLE_AUD: &str = "google-client.example";
const ORIGIN: &str = "https://app.example.com";

struct World {
    app: Router,
    oidc: Running,
    mail: Arc<MemoryMailer>,
    svc: Arc<AuthServices>,
}

fn lookup_for(oidc: &Running, extra: &[(&str, &str)]) -> HashMap<String, String> {
    let mut m: HashMap<String, String> = [
        ("DATABASE_URL", "postgres://unused".to_owned()),
        ("TRUST_FORWARDED_FOR", "true".to_owned()),
        ("APPLE_CLIENT_IDS", APPLE_AUD.to_owned()),
        ("APPLE_ISSUER", oidc.oidc.issuer(Provider::Apple)),
        ("APPLE_JWKS_URL", oidc.oidc.jwks_url(Provider::Apple)),
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
    for (k, v) in extra {
        m.insert((*k).to_owned(), (*v).to_owned());
    }
    m
}

async fn world_with(pool: &PgPool, redis: Redis, extra: &[(&str, &str)]) -> World {
    let oidc = fake_oidc::spawn_local().await;
    let map = lookup_for(&oidc, extra);
    let cfg = ApiConfig::from_lookup(|n| map.get(n).cloned()).unwrap();
    let app_db = app_pool(pool).await;
    let mail = Arc::new(MemoryMailer::default());
    let svc = Arc::new(AuthServices::new(
        app_db.clone(),
        redis.clone(),
        cfg.auth.clone(),
        mail.clone(),
    ));
    let state = AppState::new(app_db, redis, svc.clone(), cfg).with_auth_services(svc.clone());
    // A probe route shows the permissions the authenticator found for the path's workspace.
    let probe = api::authenticated(
        &state,
        Router::new().route(
            "/v1/workspaces/{ws}/probe",
            get(|ctx: RequestContext| async move { Json(json!({"permissions": ctx.permissions})) }),
        ),
    );
    let app = api::build(state.clone(), probe);
    World {
        app,
        oidc,
        mail,
        svc,
    }
}

async fn world(pool: &PgPool) -> World {
    world_with(pool, Redis::disabled(), &[]).await
}

// --- request helpers -------------------------------------------------------------------------

fn request(
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<&Value>,
) -> Request<Body> {
    let mut b = Request::builder().method(method).uri(path);
    if !headers
        .iter()
        .any(|(k, _)| k.eq_ignore_ascii_case("x-forwarded-for"))
    {
        b = b.header("x-forwarded-for", Uuid::new_v4().to_string());
    }
    for (k, v) in headers {
        b = b.header(*k, *v);
    }
    match body {
        Some(v) => b
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(v.to_string()))
            .unwrap(),
        None => b.body(Body::empty()).unwrap(),
    }
}

async fn call(
    w: &World,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<Value>,
) -> Reply {
    send(&w.app, request(method, path, headers, body.as_ref())).await
}

fn bearer(token: &str) -> [(&'static str, String); 1] {
    [("authorization", format!("Bearer {token}"))]
}

async fn get_as(w: &World, path: &str, token: &str) -> Reply {
    let h = bearer(token);
    call(w, "GET", path, &[(h[0].0, h[0].1.as_str())], None).await
}

async fn nonce(w: &World) -> String {
    let r = call(w, "POST", "/v1/auth/nonce", &[], None).await;
    assert_eq!(r.status, StatusCode::CREATED, "{:?}", r.body);
    r.body["nonce"].as_str().unwrap().to_owned()
}

fn apple_nonce_claim(raw: &str) -> String {
    hex(&Sha256::digest(raw.as_bytes()))
}

fn spec(sub: &str, aud: &str, nonce: Option<String>) -> MintSpec {
    MintSpec {
        sub: sub.to_owned(),
        aud: aud.to_owned(),
        nonce,
        ..MintSpec::default()
    }
}

/// Apple sign-in with a fresh nonce; `tweak` adjusts the minted token.
async fn apple(w: &World, sub: &str, client: &str, tweak: impl FnOnce(&mut MintSpec)) -> Reply {
    let n = nonce(w).await;
    let mut s = spec(sub, APPLE_AUD, Some(apple_nonce_claim(&n)));
    tweak(&mut s);
    let token = w.oidc.oidc.mint(Provider::Apple, &s);
    call(
        w,
        "POST",
        "/v1/auth/apple",
        &[],
        Some(json!({"id_token": token, "nonce": n, "client": client})),
    )
    .await
}

async fn google(w: &World, sub: &str, client: &str, tweak: impl FnOnce(&mut MintSpec)) -> Reply {
    let n = nonce(w).await;
    let mut s = spec(sub, GOOGLE_AUD, Some(n.clone()));
    tweak(&mut s);
    let token = w.oidc.oidc.mint(Provider::Google, &s);
    call(
        w,
        "POST",
        "/v1/auth/google",
        &[],
        Some(json!({"id_token": token, "nonce": n, "client": client})),
    )
    .await
}

fn token_of(r: &Reply) -> String {
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
    r.body["session"]["token"].as_str().unwrap().to_owned()
}

fn first_error_code(r: &Reply) -> &str {
    r.body["errors"][0]["code"].as_str().unwrap_or_default()
}

fn link_from(mail: &Mail) -> String {
    let start = mail.text.find("https://").unwrap();
    mail.text[start..]
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned()
}

fn token_from_link(link: &str) -> String {
    link.split("token=").nth(1).unwrap().to_owned()
}

/// Requests a mailed link for `email` and returns its token.
async fn email_token(w: &World, email: &str) -> String {
    let r = call(
        w,
        "POST",
        "/v1/auth/email/start",
        &[],
        Some(json!({"email": email, "client": "ios"})),
    )
    .await;
    assert_eq!(r.status, StatusCode::ACCEPTED, "{:?}", r.body);
    let sent = w.mail.sent();
    let mail = sent
        .iter()
        .rev()
        .find(|m| m.to == email.to_lowercase())
        .expect("mail sent");
    token_from_link(&link_from(mail))
}

async fn email_login(w: &World, email: &str, client: &str) -> Reply {
    let token = email_token(w, email).await;
    call(
        w,
        "POST",
        "/v1/auth/email/verify",
        &[],
        Some(json!({"token": token, "client": client})),
    )
    .await
}

fn unique_email() -> String {
    format!("brewer-{}@example.com", Uuid::new_v4().simple())
}

// --- token checks (ADR-0009 rule 1.3) ---------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn bad_tokens_are_refused_for_both_providers(pool: PgPool) {
    let w = world(&pool).await;
    type Tweak = fn(&mut MintSpec);
    let cases: Vec<(&str, &str, Tweak)> = vec![
        ("bad_signature", "wrong key", |s| {
            s.signer = KeyChoice::Wrong
        }),
        ("unknown_key", "unknown kid", |s| {
            s.kid = Some("nope".into())
        }),
        ("unsupported_alg", "alg none", |s| {
            s.alg = Some("none".into());
            s.unsigned = true;
        }),
        ("unsupported_alg", "alg HS256", |s| {
            s.alg = Some("HS256".into())
        }),
        ("bad_issuer", "wrong iss", |s| {
            s.iss = Some("https://evil.example".into())
        }),
        ("bad_audience", "wrong aud", |s| {
            s.aud = "someone-else".into()
        }),
        ("token_expired", "expired", |s| s.exp_in_secs = -120),
        ("token_not_yet_valid", "issued in the future", |s| {
            s.iat_offset_secs = 3600
        }),
        ("nonce_invalid", "wrong nonce", |s| {
            s.nonce = Some("not-the-nonce".into())
        }),
        ("nonce_invalid", "no nonce", |s| s.nonce = None),
        ("malformed_token", "unsigned", |s| s.unsigned = true),
    ];
    for (code, label, tweak) in cases {
        let r = apple(&w, "u-apple", "ios", tweak).await;
        assert_eq!(
            r.status,
            StatusCode::UNAUTHORIZED,
            "apple {label}: {:?}",
            r.body
        );
        let got = first_error_code(&r);
        // An empty signature fails as a bad signature or a malformed token; accept both there.
        assert!(
            got == code || (label == "unsigned" && got == "bad_signature"),
            "apple {label}: wanted {code}, got {got}"
        );
        let r = google(&w, "u-google", "ios", tweak).await;
        assert_eq!(
            r.status,
            StatusCode::UNAUTHORIZED,
            "google {label}: {:?}",
            r.body
        );
    }
    // Nothing was created by any refused attempt.
    let users: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(users, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn clock_skew_of_a_minute_is_tolerated(pool: PgPool) {
    let w = world(&pool).await;
    let r = apple(&w, "u1", "ios", |s| s.exp_in_secs = -30).await;
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
    let r = google(&w, "u2", "ios", |s| s.iat_offset_secs = 30).await;
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_nonce_works_once(pool: PgPool) {
    let w = world(&pool).await;
    let n = nonce(&w).await;
    let token = w
        .oidc
        .oidc
        .mint(Provider::Google, &spec("u1", GOOGLE_AUD, Some(n.clone())));
    let body = json!({"id_token": token, "nonce": n, "client": "ios"});
    let first = call(&w, "POST", "/v1/auth/google", &[], Some(body.clone())).await;
    assert_eq!(first.status, StatusCode::OK);
    // The same token and nonce again (replay).
    let again = call(&w, "POST", "/v1/auth/google", &[], Some(body)).await;
    assert_eq!(again.status, StatusCode::UNAUTHORIZED);
    assert_eq!(first_error_code(&again), "nonce_invalid");
    // A nonce the server never issued.
    let forged = "f".repeat(43);
    let token = w.oidc.oidc.mint(
        Provider::Google,
        &spec("u1", GOOGLE_AUD, Some(forged.clone())),
    );
    let r = call(
        &w,
        "POST",
        "/v1/auth/google",
        &[],
        Some(json!({"id_token": token, "nonce": forged})),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    // An expired nonce.
    let n2 = nonce(&w).await;
    sqlx::query(
        "UPDATE auth_requests SET expires_at = now() - interval '1 minute' WHERE nonce = $1",
    )
    .bind(&n2)
    .execute(&pool)
    .await
    .unwrap();
    let token = w
        .oidc
        .oidc
        .mint(Provider::Google, &spec("u1", GOOGLE_AUD, Some(n2.clone())));
    let r = call(
        &w,
        "POST",
        "/v1/auth/google",
        &[],
        Some(json!({"id_token": token, "nonce": n2})),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn jwks_is_refreshed_for_new_keys_but_not_flooded(pool: PgPool) {
    let w = world(&pool).await;
    // Prime the cache with the first key.
    assert_eq!(apple(&w, "u1", "ios", |_| {}).await.status, StatusCode::OK);
    let n = nonce(&w).await;
    let old = w.oidc.oidc.mint(
        Provider::Apple,
        &spec("u1", APPLE_AUD, Some(apple_nonce_claim(&n))),
    );
    w.oidc.oidc.rotate(Provider::Apple);
    // A token of the new key makes the verifier refresh and succeed.
    assert_eq!(apple(&w, "u1", "ios", |_| {}).await.status, StatusCode::OK);
    // A token of the retired key no longer verifies.
    let r = call(
        &w,
        "POST",
        "/v1/auth/apple",
        &[],
        Some(json!({"id_token": old, "nonce": n})),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    assert_eq!(first_error_code(&r), "unknown_key");

    // With a refresh interval, forged key ids cannot make the API fetch the JWKS repeatedly.
    let slow = world_with(
        &pool,
        Redis::disabled(),
        &[("JWKS_MIN_REFRESH_SECS", "3600")],
    )
    .await;
    assert_eq!(
        apple(&slow, "u9", "ios", |_| {}).await.status,
        StatusCode::OK
    );
    slow.oidc.oidc.rotate(Provider::Apple);
    let r = apple(&slow, "u9", "ios", |_| {}).await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    assert_eq!(first_error_code(&r), "unknown_key");
}

#[sqlx::test(migrations = "../../migrations")]
async fn unreachable_jwks_is_a_503_and_unconfigured_provider_a_404(pool: PgPool) {
    let w = world_with(
        &pool,
        Redis::disabled(),
        &[("GOOGLE_JWKS_URL", "http://127.0.0.1:1/jwks")],
    )
    .await;
    let r = google(&w, "u1", "ios", |_| {}).await;
    assert_eq!(r.status, StatusCode::SERVICE_UNAVAILABLE, "{:?}", r.body);

    let w = world_with(&pool, Redis::disabled(), &[("APPLE_CLIENT_IDS", "")]).await;
    let n = nonce(&w).await;
    let r = call(
        &w,
        "POST",
        "/v1/auth/apple",
        &[],
        Some(json!({"id_token": "a.b.c", "nonce": n})),
    )
    .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn apple_email_as_string_and_name_are_read(pool: PgPool) {
    let w = world(&pool).await;
    let email = unique_email();
    let r = apple(&w, "u1", "ios", |s| {
        s.email = Some(email.to_uppercase());
        s.email_verified = Some(true);
        s.email_verified_as_string = true;
        s.name = Some("Sam Brewer".into());
    })
    .await;
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
    assert_eq!(r.body["user"]["contact_email"], email);
    assert_eq!(r.body["user"]["display_name"], "Sam Brewer");
    // An unverified address is not kept.
    let r = google(&w, "u2", "ios", |s| {
        s.email = Some(unique_email());
        s.email_verified = Some(false);
    })
    .await;
    assert_eq!(r.body["user"]["contact_email"], Value::Null);
}

// --- first sign-in and sessions ---------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn first_sign_in_creates_user_identity_and_session_then_finds_them(pool: PgPool) {
    let w = world(&pool).await;
    let r = apple(&w, "apple-1", "ios", |_| {}).await;
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
    assert_eq!(r.body["is_new_user"], true);
    assert_eq!(r.body["account_hint"], Value::Null);
    assert_eq!(r.body["session"]["client"], "ios");
    assert_eq!(r.body["session"]["csrf_token"], Value::Null);
    assert_eq!(r.header("cache-control"), "no-store");
    assert!(r.headers.get(header::SET_COOKIE).is_none());
    let token = token_of(&r);
    let user_id = r.body["user"]["id"].as_str().unwrap().to_owned();

    // Only the digest is stored.
    let stored: Vec<u8> = sqlx::query_scalar("SELECT token_hash FROM sessions")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored.len(), 32);
    assert!(
        !stored
            .windows(8)
            .any(|w| token.as_bytes().windows(8).any(|t| t == w))
    );
    let identities: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM user_identities WHERE provider = 'apple' AND subject = 'apple-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(identities, 1);

    let me = get_as(&w, "/v1/me/session", &token).await;
    assert_eq!(me.status, StatusCode::OK, "{:?}", me.body);
    assert_eq!(me.body["user"]["id"], user_id.as_str());
    assert_eq!(me.body["workspaces"], json!([]));
    assert_eq!(me.body["session"]["csrf_token"], Value::Null);

    // The same Apple user signing in again is the same account.
    let again = apple(&w, "apple-1", "ios", |_| {}).await;
    assert_eq!(again.body["is_new_user"], false);
    assert_eq!(again.body["user"]["id"], user_id.as_str());
    let users: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(users, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn requests_without_or_with_a_wrong_token_are_401(pool: PgPool) {
    let w = world(&pool).await;
    assert_eq!(
        call(&w, "GET", "/v1/me/session", &[], None).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        get_as(&w, "/v1/me/session", "garbage").await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        get_as(&w, "/v1/me/session", &"a".repeat(500)).await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn web_session_uses_a_cookie_and_csrf(pool: PgPool) {
    let w = world(&pool).await;
    let r = google(&w, "g1", "web", |_| {}).await;
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
    assert_eq!(r.body["session"]["token"], Value::Null);
    let csrf = r.body["session"]["csrf_token"].as_str().unwrap().to_owned();
    let cookie = r.header("set-cookie").to_owned();
    for part in [
        "__Host-session=",
        "HttpOnly",
        "Secure",
        "SameSite=Lax",
        "Path=/",
    ] {
        assert!(cookie.contains(part), "{cookie}");
    }
    assert!(!cookie.contains("Domain"), "{cookie}");
    assert!(
        !cookie.contains("Max-Age"),
        "a browser-session cookie: {cookie}"
    );
    let pair = cookie.split(';').next().unwrap().to_owned();

    // Reading needs no CSRF token.
    let me = call(
        &w,
        "GET",
        "/v1/me/session",
        &[("cookie", pair.as_str())],
        None,
    )
    .await;
    assert_eq!(me.status, StatusCode::OK, "{:?}", me.body);
    assert_eq!(me.body["session"]["csrf_token"], csrf.as_str());

    let ws = json!({"name": "W", "mode": "home", "region": "us", "country": "US", "time_zone": "America/New_York"});
    let write =
        |origin: Option<&'static str>, token: Option<String>| -> Vec<(&'static str, String)> {
            let mut h: Vec<(&'static str, String)> = vec![
                ("cookie", pair.clone()),
                ("idempotency-key", Uuid::new_v4().to_string()),
            ];
            if let Some(o) = origin {
                h.push(("origin", o.to_owned()));
            }
            if let Some(t) = token {
                h.push(("x-csrf-token", t));
            }
            h
        };
    let post = |h: Vec<(&'static str, String)>| {
        let w = &w;
        let ws = ws.clone();
        async move {
            let refs: Vec<(&str, &str)> = h.iter().map(|(k, v)| (*k, v.as_str())).collect();
            call(w, "POST", "/v1/workspaces", &refs, Some(ws)).await
        }
    };
    let missing_token = post(write(Some(ORIGIN), None)).await;
    assert_eq!(
        (missing_token.status, first_error_code(&missing_token)),
        (StatusCode::FORBIDDEN, "csrf_invalid")
    );
    let wrong_token = post(write(Some(ORIGIN), Some("AAAA".into()))).await;
    assert_eq!(first_error_code(&wrong_token), "csrf_invalid");
    let wrong_origin = post(write(Some("https://evil.example"), Some(csrf.clone()))).await;
    assert_eq!(
        (wrong_origin.status, first_error_code(&wrong_origin)),
        (StatusCode::FORBIDDEN, "origin_not_allowed")
    );
    let no_origin = post(write(None, Some(csrf.clone()))).await;
    assert_eq!(first_error_code(&no_origin), "origin_not_allowed");
    let ok = post(write(Some(ORIGIN), Some(csrf.clone()))).await;
    assert_eq!(ok.status, StatusCode::CREATED, "{:?}", ok.body);

    // A token of another session does not fit.
    let other = google(&w, "g2", "web", |_| {}).await;
    let other_csrf = other.body["session"]["csrf_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let cross = post(write(Some(ORIGIN), Some(other_csrf))).await;
    assert_eq!(first_error_code(&cross), "csrf_invalid");
}

#[sqlx::test(migrations = "../../migrations")]
async fn bearer_writes_need_no_csrf_and_cookie_plus_bearer_is_refused(pool: PgPool) {
    let w = world(&pool).await;
    let ios = apple(&w, "a1", "ios", |_| {}).await;
    let token = token_of(&ios);
    let ws = json!({"name": "W", "mode": "business", "region": "uk", "country": "GB", "time_zone": "Europe/London"});
    let b = bearer(&token);
    let key = Uuid::new_v4().to_string();
    let r = call(
        &w,
        "POST",
        "/v1/workspaces",
        &[(b[0].0, b[0].1.as_str()), ("idempotency-key", key.as_str())],
        Some(ws),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{:?}", r.body);

    let web = google(&w, "g1", "web", |_| {}).await;
    let cookie = web
        .header("set-cookie")
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let both = call(
        &w,
        "GET",
        "/v1/me/session",
        &[(b[0].0, b[0].1.as_str()), ("cookie", cookie.as_str())],
        None,
    )
    .await;
    assert_eq!(both.status, StatusCode::BAD_REQUEST, "{:?}", both.body);
    assert_eq!(first_error_code(&both), "ambiguous_credentials");
}

#[sqlx::test(migrations = "../../migrations")]
async fn remembered_web_sessions_get_a_max_age_and_devices_are_named(pool: PgPool) {
    let w = world(&pool).await;
    let n = nonce(&w).await;
    let token = w
        .oidc
        .oidc
        .mint(Provider::Google, &spec("g1", GOOGLE_AUD, Some(n.clone())));
    let r = call(
        &w,
        "POST",
        "/v1/auth/google",
        &[("cf-ipcountry", "GB")],
        Some(json!({"id_token": token, "nonce": n, "client": "web", "remember": true, "device_name": "Kitchen laptop"})),
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
    let cookie = r.header("set-cookie").to_owned();
    assert!(cookie.contains("Max-Age="), "{cookie}");
    let pair = cookie.split(';').next().unwrap().to_owned();
    let list = call(
        &w,
        "GET",
        "/v1/me/sessions",
        &[("cookie", pair.as_str())],
        None,
    )
    .await;
    let item = &list.body["items"][0];
    assert_eq!(item["device_name"], "Kitchen laptop");
    assert_eq!(item["approx_location"], "GB");
    assert_eq!(item["client"], "web");
    assert_eq!(item["current"], true);
    // Too long a device name is a validation error.
    let n = nonce(&w).await;
    let token = w
        .oidc
        .oidc
        .mint(Provider::Google, &spec("g1", GOOGLE_AUD, Some(n.clone())));
    let r = call(
        &w,
        "POST",
        "/v1/auth/google",
        &[],
        Some(json!({"id_token": token, "nonce": n, "device_name": "x".repeat(101)})),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_close_one_and_close_everywhere(pool: PgPool) {
    let w = world(&pool).await;
    let first = token_of(&apple(&w, "a1", "ios", |_| {}).await);
    let second = token_of(&apple(&w, "a1", "ios", |_| {}).await);
    let third = token_of(&apple(&w, "a1", "ios", |_| {}).await);
    let stranger = token_of(&apple(&w, "someone-else", "ios", |_| {}).await);

    let list = get_as(&w, "/v1/me/sessions", &first).await;
    let items = list.body["items"].as_array().unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items.iter().filter(|i| i["current"] == true).count(), 1);
    assert_eq!(list.body["next_cursor"], Value::Null);
    let second_id = items.iter().find(|i| i["current"] == false).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();

    // Another user's session id looks like no session at all.
    let b = bearer(&stranger);
    let foreign = call(
        &w,
        "DELETE",
        &format!("/v1/me/sessions/{second_id}"),
        &[(b[0].0, b[0].1.as_str())],
        None,
    )
    .await;
    assert_eq!(foreign.status, StatusCode::NOT_FOUND);
    assert_eq!(
        get_as(&w, "/v1/me/session", &first).await.status,
        StatusCode::OK
    );

    // Close one by id. Which token it was is found by trying them.
    let b = bearer(&first);
    let r = call(
        &w,
        "DELETE",
        &format!("/v1/me/sessions/{second_id}"),
        &[(b[0].0, b[0].1.as_str())],
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    let alive: Vec<_> = {
        let mut v = Vec::new();
        for t in [&first, &second, &third] {
            v.push(get_as(&w, "/v1/me/session", t).await.status == StatusCode::OK);
        }
        v
    };
    assert_eq!(alive.iter().filter(|a| **a).count(), 2, "{alive:?}");
    let again = call(
        &w,
        "DELETE",
        &format!("/v1/me/sessions/{second_id}"),
        &[(b[0].0, b[0].1.as_str())],
        None,
    )
    .await;
    assert_eq!(again.status, StatusCode::NOT_FOUND, "already closed");

    // Sign out of this session only.
    let r = call(
        &w,
        "DELETE",
        "/v1/me/session",
        &[(b[0].0, b[0].1.as_str())],
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get_as(&w, "/v1/me/session", &first).await.status,
        StatusCode::UNAUTHORIZED
    );

    // Sign out everywhere.
    let fresh = token_of(&apple(&w, "a1", "ios", |_| {}).await);
    let b = bearer(&fresh);
    let r = call(
        &w,
        "DELETE",
        "/v1/me/sessions",
        &[(b[0].0, b[0].1.as_str())],
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    for t in [&first, &second, &third, &fresh] {
        assert_eq!(
            get_as(&w, "/v1/me/session", t).await.status,
            StatusCode::UNAUTHORIZED
        );
    }
    // The other user is untouched.
    assert_eq!(
        get_as(&w, "/v1/me/session", &stranger).await.status,
        StatusCode::OK
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn web_sign_out_clears_the_cookie(pool: PgPool) {
    let w = world(&pool).await;
    let r = google(&w, "g1", "web", |_| {}).await;
    let csrf = r.body["session"]["csrf_token"].as_str().unwrap().to_owned();
    let pair = r.header("set-cookie").split(';').next().unwrap().to_owned();
    let out = call(
        &w,
        "DELETE",
        "/v1/me/session",
        &[
            ("cookie", pair.as_str()),
            ("origin", ORIGIN),
            ("x-csrf-token", csrf.as_str()),
        ],
        None,
    )
    .await;
    assert_eq!(out.status, StatusCode::NO_CONTENT);
    assert!(out.header("set-cookie").contains("Max-Age=0"));
    assert_eq!(
        call(
            &w,
            "GET",
            "/v1/me/session",
            &[("cookie", pair.as_str())],
            None
        )
        .await
        .status,
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn sessions_end_by_idle_time_and_absolute_lifetime(pool: PgPool) {
    let w = world(&pool).await;
    let idle = token_of(&apple(&w, "a1", "ios", |_| {}).await);
    let old = token_of(&apple(&w, "a1", "ios", |_| {}).await);
    let web = google(&w, "g1", "web", |_| {}).await;
    let web_pair = web
        .header("set-cookie")
        .split(';')
        .next()
        .unwrap()
        .to_owned();

    // iOS: 30 days idle ends it.
    sqlx::query("UPDATE sessions SET last_activity_at = now() - interval '31 days' WHERE client_kind = 'ios' AND last_activity_at = (SELECT min(last_activity_at) FROM sessions WHERE client_kind = 'ios')")
        .execute(&pool).await.unwrap();
    // Absolute lifetime ends the other one even though it was active a moment ago.
    sqlx::query("UPDATE sessions SET expires_at = now() - interval '1 second' WHERE client_kind = 'ios' AND last_activity_at > now() - interval '1 day'")
        .execute(&pool).await.unwrap();
    assert_eq!(
        get_as(&w, "/v1/me/session", &idle).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        get_as(&w, "/v1/me/session", &old).await.status,
        StatusCode::UNAUTHORIZED
    );

    // Web without "remember": 12 hours.
    sqlx::query("UPDATE sessions SET last_activity_at = now() - interval '13 hours' WHERE client_kind = 'web'")
        .execute(&pool).await.unwrap();
    let r = call(
        &w,
        "GET",
        "/v1/me/session",
        &[("cookie", web_pair.as_str())],
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn activity_is_written_at_most_once_a_minute(pool: PgPool) {
    let w = world(&pool).await;
    let token = token_of(&apple(&w, "a1", "ios", |_| {}).await);
    let at = || async {
        sqlx::query_scalar::<_, chrono::DateTime<chrono::Utc>>(
            "SELECT last_activity_at FROM sessions",
        )
        .fetch_one(&pool)
        .await
        .unwrap()
    };
    let t0 = at().await;
    assert_eq!(
        get_as(&w, "/v1/me/session", &token).await.status,
        StatusCode::OK
    );
    assert_eq!(at().await, t0, "within a minute nothing is written");
    sqlx::query("UPDATE sessions SET last_activity_at = now() - interval '2 minutes'")
        .execute(&pool)
        .await
        .unwrap();
    let stale = at().await;
    assert_eq!(
        get_as(&w, "/v1/me/session", &token).await.status,
        StatusCode::OK
    );
    assert!(at().await > stale, "after a minute the time moves");
}

#[sqlx::test(migrations = "../../migrations")]
async fn redis_caches_sessions_and_revocation_beats_the_cache(pool: PgPool) {
    let w = world_with(&pool, Redis::connect_lazy(&redis_url()).unwrap(), &[]).await;
    let token = token_of(&apple(&w, "a1", "ios", |_| {}).await);
    assert_eq!(
        get_as(&w, "/v1/me/session", &token).await.status,
        StatusCode::OK
    ); // fills the cache
    // Closing through the API removes the cache entry.
    let b = bearer(&token);
    let r = call(
        &w,
        "DELETE",
        "/v1/me/session",
        &[(b[0].0, b[0].1.as_str())],
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get_as(&w, "/v1/me/session", &token).await.status,
        StatusCode::UNAUTHORIZED
    );
    // Sign out everywhere removes every entry.
    let a = token_of(&apple(&w, "a1", "ios", |_| {}).await);
    let c = token_of(&apple(&w, "a1", "ios", |_| {}).await);
    assert_eq!(
        get_as(&w, "/v1/me/session", &a).await.status,
        StatusCode::OK
    );
    assert_eq!(
        get_as(&w, "/v1/me/session", &c).await.status,
        StatusCode::OK
    );
    let b = bearer(&a);
    call(
        &w,
        "DELETE",
        "/v1/me/sessions",
        &[(b[0].0, b[0].1.as_str())],
        None,
    )
    .await;
    assert_eq!(
        get_as(&w, "/v1/me/session", &a).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        get_as(&w, "/v1/me/session", &c).await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn sessions_keep_working_when_redis_dies(pool: PgPool) {
    let mut proxy = RedisProxy::start().await;
    let w = world_with(&pool, Redis::connect_lazy(&proxy.url).unwrap(), &[]).await;
    let token = token_of(&apple(&w, "a1", "ios", |_| {}).await);
    assert_eq!(
        get_as(&w, "/v1/me/session", &token).await.status,
        StatusCode::OK
    );
    proxy.cut().await;
    assert_eq!(
        get_as(&w, "/v1/me/session", &token).await.status,
        StatusCode::OK
    );
    let again = apple(&w, "a1", "ios", |_| {}).await;
    assert_eq!(
        again.status,
        StatusCode::OK,
        "sign-in works without Redis (ADR-0009 rule 1.2)"
    );
}

// --- e-mail link (ADR-0009 rule 2) ------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn email_link_signs_in_once_and_only_a_digest_is_stored(pool: PgPool) {
    let w = world(&pool).await;
    let email = unique_email();
    let r = call(
        &w,
        "POST",
        "/v1/auth/email/start",
        &[("cf-ipcountry", "US")],
        Some(json!({"email": format!("  {}  ", email.to_uppercase()), "client": "ios"})),
    )
    .await;
    assert_eq!(r.status, StatusCode::ACCEPTED);
    assert_eq!(r.body["expires_in"], 900);
    let sent = w.mail.sent();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].to, email);
    assert!(
        sent[0].text.contains("iOS device near US"),
        "{}",
        sent[0].text
    );
    assert!(sent[0].text.contains("ignore this message"));
    let link = link_from(&sent[0]);
    assert!(
        link.starts_with("https://app.example.com/auth/email?token="),
        "{link}"
    );
    let token = token_from_link(&link);

    let stored: Vec<u8> = sqlx::query_scalar("SELECT token_hash FROM email_login_tokens")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored.len(), 32);
    let expires: f64 = sqlx::query_scalar(
        "SELECT extract(epoch FROM expires_at - created_at)::float8 FROM email_login_tokens",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!((expires - 900.0).abs() < 5.0, "{expires}");

    // The link opening does not use the token up; only the explicit verify does.
    let verify = |t: String| {
        let w = &w;
        async move {
            call(
                w,
                "POST",
                "/v1/auth/email/verify",
                &[],
                Some(json!({"token": t, "client": "ios"})),
            )
            .await
        }
    };
    let ok = verify(token.clone()).await;
    assert_eq!(ok.status, StatusCode::OK, "{:?}", ok.body);
    assert_eq!(ok.body["is_new_user"], true);
    assert_eq!(ok.body["user"]["contact_email"], email.as_str());
    let session = token_of(&ok);
    assert_eq!(
        get_as(&w, "/v1/me/session", &session).await.status,
        StatusCode::OK
    );
    let ids = get_as(&w, "/v1/me/identities", &session).await;
    assert_eq!(ids.body["items"][0]["provider"], "email");
    assert_eq!(ids.body["items"][0]["label"], email.as_str());

    // Single use.
    let replay = verify(token).await;
    assert_eq!(replay.status, StatusCode::UNAUTHORIZED);
    assert_eq!(first_error_code(&replay), "invalid_or_expired");

    // A second link for the same address is the same account.
    let second = email_login(&w, &email, "ios").await;
    assert_eq!(second.body["is_new_user"], false);
    assert_eq!(second.body["user"]["id"], ok.body["user"]["id"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn email_link_expires_after_15_minutes_and_garbage_is_refused(pool: PgPool) {
    let w = world(&pool).await;
    let email = unique_email();
    let token = email_token(&w, &email).await;
    sqlx::query("UPDATE email_login_tokens SET expires_at = now() - interval '1 second'")
        .execute(&pool)
        .await
        .unwrap();
    let r = call(
        &w,
        "POST",
        "/v1/auth/email/verify",
        &[],
        Some(json!({"token": token})),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    for bad in ["", "nonsense", &"x".repeat(300)] {
        let r = call(
            &w,
            "POST",
            "/v1/auth/email/verify",
            &[],
            Some(json!({"token": bad})),
        )
        .await;
        assert_eq!(r.status, StatusCode::UNAUTHORIZED, "{bad:?}");
    }
    let users: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(users, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn email_start_answers_the_same_for_known_and_unknown_addresses(pool: PgPool) {
    let w = world(&pool).await;
    let known = unique_email();
    email_login(&w, &known, "ios").await;
    let a = call(
        &w,
        "POST",
        "/v1/auth/email/start",
        &[],
        Some(json!({"email": known})),
    )
    .await;
    let b = call(
        &w,
        "POST",
        "/v1/auth/email/start",
        &[],
        Some(json!({"email": unique_email()})),
    )
    .await;
    assert_eq!((a.status, &a.body), (b.status, &b.body));
    // A malformed address is a validation error (it says nothing about registrations).
    for bad in ["nope", "a@b", "a@example.com\r\nBcc: x@y.z", ""] {
        let r = call(
            &w,
            "POST",
            "/v1/auth/email/start",
            &[],
            Some(json!({"email": bad})),
        )
        .await;
        assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{bad:?}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn email_start_is_limited_per_address_and_per_ip(pool: PgPool) {
    let w = world_with(&pool, Redis::connect_lazy(&redis_url()).unwrap(), &[]).await;
    let email = unique_email();
    let ip = Uuid::new_v4().to_string();
    for i in 1..=5 {
        let r = call(
            &w,
            "POST",
            "/v1/auth/email/start",
            &[("x-forwarded-for", &Uuid::new_v4().to_string())],
            Some(json!({"email": email})),
        )
        .await;
        assert_eq!(r.status, StatusCode::ACCEPTED, "request {i}");
    }
    let r = call(
        &w,
        "POST",
        "/v1/auth/email/start",
        &[("x-forwarded-for", &Uuid::new_v4().to_string())],
        Some(json!({"email": email})),
    )
    .await;
    assert_eq!(r.status, StatusCode::TOO_MANY_REQUESTS, "{:?}", r.body);
    assert!(r.headers.get(header::RETRY_AFTER).is_some());
    assert_eq!(w.mail.sent().len(), 5, "the sixth request sends nothing");

    // Per IP: 20 different addresses, the 21st is refused.
    for i in 1..=20 {
        let r = call(
            &w,
            "POST",
            "/v1/auth/email/start",
            &[("x-forwarded-for", &ip)],
            Some(json!({"email": unique_email()})),
        )
        .await;
        assert_eq!(r.status, StatusCode::ACCEPTED, "request {i}");
    }
    let r = call(
        &w,
        "POST",
        "/v1/auth/email/start",
        &[("x-forwarded-for", &ip)],
        Some(json!({"email": unique_email()})),
    )
    .await;
    assert_eq!(r.status, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn smtp_mailer_delivers_to_mailpit_and_refuses_header_injection() {
    let to = unique_email();
    let mailer = SmtpMailer::new("127.0.0.1", 1025, "no-reply@demleme.local");
    let mail = api::auth::email::sign_in_mail(
        &to,
        "https://app.example.com/auth/email?token=t",
        "iOS",
        Some("US"),
    );
    api::auth::email::Mailer::send(&mailer, &mail)
        .await
        .expect("mailpit accepts the mail (docker compose up -d mailpit)");
    let client = reqwest::Client::new();
    let mut found = None;
    for _ in 0..20 {
        let listed: Value = client
            .get(format!(
                "http://127.0.0.1:8025/api/v1/search?query=to%3A{}",
                to.replace('@', "%40")
            ))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        if let Some(m) = listed["messages"].as_array().and_then(|m| m.first()) {
            found = Some(m.clone());
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    let found = found.expect("mail arrived in mailpit");
    assert_eq!(found["Subject"], "Your sign-in link");
    assert_eq!(found["To"][0]["Address"], to.as_str());

    let evil = Mail {
        to: "a@example.com\r\nRCPT TO:<x@y.z>".into(),
        subject: "s".into(),
        text: "t".into(),
    };
    assert!(
        api::auth::email::Mailer::send(&mailer, &evil)
            .await
            .is_err()
    );
}

// --- same address in another account, linking (ADR-0009 rules 1.6 and 4) ----------------------

#[sqlx::test(migrations = "../../migrations")]
async fn same_verified_address_gives_a_hint_but_never_merges(pool: PgPool) {
    let w = world(&pool).await;
    let email = unique_email();
    let a = apple(&w, "apple-1", "ios", |s| {
        s.email = Some(email.clone());
        s.email_verified = Some(true);
    })
    .await;
    assert_eq!(a.body["account_hint"], Value::Null);
    let g = google(&w, "google-1", "ios", |s| {
        s.email = Some(email.to_uppercase());
        s.email_verified = Some(true);
    })
    .await;
    assert_eq!(g.status, StatusCode::OK, "{:?}", g.body);
    assert_eq!(g.body["is_new_user"], true);
    assert_ne!(
        g.body["user"]["id"], a.body["user"]["id"],
        "no automatic merge"
    );
    assert_eq!(g.body["account_hint"], json!({"providers": ["apple"]}));

    // The e-mail link for the same address sees both existing accounts' methods.
    let e = email_login(&w, &email, "ios").await;
    assert_eq!(e.body["is_new_user"], true);
    assert_eq!(
        e.body["account_hint"],
        json!({"providers": ["apple", "google"]})
    );

    // An unverified address gives no hint (and reveals nothing).
    let u = google(&w, "google-2", "ios", |s| {
        s.email = Some(email.clone());
        s.email_verified = Some(false);
    })
    .await;
    assert_eq!(u.body["account_hint"], Value::Null);
}

#[sqlx::test(migrations = "../../migrations")]
async fn linking_replaces_the_session_and_refuses_identities_of_others(pool: PgPool) {
    let w = world(&pool).await;
    let first = apple(&w, "apple-1", "ios", |_| {}).await;
    let old_token = token_of(&first);
    let user_id = first.body["user"]["id"].clone();

    let link = |token: String, sub: &'static str| {
        let w = &w;
        async move {
            let n = nonce(w).await;
            let id_token = w
                .oidc
                .oidc
                .mint(Provider::Google, &spec(sub, GOOGLE_AUD, Some(n.clone())));
            let b = bearer(&token);
            call(
                w,
                "POST",
                "/v1/me/identities",
                &[(b[0].0, b[0].1.as_str())],
                Some(json!({"provider": "google", "id_token": id_token, "nonce": n})),
            )
            .await
        }
    };
    let linked = link(old_token.clone(), "google-1").await;
    assert_eq!(linked.status, StatusCode::OK, "{:?}", linked.body);
    assert_eq!(linked.body["identity"]["provider"], "google");
    let new_token = token_of(&linked);
    assert_ne!(new_token, old_token);
    assert_eq!(
        get_as(&w, "/v1/me/session", &old_token).await.status,
        StatusCode::UNAUTHORIZED,
        "old token closed"
    );
    let me = get_as(&w, "/v1/me/session", &new_token).await;
    assert_eq!(me.body["user"]["id"], user_id);

    // Signing in with Google now reaches the same account.
    let g = google(&w, "google-1", "ios", |_| {}).await;
    assert_eq!(g.body["user"]["id"], user_id);
    assert_eq!(g.body["is_new_user"], false);

    // Linking the same method again is harmless.
    assert_eq!(
        link(new_token.clone(), "google-1").await.status,
        StatusCode::OK
    );
    let ids = get_as(&w, "/v1/me/identities", &new_token).await;
    // (the token was rotated by the repeat; use a fresh sign-in for the list)
    let _ = ids;
    let fresh = token_of(&apple(&w, "apple-1", "ios", |_| {}).await);
    assert_eq!(
        get_as(&w, "/v1/me/identities", &fresh).await.body["items"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    // Another account cannot take over an identity that belongs to someone else.
    let other = token_of(&apple(&w, "apple-2", "ios", |_| {}).await);
    let r = link(other, "google-1").await;
    assert_eq!(r.status, StatusCode::CONFLICT, "{:?}", r.body);
    assert_eq!(first_error_code(&r), "identity_in_use");
    let owner: Uuid =
        sqlx::query_scalar("SELECT user_id FROM user_identities WHERE subject = 'google-1'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(owner.to_string(), user_id.as_str().unwrap());

    // A bad provider or a missing token is a validation error.
    let b = bearer(&fresh);
    let r = call(
        &w,
        "POST",
        "/v1/me/identities",
        &[(b[0].0, b[0].1.as_str())],
        Some(json!({"provider": "facebook"})),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    let r = call(
        &w,
        "POST",
        "/v1/me/identities",
        &[(b[0].0, b[0].1.as_str())],
        Some(json!({"provider": "google"})),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_email_identity_can_be_linked_and_removal_has_guards(pool: PgPool) {
    let w = world(&pool).await;
    let token = token_of(&apple(&w, "apple-1", "ios", |_| {}).await);
    let email = unique_email();
    let mail_token = email_token(&w, &email).await;
    let b = bearer(&token);
    let linked = call(
        &w,
        "POST",
        "/v1/me/identities",
        &[(b[0].0, b[0].1.as_str())],
        Some(json!({"provider": "email", "token": mail_token})),
    )
    .await;
    assert_eq!(linked.status, StatusCode::OK, "{:?}", linked.body);
    let token = token_of(&linked);
    let ids = get_as(&w, "/v1/me/identities", &token).await;
    let items = ids.body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    let apple_id = items.iter().find(|i| i["provider"] == "apple").unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let email_id = items.iter().find(|i| i["provider"] == "email").unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();

    let b = bearer(&token);
    let h = [(b[0].0, b[0].1.as_str())];
    // Unknown id.
    let r = call(
        &w,
        "DELETE",
        &format!("/v1/me/identities/{}", Uuid::new_v4()),
        &h,
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    // An old sign-in needs a second factor first.
    sqlx::query(
        "UPDATE sessions SET created_at = now() - interval '11 minutes' WHERE revoked_at IS NULL",
    )
    .execute(&pool)
    .await
    .unwrap();
    let r = call(
        &w,
        "DELETE",
        &format!("/v1/me/identities/{email_id}"),
        &h,
        None,
    )
    .await;
    assert_eq!(
        (r.status, r.body["code"].as_str()),
        (StatusCode::FORBIDDEN, Some("step_up_required"))
    );
    sqlx::query("UPDATE sessions SET step_up_at = now() WHERE revoked_at IS NULL")
        .execute(&pool)
        .await
        .unwrap();
    // With a recent second factor it works ...
    let r = call(
        &w,
        "DELETE",
        &format!("/v1/me/identities/{email_id}"),
        &h,
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    // ... but the last method stays.
    let r = call(
        &w,
        "DELETE",
        &format!("/v1/me/identities/{apple_id}"),
        &h,
        None,
    )
    .await;
    assert_eq!(
        (r.status, first_error_code(&r)),
        (StatusCode::CONFLICT, "last_identity")
    );
}

// --- first workspace and membership -----------------------------------------------------------

fn workspace_body() -> Value {
    json!({"name": "  Back Garden Brewing ", "mode": "home", "region": "us", "country": "US", "time_zone": "America/New_York"})
}

async fn create_workspace(w: &World, token: &str, body: &Value, key: &str) -> Reply {
    let b = bearer(token);
    call(
        w,
        "POST",
        "/v1/workspaces",
        &[(b[0].0, b[0].1.as_str()), ("idempotency-key", key)],
        Some(body.clone()),
    )
    .await
}

#[sqlx::test(migrations = "../../migrations")]
async fn first_workspace_makes_the_user_its_administrator(pool: PgPool) {
    let w = world(&pool).await;
    let login = apple(&w, "apple-1", "ios", |_| {}).await;
    let token = token_of(&login);
    let user_id: Uuid = login.body["user"]["id"].as_str().unwrap().parse().unwrap();

    let key = Uuid::new_v4().to_string();
    let r = create_workspace(&w, &token, &workspace_body(), &key).await;
    assert_eq!(r.status, StatusCode::CREATED, "{:?}", r.body);
    assert_eq!(r.body["name"], "Back Garden Brewing");
    assert_eq!(r.body["mode"], "home");
    assert_eq!(r.body["region"], "us");
    let ws: Uuid = r.body["id"].as_str().unwrap().parse().unwrap();

    // Replaying the same request creates nothing new (ADR-0008 rule 6).
    let replay = create_workspace(&w, &token, &workspace_body(), &key).await;
    assert_eq!(replay.body["id"], r.body["id"]);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM workspaces")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);

    // The session now lists the workspace.
    let me = get_as(&w, "/v1/me/session", &token).await;
    assert_eq!(me.body["workspaces"][0]["id"], ws.to_string());
    assert_eq!(me.body["workspaces"][0]["status"], "active");
    assert_eq!(me.body["workspaces"][0]["name"], "Back Garden Brewing");

    // The authenticator finds the membership and all permissions through the context layer.
    let probe = get_as(&w, &format!("/v1/workspaces/{ws}/probe"), &token).await;
    assert_eq!(probe.status, StatusCode::OK, "{:?}", probe.body);
    let perms = probe.body["permissions"].as_array().unwrap();
    assert_eq!(perms.len(), 17);
    assert!(perms.iter().any(|p| p == "users.manage"));
    let direct = w
        .svc
        .membership(
            &api::context::Identity {
                user_id,
                session_id: None,
                actor_kind: db::ActorKind::User,
            },
            ws,
        )
        .await
        .unwrap();
    assert_eq!(direct.unwrap().permissions.len(), 17);

    // A user who is not a member sees `404`, as if the workspace did not exist.
    let stranger = token_of(&apple(&w, "apple-2", "ios", |_| {}).await);
    assert_eq!(
        get_as(&w, &format!("/v1/workspaces/{ws}/probe"), &stranger)
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    // A disabled member loses access at once.
    sqlx::query("UPDATE workspace_members SET status = 'disabled'")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        get_as(&w, &format!("/v1/workspaces/{ws}/probe"), &token)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn workspace_creation_checks_input_and_needs_a_key_and_a_session(pool: PgPool) {
    let w = world(&pool).await;
    let token = token_of(&apple(&w, "apple-1", "ios", |_| {}).await);
    let b = bearer(&token);
    let no_key = call(
        &w,
        "POST",
        "/v1/workspaces",
        &[(b[0].0, b[0].1.as_str())],
        Some(workspace_body()),
    )
    .await;
    assert_eq!(no_key.status, StatusCode::PRECONDITION_REQUIRED);

    for (field, value) in [
        ("mode", "enterprise"),
        ("region", "eu"),
        ("country", "usa"),
        ("name", " "),
        ("time_zone", ""),
    ] {
        let mut body = workspace_body();
        body[field] = json!(value);
        let r = create_workspace(&w, &token, &body, &Uuid::new_v4().to_string()).await;
        assert_eq!(
            r.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{field}: {:?}",
            r.body
        );
        assert_eq!(r.body["errors"][0]["field"], format!("/{field}"));
    }
    let anon = call(
        &w,
        "POST",
        "/v1/workspaces",
        &[("idempotency-key", "k1")],
        Some(workspace_body()),
    )
    .await;
    assert_eq!(anon.status, StatusCode::UNAUTHORIZED);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM workspaces")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn two_users_workspaces_stay_apart(pool: PgPool) {
    let w = world(&pool).await;
    let a = token_of(&apple(&w, "apple-1", "ios", |_| {}).await);
    let b = token_of(&google(&w, "google-1", "ios", |_| {}).await);
    let wa = create_workspace(&w, &a, &workspace_body(), "ka").await.body["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let wb = create_workspace(&w, &b, &workspace_body(), "kb").await.body["id"]
        .as_str()
        .unwrap()
        .to_owned();
    for (token, mine, theirs) in [(&a, &wa, &wb), (&b, &wb, &wa)] {
        assert_eq!(
            get_as(&w, &format!("/v1/workspaces/{mine}/probe"), token)
                .await
                .status,
            StatusCode::OK
        );
        assert_eq!(
            get_as(&w, &format!("/v1/workspaces/{theirs}/probe"), token)
                .await
                .status,
            StatusCode::NOT_FOUND
        );
        let me = get_as(&w, "/v1/me/session", token).await;
        assert_eq!(me.body["workspaces"].as_array().unwrap().len(), 1);
        assert_eq!(me.body["workspaces"][0]["id"], mine.as_str());
    }
}
