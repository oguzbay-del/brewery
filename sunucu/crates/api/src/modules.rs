//! The registry of API modules: the one place that lists them.
//!
//! A module owns its routes, its `OperationSpec` rows and its `OpenAPI` fragment, all in its own
//! file. To add a module, define a `pub const MODULE: Module` there and add it to [`MODULES`].
//! [`crate::build`] serves the routes, and [`crate::openapi`] reads the rows and the fragments
//! from this list.

use std::sync::OnceLock;

use axum::{Extension, Router, middleware::from_fn_with_state};

use crate::{AppState, openapi::OperationSpec};

/// What a module registers.
#[derive(Debug, Clone, Copy)]
pub struct Module {
    /// Short name, for messages.
    pub name: &'static str,
    /// The routes, already wrapped with `authenticated` / `idempotent` where needed.
    pub routes: fn(&AppState) -> Router<AppState>,
    /// One row per operation of the module.
    pub operations: &'static [OperationSpec],
    /// `OpenAPI` paths, schemas and tags of the module.
    pub openapi: fn() -> utoipa::openapi::OpenApi,
}

/// Every module. The order sets the order of the tags in the `OpenAPI` document.
pub const MODULES: &[Module] = &[
    crate::routes::MODULE,
    crate::events::MODULE,
    crate::files::MODULE,
    crate::library::MODULE,
    crate::auth::handlers::MODULE,
    crate::auth::setup::MODULE,
    crate::auth::access::MODULE,
    crate::auth::passkey::MODULE,
    crate::auth::shared_device::MODULE,
    crate::settings::MODULE,
    crate::subscription::MODULE,
    crate::recipe_org::MODULE,
    crate::recipes::MODULE,
    crate::batches::MODULE,
    crate::brew_day::MODULE,
    crate::stock::MODULE,
    crate::search::MODULE,
    crate::trash::MODULE,
    crate::devices::MODULE,
    crate::notifications::MODULE,
    crate::support::MODULE,
    crate::support_access::MODULE,
    crate::internal::MODULE,
    crate::import_export::MODULE,
    crate::account_deletion::MODULE,
];

/// The routes of all modules, with the auth services as a request extension.
pub fn routes(state: &AppState) -> Router<AppState> {
    let router = MODULES
        .iter()
        .fold(Router::new(), |router, m| router.merge((m.routes)(state)));
    let router = router.layer(from_fn_with_state(
        state.clone(),
        crate::context::device_module_layer,
    ));
    match &state.auth_services {
        Some(auth) => router.layer(Extension(auth.clone())),
        None => router,
    }
}

/// The rows of all operations.
pub fn operations() -> impl Iterator<Item = &'static OperationSpec> {
    MODULES.iter().flat_map(|m| m.operations)
}

/// The row of one operation.
#[must_use]
pub fn operation(operation_id: &str) -> Option<&'static OperationSpec> {
    operations().find(|spec| spec.operation_id == operation_id)
}

/// Product module definition. This is separate from the API Module registry type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductModule {
    /// Stable product module id.
    pub id: &'static str,
    /// Included in Home workspaces.
    pub home: bool,
    /// Included in Business workspaces.
    pub business: bool,
    /// Available without a paid entitlement.
    pub free: bool,
}

/// The one source of truth for product modules, mode availability and free access.
pub const PRODUCT_MODULES: &[ProductModule] = &[
    ProductModule {
        id: "core",
        home: true,
        business: true,
        free: true,
    },
    ProductModule {
        id: "devices",
        home: true,
        business: true,
        free: true,
    },
    // Home has simplified stock and Business has full stock; keep the shared API module free
    // until operation-level separation is implemented.
    ProductModule {
        id: "stock",
        home: true,
        business: true,
        free: true,
    },
    ProductModule {
        id: "production",
        home: false,
        business: true,
        free: false,
    },
    ProductModule {
        id: "quality",
        home: false,
        business: true,
        free: false,
    },
    ProductModule {
        id: "cost_reports",
        home: false,
        business: true,
        free: false,
    },
    ProductModule {
        id: "tax_compliance",
        home: false,
        business: true,
        free: false,
    },
    ProductModule {
        id: "team",
        home: false,
        business: true,
        free: false,
    },
];

