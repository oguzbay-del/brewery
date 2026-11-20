//! Equipment wizard and calibration proposal (T2.20).
//!
//! - Wizard: four answers (method, batch volume, kettle volume, known efficiency) become one
//!   complete equipment record. The server fills the other fields with defaults.
//! - Calibration proposal: the caller sends measured results of past batches. The engine
//!   (`BreweryService.BrewhouseEfficiency`, through `EngineClient`) calculates the brew house
//!   yield of each one. The proposal is the mean of the last three yields when they agree. The
//!   proposal changes nothing; the client applies it with `PATCH` on the equipment.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::Response,
    routing::post,
};
use serde_json::{Map, Value, json};
use uuid::Uuid;

use super::{
    RESOURCES, WS, check_body, insert_record, not_found, object, record, require_write, ws_of,
};
use crate::{
    AppState,
    concurrency::record_response,
    context::RequestContext,
    engine::{CalcScope, Source, proto},
    error::{ApiError, ApiJson, ApiResult, FieldIssue},
    settings::fields::{Field, check_object, choice, flag, num, text},
};

/// Answers of the equipment wizard.
pub const WIZARD: &[Field] = &[
    text("name", "My brew house").len(1, 200),
    choice("method", &["all_grain", "biab", "extract"]).req(),
    num("batch_volume_l", 20.0).req().gt(0.0).le(10_000.0),
    num("kettle_volume_l", 30.0).req().gt(0.0).le(10_000.0),
    num("efficiency_pct", 72.0).ge(0.0).le(100.0),
];

/// One measured batch.
pub const OBSERVATION: &[Field] = &[
    num("wort_volume_l", 20.0).req().gt(0.0),
    num("wort_extract_p", 12.5).req().gt(0.0),
    num("malt_kg", 4.5).req().gt(0.0),
    flag("measured_hot", false),
];

/// Most batches one proposal request can carry.
pub const MAX_OBSERVATIONS: usize = 20;
/// The proposal uses the last batches; this many.
const WINDOW: usize = 3; // The mean below divides by this number.
/// The yields of the window agree when their spread is at most this many percentage points.
const MAX_SPREAD_PCT: f64 = 5.0;

// TODO: The defaults below are a first guess. Product owner to verify them (to be verified).
const BOIL_TIME_MIN: f64 = 60.0;
const BOIL_OFF_L_PER_H: f64 = 3.0;
const TRUB_LOSS_L: f64 = 1.0;

pub fn routes() -> (Router<AppState>, Router<AppState>) {
    let plain = Router::new().route(
        &format!("{WS}/equipment/{{id}}/calibration-proposal"),
        post(calibration_proposal),
    );
    let creates = Router::new().route(&format!("{WS}/equipment/wizard"), post(wizard));
    (plain, creates)
}

fn default_efficiency(method: &str) -> f64 {
    // TODO: Product owner to verify these values (to be verified).
    match method {
        "biab" => 75.0,
        "extract" => 100.0,
        _ => 72.0,
    }
}

async fn wizard(
    State(state): State<AppState>,
    ctx: RequestContext,
    ApiJson(body): ApiJson<Value>,
) -> ApiResult<Response> {
    require_write(&ctx)?;
    let ws = ws_of(&ctx)?;
    let answers = object(body)?;
    let mut issues = Vec::new();
    check_object(WIZARD, &answers, "", true, &[], &mut issues);
    if !issues.is_empty() {
        return Err(ApiError::validation(issues));
    }
    let num_of = |k: &str| answers.get(k).and_then(Value::as_f64).unwrap_or_default();
    let method = answers
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or("all_grain");
    let batch = num_of("batch_volume_l");
    let kettle = num_of("kettle_volume_l");
    let boil_volume = batch + TRUB_LOSS_L + BOIL_OFF_L_PER_H * BOIL_TIME_MIN / 60.0;
    if boil_volume > kettle {
        return Err(ApiError::validation(vec![
            FieldIssue::new("/kettle_volume_l", "too_small")
                .with_params(json!({"min": boil_volume})),
        ]));
    }
    let mut eq = Map::new();
    eq.insert(
        "name".to_owned(),
        answers
            .get("name")
            .cloned()
            .unwrap_or_else(|| json!("My brew house")),
    );
    eq.insert("batch_volume_l".to_owned(), json!(batch));
    eq.insert("boil_volume_l".to_owned(), json!(boil_volume));
    eq.insert("boil_time_min".to_owned(), json!(BOIL_TIME_MIN));
    eq.insert("boil_off_l_per_h".to_owned(), json!(BOIL_OFF_L_PER_H));
    eq.insert("trub_loss_l".to_owned(), json!(TRUB_LOSS_L));
    eq.insert(
        "efficiency_pct".to_owned(),
        answers
            .get("efficiency_pct")
            .cloned()
            .unwrap_or_else(|| json!(default_efficiency(method))),
    );
    if method == "biab" {
        eq.insert("mash_tun_volume_l".to_owned(), json!(kettle));
    }
    // The wizard record must pass the same checks as a record of `POST /equipment`.
    let r = &RESOURCES[0];
    check_body(r, &eq, true)?;
    let mut tx = ctx.begin(&state.db).await?;
    let id = insert_record(&mut tx, r, ws, ctx.identity.user_id, eq).await?;
    let (rec, version) = record(&mut tx, r, ws, id, false).await?;
    tx.commit().await?;
    Ok(record_response(StatusCode::CREATED, version, &rec))
}

