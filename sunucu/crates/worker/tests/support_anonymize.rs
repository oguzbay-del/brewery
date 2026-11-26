//! Anonymisation of GitHub Issues in the account purge job (T2.31e). Needs PostgreSQL
//! (`#[sqlx::test]`). GitHub is an in-process fake.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

mod common;

use common::{Fake, Seen, api_pool, seed, worker_pool};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{
    REMOVED_TEXT,
    github::{Auth, GitHub},
};
use uuid::Uuid;

const CUSTOMER_COMMENT: &str =
    "<!-- demleme:msg:00000000-0000-0000-0000-000000000001 -->\n**Müşteri**\n\nMy secret text";
const STAFF_COMMENT: &str =
    "<!-- demleme:msg:00000000-0000-0000-0000-000000000002 -->\n**Yanıt (destek paneli)**\n\nHello";

fn github(fake: &Fake) -> GitHub {
    GitHub::new(&fake.url, "o/r", Auth::Token("t".to_owned()), "bot")
}

async fn start_fake() -> Fake {
    Fake::start(|seen: &Seen| {
        if seen.method == "GET" && seen.path.ends_with("/comments") {
            (
                200,
                json!([
                    {"id": 501, "body": CUSTOMER_COMMENT},
                    {"id": 502, "body": STAFF_COMMENT},
                ]),
            )
        } else {
            (200, json!({}))
        }
    })
    .await
}

