#![allow(clippy::unwrap_used, clippy::expect_used)]

use e2e::Context;
use reqwest::{Method, StatusCode};
use serde_json::json;

#[tokio::test]
async fn user_cannot_read_or_change_another_workspace() {
    let Some(ctx) = Context::from_env() else {
        eprintln!("skipping E2E: E2E_BASE_URL is not set");
        return;
    };
    let user_a = ctx.login().await.unwrap();
    let user_b = ctx.login().await.unwrap();
    ctx.create_workspace(&user_a, "home", "E2E Workspace A")
        .await
        .unwrap();
    let workspace_b = ctx
        .create_workspace(&user_b, "home", "E2E Workspace B")
        .await
        .unwrap();
    let ws_b = workspace_b["id"].as_str().unwrap();
    let recipe_response = ctx
        .create_recipe(&user_b, ws_b, "Private Recipe B")
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let etag = recipe_response
        .headers()
        .get("etag")
        .map_or_else(|| "\"1\"".to_owned(), |v| v.to_str().unwrap().to_owned());
    let recipe: serde_json::Value = recipe_response.json().await.unwrap();
    let recipe_id = recipe["id"].as_str().unwrap();

    let document_response = ctx
        .post_json(
            &format!("/v1/workspaces/{ws_b}/scanned-documents"),
            Some(&user_b),
            &json!({
                "form_type": "brew_log",
                "content_type": "application/pdf",
                "size_bytes": 1,
                "sha256": "6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d"
            }),
        )
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let document: serde_json::Value = document_response.json().await.unwrap();
    let document_id = document["document"]["id"].as_str().unwrap();

    for endpoint in [
        format!("/v1/workspaces/{ws_b}/recipes"),
        format!("/v1/workspaces/{ws_b}/batches"),
        format!("/v1/workspaces/{ws_b}/notifications"),
        format!("/v1/workspaces/{ws_b}/scanned-documents"),
        format!("/v1/workspaces/{ws_b}/support/requests"),
    ] {
        let response = ctx
            .request(Method::GET, &endpoint, Some(&user_a))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{endpoint}");
    }

    let recipe_get = ctx
        .request(
            Method::GET,
            &format!("/v1/workspaces/{ws_b}/recipes/{recipe_id}"),
            Some(&user_a),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(recipe_get.status(), StatusCode::NOT_FOUND);

    let document_get = ctx
        .request(
            Method::GET,
            &format!("/v1/workspaces/{ws_b}/scanned-documents/{document_id}"),
            Some(&user_a),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(document_get.status(), StatusCode::NOT_FOUND);

    let document_create = ctx
        .post_json(
            &format!("/v1/workspaces/{ws_b}/scanned-documents"),
            Some(&user_a),
            &json!({
                "form_type": "brew_log",
                "content_type": "application/pdf",
                "size_bytes": 1,
                "sha256": "6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d"
            }),
        )
        .await
        .unwrap();
    assert_eq!(document_create.status(), StatusCode::NOT_FOUND);

    let create = ctx
        .post_json(
            &format!("/v1/workspaces/{ws_b}/recipes"),
            Some(&user_a),
            &json!({"name": "Unauthorized recipe"}),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::NOT_FOUND);

    let update = ctx
        .request(
            Method::PATCH,
            &format!("/v1/workspaces/{ws_b}/recipes/{recipe_id}"),
            Some(&user_a),
        )
        .header("Content-Type", "application/merge-patch+json")
        .header("If-Match", etag)
        .body(r#"{"name":"Unauthorized update"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(update.status(), StatusCode::NOT_FOUND);
}
