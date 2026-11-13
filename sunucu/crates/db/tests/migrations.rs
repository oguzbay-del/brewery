//! Integration tests against a real PostgreSQL (`DATABASE_URL`). `#[sqlx::test]` creates a
//! throwaway database per test and applies `sunucu/migrations/` to it.

use sqlx::{PgPool, types::Uuid};

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn migrations_are_applied(pool: PgPool) -> sqlx::Result<()> {
    let applied: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations WHERE success")
        .fetch_one(&pool)
        .await?;
    assert_eq!(
        applied,
        i64::try_from(db::MIGRATOR.migrations.len()).unwrap()
    );
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn migrate_is_idempotent(pool: PgPool) {
    db::migrate(&pool).await.unwrap();
    db::migrate(&pool).await.unwrap();
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn workspace_id_is_scoped_to_the_transaction(pool: PgPool) -> sqlx::Result<()> {
    let id = Uuid::from_u128(0x0192_f1c0_7a3b_7c00_8000_0000_0000_0001);
    let mut tx = pool.begin().await?;
    db::set_workspace_id(&mut tx, id).await.unwrap();
    let inside: Option<String> =
        sqlx::query_scalar("SELECT current_setting('app.workspace_id', true)")
            .fetch_one(&mut *tx)
            .await?;
    assert_eq!(inside.as_deref(), Some(id.to_string().as_str()));
    tx.commit().await?;

    // After commit the setting is gone: empty string or NULL, never the old value.
    let after: Option<String> =
        sqlx::query_scalar("SELECT current_setting('app.workspace_id', true)")
            .fetch_one(&pool)
            .await?;
    assert!(after.is_none_or(|v| v.is_empty()));
    Ok(())
}