const fn source_name(s: Source) -> &'static str {
    match s {
        Source::Redis => "redis",
        Source::Database => "database",
        Source::Engine => "engine",
    }
}

async fn calibration_proposal(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_, id)): Path<(Uuid, Uuid)>,
    ApiJson(body): ApiJson<Value>,
) -> ApiResult<Json<Value>> {
    require_write(&ctx)?;
    let ws = ws_of(&ctx)?;
    let body = object(body)?;
    let mut issues = Vec::new();
    for key in body.keys().filter(|k| *k != "observations") {
        issues.push(FieldIssue::new(format!("/{key}"), "unknown_field"));
    }
    let list = body.get("observations").and_then(Value::as_array);
    match list {
        Some(l) if l.is_empty() || l.len() > MAX_OBSERVATIONS => {
            issues.push(
                FieldIssue::new(
                    "/observations",
                    if l.is_empty() {
                        "too_few_items"
                    } else {
                        "too_many_items"
                    },
                )
                .with_params(json!({"min": 1, "max": MAX_OBSERVATIONS})),
            );
        }
        Some(l) => {
            for (i, item) in l.iter().enumerate() {
                match item.as_object() {
                    Some(m) => {
                        check_object(
                            OBSERVATION,
                            m,
                            &format!("/observations/{i}"),
                            true,
                            &[],
                            &mut issues,
                        );
                    }
                    None => issues.push(FieldIssue::new(
                        format!("/observations/{i}"),
                        "invalid_type",
                    )),
                }
            }
        }
        None => {
            let code = if body.contains_key("observations") {
                "invalid_type"
            } else {
                "required"
            };
            issues.push(FieldIssue::new("/observations", code));
        }
    }
    if !issues.is_empty() {
        return Err(ApiError::validation(issues));
    }
    let mut tx = ctx.begin(&state.db).await?;
    let (equipment, _) = record(&mut tx, &RESOURCES[0], ws, id, false)
        .await
        .map_err(|_| not_found())?;
    let mut yields = Vec::new();
    let mut observations = Vec::new();
    let mut engine_version = String::new();
    for (i, item) in list.into_iter().flatten().enumerate() {
        let get = |k: &str| item.get(k).and_then(Value::as_f64).unwrap_or_default();
        let request = proto::BrewhouseEfficiencyRequest {
            wort_volume_l: get("wort_volume_l"),
            wort_extract_p: get("wort_extract_p"),
            measured_hot: item
                .get("measured_hot")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            malt_kg: get("malt_kg"),
            ..Default::default()
        };
        let mut scope = CalcScope::new(ws, Some(ctx.identity.user_id));
        scope.field_prefix = format!("/observations/{i}");
        let done = state
            .engine
            .calculate(
                &mut tx,
                &scope,
                "brewhouse_efficiency",
                request,
                |ch, r| async move {
                    proto::brewery_service_client::BreweryServiceClient::new(ch)
                        .brewhouse_efficiency(r)
                        .await
                },
            )
            .await?;
        engine_version = done.engine_version;
        yields.push(done.output.brewhouse_yield_pct);
        observations.push(json!({
            "brewhouse_yield_pct": done.output.brewhouse_yield_pct,
            "source": source_name(done.source),
        }));
    }
    tx.commit().await?;
    // TODO: Move the average and the spread rule into the engine when the engine has a
    // calibration function. The API holds no formulas; this rule is a stand-in.
    let (proposal, reason) = if yields.len() < WINDOW {
        (Value::Null, json!("too_few_observations"))
    } else {
        let tail = &yields[yields.len() - WINDOW..];
        let low = tail.iter().copied().fold(f64::INFINITY, f64::min);
        let high = tail.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        if high - low > MAX_SPREAD_PCT {
            (Value::Null, json!("inconsistent_observations"))
        } else {
            let mean = tail.iter().sum::<f64>() / 3.0;
            (
                json!({
                    "efficiency_pct": (mean.clamp(0.0, 100.0) * 10.0).round() / 10.0,
                    "based_on": WINDOW,
                    "spread_pct": high - low,
                }),
                Value::Null,
            )
        }
    };
    Ok(Json(json!({
        "equipment_id": id,
        "current_efficiency_pct": equipment["efficiency_pct"],
        "observations": observations,
        "proposal": proposal,
        "reason": reason,
        "engine_version": engine_version,
    })))
}
