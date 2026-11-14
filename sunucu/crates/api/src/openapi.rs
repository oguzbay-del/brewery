//! The `OpenAPI` document (ADR-0008 rule 12). Generated from code with utoipa, never edited by
//! hand; `sozlesme/openapi/openapi.json` is its checked-in output.
//!
//! utoipa describes paths and bodies. What every operation must additionally state is kept in
//! one table, [`OPERATIONS`]: the error codes it can return, `x-concurrency`, `x-permission`
//! and whether it needs a sign-in. [`openapi_json`] merges the table into the document, and a
//! test fails when an operation is missing from it, so a new endpoint cannot skip these rules.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};
use utoipa::OpenApi;

use crate::{
    app_version::ClientRequirements,
    auth::{
        access,
        handlers::{
            self as auth_handlers, AccountHint, ClientParam, EmailStartRequest, EmailStartResponse,
            EmailVerifyRequest, IdentitySummary, LinkIdentityRequest, LinkResponse, LoginResponse,
            NonceResponse, ProviderLoginRequest, SessionDetail, SessionInfo, SessionIssued,
            SessionItem, UserSummary, WorkspaceSummary,
        },
        setup::{self, WorkspaceCreateRequest, WorkspaceResponse},
    },
    concurrency::Concurrency,
    error::{ErrorCode, FieldIssue, ProblemDetails},
    files,
    routes::{self, HealthResponse, ReadyResponse, VersionResponse},
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

use ErrorCode::{
    Conflict, Forbidden, Internal, MalformedRequest, NotFound, PreconditionRequired, RateLimited,
    StepUpRequired, Unauthenticated, Unavailable, UnsupportedMediaType, UpgradeRequired,
    ValidationFailed,
};

const PUBLIC_AUTH_ERRORS: &[ErrorCode] = &[
    MalformedRequest,
    Unauthenticated,
    NotFound,
    UnsupportedMediaType,
    ValidationFailed,
    UpgradeRequired,
    RateLimited,
    Unavailable,
    Internal,
];
const ME_ERRORS: &[ErrorCode] = &[
    MalformedRequest,
    Unauthenticated,
    Forbidden,
    NotFound,
    UpgradeRequired,
    RateLimited,
    Internal,
];

use ErrorCode::{IdempotencyInProgress, IdempotencyMismatch, PayloadTooLarge, VersionMismatch};

const FILE_READ: &[ErrorCode] = &[
    MalformedRequest,
    Unauthenticated,
    NotFound,
    RateLimited,
    UpgradeRequired,
    Internal,
    Unavailable,
];
const FILE_CREATE: &[ErrorCode] = &[
    MalformedRequest,
    Unauthenticated,
    Forbidden,
    NotFound,
    IdempotencyInProgress,
    PayloadTooLarge,
    UnsupportedMediaType,
    ValidationFailed,
    IdempotencyMismatch,
    PreconditionRequired,
    RateLimited,
    UpgradeRequired,
    Internal,
    Unavailable,
];
const FILE_COMPLETE: &[ErrorCode] = &[
    MalformedRequest,
    Unauthenticated,
    Forbidden,
    NotFound,
    ValidationFailed,
    RateLimited,
    UpgradeRequired,
    Internal,
    Unavailable,
];
const FILE_DOWNLOAD: &[ErrorCode] = &[
    MalformedRequest,
    Unauthenticated,
    NotFound,
    Conflict,
    RateLimited,
    UpgradeRequired,
    Internal,
    Unavailable,
];
const FILE_DELETE: &[ErrorCode] = &[
    MalformedRequest,
    Unauthenticated,
    Forbidden,
    NotFound,
    Conflict,
    VersionMismatch,
    RateLimited,
    UpgradeRequired,
    Internal,
    Unavailable,
];

const ACCESS_READ: &[ErrorCode] = &[
    MalformedRequest,
    Unauthenticated,
    Forbidden,
    NotFound,
    RateLimited,
    UpgradeRequired,
    Internal,
];
const ACCESS_CREATE: &[ErrorCode] = &[
    MalformedRequest,
    Unauthenticated,
    Forbidden,
    NotFound,
    Conflict,
    IdempotencyInProgress,
    UnsupportedMediaType,
    ValidationFailed,
    IdempotencyMismatch,
    PreconditionRequired,
    RateLimited,
    UpgradeRequired,
    Internal,
];
const ACCESS_UPDATE: &[ErrorCode] = &[
    MalformedRequest,
    Unauthenticated,
    Forbidden,
    NotFound,
    Conflict,
    VersionMismatch,
    UnsupportedMediaType,
    ValidationFailed,
    PreconditionRequired,
    RateLimited,
    UpgradeRequired,
    Internal,
];
const ACCESS_DELETE: &[ErrorCode] = &[
    MalformedRequest,
    Unauthenticated,
    Forbidden,
    NotFound,
    Conflict,
    RateLimited,
    UpgradeRequired,
    Internal,
];
const ACCESS_ACCEPT: &[ErrorCode] = &[
    MalformedRequest,
    Unauthenticated,
    Forbidden,
    NotFound,
    Conflict,
    UnsupportedMediaType,
    RateLimited,
    UpgradeRequired,
    Internal,
];

const fn file_op(
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

/// Specs of every operation in the document. New endpoints add a line here.
pub const OPERATIONS: &[OperationSpec] = &[
    OperationSpec {
        operation_id: "events_stream",
        public: false,
        permission: None,
        concurrency: None,
        errors: &[
            ErrorCode::Unauthenticated,
            ErrorCode::NotFound,
            ErrorCode::ValidationFailed,
            UpgradeRequired,
            RateLimited,
            Internal,
            ErrorCode::Unavailable,
        ],
    },
    OperationSpec {
        operation_id: "health_check",
        public: true,
        permission: None,
        concurrency: None,
        errors: &[Internal],
    },
    OperationSpec {
        operation_id: "health_ready",
        public: true,
        permission: None,
        concurrency: None,
        errors: &[Internal, ErrorCode::Unavailable],
    },
    OperationSpec {
        operation_id: "version_get",
        public: true,
        permission: None,
        concurrency: None,
        errors: &[MalformedRequest, UpgradeRequired, RateLimited, Internal],
    },
    OperationSpec {
        operation_id: "meta_client_requirements_get",
        public: true,
        permission: None,
        concurrency: None,
        errors: &[RateLimited, Internal],
    },
    OperationSpec {
        operation_id: "meta_fields_get",
        public: true,
        permission: None,
        concurrency: None,
        errors: &[MalformedRequest, UpgradeRequired, RateLimited, Internal],
    },
    file_op(
        "attachment_create",
        Some(files::WRITE_PERMISSION),
        Some(Concurrency::Append),
        FILE_CREATE,
    ),
    file_op(
        "attachment_complete",
        Some(files::WRITE_PERMISSION),
        Some(Concurrency::FieldLww),
        FILE_COMPLETE,
    ),
    file_op("attachment_list", None, None, FILE_READ),
    file_op("attachment_get", None, None, FILE_READ),
    file_op("attachment_download", None, None, FILE_DOWNLOAD),
    file_op(
        "attachment_delete",
        Some(files::WRITE_PERMISSION),
        Some(Concurrency::FieldLww),
        FILE_DELETE,
    ),
    file_op(
        "scanned_document_create",
        Some(files::WRITE_PERMISSION),
        Some(Concurrency::Append),
        FILE_CREATE,
    ),
    file_op(
        "scanned_document_complete",
        Some(files::WRITE_PERMISSION),
        Some(Concurrency::FieldLww),
        FILE_COMPLETE,
    ),
    file_op("scanned_document_list", None, None, FILE_READ),
    file_op("scanned_document_get", None, None, FILE_READ),
    file_op("scanned_document_download", None, None, FILE_DOWNLOAD),
    file_op(
        "scanned_document_delete",
        Some(files::WRITE_PERMISSION),
        Some(Concurrency::FieldLww),
        FILE_DELETE,
    ),
    OperationSpec {
        operation_id: "auth_nonce_create",
        public: true,
        permission: None,
        concurrency: None,
        errors: &[MalformedRequest, UpgradeRequired, RateLimited, Internal],
    },
    OperationSpec {
        operation_id: "auth_apple_login",
        public: true,
        permission: None,
        concurrency: None,
        errors: PUBLIC_AUTH_ERRORS,
    },
    OperationSpec {
        operation_id: "auth_google_login",
        public: true,
        permission: None,
        concurrency: None,
        errors: PUBLIC_AUTH_ERRORS,
    },
    OperationSpec {
        operation_id: "auth_email_verify",
        public: true,
        permission: None,
        concurrency: None,
        errors: PUBLIC_AUTH_ERRORS,
    },
    OperationSpec {
        operation_id: "auth_email_start",
        public: true,
        permission: None,
        concurrency: None,
        errors: &[
            MalformedRequest,
            UnsupportedMediaType,
            ValidationFailed,
            UpgradeRequired,
            RateLimited,
            Internal,
        ],
    },
    OperationSpec {
        operation_id: "me_session_get",
        public: false,
        permission: None,
        concurrency: None,
        errors: ME_ERRORS,
    },
    OperationSpec {
        operation_id: "me_session_delete",
        public: false,
        permission: None,
        concurrency: None,
        errors: ME_ERRORS,
    },
    OperationSpec {
        operation_id: "me_sessions_list",
        public: false,
        permission: None,
        concurrency: None,
        errors: ME_ERRORS,
    },
    OperationSpec {
        operation_id: "me_sessions_revoke_all",
        public: false,
        permission: None,
        concurrency: None,
        errors: ME_ERRORS,
    },
    OperationSpec {
        operation_id: "me_identities_list",
        public: false,
        permission: None,
        concurrency: None,
        errors: ME_ERRORS,
    },
    OperationSpec {
        operation_id: "me_session_revoke",
        public: false,
        permission: None,
        concurrency: None,
        errors: ME_ERRORS,
    },
    OperationSpec {
        operation_id: "me_identities_link",
        public: false,
        permission: None,
        concurrency: None,
        errors: &[
            MalformedRequest,
            Unauthenticated,
            Forbidden,
            NotFound,
            Conflict,
            UnsupportedMediaType,
            ValidationFailed,
            UpgradeRequired,
            RateLimited,
            Unavailable,
            Internal,
        ],
    },
    OperationSpec {
        operation_id: "me_identities_unlink",
        public: false,
        permission: None,
        concurrency: None,
        errors: &[
            MalformedRequest,
            Unauthenticated,
            Forbidden,
            StepUpRequired,
            NotFound,
            Conflict,
            UpgradeRequired,
            RateLimited,
            Internal,
        ],
    },
    OperationSpec {
        operation_id: "roles_list",
        public: false,
        permission: Some("users.manage"),
        concurrency: None,
        errors: ACCESS_READ,
    },
    OperationSpec {
        operation_id: "roles_create",
        public: false,
        permission: Some("users.manage"),
        concurrency: Some(Concurrency::Append),
        errors: ACCESS_CREATE,
    },
    OperationSpec {
        operation_id: "roles_copy",
        public: false,
        permission: Some("users.manage"),
        concurrency: Some(Concurrency::Append),
        errors: ACCESS_CREATE,
    },
    OperationSpec {
        operation_id: "roles_update",
        public: false,
        permission: Some("users.manage"),
        concurrency: Some(Concurrency::Optimistic),
        errors: ACCESS_UPDATE,
    },
    OperationSpec {
        operation_id: "roles_delete",
        public: false,
        permission: Some("users.manage"),
        concurrency: Some(Concurrency::Optimistic),
        errors: ACCESS_UPDATE,
    },
    OperationSpec {
        operation_id: "members_list",
        public: false,
        permission: Some("users.manage"),
        concurrency: None,
        errors: ACCESS_READ,
    },
    OperationSpec {
        operation_id: "members_update",
        public: false,
        permission: Some("users.manage"),
        concurrency: Some(Concurrency::FieldLww),
        errors: ACCESS_UPDATE,
    },
    OperationSpec {
        operation_id: "members_remove",
        public: false,
        permission: Some("users.manage"),
        concurrency: Some(Concurrency::FieldLww),
        errors: ACCESS_DELETE,
    },
    OperationSpec {
        operation_id: "assignments_create",
        public: false,
        permission: Some("users.manage"),
        concurrency: Some(Concurrency::Append),
        errors: ACCESS_CREATE,
    },
    OperationSpec {
        operation_id: "assignments_delete",
        public: false,
        permission: Some("users.manage"),
        concurrency: Some(Concurrency::FieldLww),
        errors: ACCESS_DELETE,
    },
    OperationSpec {
        operation_id: "invitations_create",
        public: false,
        permission: Some("users.manage"),
        concurrency: Some(Concurrency::Append),
        errors: ACCESS_CREATE,
    },
    OperationSpec {
        operation_id: "invitations_list",
        public: false,
        permission: Some("users.manage"),
        concurrency: None,
        errors: ACCESS_READ,
    },
    OperationSpec {
        operation_id: "invitations_revoke",
        public: false,
        permission: Some("users.manage"),
        concurrency: Some(Concurrency::FieldLww),
        errors: ACCESS_DELETE,
    },
    OperationSpec {
        operation_id: "invitations_accept",
        public: false,
        permission: None,
        concurrency: None,
        errors: ACCESS_ACCEPT,
    },
    OperationSpec {
        operation_id: "shares_create",
        public: false,
        permission: Some("users.manage"),
        concurrency: Some(Concurrency::Append),
        errors: ACCESS_CREATE,
    },
    OperationSpec {
        operation_id: "workspaces_create",
        public: false,
        permission: None,
        concurrency: None,
        errors: &[
            MalformedRequest,
            Unauthenticated,
            Forbidden,
            Conflict,
            UnsupportedMediaType,
            ValidationFailed,
            UpgradeRequired,
            PreconditionRequired,
            RateLimited,
            Internal,
        ],
    },
];

/// The `OpenAPI` document of the public API.
#[derive(Debug, OpenApi)]
#[openapi(
    paths(
        routes::healthz,
        routes::readyz,
        routes::version,
        routes::client_requirements,
        routes::meta_fields,
        crate::events::workspace_events,
        files::attachment_create,
        files::attachment_complete,
        files::attachment_list,
        files::attachment_get,
        files::attachment_download,
        files::attachment_delete,
        files::scanned_document_create,
        files::scanned_document_complete,
        files::scanned_document_list,
        files::scanned_document_get,
        files::scanned_document_download,
        files::scanned_document_delete,
        auth_handlers::nonce_create,
        auth_handlers::apple_login,
        auth_handlers::google_login,
        auth_handlers::email_start,
        auth_handlers::email_verify,
        auth_handlers::session_get,
        auth_handlers::session_delete,
        auth_handlers::sessions_list,
        auth_handlers::sessions_revoke_all,
        auth_handlers::session_revoke,
        auth_handlers::identities_list,
        auth_handlers::identity_link,
        auth_handlers::identity_unlink,
        setup::workspace_create,
        access::roles_list,
        access::roles_create,
        access::roles_copy,
        access::roles_update,
        access::roles_delete,
        access::members_list,
        access::members_update,
        access::members_remove,
        access::assignments_create,
        access::assignments_delete,
        access::invitations_create,
        access::invitations_list,
        access::invitations_revoke,
        access::invitations_accept,
        access::shares_create
    ),
    components(schemas(
        HealthResponse,
        ReadyResponse,
        VersionResponse,
        ClientRequirements,
        ProblemDetails,
        FieldIssue,
        ErrorCode,
        files::Attachment,
        files::ScannedDocument,
        files::UploadInstructions,
        files::AttachmentCreated,
        files::ScannedDocumentCreated,
        files::DownloadLink,
        files::NewAttachment,
        files::NewScannedDocument,
        AccountHint,
        ClientParam,
        EmailStartRequest,
        EmailStartResponse,
        EmailVerifyRequest,
        IdentitySummary,
        LinkIdentityRequest,
        LinkResponse,
        LoginResponse,
        NonceResponse,
        ProviderLoginRequest,
        SessionDetail,
        SessionInfo,
        SessionIssued,
        SessionItem,
        UserSummary,
        WorkspaceSummary,
        WorkspaceCreateRequest,
        WorkspaceResponse,
        access::RoleResponse,
        access::RoleList,
        access::NewRole,
        access::CopyRole,
        access::RolePatch,
        access::AssignmentResponse,
        access::MemberResponse,
        access::MemberList,
        access::MemberPatch,
        access::NewAssignment,
        access::InvitationResponse,
        access::InvitationCreated,
        access::InvitationList,
        access::NewInvitation,
        access::AcceptInvitation,
        access::InvitationAccepted,
        access::NewShare
    )),
    tags(
        (name = "system", description = "Service status and version"),
        (name = "meta", description = "Definitions and client requirements"),
        (name = "events", description = "Live updates (SSE)"),
        (name = "files", description = "Attachments and scanned documents (direct upload to the object store)"),
        (name = "auth", description = "Sign-in with Apple, Google or an e-mail link"),
        (name = "me", description = "The signed-in user: session, sessions, sign-in methods"),
        (name = "workspaces", description = "Workspaces"),
        (name = "access", description = "Roles, members, assignments, invitations and sharing")
    )
)]
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
            let Some(spec) = OPERATIONS.iter().find(|s| s.operation_id == id) else {
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
    let mut value = serde_json::to_value(ApiDoc::openapi())?;
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
                OPERATIONS.iter().any(|s| s.operation_id == id),
                "{label}: {id} is missing from OPERATIONS"
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
            let spec = OPERATIONS.iter().find(|s| s.operation_id == id).unwrap();
            for code in spec.errors {
                assert!(
                    responses.contains_key(&code.status().as_u16().to_string()),
                    "{label}: response for {}",
                    code.as_str()
                );
            }
        }
        for spec in OPERATIONS {
            assert!(
                ops.iter()
                    .any(|(_, _, op)| op["operationId"] == spec.operation_id),
                "{} is in OPERATIONS but not in the document",
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
