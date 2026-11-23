#![allow(clippy::unwrap_used, clippy::expect_used)]

use e2e::Context;
use reqwest::StatusCode;
use serde_json::{Value, json};

#[tokio::test]
async fn member_without_recipe_edit_permission_cannot_create_a_recipe() {
    let Some(ctx) = Context::from_env() else {
        eprintln!("skipping E2E: E2E_BASE_URL is not set");
        return;
    };
    let owner = ctx.login().await.unwrap();
    let invited_email = format!("e2e-invite-{}@example.test", uuid::Uuid::new_v4());
    let workspace = ctx.create_workspace(&owner, "home", "E2E Permissions").await.unwrap();
    let ws = workspace["id"].as_str().unwrap();

    let role = ctx.post_json(
        &format!("/v1/workspaces/{ws}/roles"),
        Some(&owner),
        &json!({"name":"E2E Read Only","permissions":["recipe.view"]}),
    ).await.unwrap().error_for_status().unwrap();
    let role: Value = role.json().await.unwrap();
    let role_id = role["id"].as_str().unwrap();

    let invitation = ctx.post_json(
        &format!("/v1/workspaces/{ws}/invitations"),
        Some(&owner),
        &json!({"email":invited_email,"role_id":role_id}),
    ).await.unwrap().error_for_status().unwrap();
    assert_eq!(invitation.status(), StatusCode::CREATED);

    let token = ctx.invitation_token(&invited_email).await.unwrap();
    let member = ctx.login_with_email(&invited_email).await.unwrap();
    let accepted = ctx.post_json(
        "/v1/invitations/accept",
        Some(&member),
        &json!({"token":token}),
    ).await.unwrap().error_for_status().unwrap();
    assert_eq!(accepted.status(), StatusCode::OK);

    let create = ctx.post_json(
        &format!("/v1/workspaces/{ws}/recipes"),
        Some(&member),
        &json!({"name":"Forbidden recipe"}),
    ).await.unwrap();
    assert_eq!(create.status(), StatusCode::FORBIDDEN);
}
