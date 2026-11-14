//! The `OpenAPI` document (ADR-0008 rule 12). Generated from code with utoipa, never edited by
//! hand; `sozlesme/openapi/openapi.json` is its checked-in output.
//!
//! utoipa describes paths and bodies. What every operation must additionally state is kept in
//! one row per operation: the error codes it can return, `x-concurrency`, `x-permission` and
//! whether it needs a sign-in. Each module owns its rows (`OPERATIONS`), its `OpenAPI` fragment
//! and its routes, and registers them in [`crate::modules`]. [`openapi_json`] merges the
//! fragments and the rows into the document, and a test fails when an operation has no row, so
//! a new endpoint cannot skip these rules.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};
use utoipa::OpenApi;

use crate::{
    concurrency::Concurrency,
    error::{ErrorCode, FieldIssue, ProblemDetails},
    modules,
};

/// What an operation must state beyond what its handler macro says.
#[derive(Debug, Clone, Copy)]
pub struct OperationSpec {
    /// `operationId` (`<resource>_<action>`).
    pub operation_id: &'static str,
    /// No sign-in needed (`security: []`).
    pub public: bool,
    /// Permission the operation needs (`x-permission`); `None` is written as `"none"`.
    pub permission: Option<&'static str>,
    /// Concurrency mode of a write (`x-concurrency`); `None` is written as `"none"`.
    pub concurrency: Option<Concurrency>,
    /// Every error code the operation can return.
    pub errors: &'static [ErrorCode],
}

/// Builds a row of the common shape (no `x-concurrency`).
pub(crate) const fn op(
    operation_id: &'static str,
    public: bool,
    permission: Option<&'static str>,
    errors: &'static [ErrorCode],
) -> OperationSpec {
    OperationSpec {
        operation_id,
        public,
        permission,
        concurrency: None,
        errors,
    }
}

/// Builds a row of a signed-in operation with a concurrency mode.
pub(crate) const fn file_op(
    operation_id: &'static str,
    permission: Option<&'static str>,
    concurrency: Option<Concurrency>,
    errors: &'static [ErrorCode],
) -> OperationSpec {
    OperationSpec {
        operation_id,
        public: false,
        permission,
        concurrency,
        errors,
    }
}

/// The shared part of the `OpenAPI` document: the error schemas. Paths, schemas and tags come
/// from the modules (`crate::modules`).
#[derive(Debug, OpenApi)]
#[openapi(components(schemas(ProblemDetails, FieldIssue, ErrorCode)))]
pub struct ApiDoc;

fn example_problem(code: ErrorCode) -> Value {
    let mut v = json!({
        "type": code.type_uri(),
        "title": code.title(),
        "status": code.status().as_u16(),
        "code": code.as_str(),
        "request_id": "0192f1c07a3b7000800000000000000a",
    });
    match code {
        ErrorCode::ValidationFailed => {
            v["errors"] = json!([{
                "field": "/fermentables/0/amount_kg",
                "code": "out_of_range",
                "params": {"min": 0, "max": 1000}
            }]);
        }
        ErrorCode::VersionMismatch => v["current"] = json!({"id": "…", "version": 5}),
        ErrorCode::Conflict => {
            v["errors"] = json!([{"field": "/role_id", "code": "last_admin"}]);
        }
        _ => {}
    }
    v
}

fn problem_responses(codes: &[ErrorCode]) -> Map<String, Value> {
    let mut by_status: BTreeMap<u16, Vec<ErrorCode>> = BTreeMap::default();
    for code in codes {
        by_status
            .entry(code.status().as_u16())
            .or_default()
            .push(*code);
    }
    let mut out = Map::new();
    for (status, codes) in by_status {
        let mut examples = Map::new();
        let mut titles = Vec::new();
        for code in &codes {
            examples.insert(
                code.as_str().to_owned(),
                json!({"summary": code.title(), "value": example_problem(*code)}),
            );
            titles.push(code.title());
        }
        let mut response = json!({
            "description": titles.join("; "),
            "content": {"application/problem+json": {
                "schema": {"$ref": "#/components/schemas/ProblemDetails"},
                "examples": examples
            }}
        });
        if matches!(status, 429 | 503 | 409)
            && codes.iter().any(|c| {
                matches!(
                    c,
                    ErrorCode::RateLimited
                        | ErrorCode::Unavailable
                        | ErrorCode::IdempotencyInProgress
                )
            })
        {
            response["headers"] = json!({"Retry-After": {
                "description": "Seconds to wait before retrying",
                "schema": {"type": "integer", "minimum": 0}
            }});
        }
        out.insert(status.to_string(), response);
    }
    out
}

