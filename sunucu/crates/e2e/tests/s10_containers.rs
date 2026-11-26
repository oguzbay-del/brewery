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
    let problem: Value = response.json().await.unwrap();
    assert_eq!(problem["code"], "module_disabled");
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
    assert_eq!(problem["code"], "not_found");

    let other = ctx
        .create_workspace(&owner, "business", "E2E Other Scan")
        .await
        .unwrap();
    let other_ws = other["id"].as_str().unwrap();
    let formatted_code = format!("{other_ws}/AAAAAAAAAAAAAAAAAAAAAAAAAA");
    let response = ctx
        .post_json(
            &format!("/v1/workspaces/{ws}/scan/resolve"),
            Some(&owner),
            &json!({"code": formatted_code}),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let other_problem: Value = response.json().await.unwrap();
    assert_eq!(other_problem["code"], problem["code"]);
}

#[tokio::test]
async fn unknown_codes_fail_individually_in_batch() {
    let Some(ctx) = Context::from_env() else {
        return;
    };
    let owner = ctx.login().await.unwrap();
    let workspace = ctx
        .create_workspace(&owner, "business", "E2E Batch Scan")
        .await
        .unwrap();
    let ws = workspace["id"].as_str().unwrap();
    let codes: Vec<String> = (0..3).map(|_| Uuid::new_v4().to_string()).collect();
    let response = ctx
        .post_json(
            &format!("/v1/workspaces/{ws}/scan/status-batch"),
            Some(&owner),
            &json!({"codes": codes, "status": "dirty"}),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["changed"], 0);
    assert_eq!(body["failed"], 3);
    let results = body["results"].as_array().unwrap();
    assert_eq!(results.len(), 3);
    for (index, result) in results.iter().enumerate() {
        assert_eq!(result["index"], index);
        assert_eq!(result["ok"], false);
        assert_eq!(result["error"], "not_found");
        assert!(result["container"].is_null());
    }
}

#[tokio::test]
async fn labels_reject_invalid_targets() {
    let Some(ctx) = Context::from_env() else {
        return;
    };
    let owner = ctx.login().await.unwrap();
    let workspace = ctx
        .create_workspace(&owner, "business", "E2E Labels")
        .await
        .unwrap();
    let ws = workspace["id"].as_str().unwrap();
    let path = format!("/v1/workspaces/{ws}/labels?paper=a4");
    for body in [
        json!({"targets": []}),
        json!({"targets": [{"kind": "container", "id": Uuid::new_v4()}]}),
    ] {
        let response = ctx.post_json(&path, Some(&owner), &body).await.unwrap();
        // The handler maps a missing target to a field validation error.
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let problem: Value = response.json().await.unwrap();
        assert_eq!(problem["code"], "validation_failed");
    }
}

#[tokio::test]
async fn outsider_cannot_list_workspace_containers() {
    let Some(ctx) = Context::from_env() else {
        return;
    };
    let owner = ctx.login().await.unwrap();
    let outsider = ctx.login().await.unwrap();
    let workspace = ctx
        .create_workspace(&owner, "business", "E2E Isolation")
        .await
        .unwrap();
    let ws = workspace["id"].as_str().unwrap();
    let response = ctx
        .request(
            Method::GET,
            &format!("/v1/workspaces/{ws}/containers"),
            Some(&outsider),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
