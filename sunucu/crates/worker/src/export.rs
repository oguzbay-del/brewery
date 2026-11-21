//! Export jobs (T2.26). The API queues a row in `export_jobs`; this module builds the file, puts
//! it in the object store and marks the job `done` with an expiry time. A later pass deletes
//! the expired files.
//!
//! Files:
//! - scope `recipe`: `BeerXML`, `BeerJSON` or JSON of the newest version.
//! - scope `batch`: measurements as CSV (optional time range), batch JSON, or both in a ZIP.
//! - scope `workspace` (full export, ZIP): recipes (`recipes.beerjson`, `recipes.beerxml`),
//!   `batches.csv`, `stock_lots.csv`, `equipment.json`, `ingredients.json` (own ingredients),
//!   `measurements/<batch code>-<year>.csv` (one file for each batch and year) and
//!   `manifest.json`. CSV cells are escaped against formula injection.
//!
//! The file is built in memory (limit 512 MiB). TODO: stream the ZIP for very large accounts.
//! - scope `account_data` (account data export, ZIP, T2.31): `account.json` (profile, sign-in
//!   methods without tokens, passkey names, sessions, push targets), `workspaces.json`, and for
//!   each workspace of the user the files `workspaces/<id>/me/*.json` (own support requests,
//!   settings, favorites, notification preferences). A Home workspace where the user is the
//!   only administrator also has the full workspace export (`workspaces/<id>/...`, including
//!   water profiles, devices, support requests and documents) and the document and attachment
//!   files (`workspaces/<id>/files/`).
//!
//! TODO: photos and document files are only in the account data export, not in the workspace
//! export.

#![allow(clippy::too_many_lines)]

