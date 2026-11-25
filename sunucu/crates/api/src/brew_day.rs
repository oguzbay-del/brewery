//! Brew day package (T2.22): the steps, timers and targets of one batch, calculated by the server,
//! signed and with an expiry time.
//!
//! The package is a snapshot of what the recipe version says on the brew day. The client works
//! from it without a connection. The server signs the package (HMAC-SHA256).
//!
//! The current key is `BREW_PACKAGE_KEY` or `BREW_PACKAGE_KEY_FILE`, with `BREW_PACKAGE_KEY_ID`
//! (default `k1`). Previous IDs come from `BREW_PACKAGE_PREVIOUS_KEY_IDS` (comma separated); each
//! secret uses `BREW_PACKAGE_KEY_<ID>` or `BREW_PACKAGE_KEY_<ID>_FILE` (uppercase ID, `-` to `_`).
//! New packages use only the current key; verification selects a key by the stored `key_id`.
//!
//! A measurement that names the package ID is accepted only when the package belongs to the
//! batch, has not expired, is not revoked and still has a valid signature ([`check_package`]).

#![allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::type_complexity
)]

use std::sync::{OnceLock, RwLock};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::Response,
    routing::{get, post},
};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgConnection};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState, concurrency,
    context::RequestContext,
    error::{ApiError, ApiResult, FieldIssue},
    library::{not_found, workspace},
    recipe_org::{conflict, optional_body},
    recipes::version_json,
};

/// Permission that creates and revokes packages.
pub const PLAN_PERMISSION: &str = "batch.plan";
/// Permission that reads packages.
pub const READ_PERMISSION: &str = "recipe.view";
/// Default life of a package, in hours.
pub const DEFAULT_VALID_HOURS: i64 = 72;
/// Longest life of a package, in hours.
pub const MAX_VALID_HOURS: i64 = 168;

// --- signing key --------------------------------------------------------------------------

#[derive(Clone, Default)]
struct SigningKeys {
    current: Option<(Vec<u8>, String)>,
    previous: Vec<(String, Vec<u8>)>,
}

static KEY_OVERRIDE: RwLock<Option<SigningKeys>> = RwLock::new(None);
static KEY_ENV: OnceLock<SigningKeys> = OnceLock::new();

/// Sets the current signing key in code (tests, or a service that loads the key itself).
/// It has priority over the environment and leaves the previous-key list unchanged.
pub fn set_signing_key(key: &[u8], key_id: &str) {
    let mut keys = loaded_keys();
    keys.current = Some((key.to_vec(), key_id.to_owned()));
    if let Ok(mut g) = KEY_OVERRIDE.write() {
        *g = Some(keys);
    }
}

/// Sets the previous signing keys in code. An empty slice resets the previous-key list.
/// It has priority over `BREW_PACKAGE_PREVIOUS_KEY_IDS` and is intended for tests.
pub fn set_previous_keys(keys: &[(&str, &[u8])]) {
    let mut loaded = loaded_keys();
    loaded.previous = keys
        .iter()
        .map(|(id, key)| ((*id).to_owned(), key.to_vec()))
        .collect();
    if let Ok(mut g) = KEY_OVERRIDE.write() {
        *g = Some(loaded);
    }
}

fn key_from_env() -> Option<(Vec<u8>, String)> {
    let lookup = |n: &str| std::env::var(n).ok();
    let secret =
        common::config::secret_from_lookup(lookup, "BREW_PACKAGE_KEY", "BREW_PACKAGE_KEY_FILE")
            .ok()
            .flatten()?;
    let id = lookup("BREW_PACKAGE_KEY_ID")
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "k1".to_owned());
    Some((secret.into_bytes(), id))
}

