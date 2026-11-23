//! Scheduled account deletion contract (T2.31c): guarded DB functions and post-commit file retry.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

mod common;

use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
    time::Duration,
};

use common::{api_pool, seed, worker_pool};
use serde_json::Value;
use sqlx::PgPool;
use storage::{
    BoxFuture, Digest, ObjectMeta, ObjectStore, PresignedRequest, StorageError, StorageResult,
    UploadLimits,
};
use uuid::Uuid;

#[derive(Default)]
struct MemoryStore {
    objects: Mutex<HashMap<String, Vec<u8>>>,
    fail_once: Mutex<HashSet<String>>,
}

impl MemoryStore {
    fn insert(&self, key: &str, bytes: &[u8]) {
        self.objects
            .lock()
            .unwrap()
            .insert(key.to_owned(), bytes.to_vec());
    }

    fn fail_once(&self, key: &str) {
        self.fail_once.lock().unwrap().insert(key.to_owned());
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
            if self.fail_once.lock().unwrap().remove(key) {
                return Err(StorageError::Config(
                    "simulated object deletion failure".into(),
                ));
            }
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
        "UPDATE account_deletion_requests
         SET requested_at = now() - interval '2 seconds',
             purge_at = now() - interval '1 second'
         WHERE id = $1",
    )
    .bind(request.id)
    .execute(base)
    .await
    .unwrap();
    request.id
}

async fn create_business_workspace(
    base: &PgPool,
    user_id: Uuid,
    owner: bool,
) -> (Uuid, Uuid, Uuid) {
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
    sqlx::query(
        "INSERT INTO workspace_members (id, workspace_id, user_id, created_by, updated_by)
         VALUES ($1, $2, $3, $3, $3)",
    )
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

    if owner {
        let role_id = Uuid::now_v7();
        sqlx::query("INSERT INTO roles (id, workspace_id, name) VALUES ($1, $2, 'owner')")
            .bind(role_id)
            .bind(workspace_id)
            .execute(base)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO role_assignments (id, workspace_id, member_id, role_id, created_by, updated_by)
             VALUES ($1, $2, $3, $4, $5, $5)",
        )
        .bind(Uuid::now_v7())
        .bind(workspace_id)
        .bind(member_id)
        .bind(role_id)
        .bind(user_id)
        .execute(base)
        .await
        .unwrap();
    }
    (workspace_id, member_id, recipe_id)
}

async fn create_export_object(base: &PgPool, workspace_id: Uuid) -> String {
    let key = format!("{workspace_id}/exports/private.zip");
    sqlx::query(
        "INSERT INTO export_jobs (id, workspace_id, scope, format, params, status, result_object_key)
         VALUES ($1, $2, 'workspace', 'zip', '{}'::jsonb, 'done', $3)",
    )
    .bind(Uuid::now_v7())
    .bind(workspace_id)
    .bind(&key)
    .execute(base)
    .await
    .unwrap();
    key
}

