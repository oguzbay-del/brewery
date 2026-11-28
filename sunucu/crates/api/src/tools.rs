//! Tools API (T3.2): thin endpoints on the engine for the tools screen and the water page.
//!
//! The API does not calculate. Each handler maps the JSON body to the engine request, calls the
//! engine through [`crate::engine::EngineClient::calculate`] and maps the answer back to JSON.
//! The engine does the maths and the final range checks; the API checks only the field limits
//! of `common::catalog` (the same limits as `/v1/meta/fields`) and the shape of the body.
//!
//! Style: one path per tool (`POST /v1/workspaces/{ws}/tools/<tool>`), not one path with the
//! tool name as a path parameter. Each tool has its own request and response fields, so a
//! separate operation gives each one its own `OpenAPI` row, example and test. A path parameter
//! would give one operation with an untyped body.
//!
//! Permission: any member of the workspace (no extra permission). A tool call writes nothing for
//! the user, so it has no idempotency key. The engine result is cached; the `calc_results` row
//! is not tied to a record.
//!
//! The recipe calculation (`recipe_version_calculate`) does not return the water and pH results,
//! so the water page uses the endpoints of this module without a recipe.

#![allow(clippy::too_many_lines)]

use std::cell::RefCell;

use axum::{Json, Router, extract::State, routing::post};
use prost::Message;
use serde_json::{Map, Value, json};
use tonic::{Status, transport::Channel};

use crate::{
    AppState,
    context::RequestContext,
    engine::{CalcScope, proto},
    error::{ApiError, ApiJson, ApiResult, FieldIssue},
    library::workspace,
};

// --- reading the JSON body -----------------------------------------------------------------

/// Reads fields of a JSON object and collects field errors (one pass reports all of them).
struct Reader<'a> {
    value: &'a Value,
    path: String,
    issues: &'a RefCell<Vec<FieldIssue>>,
}

/// Longest list in a tool body (source waters, salts, grains).
const MAX_LINES: usize = 50;

impl<'a> Reader<'a> {
    fn root(value: &'a Value, issues: &'a RefCell<Vec<FieldIssue>>) -> Self {
        if !value.is_object() {
            issues
                .borrow_mut()
                .push(FieldIssue::new("", "invalid_type"));
        }
        Self {
            value,
            path: String::new(),
            issues,
        }
    }

    fn at(&self, key: &str) -> String {
        format!("{}/{key}", self.path)
    }

    fn fail(&self, key: &str, code: &str) {
        self.issues
            .borrow_mut()
            .push(FieldIssue::new(self.at(key), code));
    }

    fn missing(&self, key: &str) -> bool {
        self.value.get(key).is_none_or(Value::is_null)
    }

    /// Optional number. `limit` is a `common::catalog` field key.
    fn opt(&self, key: &str, limit: Option<&str>) -> Option<f64> {
        let raw = self.value.get(key).filter(|v| !v.is_null())?;
        let Some(n) = raw.as_f64() else {
            self.fail(key, "invalid_type");
            return None;
        };
        if let Some(limit) = limit
            && let Err(e) = common::catalog::validate(limit, n, None)
        {
            let mut issue = FieldIssue::new(self.at(key), e.code());
            if let common::catalog::FieldError::OutOfRange { min, max } = e {
                issue = issue.with_params(json!({"min": min, "max": max}));
            }
            self.issues.borrow_mut().push(issue);
        }
        Some(n)
    }

    /// Required number (a missing one is the error `required`; the value is then 0).
    fn num(&self, key: &str, limit: Option<&str>) -> f64 {
        if self.missing(key) {
            self.fail(key, "required");
            return 0.0;
        }
        self.opt(key, limit).unwrap_or(0.0)
    }

    fn flag(&self, key: &str) -> bool {
        match self.value.get(key) {
            None | Some(Value::Null) => false,
            Some(Value::Bool(b)) => *b,
            Some(_) => {
                self.fail(key, "invalid_type");
                false
            }
        }
    }

    /// An enum value by its lower-case name. `parse` maps the name to the proto number.
    fn choice(&self, key: &str, parse: fn(&str) -> Option<i32>, required: bool) -> i32 {
        match self.value.get(key) {
            None | Some(Value::Null) => {
                if required {
                    self.fail(key, "required");
                }
                0
            }
            Some(Value::String(s)) => parse(s).unwrap_or_else(|| {
                self.fail(key, "invalid_choice");
                0
            }),
            Some(_) => {
                self.fail(key, "invalid_type");
                0
            }
        }
    }

    /// A child object. A missing required one gives one error.
    fn object(&self, key: &str, required: bool) -> Option<Reader<'a>> {
        match self.value.get(key) {
            Some(v) if v.is_object() => Some(Reader {
                value: v,
                path: self.at(key),
                issues: self.issues,
            }),
            None | Some(Value::Null) => {
                if required {
                    self.fail(key, "required");
                }
                None
            }
            Some(_) => {
                self.fail(key, "invalid_type");
                None
            }
        }
    }

    fn list(&self, key: &str) -> Vec<Reader<'a>> {
        match self.value.get(key) {
            None | Some(Value::Null) => Vec::new(),
            Some(Value::Array(items)) if items.len() <= MAX_LINES => items
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    if !v.is_object() {
                        self.fail(&format!("{key}/{i}"), "invalid_type");
                    }
                    Reader {
                        value: v,
                        path: format!("{}/{i}", self.at(key)),
                        issues: self.issues,
                    }
                })
                .collect(),
            Some(Value::Array(_)) => {
                self.fail(key, "too_many_items");
                Vec::new()
            }
            Some(_) => {
                self.fail(key, "invalid_type");
                Vec::new()
            }
        }
    }
}