fn valid_key_id(id: &str) -> bool {
    (1..=32).contains(&id.len())
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

fn previous_keys_from_lookup<F>(lookup: F, current_id: &str) -> Vec<(String, Vec<u8>)>
where
    F: Fn(&str) -> Option<String>,
{
    let Some(raw_ids) = lookup("BREW_PACKAGE_PREVIOUS_KEY_IDS") else {
        return Vec::new();
    };
    let mut previous = Vec::new();
    for raw_id in raw_ids.split(',') {
        let id = raw_id.trim();
        if id.is_empty() {
            continue;
        }
        if !valid_key_id(id) {
            tracing::error!(key_id = id, reason = "invalid_id", "previous brew package key skipped");
            continue;
        }
        if id == current_id {
            tracing::error!(key_id = id, reason = "same_as_current", "previous brew package key skipped");
            continue;
        }

        let suffix = id.to_ascii_uppercase().replace('-', "_");
        let key_name = format!("BREW_PACKAGE_KEY_{suffix}");
        let file_name = format!("{key_name}_FILE");
        let secret = common::config::secret_from_lookup(
            |name| match name {
                "PREVIOUS_BREW_PACKAGE_KEY" => lookup(&key_name),
                "PREVIOUS_BREW_PACKAGE_KEY_FILE" => lookup(&file_name),
                _ => None,
            },
            "PREVIOUS_BREW_PACKAGE_KEY",
            "PREVIOUS_BREW_PACKAGE_KEY_FILE",
        );
        match secret {
            Ok(Some(secret)) => previous.push((id.to_owned(), secret.into_bytes())),
            Ok(None) => tracing::error!(
                key_id = id,
                reason = "missing_secret",
                "previous brew package key skipped"
            ),
            Err(error) => tracing::error!(
                key_id = id,
                reason = %error,
                "previous brew package key skipped"
            ),
        }
    }
    previous
}

fn keys_from_env() -> SigningKeys {
    let current = key_from_env();
    let current_id = current.as_ref().map_or("", |(_, id)| id.as_str());
    let previous = previous_keys_from_lookup(|name| std::env::var(name).ok(), current_id);
    SigningKeys { current, previous }
}

fn loaded_keys() -> SigningKeys {
    if let Ok(g) = KEY_OVERRIDE.read()
        && let Some(keys) = g.as_ref()
    {
        return keys.clone();
    }
    KEY_ENV.get_or_init(keys_from_env).clone()
}

/// The current signing key and its identifier. `503` when no current key is configured.
fn signing_key() -> ApiResult<(Vec<u8>, String)> {
    loaded_keys().current.ok_or_else(|| {
        tracing::error!("BREW_PACKAGE_KEY is not set; brew day packages cannot be signed");
        ApiError::unavailable(30)
    })
}

fn verification_key<'a>(keys: &'a SigningKeys, key_id: &str) -> Option<&'a [u8]> {
    if let Some((key, current_id)) = keys.current.as_ref()
        && current_id == key_id
    {
        return Some(key);
    }
    keys.previous
        .iter()
        .find(|(id, _)| id == key_id)
        .map(|(_, key)| key.as_slice())
}

/// Canonical text of a JSON value: object keys sorted, no spaces. It does not depend on the
/// key order of the serializer or of PostgreSQL `jsonb`.
fn canonical(v: &Value, out: &mut String) {
    match v {
        Value::Object(m) => {
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort();
            out.push('{');
            for (i, k) in keys.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(k).unwrap_or_default());
                out.push(':');
                canonical(&m[k], out);
            }
            out.push('}');
        }
        Value::Array(a) => {
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                canonical(x, out);
            }
            out.push(']');
        }
        other => out.push_str(&serde_json::to_string(other).unwrap_or_default()),
    }
}

fn signing_input(
    package_id: Uuid,
    ws: Uuid,
    batch_id: Uuid,
    content_version: i32,
    expires_at: Option<DateTime<Utc>>,
    content: &Value,
) -> Vec<u8> {
    let mut text = String::new();
    canonical(content, &mut text);
    let digest = Sha256::digest(text.as_bytes());
    let expires = expires_at.map(|t| t.timestamp_micros()).unwrap_or_default();
    let mut input =
        format!("brew-day-package-v1|{package_id}|{ws}|{batch_id}|{content_version}|{expires}|")
            .into_bytes();
    input.extend_from_slice(&digest);
    input
}

fn sign(key: &[u8], input: &[u8]) -> Vec<u8> {
    // HMAC accepts any key length.
    #[allow(clippy::expect_used)]
    let mut mac =
        <Hmac<Sha256> as KeyInit>::new_from_slice(key).expect("HMAC takes any key length");
    mac.update(input);
    mac.finalize().into_bytes().to_vec()
}

fn verify(key: &[u8], input: &[u8], signature: &[u8]) -> bool {
    #[allow(clippy::expect_used)]
    let mut mac =
        <Hmac<Sha256> as KeyInit>::new_from_slice(key).expect("HMAC takes any key length");
    mac.update(input);
    mac.verify_slice(signature).is_ok()
}

