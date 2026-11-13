//! Immutable recipe versions (ADR-0007 › 3 "Değişmezlik"; T2.6).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{Role, create_recipe, create_user, create_workspace, id, pool_as, run, sqlstate};
use sqlx::{PgPool, types::Uuid};

const INTEGRITY: &str = "23000";
const UNIQUE_VIOLATION: &str = "23505";
const FOREIGN_KEY_VIOLATION: &str = "23503";

fn line(n: u128, ws: Uuid, version: Uuid, position: u32) -> String {
    format!(
        "INSERT INTO recipe_hops (id, workspace_id, recipe_version_id, position, name, amount_kg, alpha_pct)
         VALUES ('{}', '{ws}', '{version}', {position}, 'Cascade', 0.03, 6.5);",
        id(n)
    )
}

async fn setup(pool: &PgPool) -> (PgPool, Uuid, Uuid) {
    let app = pool_as(pool, Role::Api).await;
    let (ws, user) = (id(1), id(2));
    create_workspace(&app, ws, "A").await.unwrap();
    create_user(&app, user).await.unwrap();
    (app, ws, user)
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_draft_is_editable_and_a_published_version_is_not(pool: PgPool) -> sqlx::Result<()> {
    let (app, ws, user) = setup(&pool).await;
    create_recipe(&app, ws, 10, "draft", None).await?;
    let version = id(11);

    // Draft: content and lines change freely (the client saves every ~300 ms).
    run(
        &app,
        ws,
        None,
        "UPDATE recipe_versions SET batch_volume_l = 25, change_note = 'x'",
    )
    .await?;
    run(&app, ws, None, &line(20, ws, version, 0)).await?;
    run(&app, ws, None, "UPDATE recipe_hops SET amount_kg = 0.05").await?;

    // Publish (draft -> approved) is the last change allowed.
    run(
        &app,
        ws,
        Some(user),
        &format!(
            "UPDATE recipe_versions SET status = 'approved', approved_by = '{user}', approved_at = now()"
        ),
    )
    .await?;

    for sql in [
        "UPDATE recipe_versions SET batch_volume_l = 30".to_owned(),
        "UPDATE recipe_versions SET options = '{\"schema_version\": 2}'".to_owned(),
        "UPDATE recipe_versions SET status = 'draft'".to_owned(),
        "DELETE FROM recipe_versions".to_owned(),
        "UPDATE recipe_hops SET amount_kg = 1".to_owned(),
        "DELETE FROM recipe_hops".to_owned(),
        line(21, ws, version, 1),
    ] {
        let err = run(&app, ws, None, &sql).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY), "{sql}");
    }

    // Archiving is the one allowed transition; afterwards nothing moves, not even back.
    run(
        &app,
        ws,
        None,
        "UPDATE recipe_versions SET status = 'archived', archived_at = now()",
    )
    .await?;
    for sql in [
        "UPDATE recipe_versions SET status = 'approved'",
        "UPDATE recipe_versions SET change_note = 'late'",
    ] {
        let err = run(&app, ws, None, sql).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY), "{sql}");
    }
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn in_review_is_frozen_but_can_return_to_draft_or_be_approved(
    pool: PgPool,
) -> sqlx::Result<()> {
    let (app, ws, user) = setup(&pool).await;
    create_recipe(&app, ws, 10, "draft", None).await?;
    run(
        &app,
        ws,
        None,
        "UPDATE recipe_versions SET status = 'in_review', submitted_at = now()",
    )
    .await?;

    let err = run(
        &app,
        ws,
        None,
        "UPDATE recipe_versions SET batch_volume_l = 99",
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY));
    let err = run(&app, ws, None, &line(20, ws, id(11), 0))
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY));

    // Rejected review: back to draft, editable again.
    run(
        &app,
        ws,
        None,
        "UPDATE recipe_versions SET status = 'draft'",
    )
    .await?;
    run(
        &app,
        ws,
        None,
        "UPDATE recipe_versions SET batch_volume_l = 22",
    )
    .await?;
    run(
        &app,
        ws,
        None,
        "UPDATE recipe_versions SET status = 'in_review'",
    )
    .await?;
    run(
        &app,
        ws,
        Some(user),
        &format!(
            "UPDATE recipe_versions SET status = 'approved', approved_by = '{user}', approved_at = now()"
        ),
    )
    .await
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_recipe_has_at_most_one_draft_and_versions_stay_in_the_workspace(
    pool: PgPool,
) -> sqlx::Result<()> {
    let (app, a, _) = setup(&pool).await;
    let b = id(3);
    create_workspace(&app, b, "B").await?;
    create_recipe(&app, a, 10, "draft", None).await?;
    let second_draft = format!(
        "INSERT INTO recipe_versions (id, workspace_id, recipe_id, number, batch_volume_l, efficiency_pct)
         VALUES ('{}', '{a}', '{}', 2, 20, 70)",
        id(30),
        id(10)
    );
    let err = run(&app, a, None, &second_draft).await.unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(UNIQUE_VIOLATION));

    // A version in B pointing at A's recipe: the composite key refuses.
    let err = run(
        &app,
        b,
        None,
        &format!(
            "INSERT INTO recipe_versions (id, workspace_id, recipe_id, number, batch_volume_l, efficiency_pct)
             VALUES ('{}', '{b}', '{}', 1, 20, 70)",
            id(31),
            id(10)
        ),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(FOREIGN_KEY_VIOLATION));
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn deleting_a_recipe_removes_its_published_versions(pool: PgPool) -> sqlx::Result<()> {
    let (app, ws, user) = setup(&pool).await;
    create_recipe(&app, ws, 10, "approved", Some(user)).await?;
    // Purge job (worker): the foreign-key chain may delete what a direct DELETE cannot.
    let worker = pool_as(&pool, Role::Worker).await;
    run(
        &worker,
        ws,
        None,
        &format!("DELETE FROM recipes WHERE id = '{}'", id(10)),
    )
    .await?;
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM recipe_versions")
        .fetch_one(&pool)
        .await?;
    assert_eq!(left, 0);
    Ok(())
}
