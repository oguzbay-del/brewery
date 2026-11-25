//! Automatic closure of resolved support requests (T2.43). Needs PostgreSQL.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::{Duration, Utc};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;
use worker::retention;

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

async fn workspace(pool: &PgPool) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO workspaces (id, name, mode, region, country, time_zone, support_pseudonym)
         VALUES ($1, 'Support close test', 'business', 'us', 'US', 'America/New_York', $2)",
    )
    .bind(id)
    .bind(format!("support-close-{id}"))
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn request(
    pool: &PgPool,
    ws: Uuid,
    status: &str,
    resolved_at: Option<chrono::DateTime<Utc>>,
) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO support_requests (id, workspace_id, topic, title, status, resolved_at)
         VALUES ($1, $2, 'other', 'Automatic close test', $3, $4)",
    )
    .bind(id)
    .bind(ws)
    .bind(status)
    .bind(resolved_at)
    .execute(pool)
    .await
    .unwrap();
    id
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn closes_only_old_resolved_requests_and_enqueues_each_sync_once(pool: PgPool) {
    let worker = worker_pool(&pool).await;
    let (a, b) = (workspace(&pool).await, workspace(&pool).await);
    let now = Utc::now();

    let old_a = request(&pool, a, "resolved", Some(now - Duration::days(15))).await;
    let recent = request(&pool, a, "resolved", Some(now - Duration::days(13))).await;
    let open = request(&pool, a, "open", Some(now - Duration::days(30))).await;
    let answered = request(&pool, a, "answered", Some(now - Duration::days(30))).await;
    let reopened = request(&pool, a, "resolved", None).await;
    let old_b = request(&pool, b, "resolved", Some(now - Duration::days(16))).await;

    let closed = retention::close_resolved_support_requests(&worker, now)
        .await
        .unwrap();
    assert_eq!(closed, 2);

    let states: Vec<(Uuid, String, i64, Option<Uuid>)> = sqlx::query_as(
        "SELECT id, status, version, updated_by FROM support_requests
         WHERE id = ANY($1) ORDER BY id",
    )
    .bind(vec![old_a, recent, open, answered, reopened, old_b])
    .fetch_all(&pool)
    .await
    .unwrap();
    let state = |id: Uuid| states.iter().find(|r| r.0 == id).unwrap();
    for id in [old_a, old_b] {
        assert_eq!(state(id).1, "closed");
        assert_eq!(state(id).2, 2, "the version trigger must bump the version");
        assert!(state(id).3.is_none(), "system update must clear updated_by");
    }
    for id in [recent, open, answered, reopened] {
        assert_ne!(state(id).1, "closed", "request {id} must remain unchanged");
        assert_eq!(state(id).2, 1);
    }

    let events: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM outbox
         WHERE topic = 'support.sync_requested' AND workspace_id IN ($1, $2)
           AND payload->'entity'->>'id' = ANY($3)",
    )
    .bind(a)
    .bind(b)
    .bind(vec![old_a.to_string(), old_b.to_string()])
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(events, 2);

    let again = retention::close_resolved_support_requests(&worker, now)
        .await
        .unwrap();
    assert_eq!(again, 0);
    let events_after: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM outbox
         WHERE topic = 'support.sync_requested'
           AND payload->'entity'->>'id' = ANY($1)",
    )
    .bind(vec![old_a.to_string(), old_b.to_string()])
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(events_after, 2);
}