// --- the steps of a recipe ----------------------------------------------------------------

/// One planned step of a brew day, calculated from a recipe version.
#[derive(Debug, Clone)]
pub(crate) struct PlannedStep {
    pub kind: &'static str,
    pub name: String,
    pub duration_min: f64,
    pub timers: Vec<Value>,
}

fn num(v: &Value, key: &str) -> Option<f64> {
    v.get(key).and_then(Value::as_f64)
}

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or_default()
}

fn list<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

/// Days of the fermentation: the sum of the profile steps, else 14.
pub(crate) fn fermentation_days(version: &Value) -> f64 {
    let snapshot = version
        .get("fermentation_snapshot")
        .cloned()
        .unwrap_or(Value::Null);
    let sum: f64 = list(&snapshot, "steps")
        .iter()
        .filter_map(|s| num(s, "days"))
        .sum();
    if sum > 0.0 { sum } else { 14.0 }
}

/// Days of the conditioning after the fermentation (`conditioning_days` of the profile, else 14).
pub(crate) fn conditioning_days(version: &Value) -> f64 {
    version
        .get("fermentation_snapshot")
        .and_then(|s| num(s, "conditioning_days"))
        .filter(|d| *d >= 0.0)
        .unwrap_or(14.0)
}

/// The steps of the brew day with duration and timers, in order.
pub(crate) fn plan_steps(version: &Value) -> Vec<PlannedStep> {
    let mut steps = vec![PlannedStep {
        kind: "prepare",
        name: "Prepare equipment".to_owned(),
        duration_min: 30.0,
        timers: vec![],
    }];
    let mash = version.get("mash_snapshot").cloned().unwrap_or(Value::Null);
    for (i, s) in list(&mash, "steps").iter().enumerate() {
        let minutes = num(s, "time_min").unwrap_or(60.0);
        let temp = num(s, "temp_c").unwrap_or(0.0);
        steps.push(PlannedStep {
            kind: "mash_step",
            name: format!("Mash step {}: {temp} C", i + 1),
            duration_min: minutes,
            timers: vec![json!({"label": "Hold", "at_min": 0.0, "duration_s": minutes * 60.0})],
        });
    }
    let boil = num(version, "boil_time_min").unwrap_or(0.0);
    if boil > 0.0 {
        let mut timers = Vec::new();
        for h in list(version, "hops") {
            let at = match text(h, "usage") {
                "first_wort" => 0.0,
                "boil" => (boil - num(h, "time_min").unwrap_or(0.0)).max(0.0),
                _ => continue,
            };
            timers.push(json!({
                "label": format!("Add {} ({} g)", text(h, "name"), num(h, "amount_kg").unwrap_or(0.0) * 1000.0),
                "at_min": at, "duration_s": 0.0
            }));
        }
        for m in list(version, "miscs") {
            if text(m, "usage") == "boil" {
                timers.push(json!({
                    "label": format!("Add {}", text(m, "name")),
                    "at_min": (boil - num(m, "time_min").unwrap_or(0.0)).max(0.0), "duration_s": 0.0
                }));
            }
        }
        steps.push(PlannedStep {
            kind: "boil",
            name: "Boil".to_owned(),
            duration_min: boil,
            timers,
        });
        steps.push(PlannedStep {
            kind: "cool",
            name: "Cool and transfer".to_owned(),
            duration_min: 30.0,
            timers: vec![],
        });
    }
    steps.push(PlannedStep {
        kind: "pitch",
        name: "Pitch yeast".to_owned(),
        duration_min: 10.0,
        timers: vec![],
    });
    steps.push(PlannedStep {
        kind: "ferment",
        name: "Fermentation".to_owned(),
        duration_min: fermentation_days(version) * 1440.0,
        timers: vec![],
    });
    steps.push(PlannedStep {
        kind: "package",
        name: "Package".to_owned(),
        duration_min: 60.0,
        timers: vec![],
    });
    steps
}

/// Start of the first step on `date` (08:00 UTC; the client shows the local time).
pub(crate) fn day_start(date: NaiveDate) -> DateTime<Utc> {
    date.and_hms_opt(8, 0, 0)
        .map_or(DateTime::UNIX_EPOCH, |t| t.and_utc())
}

