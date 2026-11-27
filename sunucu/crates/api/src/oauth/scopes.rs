//! Scopes and the operation allow list (ADR-0015 rules 3.3, 3.4 and 4).
//!
//! A scope narrows the effective permissions of the member; it never widens them. The
//! allow list maps `operation_id` to one scope. An operation that is not in the list is closed
//! to OAuth tokens. The management permissions never appear here, so no scope grants them.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};

use axum::http::Method;

use crate::{context::Membership, modules};

/// Cost fields (separate scope; the member also needs `cost.view`).
pub const COSTS_READ: &str = "costs:read";
/// Scope for a refresh token. It is only advertised; refresh tokens are always issued.
pub const OFFLINE_ACCESS: &str = "offline_access";

/// One scope of the catalog.
#[derive(Debug, Clone, Copy)]
pub struct ScopeInfo {
    /// The scope value.
    pub name: &'static str,
    /// Writes records.
    pub write: bool,
    /// Permissions the scope can use (the member must hold them as well).
    pub permissions: &'static [&'static str],
}

/// The scope catalog (ADR-0015 rule 4). There is no hierarchy.
pub const SCOPES: &[ScopeInfo] = &[
    ScopeInfo {
        name: "recipes:read",
        write: false,
        permissions: &["recipe.view"],
    },
    ScopeInfo {
        name: "batches:read",
        write: false,
        permissions: &["recipe.view"],
    },
    ScopeInfo {
        name: "stock:read",
        write: false,
        permissions: &["recipe.view"],
    },
    ScopeInfo {
        name: "calc",
        write: false,
        permissions: &["recipe.view"],
    },
    ScopeInfo {
        name: "recipes:draft",
        write: true,
        permissions: &["recipe.edit"],
    },
    ScopeInfo {
        name: "measurements:write",
        write: true,
        permissions: &["batch.record"],
    },
    ScopeInfo {
        name: "shopping:write",
        write: true,
        permissions: &["stock.move"],
    },
    ScopeInfo {
        name: COSTS_READ,
        write: false,
        permissions: &["cost.view"],
    },
];

/// The catalog entry of `scope`.
#[must_use]
pub fn info(scope: &str) -> Option<&'static ScopeInfo> {
    SCOPES.iter().find(|s| s.name == scope)
}

/// Scope values for the metadata documents (`offline_access` included).
#[must_use]
pub fn advertised() -> Vec<&'static str> {
    SCOPES
        .iter()
        .map(|s| s.name)
        .chain([OFFLINE_ACCESS])
        .collect()
}

/// Scopes given when a client asks for none: every read scope except the cost scope.
#[must_use]
pub fn default_scopes() -> Vec<&'static str> {
    SCOPES
        .iter()
        .filter(|s| !s.write && s.name != COSTS_READ)
        .map(|s| s.name)
        .collect()
}

/// The effective membership under `scopes`: the permissions of the role that a scope can use
/// (ADR-0015 rule 3.3). A scoped assignment keeps its scope and loses the permissions that no
/// scope allows.
#[must_use]
pub fn narrow(membership: Membership, scopes: &BTreeSet<String>) -> Membership {
    let allowed: BTreeSet<&str> = scopes
        .iter()
        .filter_map(|s| info(s))
        .flat_map(|s| s.permissions.iter().copied())
        .collect();
    let keep = |set: &BTreeSet<String>| -> BTreeSet<String> {
        set.iter()
            .filter(|p| allowed.contains(p.as_str()))
            .cloned()
            .collect()
    };
    Membership {
        permissions: keep(&membership.permissions),
        scoped: membership
            .scoped
            .into_iter()
            .map(|mut g| {
                g.permissions = keep(&g.permissions);
                g
            })
            .filter(|g| !g.permissions.is_empty())
            .collect(),
    }
}

/// The scopes that the role can use at all (for the consent screen).
#[must_use]
pub fn usable_by(permissions: &BTreeSet<String>) -> Vec<&'static str> {
    SCOPES
        .iter()
        .filter(|s| s.permissions.iter().all(|p| permissions.contains(*p)))
        .map(|s| s.name)
        .collect()
}