/// Parses a body with `build`; any collected field error becomes `422 validation_failed`.
fn read<T>(body: &Value, build: impl FnOnce(&Reader<'_>) -> T) -> ApiResult<T> {
    let issues = RefCell::new(Vec::new());
    let out = build(&Reader::root(body, &issues));
    let issues = issues.into_inner();
    if issues.is_empty() {
        Ok(out)
    } else {
        Err(ApiError::validation(issues))
    }
}

/// Proto enum value from a lower-case name (`sg` is `PREFIX_SG`). Zero (`UNSPECIFIED`) is refused.
macro_rules! parse_enum {
    ($ty:ty, $prefix:literal) => {
        |s: &str| {
            if s.is_empty() || !s.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') {
                return None;
            }
            <$ty>::from_str_name(&format!("{}{}", $prefix, s.to_ascii_uppercase()))
                .filter(|e| (*e as i32) != 0)
                .map(|e| e as i32)
        }
    };
}

/// Lower-case name of a proto enum value (`null` for an unknown number).
macro_rules! enum_name {
    ($ty:ty, $prefix:literal, $n:expr) => {
        <$ty>::try_from($n)
            .ok()
            .and_then(|e| e.as_str_name().strip_prefix($prefix))
            .map_or(Value::Null, |s| json!(s.to_ascii_lowercase()))
    };
}

// --- running a tool --------------------------------------------------------------------------

/// Calls the engine through the cache and builds the answer. `out` maps the engine response to
/// the JSON fields; the stamp (`engine_version`, `source`) is added here.
async fn run<Req, Resp, F, Fut>(
    state: &AppState,
    ctx: &RequestContext,
    kind: &str,
    request: Req,
    call: F,
    out: impl FnOnce(&Resp) -> Value,
) -> ApiResult<Json<Value>>
where
    Req: Message,
    Resp: Message + Default,
    F: FnOnce(Channel, Req) -> Fut,
    Fut: Future<Output = Result<tonic::Response<Resp>, Status>>,
{
    let ws = workspace(ctx)?;
    let mut tx = ctx.begin(&state.db).await?;
    let scope = CalcScope::new(ws, Some(ctx.identity.user_id));
    let done = state
        .engine
        .calculate(&mut tx, &scope, kind, request, call)
        .await?;
    tx.commit().await?;
    let mut body = out(&done.output);
    if let Some(map) = body.as_object_mut() {
        map.insert("engine_version".to_owned(), json!(done.engine_version));
        map.insert(
            "source".to_owned(),
            json!(match done.source {
                crate::engine::Source::Redis => "redis",
                crate::engine::Source::Database => "database",
                crate::engine::Source::Engine => "engine",
            }),
        );
    }
    Ok(Json(body))
}

const SG: Option<&str> = Some("measurement.sg");
const OG: Option<&str> = Some("measurement.og_sg");
const FG: Option<&str> = Some("measurement.fg_sg");
const TEMP: Option<&str> = Some("measurement.temp_c");
const VOL: Option<&str> = Some("tool.volume_l");
const MASS: Option<&str> = Some("tool.mass_kg");
const STEP: Option<&str> = Some("mash.step_temp_c");
const CO2: Option<&str> = Some("target.co2_vol");
const PH: Option<&str> = Some("measurement.ph");
const TARGET_PH: Option<&str> = Some("target.mash_ph");
const THICK: Option<&str> = Some("mash.thickness_l_per_kg");

// --- gravity tools -----------------------------------------------------------------------------

const GRAVITY_UNIT: fn(&str) -> Option<i32> = parse_enum!(proto::GravityUnit, "GRAVITY_UNIT_");

/// Converts a gravity reading between SG, degrees Plato and degrees Brix.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/tools/gravity-convert",
    tag = "tools",
    operation_id = "tool_gravity_convert",
    params(("workspace_id" = uuid::Uuid, Path, description = "Workspace")),
    request_body(content = Object, example = json!({"value": 1.050, "from": "sg", "to": "plato"})),
    responses((status = 200, description = "The converted value", body = Object,
        example = json!({"value": 12.39, "engine_version": "0.1.0", "source": "engine"})))
)]
pub async fn tool_gravity_convert(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<Value>,
) -> ApiResult<Json<Value>> {
    let request = read(&body, |r| {
        let from = r.choice("from", GRAVITY_UNIT, true);
        let value = r.num(
            "value",
            (from == proto::GravityUnit::Sg as i32).then_some("measurement.sg"),
        );
        proto::ConvertGravityRequest {
            value,
            from,
            to: r.choice("to", GRAVITY_UNIT, true),
        }
    })?;
    run(
        &state,
        &ctx,
        "tool_gravity_convert",
        request,
        |ch, r| async move {
            proto::tool_service_client::ToolServiceClient::new(ch)
                .convert_gravity(r)
                .await
        },
        |o| json!({"value": o.value}),
    )
    .await
}

/// Corrects a hydrometer reading for the sample temperature.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/tools/hydrometer-correct",
    tag = "tools",
    operation_id = "tool_hydrometer_correct",
    params(("workspace_id" = uuid::Uuid, Path, description = "Workspace")),
    request_body(content = Object,
        example = json!({"sg": 1.050, "sample_temp_c": 30.0, "calibration_temp_c": 20.0})),
    responses((status = 200, description = "The corrected gravity", body = Object,
        example = json!({"corrected_sg": 1.0526, "engine_version": "0.1.0", "source": "engine"})))
)]
pub async fn tool_hydrometer_correct(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<Value>,
) -> ApiResult<Json<Value>> {
    let request = read(&body, |r| proto::CorrectHydrometerRequest {
        sg: r.num("sg", SG),
        sample_temp_c: r.num("sample_temp_c", TEMP),
        calibration_temp_c: r.num("calibration_temp_c", TEMP),
    })?;
    run(
        &state,
        &ctx,
        "tool_hydrometer_correct",
        request,
        |ch, r| async move {
            proto::tool_service_client::ToolServiceClient::new(ch)
                .correct_hydrometer(r)
                .await
        },
        |o| json!({"corrected_sg": o.corrected_sg}),
    )
    .await
}