/// Start and end time of each step, one after the other from `start`.
pub(crate) fn schedule(
    steps: &[PlannedStep],
    start: DateTime<Utc>,
) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
    let mut at = start;
    steps
        .iter()
        .map(|s| {
            let begin = at;
            at += Duration::seconds((s.duration_min * 60.0).round() as i64);
            (begin, at)
        })
        .collect()
}

// --- the package --------------------------------------------------------------------------

/// A brew day package.
#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct BrewDayPackage {
    /// Package ID. Send it with a measurement.
    pub id: Uuid,
    /// The batch.
    pub batch_id: Uuid,
    /// Number of the package of this batch.
    pub content_version: i32,
    /// Steps, timers and targets (`schema_version` 1).
    #[schema(value_type = Object)]
    pub content: Value,
    /// HMAC-SHA256 signature, base64.
    pub signature: String,
    /// Identifier of the signing key.
    pub key_id: String,
    /// The package is not valid after this time.
    pub expires_at: Option<DateTime<Utc>>,
    /// The package was revoked at this time.
    pub revoked_at: Option<DateTime<Utc>>,
    /// Created at.
    pub created_at: DateTime<Utc>,
}

const PACKAGE_COLUMNS: &str = "id, batch_id, content_version, content, \
    encode(signature, 'base64') AS signature, key_id, expires_at, revoked_at, created_at";

/// Body of a package create.
#[derive(Debug, Default, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct NewPackage {
    /// Life of the package in hours, 1 to 168 (default 72).
    pub valid_hours: Option<i64>,
}

fn strip_newlines(s: &str) -> String {
    s.replace('\n', "")
}

async fn fetch_package(tx: &mut PgConnection, id: Uuid) -> ApiResult<BrewDayPackage> {
    let mut p: BrewDayPackage = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {PACKAGE_COLUMNS} FROM brew_day_packages WHERE id = $1 AND deleted_at IS NULL"
    )))
    .bind(id)
    .fetch_optional(tx)
    .await?
    .ok_or_else(not_found)?;
    p.signature = strip_newlines(&p.signature);
    Ok(p)
}

/// Why a package is not accepted for a measurement of a batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PackageProblem {
    /// No such package in the workspace.
    NotFound,
    /// The package belongs to another batch.
    BatchMismatch,
    /// The package expired.
    Expired,
    /// The package was revoked.
    Revoked,
    /// The signature does not match the stored content.
    BadSignature,
}

impl PackageProblem {
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::NotFound => "package_not_found",
            Self::BatchMismatch => "package_batch_mismatch",
            Self::Expired => "package_expired",
            Self::Revoked => "package_revoked",
            Self::BadSignature => "package_signature_invalid",
        }
    }
}