fn apply_operation_specs(doc: &mut Value) {
    let Some(paths) = doc.get_mut("paths").and_then(Value::as_object_mut) else {
        return;
    };
    for item in paths.values_mut().filter_map(Value::as_object_mut) {
        for operation in item.values_mut().filter_map(Value::as_object_mut) {
            let Some(id) = operation.get("operationId").and_then(Value::as_str) else {
                continue;
            };
            let Some(spec) = modules::operation(id) else {
                continue;
            };
            operation.insert(
                "x-permission".to_owned(),
                json!(spec.permission.unwrap_or("none")),
            );
            operation.insert(
                "x-concurrency".to_owned(),
                json!(spec.concurrency.map_or("none", Concurrency::as_str)),
            );
            operation.insert(
                "security".to_owned(),
                if spec.public {
                    json!([])
                } else {
                    json!([{"cookie": []}, {"bearer": []}])
                },
            );
            if let Some(responses) = operation
                .get_mut("responses")
                .and_then(Value::as_object_mut)
            {
                for (status, response) in problem_responses(spec.errors) {
                    responses.entry(status).or_insert(response);
                }
            }
        }
    }
    doc["components"]["securitySchemes"] = json!({
        "cookie": {"type": "apiKey", "in": "cookie", "name": "__Host-session",
                   "description": "Web session cookie (ADR-0009)"},
        "bearer": {"type": "http", "scheme": "bearer",
                   "description": "Opaque session token of the iOS app (ADR-0009)"}
    });
}

/// Renders the `OpenAPI` document as deterministic pretty JSON with a trailing newline.
///
/// # Errors
/// Returns an error if serialization fails (never expected for this document).
pub fn openapi_json() -> serde_json::Result<String> {
    // Going through `Value` sorts object keys (serde_json without `preserve_order`).
    let mut doc = ApiDoc::openapi();
    for module in modules::MODULES {
        doc.merge((module.openapi)());
    }
    let mut value = serde_json::to_value(doc)?;
    apply_operation_specs(&mut value);
    let mut out = serde_json::to_string_pretty(&value)?;
    out.push('\n');
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc() -> Value {
        serde_json::from_str(&openapi_json().unwrap()).unwrap()
    }

    fn operations(doc: &Value) -> Vec<(String, String, &Value)> {
        let mut out = Vec::new();
        for (path, item) in doc["paths"].as_object().unwrap() {
            for (method, op) in item.as_object().unwrap() {
                out.push((method.clone(), path.clone(), op));
            }
        }
        out
    }

    #[test]
    fn every_operation_follows_the_rules_of_adr_0008() {
        let doc = doc();
        let ops = operations(&doc);
        assert!(!ops.is_empty());
        for (method, path, op) in &ops {
            let label = format!("{method} {path}");
            let id = op["operationId"]
                .as_str()
                .unwrap_or_else(|| panic!("{label}: operationId"));
            assert!(
                id.split('_')
                    .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_lowercase())),
                "{label}: operationId {id} is not <resource>_<action>"
            );
            assert!(
                modules::operation(id).is_some(),
                "{label}: {id} has no row in the OPERATIONS of its module"
            );
            assert!(op.get("x-permission").is_some(), "{label}: x-permission");
            assert!(op.get("x-concurrency").is_some(), "{label}: x-concurrency");
            assert!(op.get("security").is_some(), "{label}: security");
            assert!(
                op["tags"].as_array().is_some_and(|t| !t.is_empty()),
                "{label}: tags"
            );
            let responses = op["responses"].as_object().unwrap();
            for (status, response) in responses {
                let content = response.get("content").and_then(|c| c.as_object());
                if let Some(content) = content {
                    assert!(
                        content
                            .values()
                            .all(|m| m.get("example").is_some() || m.get("examples").is_some()),
                        "{label} {status}: no example"
                    );
                } else {
                    assert!(
                        matches!(status.as_str(), "204" | "304"),
                        "{label} {status}: no content"
                    );
                }
            }
            let spec = modules::operation(id).unwrap();
            for code in spec.errors {
                assert!(
                    responses.contains_key(&code.status().as_u16().to_string()),
                    "{label}: response for {}",
                    code.as_str()
                );
            }
        }
        for spec in modules::operations() {
            assert!(
                ops.iter()
                    .any(|(_, _, op)| op["operationId"] == spec.operation_id),
                "{} has a row but is not in the document",
                spec.operation_id
            );
        }
    }

    #[test]
    fn problem_schema_and_security_schemes_are_present() {
        let doc = doc();
        assert!(doc["components"]["schemas"]["ProblemDetails"].is_object());
        let codes = doc["components"]["schemas"]["ErrorCode"]["enum"]
            .as_array()
            .unwrap();
        assert_eq!(codes.len(), ErrorCode::ALL.len());
        assert!(doc["components"]["securitySchemes"]["cookie"].is_object());
        assert!(doc["components"]["securitySchemes"]["bearer"].is_object());
        assert_eq!(doc["paths"]["/v1/version"]["get"]["security"], json!([]));
    }

    #[test]
    fn document_is_deterministic_and_newline_terminated() {
        let first = openapi_json().unwrap();
        assert_eq!(first, openapi_json().unwrap());
        assert!(first.ends_with("}\n"));
    }
}
