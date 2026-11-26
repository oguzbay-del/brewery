#![allow(clippy::unwrap_used, clippy::expect_used)]

use e2e::Context;
use reqwest::{
    Method, StatusCode,
    header::{CONTENT_TYPE, ETAG, IF_MATCH},
};
use serde_json::{Value, json};
use uuid::Uuid;

#[tokio::test]
async fn recipe_create_replays_once_and_rejects_a_changed_body() {
    let Some(ctx) = Context::from_env() else {
        eprintln!("skipping E2E: E2E_BASE_URL is not set");
        return;
    };
    let user = ctx.login().await.unwrap();
    let workspace = ctx
        .create_workspace(&user, "home", "E2E Idempotency")
        .await
        .unwrap();
    let ws = workspace["id"].as_str().unwrap();
    let path = format!("/v1/workspaces/{ws}/recipes");
    let key = Uuid::new_v4().to_string();
    let create = |body: &Value| {
        ctx.request(Method::POST, &path, Some(&user))
            .header("Idempotency-Key", key.as_str())
            .json(body)
    };
    let mut body = json!({
        "name": "E2E Replay Recipe",
        "version": {
            "batch_volume_l": 20.0,
            "efficiency_pct": 72.0
        }
    });

    let first = create(&body).send().await.unwrap();
    let first_status = first.status();
    assert_eq!(first_status, StatusCode::CREATED);
    let first: Value = first.json().await.unwrap();
    let recipe_id = first["id"].as_str().unwrap();

    let replay = create(&body).send().await.unwrap();
    assert_eq!(replay.status(), first_status);
    assert_eq!(replay.headers().get("Idempotent-Replayed").unwrap(), "true");
    let replay: Value = replay.json().await.unwrap();
    assert_eq!(replay["id"], recipe_id);

    body["name"] = json!("E2E Different Recipe");
    let conflict = create(&body).send().await.unwrap();
    assert_eq!(conflict.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let problem: Value = conflict.json().await.unwrap();
    assert_eq!(problem["code"], "idempotency_mismatch");

    let list: Value = ctx
        .request(Method::GET, &path, Some(&user))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    let items = list["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], recipe_id);
}

#[tokio::test]
async fn draft_patch_advances_the_etag_and_rejects_a_stale_etag() {
    let Some(ctx) = Context::from_env() else {
        eprintln!("skipping E2E: E2E_BASE_URL is not set");
        return;
    };
    let user = ctx.login().await.unwrap();
    let workspace = ctx
        .create_workspace(&user, "home", "E2E ETag")
        .await
        .unwrap();
    let ws = workspace["id"].as_str().unwrap();
    let recipe: Value = ctx
        .create_recipe(&user, ws, "E2E ETag Recipe")
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    let version_id = recipe["draft_version_id"].as_str().unwrap();
    let version_path = format!("/v1/workspaces/{ws}/recipe-versions/{version_id}");
    let get = || ctx.request(Method::GET, &version_path, Some(&user));
    let patch = |etag: &reqwest::header::HeaderValue, note| {
        ctx.request(Method::PATCH, &version_path, Some(&user))
            .header(CONTENT_TYPE, "application/merge-patch+json")
            .header(IF_MATCH, etag)
            .json(&json!({ "change_note": note }))
    };

    let initial = get().send().await.unwrap();
    assert_eq!(initial.status(), StatusCode::OK);
    let original_etag = initial.headers().get(ETAG).unwrap();

    let updated = patch(original_etag, "Accepted update")
        .send()
        .await
        .unwrap();
    assert_eq!(updated.status(), StatusCode::OK);
    let updated_etag = updated.headers().get(ETAG).unwrap().clone();
    assert_ne!(&updated_etag, original_etag);
    let updated: Value = updated.json().await.unwrap();
    assert_eq!(updated["change_note"], "Accepted update");

    let stale = patch(original_etag, "Stale update").send().await.unwrap();
    assert_eq!(stale.status(), StatusCode::PRECONDITION_FAILED);
    let problem: Value = stale.json().await.unwrap();
    assert_eq!(problem["code"], "version_mismatch");

    let unchanged = get().send().await.unwrap();
    assert_eq!(unchanged.status(), StatusCode::OK);
    assert_eq!(unchanged.headers().get(ETAG), Some(&updated_etag));
    let unchanged: Value = unchanged.json().await.unwrap();
    assert_eq!(unchanged["change_note"], "Accepted update");
}
