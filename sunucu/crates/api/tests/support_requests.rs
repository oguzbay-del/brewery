//! Support request endpoints of the customer (T2.29). Needs PostgreSQL.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::sync::Arc;

use api::redis_client::Redis;
use axum::{body::Body, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{FakeAuth, config, insert_user, insert_workspace, req, send, state_with};
use uuid::Uuid;

fn call(method: &str, path: &str, user: Uuid, body: Option<&Value>) -> axum::http::Request<Body> {
    let mut r = req(method, path, Some(user))
        .header("content-type", "application/json")
        .body(body.map_or_else(Body::empty, |v| Body::from(v.to_string())))
        .unwrap();
    if method == "POST" {
        r.headers_mut()
            .insert("idempotency-key", Uuid::now_v7().to_string().parse().unwrap());
    }
    r
}

struct World {
    app: axum::Router,
    ws: Uuid,
    user: Uuid,
    other: Uuid,
    pool: PgPool,
}

async fn world(pool: PgPool) -> World {
    let (ws, user, other) = (Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7());
    insert_workspace(&pool, ws).await;
    insert_user(&pool, user).await;
    insert_user(&pool, other).await;
    let auth = Arc::new(FakeAuth::default());
    auth.add(user, ws, &[]);
    auth.add(other, ws, &[]);
    let state = state_with(&pool, Redis::disabled(), auth, config("postgres://unused")).await;
    World {
        app: api::build(state, axum::Router::new()),
        ws,
        user,
        other,
        pool,
    }
}

async fn open(w: &World) -> (String, String) {
    let r = send(
        &w.app,
        call(
            "POST",
            &format!("/v1/workspaces/{}/support/requests", w.ws),
            w.user,
            Some(&json!({"topic": "recipes", "title": "Cannot publish",
                "body": "The button does nothing.",
                "diagnostics": {"app_version": "2.3.1"}})),
        ),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    (
        r.body["request"]["id"].as_str().unwrap().to_owned(),
        format!(
            "/v1/workspaces/{}/support/requests/{}",
            w.ws,
            r.body["request"]["id"].as_str().unwrap()
        ),
    )
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_request_opens_with_a_message_and_queues_the_sync(pool: PgPool) {
    let w = world(pool).await;
    let (id, path) = open(&w).await;
    let r = send(&w.app, call("GET", &path, w.user, None)).await;
    assert_eq!(r.body["request"]["status"], "open");
    assert!(r.body["request"]["public_number"].as_i64().unwrap() > 0);
    assert_eq!(r.body["messages"].as_array().unwrap().len(), 1);
    assert_eq!(r.body["messages"][0]["author"], "customer");
    // The sync event waits in the outbox; the request exists although GitHub was never asked.
    let events: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM outbox WHERE topic = 'support.sync_requested'",
    )
    .fetch_one(&w.pool)
    .await
    .unwrap();
    assert_eq!(events, 1);
    let issue: Option<i32> = sqlx::query_scalar("SELECT github_issue_number FROM support_requests")
        .fetch_one(&w.pool)
        .await
        .unwrap();
    assert_eq!(issue, None);

    // Only the opener sees the request.
    let r = send(&w.app, call("GET", &path, w.other, None)).await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    let r = send(
        &w.app,
        call("GET", &format!("/v1/workspaces/{}/support/requests", w.ws), w.other, None),
    )
    .await;
    assert_eq!(r.body["items"].as_array().unwrap().len(), 0);
    let r = send(
        &w.app,
        call("GET", &format!("/v1/workspaces/{}/support/requests", w.ws), w.user, None),
    )
    .await;
    assert_eq!(r.body["items"][0]["id"], id);

    // Validation.
    for bad in [
        json!({"topic": "nope", "title": "t", "body": "b"}),
        json!({"topic": "other", "title": "", "body": "b"}),
        json!({"topic": "other", "title": "t", "body": "  "}),
        json!({"topic": "other", "title": "t", "body": "b", "related_record_kind": "batch"}),
    ] {
        let r = send(
            &w.app,
            call("POST", &format!("/v1/workspaces/{}/support/requests", w.ws), w.user, Some(&bad)),
        )
        .await;
        assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{bad}");
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn internal_notes_never_show_and_messages_follow_the_status_rules(pool: PgPool) {
    let w = world(pool).await;
    let (id, path) = open(&w).await;
    let request: Uuid = id.parse().unwrap();
    // An internal note and a customer-visible reply, as the worker writes them.
    sqlx::query(
        "INSERT INTO support_messages (id, workspace_id, request_id, author_kind, body, visibility)
         VALUES ($1, $3, $2, 'system', 'internal: possible cache bug', 'internal'),
                ($4, $3, $2, 'system', 'Please update.', 'customer')",
    )
    .bind(Uuid::now_v7())
    .bind(request)
    .bind(w.ws)
    .bind(Uuid::now_v7())
    .execute(&w.pool)
    .await
    .unwrap();
    sqlx::query("UPDATE support_requests SET status = 'answered'")
        .execute(&w.pool)
        .await
        .unwrap();
    let r = send(&w.app, call("GET", &path, w.user, None)).await;
    let text = r.body.to_string();
    assert!(!text.contains("possible cache bug"), "internal note leaked");
    assert!(text.contains("Please update."));
    assert_eq!(r.body["messages"].as_array().unwrap().len(), 2);

    // A new customer message opens an answered request again.
    let r = send(
        &w.app,
        call("POST", &format!("{path}/messages"), w.user, Some(&json!({"body": "Still broken."}))),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let message = r.body["id"].as_str().unwrap().to_owned();
    let r = send(&w.app, call("GET", &path, w.user, None)).await;
    assert_eq!(r.body["request"]["status"], "open");

    // Edit and retract.
    let m = format!("{path}/messages/{message}");
    let r = send(&w.app, call("PATCH", &m, w.user, Some(&json!({"body": "Still broken, v2."})))).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert!(r.body["edited_at"].is_string());
    let r = send(&w.app, call("PATCH", &m, w.other, Some(&json!({"body": "x"})))).await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    let r = send(&w.app, call("DELETE", &m, w.user, None)).await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    let r = send(&w.app, call("GET", &path, w.user, None)).await;
    let retracted = r.body["messages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["id"] == message.as_str())
        .unwrap();
    assert_eq!((retracted["retracted"].clone(), retracted["body"].clone()), (json!(true), json!("")));

    // Resolve, write again (opens), resolve, close, then no more messages.
    let r = send(&w.app, call("POST", &format!("{path}/resolve"), w.user, None)).await;
    assert_eq!(r.body["request"]["status"], "resolved");
    let r = send(&w.app, call("POST", &format!("{path}/resolve"), w.user, None)).await;
    assert_eq!(r.status, StatusCode::CONFLICT);
    sqlx::query("UPDATE support_requests SET status = 'closed'")
        .execute(&w.pool)
        .await
        .unwrap();
    let r = send(
        &w.app,
        call("POST", &format!("{path}/messages"), w.user, Some(&json!({"body": "Hello?"}))),
    )
    .await;
    assert_eq!(r.status, StatusCode::CONFLICT);

    // A new request continues the closed one.
    let r = send(
        &w.app,
        call(
            "POST",
            &format!("/v1/workspaces/{}/support/requests", w.ws),
            w.user,
            Some(&json!({"topic": "recipes", "title": "Again", "body": "Back.",
                "previous_request_id": id})),
        ),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    assert_eq!(r.body["request"]["previous_request_id"], id);
}
