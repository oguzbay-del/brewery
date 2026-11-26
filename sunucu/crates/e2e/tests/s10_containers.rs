#![allow(clippy::unwrap_used, clippy::expect_used)]

use e2e::Context;
use reqwest::{Method, StatusCode};
use serde_json::{Value, json};
use uuid::Uuid;

#[tokio::test]
async fn home_workspace_cannot_list_containers() {
    let Some(ctx) = Context::from_env() else {
        eprintln!("skipping E2E: E2E_BASE_URL is not set");
        return;
    };
    let owner = ctx.login().await.unwrap();
    let workspace = ctx
        .create_workspace(&owner, "home", "E2E Home Containers")
        .await
        .unwrap();
    let ws = workspace["id"].as_str().unwrap();
    let response = ctx
        .request(
            Method::GET,
            &format!("/v1/workspaces/{ws}/containers"),
            Some(&owner),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn unknown_scan_code_is_not_found() {
    let Some(ctx) = Context::from_env() else {
        eprintln!("skipping E2E: E2E_BASE_URL is not set");
        return;
    };
    let owner = ctx.login().await.unwrap();
    let workspace = ctx
        .create_workspace(&owner, "business", "E2E Unknown Scan")
        .await
        .unwrap();
    let ws = workspace["id"].as_str().unwrap();
    let response = ctx
        .post_json(
            &format!("/v1/workspaces/{ws}/scan/resolve"),
            Some(&owner),
            &json!({"code": Uuid::new_v4().simple().to_string()}),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let problem: Value = response.json().await.unwrap();
    assert!(problem.is_object());
}