async fn add_request(base: &PgPool, ws: Uuid, user: Uuid, issue: i32) {
    sqlx::query(
        "INSERT INTO support_requests (id, workspace_id, topic, title, github_issue_number, created_by)
         VALUES ($1, $2, 'other', 'Secret title', $3, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(ws)
    .bind(issue)
    .bind(user)
    .execute(base)
    .await
    .unwrap();
}

async fn make_due(base: &PgPool, user_id: Uuid) {
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
         SET requested_at = now() - interval '2 seconds', purge_at = now() - interval '1 second'
         WHERE id = $1",
    )
    .bind(request.id)
    .execute(base)
    .await
    .unwrap();
}

async fn saved(base: &PgPool, user: Uuid) -> (Vec<i32>, i32) {
    sqlx::query_as(
        "SELECT support_issue_numbers, support_issue_attempts FROM account_deletion_requests
         WHERE user_id = $1",
    )
    .bind(user)
    .fetch_one(base)
    .await
    .unwrap()
}

async fn user_done(base: &PgPool, user: Uuid) -> bool {
    let status: String =
        sqlx::query_scalar("SELECT status FROM account_deletion_requests WHERE user_id = $1")
            .bind(user)
            .fetch_one(base)
            .await
            .unwrap();
    status == "done"
}

fn patches(fake: &Fake) -> Vec<Seen> {
    fake.seen()
        .into_iter()
        .filter(|s| s.method == "PATCH")
        .collect()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn purge_anonymizes_and_closes_the_issues_of_the_user(base: PgPool) {
    let s = seed(&base).await;
    let other = seed(&base).await;
    add_request(&base, s.ws, s.user, 11).await;
    add_request(&base, s.ws, s.user, 12).await;
    add_request(&base, other.ws, other.user, 99).await;
    make_due(&base, s.user).await;
    let fake = start_fake().await;
    let worker = worker_pool(&base).await;

    let (status, detail) =
        worker::account_deletion::purge_due_and_clean(&worker, None, None, Some(&github(&fake)))
            .await
            .unwrap();

    assert_eq!(status, db::maintenance::RunStatus::Ok, "{detail}");
    let value: Value = serde_json::from_str(&detail).unwrap();
    assert_eq!(value["support_issues"]["anonymized"], 2);
    assert!(user_done(&base, s.user).await);
    assert_eq!(saved(&base, s.user).await, (vec![], 0));

    let patches = patches(&fake);
    let issue_patches: Vec<_> = patches
        .iter()
        .filter(|p| p.path.contains("/issues/1"))
        .filter(|p| !p.path.contains("/comments/"))
        .collect();
    assert_eq!(issue_patches.len(), 2);
    for patch in &issue_patches {
        assert_eq!(patch.body["body"], REMOVED_TEXT);
        assert_eq!(patch.body["state"], "closed");
    }
    // Only the customer comment is edited, once for each Issue.
    let comment_patches: Vec<_> = patches
        .iter()
        .filter(|p| p.path.contains("/comments/"))
        .collect();
    assert_eq!(comment_patches.len(), 2);
    for patch in comment_patches {
        assert!(patch.path.ends_with("/comments/501"));
        assert_eq!(patch.body["body"], REMOVED_TEXT);
    }
    // The Issue is never deleted and the other user's Issue is untouched.
    assert!(fake.seen().iter().all(|s| s.method != "DELETE"));
    assert!(fake.seen().iter().all(|s| !s.path.contains("/99")));
    let other_rows: i64 =
        sqlx::query_scalar("SELECT count(*) FROM support_requests WHERE github_issue_number = 99")
            .fetch_one(&base)
            .await
            .unwrap();
    assert_eq!(other_rows, 1);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn github_failure_keeps_the_purge_and_the_next_run_retries(base: PgPool) {
    let s = seed(&base).await;
    add_request(&base, s.ws, s.user, 21).await;
    make_due(&base, s.user).await;
    let fake = start_fake().await;
    fake.fail_next(1, 500);
    let worker = worker_pool(&base).await;
    let gh = github(&fake);

    let (status, detail) =
        worker::account_deletion::purge_due_and_clean(&worker, None, None, Some(&gh))
            .await
            .unwrap();

    // The other data is purged although GitHub failed.
    assert_eq!(status, db::maintenance::RunStatus::Findings);
    assert!(user_done(&base, s.user).await);
    let workspaces: i64 = sqlx::query_scalar("SELECT count(*) FROM workspaces WHERE id = $1")
        .bind(s.ws)
        .fetch_one(&base)
        .await
        .unwrap();
    assert_eq!(workspaces, 0);
    let value: Value = serde_json::from_str(&detail).unwrap();
    assert_eq!(value["support_issues"]["retry_later"], 1);
    assert_eq!(saved(&base, s.user).await, (vec![21], 1));
    assert!(patches(&fake).is_empty());

    // The next tick tries again and succeeds.
    let (status, detail) =
        worker::account_deletion::purge_due_and_clean(&worker, None, None, Some(&gh))
            .await
            .unwrap();
    assert_eq!(status, db::maintenance::RunStatus::Ok, "{detail}");
    assert_eq!(saved(&base, s.user).await, (vec![], 0));
    assert_eq!(patches(&fake).len(), 2);

    // Nothing is left, so a later run does not call GitHub again.
    let calls = fake.seen().len();
    worker::account_deletion::purge_due_and_clean(&worker, None, None, Some(&gh))
        .await
        .unwrap();
    assert_eq!(fake.seen().len(), calls);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn missing_github_settings_keep_the_issue_numbers(base: PgPool) {
    let s = seed(&base).await;
    add_request(&base, s.ws, s.user, 31).await;
    make_due(&base, s.user).await;
    let worker = worker_pool(&base).await;

    let (_, detail) = worker::account_deletion::purge_due_and_clean(&worker, None, None, None)
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&detail).unwrap();
    assert_eq!(value["support_issues"]["skipped"], 1);
    assert_eq!(saved(&base, s.user).await.0, vec![31]);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn user_without_support_requests_makes_no_github_call(base: PgPool) {
    let s = seed(&base).await;
    make_due(&base, s.user).await;
    let fake = start_fake().await;
    let worker = worker_pool(&base).await;

    let (status, detail) =
        worker::account_deletion::purge_due_and_clean(&worker, None, None, Some(&github(&fake)))
            .await
            .unwrap();

    assert_eq!(status, db::maintenance::RunStatus::Ok, "{detail}");
    assert!(user_done(&base, s.user).await);
    assert!(fake.seen().is_empty());
    let value: Value = serde_json::from_str(&detail).unwrap();
    assert_eq!(value["support_issues"]["anonymized"], 0);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn api_role_cannot_call_the_anonymize_functions(base: PgPool) {
    let api = api_pool(&base).await;
    assert!(
        sqlx::query("SELECT * FROM support_issue_anonymize_pending(10)")
            .fetch_all(&api)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("SELECT account_deletion_support_issues($1)")
            .bind(Uuid::now_v7())
            .fetch_one(&api)
            .await
            .is_err()
    );
}