/// Refractometer reading of unfermented wort to original gravity.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/tools/refractometer-unfermented",
    tag = "tools",
    operation_id = "tool_refractometer_unfermented",
    params(("workspace_id" = uuid::Uuid, Path, description = "Workspace")),
    request_body(content = Object,
        example = json!({"brix": 12.0, "zero_brix": 0.0, "wort_correction_factor": 1.04})),
    responses((status = 200, description = "The original gravity", body = Object,
        example = json!({"og_sg": 1.0464, "engine_version": "0.1.0", "source": "engine"})))
)]
pub async fn tool_refractometer_unfermented(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<Value>,
) -> ApiResult<Json<Value>> {
    let request = read(&body, |r| proto::RefractometerUnfermentedRequest {
        brix: r.num("brix", None),
        zero_brix: r.opt("zero_brix", None),
        wort_correction_factor: r.opt("wort_correction_factor", None),
    })?;
    run(
        &state,
        &ctx,
        "tool_refractometer_unfermented",
        request,
        |ch, r| async move {
            proto::tool_service_client::ToolServiceClient::new(ch)
                .refractometer_unfermented(r)
                .await
        },
        |o| json!({"og_sg": o.og_sg}),
    )
    .await
}

/// Refractometer reading of fermenting wort (with a known OG) to the current gravity.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/tools/refractometer-fermenting",
    tag = "tools",
    operation_id = "tool_refractometer_fermenting",
    params(("workspace_id" = uuid::Uuid, Path, description = "Workspace")),
    request_body(content = Object, example = json!({"og_sg": 1.060, "brix": 8.0})),
    responses((status = 200, description = "The current gravity", body = Object,
        example = json!({"fg_sg": 1.012, "engine_version": "0.1.0", "source": "engine"})))
)]
pub async fn tool_refractometer_fermenting(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<Value>,
) -> ApiResult<Json<Value>> {
    let request = read(&body, |r| proto::RefractometerFermentingRequest {
        og_sg: r.num("og_sg", OG),
        brix: r.num("brix", None),
        zero_brix: r.opt("zero_brix", None),
    })?;
    run(
        &state,
        &ctx,
        "tool_refractometer_fermenting",
        request,
        |ch, r| async move {
            proto::tool_service_client::ToolServiceClient::new(ch)
                .refractometer_fermenting(r)
                .await
        },
        |o| json!({"fg_sg": o.fg_sg}),
    )
    .await
}

/// Alcohol by volume and by weight from OG and FG.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/tools/alcohol",
    tag = "tools",
    operation_id = "tool_alcohol",
    params(("workspace_id" = uuid::Uuid, Path, description = "Workspace")),
    request_body(content = Object, example = json!({"og_sg": 1.050, "fg_sg": 1.010})),
    responses((status = 200, description = "ABV and ABW in percent", body = Object,
        example = json!({"abv_pct": 5.25, "abw_pct": 4.1, "engine_version": "0.1.0",
            "source": "engine"})))
)]
pub async fn tool_alcohol(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<Value>,
) -> ApiResult<Json<Value>> {
    let request = read(&body, |r| proto::CalculateAlcoholRequest {
        og_sg: r.num("og_sg", OG),
        fg_sg: r.num("fg_sg", FG),
    })?;
    run(
        &state,
        &ctx,
        "tool_alcohol",
        request,
        |ch, r| async move {
            proto::tool_service_client::ToolServiceClient::new(ch)
                .calculate_alcohol(r)
                .await
        },
        |o| json!({"abv_pct": o.abv_pct, "abw_pct": o.abw_pct}),
    )
    .await
}

/// Dilutes a batch with water or with wort of another gravity.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/tools/dilution",
    tag = "tools",
    operation_id = "tool_dilution",
    params(("workspace_id" = uuid::Uuid, Path, description = "Workspace")),
    request_body(content = Object, example = json!({"sg": 1.070, "volume_l": 20.0,
        "added_volume_l": 5.0, "added_is_wort": false})),
    responses((status = 200, description = "The total volume and gravity", body = Object,
        example = json!({"total_volume_l": 25.0, "sg": 1.056, "engine_version": "0.1.0",
            "source": "engine"})))
)]
pub async fn tool_dilution(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<Value>,
) -> ApiResult<Json<Value>> {
    let request = read(&body, |r| proto::DiluteRequest {
        sg: r.num("sg", SG),
        volume_l: r.num("volume_l", VOL),
        added_volume_l: r.num("added_volume_l", VOL),
        added_is_wort: r.flag("added_is_wort"),
        added_sg: r.opt("added_sg", SG),
    })?;
    run(
        &state,
        &ctx,
        "tool_dilution",
        request,
        |ch, r| async move {
            proto::tool_service_client::ToolServiceClient::new(ch)
                .dilute(r)
                .await
        },
        |o| json!({"total_volume_l": o.total_volume_l, "sg": o.sg}),
    )
    .await
}

// --- carbonation ---------------------------------------------------------------------------------

