#![allow(clippy::unwrap_used, clippy::expect_used)]

use e2e::Context;
use reqwest::{Method, StatusCode};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn protected_flow_endpoints_reject_a_missing_session() {
    let Some(ctx) = Context::from_env() else {
        eprintln!("skipping E2E: E2E_BASE_URL is not set");
        return;
    };
    let ws = Uuid::new_v4();
    let id = Uuid::new_v4();
    let requests = [
        ctx.request(Method::GET, "/v1/me/session", None).send().await.unwrap(),
        ctx.post_json("/v1/workspaces", None, &json!({
            "name":"Unauthenticated","mode":"home","region":"us","country":"US","time_zone":"America/New_York"
        })).await.unwrap(),
        ctx.post_json(&format!("/v1/workspaces/{ws}/recipes"), None, &json!({"name":"Unauthenticated"})).await.unwrap(),
        ctx.request(Method::POST, &format!("/v1/workspaces/{ws}/recipe-versions/{id}/publish"), None)
            .header("If-Match", "\"1\"").header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(&json!({})).send().await.unwrap(),
        ctx.post_json(&format!("/v1/workspaces/{ws}/batches"), None, &json!({"recipe_version_id":id})).await.unwrap(),
        ctx.post_json(&format!("/v1/workspaces/{ws}/batches/{id}/brew-day-package"), None, &json!({})).await.unwrap(),
        ctx.post_json(&format!("/v1/workspaces/{ws}/batches/{id}/measurements"), None, &json!({"items":[]})).await.unwrap(),
    ];
    for response in requests {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
