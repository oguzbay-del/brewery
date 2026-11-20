//! Library endpoints end to end (T2.19): ingredients, style guides, styles and mappings, on a
//! real PostgreSQL (`#[sqlx::test]`). The shared rows come from the sample seed.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::sync::Arc;

use api::{ApiConfig, redis_client::Redis};
use axum::{Router, body::Body, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{FakeAuth, Reply, insert_user, insert_workspace, json_post, req, send, state_with};
use uuid::Uuid;

struct World {
    app: Router,
    ws: Uuid,
    other_ws: Uuid,
    /// `recipe.edit`.
    editor: Uuid,
    /// Member without permissions.
    reader: Uuid,
    /// `recipe.edit` in the other workspace.
    other_editor: Uuid,
}

async fn world(pool: &PgPool) -> World {
    let (ws, other_ws) = (Uuid::now_v7(), Uuid::now_v7());
    let (editor, reader, other_editor) = (Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7());
    insert_workspace(pool, ws).await;
    insert_workspace(pool, other_ws).await;
    for u in [editor, reader, other_editor] {
        insert_user(pool, u).await;
    }
    db::seed::seed_sample_library(pool).await.unwrap();
    let auth = Arc::new(FakeAuth::default());
    auth.add(editor, ws, &["recipe.edit"]);
    auth.add(reader, ws, &[]);
    auth.add(other_editor, other_ws, &["recipe.edit"]);
    let cfg = ApiConfig::from_lookup(|n| match n {
        "DATABASE_URL" => Some("postgres://unused".to_owned()),
        "TRUST_FORWARDED_FOR" => Some("true".to_owned()),
        _ => None,
    })
    .unwrap();
    let state = state_with(pool, Redis::disabled(), auth, cfg).await;
    World {
        app: api::router(state),
        ws,
        other_ws,
        editor,
        reader,
        other_editor,
    }
}

fn path(ws: Uuid, tail: &str) -> String {
    format!("/v1/workspaces/{ws}/{tail}")
}

async fn get(w: &World, user: Uuid, tail: &str) -> Reply {
    send(
        &w.app,
        req("GET", &path(w.ws, tail), Some(user))
            .body(Body::empty())
            .unwrap(),
    )
    .await
}

async fn create(w: &World, ws: Uuid, user: Uuid, body: &Value) -> Reply {
    send(
        &w.app,
        json_post(
            &path(ws, "ingredients"),
            user,
            Some(&Uuid::now_v7().to_string()),
            body,
        ),
    )
    .await
}

async fn patch(w: &World, user: Uuid, id: &str, version: Option<&str>, body: &Value) -> Reply {
    let mut b = req(
        "PATCH",
        &path(w.ws, &format!("ingredients/{id}")),
        Some(user),
    )
    .header("content-type", "application/merge-patch+json");
    if let Some(v) = version {
        b = b.header("if-match", v);
    }
    send(&w.app, b.body(Body::from(body.to_string())).unwrap()).await
}

async fn delete(w: &World, user: Uuid, id: &str, version: Option<&str>) -> Reply {
    let mut b = req(
        "DELETE",
        &path(w.ws, &format!("ingredients/{id}")),
        Some(user),
    );
    if let Some(v) = version {
        b = b.header("if-match", v);
    }
    send(&w.app, b.body(Body::empty()).unwrap()).await
}

fn hop() -> Value {
    json!({"kind": "hop", "name": "Own Hop", "origin": "TR",
           "price": "12.5", "price_currency": "USD", "detail": {"alpha_pct": 7.5}})
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn shared_library_is_listed_filtered_and_paged(pool: PgPool) {
    let w = world(&pool).await;
    let all = get(&w, w.reader, "ingredients?limit=200").await;
    assert_eq!(all.status, StatusCode::OK);
    let items = all.body["items"].as_array().unwrap();
    assert_eq!(items.len(), 11);
    assert!(items.iter().all(|i| i["source"] == "library"));
    let hops = get(&w, w.reader, "ingredients?kind=hop&q=cit").await;
    assert_eq!(hops.body["items"].as_array().unwrap().len(), 1);
    assert_eq!(hops.body["items"][0]["name"], "Sample Citra");
    assert_eq!(hops.body["items"][0]["detail"]["alpha_pct"], 12.0);
    // Paging: two pages of 6 and 5, no overlap.
    let p1 = get(&w, w.reader, "ingredients?limit=6").await;
    let cursor = p1.body["next_cursor"].as_str().unwrap().to_owned();
    let p2 = get(
        &w,
        w.reader,
        &format!("ingredients?limit=6&cursor={cursor}"),
    )
    .await;
    assert_eq!(p1.body["items"].as_array().unwrap().len(), 6);
    assert_eq!(p2.body["items"].as_array().unwrap().len(), 5);
    assert!(p2.body["next_cursor"].is_null());
    let bad = get(&w, w.reader, "ingredients?kind=nope").await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY);
    let id = items[0]["id"].as_str().unwrap();
    let one = get(&w, w.reader, &format!("ingredients/{id}")).await;
    assert_eq!(one.status, StatusCode::OK);
    assert!(!one.header("etag").is_empty());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn workspace_ingredient_lifecycle(pool: PgPool) {
    let w = world(&pool).await;
    // Needs recipe.edit.
    assert_eq!(
        create(&w, w.ws, w.reader, &hop()).await.status,
        StatusCode::FORBIDDEN
    );
    let made = create(&w, w.ws, w.editor, &hop()).await;
    assert_eq!(made.status, StatusCode::CREATED, "{}", made.body);
    assert_eq!(made.body["source"], "user");
    assert_eq!(made.body["price"], "12.5000");
    assert_eq!(made.body["detail"]["form"], "pellet");
    let id = made.body["id"].as_str().unwrap().to_owned();
    assert!(made.header("location").ends_with(&id));
    // The workspace sees both its own and the shared rows; the filter separates them.
    let own = get(&w, w.editor, "ingredients?source=user").await;
    assert_eq!(own.body["items"].as_array().unwrap().len(), 1);
    // If-Match is required; a stale version gives 412.
    let patch_body = json!({"name": "Renamed", "detail": {"alpha_pct": 8.0}, "origin": null});
    assert_eq!(
        patch(&w, w.editor, &id, None, &patch_body).await.status,
        StatusCode::PRECONDITION_REQUIRED
    );
    assert_eq!(
        patch(&w, w.editor, &id, Some("\"9\""), &patch_body)
            .await
            .status,
        StatusCode::PRECONDITION_FAILED
    );
    let updated = patch(&w, w.editor, &id, Some("\"1\""), &patch_body).await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.body);
    assert_eq!(updated.body["name"], "Renamed");
    assert_eq!(updated.body["detail"]["alpha_pct"], 8.0);
    assert!(updated.body["origin"].is_null());
    assert_eq!(updated.body["version"], 2);
    // Validation reports a field.
    let invalid = patch(
        &w,
        w.editor,
        &id,
        Some("\"2\""),
        &json!({"detail": {"alpha_pct": 120}}),
    )
    .await;
    assert_eq!(invalid.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(invalid.body["errors"][0]["field"], "/detail/alpha_pct");
    // kind cannot change.
    let kind = patch(&w, w.editor, &id, Some("\"2\""), &json!({"kind": "yeast"})).await;
    assert_eq!(kind.status, StatusCode::UNPROCESSABLE_ENTITY);
    // Delete.
    assert_eq!(
        delete(&w, w.reader, &id, Some("\"2\"")).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        delete(&w, w.editor, &id, Some("\"2\"")).await.status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        get(&w, w.editor, &format!("ingredients/{id}")).await.status,
        StatusCode::NOT_FOUND
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn create_validates_each_kind(pool: PgPool) {
    let w = world(&pool).await;
    for (body, field) in [
        (json!({"kind": "gas", "name": "x"}), "/kind"),
        (json!({"kind": "hop", "name": " "}), "/name"),
        (
            json!({"kind": "hop", "name": "x", "price": "1", "detail": {"alpha_pct": 5}}),
            "/price_currency",
        ),
        (
            json!({"kind": "yeast", "name": "x",
                   "detail": {"form": "dry", "min_attenuation_pct": 90, "max_attenuation_pct": 70}}),
            "/detail/min_attenuation_pct",
        ),
        (
            json!({"kind": "fermentable", "name": "x",
                   "detail": {"grain_type": "rock", "yield_pct": 80}}),
            "/detail/grain_type",
        ),
        (
            json!({"kind": "misc", "name": "x", "detail": {"unknown": 1}}),
            "/detail",
        ),
    ] {
        let r = create(&w, w.ws, w.editor, &body).await;
        assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert_eq!(r.body["errors"][0]["field"], field, "{body}");
    }
    for body in [
        json!({"kind": "fermentable", "name": "Own Malt",
               "detail": {"grain_type": "grain", "yield_pct": 78.0, "color_srm": 3.0}}),
        json!({"kind": "yeast", "name": "Own Yeast",
               "detail": {"form": "dry", "min_attenuation_pct": 70, "max_attenuation_pct": 80}}),
        json!({"kind": "misc", "name": "Own Misc"}),
    ] {
        let r = create(&w, w.ws, w.editor, &body).await;
        assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn shared_rows_are_read_only_and_other_workspace_is_hidden(pool: PgPool) {
    let w = world(&pool).await;
    let shared = get(&w, w.editor, "ingredients?q=cascade").await.body["items"][0].clone();
    let id = shared["id"].as_str().unwrap();
    let r = patch(&w, w.editor, id, Some("\"1\""), &json!({"name": "Mine"})).await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    assert_eq!(r.body["errors"][0]["code"], "read_only");
    assert_eq!(
        delete(&w, w.editor, id, Some("\"1\"")).await.status,
        StatusCode::FORBIDDEN
    );
    // A record of another workspace answers 404 (also inside the caller's own workspace path).
    let theirs = create(&w, w.other_ws, w.other_editor, &hop()).await;
    assert_eq!(theirs.status, StatusCode::CREATED, "{}", theirs.body);
    let tid = theirs.body["id"].as_str().unwrap();
    assert_eq!(
        get(&w, w.editor, &format!("ingredients/{tid}"))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        patch(&w, w.editor, tid, Some("\"1\""), &json!({"name": "x"}))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        delete(&w, w.editor, tid, Some("\"1\"")).await.status,
        StatusCode::NOT_FOUND
    );
    let list = get(&w, w.editor, "ingredients?source=user").await;
    assert!(list.body["items"].as_array().unwrap().is_empty());
    // Not a member of the workspace: hidden as well.
    let outsider = send(
        &w.app,
        req("GET", &path(w.other_ws, "ingredients"), Some(w.editor))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(outsider.status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn style_guides_styles_and_mappings(pool: PgPool) {
    let w = world(&pool).await;
    let guides = get(&w, w.reader, "style-guides").await;
    let items = guides.body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["name"], "Sample Guide A");
    assert!(
        items[0]["license_note"]
            .as_str()
            .unwrap()
            .contains("SAMPLE")
    );
    let (a, b) = (
        items[0]["id"].as_str().unwrap(),
        items[1]["id"].as_str().unwrap(),
    );
    assert_eq!(
        get(&w, w.reader, &format!("style-guides/{a}")).await.status,
        StatusCode::OK
    );
    assert_eq!(
        get(&w, w.reader, &format!("style-guides/{}", Uuid::now_v7()))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let styles = get(&w, w.reader, &format!("style-guides/{a}/styles")).await;
    assert_eq!(styles.body["items"].as_array().unwrap().len(), 2);
    assert_eq!(styles.body["items"][0]["name"], "Sample Pale Ale");
    assert_eq!(
        get(
            &w,
            w.reader,
            &format!("style-guides/{}/styles", Uuid::now_v7())
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    let all = get(&w, w.reader, "styles").await;
    assert_eq!(all.body["items"].as_array().unwrap().len(), 4);
    let found = get(&w, w.reader, "styles?q=stout").await;
    assert_eq!(found.body["items"].as_array().unwrap().len(), 2);
    let sid = styles.body["items"][0]["id"].as_str().unwrap();
    let one = get(&w, w.reader, &format!("styles/{sid}")).await;
    assert_eq!(one.body["guide_id"], a);
    assert_eq!(one.body["og_min_sg"], 1.045);
    // Mappings from guide A to guide B.
    let maps = get(
        &w,
        w.reader,
        &format!("style-mappings?from_guide_id={a}&to_guide_id={b}"),
    )
    .await;
    let rows = maps.body["items"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert!(
        rows.iter()
            .all(|m| m["to_guide_id"] == b && m["from_guide_id"] == a)
    );
    assert!(rows.iter().any(|m| m["relation"] == "equivalent"));
    let by_style = get(&w, w.reader, &format!("style-mappings?style_id={sid}")).await;
    assert_eq!(by_style.body["items"].as_array().unwrap().len(), 2);
    let page = get(&w, w.reader, "style-mappings?limit=2").await;
    assert!(page.body["next_cursor"].is_string());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn seed_is_safe_to_repeat(pool: PgPool) {
    let _ = world(&pool).await;
    db::seed::seed_sample_library(&pool).await.unwrap();
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM ingredients WHERE workspace_id IS NULL")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 11);
}