/// Priming sugar by agent, keg pressure and the residual CO2.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/tools/carbonation",
    tag = "tools",
    operation_id = "tool_carbonation",
    params(("workspace_id" = uuid::Uuid, Path, description = "Workspace")),
    request_body(content = Object, example = json!({"type": "bottle", "co2_volumes": 2.4,
        "temp_c": 20.0, "volume_l": 20.0, "agent": "corn_sugar"})),
    responses((status = 200, description = "Priming amounts and the keg pressure", body = Object,
        example = json!({"residual_co2_volumes": 0.85, "priming_g": 120.0, "corn_sugar_g": 120.0,
            "dme_g": 160.0, "table_sugar_g": 110.0, "keg_pressure_bar": 0.9, "sugar_equiv": 100.0,
            "engine_version": "0.1.0", "source": "engine"})))
)]
pub async fn tool_carbonation(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<Value>,
) -> ApiResult<Json<Value>> {
    let request = read(&body, |r| proto::CarbonationRequest {
        r#type: r.choice(
            "type",
            parse_enum!(proto::CarbonationType, "CARBONATION_TYPE_"),
            true,
        ),
        co2_volumes: r.num("co2_volumes", CO2),
        temp_c: r.num("temp_c", TEMP),
        volume_l: r.num("volume_l", VOL),
        agent: r.choice(
            "agent",
            parse_enum!(proto::PrimingAgent, "PRIMING_AGENT_"),
            false,
        ),
        carb_rate_pct: r.opt("carb_rate_pct", None),
    })?;
    run(
        &state,
        &ctx,
        "tool_carbonation",
        request,
        |ch, r| async move {
            proto::yeast_service_client::YeastServiceClient::new(ch)
                .carbonation(r)
                .await
        },
        |o| {
            json!({
                "residual_co2_volumes": o.residual_co2_volumes, "priming_g": o.priming_g,
                "corn_sugar_g": o.corn_sugar_g, "dme_g": o.dme_g,
                "table_sugar_g": o.table_sugar_g, "keg_pressure_bar": o.keg_pressure_bar,
                "sugar_equiv": o.sugar_equiv,
            })
        },
    )
    .await
}

/// Sugar for natural carbonation of a keg, with pressures and warnings.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/tools/keg-priming",
    tag = "tools",
    operation_id = "tool_keg_priming",
    params(("workspace_id" = uuid::Uuid, Path, description = "Workspace")),
    request_body(content = Object, example = json!({"co2_vol": 2.4, "fermentation_max_c": 20.0,
        "conditioning_c": 20.0, "serving_c": 4.0, "beer_l": 19.0, "headspace_l": 1.0,
        "sugar": "dextrose", "purged": true, "mode": "full"})),
    responses((status = 200, description = "Sugar amounts, pressures and warnings", body = Object,
        example = json!({"sugar_g": 95.0, "bottle_equivalent_g": 90.0, "headspace_share_pct": 5.5,
            "serving_bar": 0.9, "peak_co2_vol": 2.5, "peak_bar": 1.0,
            "peak_total_gauge_bar": 1.0, "co2_topup_bar": null, "warnings": [],
            "engine_version": "0.1.0", "source": "engine"})))
)]
pub async fn tool_keg_priming(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<Value>,
) -> ApiResult<Json<Value>> {
    let request = read(&body, |r| proto::KegPrimingRequest {
        co2_vol: r.num("co2_vol", CO2),
        fermentation_max_c: r.num("fermentation_max_c", TEMP),
        conditioning_c: r.num("conditioning_c", TEMP),
        serving_c: r.num("serving_c", TEMP),
        beer_l: r.num("beer_l", VOL),
        headspace_l: r.num("headspace_l", VOL),
        sugar: r.choice(
            "sugar",
            parse_enum!(proto::ConditioningSugar, "CONDITIONING_SUGAR_"),
            true,
        ),
        purged: r.flag("purged"),
        mode: r.choice(
            "mode",
            parse_enum!(proto::KegPrimingMode, "KEG_PRIMING_MODE_"),
            true,
        ),
        partial_fraction: r.opt("partial_fraction", None),
        dry_matter_fraction: r.opt("dry_matter_fraction", None),
        keg_limit_bar: r.opt("keg_limit_bar", None),
    })?;
    run(
        &state,
        &ctx,
        "tool_keg_priming",
        request,
        |ch, r| async move {
            proto::fermentation_service_client::FermentationServiceClient::new(ch)
                .keg_priming(r)
                .await
        },
        |o| {
            let warnings: Vec<Value> = o
                .warnings
                .iter()
                .map(|w| {
                    json!({
                        "code": enum_name!(proto::ConditioningWarningCode,
                            "CONDITIONING_WARNING_CODE_", w.code),
                        "point": enum_name!(proto::CarbonationPoint, "CARBONATION_POINT_", w.point),
                        "temp_c": w.temp_c, "co2_vol": w.co2_vol,
                        "peak_bar": w.peak_bar, "limit_bar": w.limit_bar,
                    })
                })
                .collect();
            json!({
                "sugar_g": o.sugar_g, "bottle_equivalent_g": o.bottle_equivalent_g,
                "headspace_share_pct": o.headspace_share_pct, "serving_bar": o.serving_bar,
                "peak_co2_vol": o.peak_co2_vol, "peak_bar": o.peak_bar,
                "peak_total_gauge_bar": o.peak_total_gauge_bar,
                "co2_topup_bar": o.co2_topup_bar, "warnings": warnings,
            })
        },
    )
    .await
}

// --- mash temperatures -------------------------------------------------------------------------

fn tun(r: &Reader<'_>) -> Option<proto::MashTun> {
    r.object("tun", false).map(|t| proto::MashTun {
        mass_kg: t.num("mass_kg", MASS),
        specific_heat_cal: t.num("specific_heat_cal", None),
        volume_l: t.num("volume_l", VOL),
    })
}

