#![allow(clippy::unwrap_used, clippy::expect_used)]

use e2e::Context;

#[tokio::test]
#[ignore = "the scenario requires a Compose stack with the package signing key unset; the current Compose workflow starts one shared stack, so this needs a second isolated stack"]
async fn package_operations_fail_without_the_package_key() {
    let Some(ctx) = Context::from_env() else {
        eprintln!("skipping E2E: E2E_BASE_URL is not set");
        return;
    };
    let _user = ctx.login().await.unwrap();
}
