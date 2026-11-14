//! End to end tests of T2.12 (ADR-0009 rules 5 and 7): passkey registration and verification
//! with a software authenticator, the second step on protected operations, shared devices with
//! member PINs (attempt limit, lock, idle lock) and workspace separation. Needs PostgreSQL
//! (`#[sqlx::test]`); Redis is not used.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::items_after_statements, clippy::semicolon_if_nothing_returned)]

mod support;

use std::{collections::HashMap, sync::Arc};

use api::{
    ApiConfig, AppState,
    auth::{AuthServices, email::MemoryMailer, tokens::hex},
    redis_client::Redis,
};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use fake_oidc::{MintSpec, Provider, Running};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use support::{Reply, app_pool, send};
use uuid::Uuid;
use webauthn_authenticator_rs::{
    WebauthnAuthenticator,
    prelude::{CreationChallengeResponse, RequestChallengeResponse, Url},
    softpasskey::SoftPasskey,
};

const APPLE_AUD: &str = "com.example.brewery";
const ORIGIN: &str = "https://app.example.com";

struct World {
    app: Router,
    oidc: Running,
}

async fn world(pool: &PgPool) -> World {
    let oidc = fake_oidc::spawn_local().await;
    let map: HashMap<&str, String> = HashMap::from([
        ("DATABASE_URL", "postgres://unused".to_owned()),
        ("TRUST_FORWARDED_FOR", "true".to_owned()),
        ("APPLE_CLIENT_IDS", APPLE_AUD.to_owned()),
        ("APPLE_ISSUER", oidc.oidc.issuer(Provider::Apple)),
        ("APPLE_JWKS_URL", oidc.oidc.jwks_url(Provider::Apple)),
        ("WEB_BASE_URL", ORIGIN.to_owned()),
        ("CSRF_HMAC_KEY", "test-csrf-key-0123456789".to_owned()),
        ("JWKS_MIN_REFRESH_SECS", "0".to_owned()),
    ]);
    let cfg = ApiConfig::from_lookup(|n| map.get(n).cloned()).unwrap();
    let app_db = app_pool(pool).await;
    let svc = Arc::new(AuthServices::new(
        app_db.clone(),
        Redis::disabled(),
        cfg.auth.clone(),
        Arc::new(MemoryMailer::default()),
    ));
    let state = AppState::new(app_db, Redis::disabled(), svc.clone(), cfg);
    let app = api::build(state.clone(), svc.routes(&state));
    World { app, oidc }
}

async fn call(
    w: &World,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<Value>,
) -> Reply {
    let mut b = Request::builder()
        .method(method)
        .uri(path)
        .header("x-forwarded-for", Uuid::new_v4().to_string());
    for (k, v) in headers {
        b = b.header(*k, *v);
    }
    let req = match body {
        Some(v) => b
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(v.to_string()))
            .unwrap(),
        None => b.body(Body::empty()).unwrap(),
    };
    send(&w.app, req).await
}

/// Request as the holder of `token`.
async fn as_user(w: &World, token: &str, method: &str, path: &str, body: Option<Value>) -> Reply {
    let auth = format!("Bearer {token}");
    let key = Uuid::new_v4().to_string();
    call(
        w,
        method,
        path,
        &[("authorization", &auth), ("idempotency-key", &key)],
        body,
    )
    .await
}

/// Request as a device holding `key`.
async fn as_device(w: &World, key: &str, method: &str, path: &str, body: Option<Value>) -> Reply {
    call(w, method, path, &[("x-device-key", key)], body).await
}