/// Strike water temperature for a mash step.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/tools/strike-temp",
    tag = "tools",
    operation_id = "tool_strike_temp",
    params(("workspace_id" = uuid::Uuid, Path, description = "Workspace")),
    request_body(content = Object, example = json!({"water_l": 15.0, "step_c": 66.0,
        "grain_kg": 5.0, "grain_c": 20.0,
        "tun": {"mass_kg": 4.0, "specific_heat_cal": 0.12, "volume_l": 30.0}})),
    responses((status = 200, description = "The strike temperature", body = Object,
        example = json!({"strike_c": 73.5, "engine_version": "0.1.0", "source": "engine"})))
)]
pub async fn tool_strike_temp(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<Value>,
) -> ApiResult<Json<Value>> {
    let request = read(&body, |r| proto::StrikeTempRequest {
        water_l: r.num("water_l", VOL),
        step_c: r.num("step_c", STEP),
        grain_kg: r.num("grain_kg", MASS),
        grain_c: r.num("grain_c", TEMP),
        tun: tun(r),
        tun_c: r.opt("tun_c", TEMP),
    })?;
    run(
        &state,
        &ctx,
        "tool_strike_temp",
        request,
        |ch, r| async move {
            proto::mash_service_client::MashServiceClient::new(ch)
                .strike_temp(r)
                .await
        },
        |o| json!({"strike_c": o.strike_c}),
    )
    .await
}

/// Temperature of the infusion water for a step.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/tools/infusion-temp",
    tag = "tools",
    operation_id = "tool_infusion_temp",
    params(("workspace_id" = uuid::Uuid, Path, description = "Workspace")),
    request_body(content = Object, example = json!({"added_l": 5.0, "target_c": 72.0,
        "mash_water_l": 15.0, "mash_c": 66.0, "grain_kg": 5.0,
        "tun": {"mass_kg": 4.0, "specific_heat_cal": 0.12, "volume_l": 30.0}})),
    responses((status = 200, description = "The infusion temperature", body = Object,
        example = json!({"infusion_c": 95.0, "above_boiling": false,
            "engine_version": "0.1.0", "source": "engine"})))
)]
pub async fn tool_infusion_temp(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<Value>,
) -> ApiResult<Json<Value>> {
    let request = read(&body, |r| proto::InfusionTempRequest {
        added_l: r.num("added_l", VOL),
        target_c: r.num("target_c", STEP),
        mash_water_l: r.num("mash_water_l", VOL),
        mash_c: r.num("mash_c", TEMP),
        grain_kg: r.num("grain_kg", MASS),
        tun: tun(r),
    })?;
    run(
        &state,
        &ctx,
        "tool_infusion_temp",
        request,
        |ch, r| async move {
            proto::mash_service_client::MashServiceClient::new(ch)
                .infusion_temp(r)
                .await
        },
        |o| json!({"infusion_c": o.infusion_c, "above_boiling": o.above_boiling}),
    )
    .await
}

/// Amount of infusion water for a step.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/tools/infusion-amount",
    tag = "tools",
    operation_id = "tool_infusion_amount",
    params(("workspace_id" = uuid::Uuid, Path, description = "Workspace")),
    request_body(content = Object, example = json!({"infusion_c": 95.0, "target_c": 72.0,
        "mash_water_l": 15.0, "mash_c": 66.0, "grain_kg": 5.0,
        "tun": {"mass_kg": 4.0, "specific_heat_cal": 0.12, "volume_l": 30.0}})),
    responses((status = 200, description = "The water to add", body = Object,
        example = json!({"added_l": 5.0, "engine_version": "0.1.0", "source": "engine"})))
)]
pub async fn tool_infusion_amount(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<Value>,
) -> ApiResult<Json<Value>> {
    let request = read(&body, |r| proto::InfusionAmountRequest {
        infusion_c: r.num("infusion_c", TEMP),
        target_c: r.num("target_c", STEP),
        mash_water_l: r.num("mash_water_l", VOL),
        mash_c: r.num("mash_c", TEMP),
        grain_kg: r.num("grain_kg", MASS),
        tun: tun(r),
    })?;
    run(
        &state,
        &ctx,
        "tool_infusion_amount",
        request,
        |ch, r| async move {
            proto::mash_service_client::MashServiceClient::new(ch)
                .infusion_amount(r)
                .await
        },
        |o| json!({"added_l": o.added_l}),
    )
    .await
}

// --- water ----------------------------------------------------------------------------------------

/// Mash pH model; the default is `mph3`, the engine default.
fn ph_model(r: &Reader<'_>) -> i32 {
    match r.choice("model", parse_enum!(proto::PhModel, "PH_MODEL_"), false) {
        0 => proto::PhModel::Mph3 as i32,
        model => model,
    }
}

const RECIPE_KIND: fn(&str) -> Option<i32> = parse_enum!(proto::RecipeKind, "RECIPE_KIND_");
const WATER_SALT: fn(&str) -> Option<i32> = parse_enum!(proto::WaterSalt, "WATER_SALT_");

fn ions(r: &Reader<'_>) -> proto::IonProfile {
    proto::IonProfile {
        calcium_ppm: r.num("calcium_ppm", Some("water.calcium_ppm")),
        magnesium_ppm: r.num("magnesium_ppm", Some("water.magnesium_ppm")),
        sodium_ppm: r.num("sodium_ppm", Some("water.sodium_ppm")),
        sulfate_ppm: r.num("sulfate_ppm", Some("water.sulfate_ppm")),
        chloride_ppm: r.num("chloride_ppm", Some("water.chloride_ppm")),
        bicarbonate_ppm: r.num("bicarbonate_ppm", Some("water.bicarbonate_ppm")),
    }
}

fn ions_of(r: &Reader<'_>, key: &str) -> Option<proto::IonProfile> {
    r.object(key, true).map(|o| ions(&o))
}

fn ions_json(p: Option<&proto::IonProfile>) -> Value {
    p.map_or(Value::Null, |p| {
        json!({
            "calcium_ppm": p.calcium_ppm, "magnesium_ppm": p.magnesium_ppm,
            "sodium_ppm": p.sodium_ppm, "sulfate_ppm": p.sulfate_ppm,
            "chloride_ppm": p.chloride_ppm, "bicarbonate_ppm": p.bicarbonate_ppm,
        })
    })
}

