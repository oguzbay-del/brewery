#![allow(clippy::unwrap_used, clippy::expect_used)]

use e2e::Context;

#[tokio::test]
#[ignore = "the public OpenAPI contract has no HTTP operation to alter a stored package signature; a dedicated fixture or API test seam is required to create this precondition without database access"]
async fn measurement_rejects_a_package_with_an_invalid_signature() {
    let Some(ctx) = Context::from_env() else {
        eprintln!("skipping E2E: E2E_BASE_URL is not set");
        return;
    };
    let _user = ctx.login().await.unwrap();
}
