//! Upload object cleanup (T2.44). Needs PostgreSQL (DATABASE_URL).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::{collections::HashSet, sync::Mutex, time::Duration};

use chrono::{DateTime, Utc};
use sqlx::{PgPool, postgres::PgPoolOptions, types::Uuid};
use storage::{
    BoxFuture, Digest, ObjectMeta, ObjectStore, PresignedRequest, StorageError, StorageResult,
    UploadLimits,
};
use worker::upload_cleanup;

#[derive(Default)]
struct MemoryStore {
    objects: Mutex<HashSet<String>>,
    failures: Mutex<HashSet<String>>,
}

impl MemoryStore {
    fn insert(&self, key: &str) {
        self.objects.lock().unwrap().insert(key.to_owned());
    }

    fn fail(&self, key: &str) {
        self.failures.lock().unwrap().insert(key.to_owned());
    }

    fn contains(&self, key: &str) -> bool {
        self.objects.lock().unwrap().contains(key)
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

    fn put<'a>(&'a self, key: &'a str, _: &'a str, _: Vec<u8>) -> BoxFuture<'a, StorageResult<()>> {
        Box::pin(async move {
            self.objects.lock().unwrap().insert(key.to_owned());
            Ok(())
        })
    }

    fn get<'a>(&'a self, _: &'a str, _: u64) -> BoxFuture<'a, StorageResult<Option<Vec<u8>>>> {
        Box::pin(async { Ok(None) })
    }

    fn put_locked<'a>(
        &'a self,
        key: &'a str,
        content_type: &'a str,
        body: Vec<u8>,
        _: DateTime<Utc>,
    ) -> BoxFuture<'a, StorageResult<()>> {
        self.put(key, content_type, body)
    }

    fn delete<'a>(&'a self, key: &'a str) -> BoxFuture<'a, StorageResult<()>> {
        Box::pin(async move {
            if self.failures.lock().unwrap().contains(key) {
                return Err(StorageError::Config("injected delete failure".into()));
            }
            self.objects.lock().unwrap().remove(key);
            Ok(())
        })
    }

    fn ensure_bucket(&self) -> BoxFuture<'_, StorageResult<()>> {
        Box::pin(async { Ok(()) })
    }
}

async fn worker_pool(pool: &PgPool) -> PgPool {
    PgPoolOptions::new()
        .max_connections(4)
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("SET ROLE app_worker").execute(conn).await?;
                Ok(())
            })
        })
        .connect_with((*pool.connect_options()).clone())
        .await
        .unwrap()
}

async fn create_workspace(pool: &PgPool, workspace_id: Uuid) {
    sqlx::query(
        "INSERT INTO workspaces (id, name, mode, region, country, time_zone, support_pseudonym)
         VALUES ($1, 'Upload cleanup', 'business', 'us', 'US', 'America/New_York', $2)",
    )
    .bind(workspace_id)
    .bind(format!("pseudonym-{workspace_id}"))
    .execute(pool)
    .await
    .unwrap();
}