fn source_waters(r: &Reader<'_>) -> Vec<proto::SourceWater> {
    r.list("waters")
        .iter()
        .map(|w| proto::SourceWater {
            amount_l: w.num("amount_l", VOL),
            ions: ions_of(w, "ions"),
            ph: w.num("ph", Some("water.ph")),
        })
        .collect()
}

fn volumes(r: &Reader<'_>) -> Option<proto::WaterVolumes> {
    r.object("volumes", true).map(|v| proto::WaterVolumes {
        mash_water_l: v.opt("mash_water_l", VOL).unwrap_or(0.0),
        sparge_l: v.opt("sparge_l", VOL).unwrap_or(0.0),
        mash_available_l: v.opt("mash_available_l", VOL).unwrap_or(0.0),
        kettle_top_up_l: v.opt("kettle_top_up_l", VOL).unwrap_or(0.0),
        boil_l: v.opt("boil_l", VOL).unwrap_or(0.0),
        post_boil_l: v.opt("post_boil_l", VOL).unwrap_or(0.0),
        cool_loss_l: v.opt("cool_loss_l", VOL).unwrap_or(0.0),
        trub_loss_l: v.opt("trub_loss_l", VOL).unwrap_or(0.0),
        fermenter_top_up_l: v.opt("fermenter_top_up_l", VOL).unwrap_or(0.0),
        batch_l: v.opt("batch_l", VOL).unwrap_or(0.0),
        has_boil_ingredients: v.flag("has_boil_ingredients"),
    })
}

fn grains(r: &Reader<'_>) -> Vec<proto::MashGrain> {
    r.list("grains")
        .iter()
        .map(|g| proto::MashGrain {
            amount_kg: g.num("amount_kg", MASS),
            color_srm: g.num("color_srm", Some("target.color_srm")),
            kind: g.choice(
                "kind",
                parse_enum!(proto::FermentableKind, "FERMENTABLE_KIND_"),
                true,
            ),
            acid_malt: g.flag("acid_malt"),
            acid_pct: g.opt("acid_pct", None).unwrap_or(0.0),
        })
        .collect()
}

fn stats_json(s: &proto::ProfileStats) -> Value {
    json!({
        "alkalinity_ppm": s.alkalinity_ppm, "hardness_ppm": s.hardness_ppm,
        "residual_alkalinity_ppm": s.residual_alkalinity_ppm,
        "sulfate_chloride_ratio": s.sulfate_chloride_ratio,
        "balance": enum_name!(proto::SulfateChlorideBalance, "SULFATE_CHLORIDE_BALANCE_",
            s.balance),
        "color_min_srm": s.color_min_srm, "color_max_srm": s.color_max_srm,
    })
}

fn salts_json(list: &[proto::SaltAmount]) -> Vec<Value> {
    list.iter()
        .map(|s| {
            json!({
                "salt": enum_name!(proto::WaterSalt, "WATER_SALT_", s.salt),
                "amount_g": s.amount_g,
            })
        })
        .collect()
}

/// Water screen: ion profiles, mash figures and the mash pH, for one set of source waters and
/// salts. It needs no recipe.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/tools/water-calculate",
    tag = "tools",
    operation_id = "tool_water_calculate",
    params(("workspace_id" = uuid::Uuid, Path, description = "Workspace")),
    request_body(content = Object, example = json!({"kind": "all_grain",
        "waters": [{"amount_l": 25.0, "ph": 7.4, "ions": {"calcium_ppm": 50.0,
            "magnesium_ppm": 10.0, "sodium_ppm": 15.0, "sulfate_ppm": 60.0,
            "chloride_ppm": 20.0, "bicarbonate_ppm": 120.0}}],
        "salts": [{"salt": "gypsum", "amount_g": 3.0, "usage": "mash"}],
        "volumes": {"mash_water_l": 15.0, "sparge_l": 10.0, "mash_available_l": 14.0,
            "kettle_top_up_l": 0.0, "boil_l": 25.0, "post_boil_l": 22.0, "cool_loss_l": 0.0,
            "trub_loss_l": 1.0, "fermenter_top_up_l": 0.0, "batch_l": 21.0,
            "has_boil_ingredients": true},
        "mash_ph": {"grains": [{"amount_kg": 5.0, "color_srm": 3.0, "kind": "grain"}],
            "mash_thickness_l_per_kg": 3.0, "target_ph": 5.4}})),
    responses((status = 200, description = "Ion profiles, figures and the mash pH", body = Object,
        example = json!({"base": null, "water_ph": 7.4, "mash": null, "sparge": null,
            "mash_and_sparge": null, "boil": null, "beer": null, "boil_scale": 1.0,
            "mash_stats": null, "mash_ph": null, "warnings": [],
            "engine_version": "0.1.0", "source": "engine"})))
)]
pub async fn tool_water_calculate(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<Value>,
) -> ApiResult<Json<Value>> {
    let request = read(&body, |r| proto::CalculateWaterRequest {
        kind: r.choice("kind", RECIPE_KIND, true),
        waters: source_waters(r),
        salts: r
            .list("salts")
            .iter()
            .map(|s| proto::SaltAddition {
                salt: s.choice("salt", WATER_SALT, true),
                amount_g: s.num("amount_g", None),
                usage: s.choice("usage", parse_enum!(proto::MiscUse, "MISC_USE_"), true),
            })
            .collect(),
        ignore_salts: r.flag("ignore_salts"),
        volumes: volumes(r),
        model: ph_model(r),
        beersmith_compatible: r.flag("beersmith_compatible"),
        mash_ph: r.object("mash_ph", false).map(|m| proto::MashPhInput {
            grains: grains(&m),
            mash_thickness_l_per_kg: m.num("mash_thickness_l_per_kg", THICK),
            acids: m
                .list("acids")
                .iter()
                .map(|a| proto::LiquidAcid {
                    acid: a.choice("acid", parse_enum!(proto::AcidKind, "ACID_KIND_"), true),
                    volume_ml: a.num("volume_ml", None),
                    concentration_pct: a.num("concentration_pct", None),
                })
                .collect(),
            target_ph: m.num("target_ph", TARGET_PH),
        }),
    })?;
    run(
        &state,
        &ctx,
        "tool_water_calculate",
        request,
        |ch, r| async move {
            proto::water_service_client::WaterServiceClient::new(ch)
                .calculate_water(r)
                .await
        },
        |o| {
            let mut m = Map::new();
            m.insert("base".to_owned(), ions_json(o.base.as_ref()));
            m.insert("water_ph".to_owned(), json!(o.water_ph));
            m.insert("mash".to_owned(), ions_json(o.mash.as_ref()));
            m.insert("sparge".to_owned(), ions_json(o.sparge.as_ref()));
            m.insert(
                "mash_and_sparge".to_owned(),
                ions_json(o.mash_and_sparge.as_ref()),
            );
            m.insert("boil".to_owned(), ions_json(o.boil.as_ref()));
            m.insert("beer".to_owned(), ions_json(o.beer.as_ref()));
            m.insert("boil_scale".to_owned(), json!(o.boil_scale));
            m.insert(
                "mash_stats".to_owned(),
                o.mash_stats.as_ref().map_or(Value::Null, stats_json),
            );
            m.insert(
                "mash_ph".to_owned(),
                o.mash_ph.as_ref().map_or(Value::Null, |p| {
                    json!({
                        "grist_ph": p.grist_ph, "ra_slope": p.ra_slope,
                        "estimated_ph": p.estimated_ph, "adjusted_ph": p.adjusted_ph,
                        "grain_kg": p.grain_kg,
                    })
                }),
            );
            let warnings: Vec<Value> = o
                .warnings
                .iter()
                .map(|w| enum_name!(proto::WaterWarning, "WATER_WARNING_", *w))
                .collect();
            m.insert("warnings".to_owned(), json!(warnings));
            Value::Object(m)
        },
    )
    .await
}