/// New user (Apple sign-in on iOS); returns the session token.
async fn sign_in(w: &World, sub: &str) -> String {
    let n = call(w, "POST", "/v1/auth/nonce", &[], None).await.body["nonce"]
        .as_str()
        .unwrap()
        .to_owned();
    let token = w.oidc.oidc.mint(
        Provider::Apple,
        &MintSpec {
            sub: sub.to_owned(),
            aud: APPLE_AUD.to_owned(),
            nonce: Some(hex(&Sha256::digest(n.as_bytes()))),
            ..MintSpec::default()
        },
    );
    let r = call(
        w,
        "POST",
        "/v1/auth/apple",
        &[],
        Some(json!({"id_token": token, "nonce": n, "client": "ios", "display_name": sub})),
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
    r.body["session"]["token"].as_str().unwrap().to_owned()
}

async fn new_workspace(w: &World, token: &str) -> Uuid {
    let r = as_user(
        w,
        token,
        "POST",
        "/v1/workspaces",
        Some(
            json!({"name": "Brewery", "mode": "business", "region": "us",
                    "country": "US", "time_zone": "America/New_York"}),
        ),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{:?}", r.body);
    r.body["id"].as_str().unwrap().parse().unwrap()
}

fn code(r: &Reply) -> &str {
    r.body["errors"][0]["code"].as_str().unwrap_or_default()
}

fn device_key() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

/// Registers a passkey for the holder of `token`; returns the replacement token.
async fn register(w: &World, token: &str, soft: &mut SoftPasskey) -> String {
    let start = as_user(w, token, "POST", "/v1/me/passkeys/registration/start", None).await;
    assert_eq!(start.status, StatusCode::OK, "{:?}", start.body);
    let ccr: CreationChallengeResponse =
        serde_json::from_value(start.body["options"].clone()).unwrap();
    let credential = soft
        .do_registration(Url::parse(ORIGIN).unwrap(), ccr)
        .unwrap();
    let done = as_user(
        w,
        token,
        "POST",
        "/v1/me/passkeys/registration/finish",
        Some(json!({"challenge_id": start.body["challenge_id"], "credential": credential, "name": "Test key"})),
    )
    .await;
    assert_eq!(done.status, StatusCode::CREATED, "{:?}", done.body);
    done.body["session"]["token"].as_str().unwrap().to_owned()
}

/// Runs the second step; returns the replacement token.
async fn step_up(w: &World, token: &str, soft: &mut SoftPasskey) -> String {
    let start = as_user(w, token, "POST", "/v1/me/step-up/start", None).await;
    assert_eq!(start.status, StatusCode::OK, "{:?}", start.body);
    let rcr: RequestChallengeResponse =
        serde_json::from_value(start.body["options"].clone()).unwrap();
    let credential = soft
        .do_authentication(Url::parse(ORIGIN).unwrap(), rcr)
        .unwrap();
    let done = as_user(
        w,
        token,
        "POST",
        "/v1/me/step-up/finish",
        Some(json!({"challenge_id": start.body["challenge_id"], "credential": credential})),
    )
    .await;
    assert_eq!(done.status, StatusCode::OK, "{:?}", done.body);
    done.body["session"]["token"].as_str().unwrap().to_owned()
}

fn soft() -> SoftPasskey {
    SoftPasskey::new(true)
}

// --- passkey and second step -----------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn admin_needs_a_passkey_and_a_fresh_second_step(pool: PgPool) {
    let w = world(&pool).await;
    let mut key = soft();
    let token = sign_in(&w, "admin-1").await;
    let ws = new_workspace(&w, &token).await;
    let body = json!({"name": "iPad", "device_key": device_key()});
    let path = format!("/v1/workspaces/{ws}/shared-devices");

    // No passkey: 403 step_up_required, enrollment first.
    let r = as_user(&w, &token, "POST", &path, Some(body.clone())).await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    assert_eq!(r.body["code"], "step_up_required");
    assert_eq!(code(&r), "passkey_enrollment_required");

    // Registering counts as the second step.
    let token = register(&w, &token, &mut key).await;
    let r = as_user(&w, &token, "POST", &path, Some(body)).await;
    assert_eq!(r.status, StatusCode::CREATED, "{:?}", r.body);

    // The list shows the passkey; the old token no longer works.
    let list = as_user(&w, &token, "GET", "/v1/me/passkeys", None).await;
    assert_eq!(list.body.as_array().unwrap().len(), 1);
    assert_eq!(list.body[0]["name"], "Test key");

    // Second step older than 15 minutes: refused again, a new check restores it.
    sqlx::query("UPDATE sessions SET step_up_at = now() - interval '16 minutes'")
        .execute(&pool)
        .await
        .unwrap();
    let again = json!({"name": "iPad 2", "device_key": device_key()});
    let r = as_user(&w, &token, "POST", &path, Some(again.clone())).await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    assert_eq!(code(&r), "passkey_required");
    let token = step_up(&w, &token, &mut key).await;
    let r = as_user(&w, &token, "POST", &path, Some(again)).await;
    assert_eq!(r.status, StatusCode::CREATED, "{:?}", r.body);
    let (counter,): (i64,) = sqlx::query_as("SELECT sign_count FROM passkeys")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(counter >= 1, "sign count is stored");
}

#[sqlx::test(migrations = "../../migrations")]
async fn broken_or_replayed_ceremonies_are_refused(pool: PgPool) {
    let w = world(&pool).await;
    let mut key = soft();
    let token = sign_in(&w, "admin-2").await;

    // Registration: replay of the same challenge and a credential of another challenge.
    let start = as_user(
        &w,
        &token,
        "POST",
        "/v1/me/passkeys/registration/start",
        None,
    )
    .await;
    let ccr: CreationChallengeResponse =
        serde_json::from_value(start.body["options"].clone()).unwrap();
    let credential = key
        .do_registration(Url::parse(ORIGIN).unwrap(), ccr)
        .unwrap();
    let finish = json!({"challenge_id": start.body["challenge_id"], "credential": credential});
    let ok = as_user(
        &w,
        &token,
        "POST",
        "/v1/me/passkeys/registration/finish",
        Some(finish.clone()),
    )
    .await;
    assert_eq!(ok.status, StatusCode::CREATED);
    let token = ok.body["session"]["token"].as_str().unwrap().to_owned();
    let replay = as_user(
        &w,
        &token,
        "POST",
        "/v1/me/passkeys/registration/finish",
        Some(finish),
    )
    .await;
    assert_eq!(replay.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(code(&replay), "challenge_invalid");

    // Verification: an answer to another challenge is refused and uses the challenge up.
    let first = as_user(&w, &token, "POST", "/v1/me/step-up/start", None).await;
    let second = as_user(&w, &token, "POST", "/v1/me/step-up/start", None).await;
    let rcr: RequestChallengeResponse =
        serde_json::from_value(second.body["options"].clone()).unwrap();
    let credential = key
        .do_authentication(Url::parse(ORIGIN).unwrap(), rcr)
        .unwrap();
    let body = json!({"challenge_id": first.body["challenge_id"], "credential": credential});
    let r = as_user(
        &w,
        &token,
        "POST",
        "/v1/me/step-up/finish",
        Some(body.clone()),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(code(&r), "passkey_invalid");
    let r = as_user(&w, &token, "POST", "/v1/me/step-up/finish", Some(body)).await;
    assert_eq!(code(&r), "challenge_invalid");

    // Garbage credential.
    let start = as_user(&w, &token, "POST", "/v1/me/step-up/start", None).await;
    let r = as_user(
        &w,
        &token,
        "POST",
        "/v1/me/step-up/finish",
        Some(json!({"challenge_id": start.body["challenge_id"], "credential": {"x": 1}})),
    )
    .await;
    assert_eq!(code(&r), "credential_invalid");

    // A user without a passkey has nothing to verify.
    let other = sign_in(&w, "no-key").await;
    let r = as_user(&w, &other, "POST", "/v1/me/step-up/start", None).await;
    assert_eq!(r.status, StatusCode::CONFLICT);
    assert_eq!(code(&r), "no_passkey");
}

// --- shared device ---------------------------------------------------------------------------

struct Setup {
    ws: Uuid,
    admin: String,
    device_id: Uuid,
    key: String,
    member_id: Uuid,
}

/// Admin with a passkey, one device, and a PIN (123456) set by the admin.
async fn shared_setup(w: &World, pool: &PgPool, sub: &str) -> Setup {
    let mut soft = soft();
    let token = sign_in(w, sub).await;
    let ws = new_workspace(w, &token).await;
    let admin = register(w, &token, &mut soft).await;
    let key = device_key();
    let r = as_user(
        w,
        &admin,
        "POST",
        &format!("/v1/workspaces/{ws}/shared-devices"),
        Some(json!({"name": "Brewhouse iPad", "device_key": key, "idle_timeout_min": 5})),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{:?}", r.body);
    let device_id = r.body["id"].as_str().unwrap().parse().unwrap();
    let r = as_user(
        w,
        &admin,
        "PUT",
        &format!("/v1/workspaces/{ws}/me/pin"),
        Some(json!({"pin": "123456"})),
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT, "{:?}", r.body);
    let (member_id,): (Uuid,) =
        sqlx::query_as("SELECT member_id FROM member_pins WHERE workspace_id = $1")
            .bind(ws)
            .fetch_one(pool)
            .await
            .unwrap();
    Setup {
        ws,
        admin,
        device_id,
        key,
        member_id,
    }
}

async fn switch(w: &World, s: &Setup, pin: &str) -> Reply {
    as_device(
        w,
        &s.key,
        "POST",
        &format!("/v1/shared-devices/{}/switch", s.device_id),
        Some(json!({"member_id": s.member_id, "pin": pin})),
    )
    .await
}

#[sqlx::test(migrations = "../../migrations")]
async fn pin_switch_lock_and_idle_timeout(pool: PgPool) {
    let w = world(&pool).await;
    let s = shared_setup(&w, &pool, "admin-3").await;

    // The PIN is stored as Argon2id.
    let (hash,): (String,) = sqlx::query_as("SELECT pin_hash FROM member_pins")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(hash.starts_with("$argon2id$") && !hash.contains("123456"));

    // Tiles list the member with a PIN.
    let tiles = as_device(
        &w,
        &s.key,
        "GET",
        &format!("/v1/shared-devices/{}/members", s.device_id),
        None,
    )
    .await;
    assert_eq!(tiles.status, StatusCode::OK, "{:?}", tiles.body);
    assert_eq!(tiles.body[0]["display_name"], "admin-3");

    // Switch: a device session with the member's permissions.
    let r = switch(&w, &s, "123456").await;
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
    assert_eq!(r.body["session"]["client"], "shared_device");
    let user_token = r.body["session"]["token"].as_str().unwrap().to_owned();
    let me = as_user(&w, &user_token, "GET", "/v1/me/session", None).await;
    assert_eq!(me.status, StatusCode::OK, "{:?}", me.body);
    assert_eq!(me.body["session"]["client"], "shared_device");
    let (open,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM device_sessions WHERE ended_at IS NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(open, 1);

    // A device session cannot do the second step or set PINs.
    let r = as_user(&w, &user_token, "POST", "/v1/me/step-up/start", None).await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    let r = as_user(
        &w,
        &user_token,
        "PUT",
        &format!("/v1/workspaces/{}/me/pin", s.ws),
        Some(json!({"pin": "654321"})),
    )
    .await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);

    // Idle lock: after the device's idle time the session is over (server side).
    sqlx::query("UPDATE sessions SET last_activity_at = now() - interval '6 minutes' WHERE client_kind = 'shared_device'")
        .execute(&pool)
        .await
        .unwrap();
    let me = as_user(&w, &user_token, "GET", "/v1/me/session", None).await;
    assert_eq!(me.status, StatusCode::UNAUTHORIZED);

    // Inside the idle time the session stays; locking ends it at once and closes the record.
    let r = switch(&w, &s, "123456").await;
    let user_token = r.body["session"]["token"].as_str().unwrap().to_owned();
    sqlx::query("UPDATE sessions SET last_activity_at = now() - interval '4 minutes' WHERE client_kind = 'shared_device' AND revoked_at IS NULL")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        as_user(&w, &user_token, "GET", "/v1/me/session", None)
            .await
            .status,
        StatusCode::OK
    );
    let lock = as_device(
        &w,
        &s.key,
        "POST",
        &format!("/v1/shared-devices/{}/lock", s.device_id),
        None,
    )
    .await;
    assert_eq!(lock.status, StatusCode::NO_CONTENT);
    assert_eq!(
        as_user(&w, &user_token, "GET", "/v1/me/session", None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    let (open,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM device_sessions WHERE ended_at IS NULL")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(open, 0);

    // A new switch closes the previous user's session.
    let first = switch(&w, &s, "123456").await;
    let first_token = first.body["session"]["token"].as_str().unwrap().to_owned();
    let second = switch(&w, &s, "123456").await;
    assert_eq!(second.status, StatusCode::OK);
    assert_eq!(
        as_user(&w, &first_token, "GET", "/v1/me/session", None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn pin_attempts_are_limited_and_the_member_is_locked(pool: PgPool) {
    let w = world(&pool).await;
    let s = shared_setup(&w, &pool, "admin-4").await;

    // Four wrong PINs: still 401; the correct one resets the counter.
    for _ in 0..4 {
        let r = switch(&w, &s, "000000").await;
        assert_eq!(r.status, StatusCode::UNAUTHORIZED);
        assert_eq!(code(&r), "pin_invalid");
    }
    assert_eq!(switch(&w, &s, "123456").await.status, StatusCode::OK);
    let (failed,): (i32,) = sqlx::query_as("SELECT failed_attempts FROM member_pins")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(failed, 0);

    // Five in a row lock the member for 15 minutes; even the right PIN is refused.
    for i in 0..5 {
        let r = switch(&w, &s, "000000").await;
        if i < 4 {
            assert_eq!(r.status, StatusCode::UNAUTHORIZED);
        } else {
            assert_eq!(r.status, StatusCode::TOO_MANY_REQUESTS);
            assert_eq!(code(&r), "pin_locked");
        }
    }
    let r = switch(&w, &s, "123456").await;
    assert_eq!(r.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(code(&r), "pin_locked");
    let (minutes,): (f64,) = sqlx::query_as(
        "SELECT (extract(epoch FROM locked_until - now()) / 60)::float8 FROM member_pins",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        (14.0..=15.0).contains(&minutes),
        "locked for {minutes} minutes"
    );

    // After the lock the right PIN works again.
    sqlx::query("UPDATE member_pins SET locked_until = now() - interval '1 second'")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(switch(&w, &s, "123456").await.status, StatusCode::OK);

    // An administrator clears a PIN; until a new one is set the member has no tile.
    let r = as_user(
        &w,
        &s.admin,
        "DELETE",
        &format!("/v1/workspaces/{}/members/{}/pin", s.ws, s.member_id),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    let r = switch(&w, &s, "123456").await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    assert_eq!(code(&r), "pin_invalid");

    // Bad shapes are validation errors, not attempts.
    let r = switch(&w, &s, "12345").await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "../../migrations")]
async fn other_workspaces_cannot_see_or_use_the_device(pool: PgPool) {
    let w = world(&pool).await;
    let a = shared_setup(&w, &pool, "admin-a").await;
    let b = shared_setup(&w, &pool, "admin-b").await;

    // B's administrator addresses A's device through B's workspace: 404, and through A's: 404.
    let r = as_user(
        &w,
        &b.admin,
        "DELETE",
        &format!("/v1/workspaces/{}/shared-devices/{}", b.ws, a.device_id),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    let r = as_user(
        &w,
        &b.admin,
        "DELETE",
        &format!("/v1/workspaces/{}/shared-devices/{}", a.ws, a.device_id),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    let r = as_user(
        &w,
        &b.admin,
        "GET",
        &format!("/v1/workspaces/{}/shared-devices", a.ws),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    let list = as_user(
        &w,
        &b.admin,
        "GET",
        &format!("/v1/workspaces/{}/shared-devices", b.ws),
        None,
    )
    .await;
    assert_eq!(list.body.as_array().unwrap().len(), 1);
    assert_eq!(list.body[0]["id"], json!(b.device_id));

    // Device keys: B's key on A's device is a 404, a wrong key is a 401.
    let r = as_device(
        &w,
        &b.key,
        "GET",
        &format!("/v1/shared-devices/{}/members", a.device_id),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    let r = as_device(
        &w,
        &device_key(),
        "GET",
        &format!("/v1/shared-devices/{}/members", a.device_id),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    let r = call(
        &w,
        "GET",
        &format!("/v1/shared-devices/{}/members", a.device_id),
        &[],
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);

    // A member of A (switch on A's device) is no member of B: B's endpoints answer 404.
    let r = switch(&w, &a, "123456").await;
    let a_user = r.body["session"]["token"].as_str().unwrap().to_owned();
    let r = as_user(
        &w,
        &a_user,
        "GET",
        &format!("/v1/workspaces/{}/shared-devices", b.ws),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);

    // Tiles of each device show only their own workspace.
    let tiles = as_device(
        &w,
        &b.key,
        "GET",
        &format!("/v1/shared-devices/{}/members", b.device_id),
        None,
    )
    .await;
    assert_eq!(tiles.body.as_array().unwrap().len(), 1);
    assert_eq!(tiles.body[0]["display_name"], "admin-b");

    // Revoking A's device kills its key and its sessions.
    let r = as_user(
        &w,
        &a.admin,
        "DELETE",
        &format!("/v1/workspaces/{}/shared-devices/{}", a.ws, a.device_id),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    assert_eq!(
        as_user(&w, &a_user, "GET", "/v1/me/session", None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    let r = as_device(
        &w,
        &a.key,
        "GET",
        &format!("/v1/shared-devices/{}/members", a.device_id),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn key_rotation_needs_the_second_step_and_ends_the_old_key(pool: PgPool) {
    let w = world(&pool).await;
    let s = shared_setup(&w, &pool, "admin-5").await;
    let path = format!(
        "/v1/workspaces/{}/shared-devices/{}/rotate-key",
        s.ws, s.device_id
    );
    let new_key = device_key();
    sqlx::query("UPDATE sessions SET step_up_at = now() - interval '20 minutes'")
        .execute(&pool)
        .await
        .unwrap();
    let r = as_user(
        &w,
        &s.admin,
        "POST",
        &path,
        Some(json!({"device_key": new_key})),
    )
    .await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    assert_eq!(r.body["code"], "step_up_required");
    sqlx::query("UPDATE sessions SET step_up_at = now()")
        .execute(&pool)
        .await
        .unwrap();
    // The session cache is off (no Redis), so the database change is seen at once.
    let r = as_user(
        &w,
        &s.admin,
        "POST",
        &path,
        Some(json!({"device_key": new_key})),
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT, "{:?}", r.body);
    let r = as_device(
        &w,
        &s.key,
        "GET",
        &format!("/v1/shared-devices/{}/members", s.device_id),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    let r = as_device(
        &w,
        &new_key,
        "GET",
        &format!("/v1/shared-devices/{}/members", s.device_id),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::OK);
}
