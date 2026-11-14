//! Workspace isolation (ADR-0007, rules 2.2-2.8), exercised as the application role.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{
    Role, create_role, create_user, create_workspace, id, pool_as, set_context, sqlstate,
};
use sqlx::PgPool;

const RLS_VIOLATION: &str = "42501";
const FOREIGN_KEY_VIOLATION: &str = "23503";

/// Two workspaces, each with one role; returns (A, B, role of A, role of B).
async fn two_workspaces(
    app: &PgPool,
) -> (
    sqlx::types::Uuid,
    sqlx::types::Uuid,
    sqlx::types::Uuid,
    sqlx::types::Uuid,
) {
    let (a, b) = (id(1), id(2));
    let (ra, rb) = (id(11), id(12));
    create_workspace(app, a, "A").await.unwrap();
    create_workspace(app, b, "B").await.unwrap();
    create_role(app, a, ra, "Admin").await.unwrap();
    create_role(app, b, rb, "Admin").await.unwrap();
    (a, b, ra, rb)
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn workspace_a_cannot_read_update_delete_or_insert_for_b(pool: PgPool) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let (a, b, ra, rb) = two_workspaces(&app).await;

    // Read: only A's rows are visible, B's row cannot be found even by id.
    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(a), None).await?;
    let visible: Vec<sqlx::types::Uuid> = sqlx::query_scalar("SELECT id FROM roles")
        .fetch_all(&mut *tx)
        .await?;
    assert_eq!(visible, vec![ra]);
    let other: Option<sqlx::types::Uuid> = sqlx::query_scalar("SELECT id FROM roles WHERE id = $1")
        .bind(rb)
        .fetch_optional(&mut *tx)
        .await?;
    assert!(other.is_none());
    let visible_ws: Vec<sqlx::types::Uuid> = sqlx::query_scalar("SELECT id FROM workspaces")
        .fetch_all(&mut *tx)
        .await?;
    assert_eq!(visible_ws, vec![a]);
    tx.rollback().await?;

    // Update and delete of B's rows touch nothing.
    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(a), None).await?;
    let updated = sqlx::query("UPDATE roles SET name = 'hijacked' WHERE id = $1")
        .bind(rb)
        .execute(&mut *tx)
        .await?;
    assert_eq!(updated.rows_affected(), 0);
    let deleted = sqlx::query("DELETE FROM roles WHERE id = $1")
        .bind(rb)
        .execute(&mut *tx)
        .await?;
    assert_eq!(deleted.rows_affected(), 0);
    tx.rollback().await?;
    // The API has no DELETE right on workspaces at all (T2.9: only the worker purges).
    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(a), None).await?;
    let refused = sqlx::query("DELETE FROM workspaces WHERE id = $1")
        .bind(b)
        .execute(&mut *tx)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&refused).as_deref(), Some("42501"));
    tx.rollback().await?;

    // Moving A's own row into B is refused by WITH CHECK.
    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(a), None).await?;
    let moved = sqlx::query("UPDATE roles SET workspace_id = $1 WHERE id = $2")
        .bind(b)
        .bind(ra)
        .execute(&mut *tx)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&moved).as_deref(), Some(RLS_VIOLATION));
    drop(tx);

    // Inserting on behalf of B is refused.
    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(a), None).await?;
    let insert = sqlx::query("INSERT INTO roles (id, workspace_id, name) VALUES ($1, $2, 'x')")
        .bind(id(99))
        .bind(b)
        .execute(&mut *tx)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&insert).as_deref(), Some(RLS_VIOLATION));
    drop(tx);

    // B's data is untouched (checked as the superuser, which sees everything).
    let names: Vec<String> = sqlx::query_scalar("SELECT name FROM roles ORDER BY id")
        .fetch_all(&pool)
        .await?;
    assert_eq!(names, vec!["Admin", "Admin"]);
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn without_context_no_row_is_visible_or_writable(pool: PgPool) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let (a, ..) = two_workspaces(&app).await;

    // Fresh connection: the setting was never set (NULL).
    for table in ["roles", "workspaces", "audit_log", "idempotency_keys"] {
        let sql = match table {
            "roles" => "SELECT count(*) FROM roles",
            "workspaces" => "SELECT count(*) FROM workspaces",
            "audit_log" => "SELECT count(*) FROM audit_log",
            _ => "SELECT count(*) FROM idempotency_keys",
        };
        let n: i64 = sqlx::query_scalar(sql).fetch_one(&app).await?;
        assert_eq!(n, 0, "{table} visible without context");
    }
    let err = sqlx::query("INSERT INTO roles (id, workspace_id, name) VALUES ($1, $2, 'x')")
        .bind(id(98))
        .bind(a)
        .execute(&app)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(RLS_VIOLATION));

    // The same connection after a transaction that set the context: the setting is now an
    // empty string, which must behave like "no context", not raise a cast error.
    let single = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .after_connect(|c, _| {
            Box::pin(async move {
                sqlx::query("SET ROLE app_api").execute(c).await?;
                Ok(())
            })
        })
        .connect_with((*pool.connect_options()).clone())
        .await?;
    let mut tx = single.begin().await?;
    set_context(&mut tx, Some(a), None).await?;
    let inside: i64 = sqlx::query_scalar("SELECT count(*) FROM roles")
        .fetch_one(&mut *tx)
        .await?;
    assert_eq!(inside, 1);
    tx.commit().await?;
    let after: i64 = sqlx::query_scalar("SELECT count(*) FROM roles")
        .fetch_one(&single)
        .await?;
    assert_eq!(after, 0);
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn application_role_cannot_switch_rls_off(pool: PgPool) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    two_workspaces(&app).await;

    // Role attributes: no superuser, no BYPASSRLS, not a table owner.
    for role in ["app_api", "app_worker", "app_owner"] {
        let (superuser, bypass, login): (bool, bool, bool) = sqlx::query_as(
            "SELECT rolsuper, rolbypassrls, rolcanlogin FROM pg_roles WHERE rolname = $1",
        )
        .bind(role)
        .fetch_one(&pool)
        .await?;
        assert!(!superuser && !bypass, "{role} is privileged");
        assert!(!login || role != "app_owner", "app_owner must not log in");
    }
    let owned: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_class c JOIN pg_roles r ON r.oid = c.relowner
         WHERE r.rolname IN ('app_api', 'app_worker')",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(owned, 0, "an application role owns a relation");
    let not_owner_owned: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_tables t
         WHERE t.schemaname = 'public' AND t.tablename <> '_sqlx_migrations'
           AND t.tableowner <> 'app_owner'",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(not_owner_owned, 0, "every table belongs to app_owner");

    // (`SET ROLE app_owner` would succeed here only because the test session itself is a
    // superuser; what matters is that the application roles are not members of the owner.)
    for role in ["app_api", "app_worker"] {
        let member: bool = sqlx::query_scalar("SELECT pg_has_role($1, 'app_owner', 'MEMBER')")
            .bind(role)
            .fetch_one(&pool)
            .await?;
        assert!(!member, "{role} is a member of app_owner");
    }

    // Attempts to get around RLS from the application role.
    for sql in [
        "ALTER TABLE roles DISABLE ROW LEVEL SECURITY",
        "ALTER TABLE roles NO FORCE ROW LEVEL SECURITY",
        "DROP POLICY workspace_isolation ON roles",
        "ALTER ROLE app_api BYPASSRLS",
    ] {
        let err = sqlx::query(sql).execute(&app).await.unwrap_err();
        assert!(
            sqlstate(&err).is_some_and(|c| c == "42501" || c == "42704"),
            "`{sql}` was not refused: {err}"
        );
    }

    // `row_security = off` must not turn into a bypass: under FORCE the query errors out.
    let mut conn = app.acquire().await?;
    sqlx::query("SET row_security = off")
        .execute(&mut *conn)
        .await?;
    let err = sqlx::query("SELECT count(*) FROM roles")
        .fetch_one(&mut *conn)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(RLS_VIOLATION));
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn membership_visibility_follows_the_adr_rule(pool: PgPool) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let (a, b, ..) = two_workspaces(&app).await;
    let (u1, u2) = (id(21), id(22));
    create_user(&app, u1).await?;
    create_user(&app, u2).await?;

    // Members are added inside each workspace's context: u1 joins A and B, u2 joins A.
    for (ws, user, member) in [(a, u1, id(31)), (b, u1, id(32)), (a, u2, id(33))] {
        let mut tx = app.begin().await?;
        set_context(&mut tx, Some(ws), None).await?;
        sqlx::query(
            "INSERT INTO workspace_members (id, workspace_id, user_id) VALUES ($1, $2, $3)",
        )
        .bind(member)
        .bind(ws)
        .bind(user)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
    }

    // A user without workspace context lists their own memberships in every workspace.
    let mut tx = app.begin().await?;
    set_context(&mut tx, None, Some(u1)).await?;
    let mine: Vec<sqlx::types::Uuid> =
        sqlx::query_scalar("SELECT id FROM workspace_members ORDER BY id")
            .fetch_all(&mut *tx)
            .await?;
    assert_eq!(mine, vec![id(31), id(32)]);
    // ... but cannot change them (writes need the workspace context).
    let updated = sqlx::query("UPDATE workspace_members SET status = 'disabled'")
        .execute(&mut *tx)
        .await?;
    assert_eq!(updated.rows_affected(), 0);
    let inserted = sqlx::query(
        "INSERT INTO workspace_members (id, workspace_id, user_id) VALUES ($1, $2, $3)",
    )
    .bind(id(34))
    .bind(b)
    .bind(u2)
    .execute(&mut *tx)
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&inserted).as_deref(), Some(RLS_VIOLATION));
    drop(tx);

    // Inside workspace A everyone in A is visible, nobody from B.
    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(a), None).await?;
    let in_a: Vec<sqlx::types::Uuid> =
        sqlx::query_scalar("SELECT id FROM workspace_members ORDER BY id")
            .fetch_all(&mut *tx)
            .await?;
    assert_eq!(in_a, vec![id(31), id(33)]);
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn composite_foreign_key_blocks_cross_workspace_references(pool: PgPool) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let (a, _b, _ra, rb) = two_workspaces(&app).await;
    let user = id(21);
    create_user(&app, user).await?;
    let member = id(31);

    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(a), None).await?;
    sqlx::query("INSERT INTO workspace_members (id, workspace_id, user_id) VALUES ($1, $2, $3)")
        .bind(member)
        .bind(a)
        .bind(user)
        .execute(&mut *tx)
        .await?;
    // The role belongs to B; RLS hides it, but the foreign key check bypasses RLS, so only
    // the composite key (workspace_id, role_id) stops the reference.
    let err = sqlx::query(
        "INSERT INTO role_assignments (id, workspace_id, member_id, role_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(id(41))
    .bind(a)
    .bind(member)
    .bind(rb)
    .execute(&mut *tx)
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(FOREIGN_KEY_VIOLATION));
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn version_and_updated_at_are_maintained_by_trigger(pool: PgPool) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let (a, _, ra, _) = two_workspaces(&app).await;
    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(a), None).await?;
    // The client cannot set the version itself: the trigger always adds one.
    sqlx::query("UPDATE roles SET name = 'Brewmaster', version = 100 WHERE id = $1")
        .bind(ra)
        .execute(&mut *tx)
        .await?;
    let (version, touched): (i64, bool) =
        sqlx::query_as("SELECT version, updated_at >= created_at FROM roles WHERE id = $1")
            .bind(ra)
            .fetch_one(&mut *tx)
            .await?;
    assert_eq!(version, 2);
    assert!(touched);
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn deleting_a_workspace_cascades_through_its_tables(pool: PgPool) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let (a, b, ..) = two_workspaces(&app).await;
    for ws in [a, b] {
        let mut tx = app.begin().await?;
        set_context(&mut tx, Some(ws), None).await?;
        sqlx::query(
            "INSERT INTO audit_log (id, workspace_id, seq, actor_kind, action, record_kind, prev_hash, hash)
             VALUES ($1, $2, 1, 'system', 'workspace.create', 'workspace', $3, $4)",
        )
        .bind(id(500 + u128::from(ws == b)))
        .bind(ws)
        .bind(vec![0u8; 32])
        .bind(vec![1u8; 32])
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
    }
    // Soft delete by the API, then the worker purges (T2.9: the API cannot delete).
    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(a), None).await?;
    sqlx::query("UPDATE workspaces SET deleted_at = now() WHERE id = $1")
        .bind(a)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    let worker = pool_as(&pool, Role::Worker).await;
    let purged: bool = sqlx::query_scalar("SELECT purge_workspace($1)")
        .bind(a)
        .fetch_one(&worker)
        .await?;
    assert!(purged);

    // Checked as the superuser: A's rows are gone (the append-only trigger lets the cascade
    // through), B's remain.
    let left: Vec<(sqlx::types::Uuid, i64)> = sqlx::query_as(
        "SELECT workspace_id, count(*) FROM (
             SELECT workspace_id FROM roles UNION ALL SELECT workspace_id FROM audit_log) t
         GROUP BY workspace_id",
    )
    .fetch_all(&pool)
    .await?;
    assert_eq!(left, vec![(b, 2)]);
    Ok(())
}