/// Acid amount for a mash and a sparge target pH.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/tools/acid-adjust",
    tag = "tools",
    operation_id = "tool_acid_adjust",
    params(("workspace_id" = uuid::Uuid, Path, description = "Workspace")),
    request_body(content = Object, example = json!({"acid": "lactic", "concentration_pct": 88.0,
        "mash_ph": 5.8, "target_ph": 5.4, "sparge_target_ph": 6.0, "grain_kg": 5.0,
        "mash_thickness_l_per_kg": 3.0, "sparge_bicarbonate_ppm": 120.0,
        "sparge_water_ph": 7.4, "sparge_l": 10.0})),
    responses((status = 200, description = "Mash and sparge acid", body = Object,
        example = json!({"mash_amount": 4.0, "resulting_ph": null, "converged": null,
            "sparge_acid_ml": 1.0, "sparge_l": 10.0, "engine_version": "0.1.0",
            "source": "engine"})))
)]
pub async fn tool_acid_adjust(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<Value>,
) -> ApiResult<Json<Value>> {
    let request = read(&body, |r| {
        // The sparge volume is either `sparge_l` (tool) or `total_water_l` (recipe mode).
        let sparge_volume = match (r.opt("sparge_l", VOL), r.opt("total_water_l", VOL)) {
            (Some(l), None) => Some(proto::adjust_acid_request::SpargeVolume::SpargeL(l)),
            (None, Some(l)) => Some(proto::adjust_acid_request::SpargeVolume::TotalWaterL(l)),
            (None, None) => {
                r.fail("sparge_l", "required");
                None
            }
            (Some(_), Some(_)) => {
                r.fail("total_water_l", "conflicts_with_sparge_l");
                None
            }
        };
        proto::AdjustAcidRequest {
            model: ph_model(r),
            beersmith_compatible: r.flag("beersmith_compatible"),
            acid: r.choice("acid", parse_enum!(proto::AcidKind, "ACID_KIND_"), true),
            concentration_pct: r.num("concentration_pct", None),
            mash_ph: r.opt("mash_ph", PH).unwrap_or(0.0),
            target_ph: r.num("target_ph", TARGET_PH),
            sparge_target_ph: r.opt("sparge_target_ph", PH).unwrap_or(0.0),
            grain_kg: r.num("grain_kg", MASS),
            mash_thickness_l_per_kg: r.num("mash_thickness_l_per_kg", THICK),
            grains: grains(r),
            residual_alkalinity_ppm: r.opt("residual_alkalinity_ppm", None).unwrap_or(0.0),
            sparge_bicarbonate_ppm: r
                .opt("sparge_bicarbonate_ppm", Some("water.bicarbonate_ppm"))
                .unwrap_or(0.0),
            sparge_water_ph: r.opt("sparge_water_ph", Some("water.ph")).unwrap_or(0.0),
            sparge_volume,
        }
    })?;
    run(
        &state,
        &ctx,
        "tool_acid_adjust",
        request,
        |ch, r| async move {
            proto::water_service_client::WaterServiceClient::new(ch)
                .adjust_acid(r)
                .await
        },
        |o| {
            json!({
                "mash_amount": o.mash_amount, "resulting_ph": o.resulting_ph,
                "converged": o.converged, "sparge_acid_ml": o.sparge_acid_ml,
                "sparge_l": o.sparge_l,
            })
        },
    )
    .await
}

