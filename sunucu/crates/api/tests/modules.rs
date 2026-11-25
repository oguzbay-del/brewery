//! P1 module map, entitlement gate, metadata endpoint and OpenAPI extension tests.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::{sync::Arc};

use api::{
    AppState,
    modules::{self, MODULES},
    redis_client::Redis,
};
use axum::{
    Router,
    body::Body,
    http::StatusCode,
    middleware::from_fn_with_state,
    routing::get,
};
use serde_json::Value;
use sqlx::PgPool;
use support::{FakeAuth, insert_user, insert_workspace, send, state_with};
use uuid::Uuid;

fn team_path_template() -> String {
    let module = MODULES.iter().find(|module| module.name == "access").unwrap();
    let document = (module.openapi)();
    document
        .paths
        .paths
        .keys()
        .find(|path| path.starts_with("/v1/workspaces/"))
        .unwrap()
        .clone()
}

fn concrete_path(template: &str, workspace_id: Uuid) -> String {
    template
        .split('/')
        .map(|segment| {
            if segment.starts_with('{') && segment.ends_with('}') {
                workspace_id.to_string()
            } else {
                segment.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

async fn insert_paid_feature(pool: &PgPool, workspace_id: Uuid) {
    sqlx::query(
        "INSERT INTO entitlements (id, workspace_id, feature, source, valid_from, valid_until)
         VALUES ($1, $2, $3, 'promo', now() - interval '1 day', now() + interval '1 day')",
    )
    .bind(Uuid::now_v7())
    .bind(workspace_id)
    .bind(billing::PAID_FEATURE)
    .execute(pool)
    .await
    .unwrap();
}

fn module_gate_router(state: AppState, path: &str) -> Router {
    Router::new()
        .route(path, get(|| async { StatusCode::OK }))
        .route_layer(from_fn_with_state(state.clone(), api::context::layer))
        .with_state(state)
}

#[test]
fn every_registered_api_module_has_a_product_module_mapping() {
    for module in MODULES {
        assert!(
            modules::module_id_for_name(module.name).is_some(),
            "missing product module mapping for {}",
            module.name
        );
    }
}

#[test]
fn home_business_and_single_paid_feature_rules_are_stable() {
    for module in ["core", "devices", "stock"] {
        assert!(modules::is_module_enabled(module, "home", false));
        assert!(modules::is_module_enabled(module, "business", false));
    }
    assert!(!modules::is_module_enabled("team", "home", true));
    assert!(!modules::is_module_enabled("team", "business", false));
    assert!(modules::is_module_enabled("team", "business", true));
    assert!(!modules::is_module_enabled("production", "home", true));
    assert!(modules::is_module_enabled("production", "business", true));
    assert!(!modules::is_module_enabled("unknown", "business", true));
}

#[test]
fn paths_resolve_to_product_modules_and_exports_are_exempt() {
    let team_path = team_path_template();
    assert_eq!(modules::module_for_path(&team_path), Some("team"));
    assert_eq!(
        modules::module_for_path("/v1/shared-devices/{id}/members"),
        Some("team")
    );
    assert_eq!(
        modules::module_for_path("/v1/workspaces/{workspace_id}/vessels"),
        Some("production")
    );
    assert!(modules::is_export_path("/v1/workspaces/{workspace_id}/exports"));
    assert!(!modules::is_export_path("/v1/workspaces/{workspace_id}/stock/lots"));
}

#[test]
fn every_openapi_operation_has_a_product_module_extension() {
    let document: Value = serde_json::from_str(&api::openapi_json().unwrap()).unwrap();
    for (path, item) in document["paths"].as_object().unwrap() {
        for (method, operation) in item.as_object().unwrap() {
            if !matches!(
                method.as_str(),
                "get" | "post" | "put" | "patch" | "delete" | "head" | "options" | "trace"
            ) {
                continue;
            }
            let module_id = operation["x-module"]
                .as_str()
                .unwrap_or_else(|| panic!("missing x-module for {method} {path}"));
            if !modules::is_free_module(module_id) {
                assert!(
                    operation["responses"]["403"]["content"]["application/problem+json"]
                        ["examples"]["module_disabled"]
                        .is_object(),
                    "missing module_disabled response for {method} {path}"
                );
            }
        }
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn team_routes_require_business_mode_and_one_active_paid_feature(pool: PgPool) {
    let user = Uuid::now_v7();
    let workspace = Uuid::now_v7();
    insert_user(&pool, user).await;
    insert_workspace(&pool, workspace).await;

    let auth = Arc::new(FakeAuth::default());
    auth.add(user, workspace, &[]);
    let state = state_with(
        &pool,
        Redis::disabled(),
        auth,
        support::config("postgres://unused"),
    )
    .await;

    let template = team_path_template();
    let path = concrete_path(&template, workspace);
    let app = module_gate_router(state, &template);
    let request = support::req("GET", &path, Some(user)).body(Body::empty()).unwrap();
    let disabled = send(&app, request).await;
    assert_eq!(disabled.status, StatusCode::FORBIDDEN);
    assert_eq!(disabled.body["code"], "module_disabled");

    insert_paid_feature(&pool, workspace).await;
    let request = support::req("GET", &path, Some(user)).body(Body::empty()).unwrap();
    let enabled = send(&app, request).await;
    assert_eq!(enabled.status, StatusCode::OK);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn meta_endpoint_returns_the_callers_workspace_module_state(pool: PgPool) {
    let user = Uuid::now_v7();
    let workspace = Uuid::now_v7();
    insert_user(&pool, user).await;
    insert_workspace(&pool, workspace).await;

    let auth = Arc::new(FakeAuth::default());
    auth.add(user, workspace, &[]);
    let state = state_with(
        &pool,
        Redis::disabled(),
        auth,
        support::config("postgres://unused"),
    )
    .await;
    let app = api::build(state, Router::new());

    let request = support::req(
        "GET",
        &format!("/v1/meta?workspace_id={workspace}"),
        Some(user),
    )
    .body(Body::empty())
    .unwrap();
    let reply = send(&app, request).await;
    assert_eq!(reply.status, StatusCode::OK, "{:?}", reply.body);
    let modules = reply.body["modules"].as_array().unwrap();
    let enabled = |id: &str| {
        modules
            .iter()
            .find(|module| module["id"] == id)
            .and_then(|module| module["enabled"].as_bool())
            .unwrap()
    };
    assert!(enabled("core"));
    assert!(enabled("devices"));
    assert!(enabled("stock"));
    assert!(!enabled("team"));

    insert_paid_feature(&pool, workspace).await;
    let request = support::req(
        "GET",
        &format!("/v1/meta?workspace_id={workspace}"),
        Some(user),
    )
    .body(Body::empty())
    .unwrap();
    let paid_reply = send(&app, request).await;
    assert_eq!(paid_reply.status, StatusCode::OK);
    let paid_modules = paid_reply.body["modules"].as_array().unwrap();
    assert!(paid_modules
        .iter()
        .find(|module| module["id"] == "team")
        .and_then(|module| module["enabled"].as_bool())
        .unwrap());
}
