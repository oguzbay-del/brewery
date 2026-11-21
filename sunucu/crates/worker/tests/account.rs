//! Account deletion and account data export (T2.31), end to end on a real database: the
//! request, the restore, the request again, the purge after the waiting time, and the checks that
//! nothing of the user stays, that the audit log names the user by UUID only and that the audit
//! chains still verify. The object store is in memory; Apple and the Issue tracker are fakes.
//! Needs PostgreSQL (`#[sqlx::test]`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::too_many_lines,
    clippy::items_after_statements
)]

mod common;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use account::{Envelope, FakeApple};
use common::{Seed, seed, worker_pool};
use db::{ActorKind, AuditEvent, DbContext, NewAuditEntry};
use serde_json::json;
use sqlx::{PgPool, types::Uuid};
use storage::{
    BoxFuture, Digest, ObjectMeta, ObjectStore, PresignedRequest, StorageError, StorageResult,
    UploadLimits,
};
use support::github::{GitHubError, IssueScrubber};
use worker::account::{Deps, purge_due};

#[derive(Default)]
struct MemoryStore {
    objects: Mutex<HashMap<String, Vec<u8>>>,
}

impl MemoryStore {
    fn keys(&self) -> Vec<String> {
        self.objects.lock().unwrap().keys().cloned().collect()
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

#[derive(Default)]
struct FakeIssues {
    scrubbed: Mutex<Vec<i64>>,
}

impl IssueScrubber for FakeIssues {
    fn scrub(&self, issue: i64) -> support::github::BoxFuture<'_, Result<(), GitHubError>> {
        Box::pin(async move {
            self.scrubbed.lock().unwrap().push(issue);
            Ok(())
        })
    }
}

struct Fakes {
    store: Arc<MemoryStore>,
    apple: Arc<FakeApple>,
    issues: Arc<FakeIssues>,
    envelope: Arc<Envelope>,
}

impl Fakes {
    fn new() -> Self {
        Self {
            store: Arc::default(),
            apple: Arc::default(),
            issues: Arc::default(),
            envelope: Arc::new(Envelope::from_bytes(&[5u8; 32]).unwrap()),
        }
    }
    fn deps(&self) -> Deps {
        Deps {
            store: Some(self.store.clone()),
            apple: Some(self.apple.clone()),
            envelope: Some(self.envelope.clone()),
            issues: Some(self.issues.clone()),
        }
    }
}

const NAME: &str = "Sam Brewer";

fn email_of(user: Uuid) -> String {
    format!("brewer-{}@example.com", user.simple())
}

async fn exec(base: &PgPool, sql: &str, binds: &[Uuid]) {
    let mut q = sqlx::query(sqlx::AssertSqlSafe(sql.to_owned()));
    for b in binds {
        q = q.bind(*b);
    }
    q.execute(base).await.unwrap();
}

/// Makes `member` of `ws` an administrator (`users.manage` for the whole workspace).
async fn make_admin(base: &PgPool, ws: Uuid, member: Uuid) {
    let role = Uuid::now_v7();
    exec(
        base,
        "INSERT INTO roles (id, workspace_id, name) VALUES ($1, $2, 'Owner ' || $1::text)",
        &[role, ws],
    )
    .await;
    exec(
        base,
        "INSERT INTO role_permissions (workspace_id, role_id, permission) VALUES ($2, $1, 'users.manage')",
        &[role, ws],
    )
    .await;
    exec(
        base,
        "INSERT INTO role_assignments (id, workspace_id, member_id, role_id) VALUES ($1, $2, $3, $4)",
        &[Uuid::now_v7(), ws, member, role],
    )
    .await;
}

/// The account rows of a user: identity with a sealed Apple token, passkey, session, push target.
async fn seed_account(base: &PgPool, fakes: &Fakes, user: Uuid) {
    exec(
        base,
        "UPDATE users SET display_name = 'Sam Brewer' WHERE id = $1",
        &[user],
    )
    .await;
    let sealed = fakes.envelope.seal(b"apple-refresh-token").unwrap();
    sqlx::query(
        "INSERT INTO user_identities (id, user_id, provider, subject, provider_refresh_token_enc) \
         VALUES ($1, $2, 'apple', $3, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(user)
    .bind(format!("apple-{user}"))
    .bind(sealed)
    .execute(base)
    .await
    .unwrap();
    exec(
        base,
        "INSERT INTO user_identities (id, user_id, provider, subject) \
         VALUES ($1, $2, 'email', 'sam@example.com')",
        &[Uuid::now_v7(), user],
    )
    .await;
    sqlx::query(
        "INSERT INTO passkeys (id, user_id, credential_id, public_key, name) \
         VALUES ($1, $2, $3, '\\x01', 'Phone')",
    )
    .bind(Uuid::now_v7())
    .bind(user)
    .bind(user.as_bytes().to_vec())
    .execute(base)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO sessions (id, user_id, token_hash, client_kind, expires_at) \
         VALUES ($1, $2, $3, 'ios', now() + interval '1 day')",
    )
    .bind(Uuid::now_v7())
    .bind(user)
    .bind(vec![3u8; 32])
    .execute(base)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO push_targets (id, user_id, environment, token) VALUES ($1, $2, 'sandbox', $3)",
    )
    .bind(Uuid::now_v7())
    .bind(user)
    .bind(format!("push-{user}"))
    .execute(base)
    .await
    .unwrap();
}

