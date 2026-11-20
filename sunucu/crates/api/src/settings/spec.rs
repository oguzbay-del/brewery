//! `OperationSpec` rows and the `OpenAPI` fragment of the settings module. The paths and schemas
//! of the six resources come from their field tables (`fields.rs`), so the document cannot
//! differ from what the code checks.

use serde_json::{Map, Value, json};

use super::{
    extras::{MAX_OBSERVATIONS, OBSERVATION, WIZARD},
    fields::{Field, RESOURCES, Resource, SETTING_KEYS},
};
use crate::{
    concurrency::Concurrency,
    error::ErrorCode::{
        self, CalculationFailed, Conflict, Forbidden, IdempotencyInProgress, IdempotencyMismatch,
        Internal, MalformedRequest, NotFound, PreconditionRequired, RateLimited, Unauthenticated,
        Unavailable, UnsupportedMediaType, UpgradeRequired, ValidationFailed, VersionMismatch,
    },
    openapi::OperationSpec,
};

const READ: &[ErrorCode] = &[
    MalformedRequest,
    Unauthenticated,
    Forbidden,
    NotFound,
    ValidationFailed,
    RateLimited,
    UpgradeRequired,
    Internal,
];
const CREATE: &[ErrorCode] = &[
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
const UPDATE: &[ErrorCode] = &[
    MalformedRequest,
    Unauthenticated,
    Forbidden,
    NotFound,
    Conflict,
    VersionMismatch,
    UnsupportedMediaType,
    ValidationFailed,
    RateLimited,
    UpgradeRequired,
    Internal,
];
const CALIBRATE: &[ErrorCode] = &[
    MalformedRequest,
    Unauthenticated,
    Forbidden,
    NotFound,
    UnsupportedMediaType,
    ValidationFailed,
    CalculationFailed,
    RateLimited,
    UpgradeRequired,
    Internal,
    Unavailable,
];

const fn row(
    operation_id: &'static str,
    permission: &'static str,
    concurrency: Option<Concurrency>,
    errors: &'static [ErrorCode],
) -> OperationSpec {
    OperationSpec {
        operation_id,
        public: false,
        permission: Some(permission),
        concurrency,
        errors,
    }
}

/// Seven rows of a resource: list, create, get, update, delete, history, restore.
macro_rules! crud {
    ($($n:literal),*) => {
        &[
            $(
                row(concat!($n, "_list"), "recipe.view", None, READ),
                row(concat!($n, "_create"), "settings.equipment", Some(Concurrency::Append), CREATE),
                row(concat!($n, "_get"), "recipe.view", None, READ),
                row(concat!($n, "_update"), "settings.equipment", Some(Concurrency::FieldLww), UPDATE),
                row(concat!($n, "_delete"), "settings.equipment", Some(Concurrency::FieldLww), UPDATE),
                row(concat!($n, "_history"), "recipe.view", None, READ),
                row(concat!($n, "_restore"), "settings.equipment", Some(Concurrency::FieldLww), UPDATE),
            )*
        ]
    };
}

const CRUD: &[OperationSpec] = crud!(
    "equipment",
    "mash_profiles",
    "fermentation_profiles",
    "carbonation_profiles",
    "water_profiles",
    "vessels"
);

const EXTRA: &[OperationSpec] = &[
    row(
        "equipment_wizard",
        "settings.equipment",
        Some(Concurrency::Append),
        CREATE,
    ),
    row("equipment_calibrate", "settings.equipment", None, CALIBRATE),
    row("settings_get", "none", None, READ),
    row(
        "settings_update",
        "workspace.manage",
        Some(Concurrency::FieldLww),
        UPDATE,
    ),
    row("settings_history", "none", None, READ),
    row(
        "settings_restore",
        "workspace.manage",
        Some(Concurrency::FieldLww),
        UPDATE,
    ),
    row("user_settings_get", "none", None, READ),
    row(
        "user_settings_update",
        "none",
        Some(Concurrency::FieldLww),
        UPDATE,
    ),
    row("user_settings_history", "none", None, READ),
    row(
        "user_settings_restore",
        "none",
        Some(Concurrency::FieldLww),
        UPDATE,
    ),
];

/// Every row of the module (the macro and the constants above build it at compile time).
pub static OPERATIONS: &[OperationSpec] = &{
    let mut all = [CRUD[0]; CRUD.len() + EXTRA.len()];
    let mut i = 0;
    while i < CRUD.len() {
        all[i] = CRUD[i];
        i += 1;
    }
    let mut j = 0;
    while j < EXTRA.len() {
        all[CRUD.len() + j] = EXTRA[j];
        j += 1;
    }
    all
};

// --- document ---------------------------------------------------------------------------------

const TAG: &str = "settings";
const ID_EX: &str = "0192f1c07a3b7000800000000000000e";
const TS_EX: &str = "2026-11-20T10:00:00Z";

fn schema_ref(name: &str) -> Value {
    json!({"$ref": format!("#/components/schemas/{name}")})
}

fn props(fields: &[Field]) -> Map<String, Value> {
    fields
        .iter()
        .map(|f| (f.name.to_owned(), f.schema()))
        .collect()
}

fn example(fields: &[Field]) -> Map<String, Value> {
    fields
        .iter()
        .map(|f| (f.name.to_owned(), f.ex.value()))
        .collect()
}

fn path_param(name: &str, description: &str) -> Value {
    json!({"name": name, "in": "path", "required": true, "description": description,
           "schema": {"type": "string", "format": "uuid"}})
}

fn query_param(name: &str, schema: &Value, description: &str) -> Value {
    json!({"name": name, "in": "query", "required": false, "description": description,
           "schema": schema})
}

fn header_param(name: &str, description: &str, required: bool) -> Value {
    json!({"name": name, "in": "header", "required": required, "description": description,
           "schema": {"type": "string"}})
}

fn ws_param() -> Value {
    path_param("workspace_id", "Workspace")
}

fn json_content(schema: &Value, example: &Value) -> Value {
    json!({"application/json": {"schema": schema, "example": example}})
}

/// A response with a body and an `ETag`.
fn record_response(status: &str, description: &str, schema: &Value, example: &Value) -> Value {
    json!({status: {
        "description": description,
        "headers": {"ETag": {"description": "Strong entity tag: the record version",
                             "schema": {"type": "string"}}},
        "content": json_content(schema, example)
    }})
}

fn merge_patch_body(schema: &Value, example: &Value) -> Value {
    json!({"required": true, "content": {"application/merge-patch+json":
        {"schema": schema, "example": example}}})
}

fn json_body(schema: &Value, example: &Value) -> Value {
    json!({"required": true, "content": json_content(schema, example)})
}

fn operation(
    id: &str,
    summary: &str,
    params: &[Value],
    body: Option<Value>,
    responses: &Value,
) -> Value {
    let mut op = json!({
        "tags": [TAG], "operationId": id, "summary": summary,
        "parameters": params, "responses": responses
    });
    if let Some(b) = body {
        op["requestBody"] = b;
    }
    op
}

fn history_example() -> Value {
    json!({"items": [{
        "id": ID_EX, "record_version": 3, "previous_value": {"efficiency_pct": 72.0},
        "created_at": TS_EX, "created_by": ID_EX
    }], "next_cursor": null})
}

fn merge(target: &mut Map<String, Value>, more: Value) {
    if let Value::Object(m) = more {
        target.extend(m);
    }
}

fn resource_schemas(r: &Resource, schemas: &mut Map<String, Value>) {
    let mut record = props(r.fields);
    let mut create = props(r.fields);
    let mut record_ex = example(r.fields);
    record_ex.insert("id".to_owned(), json!(ID_EX));
    record.insert("id".to_owned(), json!({"type": "string", "format": "uuid"}));
    if let Some(steps) = &r.steps {
        let step_ref = json!({"type": "array", "items": schema_ref(steps.schema)});
        record.insert("steps".to_owned(), step_ref.clone());
        create.insert("steps".to_owned(), step_ref);
        record_ex.insert("steps".to_owned(), json!([example(steps.fields)]));
        schemas.insert(
            steps.schema.to_owned(),
            json!({
                "type": "object", "properties": props(steps.fields),
                "required": steps.fields.iter().filter(|f| f.required).map(|f| f.name).collect::<Vec<_>>()
            }),
        );
    }
    record.insert(
        "version".to_owned(),
        json!({"type": "integer", "format": "int64"}),
    );
    for k in ["created_at", "updated_at"] {
        record.insert(
            k.to_owned(),
            json!({"type": "string", "format": "date-time"}),
        );
    }
    record.insert(
        "deleted_at".to_owned(),
        json!({"type": ["string", "null"], "format": "date-time"}),
    );
    record_ex.insert("version".to_owned(), json!(1));
    record_ex.insert("created_at".to_owned(), json!(TS_EX));
    record_ex.insert("updated_at".to_owned(), json!(TS_EX));
    record_ex.insert("deleted_at".to_owned(), Value::Null);
    let required: Vec<&str> = r
        .fields
        .iter()
        .filter(|f| f.required)
        .map(|f| f.name)
        .collect();
    let name = r.schema;
    schemas.insert(
        name.to_owned(),
        json!({"type": "object", "description": r.about, "properties": record,
               "required": ["id", "version", "created_at", "updated_at"], "example": record_ex}),
    );
    schemas.insert(
        format!("{name}Create"),
        json!({"type": "object", "properties": create, "required": required}),
    );
    let mut patch = create.clone();
    for v in patch.values_mut() {
        if let Some(o) = v.as_object_mut() {
            // A merge patch can send `null` for any field it may clear; plain fields stay as is.
            o.remove("minLength");
        }
    }
    schemas.insert(
        format!("{name}Patch"),
        json!({"type": "object", "description":
            "JSON Merge Patch: only the sent fields change; the last writer wins for each field",
            "properties": patch}),
    );
    schemas.insert(
        format!("{name}List"),
        json!({"type": "object", "properties": {
            "items": {"type": "array", "items": schema_ref(name)},
            "next_cursor": {"type": ["string", "null"]}
        }, "required": ["items", "next_cursor"]}),
    );
}

fn resource_paths(r: &Resource, paths: &mut Map<String, Value>) {
    let name = r.name;
    let base = format!("/v1/workspaces/{{workspace_id}}/{}", r.path);
    let one = format!("{base}/{{id}}");
    let record_ex = &r.schema;
    let rec_schema = schema_ref(record_ex);
    let rec_example = {
        let mut e = example(r.fields);
        e.insert("id".to_owned(), json!(ID_EX));
        if let Some(steps) = &r.steps {
            e.insert("steps".to_owned(), json!([example(steps.fields)]));
        }
        e.insert("version".to_owned(), json!(1));
        e.insert("created_at".to_owned(), json!(TS_EX));
        e.insert("updated_at".to_owned(), json!(TS_EX));
        e.insert("deleted_at".to_owned(), Value::Null);
        Value::Object(e)
    };
    let create_example = Value::Object(example(r.fields));
    let list_example = json!({"items": [rec_example.clone()], "next_cursor": null});
    let id_param = path_param("id", "Record");
    let sort_schema = json!({"type": "string", "enum": r.sort.iter().flat_map(|s| [s.to_string(), format!("-{s}")]).collect::<Vec<_>>()});
    let list_params = [
        ws_param(),
        query_param(
            "limit",
            &json!({"type": "integer", "minimum": 1, "maximum": 200}),
            "Page size",
        ),
        query_param(
            "cursor",
            &json!({"type": "string"}),
            "Cursor of the previous page",
        ),
        query_param(
            "sort",
            &sort_schema,
            "Sort field; `-` prefix for descending",
        ),
        query_param(
            "updated_since",
            &json!({"type": "string", "format": "date-time"}),
            "Only records changed since this time, deleted ones included",
        ),
        query_param(
            "include_deleted",
            &json!({"type": "boolean"}),
            "Include deleted records",
        ),
    ];
    paths.insert(
        base.clone(),
        json!({
            "get": operation(&format!("{name}_list"), &format!("Lists records: {}", r.about), &list_params, None,
                &json!({"200": {"description": "A page of records",
                    "content": json_content(&schema_ref(&format!("{record_ex}List")), &list_example)}})),
            "post": operation(&format!("{name}_create"), &format!("Creates a record: {}", r.about),
                &[ws_param(), header_param("Idempotency-Key", "Client generated key (ADR-0008 rule 6)", true)],
                Some(json_body(&schema_ref(&format!("{record_ex}Create")), &create_example)),
                &record_response("201", "Record created", &rec_schema, &rec_example)),
        }),
    );
    let mut get_responses = Map::new();
    merge(
        &mut get_responses,
        record_response("200", "The record", &rec_schema, &rec_example),
    );
    get_responses.insert("304".to_owned(), json!({"description": "Not modified"}));
    paths.insert(
        one.clone(),
        json!({
            "get": operation(&format!("{name}_get"), "Reads one record",
                &[ws_param(), id_param.clone(), header_param("If-None-Match", "Version `ETag` the client holds", false)],
                None, &Value::Object(get_responses)),
            "patch": operation(&format!("{name}_update"),
                "Changes fields of a record (merge patch). Last writer wins for each field; `If-Match` is optional",
                &[ws_param(), id_param.clone(), header_param("If-Match", "Version `ETag` of the record (optional)", false)],
                Some(merge_patch_body(&schema_ref(&format!("{record_ex}Patch")), &json!({r.fields.iter().find(|f| !f.required).map_or("name", |f| f.name): r.fields.iter().find(|f| !f.required).map_or(Value::Null, |f| f.ex.value())}))),
                &record_response("200", "The updated record", &rec_schema, &rec_example)),
            "delete": operation(&format!("{name}_delete"), "Deletes a record (soft delete; the history can restore it)",
                &[ws_param(), id_param.clone(), header_param("If-Match", "Version `ETag` of the record (optional)", false)],
                None, &json!({"204": {"description": "Deleted"}})),
        }),
    );
    paths.insert(
        format!("{one}/history"),
        json!({"get": operation(&format!("{name}_history"),
            "Change history of a record, newest first. `previous_value` holds the replaced values of the changed fields",
            &[ws_param(), id_param.clone(),
              query_param("limit", &json!({"type": "integer", "minimum": 1, "maximum": 200}), "Page size"),
              query_param("cursor", &json!({"type": "string"}), "Cursor of the previous page")],
            None,
            &json!({"200": {"description": "A page of history rows",
                "content": json_content(&schema_ref("ChangeHistoryList"), &history_example())}}))}),
    );
    paths.insert(
        format!("{one}/history/{{history_id}}/restore"),
        json!({"post": operation(&format!("{name}_restore"),
            "Writes the values of a history row back (a new change with its own history row)",
            &[ws_param(), id_param, path_param("history_id", "History row")],
            None, &record_response("200", "The restored record", &rec_schema, &rec_example))}),
    );
}

fn settings_paths(paths: &mut Map<String, Value>, prefix: &str, id_prefix: &str, what: &str) {
    let base = format!("/v1/workspaces/{{workspace_id}}/{prefix}");
    let ex = json!({"settings": {"unit_system": "us", "temperature_unit": "fahrenheit"},
                    "version": 2, "updated_at": TS_EX});
    let get_ok = {
        let mut m = Map::new();
        merge(
            &mut m,
            record_response("200", "The settings", &schema_ref("Settings"), &ex),
        );
        m.insert("304".to_owned(), json!({"description": "Not modified"}));
        Value::Object(m)
    };
    paths.insert(
        base.clone(),
        json!({
            "get": operation(&format!("{id_prefix}_get"), &format!("Reads the {what}"),
                &[ws_param(), header_param("If-None-Match", "Version `ETag` the client holds", false)],
                None, &get_ok),
            "patch": operation(&format!("{id_prefix}_update"),
                &format!("Changes keys of the {what} (merge patch; `null` removes a key). Last writer wins for each key"),
                &[ws_param(), header_param("If-Match", "Version `ETag` (optional)", false)],
                Some(merge_patch_body(&schema_ref("SettingsValues"), &json!({"unit_system": "us"}))),
                &record_response("200", "The updated settings", &schema_ref("Settings"), &ex)),
        }),
    );
    paths.insert(
        format!("{base}/history"),
        json!({"get": operation(&format!("{id_prefix}_history"),
            &format!("Change history of the {what}, newest first"),
            &[ws_param(),
              query_param("limit", &json!({"type": "integer", "minimum": 1, "maximum": 200}), "Page size"),
              query_param("cursor", &json!({"type": "string"}), "Cursor of the previous page")],
            None,
            &json!({"200": {"description": "A page of history rows",
                "content": json_content(&schema_ref("ChangeHistoryList"), &history_example())}}))}),
    );
    paths.insert(
        format!("{base}/history/{{history_id}}/restore"),
        json!({"post": operation(&format!("{id_prefix}_restore"),
            &format!("Writes the keys of a history row back into the {what}"),
            &[ws_param(), path_param("history_id", "History row")], None,
            &record_response("200", "The restored settings", &schema_ref("Settings"), &ex))}),
    );
}

fn openapi() -> utoipa::openapi::OpenApi {
    let mut paths = Map::new();
    let mut schemas = Map::new();
    for r in &RESOURCES {
        resource_schemas(r, &mut schemas);
        resource_paths(r, &mut paths);
    }
    settings_paths(&mut paths, "settings", "settings", "workspace settings");
    settings_paths(
        &mut paths,
        "me/settings",
        "user_settings",
        "user settings of the caller",
    );

    let wizard_ex = Value::Object(example(WIZARD));
    let equipment_ex = {
        let mut e = example(RESOURCES[0].fields);
        e.insert("id".to_owned(), json!(ID_EX));
        e.insert("version".to_owned(), json!(1));
        e.insert("created_at".to_owned(), json!(TS_EX));
        e.insert("updated_at".to_owned(), json!(TS_EX));
        e.insert("deleted_at".to_owned(), Value::Null);
        Value::Object(e)
    };
    paths.insert(
        "/v1/workspaces/{workspace_id}/equipment/wizard".to_owned(),
        json!({"post": operation("equipment_wizard",
            "Creates an equipment record from the four answers of the equipment wizard",
            &[ws_param(), header_param("Idempotency-Key", "Client generated key (ADR-0008 rule 6)", true)],
            Some(json_body(&schema_ref("EquipmentWizardRequest"), &wizard_ex)),
            &record_response("201", "Equipment created", &schema_ref("Equipment"), &equipment_ex))}),
    );
    let observation_ex = Value::Object(example(OBSERVATION));
    let calibration_ex = json!({
        "equipment_id": ID_EX, "current_efficiency_pct": 72.0,
        "observations": [{"brewhouse_yield_pct": 74.1, "source": "engine"}],
        "proposal": {"efficiency_pct": 74.0, "based_on": 3, "spread_pct": 1.2},
        "reason": null, "engine_version": "1.0.0"
    });
    paths.insert(
        "/v1/workspaces/{workspace_id}/equipment/{id}/calibration-proposal".to_owned(),
        json!({"post": operation("equipment_calibrate",
            "Proposes a new efficiency from measured results of past batches (the engine calculates the yields). Changes nothing",
            &[ws_param(), path_param("id", "Equipment")],
            Some(json_body(&schema_ref("CalibrationRequest"), &json!({"observations": [observation_ex]}))),
            &json!({"200": {"description": "The proposal",
                "content": json_content(&schema_ref("CalibrationResponse"), &calibration_ex)}}))}),
    );

    schemas.insert(
        "ChangeHistoryEntry".to_owned(),
        json!({"type": "object", "properties": {
            "id": {"type": "string", "format": "uuid"},
            "record_version": {"type": "integer", "format": "int64", "description": "Version the change produced"},
            "previous_value": {"type": "object", "description": "Replaced values of the changed fields"},
            "created_at": {"type": "string", "format": "date-time"},
            "created_by": {"type": ["string", "null"], "format": "uuid"}
        }, "required": ["id", "record_version", "previous_value", "created_at"]}),
    );
    schemas.insert(
        "ChangeHistoryList".to_owned(),
        json!({"type": "object", "properties": {
            "items": {"type": "array", "items": schema_ref("ChangeHistoryEntry")},
            "next_cursor": {"type": ["string", "null"]}
        }, "required": ["items", "next_cursor"]}),
    );
    schemas.insert(
        "SettingsValues".to_owned(),
        json!({"type": "object", "description":
            "Unit and calculation option keys. The option names follow `RecipeOptions` of the engine contract",
            "properties": props(SETTING_KEYS)}),
    );
    schemas.insert(
        "Settings".to_owned(),
        json!({"type": "object", "properties": {
            "settings": schema_ref("SettingsValues"),
            "version": {"type": "integer", "format": "int64", "description": "0 before the first write of user settings"},
            "updated_at": {"type": ["string", "null"], "format": "date-time"}
        }, "required": ["settings", "version", "updated_at"]}),
    );
    schemas.insert(
        "EquipmentWizardRequest".to_owned(),
        json!({"type": "object", "properties": props(WIZARD),
               "required": WIZARD.iter().filter(|f| f.required).map(|f| f.name).collect::<Vec<_>>()}),
    );
    schemas.insert(
        "CalibrationObservation".to_owned(),
        json!({"type": "object", "properties": props(OBSERVATION),
               "required": OBSERVATION.iter().filter(|f| f.required).map(|f| f.name).collect::<Vec<_>>()}),
    );
    schemas.insert(
        "CalibrationRequest".to_owned(),
        json!({"type": "object", "properties": {"observations": {
            "type": "array", "minItems": 1, "maxItems": MAX_OBSERVATIONS,
            "items": schema_ref("CalibrationObservation")}}, "required": ["observations"]}),
    );
    schemas.insert(
        "CalibrationResponse".to_owned(),
        json!({"type": "object", "properties": {
            "equipment_id": {"type": "string", "format": "uuid"},
            "current_efficiency_pct": {"type": "number", "format": "double"},
            "observations": {"type": "array", "items": {"type": "object", "properties": {
                "brewhouse_yield_pct": {"type": "number", "format": "double"},
                "source": {"type": "string", "enum": ["engine", "redis", "database"]}}}},
            "proposal": {"type": ["object", "null"], "properties": {
                "efficiency_pct": {"type": "number", "format": "double"},
                "based_on": {"type": "integer"},
                "spread_pct": {"type": "number", "format": "double"}}},
            "reason": {"type": ["string", "null"], "enum": ["too_few_observations", "inconsistent_observations", null]},
            "engine_version": {"type": "string"}
        }, "required": ["equipment_id", "current_efficiency_pct", "observations", "proposal", "reason", "engine_version"]}),
    );

    let doc = json!({
        "openapi": "3.1.0",
        "info": {"title": "settings", "version": "0"},
        "paths": paths,
        "components": {"schemas": schemas},
        "tags": [{"name": TAG, "description":
            "Equipment, profiles, vessels, workspace and user settings (last writer wins for each field, with a change history)"}]
    });
    serde_json::from_value(doc).unwrap_or_else(|e| {
        tracing::error!(error = %e, "settings OpenAPI fragment is invalid");
        utoipa::openapi::OpenApi::default()
    })
}

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "settings",
    routes: super::routes,
    operations: OPERATIONS,
    openapi,
};
