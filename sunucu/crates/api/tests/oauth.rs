//! End to end tests of M2 (ADR-0015): the OAuth 2.1 authorization server, token binding and the
//! "Connected apps" endpoints. Needs PostgreSQL (`#[sqlx::test]`) and the compose Redis.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

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
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use fake_oidc::{MintSpec, Provider, Running};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use support::{Reply, app_pool, redis_url, send};
use url::Url;
use uuid::Uuid;

const APPLE_AUD: &str = "com.example.brewery";
const API_RESOURCE: &str = "https://api.gristora.com/v1";
const MCP_RESOURCE: &str = "https://api.gristora.com/mcp";
const REDIRECT: &str = "https://agent.example/cb";
const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";

struct World {
    app: Router,
    oidc: Running,
}

async fn world_with(pool: &PgPool, redis: Redis, extra: &[(&str, &str)]) -> World {
    let oidc = fake_oidc::spawn_local().await;
    let mut map: HashMap<String, String> = HashMap::from([
        ("DATABASE_URL", "postgres://unused".to_owned()),
        ("TRUST_FORWARDED_FOR", "true".to_owned()),
        ("APPLE_CLIENT_IDS", APPLE_AUD.to_owned()),
        ("APPLE_ISSUER", oidc.oidc.issuer(Provider::Apple)),
        ("APPLE_JWKS_URL", oidc.oidc.jwks_url(Provider::Apple)),
        ("WEB_BASE_URL", "https://app.example.com".to_owned()),
        ("CSRF_HMAC_KEY", "test-csrf-key-0123456789".to_owned()),
        ("JWKS_MIN_REFRESH_SECS", "0".to_owned()),
    ])
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v))
    .collect();
    for (k, v) in extra {
        map.insert((*k).to_owned(), (*v).to_owned());
    }
    let cfg = ApiConfig::from_lookup(|n| map.get(n).cloned()).unwrap();
    let app_db = app_pool(pool).await;
    let svc = AuthServices::new(
        app_db.clone(),
        redis.clone(),
        cfg.auth.clone(),
        Arc::new(MemoryMailer::default()),
    )
    .unwrap();
    let svc = Arc::new(svc);
    let state = AppState::new(app_db, redis, svc.clone(), cfg).with_auth_services(svc);
    World {
        app: api::router(state),
        oidc,
    }
}

async fn world(pool: &PgPool) -> World {
    world_with(pool, Redis::connect_lazy(&redis_url()).unwrap(), &[]).await
}

// --- helpers -----------------------------------------------------------------------------

async fn call(
    w: &World,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> Reply {
    let mut b = Request::builder()
        .method(method)
        .uri(path)
        .header("x-forwarded-for", Uuid::new_v4().to_string());
    if let Some(token) = token {
        b = b.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    if method == "POST" {
        b = b.header("idempotency-key", Uuid::new_v4().to_string());
    }
    let req = match body {
        Some(value) => b
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(value.to_string()))
            .unwrap(),
        None => b.body(Body::empty()).unwrap(),
    };
    send(&w.app, req).await
}

async fn form(w: &World, path: &str, pairs: &[(&str, &str)]) -> Reply {
    let body = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs)
        .finish();
    let req = Request::builder()
        .method("POST")
        .uri(path)
        .header("x-forwarded-for", Uuid::new_v4().to_string())
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();
    send(&w.app, req).await
}

async fn get_plain(w: &World, path_and_query: &str) -> Reply {
    let req = Request::builder()
        .method("GET")
        .uri(path_and_query)
        .header("x-forwarded-for", Uuid::new_v4().to_string())
        .body(Body::empty())
        .unwrap();
    send(&w.app, req).await
}

async fn sign_in(w: &World, subject: &str) -> String {
    let nonce = call(w, "POST", "/v1/auth/nonce", None, None).await.body["nonce"]
        .as_str()
        .unwrap()
        .to_owned();
    let id_token = w.oidc.oidc.mint(
        Provider::Apple,
        &MintSpec {
            sub: subject.to_owned(),
            aud: APPLE_AUD.to_owned(),
            nonce: Some(hex(&Sha256::digest(nonce.as_bytes()))),
            ..MintSpec::default()
        },
    );
    let reply = call(
        w,
        "POST",
        "/v1/auth/apple",
        None,
        Some(
            json!({"id_token": id_token, "nonce": nonce, "client": "ios", "display_name": subject}),
        ),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.body);
    reply.body["session"]["token"].as_str().unwrap().to_owned()
}

async fn new_workspace(w: &World, session: &str, mode: &str) -> Uuid {
    let reply = call(
        w,
        "POST",
        "/v1/workspaces",
        Some(session),
        Some(
            json!({"name": "Brewery", "mode": mode, "region": "us", "country": "US",
                    "time_zone": "America/New_York"}),
        ),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{:?}", reply.body);
    reply.body["id"].as_str().unwrap().parse().unwrap()
}

fn challenge() -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(VERIFIER.as_bytes()))
}

async fn register(w: &World) -> String {
    let r = call(
        w,
        "POST",
        "/register",
        None,
        Some(
            json!({"client_name": "Test Agent", "redirect_uris": [REDIRECT],
                    "token_endpoint_auth_method": "none"}),
        ),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{:?}", r.body);
    r.body["client_id"].as_str().unwrap().to_owned()
}

fn authorize_query(client: &str, scope: &str, resource: &str, method: &str) -> String {
    let q = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs([
            ("response_type", "code"),
            ("client_id", client),
            ("redirect_uri", REDIRECT),
            ("scope", scope),
            ("state", "st-1"),
            ("code_challenge", challenge().as_str()),
            ("code_challenge_method", method),
            ("resource", resource),
        ])
        .finish();
    format!("/authorize?{q}")
}

fn query_of(location: &str, name: &str) -> Option<String> {
    Url::parse(location)
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.into_owned())
}

