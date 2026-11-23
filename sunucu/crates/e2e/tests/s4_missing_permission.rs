#![allow(clippy::unwrap_used, clippy::expect_used)]

use e2e::Context;
use reqwest::{Method, StatusCode};
use serde_json::{Value, json};
use uuid::Uuid;

#[allow(clippy::too_many_lines)]
#[tokio::test]
async fn member_without_permissions_cannot_perform_protected_actions() {
    let Some(ctx) = Context::from_env() else {
        eprintln!("skipping E2E: E2E_BASE_URL is not set");
        return;
    };
    let owner = ctx.login().await.unwrap();
    let invited_email = format!("e2e-invite-{}@example.test", Uuid::new_v4());
    let workspace = ctx
        .create_workspace(&owner, "business", "E2E Permissions")
        .await
        .unwrap();
    let ws = workspace["id"].as_str().unwrap();

    // Prepare valid records as the owner so every denied request reaches its permission check.
    let recipe_response = ctx
        .create_recipe(&owner, ws, "E2E Permission Recipe")
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let recipe: Value = recipe_response.json().await.unwrap();
    let version_id = recipe["draft_version_id"].as_str().unwrap();
    let version_response = ctx
        .request(
            Method::GET,
            &format!("/v1/workspaces/{ws}/recipe-versions/{version_id}"),
            Some(&owner),
        )
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let version_etag = version_response
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
            Some(&owner),
        )
        .header("If-Match", &version_etag)
        .header("Idempotency-Key", Uuid::new_v4().to_string())
        .json(&json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(publish.status(), StatusCode::OK);

    let batch = ctx
        .post_json(
            &format!("/v1/workspaces/{ws}/batches"),
            Some(&owner),
            &json!({"recipe_version_id": version_id, "target_volume_l": 20.0}),
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
            Some(&owner),
            &json!({}),
        )
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let package: Value = package.json().await.unwrap();
    let package_id = package["id"].as_str().unwrap();

    let role = ctx
        .post_json(
            &format!("/v1/workspaces/{ws}/roles"),
            Some(&owner),
            &json!({"name":"E2E Cost Viewer","permissions":["cost.view"]}),
        )
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let role: Value = role.json().await.unwrap();
    let role_id = role["id"].as_str().unwrap();

    let invitation = ctx
        .post_json(
            &format!("/v1/workspaces/{ws}/invitations"),
            Some(&owner),
            &json!({"email":invited_email,"role_id":role_id}),
        )
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    assert_eq!(invitation.status(), StatusCode::CREATED);

    let token = ctx.invitation_token(&invited_email).await.unwrap();
    let member = ctx.login_with_email(&invited_email).await.unwrap();
    let accepted = ctx
        .post_json(
            "/v1/invitations/accept",
            Some(&member),
            &json!({"token":token}),
        )
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    assert_eq!(accepted.status(), StatusCode::OK);

    let denied = [
        ctx.post_json(
            &format!("/v1/workspaces/{ws}/recipes"),
            Some(&member),
            &json!({"name":"Forbidden recipe"}),
        )
        .await
        .unwrap(),
        ctx.request(
            Method::POST,
            &format!("/v1/workspaces/{ws}/recipe-versions/{version_id}/publish"),
            Some(&member),
        )
        .header("If-Match", version_etag)
        .header("Idempotency-Key", Uuid::new_v4().to_string())
        .json(&json!({}))
        .send()
        .await
        .unwrap(),
        ctx.post_json(
            &format!("/v1/workspaces/{ws}/batches"),
            Some(&member),
            &json!({"recipe_version_id":version_id}),
        )
        .await
        .unwrap(),
        ctx.post_json(
            &format!("/v1/workspaces/{ws}/batches/{batch_id}/brew-day-package"),
            Some(&member),
            &json!({}),
        )
        .await
        .unwrap(),
        ctx.post_json(
            &format!("/v1/workspaces/{ws}/batches/{batch_id}/measurements"),
            Some(&member),
            &json!({"items":[{
                "id":Uuid::new_v4(),"kind":"sg","value":1.05,
                "measured_at":"2026-11-23T12:00:00Z","package_id":package_id
            }]}),
        )
        .await
        .unwrap(),
        ctx.request(
            Method::GET,
            &format!("/v1/workspaces/{ws}/notifications"),
            Some(&member),
        )
        .send()
        .await
        .unwrap(),
        ctx.post_json(
            &format!("/v1/workspaces/{ws}/support/requests"),
            Some(&member),
            &json!({"topic":"recipes","title":"Forbidden support request","body":"No permission"}),
        )
        .await
        .unwrap(),
    ];
    for (index, response) in denied.into_iter().enumerate() {
        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "permission case {}",
            index + 1
        );
    }
}
