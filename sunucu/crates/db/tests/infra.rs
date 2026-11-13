//! Outbox, idempotency keys and the other infrastructure tables (ADR-0007, 2.5; ADR-0008, 6).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{Role, create_user, create_workspace, id, pool_as, set_context, sqlstate};
use sqlx::{PgPool, types::Uuid};

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn outbox_is_insert_only_for_the_api_and_readable_by_the_worker(
    pool: PgPool,
) -> sqlx::Result<()> {
    let api = pool_as(&pool, Role::Api).await;
    let worker = pool_as(&pool, Role::Worker).await;
    let ws = id(1);
    create_workspace(&api, ws, "A").await?;

    // The insert happens in the transaction of the change that caused the event.
    let mut tx = api.begin().await?;
    set_context(&mut tx, Some(ws), None).await?;
    sqlx::query("INSERT INTO outbox (id, topic, workspace_id, payload) VALUES ($1, 'recipe.updated', $2, '{\"schema_version\": 1}')")
        .bind(id(200))
        .bind(ws)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    let err = sqlx::query("SELECT * FROM outbox")
        .execute(&api)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some("42501"));

    // The worker sees events of every workspace without any context.
    let pending: Vec<(Uuid,)> = sqlx::query_as("SELECT id FROM outbox WHERE published_at IS NULL")
        .fetch_all(&worker)
        .await?;
    assert_eq!(pending, vec![(id(200),)]);
    sqlx::query("UPDATE outbox SET published_at = now() WHERE id = $1")
        .bind(id(200))
        .execute(&worker)
        .await?;
    // Payload must be an object that carries schema_version.
    let err = sqlx::query("INSERT INTO outbox (id, topic, payload) VALUES ($1, 't', '{}')")
        .bind(id(201))
        .execute(&api)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some("23514"));
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn external_events_deduplicate_deliveries(pool: PgPool) -> sqlx::Result<()> {
    let api = pool_as(&pool, Role::Api).await;
    let insert = "INSERT INTO external_events (id, source, delivery_id, payload) VALUES ($1, 'github', 'd-1', '{}')";
    sqlx::query(insert).bind(id(300)).execute(&api).await?;
    let err = sqlx::query(insert)
        .bind(id(301))
        .execute(&api)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some("23505"));
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn idempotency_keys_are_scoped_unique_and_isolated(pool: PgPool) -> sqlx::Result<()> {
    let api = pool_as(&pool, Role::Api).await;
    let (a, b, user) = (id(1), id(2), id(21));
    create_workspace(&api, a, "A").await?;
    create_workspace(&api, b, "B").await?;
    create_user(&api, user).await?;
    let insert_ws = "INSERT INTO idempotency_keys (id, workspace_id, key, request_hash) VALUES ($1, $2, $3, $4)";

    // Same key in two workspaces is fine; twice in one is not.
    for (n, ws) in [(1u128, a), (2, b)] {
        let mut tx = api.begin().await?;
        set_context(&mut tx, Some(ws), None).await?;
        sqlx::query(insert_ws)
            .bind(id(400 + n))
            .bind(ws)
            .bind("key-1")
            .bind(vec![1u8; 32])
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
    }
    let mut tx = api.begin().await?;
    set_context(&mut tx, Some(a), None).await?;
    let err = sqlx::query(insert_ws)
        .bind(id(410))
        .bind(a)
        .bind("key-1")
        .bind(vec![2u8; 32])
        .execute(&mut *tx)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some("23505"));
    drop(tx);

    // Only the caller's workspace rows are visible; B's key cannot be read from A.
    let mut tx = api.begin().await?;
    set_context(&mut tx, Some(a), None).await?;
    let seen: Vec<(Uuid,)> = sqlx::query_as("SELECT id FROM idempotency_keys")
        .fetch_all(&mut *tx)
        .await?;
    assert_eq!(seen, vec![(id(401),)]);
    drop(tx);

    // Workspace-less endpoints key by user; a row needs exactly one scope.
    let mut tx = api.begin().await?;
    set_context(&mut tx, None, Some(user)).await?;
    sqlx::query(
        "INSERT INTO idempotency_keys (id, user_id, key, request_hash) VALUES ($1, $2, 'k', $3)",
    )
    .bind(id(420))
    .bind(user)
    .bind(vec![1u8; 32])
    .execute(&mut *tx)
    .await?;
    set_context(&mut tx, Some(a), Some(user)).await?;
    let err = sqlx::query("INSERT INTO idempotency_keys (id, workspace_id, user_id, key, request_hash) VALUES ($1, $2, $3, 'z', $4)")
        .bind(id(421))
        .bind(a)
        .bind(user)
        .bind(vec![1u8; 32])
        .execute(&mut *tx)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some("23514"));
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn worker_purges_expired_keys_without_context(pool: PgPool) -> sqlx::Result<()> {
    let api = pool_as(&pool, Role::Api).await;
    let worker = pool_as(&pool, Role::Worker).await;
    let a = id(1);
    create_workspace(&api, a, "A").await?;
    let mut tx = api.begin().await?;
    set_context(&mut tx, Some(a), None).await?;
    for (n, ttl) in [(1u128, "24 hours"), (2, "-1 hour")] {
        sqlx::query(
            "INSERT INTO idempotency_keys (id, workspace_id, key, request_hash, expires_at)
             VALUES ($1, $2, $3, $4, now() + $5::interval)",
        )
        .bind(id(500 + n))
        .bind(a)
        .bind(format!("k{n}"))
        .bind(vec![1u8; 32])
        .bind(ttl)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    let purged: i64 = sqlx::query_scalar("SELECT purge_expired_idempotency_keys()")
        .fetch_one(&worker)
        .await?;
    assert_eq!(purged, 1);
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM idempotency_keys")
        .fetch_one(&pool)
        .await?;
    assert_eq!(left, 1);
    // The API role may not call it.
    let err = sqlx::query("SELECT purge_expired_idempotency_keys()")
        .execute(&api)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some("42501"));
    Ok(())
}
