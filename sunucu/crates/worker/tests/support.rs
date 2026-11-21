//! Support requests (T2.29): request -> Issue, `/yanit` -> notification, internal notes, GitHub
//! down -> sync later. Needs PostgreSQL (`DATABASE_URL`), NATS and mailpit (compose defaults).
//! The GitHub and APNs servers are fake servers in the test process.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::{
    sync::{
        Arc,
        atomic::{AtomicI64, Ordering},
    },
    time::Duration,
};

use async_nats::jetstream;
use chrono::Utc;
use common::{Fake, Seed, nats_url, seed, wait_for, worker_pool};
use events::{
    consumer::{self, ConsumerConfig},
    relay,
};
use notifier::{
    apns::ApnsConfig,
    deliver::{Deliverer, Outcome, deliver_one},
    enter_workspace,
    smtp::SmtpConfig,
};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{
    github::{Auth, GitHub},
    reply, sync,
    sync::SyncHandler,
    webhook::{self, Applied},
};
use tokio::sync::watch;
use uuid::Uuid;

const BOT: &str = "demleme-bot";

async fn fake_github() -> Fake {
    let comments = Arc::new(AtomicI64::new(100));
    Fake::start(move |seen| {
        let path = seen.path.as_str();
        match (seen.method.as_str(), path) {
            ("POST", "/repos/o/r/issues") => (201, json!({"number": 7})),
            ("POST", p) if p.ends_with("/comments") => (
                201,
                json!({"id": comments.fetch_add(1, Ordering::SeqCst)}),
            ),
            ("DELETE", _) => (204, Value::Null),
            _ => (200, json!({})),
        }
    })
    .await
}

fn client(fake: &Fake) -> GitHub {
    GitHub::new(&fake.url, "o/r", Auth::Token("t".to_owned()), BOT)
}

/// A request with its first customer message (as the API writes them).
async fn open_request(base: &PgPool, s: &Seed) -> (Uuid, Uuid, i64) {
    let (request, message) = (Uuid::now_v7(), Uuid::now_v7());
    let number: i64 = sqlx::query_scalar(
        "INSERT INTO support_requests (id, workspace_id, topic, title, diagnostics, created_by)
         VALUES ($1, $2, 'recipes', 'Cannot publish', $3, $4) RETURNING public_number",
    )
    .bind(request)
    .bind(s.ws)
    .bind(json!({"schema_version": 1, "app_version": "2.3.1"}))
    .bind(s.user)
    .fetch_one(base)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO support_messages (id, workspace_id, request_id, author_kind, author_user_id, body)
         VALUES ($1, $2, $3, 'customer', $4, 'The publish button does nothing.')",
    )
    .bind(message)
    .bind(s.ws)
    .bind(request)
    .bind(s.user)
    .execute(base)
    .await
    .unwrap();
    (request, message, number)
}

async fn run_sync(pool: &PgPool, gh: &GitHub, s: &Seed, request: Uuid) -> sync::Outcome {
    let mut tx = pool.begin().await.unwrap();
    enter_workspace(&mut tx, s.ws).await.unwrap();
    let out = sync::sync_request(&mut tx, gh, request).await.unwrap();
    tx.commit().await.unwrap();
    out
}

async fn apply(pool: &PgPool, event: &str, payload: &Value) -> Applied {
    let mut tx = pool.begin().await.unwrap();
    let out = webhook::apply(&mut tx, BOT, event, payload).await.unwrap();
    tx.commit().await.unwrap();
    out
}

