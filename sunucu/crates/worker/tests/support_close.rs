//! Automatic closure of resolved support requests after 14 days (T2.43).
//! Needs PostgreSQL (DATABASE_URL).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::{DateTime, Duration, Utc};
use serde_json::Value;
use sqlx::{PgPool, postgres::PgPoolOptions, types::Uuid};
use worker::maintenance::CLOSE_AFTER_DAYS;

fn id(n: u128) -> Uuid {
    Uuid::from_u128(0x0192_f1c0_7a3b_7000_8000_0000_0000_0000 + n)
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

async fn create_workspace(pool: &PgPool, workspace_id: Uuid) {
    sqlx::query(
        "INSERT INTO workspaces (id, name, mode, region, country, time_zone, support_pseudonym)
         VALUES ($1, 'Support close test', 'business', 'us', 'US', 'America/New_York', $2)",
    )
    .bind(workspace_id)
    .bind(format!("support-close-{workspace_id}"))
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_request(
    pool: &PgPool,
    workspace_id: Uuid,
    request_id: Uuid,
    status: &str,
    resolved_at: Option<DateTime<Utc>>,
) {
    sqlx::query(
        "INSERT INTO support_requests (id, workspace_id, topic, title, status, resolved_at)
         VALUES ($1, $2, 'other', 'Support close test', $3, $4)",
    )
    .bind(request_id)
    .bind(workspace_id)
    .bind(status)
    .bind(resolved_at)
    .execute(pool)
    .await
    .unwrap();
}

async fn request_state(pool: &PgPool, request_id: Uuid) -> (String, Option<DateTime<Utc>>, i64) {
    sqlx::query_as(
        "SELECT status, resolved_at, version FROM support_requests WHERE id = $1",
    )
    .bind(request_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn closes_only_old_resolved_requests_and_enqueues_sync_once_per_workspace(base: PgPool) {
    let (workspace_a, workspace_b) = (id(1), id(2));
    create_workspace(&base, workspace_a).await;
    create_workspace(&base, workspace_b).await;

    let now = Utc::now();
    let old = now - Duration::days(CLOSE_AFTER_DAYS + 1);
    let recent = now - Duration::days(CLOSE_AFTER_DAYS - 1);
    let very_old = now - Duration::days(CLOSE_AFTER_DAYS + 6);

    let (closed_a, recent_resolved, open_old, answered_old, reopened, closed_b) =
        (id(11), id(12), id(13), id(14), id(15), id(16));

    insert_request(&base, workspace_a, closed_a, "resolved", Some(old)).await;
    insert_request(&base, workspace_a, recent_resolved, "resolved", Some(recent)).await;
    insert_request(&base, workspace_a, open_old, "open", Some(very_old)).await;
    insert_request(&base, workspace_a, answered_old, "answered", Some(very_old)).await;
    insert_request(&base, workspace_a, reopened, "resolved", Some(very_old)).await;
    insert_request(&base, workspace_b, closed_b, "resolved", Some(old)).await;

    // A customer message reopens a resolved request and clears resolved_at.
    sqlx::query(
        "UPDATE support_requests SET status = 'open', resolved_at = NULL WHERE id = $1",
    )
    .bind(reopened)
    .execute(&base)
    .await
    .unwrap();

    let worker = worker_pool(&base).await;
    let cutoff = now - Duration::days(CLOSE_AFTER_DAYS);
    let closed = db::maintenance::close_resolved_support_requests(&worker, cutoff)
        .await
        .unwrap();
    assert_eq!(closed, 2);

    let state = request_state(&base, closed_a).await;
    assert_eq!(state.0, "closed");
    assert_eq!(state.1, Some(old));
    assert_eq!(state.2, 2);

    let state = request_state(&base, closed_b).await;
    assert_eq!(state.0, "closed");
    assert_eq!(state.1, Some(old));
    assert_eq!(state.2, 2);

    assert_eq!(request_state(&base, recent_resolved).await.0, "resolved");
    assert_eq!(request_state(&base, open_old).await.0, "open");
    assert_eq!(request_state(&base, answered_old).await.0, "answered");
    let reopened_state = request_state(&base, reopened).await;
    assert_eq!(reopened_state.0, "open");
    assert_eq!(reopened_state.1, None);

    let events: Vec<(Uuid, Value)> = sqlx::query_as(
        "SELECT workspace_id, payload FROM outbox WHERE topic = 'support.sync_requested'",
    )
    .fetch_all(&base)
    .await
    .unwrap();
    assert_eq!(events.len(), 2);

    let mut event_requests = Vec::new();
    for (workspace_id, payload) in &events {
        assert_eq!(payload["entity"]["kind"], "support_request");
        let request_id = Uuid::parse_str(payload["entity"]["id"].as_str().unwrap()).unwrap();
        let version = payload["entity"]["version"].as_i64().unwrap();
        assert_eq!(version, 2);
        event_requests.push((*workspace_id, request_id));
    }
    assert!(event_requests.contains(&(workspace_a, closed_a)));
    assert!(event_requests.contains(&(workspace_b, closed_b)));
    assert!(!event_requests.iter().any(|(_, request_id)| {
        [recent_resolved, open_old, answered_old, reopened].contains(request_id)
    }));

    // A second run is harmless and does not enqueue duplicate events.
    let again = db::maintenance::close_resolved_support_requests(&worker, cutoff)
        .await
        .unwrap();
    assert_eq!(again, 0);
    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM outbox WHERE topic = 'support.sync_requested'",
    )
    .fetch_one(&base)
    .await
    .unwrap();
    assert_eq!(event_count, 2);
}
