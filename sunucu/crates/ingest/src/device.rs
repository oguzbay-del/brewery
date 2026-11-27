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
//! Rate limits (T2.54, `ratelimit.rs`): `429` with `Retry-After` over the limit for a device key or
//! for failed requests of one client IP.

use axum::{
    Json,
    body::Bytes,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use devices::{
    formats::{self, Format, GravityUnit, ParseError},
    intake,
};
use serde_json::{Value, json};

use crate::{
    IngestState,
    ratelimit::{Caller, DeviceLimiter},
};

/// Largest accepted body. A reading is a few hundred bytes.
pub const BODY_LIMIT: usize = 8 * 1024;

type Reply = Response;

fn reply(status: StatusCode, text: &str) -> Reply {
    (status, Json(json!({ "status": text }))).into_response()
}

pub(crate) async fn ingest_default(
    state: State<IngestState>,
    Extension(limiter): Extension<DeviceLimiter>,
    caller: Caller,
    Path(key): Path<String>,
    body: Bytes,
) -> Reply {
    handle(&state, &limiter, &caller, &key, None, &body).await
}

pub(crate) async fn ingest_format(
    state: State<IngestState>,
    Extension(limiter): Extension<DeviceLimiter>,
    caller: Caller,
    Path((key, format)): Path<(String, String)>,
    body: Bytes,
) -> Reply {
    let Some(format) = Format::from_name(&format) else {
        return reply(StatusCode::NOT_FOUND, "unknown_format");
    };
    handle(&state, &limiter, &caller, &key, Some(format), &body).await
}

/// `401`, or `429` when this failure is over the limit of the client IP.
async fn invalid_key(limiter: &DeviceLimiter, caller: &Caller) -> Reply {
    match limiter.note_invalid_key(caller).await {
        Ok(()) => reply(StatusCode::UNAUTHORIZED, "invalid_key"),
        Err(over) => over.into_response(),
    }
}

async fn handle(
    state: &IngestState,
    limiter: &DeviceLimiter,
    caller: &Caller,
    key: &str,
    format: Option<Format>,
    body: &[u8],
) -> Reply {
    if let Err(limited) = limiter.check_ip_blocked(caller).await {
        return limited.into_response();
    }
    if !devices::is_key_shape(key) {
        return invalid_key(limiter, caller).await;
    }
    let device = match intake::find(&state.pool, key).await {
        Ok(Some(d)) => d,
        Ok(None) => return invalid_key(limiter, caller).await,
        Err(e) => {
            tracing::error!(error = %e, "device lookup failed");
            return reply(StatusCode::SERVICE_UNAVAILABLE, "unavailable");
        }
    };
    if let Err(limited) = limiter.check_device(key).await {
        return limited.into_response();
    }
    let format = format.unwrap_or_else(|| Format::for_kind(&device.kind));
    let unit = GravityUnit::from_calibration(device.calibration.as_ref());
    let readings = match formats::parse(format, body, unit) {
        Ok(r) => r,
        Err(ParseError::Malformed(why)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"status": "malformed", "detail": why})),
            )
                .into_response();
        }
        Err(ParseError::Rejected(issues)) => {
            let issues: Vec<Value> = issues
                .iter()
                .map(|i| json!({"field": i.field, "code": i.code}))
                .collect();
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"status": "rejected", "issues": issues})),
            )
                .into_response();
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