/// Checks a package for a measurement of `batch_id`.
pub(crate) async fn check_package(
    tx: &mut PgConnection,
    ws: Uuid,
    batch_id: Uuid,
    package_id: Uuid,
) -> ApiResult<Result<(), PackageProblem>> {
    type Row = (
        Uuid,
        i32,
        Value,
        Vec<u8>,
        String,
        Option<DateTime<Utc>>,
        Option<DateTime<Utc>>,
        bool,
    );
    let row: Option<Row> = sqlx::query_as(
        "SELECT batch_id, content_version, content, signature, key_id, expires_at, revoked_at, \
                (expires_at IS NOT NULL AND expires_at <= now()) \
         FROM brew_day_packages WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(package_id)
    .fetch_optional(tx)
    .await?;
    let Some((
        pkg_batch,
        content_version,
        content,
        signature,
        key_id,
        expires_at,
        revoked_at,
        expired,
    )) = row
    else {
        return Ok(Err(PackageProblem::NotFound));
    };
    if pkg_batch != batch_id {
        return Ok(Err(PackageProblem::BatchMismatch));
    }
    if revoked_at.is_some() {
        return Ok(Err(PackageProblem::Revoked));
    }
    if expired {
        return Ok(Err(PackageProblem::Expired));
    }
    let keys = loaded_keys();
    let input = signing_input(
        package_id,
        ws,
        batch_id,
        content_version,
        expires_at,
        &content,
    );
    let Some(key) = verification_key(&keys, &key_id) else {
        return Ok(Err(PackageProblem::BadSignature));
    };
    if !verify(key, &input, &signature) {
        return Ok(Err(PackageProblem::BadSignature));
    }
    Ok(Ok(()))
}

/// Creates a signed brew day package for a batch.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/batches/{id}/brew-day-package",
    tag = "batches",
    operation_id = "brewdaypackage_create",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Batch"),
        ("Idempotency-Key" = String, Header, description = "Client generated key (ADR-0008 rule 6)")
    ),
    request_body = NewPackage,
    responses((status = 201, description = "The signed package", body = BrewDayPackage,
        example = json!({"id": "0192f1c07a3b7000800000000000000b",
            "batch_id": "0192f1c07a3b7000800000000000000c", "content_version": 1,
            "content": {"schema_version": 1, "targets": {"og_sg": 1.055}, "steps": []},
            "signature": "c2lnbmF0dXJl", "key_id": "k1", "expires_at": "2026-11-23T10:00:00Z",
            "revoked_at": null, "created_at": "2026-11-20T10:00:00Z"})))
)]
pub async fn brewdaypackage_create(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, batch_id)): Path<(Uuid, Uuid)>,
    body: axum::body::Bytes,
) -> ApiResult<Response> {
    ctx.require(PLAN_PERMISSION)?;
    let ws = workspace(&ctx)?;
    let input: NewPackage = optional_body(&body)?;
    let hours = input.valid_hours.unwrap_or(DEFAULT_VALID_HOURS);
    if !(1..=MAX_VALID_HOURS).contains(&hours) {
        return Err(ApiError::validation(vec![
            FieldIssue::new("/valid_hours", "out_of_range")
                .with_params(json!({"min": 1, "max": MAX_VALID_HOURS})),
        ]));
    }
    let (key, key_id) = signing_key()?;
    let user = ctx.identity.user_id;
    let mut tx = ctx.begin(&state.db).await?;
    let batch: Option<(String, String, String, Uuid, Option<f64>, Option<NaiveDate>)> =
        sqlx::query_as(
            "SELECT stage, code, name, recipe_version_id, target_volume_l, planned_date \
         FROM batches WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(batch_id)
        .fetch_optional(&mut *tx)
        .await?;
    let (batch_stage, code, name, version_id, target_volume, planned_date) =
        batch.ok_or_else(not_found)?;
    if matches!(batch_stage.as_str(), "completed" | "cancelled") {
        return Err(conflict("/stage", "batch_closed"));
    }
    let v = version_json(&mut tx, version_id).await?;
    let summary = v.get("calc_summary").cloned().unwrap_or(Value::Null);
    let steps = plan_steps(&v);
    let start = planned_date.map(day_start);
    let times = start.map(|s| schedule(&steps, s));
    let step_ids: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM batch_steps WHERE batch_id = $1 AND deleted_at IS NULL ORDER BY position",
    )
    .bind(batch_id)
    .fetch_all(&mut *tx)
    .await?;
    let step_values: Vec<Value> = steps
        .iter()
        .enumerate()
        .map(|(i, s)| {
            json!({
                "position": i, "step_id": step_ids.get(i), "kind": s.kind, "name": s.name,
                "duration_min": s.duration_min, "timers": s.timers,
                "planned_start_at": times.as_ref().and_then(|t| t.get(i)).map(|t| t.0),
                "planned_end_at": times.as_ref().and_then(|t| t.get(i)).map(|t| t.1),
            })
        })
        .collect();
    let ingredients: Vec<Value> = [
        ("fermentable", "fermentables", "amount_kg"),
        ("hop", "hops", "amount_kg"),
        ("yeast", "yeasts", "amount_kg"),
        ("misc", "miscs", "amount"),
        ("water_addition", "water_additions", "amount_kg"),
    ]
    .iter()
    .flat_map(|(kind, key, amount)| {
        list(&v, key).iter().map(move |l| {
            json!({
                "line_kind": kind, "line_id": l.get("id"),
                "name": l.get("name").or_else(|| l.get("agent")),
                "amount": l.get(*amount), "usage": l.get("usage").or_else(|| l.get("stage")),
                "time_min": l.get("time_min"),
            })
        })
    })
    .collect();
    let now = Utc::now();
    let expires_at = now + Duration::hours(hours);
    let package_id = Uuid::now_v7();
    let content = json!({
        "schema_version": 1,
        "issued_at": now,
        "expires_at": expires_at,
        "batch": {"id": batch_id, "code": code, "name": name, "planned_date": planned_date,
                  "target_volume_l": target_volume.or_else(|| num(&v, "batch_volume_l"))},
        "recipe": {"recipe_id": v["recipe_id"], "version_id": version_id, "number": v["number"],
                   "kind": v["kind"]},
        "targets": {
            "og_sg": summary.get("og_sg"), "fg_sg": summary.get("fg_sg"),
            "ibu": summary.get("ibu"), "color_srm": summary.get("color_srm"),
            "abv_pct": summary.get("abv_pct"),
            "batch_volume_l": v["batch_volume_l"], "boil_volume_l": v["boil_volume_l"],
            "boil_time_min": v["boil_time_min"], "efficiency_pct": v["efficiency_pct"],
        },
        "ingredients": ingredients,
        "steps": step_values,
    });
    let content_version: i32 = sqlx::query_scalar(
        "SELECT COALESCE(max(content_version), 0) + 1 FROM brew_day_packages WHERE batch_id = $1",
    )
    .bind(batch_id)
    .fetch_one(&mut *tx)
    .await?;
    // The signature covers the content as the database returns it (jsonb order and number text).
    sqlx::query(
        "INSERT INTO brew_day_packages (id, workspace_id, batch_id, content_version, content, \
             signature, key_id, expires_at, created_by, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)",
    )
    .bind(package_id)
    .bind(ws)
    .bind(batch_id)
    .bind(content_version)
    .bind(&content)
    .bind(Vec::<u8>::new())
    .bind(&key_id)
    .bind(expires_at)
    .bind(user)
    .execute(&mut *tx)
    .await?;
    let (stored, expires): (Value, Option<DateTime<Utc>>) =
        sqlx::query_as("SELECT content, expires_at FROM brew_day_packages WHERE id = $1")
            .bind(package_id)
            .fetch_one(&mut *tx)
            .await?;
    let signature = sign(
        &key,
        &signing_input(package_id, ws, batch_id, content_version, expires, &stored),
    );
    sqlx::query("UPDATE brew_day_packages SET signature = $2 WHERE id = $1")
        .bind(package_id)
        .bind(&signature)
        .execute(&mut *tx)
        .await?;
    let out = fetch_package(&mut tx, package_id).await?;
    tx.commit().await?;
    Ok(crate::library::created_response(
        &format!("/v1/workspaces/{ws}/brew-day-packages/{package_id}"),
        1,
        &out,
    ))
}