async fn run_worker(base: &PgPool, store: Option<&dyn ObjectStore>) {
    let worker = worker_pool(base).await;
    worker::maintenance::tick_with_files(&worker, None, store).await;
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn due_deletion_purges_home_and_disables_business_membership_without_erasing_attribution(
    base: PgPool,
) {
    let s = seed(&base).await;
    let (business_ws, member_id, recipe_id) = create_business_workspace(&base, s.user, false).await;
    let object_key = create_export_object(&base, s.ws).await;
    let request_id = create_due_request(&base, s.user).await;
    let store = MemoryStore::default();
    store.insert(&object_key, b"private-home-export");

    run_worker(&base, Some(&store)).await;

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

    let (member_status, member_deleted_at): (String, Option<chrono::DateTime<chrono::Utc>>) =
        sqlx::query_as("SELECT status, deleted_at FROM workspace_members WHERE id = $1")
            .bind(member_id)
            .fetch_one(&base)
            .await
            .unwrap();
    assert_eq!(member_status, "disabled");
    assert!(member_deleted_at.is_some());

    let (recipe_created_by, recipe_updated_by): (Option<Uuid>, Option<Uuid>) =
        sqlx::query_as("SELECT created_by, updated_by FROM recipes WHERE id = $1")
            .bind(recipe_id)
            .fetch_one(&base)
            .await
            .unwrap();
    assert_eq!(recipe_created_by, Some(s.user));
    assert_eq!(recipe_updated_by, Some(s.user));

    let (workspace_created_by, workspace_updated_by): (Option<Uuid>, Option<Uuid>) =
        sqlx::query_as("SELECT created_by, updated_by FROM workspaces WHERE id = $1")
            .bind(business_ws)
            .fetch_one(&base)
            .await
            .unwrap();
    assert_eq!(workspace_created_by, Some(s.user));
    assert_eq!(workspace_updated_by, Some(s.user));

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

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn last_owner_stays_pending_and_worker_continues_with_the_next_due_user(base: PgPool) {
    let owner = seed(&base).await;
    let next_user = seed(&base).await;
    let (business_ws, owner_member, _) = create_business_workspace(&base, owner.user, true).await;
    let owner_request = create_due_request(&base, owner.user).await;
    let next_request = create_due_request(&base, next_user.user).await;

    run_worker(&base, None).await;

    let owner_status: String =
        sqlx::query_scalar("SELECT status FROM account_deletion_requests WHERE id = $1")
            .bind(owner_request)
            .fetch_one(&base)
            .await
            .unwrap();
    assert_eq!(owner_status, "pending");
    let (member_status, member_deleted_at): (String, Option<chrono::DateTime<chrono::Utc>>) =
        sqlx::query_as("SELECT status, deleted_at FROM workspace_members WHERE id = $1")
            .bind(owner_member)
            .fetch_one(&base)
            .await
            .unwrap();
    assert_eq!(member_status, "active");
    assert!(member_deleted_at.is_none());
    let owner_home_exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM workspaces WHERE id = $1)")
            .bind(owner.ws)
            .fetch_one(&base)
            .await
            .unwrap();
    assert!(owner_home_exists);

    let next_status: String =
        sqlx::query_scalar("SELECT status FROM account_deletion_requests WHERE id = $1")
            .bind(next_request)
            .fetch_one(&base)
            .await
            .unwrap();
    assert_eq!(next_status, "done");
    let next_home_exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM workspaces WHERE id = $1)")
            .bind(next_user.ws)
            .fetch_one(&base)
            .await
            .unwrap();
    assert!(!next_home_exists);

    let detail: String = sqlx::query_scalar(
        "SELECT last_detail FROM maintenance_runs WHERE job = 'account-deletion-purge'",
    )
    .fetch_one(&base)
    .await
    .unwrap();
    let value: Value = serde_json::from_str(&detail).unwrap();
    assert_eq!(
        value["failed_users"][0]["code"].as_str(),
        Some("last_owner")
    );
    let business_exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM workspaces WHERE id = $1)")
            .bind(business_ws)
            .fetch_one(&base)
            .await
            .unwrap();
    assert!(business_exists);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn failed_object_delete_is_recorded_and_retried_on_the_next_run(base: PgPool) {
    let s = seed(&base).await;
    let object_key = create_export_object(&base, s.ws).await;
    let request_id = create_due_request(&base, s.user).await;
    let store = MemoryStore::default();
    store.insert(&object_key, b"private-home-export");
    store.fail_once(&object_key);

    run_worker(&base, Some(&store)).await;

    let status: String =
        sqlx::query_scalar("SELECT status FROM account_deletion_requests WHERE id = $1")
            .bind(request_id)
            .fetch_one(&base)
            .await
            .unwrap();
    assert_eq!(status, "done");
    assert!(store.contains(&object_key));

    let detail: String = sqlx::query_scalar(
        "SELECT last_detail FROM maintenance_runs WHERE job = 'account-deletion-purge'",
    )
    .fetch_one(&base)
    .await
    .unwrap();
    let value: Value = serde_json::from_str(&detail).unwrap();
    assert!(
        value["pending_object_keys"]
            .as_array()
            .unwrap()
            .iter()
            .any(|key| key.as_str() == Some(&object_key))
    );

    sqlx::query(
        "UPDATE maintenance_runs SET last_started_at = to_timestamp(0)
         WHERE job = 'account-deletion-purge'",
    )
    .execute(&base)
    .await
    .unwrap();
    run_worker(&base, Some(&store)).await;
    assert!(!store.contains(&object_key));

    let detail: String = sqlx::query_scalar(
        "SELECT last_detail FROM maintenance_runs WHERE job = 'account-deletion-purge'",
    )
    .fetch_one(&base)
    .await
    .unwrap();
    let value: Value = serde_json::from_str(&detail).unwrap();
    assert_eq!(value["pending_object_keys"].as_array().unwrap().len(), 0);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn worker_functions_are_not_callable_by_api_and_require_a_due_request(base: PgPool) {
    let s = seed(&base).await;
    let api = api_pool(&base).await;
    let denied = sqlx::query("SELECT * FROM account_deletion_memberships($1)")
        .bind(s.user)
        .fetch_all(&api)
        .await;
    assert!(
        denied.is_err(),
        "app_api must not execute worker-only functions"
    );

    let worker = worker_pool(&base).await;
    let mut tx = worker.begin().await.unwrap();
    let not_due = db::account_deletion_memberships(&mut tx, s.user).await;
    assert!(
        not_due.is_err(),
        "a user without a due request must be refused"
    );
    tx.rollback().await.unwrap();
}
