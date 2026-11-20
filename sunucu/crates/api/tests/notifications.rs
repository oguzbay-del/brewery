//! Notification endpoints (T2.24): inbox, actions, preferences, alert rules. Needs PostgreSQL.

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
    let b = req(method, path, Some(user)).header("content-type", "application/json");
    b.body(body.map_or_else(Body::empty, |v| Body::from(v.to_string())))
        .unwrap()
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
    for (u, m) in [(user, Uuid::now_v7()), (other, Uuid::now_v7())] {
        sqlx::query("INSERT INTO workspace_members (id, workspace_id, user_id) VALUES ($1, $2, $3)")
            .bind(m)
            .bind(ws)
            .bind(u)
            .execute(&pool)
            .await
            .unwrap();
    }
    let auth = Arc::new(FakeAuth::default());
    auth.add(user, ws, &["batch.record", "recipe.view"]);
    auth.add(other, ws, &["recipe.view"]);
    let state = state_with(&pool, Redis::disabled(), auth, config("postgres://unused")).await;
    World {
        app: api::build(state, axum::Router::new()),
        ws,
        user,
        other,
        pool,
    }
}

async fn notify(w: &World, user: Uuid, priority: i16) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO notifications (id, workspace_id, recipient_member_id, priority, kind, text_key)
         SELECT $1, $2, id, $4, 'reminder', 'notification.reminder'
         FROM workspace_members WHERE workspace_id = $2 AND user_id = $3",
    )
    .bind(id)
    .bind(w.ws)
    .bind(user)
    .bind(priority)
    .execute(&w.pool)
    .await
    .unwrap();
    id
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn inbox_shows_own_notifications_and_actions_change_them(pool: PgPool) {
    let w = world(pool).await;
    let mine = notify(&w, w.user, 2).await;
    let theirs = notify(&w, w.other, 2).await;
    let base = format!("/v1/workspaces/{}/notifications", w.ws);

    let r = send(&w.app, call("GET", &base, w.user, None)).await;
    assert_eq!(r.status, StatusCode::OK);
    let items = r.body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], mine.to_string());

    // Another member's notification does not exist for the caller.
    let r = send(
        &w.app,
        call(
            "POST",
            &format!("{base}/{theirs}/actions"),
            w.user,
            Some(&json!({"action": "approve"})),
        ),
    )
    .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);

    let act = |body: Value| call("POST", &format!("{base}/{mine}/actions"), w.user, Some(&body));
    let r = send(&w.app, act(json!({"action": "postpone"}))).await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body["status"], "snoozed");
    assert!(r.body["snoozed_until"].is_string());
    let r = send(
        &w.app,
        act(json!({"action": "postpone", "until": "2020-01-01T00:00:00Z"})),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    let r = send(&w.app, act(json!({"action": "approve"}))).await;
    assert_eq!(r.body["status"], "acknowledged");
    // A closed notification takes no more actions.
    let r = send(&w.app, act(json!({"action": "ignore"}))).await;
    assert_eq!(r.status, StatusCode::CONFLICT);
    let r = send(&w.app, act(json!({"action": "dance"}))).await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);

    let events: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM notification_events WHERE notification_id = $1",
    )
    .bind(mine)
    .fetch_one(&w.pool)
    .await
    .unwrap();
    assert_eq!(events, 2);

    let r = send(&w.app, call("GET", &format!("{base}?status=open"), w.user, None)).await;
    assert_eq!(r.body["items"].as_array().unwrap().len(), 0);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn preferences_default_then_save(pool: PgPool) {
    let w = world(pool).await;
    let path = format!("/v1/workspaces/{}/me/notification-preferences", w.ws);
    let r = send(&w.app, call("GET", &path, w.user, None)).await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body["timezone"], "America/New_York");
    assert_eq!(r.body["daily_summary"], false);

    let r = send(
        &w.app,
        call(
            "PUT",
            &path,
            w.user,
            Some(&json!({"quiet_start": "22:00", "quiet_end": "07:00", "daily_summary": true})),
        ),
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert_eq!(r.body["quiet_start"], "22:00");
    let r = send(&w.app, call("GET", &path, w.user, None)).await;
    assert_eq!(r.body["daily_summary"], true);
    assert_eq!(r.body["quiet_end"], "07:00");

    for bad in [
        json!({"timezone": "Mars/Base"}),
        json!({"quiet_start": "22:00"}),
        json!({"summary_time": "25:00"}),
    ] {
        let r = send(&w.app, call("PUT", &path, w.user, Some(&bad))).await;
        assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{bad}");
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn alert_rules_are_validated_and_need_the_permission(pool: PgPool) {
    let w = world(pool).await;
    let path = format!("/v1/workspaces/{}/alert-rules", w.ws);
    let good = json!({"name": "Too warm", "priority": 1,
        "condition": {"schema_version": 1, "measurement": "temp_c", "op": "gt", "value": 22.0}});
    let post = |user: Uuid, body: &Value| {
        let mut r = call("POST", &path, user, Some(body));
        r.headers_mut()
            .insert("idempotency-key", Uuid::now_v7().to_string().parse().unwrap());
        r
    };
    let r = send(&w.app, post(w.other, &good)).await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    let r = send(
        &w.app,
        post(
            w.user,
            &json!({"name": "x", "condition": {"schema_version": 1, "measurement": "nope",
                "op": "gt", "value": 1}}),
        ),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    let r = send(&w.app, post(w.user, &good)).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let id = r.body["id"].as_str().unwrap().to_owned();

    let r = send(&w.app, call("GET", &path, w.other, None)).await;
    assert_eq!(r.body.as_array().unwrap().len(), 1);
    let r = send(
        &w.app,
        call("PATCH", &format!("{path}/{id}"), w.user, Some(&json!({"enabled": false}))),
    )
    .await;
    assert_eq!(r.body["enabled"], false);
    let r = send(&w.app, call("DELETE", &format!("{path}/{id}"), w.user, None)).await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    let r = send(&w.app, call("GET", &path, w.user, None)).await;
    assert_eq!(r.body.as_array().unwrap().len(), 0);
}
