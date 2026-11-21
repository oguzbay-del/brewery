//! Support panel (`/internal`) and the support access of the customer (T2.30). Needs PostgreSQL.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::sync::Arc;

use api::{internal::InternalConfig, redis_client::Redis};
use axum::{body::Body, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{FakeAuth, Reply, config, insert_user, insert_workspace, req, send, state_with};
use uuid::Uuid;

const TOKEN: &str = "proxy-secret";

#[allow(dead_code)]
struct World {
    app: axum::Router,
    ws: Uuid,
    user: Uuid,
    member: Uuid,
    pool: PgPool,
}

async fn world(pool: PgPool, internal: bool) -> World {
    let (ws, user, member) = (Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7());
    insert_workspace(&pool, ws).await;
    insert_user(&pool, user).await;
    sqlx::query("INSERT INTO workspace_members (id, workspace_id, user_id) VALUES ($1, $2, $3)")
        .bind(member)
        .bind(ws)
        .bind(user)
        .execute(&pool)
        .await
        .unwrap();
    let auth = Arc::new(FakeAuth::default());
    auth.add(user, ws, &["support.grant", "recipe.view"]);
    let mut state = state_with(&pool, Redis::disabled(), auth, config("postgres://unused")).await;
    if internal {
        state = state.with_internal(InternalConfig::new(TOKEN));
    }
    World {
        app: api::build(state, axum::Router::new()),
        ws,
        user,
        member,
        pool,
    }
}

async fn staff(pool: &PgPool, email: &str, role: &str) {
    sqlx::query(
        "INSERT INTO staff_users (id, google_subject, email, role) VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(format!("sub-{email}"))
    .bind(email)
    .bind(role)
    .execute(pool)
    .await
    .unwrap();
}

async fn panel(
    w: &World,
    method: &str,
    path: &str,
    email: Option<&str>,
    body: Option<Value>,
) -> Reply {
    let mut b = req(method, path, None).header("content-type", "application/json");
    b = b.header("x-internal-proxy-token", TOKEN);
    if let Some(e) = email {
        b = b.header("x-auth-request-email", e);
    }
    send(
        &w.app,
        b.body(body.map_or_else(Body::empty, |v| Body::from(v.to_string())))
            .unwrap(),
    )
    .await
}

async fn customer(w: &World, method: &str, path: &str, body: Option<Value>) -> Reply {
    let mut r = req(method, path, Some(w.user))
        .header("content-type", "application/json")
        .body(body.map_or_else(Body::empty, |v| Body::from(v.to_string())))
        .unwrap();
    if method == "POST" {
        r.headers_mut().insert(
            "idempotency-key",
            Uuid::now_v7().to_string().parse().unwrap(),
        );
    }
    send(&w.app, r).await
}

async fn count(pool: &PgPool, sql: &'static str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn the_panel_needs_the_proxy_token_and_a_staff_member(pool: PgPool) {
    let w = world(pool.clone(), true).await;
    staff(&pool, "ann@company.example", "support").await;
    let path = format!("/internal/v1/workspaces/{}/summary?reason=check", w.ws);
    let r = send(&w.app, req("GET", &path, None).body(Body::empty()).unwrap()).await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    let r = send(
        &w.app,
        req("GET", &path, None)
            .header("x-internal-proxy-token", "wrong")
            .header("x-auth-request-email", "ann@company.example")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    let r = panel(&w, "GET", &path, Some("stranger@example.com"), None).await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    let r = panel(&w, "GET", &path, None, None).await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    let r = panel(&w, "GET", &path, Some("ann@company.example"), None).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert_eq!(r.body["members"], 1);
    assert_eq!(r.body["grant_active"], false);
    // A reason is mandatory.
    let r = panel(
        &w,
        "GET",
        &format!("/internal/v1/workspaces/{}/summary", w.ws),
        Some("ann@company.example"),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    // Without settings the panel is off.
    let off = world(pool.clone(), false).await;
    let r = panel(&off, "GET", &path, Some("ann@company.example"), None).await;
    assert_eq!(r.status, StatusCode::SERVICE_UNAVAILABLE);
    // The search finds the workspace by pseudonym and gives no content.
    let r = panel(
        &w,
        "GET",
        &format!("/internal/v1/workspaces?q=pseudonym-{}", w.ws),
        Some("ann@company.example"),
        None,
    )
    .await;
    assert_eq!(r.body.as_array().unwrap().len(), 1);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn content_needs_a_valid_grant_and_every_operation_is_logged(pool: PgPool) {
    let w = world(pool.clone(), true).await;
    staff(&pool, "ann@company.example", "support").await;
    sqlx::query("INSERT INTO recipes (id, workspace_id, name) VALUES ($1, $2, 'Secret Stout')")
        .bind(Uuid::now_v7())
        .bind(w.ws)
        .execute(&pool)
        .await
        .unwrap();
    let path = format!(
        "/internal/v1/workspaces/{}/content/recipes?reason=Request%20%234182",
        w.ws
    );
    let ann = Some("ann@company.example");

    // No grant: 403, and the attempt is in the log.
    let r = panel(&w, "GET", &path, ann, None).await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    assert!(!r.body.to_string().contains("Secret Stout"));
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM support_actions WHERE action = 'content.denied.recipes'"
        )
        .await,
        1
    );

    // The customer gives a grant (not more than 72 hours; needs the permission).
    let grants = format!("/v1/workspaces/{}/support/grants", w.ws);
    assert_eq!(
        customer(&w, "POST", &grants, Some(json!({"hours": 100})))
            .await
            .status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let r = customer(&w, "POST", &grants, Some(json!({}))).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let grant = r.body["id"].as_str().unwrap().to_owned();
    let r = panel(&w, "GET", &path, ann, None).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert_eq!(r.body["items"][0]["name"], "Secret Stout");

    // The grant ends: the access stops.
    sqlx::query("UPDATE support_grants SET starts_at = now() - interval '3 hours', ends_at = now() - interval '1 hour'")
        .execute(&pool)
        .await
        .unwrap();
    let r = panel(&w, "GET", &path, ann, None).await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);

    // A new grant, revoked by the customer: the access stops at once.
    let r = customer(&w, "POST", &grants, Some(json!({"hours": 1}))).await;
    let second = r.body["id"].as_str().unwrap().to_owned();
    assert_eq!(
        panel(&w, "GET", &path, ann, None).await.status,
        StatusCode::OK
    );
    let r = customer(&w, "DELETE", &format!("{grants}/{second}"), None).await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    assert_eq!(
        panel(&w, "GET", &path, ann, None).await.status,
        StatusCode::FORBIDDEN
    );
    assert_ne!(grant, second);

    // Every operation is in the action log and in the audit chain.
    let actions = count(&pool, "SELECT count(*) FROM support_actions").await;
    assert_eq!(actions, 5, "denied, read, denied, read, denied");
    let audited = count(
        &pool,
        "SELECT count(*) FROM audit_log WHERE action = 'support.action'",
    )
    .await;
    assert_eq!(audited, actions);
    let grant_audit = count(
        &pool,
        "SELECT count(*) FROM audit_log WHERE action = 'support.grant_change'",
    )
    .await;
    assert_eq!(grant_audit, 3, "given, given, revoked");

    // The customer reads the history: no name or e-mail of the staff member.
    let r = customer(
        &w,
        "GET",
        &format!("/v1/workspaces/{}/support/access-history", w.ws),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body["grants"].as_array().unwrap().len(), 2);
    assert_eq!(r.body["actions"].as_array().unwrap().len(), 5);
    assert_eq!(r.body["actions"][0]["staff_role"], "support");
    assert!(!r.body.to_string().contains("company.example"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn emergency_access_needs_two_different_approvals(pool: PgPool) {
    let w = world(pool.clone(), true).await;
    staff(&pool, "ann@company.example", "support").await;
    staff(&pool, "root1@company.example", "support_admin").await;
    staff(&pool, "root2@company.example", "support_admin").await;
    let base = format!("/internal/v1/workspaces/{}/emergency-access", w.ws);
    let reason = json!({"reason": "Security incident 77"});

    let r = panel(
        &w,
        "POST",
        &base,
        Some("root1@company.example"),
        Some(reason.clone()),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let id = r.body["id"].as_str().unwrap().to_owned();
    let approve = format!("{base}/{id}/approve");
    // The same person, a staff member who is not an admin: refused.
    let r = panel(&w, "POST", &approve, Some("root1@company.example"), None).await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    let r = panel(&w, "POST", &approve, Some("ann@company.example"), None).await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    assert_eq!(count(&pool, "SELECT count(*) FROM support_grants").await, 0);
    // A second admin approves.
    let r = panel(&w, "POST", &approve, Some("root2@company.example"), None).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert_eq!(
        count(&pool, "SELECT count(*) FROM support_grants WHERE emergency AND customer_notified_at IS NOT NULL").await,
        1
    );
    assert_eq!(
        count(&pool, "SELECT count(*) FROM notifications WHERE kind = 'support.emergency_access' AND priority = 1").await,
        1
    );
    let r = panel(&w, "POST", &approve, Some("root2@company.example"), None).await;
    assert_eq!(r.status, StatusCode::CONFLICT);
    // The grant opens content.
    let r = panel(
        &w,
        "GET",
        &format!(
            "/internal/v1/workspaces/{}/content/batches?reason=incident",
            w.ws
        ),
        Some("root2@company.example"),
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    // The customer sees it in the history.
    let r = customer(
        &w,
        "GET",
        &format!("/v1/workspaces/{}/support/access-history", w.ws),
        None,
    )
    .await;
    assert_eq!(r.body["grants"][0]["emergency"], true);
    let _ = w.member;
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn panel_operations(pool: PgPool) {
    let w = world(pool.clone(), true).await;
    staff(&pool, "ann@company.example", "support").await;
    let ann = Some("ann@company.example");
    let ws = w.ws;

    // Sessions.
    sqlx::query(
        "INSERT INTO sessions (id, user_id, token_hash, client_kind, expires_at)
         VALUES ($1, $2, $3, 'web', now() + interval '1 day')",
    )
    .bind(Uuid::now_v7())
    .bind(w.user)
    .bind(vec![7u8; 32])
    .execute(&pool)
    .await
    .unwrap();
    let r = panel(
        &w,
        "POST",
        &format!(
            "/internal/v1/workspaces/{ws}/users/{}/sessions/revoke",
            w.user
        ),
        ann,
        Some(json!({"reason": "Account takeover report"})),
    )
    .await;
    assert_eq!(
        (r.status, r.body["revoked"].clone()),
        (StatusCode::OK, json!(1))
    );
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM sessions WHERE revoked_at IS NULL"
        )
        .await,
        0
    );
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM audit_log WHERE action = 'session.revoke'"
        )
        .await,
        1
    );

    // Device key.
    let device = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO devices (id, workspace_id, kind, name, ingest_key_hash)
         VALUES ($1, $2, 'ispindel', 'Tank 1', $3)",
    )
    .bind(device)
    .bind(ws)
    .bind(vec![1u8; 32])
    .execute(&pool)
    .await
    .unwrap();
    let r = panel(
        &w,
        "POST",
        &format!("/internal/v1/workspaces/{ws}/devices/{device}/rotate-key"),
        ann,
        Some(json!({"reason": "Key leaked in a forum post"})),
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM devices WHERE ingest_key_hash <> decode(repeat('01', 32), 'hex')"
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM audit_log WHERE action = 'device_key.renew'"
        )
        .await,
        1
    );

    // Correction: a request with a reason. The API writes no entitlement.
    let r = panel(
        &w,
        "POST",
        &format!("/internal/v1/workspaces/{ws}/corrections"),
        ann,
        Some(json!({"kind": "entitlement_grant", "reason": "Store outage compensation"})),
    )
    .await;
    assert_eq!(r.status, StatusCode::ACCEPTED, "{}", r.body);
    assert_eq!(count(&pool, "SELECT count(*) FROM entitlements").await, 0);
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM outbox WHERE topic = 'support.correction_requested'"
        )
        .await,
        1
    );
    let r = panel(
        &w,
        "POST",
        &format!("/internal/v1/workspaces/{ws}/corrections"),
        ann,
        Some(json!({"kind": "entitlement_grant"})),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    let r = panel(
        &w,
        "POST",
        &format!("/internal/v1/workspaces/{ws}/corrections"),
        ann,
        Some(json!({"kind": "wipe", "reason": "x"})),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);

    // Reply and note from the panel on a request.
    let request = Uuid::now_v7();
    let number: i64 = sqlx::query_scalar(
        "INSERT INTO support_requests (id, workspace_id, topic, title, created_by)
         VALUES ($1, $2, 'other', 'Help', $3) RETURNING public_number",
    )
    .bind(request)
    .bind(ws)
    .bind(w.user)
    .fetch_one(&pool)
    .await
    .unwrap();
    let messages = format!("/internal/v1/requests/{number}/messages");
    let r = panel(
        &w,
        "POST",
        &messages,
        ann,
        Some(json!({"body": "dev note", "internal": true, "reason": "triage"})),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    assert_eq!(count(&pool, "SELECT count(*) FROM notifications").await, 0);
    let r = panel(
        &w,
        "POST",
        &messages,
        ann,
        Some(json!({"body": "Try again now.", "reason": "answer"})),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM notifications WHERE kind = 'support.reply'"
        )
        .await,
        1
    );
    let status: String = sqlx::query_scalar("SELECT status FROM support_requests")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "answered");
    let r = panel(
        &w,
        "GET",
        &format!("/internal/v1/requests/{number}?reason=read"),
        ann,
        None,
    )
    .await;
    assert_eq!(
        r.body["messages"].as_array().unwrap().len(),
        2,
        "staff see the internal note"
    );
    // The customer does not.
    let r = customer(
        &w,
        "GET",
        &format!("/v1/workspaces/{ws}/support/requests/{request}"),
        None,
    )
    .await;
    assert_eq!(r.body["messages"].as_array().unwrap().len(), 1);
    assert!(!r.body.to_string().contains("dev note"));
    // Unknown number.
    let r = panel(
        &w,
        "GET",
        "/internal/v1/requests/999999?reason=read",
        ann,
        None,
    )
    .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
}
