//! The audit log is append-only for everybody (ADR-0007, rule 5.4): the application roles
//! have no UPDATE/DELETE/TRUNCATE privilege, and triggers refuse the same for the owner.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{Role, create_workspace, id, pool_as, set_context, sqlstate};
use sqlx::PgPool;

const INSUFFICIENT_PRIVILEGE: &str = "42501";
const INTEGRITY_VIOLATION: &str = "23000";

async fn seed(app: &PgPool) -> sqlx::types::Uuid {
    let ws = id(1);
    create_workspace(app, ws, "A").await.unwrap();
    let mut tx = app.begin().await.unwrap();
    set_context(&mut tx, Some(ws), None).await.unwrap();
    sqlx::query(
        "INSERT INTO audit_log (id, workspace_id, seq, actor_kind, action, record_kind, prev_hash, hash)
         VALUES ($1, $2, 1, 'system', 'workspace.create', 'workspace', $3, $4)",
    )
    .bind(id(100))
    .bind(ws)
    .bind(vec![0u8; 32])
    .bind(vec![1u8; 32])
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    ws
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn application_roles_can_insert_and_read_but_not_change(pool: PgPool) -> sqlx::Result<()> {
    for role in [Role::Api, Role::Worker] {
        let app = pool_as(&pool, role).await;
        let ws = if matches!(role, Role::Api) {
            seed(&app).await
        } else {
            id(1)
        };

        let mut tx = app.begin().await?;
        set_context(&mut tx, Some(ws), None).await?;
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM audit_log")
            .fetch_one(&mut *tx)
            .await?;
        assert_eq!(n, 1);
        for sql in [
            "UPDATE audit_log SET action = 'tampered'",
            "DELETE FROM audit_log",
            "TRUNCATE audit_log",
        ] {
            let mut inner = app.begin().await?;
            set_context(&mut inner, Some(ws), None).await?;
            let err = sqlx::query(sql).execute(&mut *inner).await.unwrap_err();
            assert_eq!(
                sqlstate(&err).as_deref(),
                Some(INSUFFICIENT_PRIVILEGE),
                "`{sql}`: {err}"
            );
        }
    }
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn triggers_refuse_changes_even_for_the_owner(pool: PgPool) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let ws = seed(&app).await;
    let owner = pool_as(&pool, Role::Owner).await;

    for sql in [
        "UPDATE audit_log SET action = 'tampered'",
        "DELETE FROM audit_log",
        "TRUNCATE audit_log",
    ] {
        let mut tx = owner.begin().await?;
        set_context(&mut tx, Some(ws), None).await?;
        let err = sqlx::query(sql).execute(&mut *tx).await.unwrap_err();
        assert_eq!(
            sqlstate(&err).as_deref(),
            Some(INTEGRITY_VIOLATION),
            "`{sql}`: {err}"
        );
    }
    // Even the superuser's direct DELETE is refused by the trigger.
    let err = sqlx::query("DELETE FROM audit_log")
        .execute(&pool)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY_VIOLATION));

    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM audit_log")
        .fetch_one(&pool)
        .await?;
    assert_eq!(n, 1);
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn table_shape_rejects_malformed_chain_fields(pool: PgPool) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let ws = seed(&app).await;
    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(ws), None).await?;
    // Wrong hash length.
    let err = sqlx::query(
        "INSERT INTO audit_log (id, workspace_id, seq, actor_kind, action, record_kind, prev_hash, hash)
         VALUES ($1, $2, 2, 'system', 'x', 'y', $3, $4)",
    )
    .bind(id(101))
    .bind(ws)
    .bind(vec![0u8; 31])
    .bind(vec![1u8; 32])
    .execute(&mut *tx)
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some("23514"));
    drop(tx);

    // A sequence number is used once per workspace.
    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(ws), None).await?;
    let err = sqlx::query(
        "INSERT INTO audit_log (id, workspace_id, seq, actor_kind, action, record_kind, prev_hash, hash)
         VALUES ($1, $2, 1, 'system', 'x', 'y', $3, $4)",
    )
    .bind(id(102))
    .bind(ws)
    .bind(vec![0u8; 32])
    .bind(vec![1u8; 32])
    .execute(&mut *tx)
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some("23505"));
    Ok(())
}