use std::{collections::BTreeMap, sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use db::{ActorKind, DbContext};
use futures_util::StreamExt;
use interchange::{Recipe, csv, zip::ZipWriter};
use serde_json::{Value, json};
use sqlx::{PgConnection, PgPool, Row};
use storage::ObjectStore;
use tokio::sync::watch;
use uuid::Uuid;

use crate::{Error, Result};

/// How long a finished file stays in the object store.
pub const KEEP: Duration = Duration::from_hours(7 * 24);
/// Largest file the worker builds in memory.
pub const MAX_BYTES: usize = 512 * 1024 * 1024;
const POLL: Duration = Duration::from_secs(5);

fn format_err(msg: impl Into<String>) -> Error {
    Error::Format(msg.into())
}

/// A built file.
#[derive(Debug)]
pub struct Built {
    /// The bytes.
    pub bytes: Vec<u8>,
    /// Media type.
    pub content_type: &'static str,
    /// File name extension.
    pub ext: &'static str,
}

#[derive(Debug)]
struct Job {
    created_by: Option<Uuid>,
    scope: String,
    scope_id: Option<Uuid>,
    format: String,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
}

const SECTIONS: &[(&str, &str)] = &[
    ("fermentables", "recipe_fermentables"),
    ("hops", "recipe_hops"),
    ("yeasts", "recipe_yeasts"),
    ("miscs", "recipe_miscs"),
    ("water_additions", "recipe_water_additions"),
];

fn version_sql() -> String {
    let mut parts = Vec::new();
    for (key, table) in SECTIONS {
        parts.push(format!(
            "'{key}', (SELECT COALESCE(jsonb_agg(to_jsonb(l) ORDER BY l.position), '[]'::jsonb) \
             FROM {table} l WHERE l.recipe_version_id = v.id)"
        ));
    }
    format!(
        "SELECT (to_jsonb(v) - ARRAY['workspace_id','created_by','updated_by','deleted_at']) \
         || jsonb_build_object({}) FROM recipe_versions v WHERE v.id = $1 AND v.deleted_at IS NULL",
        parts.join(", ")
    )
}

/// The newest version of a recipe as the neutral model plus the raw version JSON.
async fn load_recipe(tx: &mut PgConnection, recipe_id: Uuid) -> Result<(Recipe, Value)> {
    let row = sqlx::query(
        "SELECT name, notes, COALESCE(approved_version_id, draft_version_id) AS version_id \
         FROM recipes WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(recipe_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::Error::from)?
    .ok_or_else(|| format_err("recipe not found"))?;
    let name: String = row.get("name");
    let notes: Option<String> = row.get("notes");
    let version_id: Option<Uuid> = row.get("version_id");
    let version_id = version_id.ok_or_else(|| format_err("recipe has no version"))?;
    let v: Value = sqlx::query_scalar(sqlx::AssertSqlSafe(version_sql()))
        .bind(version_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(db::Error::from)?;
    let mut recipe = Recipe::from_version_json(&name, notes.as_deref(), &v);
    if let Some(style_id) = v
        .get("style_id")
        .and_then(Value::as_str)
        .and_then(|s| s.parse::<Uuid>().ok())
    {
        recipe.style = sqlx::query_scalar("SELECT name FROM styles WHERE id = $1")
            .bind(style_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db::Error::from)?;
    }
    Ok((recipe, v))
}

fn json_bytes(v: &Value) -> Vec<u8> {
    serde_json::to_vec_pretty(v).unwrap_or_default()
}

fn safe_name(code: &str) -> String {
    let s: String = code
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .take(60)
        .collect();
    if s.is_empty() { "batch".to_owned() } else { s }
}

fn measurement_header() -> String {
    csv::text_line(&["measured_at", "kind", "value", "source", "device_id"])
}

fn measurement_line(
    at: DateTime<Utc>,
    kind: &str,
    value: f64,
    source: &str,
    device: Option<Uuid>,
) -> String {
    csv::line(&[
        csv::cell(&at.to_rfc3339()),
        csv::cell(kind),
        csv::number(value),
        csv::cell(source),
        csv::cell(&device.map(|d| d.to_string()).unwrap_or_default()),
    ])
}

async fn build_recipe(tx: &mut PgConnection, job: &Job) -> Result<Built> {
    let id = job.scope_id.ok_or_else(|| format_err("scope_id missing"))?;
    let (recipe, version) = load_recipe(tx, id).await?;
    Ok(match job.format.as_str() {
        "beerxml" => Built {
            bytes: interchange::beerxml::write(&[recipe]).into_bytes(),
            content_type: "application/xml",
            ext: "xml",
        },
        "beerjson" => Built {
            bytes: interchange::beerjson::write(&[recipe]).into_bytes(),
            content_type: "application/json",
            ext: "json",
        },
        "json" => Built {
            bytes: json_bytes(
                &json!({"recipe": {"id": id, "name": recipe.name, "notes": recipe.notes},
                "version": version}),
            ),
            content_type: "application/json",
            ext: "json",
        },
        other => {
            return Err(format_err(format!(
                "format {other} is not valid for a recipe"
            )));
        }
    })
}

async fn build_batch(tx: &mut PgConnection, job: &Job) -> Result<Built> {
    let id = job.scope_id.ok_or_else(|| format_err("scope_id missing"))?;
    let batch: Value = sqlx::query_scalar(
        "SELECT to_jsonb(b) - ARRAY['workspace_id','search','created_by','updated_by','deleted_at'] \
         FROM batches b WHERE b.id = $1 AND b.deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::Error::from)?
    .ok_or_else(|| format_err("batch not found"))?;
    let mut csv_text = measurement_header();
    let mut list = Vec::new();
    {
        let mut rows = sqlx::query(
            "SELECT measured_at, kind, value, source, device_id FROM measurements \
             WHERE batch_id = $1 AND ($2::timestamptz IS NULL OR measured_at >= $2) \
             AND ($3::timestamptz IS NULL OR measured_at <= $3) ORDER BY measured_at, id",
        )
        .bind(id)
        .bind(job.from)
        .bind(job.to)
        .fetch(&mut *tx);
        while let Some(row) = rows.next().await {
            let row = row.map_err(db::Error::from)?;
            let at: DateTime<Utc> = row.get("measured_at");
            let kind: String = row.get("kind");
            let value: f64 = row.get("value");
            let source: String = row.get("source");
            let device: Option<Uuid> = row.get("device_id");
            csv_text.push_str(&measurement_line(at, &kind, value, &source, device));
            list.push(
                json!({"measured_at": at, "kind": kind, "value": value, "source": source,
                "device_id": device}),
            );
            if csv_text.len() > MAX_BYTES {
                return Err(format_err("export is too large"));
            }
        }
    }
    let doc = json!({"batch": batch, "from": job.from, "to": job.to, "measurements": list});
    Ok(match job.format.as_str() {
        "csv" => Built {
            bytes: csv_text.into_bytes(),
            content_type: "text/csv",
            ext: "csv",
        },
        "json" => Built {
            bytes: json_bytes(&doc),
            content_type: "application/json",
            ext: "json",
        },
        "zip" => {
            let mut z = ZipWriter::new();
            z.add("batch.json", &json_bytes(&doc))
                .map_err(|e| format_err(e.to_string()))?;
            z.add("measurements.csv", csv_text.as_bytes())
                .map_err(|e| format_err(e.to_string()))?;
            Built {
                bytes: z.finish().map_err(|e| format_err(e.to_string()))?,
                content_type: "application/zip",
                ext: "zip",
            }
        }
        other => {
            return Err(format_err(format!(
                "format {other} is not valid for a batch"
            )));
        }
    })
}

async fn build_workspace(tx: &mut PgConnection, ws: Uuid) -> Result<Built> {
    let zerr = |e: interchange::Error| format_err(e.to_string());
    let mut zip = ZipWriter::new();
    fill_workspace(tx, ws, &mut zip, "").await?;
    Ok(Built {
        bytes: zip.finish().map_err(zerr)?,
        content_type: "application/zip",
        ext: "zip",
    })
}

/// Writes the files of the full workspace export into `zip`, each name under `prefix`.
async fn fill_workspace(
    tx: &mut PgConnection,
    ws: Uuid,
    zip: &mut ZipWriter,
    prefix: &str,
) -> Result<()> {
    let zerr = |e: interchange::Error| format_err(e.to_string());
    let name_of = |n: &str| format!("{prefix}{n}");
    let db_err = |e: sqlx::Error| Error::from(db::Error::from(e));

    // Recipes (the newest version of each).
    let ids: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM recipes WHERE deleted_at IS NULL ORDER BY created_at, id",
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(db_err)?;
    let mut recipes = Vec::new();
    for id in &ids {
        match load_recipe(tx, *id).await {
            Ok((r, _)) => recipes.push(r),
            // A recipe with no version cannot be written; the manifest counts only the written.
            Err(Error::Format(_)) => {}
            Err(e) => return Err(e),
        }
    }
    zip.add(
        &name_of("recipes.beerjson"),
        interchange::beerjson::write(&recipes).as_bytes(),
    )
    .map_err(zerr)?;
    zip.add(
        &name_of("recipes.beerxml"),
        interchange::beerxml::write(&recipes).as_bytes(),
    )
    .map_err(zerr)?;

    // Batches.
    let mut batches = csv::text_line(&[
        "code",
        "name",
        "stage",
        "recipe_version_id",
        "planned_date",
        "started_at",
        "completed_at",
        "target_volume_l",
        "actual_volume_l",
        "notes",
    ]);
    let rows = sqlx::query(
        "SELECT code, name, stage, recipe_version_id, planned_date::text AS planned, \
         started_at::text AS started, completed_at::text AS completed, target_volume_l, \
         actual_volume_l, notes FROM batches WHERE deleted_at IS NULL ORDER BY code",
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(db_err)?;
    let opt_text = |r: &sqlx::postgres::PgRow, k: &str| -> String {
        csv::cell(&r.get::<Option<String>, _>(k).unwrap_or_default())
    };
    let opt_num = |r: &sqlx::postgres::PgRow, k: &str| -> String {
        r.get::<Option<f64>, _>(k)
            .map(csv::number)
            .unwrap_or_default()
    };
    for r in &rows {
        batches.push_str(&csv::line(&[
            opt_text(r, "code"),
            opt_text(r, "name"),
            opt_text(r, "stage"),
            csv::cell(&r.get::<Uuid, _>("recipe_version_id").to_string()),
            opt_text(r, "planned"),
            opt_text(r, "started"),
            opt_text(r, "completed"),
            opt_num(r, "target_volume_l"),
            opt_num(r, "actual_volume_l"),
            opt_text(r, "notes"),
        ]));
    }
    zip.add(&name_of("batches.csv"), batches.as_bytes()).map_err(zerr)?;
    let batch_count = rows.len();

    // Stock lots.
    let mut stock = csv::text_line(&[
        "ingredient",
        "lot_code",
        "supplier",
        "received_on",
        "expires_on",
        "unit",
        "quantity",
        "unit_cost",
        "cost_currency",
        "notes",
    ]);
    let rows = sqlx::query(
        "SELECT i.name AS ingredient, l.lot_code, l.supplier, l.received_on::text AS received, \
         l.expires_on::text AS expires, l.unit, l.quantity, l.unit_cost::text AS cost, \
         l.cost_currency::text AS currency, l.notes FROM stock_lots l \
         JOIN ingredients i ON i.id = l.ingredient_id WHERE l.deleted_at IS NULL \
         ORDER BY i.name, l.lot_code",
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(db_err)?;
    for r in &rows {
        stock.push_str(&csv::line(&[
            opt_text(r, "ingredient"),
            opt_text(r, "lot_code"),
            opt_text(r, "supplier"),
            opt_text(r, "received"),
            opt_text(r, "expires"),
            opt_text(r, "unit"),
            opt_num(r, "quantity"),
            opt_text(r, "cost"),
            opt_text(r, "currency"),
            opt_text(r, "notes"),
        ]));
    }
    zip.add(&name_of("stock_lots.csv"), stock.as_bytes()).map_err(zerr)?;

    // Equipment and own ingredients as JSON.
    let equipment: Value = sqlx::query_scalar(
        "SELECT COALESCE(jsonb_agg(to_jsonb(e) - ARRAY['workspace_id','created_by','updated_by','deleted_at'] \
         ORDER BY e.created_at), '[]'::jsonb) FROM equipment e WHERE e.deleted_at IS NULL",
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(db_err)?;
    zip.add(&name_of("equipment.json"), &json_bytes(&equipment))
        .map_err(zerr)?;
    let ingredients: Value = sqlx::query_scalar(
        "SELECT COALESCE(jsonb_agg(to_jsonb(i) - ARRAY['workspace_id','search','created_by','updated_by','deleted_at'] \
         ORDER BY i.name), '[]'::jsonb) FROM ingredients i \
         WHERE i.workspace_id = $1 AND i.deleted_at IS NULL",
    )
    .bind(ws)
    .fetch_one(&mut *tx)
    .await
    .map_err(db_err)?;
    zip.add(&name_of("ingredients.json"), &json_bytes(&ingredients))
        .map_err(zerr)?;

    // Measurements: one CSV for each batch and year.
    let mut files: BTreeMap<String, String> = BTreeMap::new();
    let mut total = 0usize;
    let mut measurement_count = 0usize;
    {
        let mut rows = sqlx::query(
            "SELECT b.code, m.measured_at, m.kind, m.value, m.source, m.device_id \
             FROM measurements m JOIN batches b ON b.id = m.batch_id AND b.workspace_id = m.workspace_id \
             WHERE b.deleted_at IS NULL ORDER BY b.code, m.measured_at, m.id",
        )
        .fetch(&mut *tx);
        let mut current: Option<(String, i32, String)> = None; // (code, year, file name)
        while let Some(row) = rows.next().await {
            let row = row.map_err(db_err)?;
            let code: String = row.get("code");
            let at: DateTime<Utc> = row.get("measured_at");
            let year = chrono::Datelike::year(&at);
            if current
                .as_ref()
                .is_none_or(|(c, y, _)| *c != code || *y != year)
            {
                let base = format!("{prefix}measurements/{}-{year}", safe_name(&code));
                let mut name = format!("{base}.csv");
                let mut n = 2;
                while files.contains_key(&name) {
                    name = format!("{base}-{n}.csv");
                    n += 1;
                }
                files.insert(name.clone(), measurement_header());
                current = Some((code, year, name));
            }
            let (_, _, name) = current.as_ref().ok_or_else(|| format_err("no file"))?;
            let line = measurement_line(
                at,
                &row.get::<String, _>("kind"),
                row.get("value"),
                &row.get::<String, _>("source"),
                row.get("device_id"),
            );
            total += line.len();
            if total > MAX_BYTES {
                return Err(format_err("export is too large"));
            }
            if let Some(f) = files.get_mut(name) {
                f.push_str(&line);
            }
            measurement_count += 1;
        }
    }
    let measurement_files: Vec<String> = files.keys().cloned().collect();
    for (name, text) in &files {
        zip.add(name, text.as_bytes()).map_err(zerr)?;
    }
    // Water profiles, devices (no key digest), support requests (no internal notes) and the
    // metadata of documents and attachments (T2.31).
    let water: Value = sqlx::query_scalar(
        "SELECT COALESCE(jsonb_agg(to_jsonb(w) - ARRAY['workspace_id','created_by','updated_by','deleted_at'] \
         ORDER BY w.name), '[]'::jsonb) FROM water_profiles w WHERE w.deleted_at IS NULL",
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(db_err)?;
    zip.add(&name_of("water_profiles.json"), &json_bytes(&water))
        .map_err(zerr)?;
    let devices: Value = sqlx::query_scalar(
        "SELECT COALESCE(jsonb_agg(to_jsonb(d) - ARRAY['workspace_id','ingest_key_hash','created_by','updated_by','deleted_at'] \
         ORDER BY d.name), '[]'::jsonb) FROM devices d WHERE d.deleted_at IS NULL",
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(db_err)?;
    zip.add(&name_of("devices.json"), &json_bytes(&devices))
        .map_err(zerr)?;
    let support: Value = sqlx::query_scalar(
        "SELECT COALESCE(jsonb_agg((to_jsonb(r) - ARRAY['workspace_id','created_by','updated_by','deleted_at'] \
            ) || jsonb_build_object('messages', (SELECT COALESCE(jsonb_agg( \
                to_jsonb(m) - ARRAY['workspace_id','created_by','updated_by','deleted_at','visibility','author_staff_id'] \
                ORDER BY m.created_at), '[]'::jsonb) FROM support_messages m \
                WHERE m.request_id = r.id AND m.visibility = 'customer' AND m.deleted_at IS NULL)) \
            ORDER BY r.public_number), '[]'::jsonb) FROM support_requests r WHERE r.deleted_at IS NULL",
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(db_err)?;
    zip.add(&name_of("support_requests.json"), &json_bytes(&support))
        .map_err(zerr)?;
    let documents: Value = sqlx::query_scalar(
        "SELECT jsonb_build_object( \
           'scanned_documents', (SELECT COALESCE(jsonb_agg(to_jsonb(d) - ARRAY['workspace_id','created_by','updated_by','deleted_at'] \
               ORDER BY d.created_at), '[]'::jsonb) FROM scanned_documents d WHERE d.deleted_at IS NULL), \
           'attachments', (SELECT COALESCE(jsonb_agg(to_jsonb(a) - ARRAY['workspace_id','created_by','updated_by','deleted_at'] \
               ORDER BY a.created_at), '[]'::jsonb) FROM attachments a WHERE a.deleted_at IS NULL))",
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(db_err)?;
    zip.add(&name_of("documents.json"), &json_bytes(&documents))
        .map_err(zerr)?;
    let manifest = json!({
        "schema_version": 1,
        "generated_at": Utc::now(),
        "counts": {"recipes": recipes.len(), "batches": batch_count,
                   "measurements": measurement_count, "measurement_files": measurement_files.len()},
        "files": ["recipes.beerjson", "recipes.beerxml", "batches.csv", "stock_lots.csv",
                  "equipment.json", "ingredients.json", "water_profiles.json", "devices.json",
                  "support_requests.json", "documents.json", "measurements/"],
        "not_included_yet": ["photos"],
    });
    zip.add(&name_of("manifest.json"), &json_bytes(&manifest))
        .map_err(zerr)?;
    Ok(())
}

/// The personal slice of a workspace: rows that belong to `user`.
async fn fill_personal(
    tx: &mut PgConnection,
    user: Uuid,
    zip: &mut ZipWriter,
    prefix: &str,
) -> Result<()> {
    let zerr = |e: interchange::Error| format_err(e.to_string());
    let db_err = |e: sqlx::Error| Error::from(db::Error::from(e));
    let drop = "ARRAY['workspace_id','created_by','updated_by','deleted_at']";
    let slices: [(&str, String); 4] = [
        (
            "support_requests.json",
            format!(
                "SELECT COALESCE(jsonb_agg((to_jsonb(r) - {drop}) || jsonb_build_object('messages', \
                 (SELECT COALESCE(jsonb_agg(to_jsonb(m) - ARRAY['workspace_id','created_by','updated_by','deleted_at','visibility','author_staff_id'] \
                  ORDER BY m.created_at), '[]'::jsonb) FROM support_messages m \
                  WHERE m.request_id = r.id AND m.visibility = 'customer' AND m.deleted_at IS NULL)) \
                 ORDER BY r.public_number), '[]'::jsonb) FROM support_requests r \
                 WHERE r.created_by = $1 AND r.deleted_at IS NULL"
            ),
        ),
        (
            "settings.json",
            format!(
                "SELECT COALESCE(jsonb_agg(to_jsonb(s) - {drop}), '[]'::jsonb) FROM user_settings s \
                 WHERE s.user_id = $1 AND s.deleted_at IS NULL"
            ),
        ),
        (
            "notification_preferences.json",
            format!(
                "SELECT COALESCE(jsonb_agg(to_jsonb(p) - {drop}), '[]'::jsonb) \
                 FROM notification_preferences p WHERE p.user_id = $1 AND p.deleted_at IS NULL"
            ),
        ),
        (
            "favorites.json",
            "SELECT COALESCE(jsonb_agg(to_jsonb(f) - ARRAY['workspace_id']), '[]'::jsonb) \
             FROM favorites f WHERE f.user_id = $1"
                .to_owned(),
        ),
    ];
    for (name, sql) in slices {
        let v: Value = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .bind(user)
            .fetch_one(&mut *tx)
            .await
            .map_err(db_err)?;
        zip.add(&format!("{prefix}me/{name}"), &json_bytes(&v))
            .map_err(zerr)?;
    }
    Ok(())
}

/// Builds the account data export: the account profile and, for each workspace of the user, the
/// personal slice; the whole workspace for a Home workspace where the user is the only
/// administrator.
pub async fn build_account_data(
    pool: &PgPool,
    store: &dyn ObjectStore,
    user: Uuid,
) -> Result<Built> {
    let zerr = |e: interchange::Error| format_err(e.to_string());
    let db_err = |e: sqlx::Error| Error::from(db::Error::from(e));
    let mut zip = ZipWriter::new();
    let profile: Value = sqlx::query_scalar("SELECT account_export_profile($1)")
        .bind(user)
        .fetch_one(pool)
        .await
        .map_err(db_err)?;
    zip.add("account.json", &json_bytes(&profile)).map_err(zerr)?;
    let members: Vec<(Uuid, String, bool, Uuid)> = sqlx::query_as(
        "SELECT workspace_id, mode, sole_admin, member_id FROM account_export_workspaces($1)",
    )
    .bind(user)
    .fetch_all(pool)
    .await
    .map_err(db_err)?;
    let mut listing = Vec::new();
    let mut total = 0usize;
    for (ws, mode, sole_admin, _) in &members {
        let ctx = DbContext {
            user_id: Some(user),
            workspace_id: Some(*ws),
            actor_kind: ActorKind::System,
        };
        let mut tx = db::begin(pool, &ctx).await?;
        let name: String = sqlx::query_scalar("SELECT name FROM workspaces WHERE id = $1")
            .bind(ws)
            .fetch_one(&mut *tx)
            .await
            .map_err(db_err)?;
        let prefix = format!("workspaces/{ws}/");
        fill_personal(&mut tx, user, &mut zip, &prefix).await?;
        let whole = mode == "home" && *sole_admin;
        let mut file_count = 0usize;
        if whole {
            fill_workspace(&mut tx, *ws, &mut zip, &prefix).await?;
            let keys: Vec<(Uuid, String)> = sqlx::query_as(
                "SELECT id, object_key FROM scanned_documents WHERE deleted_at IS NULL \
                 UNION ALL SELECT id, object_key FROM attachments WHERE deleted_at IS NULL",
            )
            .fetch_all(&mut *tx)
            .await
            .map_err(db_err)?;
            for (id, key) in keys {
                let Some(bytes) = store.get(&key, MAX_BYTES as u64).await? else {
                    continue;
                };
                total += bytes.len();
                if total > MAX_BYTES {
                    return Err(format_err("export is too large"));
                }
                let base = key.rsplit('/').next().unwrap_or("file");
                zip.add(&format!("{prefix}files/{id}-{}", safe_name(base)), &bytes)
                    .map_err(zerr)?;
                file_count += 1;
            }
        }
        tx.commit().await.map_err(db_err)?;
        listing.push(json!({"id": ws, "name": name, "mode": mode,
            "full_workspace_export": whole, "files": file_count}));
    }
    zip.add("workspaces.json", &json_bytes(&Value::Array(listing)))
        .map_err(zerr)?;
    let manifest = json!({
        "schema_version": 1,
        "generated_at": Utc::now(),
        "scope": "account_data",
        "files": ["account.json", "workspaces.json", "workspaces/<id>/me/", "workspaces/<id>/"],
    });
    zip.add("manifest.json", &json_bytes(&manifest)).map_err(zerr)?;
    Ok(Built {
        bytes: zip.finish().map_err(zerr)?,
        content_type: "application/zip",
        ext: "zip",
    })
}

fn context(ws: Uuid) -> DbContext {
    DbContext {
        user_id: None,
        workspace_id: Some(ws),
        actor_kind: ActorKind::System,
    }
}

async fn load_job(tx: &mut PgConnection, id: Uuid) -> Result<Job> {
    let row = sqlx::query("SELECT scope, scope_id, format, params, created_by FROM export_jobs WHERE id = $1")
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(db::Error::from)?;
    let params: Value = row.get("params");
    let time = |k: &str| {
        params
            .get(k)
            .and_then(Value::as_str)
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|t| t.with_timezone(&Utc))
    };
    Ok(Job {
        created_by: row.get("created_by"),
        scope: row.get("scope"),
        scope_id: row.get("scope_id"),
        format: row.get("format"),
        from: time("from"),
        to: time("to"),
    })
}

/// Builds the file of one claimed job and finishes the job row.
pub async fn process(pool: &PgPool, store: &dyn ObjectStore, id: Uuid, ws: Uuid) -> Result<()> {
    let outcome: Result<(String, usize)> = async {
        let mut tx = db::begin(pool, &context(ws)).await?;
        let job = load_job(&mut tx, id).await?;
        let built = match job.scope.as_str() {
            "recipe" => build_recipe(&mut tx, &job).await?,
            "batch" => build_batch(&mut tx, &job).await?,
            "workspace" => build_workspace(&mut tx, ws).await?,
            "account_data" => {
                let user = job
                    .created_by
                    .ok_or_else(|| format_err("account data job has no user"))?;
                // The data spans workspaces: it uses its own transactions.
                tx.commit().await.map_err(db::Error::from)?;
                let built = build_account_data(pool, store, user).await?;
                let key = format!("{ws}/exports/{id}.{}", built.ext);
                let size = built.bytes.len();
                store.put(&key, built.content_type, built.bytes).await?;
                return Ok((key, size));
            }
            other => return Err(format_err(format!("scope {other} is not supported"))),
        };
        tx.commit().await.map_err(db::Error::from)?;
        let key = format!("{ws}/exports/{id}.{}", built.ext);
        let size = built.bytes.len();
        store.put(&key, built.content_type, built.bytes).await?;
        Ok((key, size))
    }
    .await;
    let mut tx = db::begin(pool, &context(ws)).await?;
    match &outcome {
        Ok((key, size)) => {
            sqlx::query(
                "UPDATE export_jobs SET status = 'done', result_object_key = $2, size_bytes = $3, \
                 expires_at = now() + make_interval(secs => $4), error = NULL WHERE id = $1",
            )
            .bind(id)
            .bind(key)
            .bind(i64::try_from(*size).unwrap_or(i64::MAX))
            .bind(KEEP.as_secs_f64())
            .execute(&mut *tx)
            .await
            .map_err(db::Error::from)?;
        }
        Err(e) => {
            let mut text = e.to_string();
            text.truncate(300);
            sqlx::query("UPDATE export_jobs SET status = 'failed', error = $2 WHERE id = $1")
                .bind(id)
                .bind(text)
                .execute(&mut *tx)
                .await
                .map_err(db::Error::from)?;
        }
    }
    tx.commit().await.map_err(db::Error::from)?;
    outcome.map(|_| ())
}

/// Claims and processes queued jobs until none is left. Returns the number of jobs handled.
pub async fn run_pending(pool: &PgPool, store: &dyn ObjectStore) -> Result<usize> {
    let mut count = 0;
    loop {
        let claimed: Option<(Uuid, Uuid)> =
            sqlx::query_as("SELECT id, workspace_id FROM claim_export_job()")
                .fetch_optional(pool)
                .await
                .map_err(db::Error::from)?;
        let Some((id, ws)) = claimed else {
            return Ok(count);
        };
        count += 1;
        if let Err(e) = process(pool, store, id, ws).await {
            tracing::warn!(job = %id, error = %e, "export job failed");
        }
    }
}

/// Marks the expired exports and deletes their files. Returns the number of files handled.
pub async fn expire(pool: &PgPool, store: &dyn ObjectStore) -> Result<usize> {
    let rows: Vec<(Uuid, Uuid, Option<String>)> =
        sqlx::query_as("SELECT id, workspace_id, result_object_key FROM expire_export_jobs(100)")
            .fetch_all(pool)
            .await
            .map_err(db::Error::from)?;
    for (_, _, key) in &rows {
        if let Some(key) = key {
            store.delete(key).await?;
        }
    }
    Ok(rows.len())
}

/// Runs the job loop until `stop` turns `true`.
pub async fn run(pool: PgPool, store: Arc<dyn ObjectStore>, mut stop: watch::Receiver<bool>) {
    let mut last_expire = std::time::Instant::now();
    loop {
        if let Err(e) = run_pending(&pool, store.as_ref()).await {
            tracing::warn!(error = %e, "export job loop failed");
        }
        if last_expire.elapsed() > Duration::from_hours(1) {
            last_expire = std::time::Instant::now();
            if let Err(e) = expire(&pool, store.as_ref()).await {
                tracing::warn!(error = %e, "export expiry failed");
            }
        }
        tokio::select! {
            _ = stop.wait_for(|stopped| *stopped) => return,
            () = tokio::time::sleep(POLL) => {}
        }
    }
}