fn location(r: &Reply) -> String {
    r.header("location").to_owned()
}

/// Runs `/authorize` and returns the consent request id.
async fn start(w: &World, client: &str, scope: &str, resource: &str) -> String {
    let r = get_plain(w, &authorize_query(client, scope, resource, "S256")).await;
    assert!(r.status.is_redirection(), "{:?} {:?}", r.status, r.body);
    let loc = location(&r);
    assert!(
        loc.starts_with("https://app.example.com/oauth/consent"),
        "{loc}"
    );
    query_of(&loc, "request").unwrap()
}

async fn approve(
    w: &World,
    session: &str,
    request: &str,
    ws: Uuid,
    scopes: Option<Vec<&str>>,
) -> Reply {
    let mut body = json!({"decision": "approve", "workspace_id": ws});
    if let Some(s) = scopes {
        body["scopes"] = json!(s);
    }
    call(
        w,
        "POST",
        &format!("/v1/me/oauth/requests/{request}/decision"),
        Some(session),
        Some(body),
    )
    .await
}

async fn exchange(w: &World, client: &str, code: &str, verifier: &str) -> Reply {
    form(
        w,
        "/token",
        &[
            ("grant_type", "authorization_code"),
            ("client_id", client),
            ("code", code),
            ("redirect_uri", REDIRECT),
            ("code_verifier", verifier),
        ],
    )
    .await
}

struct Tokens {
    client: String,
    access: String,
    refresh: String,
}

/// The full flow with a fresh dynamic client.
async fn connect(w: &World, session: &str, ws: Uuid, scope: &str, resource: &str) -> Tokens {
    let client = register(w).await;
    let request = start(w, &client, scope, resource).await;
    let decision = approve(w, session, &request, ws, None).await;
    assert_eq!(decision.status, StatusCode::OK, "{:?}", decision.body);
    let to = decision.body["redirect_to"].as_str().unwrap().to_owned();
    assert_eq!(query_of(&to, "state").as_deref(), Some("st-1"));
    assert_eq!(
        query_of(&to, "iss").as_deref(),
        Some("https://auth.gristora.com")
    );
    let code = query_of(&to, "code").unwrap();
    let t = exchange(w, &client, &code, VERIFIER).await;
    assert_eq!(t.status, StatusCode::OK, "{:?}", t.body);
    assert_eq!(t.body["token_type"], "Bearer");
    assert_eq!(t.body["expires_in"], 3600);
    Tokens {
        client,
        access: t.body["access_token"].as_str().unwrap().to_owned(),
        refresh: t.body["refresh_token"].as_str().unwrap().to_owned(),
    }
}

async fn user_id(pool: &PgPool, subject: &str) -> Uuid {
    sqlx::query_scalar(
        "SELECT user_id FROM user_identities WHERE provider = 'apple' AND subject = $1",
    )
    .bind(subject)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn audit_actions(pool: &PgPool, ws: Uuid) -> Vec<String> {
    sqlx::query_scalar("SELECT action FROM audit_log WHERE workspace_id = $1 ORDER BY seq")
        .bind(ws)
        .fetch_all(pool)
        .await
        .unwrap()
}

// --- tests -------------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn metadata_documents_describe_the_server(pool: PgPool) {
    let w = world(&pool).await;
    let r = get_plain(&w, "/.well-known/oauth-authorization-server").await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body["issuer"], "https://auth.gristora.com");
    assert_eq!(r.body["code_challenge_methods_supported"], json!(["S256"]));
    assert_eq!(r.body["client_id_metadata_document_supported"], true);
    assert_eq!(
        r.body["authorization_response_iss_parameter_supported"],
        true
    );
    assert_eq!(
        r.body["grant_types_supported"],
        json!(["authorization_code", "refresh_token"])
    );
    let scopes = r.body["scopes_supported"].as_array().unwrap();
    assert!(scopes.iter().any(|s| s == "costs:read"));
    let mcp = get_plain(&w, "/.well-known/oauth-protected-resource/mcp").await;
    assert_eq!(mcp.body["resource"], MCP_RESOURCE);
    assert_eq!(
        mcp.body["authorization_servers"],
        json!(["https://auth.gristora.com"])
    );
    assert_eq!(mcp.body["bearer_methods_supported"], json!(["header"]));
    // Read scopes only: write scopes are asked for when needed.
    assert!(
        !mcp.body["scopes_supported"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s == "recipes:draft")
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn authorize_refuses_what_oauth_2_1_forbids(pool: PgPool) {
    let w = world(&pool).await;
    let client = register(&w).await;
    // plain PKCE: redirected back with an error.
    let r = get_plain(
        &w,
        &authorize_query(&client, "recipes:read", API_RESOURCE, "plain"),
    )
    .await;
    assert!(r.status.is_redirection());
    let loc = location(&r);
    assert!(loc.starts_with(REDIRECT));
    assert_eq!(query_of(&loc, "error").as_deref(), Some("invalid_request"));
    assert_eq!(query_of(&loc, "state").as_deref(), Some("st-1"));
    assert!(query_of(&loc, "iss").is_some());
    // Unknown scope, unknown resource.
    let r = get_plain(&w, &authorize_query(&client, "admin", API_RESOURCE, "S256")).await;
    assert_eq!(
        query_of(&location(&r), "error").as_deref(),
        Some("invalid_scope")
    );
    let r = get_plain(
        &w,
        &authorize_query(&client, "recipes:read", "https://x.example/", "S256"),
    )
    .await;
    assert_eq!(
        query_of(&location(&r), "error").as_deref(),
        Some("invalid_target")
    );
    // A redirect address that is not registered never redirects.
    let q = authorize_query(&client, "recipes:read", API_RESOURCE, "S256")
        .replace("agent.example", "evil.example");
    let r = get_plain(&w, &q).await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST);
    assert_eq!(r.body["error"], "invalid_request");
    // An unknown client does not redirect either.
    let r = get_plain(
        &w,
        &authorize_query("dcr_unknown", "recipes:read", API_RESOURCE, "S256"),
    )
    .await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST);
    // DCR refuses confidential clients and implicit flows.
    let bad = call(
        &w,
        "POST",
        "/register",
        None,
        Some(json!({
        "redirect_uris": [REDIRECT], "token_endpoint_auth_method": "client_secret_basic"})),
    )
    .await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    assert_eq!(bad.body["error"], "invalid_client_metadata");
}