/// Reads the newest package of a batch.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/batches/{id}/brew-day-package",
    tag = "batches",
    operation_id = "brewdaypackage_latest",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Batch")
    ),
    responses((status = 200, description = "The newest package", body = BrewDayPackage,
        example = json!({"id": "0192f1c07a3b7000800000000000000b",
            "batch_id": "0192f1c07a3b7000800000000000000c", "content_version": 1,
            "content": {"schema_version": 1, "targets": {"og_sg": 1.055}, "steps": []},
            "signature": "c2lnbmF0dXJl", "key_id": "k1", "expires_at": "2026-11-23T10:00:00Z",
            "revoked_at": null, "created_at": "2026-11-20T10:00:00Z"})))
)]
pub async fn brewdaypackage_latest(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, batch_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<BrewDayPackage>> {
    ctx.require(READ_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    let id: Uuid = sqlx::query_scalar(
        "SELECT id FROM brew_day_packages WHERE batch_id = $1 AND deleted_at IS NULL \
         ORDER BY content_version DESC LIMIT 1",
    )
    .bind(batch_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(not_found)?;
    Ok(Json(fetch_package(&mut tx, id).await?))
}

/// Reads one package.
#[utoipa::path(
    get,
    path = "/v1/workspaces/{workspace_id}/brew-day-packages/{id}",
    tag = "batches",
    operation_id = "brewdaypackage_get",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Package")
    ),
    responses((status = 200, description = "The package", body = BrewDayPackage,
        example = json!({"id": "0192f1c07a3b7000800000000000000b",
            "batch_id": "0192f1c07a3b7000800000000000000c", "content_version": 1,
            "content": {"schema_version": 1, "targets": {"og_sg": 1.055}, "steps": []},
            "signature": "c2lnbmF0dXJl", "key_id": "k1", "expires_at": "2026-11-23T10:00:00Z",
            "revoked_at": null, "created_at": "2026-11-20T10:00:00Z"})))
)]
pub async fn brewdaypackage_get(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<BrewDayPackage>> {
    ctx.require(READ_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    Ok(Json(fetch_package(&mut tx, id).await?))
}

/// Revokes a package. The server then rejects measurements that name it. The call is safe to repeat.
#[utoipa::path(
    post,
    path = "/v1/workspaces/{workspace_id}/brew-day-packages/{id}/revoke",
    tag = "batches",
    operation_id = "brewdaypackage_revoke",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace"),
        ("id" = Uuid, Path, description = "Package")
    ),
    responses((status = 200, description = "The revoked package", body = BrewDayPackage,
        example = json!({"id": "0192f1c07a3b7000800000000000000b",
            "batch_id": "0192f1c07a3b7000800000000000000c", "content_version": 1,
            "content": {"schema_version": 1, "targets": {"og_sg": 1.055}, "steps": []},
            "signature": "c2lnbmF0dXJl", "key_id": "k1", "expires_at": "2026-11-23T10:00:00Z",
            "revoked_at": "2026-11-20T11:00:00Z", "created_at": "2026-11-20T10:00:00Z"})))
)]
pub async fn brewdaypackage_revoke(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path((_ws, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Response> {
    ctx.require(PLAN_PERMISSION)?;
    let mut tx = ctx.begin(&state.db).await?;
    fetch_package(&mut tx, id).await?;
    sqlx::query(
        "UPDATE brew_day_packages SET revoked_at = now(), updated_by = $2 \
         WHERE id = $1 AND revoked_at IS NULL",
    )
    .bind(id)
    .bind(ctx.identity.user_id)
    .execute(&mut *tx)
    .await?;
    let out = fetch_package(&mut tx, id).await?;
    tx.commit().await?;
    Ok(concurrency::record_response(StatusCode::OK, 1, &out))
}

// --- routes -------------------------------------------------------------------------------

/// The routes of this module.
pub fn routes(state: &AppState) -> Router<AppState> {
    const WS: &str = "/v1/workspaces/{ws}";
    let creates = crate::idempotent(
        state,
        Router::new().route(
            &format!("{WS}/batches/{{id}}/brew-day-package"),
            post(brewdaypackage_create),
        ),
    );
    let others = Router::new()
        .route(
            &format!("{WS}/batches/{{id}}/brew-day-package"),
            get(brewdaypackage_latest),
        )
        .route(
            &format!("{WS}/brew-day-packages/{{id}}"),
            get(brewdaypackage_get),
        )
        .route(
            &format!("{WS}/brew-day-packages/{{id}}/revoke"),
            post(brewdaypackage_revoke),
        );
    crate::authenticated(state, creates.merge(others))
}

mod spec {
    use crate::concurrency::Concurrency;
    use crate::error::ErrorCode;
    use crate::error::ErrorCode::{
        Conflict, Forbidden, IdempotencyInProgress, IdempotencyMismatch, Internal,
        MalformedRequest, NotFound, PayloadTooLarge, RateLimited, Unauthenticated, Unavailable,
        UnsupportedMediaType, UpgradeRequired, ValidationFailed,
    };
    use crate::openapi::{OperationSpec, file_op};

    const READ: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        RateLimited,
        UpgradeRequired,
        Internal,
        Unavailable,
    ];
    const CREATE: &[ErrorCode] = &[
        MalformedRequest,
        Unauthenticated,
        Forbidden,
        NotFound,
        Conflict,
        IdempotencyInProgress,
        PayloadTooLarge,
        UnsupportedMediaType,
        ValidationFailed,
        IdempotencyMismatch,
        RateLimited,
        UpgradeRequired,
        Internal,
        Unavailable,
    ];
    pub(super) const OPERATIONS: &[OperationSpec] = &[
        file_op(
            "brewdaypackage_create",
            Some("batch.plan"),
            Some(Concurrency::Append),
            CREATE,
        ),
        file_op("brewdaypackage_latest", Some("recipe.view"), None, READ),
        file_op("brewdaypackage_get", Some("recipe.view"), None, READ),
        file_op("brewdaypackage_revoke", Some("batch.plan"), None, READ),
    ];
}

/// `OpenAPI` fragment of this module.
#[derive(Debug, utoipa::OpenApi)]
#[openapi(
    paths(
        brewdaypackage_create,
        brewdaypackage_latest,
        brewdaypackage_get,
        brewdaypackage_revoke
    ),
    components(schemas(BrewDayPackage, NewPackage))
)]
struct ModuleDoc;

