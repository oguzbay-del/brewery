//! Export jobs (T2.26): the worker builds the files from real rows, marks the job done with an
//! expiry time and later deletes the expired files. Needs PostgreSQL (`DATABASE_URL`). The
//! object store is an in-memory implementation of `ObjectStore`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::too_many_lines,
    clippy::float_cmp
)]

use std::{collections::HashMap, sync::Mutex, time::Duration};

use sqlx::{PgPool, postgres::PgPoolOptions, types::Uuid};
use storage::{
    BoxFuture, Digest, ObjectMeta, ObjectStore, PresignedRequest, StorageError, StorageResult,
    UploadLimits,
};

#[derive(Default)]
struct MemoryStore {
    objects: Mutex<HashMap<String, Vec<u8>>>,
}

impl MemoryStore {
    fn get_bytes(&self, key: &str) -> Option<Vec<u8>> {
        self.objects.lock().unwrap().get(key).cloned()
    }
}

impl ObjectStore for MemoryStore {
    fn presign_put(
        &self,
        _: &str,
        _: &UploadLimits,
        _: Duration,
    ) -> StorageResult<PresignedRequest> {
        Err(StorageError::Config("not supported".into()))
    }
    fn presign_get(&self, _: &str, _: Duration) -> StorageResult<PresignedRequest> {
        Err(StorageError::Config("not supported".into()))
    }
    fn head<'a>(&'a self, _: &'a str) -> BoxFuture<'a, StorageResult<Option<ObjectMeta>>> {
        Box::pin(async { Ok(None) })
    }
    fn digest<'a>(&'a self, _: &'a str, _: u64) -> BoxFuture<'a, StorageResult<Option<Digest>>> {
        Box::pin(async { Ok(None) })
    }
    fn put<'a>(
        &'a self,
        key: &'a str,
        _: &'a str,
        body: Vec<u8>,
    ) -> BoxFuture<'a, StorageResult<()>> {
        Box::pin(async move {
            self.objects.lock().unwrap().insert(key.to_owned(), body);
            Ok(())
        })
    }
    fn get<'a>(&'a self, key: &'a str, _: u64) -> BoxFuture<'a, StorageResult<Option<Vec<u8>>>> {
        Box::pin(async move { Ok(self.objects.lock().unwrap().get(key).cloned()) })
    }
    fn put_locked<'a>(
        &'a self,
        key: &'a str,
        _: &'a str,
        body: Vec<u8>,
        _: chrono::DateTime<chrono::Utc>,
    ) -> BoxFuture<'a, StorageResult<()>> {
        Box::pin(async move {
            self.objects.lock().unwrap().insert(key.to_owned(), body);
            Ok(())
        })
    }
    fn delete<'a>(&'a self, key: &'a str) -> BoxFuture<'a, StorageResult<()>> {
        Box::pin(async move {
            self.objects.lock().unwrap().remove(key);
            Ok(())
        })
    }
    fn ensure_bucket(&self) -> BoxFuture<'_, StorageResult<()>> {
        Box::pin(async { Ok(()) })
    }
}

async fn worker_pool(base: &PgPool) -> PgPool {
    PgPoolOptions::new()
        .max_connections(4)
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("SET ROLE app_worker").execute(conn).await?;
                Ok(())
            })
        })
        .connect_with((*base.connect_options()).clone())
        .await
        .unwrap()
}

struct Seed {
    ws: Uuid,
    recipe: Uuid,
    batch: Uuid,
}

async fn exec(base: &PgPool, sql: &'static str, binds: &[Uuid]) {
    let mut q = sqlx::query(sql);
    for b in binds {
        q = q.bind(*b);
    }
    q.execute(base).await.unwrap();
}

