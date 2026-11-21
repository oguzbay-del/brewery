//! Device data intake (T2.25): `POST /v1/ingest/{key}` and `POST /v1/ingest/{key}/{format}`.
//!
//! The key in the address is the only credential. An unknown, rotated or deleted key gives `401`
//! (the same answer for all three, so the answer does not reveal which keys existed). The format
//! is the default of the device kind (iSpindel, RAPT, otherwise Brewfather Custom Stream) unless
//! the address names one (`ispindel`, `rapt`, `brewfather`). Formats and ranges: `devices::formats`.
//!
//! Answers: `200 {"status":"accepted"}`, `400 malformed`, `401 invalid_key`, `413` for a body over
//! 8 KiB, `422 rejected` with the fields out of range, `503 unavailable` (the sender retries).
//!
//! TODO: limit the rate for each device (readings come every 15 minutes or less often).

use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use devices::{
    formats::{self, Format, GravityUnit, ParseError},
    intake,
};
use serde_json::{Value, json};

use crate::IngestState;

/// Largest accepted body. A reading is a few hundred bytes.
pub const BODY_LIMIT: usize = 8 * 1024;

type Reply = (StatusCode, Json<Value>);

fn reply(status: StatusCode, text: &str) -> Reply {
    (status, Json(json!({ "status": text })))
}

pub(crate) async fn ingest_default(
    state: State<IngestState>,
    Path(key): Path<String>,
    body: Bytes,
) -> Reply {
    handle(&state, &key, None, &body).await
}

pub(crate) async fn ingest_format(
    state: State<IngestState>,
    Path((key, format)): Path<(String, String)>,
    body: Bytes,
) -> Reply {
    let Some(format) = Format::from_name(&format) else {
        return reply(StatusCode::NOT_FOUND, "unknown_format");
    };
    handle(&state, &key, Some(format), &body).await
}

async fn handle(state: &IngestState, key: &str, format: Option<Format>, body: &[u8]) -> Reply {
    if !devices::is_key_shape(key) {
        return reply(StatusCode::UNAUTHORIZED, "invalid_key");
    }
    let device = match intake::find(&state.pool, key).await {
        Ok(Some(d)) => d,
        Ok(None) => return reply(StatusCode::UNAUTHORIZED, "invalid_key"),
        Err(e) => {
            tracing::error!(error = %e, "device lookup failed");
            return reply(StatusCode::SERVICE_UNAVAILABLE, "unavailable");
        }
    };
    let format = format.unwrap_or_else(|| Format::for_kind(&device.kind));
    let unit = GravityUnit::from_calibration(device.calibration.as_ref());
    let readings = match formats::parse(format, body, unit) {
        Ok(r) => r,
        Err(ParseError::Malformed(why)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"status": "malformed", "detail": why})),
            );
        }
        Err(ParseError::Rejected(issues)) => {
            let issues: Vec<Value> = issues
                .iter()
                .map(|i| json!({"field": i.field, "code": i.code}))
                .collect();
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"status": "rejected", "issues": issues})),
            );
        }
    };
    let result = async {
        let mut tx = state.pool.begin().await?;
        intake::store(&mut tx, &device, readings, Utc::now()).await?;
        tx.commit().await?;
        Ok::<_, db::Error>(())
    }
    .await;
    match result {
        Ok(()) => reply(StatusCode::OK, "accepted"),
        Err(e) => {
            tracing::error!(error = %e, device_id = %device.id, "storing a reading failed");
            reply(StatusCode::SERVICE_UNAVAILABLE, "unavailable")
        }
    }
}
