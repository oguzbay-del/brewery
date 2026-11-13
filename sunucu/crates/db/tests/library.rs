//! Shared library and workspace-owned library rows (ADR-0007, rules 2.2 and 2.7; T2.6).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{Role, create_workspace, id, pool_as, run, set_context, sqlstate};
use sqlx::{PgPool, types::Uuid};

const RLS_VIOLATION: &str = "42501";
const FOREIGN_KEY_VIOLATION: &str = "23503";

/// Seeds a shared (library) ingredient the way migrations and seed jobs do: as `app_owner`.
async fn seed_shared_ingredient(pool: &PgPool, ingredient: Uuid, kind: &str) {
    let owner = pool_as(pool, Role::Owner).await;
    sqlx::query(
        "INSERT INTO ingredients (id, workspace_id, kind, name, source)
         VALUES ($1, NULL, $2, 'Pale Malt', 'library')",
    )
    .bind(ingredient)
    .bind(kind)
    .execute(&owner)
    .await
    .unwrap();
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn shared_rows_are_readable_by_everyone_but_not_writable_by_the_application(
    pool: PgPool,
) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let (a, b) = (id(1), id(2));
    create_workspace(&app, a, "A").await?;
    create_workspace(&app, b, "B").await?;
    let shared = id(100);
    seed_shared_ingredient(&pool, shared, "fermentable").await;

    // Both workspaces read the shared row.
    for ws in [a, b] {
        let mut tx = app.begin().await?;
        set_context(&mut tx, Some(ws), None).await?;
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM ingredients WHERE id = $1")
            .bind(shared)
            .fetch_one(&mut *tx)
            .await?;
        assert_eq!(n, 1);
        tx.rollback().await?;
    }

    // The application cannot insert, update or delete a shared row.
    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(a), None).await?;
    let insert = sqlx::query(
        "INSERT INTO ingredients (id, workspace_id, kind, name, source)
         VALUES ($1, NULL, 'hop', 'Sneaky', 'library')",
    )
    .bind(id(101))
    .execute(&mut *tx)
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&insert).as_deref(), Some(RLS_VIOLATION));
    drop(tx);

    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(a), None).await?;
    let updated = sqlx::query("UPDATE ingredients SET name = 'Hijacked' WHERE id = $1")
        .bind(shared)
        .execute(&mut *tx)
        .await?;
    assert_eq!(updated.rows_affected(), 0);
    let deleted = sqlx::query("DELETE FROM ingredients WHERE id = $1")
        .bind(shared)
        .execute(&mut *tx)
        .await?;
    assert_eq!(deleted.rows_affected(), 0);
    tx.rollback().await?;

    // A user row cannot claim to be a library row, and a library row needs no workspace.
    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(a), None).await?;
    let bad = sqlx::query(
        "INSERT INTO ingredients (id, workspace_id, kind, name, source)
         VALUES ($1, $2, 'hop', 'Mine', 'library')",
    )
    .bind(id(102))
    .bind(a)
    .execute(&mut *tx)
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&bad).as_deref(), Some("23514"));
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn workspace_library_rows_stay_private(pool: PgPool) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let (a, b) = (id(1), id(2));
    create_workspace(&app, a, "A").await?;
    create_workspace(&app, b, "B").await?;
    run(
        &app,
        a,
        None,
        &format!(
            "INSERT INTO ingredients (id, workspace_id, kind, name, source)
             VALUES ('{}', '{a}', 'hop', 'My Hop', 'user');
             INSERT INTO ingredient_hops (id, workspace_id, alpha_pct) VALUES ('{}', '{a}', 12.5);
             INSERT INTO style_guides (id, workspace_id, name) VALUES ('{}', '{a}', 'House');",
            id(110),
            id(110),
            id(111)
        ),
    )
    .await?;

    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(b), None).await?;
    for table in ["ingredients", "ingredient_hops", "style_guides"] {
        let n: i64 =
            sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT count(*) FROM {table}")))
                .fetch_one(&mut *tx)
                .await?;
        assert_eq!(n, 0, "{table} leaks across workspaces");
    }
    tx.rollback().await?;
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn references_to_ingredients_must_be_shared_or_in_the_same_workspace(
    pool: PgPool,
) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let (a, b) = (id(1), id(2));
    create_workspace(&app, a, "A").await?;
    create_workspace(&app, b, "B").await?;
    let shared = id(100);
    seed_shared_ingredient(&pool, shared, "fermentable").await;
    let a_private = id(110);
    run(
        &app,
        a,
        None,
        &format!(
            "INSERT INTO ingredients (id, workspace_id, kind, name, source)
             VALUES ('{a_private}', '{a}', 'misc', 'Private', 'user');"
        ),
    )
    .await?;
    common::create_recipe(&app, b, 10, "draft", None).await?;

    let line = |ingredient: Uuid, n: u128| {
        format!(
            "INSERT INTO recipe_fermentables (id, workspace_id, recipe_version_id, position,
                 ingredient_id, name, amount_kg, grain_type, yield_pct)
             VALUES ('{}', '{b}', '{}', {n}, '{ingredient}', 'Pale', 4, 'grain', 80);",
            id(200 + n),
            id(11)
        )
    };
    // Shared: fine. Another workspace's private row: rejected by the trigger although the
    // single-column foreign key would accept it.
    run(&app, b, None, &line(shared, 0)).await?;
    let err = run(&app, b, None, &line(a_private, 1)).await.unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(FOREIGN_KEY_VIOLATION));

    // Same trigger on stock lots and style references.
    let err = run(
        &app,
        b,
        None,
        &format!(
            "INSERT INTO stock_lots (id, workspace_id, ingredient_id)
             VALUES ('{}', '{b}', '{a_private}')",
            id(300)
        ),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(FOREIGN_KEY_VIOLATION));
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn type_tables_must_match_their_ingredient(pool: PgPool) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let a = id(1);
    create_workspace(&app, a, "A").await?;
    run(
        &app,
        a,
        None,
        &format!(
            "INSERT INTO ingredients (id, workspace_id, kind, name, source)
             VALUES ('{}', '{a}', 'fermentable', 'Malt', 'user');",
            id(110)
        ),
    )
    .await?;
    // A hop row for a fermentable ingredient is refused; so is one without an ingredient.
    for ingredient in [id(110), id(999)] {
        let err = run(
            &app,
            a,
            None,
            &format!(
                "INSERT INTO ingredient_hops (id, workspace_id, alpha_pct)
                 VALUES ('{ingredient}', '{a}', 5)"
            ),
        )
        .await
        .unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(FOREIGN_KEY_VIOLATION));
    }
    run(
        &app,
        a,
        None,
        &format!(
            "INSERT INTO ingredient_fermentables (id, workspace_id, grain_type, yield_pct)
             VALUES ('{}', '{a}', 'grain', 80)",
            id(110)
        ),
    )
    .await
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn styles_reference_visible_guides_only(pool: PgPool) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let (a, b) = (id(1), id(2));
    create_workspace(&app, a, "A").await?;
    create_workspace(&app, b, "B").await?;
    run(
        &app,
        a,
        None,
        &format!(
            "INSERT INTO style_guides (id, workspace_id, name) VALUES ('{}', '{a}', 'A guide')",
            id(120)
        ),
    )
    .await?;
    let err = run(
        &app,
        b,
        None,
        &format!(
            "INSERT INTO styles (id, workspace_id, guide_id, name) VALUES ('{}', '{b}', '{}', 'X')",
            id(121),
            id(120)
        ),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(FOREIGN_KEY_VIOLATION));
    Ok(())
}