/// Operations that an OAuth token may call, with the scope each needs (ADR-0015 rule 4).
/// Publish, approve, delete, state change, stock movement and protected operations are not here.
pub const OPERATION_SCOPES: &[(&str, &str)] = &[
    ("search", "recipes:read"),
    ("recipe_list", "recipes:read"),
    ("recipe_get", "recipes:read"),
    ("recipe_version_list", "recipes:read"),
    ("recipe_version_get", "recipes:read"),
    ("recipe_version_compare", "recipes:read"),
    ("batch_list", "batches:read"),
    ("batch_get", "batches:read"),
    ("batchstep_list", "batches:read"),
    ("batch_summary", "batches:read"),
    ("measurement_list", "batches:read"),
    ("stock_lot_list", "stock:read"),
    ("stock_totals", "stock:read"),
    ("shopping_list", "stock:read"),
    ("recipe_version_calculate", "calc"),
    ("recipe_version_scale", "calc"),
    ("recipe_create", "recipes:draft"),
    ("recipe_version_create", "recipes:draft"),
    ("recipe_version_update", "recipes:draft"),
    ("measurement_create", "measurements:write"),
    ("shopping_from_recipe", "shopping:write"),
    ("shopping_update", "shopping:write"),
];

/// The scope an operation needs; `None` when OAuth tokens may not call it.
#[must_use]
pub fn scope_of_operation(operation_id: &str) -> Option<&'static str> {
    OPERATION_SCOPES
        .iter()
        .find_map(|(id, scope)| (*id == operation_id).then_some(*scope))
}

type Table = BTreeMap<String, Vec<(Method, String)>>;

/// `path template` → `(method, operation id)`, from the module fragments.
fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut out = Table::new();
        for module in modules::MODULES {
            let doc = (module.openapi)();
            for (path, item) in &doc.paths.paths {
                let ops = [
                    (Method::GET, &item.get),
                    (Method::POST, &item.post),
                    (Method::PUT, &item.put),
                    (Method::PATCH, &item.patch),
                    (Method::DELETE, &item.delete),
                ];
                for (method, op) in ops {
                    if let Some(id) = op.as_ref().and_then(|o| o.operation_id.clone()) {
                        out.entry(path.clone()).or_default().push((method, id));
                    }
                }
            }
        }
        out
    })
}

/// The `operation_id` of a request, when the document knows it. A template with more literal
/// segments wins over one with parameters.
#[must_use]
pub fn operation_for(method: &Method, path: &str) -> Option<&'static str> {
    let literal = |t: &str| t.split('/').filter(|s| !s.starts_with('{')).count();
    table()
        .iter()
        .filter(|(template, _)| modules::path_matches(template, path))
        .filter_map(|(template, ops)| {
            ops.iter()
                .find(|(m, _)| m == method)
                .map(|(_, id)| (literal(template), id.as_str()))
        })
        .max_by_key(|(n, _)| *n)
        .map(|(_, id)| id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authz::{ScopeKind, ScopedGrant};
    use uuid::Uuid;

    fn set(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn a_scope_narrows_the_role_and_never_widens_it() {
        let m = Membership {
            permissions: set(&["recipe.view", "recipe.edit", "cost.view", "users.manage"]),
            scoped: vec![ScopedGrant {
                kind: ScopeKind::Batch,
                id: Uuid::nil(),
                permissions: set(&["batch.record", "recipe.view"]),
            }],
        };
        let read = narrow(m.clone(), &set(&["recipes:read"]));
        assert_eq!(read.permissions, set(&["recipe.view"]));
        assert_eq!(read.scoped[0].permissions, set(&["recipe.view"]));
        let costs = narrow(m.clone(), &set(&["recipes:read", "costs:read"]));
        assert!(costs.permissions.contains("cost.view"));
        assert!(!read.permissions.contains("cost.view"));
        // A scope cannot give what the role lacks.
        let none = narrow(
            Membership {
                permissions: set(&["recipe.view"]),
                scoped: vec![],
            },
            &set(&["recipes:draft"]),
        );
        assert!(none.permissions.is_empty());
        // No scope opens a management permission.
        let all: BTreeSet<String> = SCOPES.iter().map(|s| s.name.to_owned()).collect();
        assert!(!narrow(m, &all).permissions.contains("users.manage"));
    }

    #[test]
    fn every_listed_operation_exists_and_is_known_to_the_document() {
        for (id, scope) in OPERATION_SCOPES {
            assert!(info(scope).is_some(), "{id}: unknown scope {scope}");
            assert!(
                table().values().flatten().any(|(_, op)| op == id),
                "{id} is not an operation of the API"
            );
        }
    }

    #[test]
    fn a_request_maps_to_its_operation() {
        let ws = Uuid::nil();
        assert_eq!(
            operation_for(&Method::GET, &format!("/v1/workspaces/{ws}/recipes")),
            Some("recipe_list")
        );
        assert_eq!(
            operation_for(
                &Method::DELETE,
                &format!("/v1/workspaces/{ws}/recipes/{ws}")
            ),
            Some("recipe_delete")
        );
        assert_eq!(operation_for(&Method::GET, "/nothing"), None);
    }
}
