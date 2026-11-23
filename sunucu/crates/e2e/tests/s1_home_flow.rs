#![allow(clippy::unwrap_used, clippy::expect_used)]

use e2e::Context;
use reqwest::Method;
use serde_json::{Value, json};
use uuid::Uuid;

#[tokio::test]
async fn home_user_completes_the_main_flow() {
    let Some(ctx) = Context::from_env() else {
        eprintln!("skipping E2E: E2E_BASE_URL is not set");
        return;
    };
    let user = ctx.login().await.unwrap();
    let workspace = ctx.create_workspace(&user, "home", "E2E Home Flow").await.unwrap();
    let ws = workspace["id"].as_str().unwrap();

    let recipe = ctx.create_recipe(&user, ws, "E2E House IPA").await.unwrap();
    assert_eq!(recipe.status(), reqwest::StatusCode::CREATED, "{}", recipe.text().await.unwrap());
    let recipe: Value = recipe.json().await.unwrap();
    let version_id = recipe["draft_version_id"].as_str().unwrap();

    let version = ctx.request(
        Method::GET,
        &format!("/v1/workspaces/{ws}/recipe-versions/{version_id}"),
        Some(&user),
    ).send().await.unwrap().error_for_status().unwrap();
    let etag = version.headers().get("etag").unwrap().to_str().unwrap().to_owned();
    let publish = ctx.request(
        Method::POST,
        &format!("/v1/workspaces/{ws}/recipe-versions/{version_id}/publish"),
        Some(&user),
    )
    .header("If-Match", etag)
    .header("Idempotency-Key", Uuid::new_v4().to_string())
    .json(&json!({}))
    .send().await.unwrap();
    assert_eq!(publish.status(), reqwest::StatusCode::OK, "{}", publish.text().await.unwrap());

    let batch = ctx.post_json(
        &format!("/v1/workspaces/{ws}/batches"),
        Some(&user),
        &json!({"recipe_version_id": version_id, "target_volume_l": 20.0}),
    ).await.unwrap().error_for_status().unwrap();
    let batch: Value = batch.json().await.unwrap();
    assert_eq!(batch["stage"], "planned");
    let batch_id = batch["id"].as_str().unwrap();

    let package = ctx.post_json(
        &format!("/v1/workspaces/{ws}/batches/{batch_id}/brew-day-package"),
        Some(&user),
        &json!({"valid_hours": 24}),
    ).await.unwrap().error_for_status().unwrap();
    let package: Value = package.json().await.unwrap();
    let package_id = package["id"].as_str().unwrap();

    let measurement = ctx.post_json(
        &format!("/v1/workspaces/{ws}/batches/{batch_id}/measurements"),
        Some(&user),
        &json!({"items": [{
            "id": Uuid::new_v4(),
            "kind": "sg",
            "value": 1.052,
            "measured_at": "2026-11-23T12:00:00Z",
            "package_id": package_id,
            "source": "manual"
        }]}),
    ).await.unwrap().error_for_status().unwrap();
    let measurement: Value = measurement.json().await.unwrap();
    for item in measurement["items"].as_array().unwrap() {
        assert!(matches!(item["status"].as_str(), Some("created" | "duplicate")));
    }

    let notifications = ctx.request(
        Method::GET,
        &format!("/v1/workspaces/{ws}/notifications"),
        Some(&user),
    ).send().await.unwrap();
    assert_eq!(notifications.status(), reqwest::StatusCode::OK);

    let support = ctx.post_json(
        &format!("/v1/workspaces/{ws}/support/requests"),
        Some(&user),
        &json!({
            "topic": "recipes",
            "title": "E2E support request",
            "body": "The end-to-end test is checking the customer support flow."
        }),
    ).await.unwrap();
    assert_eq!(support.status(), reqwest::StatusCode::CREATED, "{}", support.text().await.unwrap());
    let support: Value = support.json().await.unwrap();
    assert!(support["request"]["id"].as_str().is_some());
}
