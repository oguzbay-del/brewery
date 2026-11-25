//! Container schema, tenant isolation, fill guard and append-only history tests.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{Role, create_workspace, id, pool_as, run, set_context, sqlstate};
use sqlx::PgPool;

async fn setup(pool: &PgPool) -> (PgPool, uuid::Uuid, uuid::Uuid, uuid::Uuid, uuid::Uuid) {
    let api = pool_as(pool, Role::Api).await;
    let owner = pool_as(pool, Role::Owner).await;
    let package_type = id(900);
    sqlx::query(
        "INSERT INTO package_types (id, workspace_id, kind, name, volume_l, single_use)
         VALUES ($1, NULL, 'can', 'Test can', 0.33, false)",
    )
    .bind(package_type)
    .execute(&owner)
    .await
    .unwrap();
    let workspace_a = id(901);
    let workspace_b = id(902);
    create_workspace(&api, workspace_a, "A").await.unwrap();
    create_workspace(&api, workspace_b, "B").await.unwrap();
    let container_a = id(903);
    let container_b = id(904);
    for (workspace, container_id, serial) in [
        (workspace_a, container_a, "CAN-A"),
        (workspace_b, container_b, "CAN-B"),
    ] {
        run(
            &api,
            workspace,
            None,
            &format!(
                "INSERT INTO container (id, workspace_id, serial_number, package_type_id)
                 VALUES ('{container_id}', '{workspace}', '{serial}', '{package_type}')"
            ),
        )
        .await
        .unwrap();
    }
    (api, workspace_a, workspace_b, container_a, container_b)
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn containers_are_isolated_by_workspace(pool: PgPool) -> sqlx::Result<()> {
    let (api, workspace_a, _workspace_b, container_a, _container_b) = setup(&pool).await;
    let mut tx = api.begin().await?;
    set_context(&mut tx, Some(workspace_a), None).await?;
    let rows: Vec<(uuid::Uuid, String)> =
        sqlx::query_as("SELECT id, serial_number FROM container ORDER BY serial_number")
            .fetch_all(&mut *tx)
            .await?;
    assert_eq!(rows, [(container_a, "CAN-A".to_owned())]);
    tx.rollback().await?;
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_dirty_container_cannot_be_filled(pool: PgPool) -> sqlx::Result<()> {
    let (api, workspace_a, _workspace_b, container_id, _container_b) = setup(&pool).await;
    run(&api, workspace_a, None,
        &format!("UPDATE container SET status = 'dirty' WHERE id = '{container_id}'")).await?;
    let error = run(&api, workspace_a, None,
        &format!("UPDATE container SET status = 'filled' WHERE id = '{container_id}'"))
        .await.unwrap_err();
    assert_eq!(sqlstate(&error).as_deref(), Some("23000"));
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn container_events_are_append_only_for_application_roles_and_owner(pool: PgPool) -> sqlx::Result<()> {
    let (api, workspace_a, _workspace_b, container_id, _container_b) = setup(&pool).await;
    let event_id = id(905);
    run(&api, workspace_a, None, &format!(
        "INSERT INTO container_event (id, workspace_id, container_id, event_type, note)
         VALUES ('{event_id}', '{workspace_a}', '{container_id}', 'scan', 'test scan')"
    )).await?;

    for role in [Role::Api, Role::Worker] {
        let app = pool_as(&pool, role).await;
        let update = run(&app, workspace_a, None,
            &format!("UPDATE container_event SET note = 'changed' WHERE id = '{event_id}'"))
            .await.unwrap_err();
        assert_eq!(sqlstate(&update).as_deref(), Some("42501"));
        let delete = run(&app, workspace_a, None,
            &format!("DELETE FROM container_event WHERE id = '{event_id}'"))
            .await.unwrap_err();
        assert_eq!(sqlstate(&delete).as_deref(), Some("42501"));
    }

    let owner = pool_as(&pool, Role::Owner).await;
    let update = run(&owner, workspace_a, None,
        &format!("UPDATE container_event SET note = 'changed' WHERE id = '{event_id}'"))
        .await.unwrap_err();
    assert_eq!(sqlstate(&update).as_deref(), Some("23000"));
    let delete = run(&owner, workspace_a, None,
        &format!("DELETE FROM container_event WHERE id = '{event_id}'"))
        .await.unwrap_err();
    assert_eq!(sqlstate(&delete).as_deref(), Some("23000"));
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn scan_codes_have_one_target_and_can_be_revoked(pool: PgPool) -> sqlx::Result<()> {
    let (api, workspace_a, _workspace_b, container_id, _container_b) = setup(&pool).await;
    let code_id = id(906);
    run(&api, workspace_a, None, &format!(
        "INSERT INTO scan_code (id, workspace_id, code_hash, container_id)
         VALUES ('{code_id}', '{workspace_a}', decode(repeat('ab', 32), 'hex'), '{container_id}');
         UPDATE scan_code SET revoked_at = now() WHERE id = '{code_id}'"
    )).await?;
    let error = run(&api, workspace_a, None, &format!(
        "INSERT INTO scan_code (id, workspace_id, code_hash, container_id, vessel_id)
         VALUES ('{}', '{workspace_a}', decode(repeat('cd', 32), 'hex'), '{container_id}', '{}')",
        id(907), id(908)
    )).await.unwrap_err();
    assert_eq!(sqlstate(&error).as_deref(), Some("23514"));
    Ok(())
}