/// A support request of `user` in `ws` with a message, an attachment and an Issue number.
async fn seed_support(
    base: &PgPool,
    store: &MemoryStore,
    ws: Uuid,
    user: Uuid,
    issue: i32,
) -> String {
    let (request, message) = (Uuid::now_v7(), Uuid::now_v7());
    sqlx::query(
        "INSERT INTO support_requests (id, workspace_id, topic, title, github_issue_number, created_by) \
         VALUES ($1, $2, 'other', 'My title', $3, $4)",
    )
    .bind(request)
    .bind(ws)
    .bind(issue)
    .bind(user)
    .execute(base)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO support_messages (id, workspace_id, request_id, author_kind, author_user_id, body, created_by) \
         VALUES ($1, $2, $3, 'customer', $4, 'Please help, I am Sam Brewer', $4)",
    )
    .bind(message)
    .bind(ws)
    .bind(request)
    .bind(user)
    .execute(base)
    .await
    .unwrap();
    let key = format!("{ws}/support/{message}.png");
    store.put(&key, "image/png", vec![1, 2, 3]).await.unwrap();
    sqlx::query(
        "INSERT INTO attachments (id, workspace_id, kind, record_kind, record_id, object_key, \
         content_type, size_bytes, sha256, created_by) \
         VALUES ($1, $2, 'photo', 'support_message', $3, $4, 'image/png', 3, $5, $6)",
    )
    .bind(Uuid::now_v7())
    .bind(ws)
    .bind(message)
    .bind(&key)
    .bind(vec![0u8; 32])
    .bind(user)
    .execute(base)
    .await
    .unwrap();
    key
}

