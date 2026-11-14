//! The registry of API modules: the one place that lists them.
//!
//! A module owns its routes, its `OperationSpec` rows and its `OpenAPI` fragment, all in its own
//! file. To add a module, define a `pub const MODULE: Module` there and add it to [`MODULES`].
//! [`crate::build`] serves the routes, and [`crate::openapi`] reads the rows and the fragments
//! from this list.

use axum::{Extension, Router};

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
    crate::auth::handlers::MODULE,
    crate::auth::setup::MODULE,
    crate::auth::access::MODULE,
    crate::auth::passkey::MODULE,
    crate::auth::shared_device::MODULE,
];

/// The routes of all modules, with the auth services as a request extension.
pub fn routes(state: &AppState) -> Router<AppState> {
    let router = MODULES
        .iter()
        .fold(Router::new(), |router, m| router.merge((m.routes)(state)));
    match &state.auth_services {
        Some(auth) => router.layer(Extension(auth.clone())),
        None => router,
    }
}

/// The rows of all modules.
pub fn operations() -> impl Iterator<Item = &'static OperationSpec> {
    MODULES.iter().flat_map(|m| m.operations)
}

/// The row of one operation.
#[must_use]
pub fn operation(operation_id: &str) -> Option<&'static OperationSpec> {
    operations().find(|spec| spec.operation_id == operation_id)
}