/// Registry entry (`crate::modules`).
pub const MODULE: crate::modules::Module = crate::modules::Module {
    name: "brew_day",
    routes,
    operations: spec::OPERATIONS,
    openapi: <ModuleDoc as utoipa::OpenApi>::openapi,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previous_key_ids_parse_valid_entries_and_ignore_empty_items() {
        let keys = previous_keys_from_lookup(
            |name| match name {
                "BREW_PACKAGE_PREVIOUS_KEY_IDS" => Some(" k1, ,k2,, ".to_owned()),
                "BREW_PACKAGE_KEY_K1" => Some("first-secret".to_owned()),
                "BREW_PACKAGE_KEY_K2" => Some("second-secret".to_owned()),
                _ => None,
            },
            "k0",
        );
        assert_eq!(
            keys,
            vec![
                ("k1".to_owned(), b"first-secret".to_vec()),
                ("k2".to_owned(), b"second-secret".to_vec()),
            ]
        );
    }

    #[test]
    fn previous_key_ids_skip_invalid_current_and_missing_secrets() {
        let keys = previous_keys_from_lookup(
            |name| match name {
                "BREW_PACKAGE_PREVIOUS_KEY_IDS" => {
                    Some("k1, bad id, k0, k-missing".to_owned())
                }
                "BREW_PACKAGE_KEY_K1" => Some("first-secret".to_owned()),
                _ => None,
            },
            "k0",
        );
        assert_eq!(keys, vec![("k1".to_owned(), b"first-secret".to_vec())]);
    }

    #[test]
    fn previous_key_secret_can_be_loaded_from_a_file() {
        let path = std::env::temp_dir().join(format!("brew-package-key-{}.txt", Uuid::now_v7()));
        let write = std::fs::write(&path, "file-secret\\n");
        assert!(write.is_ok(), "temporary key file should be written");
        let file_name = path.to_string_lossy().into_owned();
        let keys = previous_keys_from_lookup(
            |name| match name {
                "BREW_PACKAGE_PREVIOUS_KEY_IDS" => Some("k-file".to_owned()),
                "BREW_PACKAGE_KEY_K_FILE_FILE" => Some(file_name.clone()),
                _ => None,
            },
            "k0",
        );
        let remove = std::fs::remove_file(path);
        assert!(remove.is_ok(), "temporary key file should be removed");
        assert_eq!(keys, vec![("k-file".to_owned(), b"file-secret".to_vec())]);
    }

    #[test]
    fn canonical_text_does_not_depend_on_key_order() {
        let a = json!({"b": [1, {"y": 1, "x": 2}], "a": null});
        let b = json!({"a": null, "b": [1, {"x": 2, "y": 1}]});
        let (mut ta, mut tb) = (String::new(), String::new());
        canonical(&a, &mut ta);
        canonical(&b, &mut tb);
        assert_eq!(ta, tb);
        assert_eq!(ta, r#"{"a":null,"b":[1,{"x":2,"y":1}]}"#);
    }

    #[test]
    fn signature_detects_a_change() {
        let id = Uuid::nil();
        let input = signing_input(id, id, id, 1, None, &json!({"a": 1}));
        let sig = sign(b"key", &input);
        assert!(verify(b"key", &input, &sig));
        let other = signing_input(id, id, id, 1, None, &json!({"a": 2}));
        assert!(!verify(b"key", &other, &sig));
        assert!(!verify(b"other", &input, &sig));
    }

    #[test]
    fn steps_follow_the_recipe() {
        let v = json!({"boil_time_min": 60.0, "hops": [
            {"name": "A", "usage": "boil", "time_min": 60.0, "amount_kg": 0.03},
            {"name": "B", "usage": "boil", "time_min": 5.0, "amount_kg": 0.02},
            {"name": "C", "usage": "dry_hop", "time_min": 0.0, "amount_kg": 0.02}],
            "mash_snapshot": {"steps": [{"temp_c": 67.0, "time_min": 60.0}]}});
        let steps = plan_steps(&v);
        let kinds: Vec<&str> = steps.iter().map(|s| s.kind).collect();
        assert_eq!(
            kinds,
            [
                "prepare",
                "mash_step",
                "boil",
                "cool",
                "pitch",
                "ferment",
                "package"
            ]
        );
        let boil = &steps[2];
        assert_eq!(boil.timers.len(), 2);
        assert_eq!(boil.timers[1]["at_min"], 55.0);
        let times = schedule(&steps, DateTime::UNIX_EPOCH);
        assert_eq!(times[0].1, times[1].0);
    }
}