async fn audit(base: &PgPool, ws: Uuid, user: Uuid, n: usize) {
    for i in 0..n {
        let mut tx = base.begin().await.unwrap();
        db::append_audit(
            &mut tx,
            ws,
            &NewAuditEntry {
                actor_kind: ActorKind::User,
                actor_id: Some(user),
                event: AuditEvent::RoleUpdate,
                record_kind: "role".into(),
                record_id: Some(Uuid::now_v7()),
                previous_value: Some(json!({"n": i, "name": db::changed_field()})),
                new_value: Some(json!({"n": i + 1, "name": db::changed_field()})),
                reason: None,
            },
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
    }
}

async fn count(base: &PgPool, sql: &str, id: Uuid) -> i64 {
    sqlx::query_scalar(sqlx::AssertSqlSafe(sql.to_owned()))
        .bind(id)
        .fetch_one(base)
        .await
        .unwrap()
}

/// Request, restore, request again (as the API does), then make the request due.
async fn request_restore_request(base: &PgPool, user: Uuid) {
    let ctx = DbContext::user(user, None);
    let mut tx = db::begin(base, &ctx).await.unwrap();
    let (first, created) = db::account::request(&mut tx, user).await.unwrap();
    assert!(created);
    let (again, created) = db::account::request(&mut tx, user).await.unwrap();
    assert!(!created);
    assert_eq!(first, again);
    tx.commit().await.unwrap();
    let days = (first.purge_at - first.requested_at).num_days();
    assert_eq!(days, db::account::RETENTION_DAYS);

    let mut tx = db::begin(base, &ctx).await.unwrap();
    assert!(db::account::restore(&mut tx, user).await.unwrap());
    assert!(db::account::pending(&mut tx, user).await.unwrap().is_none());
    assert!(!db::account::restore(&mut tx, user).await.unwrap());
    tx.commit().await.unwrap();

    let mut tx = db::begin(base, &ctx).await.unwrap();
    let (second, created) = db::account::request(&mut tx, user).await.unwrap();
    assert!(created);
    assert!(second.requested_at >= first.requested_at);
    tx.commit().await.unwrap();
}

async fn make_due(base: &PgPool, user: Uuid) {
    exec(
        base,
        "UPDATE account_deletions SET requested_at = now() - interval '31 days', \
         purge_at = now() - interval '1 day' WHERE user_id = $1",
        &[user],
    )
    .await;
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn home_account_is_deleted_for_good_with_everything_in_it(base: PgPool) {
    let fakes = Fakes::new();
    let s: Seed = seed(&base).await;
    make_admin(&base, s.ws, s.member).await;
    seed_account(&base, &fakes, s.user).await;
    let key = seed_support(&base, &fakes.store, s.ws, s.user, 17).await;
    let doc_key = format!("{}/documents/scan.jpg", s.ws);
    fakes
        .store
        .put(&doc_key, "image/jpeg", vec![7; 10])
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO scanned_documents (id, workspace_id, form_type, object_key, content_type, size_bytes, sha256) \
         VALUES (gen_random_uuid(), $1, 'brew_sheet', $2, 'image/jpeg', 10, decode(repeat('00', 32), 'hex'))",
    )
    .bind(s.ws)
    .bind(&doc_key)
    .execute(&base)
    .await
    .unwrap();
    let pool = worker_pool(&base).await;

    // Not due: nothing happens.
    request_restore_request(&base, s.user).await;
    assert_eq!(purge_due(&pool, &fakes.deps()).await.unwrap(), 0);
    assert_eq!(
        count(&base, "SELECT count(*) FROM workspaces WHERE id = $1", s.ws).await,
        1
    );

    make_due(&base, s.user).await;
    // Apple does not answer: the account stays complete and the failure is recorded.
    *fakes.apple.fail_revoke.lock().unwrap() = true;
    assert_eq!(purge_due(&pool, &fakes.deps()).await.unwrap(), 0);
    assert_eq!(
        count(
            &base,
            "SELECT count(*) FROM user_identities WHERE user_id = $1",
            s.user
        )
        .await,
        2
    );
    let (attempts, error): (i32, Option<String>) =
        sqlx::query_as("SELECT attempts, last_error FROM account_deletions WHERE user_id = $1")
            .bind(s.user)
            .fetch_one(&base)
            .await
            .unwrap();
    assert_eq!(attempts, 1);
    assert!(error.unwrap().contains("apple"));
    // A failed account waits before the next try.
    assert_eq!(purge_due(&pool, &fakes.deps()).await.unwrap(), 0);
    exec(&base, "UPDATE account_deletions SET last_attempt_at = now() - interval '1 hour' WHERE user_id = $1", &[s.user]).await;
    *fakes.apple.fail_revoke.lock().unwrap() = false;

    assert_eq!(purge_due(&pool, &fakes.deps()).await.unwrap(), 1);

    // Apple token revoked; the Issue scrubbed.
    assert_eq!(
        fakes.apple.revoked_tokens(),
        vec!["apple-refresh-token".to_owned()]
    );
    assert_eq!(*fakes.issues.scrubbed.lock().unwrap(), vec![17]);
    // No row of the user and no object stays.
    for table in ["user_identities", "passkeys", "sessions", "push_targets"] {
        let n = count(
            &base,
            &format!("SELECT count(*) FROM {table} WHERE user_id = $1"),
            s.user,
        )
        .await;
        assert_eq!(n, 0, "{table}");
    }
    for table in [
        "workspaces",
        "recipes",
        "batches",
        "workspace_members",
        "support_requests",
        "support_messages",
        "attachments",
        "audit_log",
        "scanned_documents",
    ] {
        let col = if table == "workspaces" {
            "id"
        } else {
            "workspace_id"
        };
        let n = count(
            &base,
            &format!("SELECT count(*) FROM {table} WHERE {col} = $1"),
            s.ws,
        )
        .await;
        assert_eq!(n, 0, "{table}");
    }
    assert!(
        fakes.store.keys().is_empty(),
        "objects left: {:?} ({key})",
        fakes.store.keys()
    );
    // The user row is a tombstone.
    let (name, email, deleted): (String, Option<String>, bool) = sqlx::query_as(
        "SELECT display_name, contact_email, deleted_at IS NOT NULL FROM users WHERE id = $1",
    )
    .bind(s.user)
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!((name.as_str(), email, deleted), ("", None, true));
    let purged: bool = sqlx::query_scalar(
        "SELECT purged_at IS NOT NULL FROM account_deletions WHERE user_id = $1",
    )
    .bind(s.user)
    .fetch_one(&base)
    .await
    .unwrap();
    assert!(purged);
    // The deletion record of the workspace names the user by UUID only.
    let tomb = count(
        &base,
        "SELECT count(*) FROM workspace_tombstones WHERE former_workspace_id = $1",
        s.ws,
    )
    .await;
    assert_eq!(tomb, 1);
    // Nothing is due any more.
    assert_eq!(purge_due(&pool, &fakes.deps()).await.unwrap(), 0);
    // No text anywhere holds the name or the address.
    let leaks: i64 = sqlx::query_scalar(
        "SELECT (SELECT count(*) FROM users u WHERE to_jsonb(u)::text ILIKE '%Sam Brewer%' OR to_jsonb(u)::text ILIKE $1) \
              + (SELECT count(*) FROM audit_log a WHERE to_jsonb(a)::text ILIKE '%Sam Brewer%' OR to_jsonb(a)::text ILIKE $1) \
              + (SELECT count(*) FROM support_messages m WHERE m.body ILIKE '%Sam Brewer%')",
    )
    .bind(format!("%{}%", email_of(s.user)))
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!(leaks, 0);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn business_workspace_keeps_its_records_and_the_audit_chain_stays_valid(base: PgPool) {
    let fakes = Fakes::new();
    let s = seed(&base).await;
    exec(
        &base,
        "UPDATE workspaces SET mode = 'business' WHERE id = $1",
        &[s.ws],
    )
    .await;
    // A second administrator.
    let (other, other_member) = (Uuid::now_v7(), Uuid::now_v7());
    exec(
        &base,
        "INSERT INTO users (id, display_name) VALUES ($1, 'Pat Partner')",
        &[other],
    )
    .await;
    exec(
        &base,
        "INSERT INTO workspace_members (id, workspace_id, user_id) VALUES ($1, $2, $3)",
        &[other_member, s.ws, other],
    )
    .await;
    make_admin(&base, s.ws, s.member).await;
    make_admin(&base, s.ws, other_member).await;
    seed_account(&base, &fakes, s.user).await;
    seed_support(&base, &fakes.store, s.ws, s.user, 21).await;
    // The other member's request stays.
    let their_key = seed_support(&base, &fakes.store, s.ws, other, 22).await;
    audit(&base, s.ws, s.user, 3).await;
    audit(&base, s.ws, other, 2).await;
    let rows_before = count(
        &base,
        "SELECT count(*) FROM audit_log WHERE workspace_id = $1",
        s.ws,
    )
    .await;
    let pool = worker_pool(&base).await;

    request_restore_request(&base, s.user).await;
    make_due(&base, s.user).await;
    assert_eq!(purge_due(&pool, &fakes.deps()).await.unwrap(), 1);

    // The workspace and its records stay.
    assert_eq!(
        count(&base, "SELECT count(*) FROM workspaces WHERE id = $1", s.ws).await,
        1
    );
    assert_eq!(
        count(
            &base,
            "SELECT count(*) FROM recipes WHERE workspace_id = $1",
            s.ws
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &base,
            "SELECT count(*) FROM batches WHERE workspace_id = $1",
            s.ws
        )
        .await,
        1
    );
    // The user's membership is closed, the other one is open.
    let active: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM workspace_members WHERE workspace_id = $1 AND deleted_at IS NULL AND status = 'active'",
    )
    .bind(s.ws)
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!(active, 1);
    // The user's support data is gone, the other member's data stays.
    assert_eq!(
        count(
            &base,
            "SELECT count(*) FROM support_requests WHERE created_by = $1",
            s.user
        )
        .await,
        0
    );
    assert_eq!(
        count(
            &base,
            "SELECT count(*) FROM support_requests WHERE created_by = $1",
            other
        )
        .await,
        1
    );
    assert_eq!(*fakes.issues.scrubbed.lock().unwrap(), vec![21]);
    assert_eq!(fakes.store.keys(), vec![their_key]);
    // Created-by references still point to the tombstone, which shows no personal data.
    let tomb: (String, Option<String>) =
        sqlx::query_as("SELECT display_name, contact_email FROM users WHERE id = $1")
            .bind(s.user)
            .fetch_one(&base)
            .await
            .unwrap();
    assert_eq!(tomb, (String::new(), None));
    let by: i64 = count(
        &base,
        "SELECT count(*) FROM recipe_versions WHERE approved_by = $1",
        s.user,
    )
    .await;
    assert_eq!(by, 1);

    // The audit log keeps its rows (one more row says "purged"), names the user by UUID only
    // and the chain still verifies.
    let rows_after = count(
        &base,
        "SELECT count(*) FROM audit_log WHERE workspace_id = $1",
        s.ws,
    )
    .await;
    assert_eq!(rows_after, rows_before + 1);
    let text: String = sqlx::query_scalar(
        "SELECT string_agg(to_jsonb(a)::text, ' ') FROM audit_log a WHERE workspace_id = $1",
    )
    .bind(s.ws)
    .fetch_one(&base)
    .await
    .unwrap();
    for needle in [NAME, "Sam", "sam@example.com", &email_of(s.user)] {
        assert!(!text.contains(needle), "audit log holds {needle}");
    }
    assert!(text.contains(&s.user.to_string()));
    let mut conn = base.acquire().await.unwrap();
    let report = db::verify_chain(&mut conn, s.ws).await.unwrap();
    assert!(report.is_ok(), "{report:?}");
    assert_eq!(report.rows, rows_after);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_home_workspace_with_another_administrator_stays(base: PgPool) {
    let fakes = Fakes::new();
    let s = seed(&base).await;
    let (other, other_member) = (Uuid::now_v7(), Uuid::now_v7());
    exec(
        &base,
        "INSERT INTO users (id, display_name) VALUES ($1, 'Pat')",
        &[other],
    )
    .await;
    exec(
        &base,
        "INSERT INTO workspace_members (id, workspace_id, user_id) VALUES ($1, $2, $3)",
        &[other_member, s.ws, other],
    )
    .await;
    make_admin(&base, s.ws, s.member).await;
    make_admin(&base, s.ws, other_member).await;
    seed_account(&base, &fakes, s.user).await;
    let pool = worker_pool(&base).await;
    request_restore_request(&base, s.user).await;
    make_due(&base, s.user).await;
    assert_eq!(purge_due(&pool, &fakes.deps()).await.unwrap(), 1);
    assert_eq!(
        count(&base, "SELECT count(*) FROM workspaces WHERE id = $1", s.ws).await,
        1
    );
    assert_eq!(
        count(
            &base,
            "SELECT count(*) FROM batches WHERE workspace_id = $1",
            s.ws
        )
        .await,
        1
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn account_data_export_holds_profile_workspace_data_and_files(base: PgPool) {
    let fakes = Fakes::new();
    let s = seed(&base).await;
    make_admin(&base, s.ws, s.member).await;
    seed_account(&base, &fakes, s.user).await;
    seed_support(&base, &fakes.store, s.ws, s.user, 5).await;
    exec(
        &base,
        "INSERT INTO water_profiles (id, workspace_id, name, kind) VALUES (gen_random_uuid(), $1, 'Tap Water', 'source')",
        &[s.ws],
    )
    .await;
    exec(
        &base,
        "INSERT INTO devices (id, workspace_id, kind, name, ingest_key_hash) \
         VALUES (gen_random_uuid(), $1, 'ispindel', 'Fermenter Spindel', decode(repeat('ab', 32), 'hex'))",
        &[s.ws],
    )
    .await;
    // Queue the job as the API does and run the loop.
    let job = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO export_jobs (id, workspace_id, scope, format, created_by, updated_by) \
         VALUES ($1, $2, 'account_data', 'zip', $3, $3)",
    )
    .bind(job)
    .bind(s.ws)
    .bind(s.user)
    .execute(&base)
    .await
    .unwrap();
    let pool = worker_pool(&base).await;
    assert_eq!(
        worker::export::run_pending(&pool, fakes.store.as_ref())
            .await
            .unwrap(),
        1
    );
    let (status, key): (String, Option<String>) =
        sqlx::query_as("SELECT status, result_object_key FROM export_jobs WHERE id = $1")
            .bind(job)
            .fetch_one(&base)
            .await
            .unwrap();
    assert_eq!(status, "done");
    let zip = fakes
        .store
        .objects
        .lock()
        .unwrap()
        .get(&key.unwrap())
        .cloned()
        .unwrap();
    assert_eq!(&zip[..4], b"PK\x03\x04");
    let has = |needle: &str| zip.windows(needle.len()).any(|w| w == needle.as_bytes());
    for name in [
        "account.json",
        "workspaces.json",
        "manifest.json",
        &format!("workspaces/{}/recipes.beerjson", s.ws),
        &format!("workspaces/{}/water_profiles.json", s.ws),
        &format!("workspaces/{}/devices.json", s.ws),
        &format!("workspaces/{}/support_requests.json", s.ws),
        &format!("workspaces/{}/documents.json", s.ws),
        &format!("workspaces/{}/me/settings.json", s.ws),
        &format!("workspaces/{}/files/", s.ws),
    ] {
        assert!(has(name), "{name} is missing");
    }
    // Profile and content are in the file; secrets are not.
    for text in [
        NAME,
        "Tap Water",
        "Fermenter Spindel",
        "Please help",
        "Phone",
    ] {
        assert!(has(text), "{text} is missing");
    }
    assert!(!has("apple-refresh-token"));
    assert!(!has("provider_refresh_token_enc"));
    assert!(!has("ingest_key_hash"));
}
