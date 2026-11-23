#![allow(clippy::unwrap_used, clippy::expect_used)]

use e2e::Context;
use reqwest::{Method, StatusCode};

#[tokio::test]
async fn stale_recipe_version_update_returns_412() {
    let Some(ctx) = Context::from_env() else {
        eprintln!("skipping E2E: E2E_BASE_URL is not set");
        return;
    };
    let user = ctx.login().await.unwrap();
    let workspace = ctx
        .create_workspace(&user, "home", "E2E Stale Version")
        .await
        .unwrap();
    let ws = workspace["id"].as_str().unwrap();
    let recipe = ctx
        .create_recipe(&user, ws, "E2E Versioned Recipe")
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let recipe: serde_json::Value = recipe.json().await.unwrap();
    let version_id = recipe["draft_version_id"].as_str().unwrap();

    let current = ctx
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
    let etag = current
        .headers()
        .get("etag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    let first = ctx
        .request(
            Method::PATCH,
            &format!("/v1/workspaces/{ws}/recipe-versions/{version_id}"),
            Some(&user),
        )
        .header("Content-Type", "application/merge-patch+json")
        .header("If-Match", &etag)
        .body(r#"{"change_note":"first update"}"#)
        .send()
        .await
        .unwrap();
    let first_status = first.status();
    let first_body = first.text().await.unwrap();
    assert_eq!(first_status, StatusCode::OK, "{first_body}");

    let stale = ctx
        .request(
            Method::PATCH,
            &format!("/v1/workspaces/{ws}/recipe-versions/{version_id}"),
            Some(&user),
        )
        .header("Content-Type", "application/merge-patch+json")
        .header("If-Match", etag)
        .body(r#"{"change_note":"stale update"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(stale.status(), StatusCode::PRECONDITION_FAILED);
    let stale_body: serde_json::Value = stale.json().await.unwrap();
    assert!(stale_body["current"].is_object());
    assert!(stale_body["conflicting_fields"].is_array());

    let unchanged = ctx
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
    let unchanged: serde_json::Value = unchanged.json().await.unwrap();
    assert_eq!(unchanged["change_note"], "first update");
}
