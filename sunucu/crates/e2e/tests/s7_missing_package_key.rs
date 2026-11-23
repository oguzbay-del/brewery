#![allow(clippy::unwrap_used, clippy::expect_used)]

use e2e::Context;
use reqwest::{Method, StatusCode};
use serde_json::{Value, json};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires a second isolated Compose stack with the package signing key unset; the standard Compose workflow provides one shared stack"]
async fn package_operations_fail_without_the_package_key_but_manual_measurements_work() {
    let Some(ctx) = Context::from_env() else {
        eprintln!("skipping E2E: E2E_BASE_URL is not set");
        return;
    };
    let user = ctx.login().await.unwrap();
    let workspace = ctx
        .create_workspace(&user, "home", "E2E Missing Package Key")
        .await
        .unwrap();
    let ws = workspace["id"].as_str().unwrap();

    let recipe_response = ctx
        .create_recipe(&user, ws, "E2E Package Key Recipe")
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let recipe: Value = recipe_response.json().await.unwrap();
    let version_id = recipe["draft_version_id"].as_str().unwrap();
    let version = ctx
        .request(
            Method::GET,
            &format!("/v1/workspaces/{ws}/recipe-versions/{version_id}"),
            Some(&user),
        )
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let etag = version
        .headers()
        .get("etag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let publish = ctx
        .request(
            Method::POST,
            &format!("/v1/workspaces/{ws}/recipe-versions/{version_id}/publish"),
            Some(&user),
        )
        .header("If-Match", etag)
        .header("Idempotency-Key", Uuid::new_v4().to_string())
        .json(&json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(publish.status(), StatusCode::OK);

    let batch = ctx
        .post_json(
            &format!("/v1/workspaces/{ws}/batches"),
            Some(&user),
            &json!({"recipe_version_id":version_id,"target_volume_l":20.0}),
        )
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let batch: Value = batch.json().await.unwrap();
    let batch_id = batch["id"].as_str().unwrap();

    let package = ctx
        .post_json(
            &format!("/v1/workspaces/{ws}/batches/{batch_id}/brew-day-package"),
            Some(&user),
            &json!({}),
        )
        .await
        .unwrap();
    assert_eq!(package.status(), StatusCode::SERVICE_UNAVAILABLE);

    let with_package = ctx
        .post_json(
            &format!("/v1/workspaces/{ws}/batches/{batch_id}/measurements"),
            Some(&user),
            &json!({"items":[{
                "id":Uuid::new_v4(),"kind":"sg","value":1.05,
                "measured_at":"2026-11-23T12:00:00Z","package_id":Uuid::new_v4()
            }]}),
        )
        .await
        .unwrap();
    assert_eq!(with_package.status(), StatusCode::SERVICE_UNAVAILABLE);

    let without_package = ctx
        .post_json(
            &format!("/v1/workspaces/{ws}/batches/{batch_id}/measurements"),
            Some(&user),
            &json!({"items":[{
                "id":Uuid::new_v4(),"kind":"sg","value":1.05,
                "measured_at":"2026-11-23T12:00:00Z"
            }]}),
        )
        .await
        .unwrap();
    assert_eq!(without_package.status(), StatusCode::OK);
}