/// Best salt additions for a target water profile.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/tools/profile-match",
    tag = "tools",
    operation_id = "tool_profile_match",
    params(("workspace_id" = uuid::Uuid, Path, description = "Workspace")),
    request_body(content = Object, example = json!({"kind": "all_grain",
        "waters": [{"amount_l": 25.0, "ph": 7.0, "ions": {"calcium_ppm": 10.0,
            "magnesium_ppm": 2.0, "sodium_ppm": 5.0, "sulfate_ppm": 5.0,
            "chloride_ppm": 5.0, "bicarbonate_ppm": 20.0}}],
        "target": {"calcium_ppm": 100.0, "magnesium_ppm": 10.0, "sodium_ppm": 10.0,
            "sulfate_ppm": 150.0, "chloride_ppm": 60.0, "bicarbonate_ppm": 20.0},
        "volumes": {"mash_water_l": 15.0, "sparge_l": 10.0, "batch_l": 21.0},
        "excluded": ["chalk"], "combine_salts": false})),
    responses((status = 200, description = "Salt additions and the resulting profile",
        body = Object,
        example = json!({"mash": [{"salt": "gypsum", "amount_g": 3.0}], "sparge": [],
            "best_fit": null, "residual_ppm": 2.0, "engine_version": "0.1.0",
            "source": "engine"})))
)]
pub async fn tool_profile_match(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<Value>,
) -> ApiResult<Json<Value>> {
    let request = read(&body, |r| {
        let excluded = match r.value.get("excluded") {
            None | Some(Value::Null) => Vec::new(),
            Some(Value::Array(items)) => items
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    v.as_str().and_then(WATER_SALT).unwrap_or_else(|| {
                        r.fail(&format!("excluded/{i}"), "invalid_choice");
                        0
                    })
                })
                .collect(),
            Some(_) => {
                r.fail("excluded", "invalid_type");
                Vec::new()
            }
        };
        proto::MatchProfileRequest {
            kind: r.choice("kind", RECIPE_KIND, true),
            waters: source_waters(r),
            target: ions_of(r, "target"),
            volumes: volumes(r),
            excluded,
            combine_salts: r.flag("combine_salts"),
        }
    })?;
    run(
        &state,
        &ctx,
        "tool_profile_match",
        request,
        |ch, r| async move {
            proto::water_service_client::WaterServiceClient::new(ch)
                .match_profile(r)
                .await
        },
        |o| {
            json!({
                "mash": salts_json(&o.mash), "sparge": salts_json(&o.sparge),
                "best_fit": ions_json(o.best_fit.as_ref()), "residual_ppm": o.residual_ppm,
            })
        },
    )
    .await
}

// --- routes and registry entry ---------------------------------------------------------------

/// The tool routes, wrapped with the sign-in layer only (no idempotency key: no write).
pub fn routes(state: &AppState) -> Router<AppState> {
    const T: &str = "/v1/workspaces/{ws}/tools";
    crate::authenticated(
        state,
        Router::new()
            .route(&format!("{T}/gravity-convert"), post(tool_gravity_convert))
            .route(
                &format!("{T}/hydrometer-correct"),
                post(tool_hydrometer_correct),
            )
            .route(
                &format!("{T}/refractometer-unfermented"),
                post(tool_refractometer_unfermented),
            )
            .route(
                &format!("{T}/refractometer-fermenting"),
                post(tool_refractometer_fermenting),
            )
            .route(&format!("{T}/alcohol"), post(tool_alcohol))
            .route(&format!("{T}/dilution"), post(tool_dilution))
            .route(&format!("{T}/carbonation"), post(tool_carbonation))
            .route(&format!("{T}/keg-priming"), post(tool_keg_priming))
            .route(&format!("{T}/strike-temp"), post(tool_strike_temp))
            .route(&format!("{T}/infusion-temp"), post(tool_infusion_temp))
            .route(&format!("{T}/infusion-amount"), post(tool_infusion_amount))
            .route(&format!("{T}/water-calculate"), post(tool_water_calculate))
            .route(&format!("{T}/acid-adjust"), post(tool_acid_adjust))
            .route(&format!("{T}/profile-match"), post(tool_profile_match)),
    )
}

mod spec {
    use crate::error::ErrorCode::{
        self, CalculationFailed, Forbidden, Internal, MalformedRequest, NotFound, PayloadTooLarge,
        RateLimited, Unauthenticated, Unavailable, UnsupportedMediaType, UpgradeRequired,
        ValidationFailed,
    };
    use crate::openapi::{OperationSpec, file_op};

    const CALC: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        PayloadTooLarge,
        UnsupportedMediaType,
        ValidationFailed,
        CalculationFailed,
        RateLimited,
        UpgradeRequired,
        Internal,
        Unavailable,
    ];

    pub(super) const OPERATIONS: &[OperationSpec] = &[
        file_op("tool_gravity_convert", None, None, CALC),
        file_op("tool_hydrometer_correct", None, None, CALC),
        file_op("tool_refractometer_unfermented", None, None, CALC),
        file_op("tool_refractometer_fermenting", None, None, CALC),
        file_op("tool_alcohol", None, None, CALC),
        file_op("tool_dilution", None, None, CALC),
        file_op("tool_carbonation", None, None, CALC),
        file_op("tool_keg_priming", None, None, CALC),
        file_op("tool_strike_temp", None, None, CALC),
        file_op("tool_infusion_temp", None, None, CALC),
        file_op("tool_infusion_amount", None, None, CALC),
        file_op("tool_water_calculate", None, None, CALC),
        file_op("tool_acid_adjust", None, None, CALC),
        file_op("tool_profile_match", None, None, CALC),
    ];
}

/// `OpenAPI` fragment of this module.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(
        tool_gravity_convert,
        tool_hydrometer_correct,
        tool_refractometer_unfermented,
        tool_refractometer_fermenting,
        tool_alcohol,
        tool_dilution,
        tool_carbonation,
        tool_keg_priming,
        tool_strike_temp,
        tool_infusion_temp,
        tool_infusion_amount,
        tool_water_calculate,
        tool_acid_adjust,
        tool_profile_match
    ),
    tags((name = "tools", description = "Calculators and water tools on the engine (no recipe needed)"))
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "tools",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};