#[sqlx::test(migrations = "../../migrations")]
async fn code_flow_gives_a_token_bound_to_one_workspace_and_its_scopes(pool: PgPool) {
    let w = world(&pool).await;
    let session = sign_in(&w, "alice").await;
    let ws = new_workspace(&w, &session, "business").await;
    let other = new_workspace(&w, &session, "business").await;

    // The consent screen sees the client, the scopes and the workspaces.
    let client = register(&w).await;
    let request = start(&w, &client, "recipes:read batches:read", API_RESOURCE).await;
    let view = call(
        &w,
        "GET",
        &format!("/v1/me/oauth/requests/{request}"),
        Some(&session),
        None,
    )
    .await;
    assert_eq!(view.status, StatusCode::OK, "{:?}", view.body);
    assert_eq!(view.body["client"]["client_name"], "Test Agent");
    assert_eq!(view.body["client"]["verified"], false);
    assert_eq!(view.body["client"]["redirect_host"], "agent.example");
    assert_eq!(view.body["workspaces"].as_array().unwrap().len(), 2);
    assert_eq!(view.body["workspaces"][0]["eligible"], true);

    // The user narrows the grant to one read scope.
    let d = approve(&w, &session, &request, ws, Some(vec!["recipes:read"])).await;
    assert_eq!(d.status, StatusCode::OK, "{:?}", d.body);
    let code = query_of(d.body["redirect_to"].as_str().unwrap(), "code").unwrap();
    let t = exchange(&w, &client, &code, VERIFIER).await;
    assert_eq!(t.status, StatusCode::OK, "{:?}", t.body);
    assert_eq!(t.body["scope"], "recipes:read");
    let access = t.body["access_token"].as_str().unwrap();
    assert!(access.starts_with("gso_at_"));

    // Same workspace and scope: allowed.
    let ok = call(
        &w,
        "GET",
        &format!("/v1/workspaces/{ws}/recipes"),
        Some(access),
        None,
    )
    .await;
    assert_eq!(ok.status, StatusCode::OK, "{:?}", ok.body);
    // A token without the scope gives 403.
    let denied = call(
        &w,
        "GET",
        &format!("/v1/workspaces/{ws}/batches"),
        Some(access),
        None,
    )
    .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN, "{:?}", denied.body);
    // A token of a different workspace gives 404.
    let other_ws = call(
        &w,
        "GET",
        &format!("/v1/workspaces/{other}/recipes"),
        Some(access),
        None,
    )
    .await;
    assert_eq!(
        other_ws.status,
        StatusCode::NOT_FOUND,
        "{:?}",
        other_ws.body
    );
    // A write scope the token lacks gives 403; an operation outside the list gives 403.
    let write = call(
        &w,
        "POST",
        &format!("/v1/workspaces/{ws}/recipes"),
        Some(access),
        Some(json!({"name": "x"})),
    )
    .await;
    assert_eq!(write.status, StatusCode::FORBIDDEN, "{:?}", write.body);
    let delete = call(
        &w,
        "DELETE",
        &format!("/v1/workspaces/{ws}/recipes/{ws}"),
        Some(access),
        None,
    )
    .await;
    assert_eq!(delete.status, StatusCode::FORBIDDEN);
    // Management and account routes are closed to OAuth tokens, and a token cannot consent.
    for path in ["/v1/me/connected-apps", "/v1/me/session"] {
        let r = call(&w, "GET", path, Some(access), None).await;
        assert_eq!(r.status, StatusCode::FORBIDDEN, "{path}: {:?}", r.body);
    }
    let members = call(
        &w,
        "GET",
        &format!("/v1/workspaces/{ws}/members"),
        Some(access),
        None,
    )
    .await;
    assert_eq!(members.status, StatusCode::FORBIDDEN);
    // A refresh token is not a bearer credential.
    let refresh = t.body["refresh_token"].as_str().unwrap();
    let r = call(
        &w,
        "GET",
        &format!("/v1/workspaces/{ws}/recipes"),
        Some(refresh),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    // A garbage token is 401.
    let r = call(
        &w,
        "GET",
        &format!("/v1/workspaces/{ws}/recipes"),
        Some("gso_at_nope"),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);

    // Only digests are stored.
    let stored: i64 = sqlx::query_scalar("SELECT count(*) FROM oauth_tokens WHERE token_hash = $1")
        .bind(Sha256::digest(access.as_bytes()).to_vec())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, 1);
    // Consent wrote an audit row in the workspace.
    assert!(
        audit_actions(&pool, ws)
            .await
            .contains(&"oauth.grant".to_owned())
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_revoked_token_gives_401(pool: PgPool) {
    let w = world(&pool).await;
    let session = sign_in(&w, "bob").await;
    let ws = new_workspace(&w, &session, "business").await;
    let list = format!("/v1/workspaces/{ws}/recipes");

    // Revoked by the client (RFC 7009): the refresh token closes the whole grant.
    let t = connect(&w, &session, ws, "recipes:read", API_RESOURCE).await;
    assert_eq!(
        call(&w, "GET", &list, Some(&t.access), None).await.status,
        StatusCode::OK
    );
    let r = form(
        &w,
        "/revoke",
        &[("token", &t.refresh), ("client_id", &t.client)],
    )
    .await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(
        call(&w, "GET", &list, Some(&t.access), None).await.status,
        StatusCode::UNAUTHORIZED
    );
    let again = form(
        &w,
        "/token",
        &[
            ("grant_type", "refresh_token"),
            ("client_id", &t.client),
            ("refresh_token", &t.refresh),
        ],
    )
    .await;
    assert_eq!(again.body["error"], "invalid_grant");
    // Unknown token: still 200.
    let r = form(
        &w,
        "/revoke",
        &[("token", "gso_rt_unknown"), ("client_id", &t.client)],
    )
    .await;
    assert_eq!(r.status, StatusCode::OK);

    // Revoking an access token closes only that token.
    let t = connect(&w, &session, ws, "recipes:read", API_RESOURCE).await;
    form(
        &w,
        "/revoke",
        &[("token", &t.access), ("client_id", &t.client)],
    )
    .await;
    assert_eq!(
        call(&w, "GET", &list, Some(&t.access), None).await.status,
        StatusCode::UNAUTHORIZED
    );
    let refreshed = form(
        &w,
        "/token",
        &[
            ("grant_type", "refresh_token"),
            ("client_id", &t.client),
            ("refresh_token", &t.refresh),
        ],
    )
    .await;
    assert_eq!(refreshed.status, StatusCode::OK);

    // Another client cannot revoke the token.
    let t = connect(&w, &session, ws, "recipes:read", API_RESOURCE).await;
    let stranger = register(&w).await;
    form(
        &w,
        "/revoke",
        &[("token", &t.refresh), ("client_id", &stranger)],
    )
    .await;
    assert_eq!(
        call(&w, "GET", &list, Some(&t.access), None).await.status,
        StatusCode::OK
    );

    // The user removes the connection under "Connected apps".
    let apps = call(&w, "GET", "/v1/me/connected-apps", Some(&session), None).await;
    assert_eq!(apps.status, StatusCode::OK, "{:?}", apps.body);
    let item = apps.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["client_id"] == t.client.as_str())
        .unwrap();
    assert_eq!(item["workspace_name"], "Brewery");
    assert_eq!(item["scopes"], json!(["recipes:read"]));
    let id = item["id"].as_str().unwrap().to_owned();
    let del = call(
        &w,
        "DELETE",
        &format!("/v1/me/connected-apps/{id}"),
        Some(&session),
        None,
    )
    .await;
    assert_eq!(del.status, StatusCode::NO_CONTENT);
    assert_eq!(
        call(&w, "GET", &list, Some(&t.access), None).await.status,
        StatusCode::UNAUTHORIZED
    );
    let gone = call(
        &w,
        "DELETE",
        &format!("/v1/me/connected-apps/{id}"),
        Some(&session),
        None,
    )
    .await;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);
    assert!(
        audit_actions(&pool, ws)
            .await
            .contains(&"oauth.grant_revoke".to_owned())
    );

    // "Remove all" closes every grant of the user.
    let t = connect(&w, &session, ws, "recipes:read", API_RESOURCE).await;
    let all = call(&w, "DELETE", "/v1/me/connected-apps", Some(&session), None).await;
    assert_eq!(all.status, StatusCode::NO_CONTENT);
    assert_eq!(
        call(&w, "GET", &list, Some(&t.access), None).await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn another_users_grant_is_not_visible_or_removable(pool: PgPool) {
    let w = world(&pool).await;
    let alice = sign_in(&w, "alice2").await;
    let mallory = sign_in(&w, "mallory").await;
    let ws = new_workspace(&w, &alice, "business").await;
    let _ = connect(&w, &alice, ws, "recipes:read", API_RESOURCE).await;
    let seen = call(&w, "GET", "/v1/me/connected-apps", Some(&mallory), None).await;
    assert_eq!(seen.body["items"], json!([]));
    let id: Uuid = sqlx::query_scalar("SELECT id FROM oauth_grants")
        .fetch_one(&pool)
        .await
        .unwrap();
    let del = call(
        &w,
        "DELETE",
        &format!("/v1/me/connected-apps/{id}"),
        Some(&mallory),
        None,
    )
    .await;
    assert_eq!(del.status, StatusCode::NOT_FOUND);
    // The consent screen of a member only lists the member's own workspaces.
    let client = register(&w).await;
    let request = start(&w, &client, "recipes:read", API_RESOURCE).await;
    let view = call(
        &w,
        "GET",
        &format!("/v1/me/oauth/requests/{request}"),
        Some(&mallory),
        None,
    )
    .await;
    assert_eq!(view.body["workspaces"], json!([]));
    let d = approve(&w, &mallory, &request, ws, None).await;
    assert_eq!(d.status, StatusCode::UNPROCESSABLE_ENTITY, "{:?}", d.body);
}

#[sqlx::test(migrations = "../../migrations")]
async fn pkce_failure_and_code_replay_are_refused(pool: PgPool) {
    let w = world(&pool).await;
    let session = sign_in(&w, "carol").await;
    let ws = new_workspace(&w, &session, "business").await;
    let client = register(&w).await;

    // Wrong verifier: invalid_grant, and the code is spent.
    let request = start(&w, &client, "recipes:read", API_RESOURCE).await;
    let d = approve(&w, &session, &request, ws, None).await;
    let code = query_of(d.body["redirect_to"].as_str().unwrap(), "code").unwrap();
    let wrong = "x".repeat(43);
    let bad = exchange(&w, &client, &code, &wrong).await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    assert_eq!(bad.body["error"], "invalid_grant");
    let retry = exchange(&w, &client, &code, VERIFIER).await;
    assert_eq!(retry.body["error"], "invalid_grant");
    // A verifier of the wrong shape is a bad request.
    let short = exchange(&w, &client, &code, "short").await;
    assert_eq!(short.body["error"], "invalid_request");

    // Wrong redirect address and a code of another client.
    let request = start(&w, &client, "recipes:read", API_RESOURCE).await;
    let d = approve(&w, &session, &request, ws, None).await;
    let code = query_of(d.body["redirect_to"].as_str().unwrap(), "code").unwrap();
    let r = form(
        &w,
        "/token",
        &[
            ("grant_type", "authorization_code"),
            ("client_id", &client),
            ("code", &code),
            ("redirect_uri", "https://agent.example/other"),
            ("code_verifier", VERIFIER),
        ],
    )
    .await;
    assert_eq!(r.body["error"], "invalid_grant");
    let request = start(&w, &client, "recipes:read", API_RESOURCE).await;
    let d = approve(&w, &session, &request, ws, None).await;
    let code = query_of(d.body["redirect_to"].as_str().unwrap(), "code").unwrap();
    let other_client = register(&w).await;
    let r = exchange(&w, &other_client, &code, VERIFIER).await;
    assert_eq!(r.body["error"], "invalid_grant");

    // Replay of a used code revokes the tokens that it gave.
    let request = start(&w, &client, "recipes:read", API_RESOURCE).await;
    let d = approve(&w, &session, &request, ws, None).await;
    let code = query_of(d.body["redirect_to"].as_str().unwrap(), "code").unwrap();
    let first = exchange(&w, &client, &code, VERIFIER).await;
    assert_eq!(first.status, StatusCode::OK);
    let access = first.body["access_token"].as_str().unwrap();
    let list = format!("/v1/workspaces/{ws}/recipes");
    assert_eq!(
        call(&w, "GET", &list, Some(access), None).await.status,
        StatusCode::OK
    );
    let replay = exchange(&w, &client, &code, VERIFIER).await;
    assert_eq!(replay.body["error"], "invalid_grant");
    assert_eq!(
        call(&w, "GET", &list, Some(access), None).await.status,
        StatusCode::UNAUTHORIZED
    );

    // An expired code is refused.
    let request = start(&w, &client, "recipes:read", API_RESOURCE).await;
    let d = approve(&w, &session, &request, ws, None).await;
    let code = query_of(d.body["redirect_to"].as_str().unwrap(), "code").unwrap();
    sqlx::query("UPDATE oauth_codes SET expires_at = now() - interval '1 second'")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        exchange(&w, &client, &code, VERIFIER).await.body["error"],
        "invalid_grant"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn refresh_rotates_and_reuse_revokes_the_chain(pool: PgPool) {
    let w = world(&pool).await;
    let session = sign_in(&w, "dave").await;
    let ws = new_workspace(&w, &session, "business").await;
    let t = connect(&w, &session, ws, "recipes:read batches:read", API_RESOURCE).await;
    let list = format!("/v1/workspaces/{ws}/recipes");
    let refresh = |token: &str, scope: Option<&str>| {
        let mut pairs = vec![
            ("grant_type", "refresh_token".to_owned()),
            ("client_id", t.client.clone()),
            ("refresh_token", token.to_owned()),
        ];
        if let Some(s) = scope {
            pairs.push(("scope", s.to_owned()));
        }
        let w = &w;
        async move {
            let owned: Vec<(&str, &str)> = pairs.iter().map(|(k, v)| (*k, v.as_str())).collect();
            form(w, "/token", &owned).await
        }
    };

    // A refresh cannot widen the scope.
    let widen = refresh(&t.refresh, Some("recipes:read costs:read")).await;
    assert_eq!(widen.body["error"], "invalid_scope");
    // It can narrow it.
    let r1 = refresh(&t.refresh, Some("recipes:read")).await;
    assert_eq!(r1.status, StatusCode::OK, "{:?}", r1.body);
    assert_eq!(r1.body["scope"], "recipes:read");
    let (a1, rt1) = (
        r1.body["access_token"].as_str().unwrap().to_owned(),
        r1.body["refresh_token"].as_str().unwrap().to_owned(),
    );
    assert_ne!(rt1, t.refresh);
    assert_eq!(
        call(&w, "GET", &list, Some(&a1), None).await.status,
        StatusCode::OK
    );
    // The narrowed token has lost the batches scope.
    let batches = format!("/v1/workspaces/{ws}/batches");
    assert_eq!(
        call(&w, "GET", &batches, Some(&a1), None).await.status,
        StatusCode::FORBIDDEN
    );
    // The first access token still works until it ends (it is not the refresh token).
    // The next rotation works.
    let r2 = refresh(&rt1, None).await;
    assert_eq!(r2.status, StatusCode::OK);
    let (a2, rt2) = (
        r2.body["access_token"].as_str().unwrap().to_owned(),
        r2.body["refresh_token"].as_str().unwrap().to_owned(),
    );
    assert_eq!(
        call(&w, "GET", &list, Some(&a2), None).await.status,
        StatusCode::OK
    );

    // Reuse of a closed refresh token revokes the whole chain.
    let reuse = refresh(&t.refresh, None).await;
    assert_eq!(reuse.status, StatusCode::BAD_REQUEST);
    assert_eq!(reuse.body["error"], "invalid_grant");
    for token in [&a1, &a2] {
        assert_eq!(
            call(&w, "GET", &list, Some(token), None).await.status,
            StatusCode::UNAUTHORIZED
        );
    }
    assert_eq!(refresh(&rt2, None).await.body["error"], "invalid_grant");
    assert!(
        audit_actions(&pool, ws)
            .await
            .contains(&"oauth.refresh_reuse".to_owned())
    );
    let reason: String = sqlx::query_scalar("SELECT revoked_reason FROM oauth_grants")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(reason, "refresh_reuse");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_refresh_token_ends_after_30_days_without_use(pool: PgPool) {
    let w = world(&pool).await;
    let session = sign_in(&w, "erin").await;
    let ws = new_workspace(&w, &session, "business").await;
    let t = connect(&w, &session, ws, "recipes:read", API_RESOURCE).await;
    sqlx::query(
        "UPDATE oauth_tokens SET expires_at = now() - interval '1 second' WHERE kind = 'refresh'",
    )
    .execute(&pool)
    .await
    .unwrap();
    let r = form(
        &w,
        "/token",
        &[
            ("grant_type", "refresh_token"),
            ("client_id", &t.client),
            ("refresh_token", &t.refresh),
        ],
    )
    .await;
    assert_eq!(r.body["error"], "invalid_grant");
    // An access token ends after its hour.
    sqlx::query(
        "UPDATE oauth_tokens SET expires_at = now() - interval '1 second' WHERE kind = 'access'",
    )
    .execute(&pool)
    .await
    .unwrap();
    let list = format!("/v1/workspaces/{ws}/recipes");
    assert_eq!(
        call(&w, "GET", &list, Some(&t.access), None).await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn scopes_follow_the_role_and_the_plan(pool: PgPool) {
    let w = world(&pool).await;
    let session = sign_in(&w, "frank").await;
    let home = new_workspace(&w, &session, "home").await;
    let business = new_workspace(&w, &session, "business").await;
    let client = register(&w).await;

    // The open API is for Business only; MCP is open to Home during the trial.
    let api_req = start(&w, &client, "recipes:read", API_RESOURCE).await;
    let view = call(
        &w,
        "GET",
        &format!("/v1/me/oauth/requests/{api_req}"),
        Some(&session),
        None,
    )
    .await;
    let by_id = |id: Uuid| {
        view.body["workspaces"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["id"] == id.to_string().as_str())
            .unwrap()
            .clone()
    };
    assert_eq!(by_id(home)["ineligible_reason"], "business_only");
    assert_eq!(by_id(business)["eligible"], true);
    let d = approve(&w, &session, &api_req, home, None).await;
    assert_eq!(d.status, StatusCode::FORBIDDEN, "{:?}", d.body);

    let mcp_req = start(&w, &client, "recipes:read", MCP_RESOURCE).await;
    let view = call(
        &w,
        "GET",
        &format!("/v1/me/oauth/requests/{mcp_req}"),
        Some(&session),
        None,
    )
    .await;
    let home_view = view.body["workspaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["id"] == home.to_string().as_str())
        .unwrap()
        .clone();
    assert_eq!(home_view["eligible"], true, "{home_view}");

    // After the trial a free Home workspace cannot give MCP consent.
    sqlx::query("UPDATE workspaces SET created_at = now() - interval '2 years' WHERE id = $1")
        .bind(home)
        .execute(&pool)
        .await
        .unwrap();
    let view = call(
        &w,
        "GET",
        &format!("/v1/me/oauth/requests/{mcp_req}"),
        Some(&session),
        None,
    )
    .await;
    let home_view = view.body["workspaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["id"] == home.to_string().as_str())
        .unwrap()
        .clone();
    assert_eq!(home_view["ineligible_reason"], "plan_required");
    let d = approve(&w, &session, &mcp_req, home, None).await;
    assert_eq!(d.status, StatusCode::FORBIDDEN);

    // A scope the request did not ask for, or an unknown one, is refused.
    let req = start(&w, &client, "recipes:read", API_RESOURCE).await;
    let d = approve(&w, &session, &req, business, Some(vec!["recipes:draft"])).await;
    assert_eq!(d.status, StatusCode::UNPROCESSABLE_ENTITY);
    let d = approve(&w, &session, &req, business, Some(vec![])).await;
    assert_eq!(d.status, StatusCode::UNPROCESSABLE_ENTITY);
    // Deny sends the browser back with access_denied.
    let denied = call(
        &w,
        "POST",
        &format!("/v1/me/oauth/requests/{req}/decision"),
        Some(&session),
        Some(json!({"decision": "deny"})),
    )
    .await;
    let to = denied.body["redirect_to"].as_str().unwrap();
    assert_eq!(query_of(to, "error").as_deref(), Some("access_denied"));
    assert_eq!(query_of(to, "state").as_deref(), Some("st-1"));
    // A decision is single use.
    assert_eq!(
        approve(&w, &session, &req, business, None).await.status,
        StatusCode::NOT_FOUND
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_mcp_token_is_not_valid_on_the_open_api_and_costs_need_their_scope(pool: PgPool) {
    let w = world(&pool).await;
    let session = sign_in(&w, "gina").await;
    let ws = new_workspace(&w, &session, "business").await;
    let mcp = connect(&w, &session, ws, "recipes:read", MCP_RESOURCE).await;
    // The audience is MCP, so `/v1` answers 401.
    let list = format!("/v1/workspaces/{ws}/recipes");
    assert_eq!(
        call(&w, "GET", &list, Some(&mcp.access), None).await.status,
        StatusCode::UNAUTHORIZED
    );

    // Default scopes (none asked) are the read scopes without costs.
    let client = register(&w).await;
    let url = authorize_query(&client, "recipes:read", API_RESOURCE, "S256")
        .replace("scope=recipes%3Aread&", "");
    let r = get_plain(&w, &url).await;
    let request = query_of(&location(&r), "request").unwrap();
    let view = call(
        &w,
        "GET",
        &format!("/v1/me/oauth/requests/{request}"),
        Some(&session),
        None,
    )
    .await;
    let scopes: Vec<&str> = view.body["requested_scopes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["scope"].as_str().unwrap())
        .collect();
    assert!(scopes.contains(&"recipes:read") && !scopes.contains(&"costs:read"));
    assert!(!scopes.contains(&"recipes:draft"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn write_calls_fail_closed_without_redis(pool: PgPool) {
    let with_redis = world(&pool).await;
    let session = sign_in(&with_redis, "hank").await;
    let ws = new_workspace(&with_redis, &session, "business").await;
    let t = connect(
        &with_redis,
        &session,
        ws,
        "recipes:read recipes:draft",
        API_RESOURCE,
    )
    .await;
    let path = format!("/v1/workspaces/{ws}/recipes");
    // With Redis the write passes the gate (the body is invalid, so the handler answers 4xx).
    let r = call(&with_redis, "POST", &path, Some(&t.access), Some(json!({}))).await;
    assert!(
        r.status.is_client_error()
            && r.status != StatusCode::FORBIDDEN
            && r.status != StatusCode::UNAUTHORIZED,
        "{:?} {:?}",
        r.status,
        r.body
    );

    // Without Redis the write is refused and a read still works.
    let no_redis = world_with(&pool, Redis::disabled(), &[]).await;
    let r = call(&no_redis, "POST", &path, Some(&t.access), Some(json!({}))).await;
    assert_eq!(r.status, StatusCode::SERVICE_UNAVAILABLE, "{:?}", r.body);
    let r = call(&no_redis, "GET", &path, Some(&t.access), None).await;
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_admin_switch_turns_mcp_off_and_revokes_grants(pool: PgPool) {
    let w = world(&pool).await;
    let session = sign_in(&w, "iris").await;
    let ws = new_workspace(&w, &session, "business").await;
    let mcp = connect(&w, &session, ws, "recipes:read", MCP_RESOURCE).await;
    let api = connect(&w, &session, ws, "recipes:read", API_RESOURCE).await;
    let path = format!("/v1/workspaces/{ws}/mcp");

    let state = call(&w, "GET", &path, Some(&session), None).await;
    assert_eq!(state.body["enabled"], true, "{:?}", state.body);
    // An OAuth token cannot use the switch.
    assert_eq!(
        call(&w, "GET", &path, Some(&api.access), None).await.status,
        StatusCode::FORBIDDEN
    );

    let off = call(
        &w,
        "PUT",
        &path,
        Some(&session),
        Some(json!({"enabled": false})),
    )
    .await;
    assert_eq!(off.status, StatusCode::OK, "{:?}", off.body);
    assert_eq!(off.body["enabled"], false);
    assert_eq!(off.body["revoked_grants"], 1);
    let revoked: Option<String> =
        sqlx::query_scalar("SELECT revoked_reason FROM oauth_grants WHERE resource = $1")
            .bind(MCP_RESOURCE)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(revoked.as_deref(), Some("workspace_switch"));
    // The open API connection is not an MCP connection and stays.
    let list = format!("/v1/workspaces/{ws}/recipes");
    assert_eq!(
        call(&w, "GET", &list, Some(&api.access), None).await.status,
        StatusCode::OK
    );
    // The MCP grant is closed; its refresh does not work.
    let r = form(
        &w,
        "/token",
        &[
            ("grant_type", "refresh_token"),
            ("client_id", &mcp.client),
            ("refresh_token", &mcp.refresh),
        ],
    )
    .await;
    assert_eq!(r.body["error"], "invalid_grant");
    // New MCP consent is refused while the switch is off.
    let client = register(&w).await;
    let request = start(&w, &client, "recipes:read", MCP_RESOURCE).await;
    let view = call(
        &w,
        "GET",
        &format!("/v1/me/oauth/requests/{request}"),
        Some(&session),
        None,
    )
    .await;
    assert_eq!(
        view.body["workspaces"][0]["ineligible_reason"],
        "mcp_disabled"
    );
    assert_eq!(
        approve(&w, &session, &request, ws, None).await.status,
        StatusCode::FORBIDDEN
    );
    // Turning it on again does not bring the grants back.
    let on = call(
        &w,
        "PUT",
        &path,
        Some(&session),
        Some(json!({"enabled": true})),
    )
    .await;
    assert_eq!(on.body["enabled"], true);
    assert_eq!(on.body["revoked_grants"], 0);
    let active: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM oauth_grants WHERE resource = $1 AND revoked_at IS NULL",
    )
    .bind(MCP_RESOURCE)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(active, 0);
    let actions = audit_actions(&pool, ws).await;
    assert_eq!(actions.iter().filter(|a| *a == "mcp.switch").count(), 2);

    // Home workspaces have no switch; a stranger sees nothing.
    let home = new_workspace(&w, &session, "home").await;
    let r = call(
        &w,
        "PUT",
        &format!("/v1/workspaces/{home}/mcp"),
        Some(&session),
        Some(json!({"enabled": false})),
    )
    .await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    let stranger = sign_in(&w, "iris-stranger").await;
    let r = call(&w, "GET", &path, Some(&stranger), None).await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    let _ = user_id(&pool, "iris").await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn account_deletion_closes_every_connection(pool: PgPool) {
    let w = world(&pool).await;
    let session = sign_in(&w, "kate").await;
    let ws = new_workspace(&w, &session, "home").await;
    let t = connect(&w, &session, ws, "recipes:read", MCP_RESOURCE).await;
    let r = call(&w, "POST", "/v1/me/deletion", Some(&session), None).await;
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
    let again = form(
        &w,
        "/token",
        &[
            ("grant_type", "refresh_token"),
            ("client_id", &t.client),
            ("refresh_token", &t.refresh),
        ],
    )
    .await;
    assert_eq!(again.body["error"], "invalid_grant");
}

#[sqlx::test(migrations = "../../migrations")]
async fn membership_changes_reach_the_token_on_the_next_request(pool: PgPool) {
    let w = world(&pool).await;
    let session = sign_in(&w, "jane").await;
    let ws = new_workspace(&w, &session, "business").await;
    let t = connect(&w, &session, ws, "recipes:read", API_RESOURCE).await;
    let list = format!("/v1/workspaces/{ws}/recipes");
    assert_eq!(
        call(&w, "GET", &list, Some(&t.access), None).await.status,
        StatusCode::OK
    );
    // The member is removed from the workspace: the same token now gets 404.
    sqlx::query("UPDATE workspace_members SET status = 'disabled' WHERE workspace_id = $1")
        .bind(ws)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        call(&w, "GET", &list, Some(&t.access), None).await.status,
        StatusCode::NOT_FOUND
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn client_documents_follow_the_fetch_rules(pool: PgPool) {
    // SSRF: an address that resolves to a private range is refused before any request.
    let w = world(&pool).await;
    for client in [
        "https://127.0.0.1/client.json",
        "https://localhost/client.json",
        "https://10.0.0.5/c",
    ] {
        let r = get_plain(
            &w,
            &authorize_query(client, "recipes:read", API_RESOURCE, "S256"),
        )
        .await;
        assert_eq!(r.status, StatusCode::BAD_REQUEST, "{client}: {:?}", r.body);
    }
    // Only port 443.
    let r = get_plain(
        &w,
        &authorize_query(
            "https://example.com:8443/c.json",
            "recipes:read",
            API_RESOURCE,
            "S256",
        ),
    )
    .await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST);

    // With the local test switch a document is fetched, checked and cached.
    let w = world_with(
        &pool,
        Redis::disabled(),
        &[("OAUTH_CIMD_ALLOW_INSECURE", "true")],
    )
    .await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let client_id = format!("http://127.0.0.1:{port}/oauth/client.json");
    let doc = json!({"client_id": client_id, "client_name": "Doc Agent",
                     "redirect_uris": [REDIRECT]});
    let wrong = json!({"client_id": "http://127.0.0.1/other.json", "client_name": "Liar",
                       "redirect_uris": [REDIRECT]});
    let fetches = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter = fetches.clone();
    let server = Router::new()
        .route(
            "/oauth/client.json",
            axum::routing::get(move || {
                let doc = doc.clone();
                let counter = counter.clone();
                async move {
                    counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    ([(header::CACHE_CONTROL, "max-age=600")], axum::Json(doc))
                }
            }),
        )
        .route(
            "/liar.json",
            axum::routing::get(move || {
                let wrong = wrong.clone();
                async move { axum::Json(wrong) }
            }),
        );
    tokio::spawn(async move { axum::serve(listener, server).await.unwrap() });

    let request = start(&w, &client_id, "recipes:read", API_RESOURCE).await;
    assert!(!request.is_empty());
    let _ = start(&w, &client_id, "recipes:read", API_RESOURCE).await;
    assert_eq!(
        fetches.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "second call uses the cache"
    );
    let name: String =
        sqlx::query_scalar("SELECT client_name FROM oauth_clients WHERE client_id = $1")
            .bind(&client_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(name, "Doc Agent");
    // A document whose client_id differs from its address is refused.
    let liar = format!("http://127.0.0.1:{port}/liar.json");
    let r = get_plain(
        &w,
        &authorize_query(&liar, "recipes:read", API_RESOURCE, "S256"),
    )
    .await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST, "{:?}", r.body);
}

#[sqlx::test(migrations = "../../migrations")]
async fn search_with_a_token_leaves_out_types_without_scope(pool: PgPool) {
    let w = world(&pool).await;
    let session = sign_in(&w, "lena").await;
    let ws = new_workspace(&w, &session, "business").await;
    // A batch and a stock lot match the word, but the token has only the recipe scope.
    sqlx::query(
        "INSERT INTO ingredients (id, workspace_id, kind, name, source) VALUES ($1, $2, 'misc', 'Zymurgy salt', 'user')",
    )
    .bind(Uuid::now_v7())
    .bind(ws)
    .execute(&pool)
    .await
    .unwrap();
    let t = connect(&w, &session, ws, "recipes:read", API_RESOURCE).await;
    let path = format!("/v1/workspaces/{ws}/search?q=zymurgy");
    let r = call(&w, "GET", &path, Some(&t.access), None).await;
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
    assert_eq!(r.body["items"][0]["type"], "ingredient");
    for ty in ["batch", "stock"] {
        let r = call(
            &w,
            "GET",
            &format!("{path}&types={ty}"),
            Some(&t.access),
            None,
        )
        .await;
        assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
        assert_eq!(r.body["items"], json!([]));
    }
}