/// A workspace with one published recipe, one batch and measurements in two years. The batch
/// name tries a formula injection.
async fn seed(base: &PgPool) -> Seed {
    let ws = Uuid::now_v7();
    let user = Uuid::now_v7();
    let (recipe, version, batch) = (Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7());
    exec(
        base,
        "INSERT INTO workspaces (id, name, mode, region, country, time_zone, support_pseudonym)
         VALUES ($1, 'W', 'home', 'us', 'US', 'America/New_York', 'pseudonym-' || $1::text)",
        &[ws],
    )
    .await;
    exec(
        base,
        "INSERT INTO users (id, display_name) VALUES ($1, 'T')",
        &[user],
    )
    .await;
    exec(
        base,
        "INSERT INTO recipes (id, workspace_id, name, notes) VALUES ($1, $2, 'Export Pale', 'a, b')",
        &[recipe, ws],
    )
    .await;
    exec(
        base,
        "INSERT INTO recipe_versions (id, workspace_id, recipe_id, number, status, batch_volume_l,
            efficiency_pct, target_og_sg)
         VALUES ($1, $2, $3, 1, 'draft', 20, 72, 1.050)",
        &[version, ws, recipe],
    )
    .await;
    exec(
        base,
        "INSERT INTO recipe_fermentables (id, workspace_id, recipe_version_id, position, name,
            amount_kg, grain_type, yield_pct) VALUES (gen_random_uuid(), $2, $1, 0, 'Pale malt',
            4.5, 'grain', 80)",
        &[version, ws],
    )
    .await;
    exec(
        base,
        "INSERT INTO recipe_hops (id, workspace_id, recipe_version_id, position, name, amount_kg,
            alpha_pct, form, usage, time_min) VALUES (gen_random_uuid(), $2, $1, 0, 'Cascade',
            0.03, 5.5, 'pellet', 'boil', 60)",
        &[version, ws],
    )
    .await;
    // Publish after the lines exist (a published version is immutable).
    exec(
        base,
        "UPDATE recipe_versions SET status = 'approved', approved_by = $2, approved_at = now() WHERE id = $1",
        &[version, user],
    )
    .await;
    exec(
        base,
        "UPDATE recipes SET approved_version_id = $2 WHERE id = $1",
        &[recipe, version],
    )
    .await;
    exec(
        base,
        "INSERT INTO batches (id, workspace_id, code, name, recipe_version_id)
         VALUES ($1, $2, 'B-0001', '=HYPERLINK(\"http://x\")', $3)",
        &[batch, ws, version],
    )
    .await;
    sqlx::query("SELECT create_measurement_partition('2027-12-01'), create_measurement_partition('2028-01-01')")
        .execute(base)
        .await
        .unwrap();
    for (at, v) in [
        ("2027-12-20T10:00:00Z", 1.050),
        ("2027-12-25T10:00:00Z", 1.030),
        ("2028-01-05T10:00:00Z", 1.012),
    ] {
        sqlx::query(
            "INSERT INTO measurements (id, workspace_id, batch_id, kind, value, measured_at, source)
             VALUES (gen_random_uuid(), $1, $2, 'sg', $3, $4::timestamptz, 'manual')",
        )
        .bind(ws)
        .bind(batch)
        .bind(v)
        .bind(at)
        .execute(base)
        .await
        .unwrap();
    }
    Seed { ws, recipe, batch }
}

async fn queue(
    base: &PgPool,
    s: &Seed,
    scope: &str,
    scope_id: Option<Uuid>,
    format: &str,
    params: serde_json::Value,
) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO export_jobs (id, workspace_id, scope, scope_id, format, params)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(s.ws)
    .bind(scope)
    .bind(scope_id)
    .bind(format)
    .bind(params)
    .execute(base)
    .await
    .unwrap();
    id
}

async fn status(base: &PgPool, id: Uuid) -> (String, Option<String>) {
    sqlx::query_as("SELECT status, result_object_key FROM export_jobs WHERE id = $1")
        .bind(id)
        .fetch_one(base)
        .await
        .unwrap()
}