fn comment(action: &str, id: i64, login: &str, body: &str) -> Value {
    json!({"action": action, "issue": {"number": 7},
        "comment": {"id": id, "body": body, "user": {"login": login}},
        "sender": {"login": login}})
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn request_becomes_an_issue_and_changes_follow(base: PgPool) {
    let s = seed(&base).await;
    let pool = worker_pool(&base).await;
    let fake = fake_github().await;
    let gh = client(&fake);
    let (request, message, number) = open_request(&base, &s).await;

    assert_eq!(run_sync(&pool, &gh, &s, request).await, sync::Outcome::Synced);
    let seen = fake.seen();
    assert_eq!(seen.len(), 2);
    let issue = &seen[0];
    assert_eq!(issue.path, "/repos/o/r/issues");
    assert_eq!(issue.body["title"], format!("#{number} Cannot publish"));
    let text = issue.body["body"].as_str().unwrap();
    assert!(text.contains(&format!("pseudonym-{}", s.ws)), "alias in the Issue");
    assert!(text.contains("2.3.1"));
    assert!(!text.contains("example.com"), "no e-mail in the Issue");
    assert_eq!(issue.body["labels"], json!(["konu:recipes", "mod:ev"]));
    let comment_body = seen[1].body["body"].as_str().unwrap();
    assert!(comment_body.starts_with(&format!("<!-- demleme:msg:{message} -->")));
    assert!(comment_body.contains("**Müşteri**"));
    assert!(comment_body.contains("The publish button does nothing."));
    let (issue_no, comment_id): (i32, i64) = sqlx::query_as(
        "SELECT r.github_issue_number, m.github_comment_id FROM support_requests r
         JOIN support_messages m ON m.request_id = r.id WHERE r.id = $1",
    )
    .bind(request)
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!((issue_no, comment_id), (7, 100));

    // A second run changes nothing.
    assert_eq!(run_sync(&pool, &gh, &s, request).await, sync::Outcome::Synced);
    assert_eq!(fake.seen().len(), 2);

    // Edit, then retract, of the customer message.
    sqlx::query("UPDATE support_messages SET body = 'It does nothing at all.', edited_at = now() WHERE id = $1")
        .bind(message)
        .execute(&base)
        .await
        .unwrap();
    run_sync(&pool, &gh, &s, request).await;
    let seen = fake.seen();
    let edit = seen.last().unwrap();
    assert_eq!((edit.method.as_str(), edit.path.as_str()), ("PATCH", "/repos/o/r/issues/comments/100"));
    assert!(edit.body["body"].as_str().unwrap().contains("at all"));
    sqlx::query("UPDATE support_messages SET retracted_at = now() WHERE id = $1")
        .bind(message)
        .execute(&base)
        .await
        .unwrap();
    run_sync(&pool, &gh, &s, request).await;
    let del = fake.seen().last().cloned().unwrap();
    assert_eq!((del.method.as_str(), del.path.as_str()), ("DELETE", "/repos/o/r/issues/comments/100"));
    let before = fake.seen().len();
    run_sync(&pool, &gh, &s, request).await;
    assert_eq!(fake.seen().len(), before, "a retracted message syncs once");

    // Status: answered adds the waiting label, resolved closes the Issue.
    sqlx::query("UPDATE support_requests SET status = 'answered' WHERE id = $1")
        .bind(request)
        .execute(&base)
        .await
        .unwrap();
    run_sync(&pool, &gh, &s, request).await;
    let patch = fake.seen().last().cloned().unwrap();
    assert_eq!(patch.path, "/repos/o/r/issues/7");
    assert_eq!(patch.body["state"], "open");
    assert!(patch.body["labels"].as_array().unwrap().contains(&json!("bekliyor:musteri")));
    sqlx::query("UPDATE support_requests SET status = 'resolved' WHERE id = $1")
        .bind(request)
        .execute(&base)
        .await
        .unwrap();
    run_sync(&pool, &gh, &s, request).await;
    let patch = fake.seen().last().cloned().unwrap();
    assert_eq!(patch.body["state"], "closed");
    assert!(!patch.body["labels"].as_array().unwrap().contains(&json!("bekliyor:musteri")));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn when_github_is_down_the_request_syncs_later(base: PgPool) {
    let s = seed(&base).await;
    let pool = worker_pool(&base).await;
    let fake = fake_github().await;
    let gh = client(&fake);
    let (request, _, _) = open_request(&base, &s).await;

    // The request exists and the customer can read it while GitHub fails.
    fake.fail_next(1, 503);
    let out = run_sync(&pool, &gh, &s, request).await;
    assert!(matches!(out, sync::Outcome::Later(_)), "{out:?}");
    let (status, issue): (String, Option<i32>) =
        sqlx::query_as("SELECT status, github_issue_number FROM support_requests WHERE id = $1")
            .bind(request)
            .fetch_one(&base)
            .await
            .unwrap();
    assert_eq!((status.as_str(), issue), ("open", None));

    // The retry job runs once GitHub answers, and only one job exists for the request.
    let mut tx = pool.begin().await.unwrap();
    enter_workspace(&mut tx, s.ws).await.unwrap();
    sync::schedule(&mut tx, s.ws, request, Duration::ZERO).await.unwrap();
    sync::schedule(&mut tx, s.ws, request, Duration::ZERO).await.unwrap();
    tx.commit().await.unwrap();
    let jobs: i64 = sqlx::query_scalar("SELECT count(*) FROM scheduled_jobs WHERE kind = 'support.github_sync'")
        .fetch_one(&base)
        .await
        .unwrap();
    assert_eq!(jobs, 1);
    assert_eq!(sync::run_due(&pool, &gh).await.unwrap(), 1);
    let issue: Option<i32> =
        sqlx::query_scalar("SELECT github_issue_number FROM support_requests WHERE id = $1")
            .bind(request)
            .fetch_one(&base)
            .await
            .unwrap();
    assert_eq!(issue, Some(7));
    let status: String = sqlx::query_scalar("SELECT status FROM scheduled_jobs WHERE kind = 'support.github_sync'")
        .fetch_one(&base)
        .await
        .unwrap();
    assert_eq!(status, "done");

    // A job that fails again waits longer.
    let (request2, _, _) = open_request(&base, &s).await;
    fake.fail_next(1, 429);
    let mut tx = pool.begin().await.unwrap();
    enter_workspace(&mut tx, s.ws).await.unwrap();
    sync::schedule(&mut tx, s.ws, request2, Duration::ZERO).await.unwrap();
    tx.commit().await.unwrap();
    assert_eq!(sync::run_due(&pool, &gh).await.unwrap(), 1);
    let (attempts, status, waits): (i32, String, bool) = sqlx::query_as(
        "SELECT attempts, status, run_at > now() FROM scheduled_jobs
         WHERE payload ->> 'request_id' = $1",
    )
    .bind(request2.to_string())
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!((attempts, status.as_str(), waits), (1, "pending", true));
}

async fn mail_subjects(to: &str) -> Vec<(String, String)> {
    let url = format!("http://127.0.0.1:8025/api/v1/search?query=to:{to}");
    let reply: Value = reqwest::get(url).await.unwrap().json().await.unwrap();
    let mut out = Vec::new();
    for m in reply["messages"].as_array().cloned().unwrap_or_default() {
        let id = m["ID"].as_str().unwrap();
        let full: Value = reqwest::get(format!("http://127.0.0.1:8025/api/v1/message/{id}"))
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        out.push((
            m["Subject"].as_str().unwrap().to_owned(),
            full["Text"].as_str().unwrap_or_default().to_owned(),
        ));
    }
    out
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn yanit_reaches_the_customer_and_notes_do_not(base: PgPool) {
    let s = seed(&base).await;
    let pool = worker_pool(&base).await;
    let fake = fake_github().await;
    let gh = client(&fake);
    let (request, _, number) = open_request(&base, &s).await;
    run_sync(&pool, &gh, &s, request).await;
    sqlx::query(
        "INSERT INTO staff_users (id, google_subject, email, role, github_login)
         VALUES ($1, $2, $3, 'support', 'alice')",
    )
    .bind(Uuid::now_v7())
    .bind(format!("sub-{}", s.ws))
    .bind(format!("alice-{}@example.com", s.ws.simple()))
    .execute(&base)
    .await
    .unwrap();

    let count = |sql: &'static str| {
        let base = base.clone();
        async move { sqlx::query_scalar::<_, i64>(sql).fetch_one(&base).await.unwrap() }
    };
    let notifications = || count("SELECT count(*) FROM notifications WHERE kind = 'support.reply'");

    // Internal notes: from the team and from anyone else. No notification, no status change.
    assert_eq!(apply(&pool, "issue_comment", &comment("created", 501, "alice", "Looks like the cache.")).await, Applied::Note);
    assert_eq!(apply(&pool, "issue_comment", &comment("created", 502, "bob-dev", "Related to #99")).await, Applied::Note);
    assert_eq!(notifications().await, 0);
    let internal: i64 = count("SELECT count(*) FROM support_messages WHERE visibility = 'internal'").await;
    assert_eq!(internal, 2);
    let author: String = sqlx::query_scalar("SELECT author_kind FROM support_messages WHERE github_comment_id = 502")
        .fetch_one(&base)
        .await
        .unwrap();
    assert_eq!(author, "system");
    let status: String = sqlx::query_scalar("SELECT status FROM support_requests WHERE id = $1")
        .bind(request)
        .fetch_one(&base)
        .await
        .unwrap();
    assert_eq!(status, "open");

    // Ignored: /yanit from outside the team, the App's own comment, a marked comment, a repeat.
    assert_eq!(apply(&pool, "issue_comment", &comment("created", 503, "bob-dev", "/yanit Hi")).await, Applied::Ignored);
    assert_eq!(apply(&pool, "issue_comment", &comment("created", 504, BOT, "/yanit Echo")).await, Applied::Ignored);
    assert_eq!(apply(&pool, "issue_comment", &comment("created", 505, "alice", "<!-- demleme:msg:x -->\n/yanit Hi")).await, Applied::Ignored);
    assert_eq!(apply(&pool, "issue_comment", &comment("created", 501, "alice", "Looks like the cache.")).await, Applied::Ignored);
    assert_eq!(apply(&pool, "issue_comment", &comment("created", 506, "alice", "/yanit")).await, Applied::Ignored);
    assert_eq!(notifications().await, 0);

    // The reply.
    assert_eq!(
        apply(&pool, "issue_comment", &comment("created", 510, "Alice", "/yanit Please update to 2.3.2.")).await,
        Applied::Reply
    );
    let (body, visibility): (String, String) = sqlx::query_as(
        "SELECT body, visibility FROM support_messages WHERE github_comment_id = 510",
    )
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!((body.as_str(), visibility.as_str()), ("Please update to 2.3.2.", "customer"));
    let status: String = sqlx::query_scalar("SELECT status FROM support_requests WHERE id = $1")
        .bind(request)
        .fetch_one(&base)
        .await
        .unwrap();
    assert_eq!(status, "answered");
    assert_eq!(notifications().await, 1);
    // The label goes to the Issue on the next sync.
    run_sync(&pool, &gh, &s, request).await;
    let patch = fake.seen().last().cloned().unwrap();
    assert!(patch.body["labels"].as_array().unwrap().contains(&json!("bekliyor:musteri")));

    // Delivery: push and e-mail, without the text of the reply.
    let apns = Fake::start(|_| (200, json!({}))).await;
    sqlx::query("INSERT INTO push_targets (id, user_id, environment, token) VALUES ($1, $2, 'sandbox', 'cc33')")
        .bind(Uuid::now_v7())
        .bind(s.user)
        .execute(&base)
        .await
        .unwrap();
    let d = Deliverer::new(
        Some(ApnsConfig {
            base_url: Some(apns.url.clone()),
            topic: "app.demleme.test".to_owned(),
            signer: None,
        }),
        Some(SmtpConfig {
            host: "127.0.0.1".to_owned(),
            port: 1025,
            from: "noreply@demleme.local".to_owned(),
        }),
    );
    let nid: Uuid = sqlx::query_scalar("SELECT id FROM notifications WHERE kind = 'support.reply'")
        .fetch_one(&base)
        .await
        .unwrap();
    let mut tx = pool.begin().await.unwrap();
    enter_workspace(&mut tx, s.ws).await.unwrap();
    let out = deliver_one(&mut tx, &d, s.ws, nid, Utc::now()).await.unwrap();
    tx.commit().await.unwrap();
    assert_eq!(out, Outcome::Delivered { pushes: 1, email: true });
    let push = apns.seen().remove(0);
    assert_eq!(push.body["aps"]["alert"]["title-loc-key"], "notification.support.reply");
    assert!(!push.body.to_string().contains("2.3.2"));
    let email: String = sqlx::query_scalar("SELECT contact_email FROM users WHERE id = $1")
        .bind(s.user)
        .fetch_one(&base)
        .await
        .unwrap();
    let mails = mail_subjects(&email).await;
    assert_eq!(mails.len(), 1);
    assert!(mails[0].0.contains(&format!("#{number}")));
    assert!(!mails[0].1.contains("2.3.2"), "the mail has no reply text");

    // Edit and delete of the reply on the GitHub side reach the customer record.
    assert_eq!(apply(&pool, "issue_comment", &comment("edited", 510, "alice", "/yanit Update to 2.3.3.")).await, Applied::Updated);
    let (body, edited): (String, bool) = sqlx::query_as(
        "SELECT body, edited_at IS NOT NULL FROM support_messages WHERE github_comment_id = 510",
    )
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!((body.as_str(), edited), ("Update to 2.3.3.", true));
    assert_eq!(apply(&pool, "issue_comment", &comment("deleted", 510, "alice", "")).await, Applied::Updated);
    let retracted: bool = sqlx::query_scalar(
        "SELECT retracted_at IS NOT NULL FROM support_messages WHERE github_comment_id = 510",
    )
    .fetch_one(&base)
    .await
    .unwrap();
    assert!(retracted);
    // Editing a comment that the application wrote does nothing.
    let marked = comment("edited", 100, "alice", "x");
    assert_eq!(apply(&pool, "issue_comment", &marked).await, Applied::Ignored);

    // Issue events.
    let issues = |action: &str, login: &str| {
        json!({"action": action, "issue": {"number": 7}, "sender": {"login": login}})
    };
    assert_eq!(apply(&pool, "issues", &issues("closed", BOT)).await, Applied::Ignored);
    assert_eq!(apply(&pool, "issues", &issues("closed", "alice")).await, Applied::Updated);
    let status: String = sqlx::query_scalar("SELECT status FROM support_requests WHERE id = $1")
        .bind(request)
        .fetch_one(&base)
        .await
        .unwrap();
    assert_eq!(status, "resolved");
    assert_eq!(apply(&pool, "issues", &issues("reopened", "alice")).await, Applied::Updated);
    let status: String = sqlx::query_scalar("SELECT status FROM support_requests WHERE id = $1")
        .bind(request)
        .fetch_one(&base)
        .await
        .unwrap();
    assert_eq!(status, "open");
    // An unknown Issue is ignored.
    let other = json!({"action": "closed", "issue": {"number": 999}, "sender": {"login": "alice"}});
    assert_eq!(apply(&pool, "issues", &other).await, Applied::Ignored);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn the_outbox_event_starts_the_sync(base: PgPool) {
    let s = seed(&base).await;
    let pool = worker_pool(&base).await;
    let fake = fake_github().await;
    let (request, _, _) = open_request(&base, &s).await;
    let mut tx = base.begin().await.unwrap();
    reply::enqueue_sync(&mut tx, s.ws, request).await.unwrap();
    tx.commit().await.unwrap();

    let client_nats = events::connect(&nats_url()).await.unwrap();
    let js = jetstream::new(client_nats);
    events::ensure_streams(&js, 1).await.unwrap();
    let (stop_tx, stop_rx) = watch::channel(false);
    let stopped = |mut rx: watch::Receiver<bool>| async move {
        while !*rx.borrow() {
            if rx.changed().await.is_err() {
                return;
            }
        }
    };
    let relay_task = tokio::spawn(relay::run(
        pool.clone(),
        js.clone(),
        relay::RelayConfig::default(),
        stopped(stop_rx.clone()),
    ));
    let sync_task = tokio::spawn(consumer::run(
        pool.clone(),
        js.clone(),
        ConsumerConfig::new(
            &format!("test-support-{}", s.ws),
            &format!("ev.{}.support_request.>", s.ws),
        ),
        Arc::new(SyncHandler::new(Some(Arc::new(client(&fake))))),
        stopped(stop_rx.clone()),
    ));
    let done = wait_for(15, || {
        let base = base.clone();
        async move {
            sqlx::query_scalar::<_, Option<i32>>(
                "SELECT github_issue_number FROM support_requests",
            )
            .fetch_one(&base)
            .await
            .unwrap()
            .is_some()
        }
    })
    .await;
    assert!(done, "the Issue was not created");
    stop_tx.send(true).unwrap();
    let _ = tokio::join!(relay_task, sync_task);
}