/// Maps each API module name to the product module that owns its operations.
/// Keep this table here; do not add a field to Module.
pub const MODULE_KEY_MAP: &[(&str, &str)] = &[
    ("system", "core"),
    ("events", "core"),
    ("files", "core"),
    ("library", "core"),
    ("auth", "core"),
    ("workspaces", "core"),
    ("access", "team"),
    ("passkey", "core"),
    ("shared-devices", "team"),
    ("settings", "core"),
    ("subscription", "core"),
    ("recipe_org", "core"),
    ("recipes", "core"),
    ("batches", "core"),
    ("brew_day", "core"),
    ("stock", "stock"),
    ("search", "core"),
    ("trash", "core"),
    ("devices", "devices"),
    ("notifications", "core"),
    ("support", "core"),
    ("support_access", "core"),
    ("internal", "core"),
    ("import-export", "core"),
    ("account_deletion", "core"),
];

/// Returns the product module for an API module name.
#[must_use]
pub fn module_id_for_name(name: &str) -> Option<&'static str> {
    MODULE_KEY_MAP
        .iter()
        .find_map(|(api_name, product_id)| (*api_name == name).then_some(*product_id))
}

/// Returns whether a product module is free in every workspace mode.
#[must_use]
pub fn is_free_module(module_id: &str) -> bool {
    PRODUCT_MODULES
        .iter()
        .any(|module| module.id == module_id && module.free)
}

/// Evaluates the mode + single `PAID_FEATURE` rule.
#[must_use]
pub fn is_module_enabled(module_id: &str, mode: &str, has_paid_feature: bool) -> bool {
    let Some(module) = PRODUCT_MODULES.iter().find(|module| module.id == module_id) else {
        return false;
    };
    let included = match mode {
        "home" => module.home,
        "business" => module.business,
        _ => false,
    };
    included && (module.free || has_paid_feature)
}

/// Whether the path is an export operation. Exports remain available even when paid modules are off.
#[must_use]
pub fn is_export_path(path: &str) -> bool {
    path.split('/')
        .any(|segment| matches!(segment, "export" | "exports"))
}

fn path_matches(template: &str, actual: &str) -> bool {
    let template: Vec<_> = template.trim_start_matches('/').split('/').collect();
    let actual: Vec<_> = actual.trim_start_matches('/').split('/').collect();
    template.len() == actual.len()
        && template.iter().zip(actual.iter()).all(|(expected, found)| {
            (expected.starts_with('{') && expected.ends_with('}')) || expected == found
        })
}

fn path_module_map() -> &'static [(String, &'static str)] {
    static PATHS: OnceLock<Vec<(String, &'static str)>> = OnceLock::new();
    PATHS.get_or_init(|| {
        let mut paths = Vec::new();
        for module in MODULES {
            let Some(product_id) = module_id_for_name(module.name) else {
                continue;
            };
            let document = (module.openapi)();
            for path in document.paths.paths.keys() {
                paths.push((path.clone(), product_id));
            }
        }
        paths
    })
}

/// Resolves a request path (or `OpenAPI` path template) to its product module.
#[must_use]
pub fn module_for_path(path: &str) -> Option<&'static str> {
    // Accepting an invitation is onboarding; workspace-scoped invitation management is Team.
    if path == "/v1/invitations/accept" {
        return Some("core");
    }
    // Vessel/tank endpoints currently live in the mixed settings API module.
    // Keep general settings free; gate the vessel resource as Production.
    if path.split('/').any(|segment| segment == "vessels") {
        return Some("production");
    }
    path_module_map()
        .iter()
        .find_map(|(template, product_id)| path_matches(template, path).then_some(*product_id))
}