fn contains(haystack: &[u8], needle: &str) -> bool {
    haystack
        .windows(needle.len())
        .any(|w| w == needle.as_bytes())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn recipe_export_round_trips_through_beerjson(base: PgPool) {
    let s = seed(&base).await;
    let pool = worker_pool(&base).await;
    let store = MemoryStore::default();
    let job = queue(
        &base,
        &s,
        "recipe",
        Some(s.recipe),
        "beerjson",
        serde_json::json!({}),
    )
    .await;
    let xml = queue(
        &base,
        &s,
        "recipe",
        Some(s.recipe),
        "beerxml",
        serde_json::json!({}),
    )
    .await;
    assert_eq!(worker::export::run_pending(&pool, &store).await.unwrap(), 2);
    let (st, key) = status(&base, job).await;
    assert_eq!(st, "done");
    let bytes = store.get_bytes(&key.unwrap()).unwrap();
    let doc = interchange::beerjson::parse(&bytes).unwrap();
    assert_eq!(doc.recipes[0].name, "Export Pale");
    assert_eq!(doc.recipes[0].fermentables[0].amount_kg, 4.5);
    assert_eq!(doc.recipes[0].hops[0].name, "Cascade");
    assert_eq!(doc.recipes[0].source.og_sg, Some(1.05));
    let (_, key) = status(&base, xml).await;
    let doc = interchange::beerxml::parse(&store.get_bytes(&key.unwrap()).unwrap()).unwrap();
    assert_eq!(doc.recipes[0].hops[0].alpha_pct, Some(5.5));
    let expires: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT expires_at FROM export_jobs WHERE id = $1")
            .bind(job)
            .fetch_one(&base)
            .await
            .unwrap();
    assert!(expires.unwrap() > chrono::Utc::now() + chrono::Duration::days(6));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn full_export_has_one_measurement_file_for_each_batch_and_year(base: PgPool) {
    let s = seed(&base).await;
    let pool = worker_pool(&base).await;
    let store = MemoryStore::default();
    let job = queue(&base, &s, "workspace", None, "zip", serde_json::json!({})).await;
    worker::export::run_pending(&pool, &store).await.unwrap();
    let (st, key) = status(&base, job).await;
    assert_eq!(
        st,
        "done",
        "{:?}",
        sqlx::query_scalar::<_, Option<String>>("SELECT error FROM export_jobs WHERE id = $1")
            .bind(job)
            .fetch_one(&base)
            .await
    );
    let zip = store.get_bytes(&key.unwrap()).unwrap();
    assert_eq!(&zip[..4], b"PK\x03\x04");
    for name in [
        "recipes.beerjson",
        "recipes.beerxml",
        "batches.csv",
        "stock_lots.csv",
        "equipment.json",
        "ingredients.json",
        "manifest.json",
        "measurements/B-0001-2027.csv",
        "measurements/B-0001-2028.csv",
    ] {
        assert!(contains(&zip, name), "{name} is missing");
    }
    // Stored without compression: the cells are readable. The formula is escaped.
    assert!(contains(&zip, "'=HYPERLINK"));
    assert!(!contains(&zip, ",=HYPERLINK"));
    assert!(contains(&zip, "2027-12-25T10:00:00+00:00,sg,1.03,manual,"));
    assert!(contains(&zip, "\"measurements\":3") || contains(&zip, "\"measurements\": 3"));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn batch_export_filters_by_time_range_and_failures_are_recorded(base: PgPool) {
    let s = seed(&base).await;
    let pool = worker_pool(&base).await;
    let store = MemoryStore::default();
    let job = queue(
        &base,
        &s,
        "batch",
        Some(s.batch),
        "csv",
        serde_json::json!({"from": "2027-12-21T00:00:00Z", "to": "2027-12-31T00:00:00Z"}),
    )
    .await;
    let missing = queue(
        &base,
        &s,
        "recipe",
        Some(Uuid::now_v7()),
        "beerjson",
        serde_json::json!({}),
    )
    .await;
    worker::export::run_pending(&pool, &store).await.unwrap();
    let (_, key) = status(&base, job).await;
    let text = String::from_utf8(store.get_bytes(&key.unwrap()).unwrap()).unwrap();
    assert_eq!(text.lines().count(), 2, "{text}");
    assert!(text.contains("1.03"));
    let (st, key) = status(&base, missing).await;
    assert_eq!(st, "failed");
    assert!(key.is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn expired_files_are_deleted(base: PgPool) {
    let s = seed(&base).await;
    let pool = worker_pool(&base).await;
    let store = MemoryStore::default();
    let job = queue(
        &base,
        &s,
        "batch",
        Some(s.batch),
        "json",
        serde_json::json!({}),
    )
    .await;
    worker::export::run_pending(&pool, &store).await.unwrap();
    let (_, key) = status(&base, job).await;
    let key = key.unwrap();
    assert!(store.get_bytes(&key).is_some());
    assert_eq!(worker::export::expire(&pool, &store).await.unwrap(), 0);
    sqlx::query("UPDATE export_jobs SET expires_at = now() - interval '1 minute' WHERE id = $1")
        .bind(job)
        .execute(&base)
        .await
        .unwrap();
    assert_eq!(worker::export::expire(&pool, &store).await.unwrap(), 1);
    assert!(store.get_bytes(&key).is_none());
    assert_eq!(status(&base, job).await, ("expired".to_owned(), None));
}
