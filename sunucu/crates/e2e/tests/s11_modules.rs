#![allow(clippy::unwrap_used, clippy::expect_used)]

use e2e::Context;
use reqwest::{Method, StatusCode};
use serde_json::Value;

async fn modules(ctx: &Context, user: &e2e::User, ws: &str) -> Value {
    let response = ctx
        .request(
            Method::GET,
            &format!("/v1/meta?workspace_id={ws}"),
            Some(user),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    response.json().await.unwrap()
}

fn module<'a>(response: &'a Value, id: &str) -> &'a Value {
    response["modules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["id"] == id)
        .unwrap()
}

#[tokio::test]
async fn home_modules_are_subset_of_business_modules() {
    let Some(ctx) = Context::from_env() else {
        eprintln!("skipping E2E: E2E_BASE_URL is not set");
        return;
    };
    let owner = ctx.login().await.unwrap();
    let home = ctx
        .create_workspace(&owner, "home", "E2E Home Modules")
        .await
        .unwrap();
    let business = ctx
        .create_workspace(&owner, "business", "E2E Business Modules")
        .await
        .unwrap();
    let home_modules = modules(&ctx, &owner, home["id"].as_str().unwrap()).await;
    let business_modules = modules(&ctx, &owner, business["id"].as_str().unwrap()).await;

    assert_eq!(module(&home_modules, "core")["enabled"], true);
    assert_eq!(module(&home_modules, "core")["read_only"], false);
    assert_eq!(module(&home_modules, "team")["enabled"], false);
    assert_eq!(module(&home_modules, "team")["read_only"], false);

    let home_list = home_modules["modules"].as_array().unwrap();
    let business_list = business_modules["modules"].as_array().unwrap();
    assert_eq!(home_list.len(), business_list.len());
    for home_item in home_list {
        let id = home_item["id"].as_str().unwrap();
        let business_item = module(&business_modules, id);
        if home_item["enabled"] == true {
            assert_eq!(business_item["enabled"], true, "{id}");
        }
    }
    assert!(
        business_list.iter().any(|item| {
            item["enabled"] == true
                && module(&home_modules, item["id"].as_str().unwrap())["enabled"] == false
        }),
        "business must have a module that is disabled for home"
    );
}

#[tokio::test]
async fn business_trial_enables_production_quality_and_team() {
    let Some(ctx) = Context::from_env() else {
        eprintln!("skipping E2E: E2E_BASE_URL is not set");
        return;
    };
    let owner = ctx.login().await.unwrap();
    let ws = ctx
        .create_workspace(&owner, "business", "E2E Business Trial")
        .await
        .unwrap();
    let result = modules(&ctx, &owner, ws["id"].as_str().unwrap()).await;
    for id in ["production", "quality", "team"] {
        assert_eq!(module(&result, id)["enabled"], true, "{id}");
        assert_eq!(module(&result, id)["read_only"], false, "{id}");
    }
}

#[tokio::test]
async fn home_container_endpoint_is_disabled() {
    let Some(ctx) = Context::from_env() else {
        eprintln!("skipping E2E: E2E_BASE_URL is not set");
        return;
    };
    let owner = ctx.login().await.unwrap();
    let ws = ctx
        .create_workspace(&owner, "home", "E2E Module Gate")
        .await
        .unwrap();
    let response = ctx
        .request(
            Method::GET,
            &format!("/v1/workspaces/{}/containers", ws["id"].as_str().unwrap()),
            Some(&owner),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["code"], "module_disabled");
    assert!(body["request_id"].as_str().is_some_and(|id| !id.is_empty()));
}

#[tokio::test]
async fn meta_requires_membership_and_authentication() {
    let Some(ctx) = Context::from_env() else {
        eprintln!("skipping E2E: E2E_BASE_URL is not set");
        return;
    };
    let owner = ctx.login().await.unwrap();
    let stranger = ctx.login().await.unwrap();
    let ws = ctx
        .create_workspace(&owner, "home", "E2E Meta Isolation")
        .await
        .unwrap();
    let path = format!("/v1/meta?workspace_id={}", ws["id"].as_str().unwrap());

    let foreign = ctx
        .request(Method::GET, &path, Some(&stranger))
        .send()
        .await
        .unwrap();
    assert_eq!(foreign.status(), StatusCode::NOT_FOUND);

    let missing = ctx
        .request(Method::GET, "/v1/meta", Some(&owner))
        .send()
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::BAD_REQUEST);

    let anonymous = ctx.request(Method::GET, &path, None).send().await.unwrap();
    assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);
}
