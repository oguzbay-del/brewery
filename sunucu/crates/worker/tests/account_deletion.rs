//! Scheduled account deletion (T2.31c): workspace purge, object cleanup and anonymisation.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::too_many_lines
)]

mod common;

use std::{collections::HashMap, sync::Mutex, time::Duration};

use common::{api_pool, seed, worker_pool};
use sqlx::PgPool;
use storage::{
    BoxFuture, Digest, ObjectMeta, ObjectStore, PresignedRequest, StorageError, StorageResult,
    UploadLimits,
};
use uuid::Uuid;

#[derive(Default)]
struct MemoryStore {
    objects: Mutex<HashMap<String, Vec<u8>>>,
}

impl MemoryStore {
    fn insert(&self, key: &str, bytes: &[u8]) {
        self.objects
            .lock()
            .unwrap()
            .insert(key.to_owned(), bytes.to_vec());
    }

    fn contains(&self, key: &str) -> bool {
        self.objects.lock().unwrap().contains_key(key)
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

async fn create_due_request(base: &PgPool, user_id: Uuid) -> Uuid {
    let api = api_pool(base).await;
    let mut tx = db::begin(
        &api,
        &db::DbContext {
            user_id: Some(user_id),
            workspace_id: None,
            actor_kind: db::ActorKind::User,
        },
    )
    .await
    .unwrap();
    let request = db::request_account_deletion(&mut tx, 30).await.unwrap();
    tx.commit().await.unwrap();
    sqlx::query(
        "UPDATE account_deletion_requests SET purge_at = now() - interval '1 second'
         WHERE id = $1",
    )
    .bind(request.id)
    .execute(base)
    .await
    .unwrap();
    request.id
}

async fn create_business_workspace(base: &PgPool, user_id: Uuid) -> (Uuid, Uuid, Uuid) {
    let workspace_id = Uuid::now_v7();
    let member_id = Uuid::now_v7();
    let recipe_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO workspaces (id, name, mode, region, country, time_zone, support_pseudonym,
                                 created_by, updated_by)
         VALUES ($1, 'Business', 'business', 'us', 'US', 'America/New_York', $2, $3, $3)",
    )
    .bind(workspace_id)
    .bind(format!("pseudonym-{workspace_id}"))
    .bind(user_id)
    .execute(base)
    .await
    .unwrap();
    sqlx::query("INSERT INTO workspace_members (id, workspace_id, user_id) VALUES ($1, $2, $3)")
        .bind(member_id)
        .bind(workspace_id)
        .bind(user_id)
        .execute(base)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO recipes (id, workspace_id, name, created_by, updated_by)
         VALUES ($1, $2, 'Business recipe', $3, $3)",
    )
    .bind(recipe_id)
    .bind(workspace_id)
    .bind(user_id)
    .execute(base)
    .await
    .unwrap();
    (workspace_id, member_id, recipe_id)
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn due_deletion_purges_home_files_removes_business_membership_and_anonymizes(base: PgPool) {
    let s = seed(&base).await;
    let (business_ws, _business_member, recipe_id) = create_business_workspace(&base, s.user).await;
    let object_key = format!("{}/exports/private.zip", s.ws);
    sqlx::query(
        "INSERT INTO export_jobs (id, workspace_id, scope, format, params, status, result_object_key)
         VALUES ($1, $2, 'workspace', 'zip', '{}'::jsonb, 'done', $3)",
    )
    .bind(Uuid::now_v7())
    .bind(s.ws)
    .bind(&object_key)
    .execute(&base)
    .await
    .unwrap();

    let request_id = create_due_request(&base, s.user).await;
    let store = MemoryStore::default();
    store.insert(&object_key, b"private-home-export");
    let worker = worker_pool(&base).await;

    worker::maintenance::tick_with_files(&worker, None, Some(&store)).await;
    // The hourly claim prevents a second tick from writing another run row.
    worker::maintenance::tick_with_files(&worker, None, Some(&store)).await;

    let status: String =
        sqlx::query_scalar("SELECT status FROM account_deletion_requests WHERE id = $1")
            .bind(request_id)
            .fetch_one(&base)
            .await
            .unwrap();
    assert_eq!(status, "done");

    let home_exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM workspaces WHERE id = $1)")
            .bind(s.ws)
            .fetch_one(&base)
            .await
            .unwrap();
    assert!(!home_exists);
    let tombstones: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM workspace_tombstones WHERE former_workspace_id = $1",
    )
    .bind(s.ws)
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!(tombstones, 1);
    assert!(!store.contains(&object_key));

    let business_exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM workspaces WHERE id = $1)")
            .bind(business_ws)
            .fetch_one(&base)
            .await
            .unwrap();
    assert!(business_exists);
    let membership_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM workspace_members WHERE workspace_id = $1 AND user_id = $2",
    )
    .bind(business_ws)
    .bind(s.user)
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!(membership_count, 0);

    let (recipe_name, created_by, updated_by): (String, Option<Uuid>, Option<Uuid>) =
        sqlx::query_as("SELECT name, created_by, updated_by FROM recipes WHERE id = $1")
            .bind(recipe_id)
            .fetch_one(&base)
            .await
            .unwrap();
    assert_eq!(recipe_name, "Business recipe");
    assert_eq!(created_by, None);
    assert_eq!(updated_by, None);

    // Check every workspace table's user_id/created_by columns, not only the seeded recipe.
    let columns: Vec<(String, String)> = sqlx::query_as(
        "SELECT c.table_name, c.column_name
         FROM information_schema.columns c
         WHERE c.table_schema = 'public'
           AND c.column_name IN ('user_id', 'created_by')
           AND EXISTS (
               SELECT 1 FROM information_schema.columns w
               WHERE w.table_schema = c.table_schema
                 AND w.table_name = c.table_name
                 AND w.column_name = 'workspace_id'
           )
         ORDER BY c.table_name, c.column_name",
    )
    .fetch_all(&base)
    .await
    .unwrap();
    for (table, column) in columns {
        let sql = format!(
            "SELECT count(*) FROM \"{table}\" WHERE workspace_id = $1 AND \"{column}\" = $2"
        );
        let remaining: i64 = sqlx::query_scalar(&sql)
            .bind(business_ws)
            .bind(s.user)
            .fetch_one(&base)
            .await
            .unwrap();
        assert_eq!(remaining, 0, "{table}.{column} still references deleted user");
    }

    let deleted_at: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT deleted_at FROM users WHERE id = $1")
            .bind(s.user)
            .fetch_one(&base)
            .await
            .unwrap();
    assert!(deleted_at.is_some());

    let mut conn = base.acquire().await.unwrap();
    let chain = db::verify_chain(&mut conn, business_ws).await.unwrap();
    assert!(chain.is_ok(), "{chain:?}");

    let runs: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM maintenance_runs WHERE job = 'account-deletion-purge'",
    )
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!(runs, 1);
}