async fn create_upload_rows(
    pool: &PgPool,
    workspace_id: Uuid,
    base: u128,
    now: DateTime<Utc>,
) -> Vec<String> {
    let request_id = Uuid::from_u128(base + 1);
    let message_id = Uuid::from_u128(base + 2);
    let keys = vec![
        format!("{workspace_id}/attachments/deleted-attachment"),
        format!("{workspace_id}/attachments/incomplete-attachment"),
        format!("{workspace_id}/attachments/complete-attachment"),
        format!("{workspace_id}/attachments/retry-attachment"),
        format!("{workspace_id}/documents/deleted-document"),
        format!("{workspace_id}/documents/recent-document"),
        format!("{workspace_id}/documents/legal-hold-document"),
    ];
    sqlx::query(
        "INSERT INTO support_requests (id, workspace_id, topic, title)
         VALUES ($1, $2, 'recipes', 'Upload cleanup test')",
    )
    .bind(request_id)
    .bind(workspace_id)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO support_messages (id, workspace_id, request_id, author_kind,
            author_user_id, body, visibility)
         VALUES ($1, $2, $3, 'system', NULL, 'test', 'internal')",
    )
    .bind(message_id)
    .bind(workspace_id)
    .bind(request_id)
    .execute(pool)
    .await
    .unwrap();

    // A completed, soft-deleted attachment.
    sqlx::query(
        "INSERT INTO attachments (id, workspace_id, kind, record_kind, record_id, object_key,
            content_type, size_bytes, sha256, uploaded_at, created_at, deleted_at)
         VALUES ($1, $2, 'file', 'support_message', $3, $4,
            'text/plain', 4, decode(repeat('11', 32), 'hex'), $5, $6, $7)",
    )
    .bind(Uuid::from_u128(base + 3))
    .bind(workspace_id)
    .bind(message_id)
    .bind(&keys[0])
    .bind(now - chrono::Duration::days(3))
    .bind(now - chrono::Duration::days(3))
    .bind(now - chrono::Duration::days(2))
    .execute(pool)
    .await
    .unwrap();

    // An incomplete attachment older than 24 hours.
    sqlx::query(
        "INSERT INTO attachments (id, workspace_id, kind, record_kind, record_id, object_key,
            content_type, size_bytes, sha256, created_at)
         VALUES ($1, $2, 'file', 'support_message', $3, $4,
            'text/plain', 4, decode(repeat('22', 32), 'hex'), $5)",
    )
    .bind(Uuid::from_u128(base + 4))
    .bind(workspace_id)
    .bind(message_id)
    .bind(&keys[1])
    .bind(now - chrono::Duration::hours(25))
    .execute(pool)
    .await
    .unwrap();

    // A completed attachment that has not been deleted must remain.
    sqlx::query(
        "INSERT INTO attachments (id, workspace_id, kind, record_kind, record_id, object_key,
            content_type, size_bytes, sha256, uploaded_at, created_at)
         VALUES ($1, $2, 'file', 'support_message', $3, $4,
            'text/plain', 4, decode(repeat('33', 32), 'hex'), $5, $6)",
    )
    .bind(Uuid::from_u128(base + 5))
    .bind(workspace_id)
    .bind(message_id)
    .bind(&keys[2])
    .bind(now - chrono::Duration::days(2))
    .bind(now - chrono::Duration::days(2))
    .execute(pool)
    .await
    .unwrap();

    // A failed deletion remains eligible for the next run.
    sqlx::query(
        "INSERT INTO attachments (id, workspace_id, kind, record_kind, record_id, object_key,
            content_type, size_bytes, sha256, uploaded_at, created_at, deleted_at)
         VALUES ($1, $2, 'file', 'support_message', $3, $4,
            'text/plain', 4, decode(repeat('44', 32), 'hex'), $5, $6, $7)",
    )
    .bind(Uuid::from_u128(base + 6))
    .bind(workspace_id)
    .bind(message_id)
    .bind(&keys[3])
    .bind(now - chrono::Duration::days(3))
    .bind(now - chrono::Duration::days(3))
    .bind(now - chrono::Duration::days(2))
    .execute(pool)
    .await
    .unwrap();

    // A soft-deleted scanned document.
    sqlx::query(
        "INSERT INTO scanned_documents (id, workspace_id, form_type, object_key,
            content_type, size_bytes, sha256, uploaded_at, created_at, deleted_at)
         VALUES ($1, $2, 'brew_log', $3,
            'application/pdf', 4, decode(repeat('55', 32), 'hex'), $4, $5, $6)",
    )
    .bind(Uuid::from_u128(base + 7))
    .bind(workspace_id)
    .bind(&keys[4])
    .bind(now - chrono::Duration::days(3))
    .bind(now - chrono::Duration::days(3))
    .bind(now - chrono::Duration::days(2))
    .execute(pool)
    .await
    .unwrap();

    // An incomplete upload younger than 24 hours must stay.
    sqlx::query(
        "INSERT INTO scanned_documents (id, workspace_id, form_type, object_key,
            content_type, size_bytes, sha256, created_at)
         VALUES ($1, $2, 'brew_log', $3,
            'application/pdf', 4, decode(repeat('66', 32), 'hex'), $4)",
    )
    .bind(Uuid::from_u128(base + 8))
    .bind(workspace_id)
    .bind(&keys[5])
    .bind(now - chrono::Duration::hours(23))
    .execute(pool)
    .await
    .unwrap();

    // An incomplete scanned document under a live legal hold must stay.
    sqlx::query(
        "INSERT INTO scanned_documents (id, workspace_id, form_type, object_key,
            content_type, size_bytes, sha256, legal_hold, retention_until, created_at)
         VALUES ($1, $2, 'brew_log', $3,
            'application/pdf', 4, decode(repeat('77', 32), 'hex'), true, $4, $5)",
    )
    .bind(Uuid::from_u128(base + 9))
    .bind(workspace_id)
    .bind(&keys[6])
    .bind(now + chrono::Duration::days(30))
    .bind(now - chrono::Duration::days(2))
    .execute(pool)
    .await
    .unwrap();

    keys
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn cleanup_deletes_only_eligible_objects_and_retries_failures(pool: PgPool) {
    let now = Utc::now();
    let workspaces = [
        Uuid::from_u128(0x0192_f1c0_7a3b_7000_8000_0000_0000_0101),
        Uuid::from_u128(0x0192_f1c0_7a3b_7000_8000_0000_0000_0102),
    ];
    let mut keys = Vec::new();
    for (index, workspace_id) in workspaces.iter().enumerate() {
        create_workspace(&pool, *workspace_id).await;
        keys.extend(
            create_upload_rows(&pool, *workspace_id, 10_000 + (index as u128 * 100), now).await,
        );
    }

    let worker = worker_pool(&pool).await;
    let store = MemoryStore::default();
    for key in &keys {
        store.insert(key);
        if key.ends_with("/retry-attachment") {
            store.fail(key);
        }
    }

    let first = upload_cleanup::run(&worker, &store, now).await.unwrap();
    assert_eq!(first.deleted, 4);
    assert_eq!(first.incomplete, 2);
    assert_eq!(first.failed, 2);

    for suffix in [
        "/deleted-attachment",
        "/incomplete-attachment",
        "/deleted-document",
    ] {
        assert!(
            keys.iter()
                .filter(|key| key.ends_with(suffix))
                .all(|key| !store.contains(key)),
            "{suffix} objects should be deleted in both workspaces"
        );
    }
    for suffix in [
        "/complete-attachment",
        "/recent-document",
        "/legal-hold-document",
        "/retry-attachment",
    ] {
        assert!(
            keys.iter()
                .filter(|key| key.ends_with(suffix))
                .all(|key| store.contains(key)),
            "{suffix} objects should remain after the first run"
        );
    }

    let purged_attachments: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM attachments WHERE object_purged_at IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(purged_attachments, 4);

    let purged_documents: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM scanned_documents WHERE object_purged_at IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(purged_documents, 2);

    for (index, _) in workspaces.iter().enumerate() {
        let base = 10_000 + (index as u128 * 100);
        let (deleted_at_set, purged_at_set): (bool, bool) = sqlx::query_as(
            "SELECT deleted_at IS NOT NULL, object_purged_at IS NOT NULL
             FROM attachments WHERE id = $1",
        )
        .bind(Uuid::from_u128(base + 4))
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(deleted_at_set && purged_at_set, "old incomplete upload is soft-deleted and purged");

        let retry_purged: bool = sqlx::query_scalar(
            "SELECT object_purged_at IS NOT NULL FROM attachments WHERE id = $1",
        )
        .bind(Uuid::from_u128(base + 6))
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(!retry_purged, "failed delete must not be marked as purged");

        let legal_hold_purged: bool = sqlx::query_scalar(
            "SELECT object_purged_at IS NOT NULL FROM scanned_documents WHERE id = $1",
        )
        .bind(Uuid::from_u128(base + 9))
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(!legal_hold_purged, "active legal hold must not be touched");
    }

    // Remove the injected failures: the next run retries both failed objects.
    store.failures.lock().unwrap().clear();
    let second = upload_cleanup::run(&worker, &store, now).await.unwrap();
    assert_eq!(second.deleted, 2);
    assert_eq!(second.incomplete, 0);
    assert_eq!(second.failed, 0);
    assert!(
        keys.iter()
            .filter(|key| key.ends_with("/retry-attachment"))
            .all(|key| !store.contains(key))
    );

    let third = upload_cleanup::run(&worker, &store, now).await.unwrap();
    assert_eq!(third, Default::default());
}
