//! Fixes for the F2.1 findings (T2.9): recipe line race (1), stock quantity and count (2, 3),
//! permanent deletion (5, 6, 7, 8), mandatory retention (11), external event payload (10),
//! measurement history (12), push token takeover (13).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::time::Duration;

use common::{
    Role, create_recipe, create_user, create_workspace, id, pool_as, run, set_context, sqlstate,
};
use db::{ActorKind, AuditEvent, NewAuditEntry, append_audit};
use sqlx::{PgPool, types::Uuid};

const INTEGRITY: &str = "23000";
const CHECK_VIOLATION: &str = "23514";
const PERMISSION_DENIED: &str = "42501";

/// Workspace `ws` (id 1, business) with user (id 2), an approved recipe version (id 11) and
/// batch (id 20).
async fn with_batch(pool: &PgPool) -> (PgPool, Uuid, Uuid) {
    let app = pool_as(pool, Role::Api).await;
    let (ws, user) = (id(1), id(2));
    create_workspace(&app, ws, "A").await.unwrap();
    create_user(&app, user).await.unwrap();
    create_recipe(&app, ws, 10, "approved", Some(user))
        .await
        .unwrap();
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO batches (id, workspace_id, code, name, recipe_version_id)
             VALUES ('{}', '{ws}', 'B-001', 'First', '{}')",
            id(20),
            id(11)
        ),
    )
    .await
    .unwrap();
    (app, ws, user)
}

/// Soft deletes the workspace as the application (the API's part of a deletion).
async fn soft_delete_workspace(app: &PgPool, ws: Uuid) {
    run(
        app,
        ws,
        None,
        &format!("UPDATE workspaces SET deleted_at = now() WHERE id = '{ws}'"),
    )
    .await
    .unwrap();
}

async fn purge(worker: &PgPool, ws: Uuid) -> sqlx::Result<()> {
    sqlx::query("SELECT purge_workspace($1, $2, 'test')")
        .bind(ws)
        .bind(id(2))
        .execute(worker)
        .await
        .map(|_| ())
}

// --- finding 1 ----------------------------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_line_insert_waits_for_a_concurrent_publish_and_is_refused(
    pool: PgPool,
) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let (ws, user) = (id(1), id(2));
    create_workspace(&app, ws, "A").await?;
    create_user(&app, user).await?;
    create_recipe(&app, ws, 10, "draft", None).await?;

    // Device A publishes (not committed yet).
    let mut publish = app.begin().await?;
    set_context(&mut publish, Some(ws), None).await?;
    sqlx::query(
        "UPDATE recipe_versions SET status = 'approved', approved_by = $1, approved_at = now()
         WHERE id = $2",
    )
    .bind(user)
    .bind(id(11))
    .execute(&mut *publish)
    .await?;

    // Device B adds a line at the same moment.
    let line = format!(
        "INSERT INTO recipe_hops (id, workspace_id, recipe_version_id, position, name, amount_kg, alpha_pct)
         VALUES ('{}', '{ws}', '{}', 0, 'Late', 0.03, 6.5)",
        id(30),
        id(11)
    );
    let writer = {
        let app = app.clone();
        tokio::spawn(async move { run(&app, ws, None, &line).await })
    };
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(
        !writer.is_finished(),
        "the insert must wait for the publish transaction"
    );
    publish.commit().await?;
    let err = writer.await.unwrap().unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY));
    let lines: i64 = sqlx::query_scalar("SELECT count(*) FROM recipe_hops")
        .fetch_one(&pool)
        .await?;
    assert_eq!(lines, 0, "a published version never gets a late line");
    Ok(())
}

// --- findings 2 and 3 ---------------------------------------------------------------------

async fn with_lot(pool: &PgPool) -> (PgPool, Uuid, Uuid) {
    let (app, ws, _) = with_batch(pool).await;
    let owner = pool_as(pool, Role::Owner).await;
    sqlx::query(
        "INSERT INTO ingredients (id, workspace_id, kind, name, source)
         VALUES ($1, NULL, 'fermentable', 'Pale Malt', 'library')",
    )
    .bind(id(100))
    .execute(&owner)
    .await
    .unwrap();
    let lot = id(110);
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO stock_lots (id, workspace_id, ingredient_id)
             VALUES ('{lot}', '{ws}', '{}')",
            id(100)
        ),
    )
    .await
    .unwrap();
    (app, ws, lot)
}

fn movement(ws: Uuid, lot: Uuid, n: u128, kind: &str, qty: f64) -> String {
    format!(
        "INSERT INTO stock_movements (id, workspace_id, lot_id, kind, quantity)
         VALUES ('{}', '{ws}', '{lot}', '{kind}', {qty})",
        id(n)
    )
}

fn count_movement(ws: Uuid, lot: Uuid, n: u128, counted: f64) -> String {
    format!(
        "INSERT INTO stock_movements (id, workspace_id, lot_id, kind, quantity, counted_quantity)
         VALUES ('{}', '{ws}', '{lot}', 'count', 0, {counted})",
        id(n)
    )
}

async fn lot_quantity(pool: &PgPool) -> f64 {
    sqlx::query_scalar("SELECT quantity FROM stock_lots")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_lot_cannot_start_with_a_quantity(pool: PgPool) -> sqlx::Result<()> {
    let (app, ws, _) = with_lot(&pool).await;
    let err = run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO stock_lots (id, workspace_id, ingredient_id, lot_code, quantity)
             VALUES ('{}', '{ws}', '{}', 'L2', 1000)",
            id(111),
            id(100)
        ),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY));
    // The opening quantity comes in as a receive movement.
    run(&app, ws, None, &movement(ws, id(110), 120, "receive", 25.0)).await?;
    assert!((lot_quantity(&pool).await - 25.0).abs() < 1e-9);
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_count_sets_the_counted_value_whatever_happened_in_between(
    pool: PgPool,
) -> sqlx::Result<()> {
    let (app, ws, lot) = with_lot(&pool).await;
    run(&app, ws, None, &movement(ws, lot, 120, "receive", 15.0)).await?;

    // The storekeeper reads 15.0, counts 12.0 ... meanwhile a brew consumes 2.0.
    let mut brew = app.begin().await?;
    set_context(&mut brew, Some(ws), None).await?;
    sqlx::query(sqlx::AssertSqlSafe(movement(ws, lot, 121, "consume", -2.0)))
        .execute(&mut *brew)
        .await?;
    let counter = {
        let (app, sql) = (app.clone(), count_movement(ws, lot, 122, 12.0));
        tokio::spawn(async move { run(&app, ws, None, &sql).await })
    };
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!counter.is_finished(), "the count waits for the lot lock");
    brew.commit().await?;
    counter.await.unwrap()?;
    assert!(
        (lot_quantity(&pool).await - 12.0).abs() < 1e-9,
        "the lot holds the counted value"
    );
    let delta: f64 =
        sqlx::query_scalar("SELECT quantity FROM stock_movements WHERE kind = 'count'")
            .fetch_one(&pool)
            .await?;
    assert!(
        (delta - -1.0).abs() < 1e-9,
        "the trigger computed the delta"
    );

    // A count needs the counted value; other kinds cannot carry it.
    for sql in [
        movement(ws, lot, 123, "count", -3.0),
        format!(
            "INSERT INTO stock_movements (id, workspace_id, lot_id, kind, quantity, counted_quantity)
             VALUES ('{}', '{ws}', '{lot}', 'adjust', 1, 5)",
            id(124)
        ),
    ] {
        let err = run(&app, ws, None, &sql).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(CHECK_VIOLATION), "{sql}");
    }
    Ok(())
}

// --- findings 5, 6, 8 ---------------------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn only_the_worker_deletes_a_workspace_and_a_tombstone_stays(
    pool: PgPool,
) -> sqlx::Result<()> {
    let (app, a, _) = with_batch(&pool).await;
    let worker = pool_as(&pool, Role::Worker).await;
    let b = id(3);
    create_workspace(&app, b, "B").await?;

    // The API has no DELETE right on workspaces and users any more.
    for sql in [
        format!("DELETE FROM workspaces WHERE id = '{a}'"),
        "DELETE FROM users".to_owned(),
    ] {
        let err = run(&app, a, Some(id(2)), &sql).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED), "{sql}");
    }

    // The worker lists workspaces without a context; the API cannot call the function.
    let listed: Vec<(Uuid, Option<chrono::DateTime<chrono::Utc>>)> =
        sqlx::query_as("SELECT id, deleted_at FROM list_workspace_ids()")
            .fetch_all(&worker)
            .await?;
    assert_eq!(listed.len(), 2);
    let err = sqlx::query("SELECT * FROM list_workspace_ids()")
        .execute(&app)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));
    let err = purge(&app, a).await.unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));

    // A workspace that is not soft deleted cannot be purged.
    let err = purge(&worker, a).await.unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY));

    // Chain of A: one audit row; the head is what the tombstone keeps.
    {
        let mut tx = app.begin().await?;
        set_context(&mut tx, Some(a), None).await?;
        append_audit(
            &mut tx,
            a,
            &NewAuditEntry {
                actor_kind: ActorKind::User,
                actor_id: Some(id(2)),
                event: AuditEvent::RoleUpdate,
                record_kind: "role".into(),
                record_id: Some(id(600)),
                previous_value: None,
                new_value: None,
                reason: None,
            },
        )
        .await
        .unwrap();
        tx.commit().await?;
    }
    let (head_seq, head_hash): (i64, Vec<u8>) =
        sqlx::query_as("SELECT last_seq, last_hash FROM audit_heads WHERE workspace_id = $1")
            .bind(a)
            .fetch_one(&pool)
            .await?;
    soft_delete_workspace(&app, a).await;
    let listed: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM list_workspace_ids(true)")
        .fetch_all(&worker)
        .await?;
    assert_eq!(listed, vec![a]);

    purge(&worker, a).await?;
    let gone: i64 = sqlx::query_scalar(
        "SELECT (SELECT count(*) FROM workspaces WHERE id = $1)
              + (SELECT count(*) FROM batches WHERE workspace_id = $1)
              + (SELECT count(*) FROM audit_log WHERE workspace_id = $1)",
    )
    .bind(a)
    .fetch_one(&pool)
    .await?;
    assert_eq!(gone, 0);
    let kept: i64 = sqlx::query_scalar("SELECT count(*) FROM workspaces WHERE id = $1")
        .bind(b)
        .fetch_one(&pool)
        .await?;
    assert_eq!(kept, 1, "the other workspace is untouched");

    let tomb: (Uuid, String, Option<Uuid>, i64, Vec<u8>, i32) = sqlx::query_as(
        "SELECT former_workspace_id, mode, deleted_by, audit_last_seq, audit_last_hash,
                support_action_count
         FROM workspace_tombstones",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(tomb.0, a);
    assert_eq!(tomb.1, "business");
    assert_eq!(tomb.2, Some(id(2)));
    assert_eq!((tomb.3, tomb.4), (head_seq, head_hash));
    assert_eq!(tomb.5, 0);

    // The record is append-only and closed to the application roles.
    for sql in [
        "UPDATE workspace_tombstones SET reason = 'x'",
        "DELETE FROM workspace_tombstones",
        "TRUNCATE workspace_tombstones",
    ] {
        let err = sqlx::query(sql).execute(&pool).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY), "{sql}");
    }
    for role in [Role::Api, Role::Worker] {
        let err = sqlx::query("SELECT * FROM workspace_tombstones")
            .execute(&pool_as(&pool, role).await)
            .await
            .unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));
    }
    Ok(())
}

// --- finding 11 ---------------------------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn retention_and_legal_hold_block_deletion_of_production_records(
    pool: PgPool,
) -> sqlx::Result<()> {
    let (app, ws, _) = with_batch(&pool).await;
    let worker = pool_as(&pool, Role::Worker).await;
    let batch = id(20);

    // Retention in the future: no soft delete, no purge of the workspace.
    run(
        &app,
        ws,
        None,
        &format!(
            "UPDATE batches SET retention_until = now() + interval '1 year' WHERE id = '{batch}'"
        ),
    )
    .await?;
    let err = run(
        &app,
        ws,
        None,
        &format!("UPDATE batches SET deleted_at = now() WHERE id = '{batch}'"),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY));
    soft_delete_workspace(&app, ws).await;
    let err = purge(&worker, ws).await.unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY));
    let tombs: i64 = sqlx::query_scalar("SELECT count(*) FROM workspace_tombstones")
        .fetch_one(&pool)
        .await?;
    assert_eq!(tombs, 0, "a refused purge leaves no tombstone");

    // A release record under an open-ended hold blocks the same way.
    run(
        &app,
        ws,
        None,
        &format!(
            "UPDATE batches SET retention_until = now() - interval '1 day' WHERE id = '{batch}';
             INSERT INTO releases (id, workspace_id, batch_id, decision, decided_by, legal_hold)
             VALUES ('{}', '{ws}', '{batch}', 'released', '{}', true);",
            id(40),
            id(2)
        ),
    )
    .await?;
    let err = purge(&worker, ws).await.unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY));

    // Hold lifted (an administrative action: the API cannot update the append-only table)
    // and retention over: the purge goes through.
    sqlx::query("UPDATE releases SET legal_hold = false")
        .execute(&pool)
        .await?;
    purge(&worker, ws).await?;
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM batches")
        .fetch_one(&pool)
        .await?;
    assert_eq!(left, 0);
    Ok(())
}

// --- finding 7 ----------------------------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn permanent_delete_turns_immutable_references_into_snapshots(
    pool: PgPool,
) -> sqlx::Result<()> {
    let (app, ws, user) = with_batch(&pool).await;
    let owner = pool_as(&pool, Role::Owner).await;
    let worker = pool_as(&pool, Role::Worker).await;
    let (equipment, device, ingredient, lot) = (id(50), id(51), id(52), id(53));

    sqlx::query(
        "INSERT INTO ingredients (id, workspace_id, kind, name, source)
         VALUES ($1, $2, 'fermentable', 'Custom Malt', 'user')",
    )
    .bind(ingredient)
    .bind(ws)
    .execute(&owner)
    .await?;
    // A draft version points at the equipment, then it is published; the batch points at it too.
    create_recipe(&app, ws, 60, "draft", None).await?;
    run(
        &app,
        ws,
        Some(user),
        &format!(
            "INSERT INTO equipment (id, workspace_id, name, batch_volume_l, efficiency_pct)
                 VALUES ('{equipment}', '{ws}', 'Kettle 1', 20, 72);
             INSERT INTO recipe_fermentables (id, workspace_id, recipe_version_id, position, ingredient_id,
                 name, amount_kg, usage, grain_type, yield_pct)
                 VALUES ('{}', '{ws}', '{}', 0, '{ingredient}', 'Custom Malt', 4, 'mash', 'grain', 80);
             UPDATE recipe_versions SET equipment_id = '{equipment}' WHERE id = '{}';
             UPDATE recipe_versions SET status = 'approved', approved_by = '{user}', approved_at = now()
                 WHERE id = '{}';
             UPDATE batches SET equipment_id = '{equipment}' WHERE id = '{}';
             INSERT INTO devices (id, workspace_id, kind, name, ingest_key_hash, batch_id)
                 VALUES ('{device}', '{ws}', 'ispindel', 'Spindel 1',
                         decode(repeat('ab', 32), 'hex'), '{}');
             INSERT INTO measurements (id, workspace_id, batch_id, kind, value, measured_at, source, device_id)
                 VALUES ('{}', '{ws}', '{}', 'sg', 1.050, '2026-03-01 10:00:00+00', 'device', '{device}');",
            id(54),
            id(61),
            id(61),
            id(61),
            id(20),
            id(20),
            id(55),
            id(20),
        ),
    )
    .await?;
    // A stock movement of the batch (ledger) and a yeast generation sourced from it.
    sqlx::query(
        "INSERT INTO ingredients (id, workspace_id, kind, name, source)
         VALUES ($1, NULL, 'fermentable', 'Pale Malt', 'library')",
    )
    .bind(id(100))
    .execute(&owner)
    .await?;
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO stock_lots (id, workspace_id, ingredient_id) VALUES ('{lot}', '{ws}', '{}');
             INSERT INTO stock_movements (id, workspace_id, lot_id, kind, quantity, batch_id)
                 VALUES ('{}', '{ws}', '{lot}', 'receive', 5, '{}');",
            id(100),
            id(56),
            id(20)
        ),
    )
    .await?;

    // Permanent delete as the worker, in the workspace context. Before the fix every one of
    // these fails with a foreign key violation.
    for sql in [
        format!("DELETE FROM equipment WHERE id = '{equipment}'"),
        format!("DELETE FROM devices WHERE id = '{device}'"),
        format!("DELETE FROM ingredients WHERE id = '{ingredient}'"),
    ] {
        run(&worker, ws, None, &sql).await?;
    }
    let (version_equipment, version_name): (Option<Uuid>, Option<String>) = sqlx::query_as(
        "SELECT equipment_id, equipment_snapshot ->> 'schema_version' FROM recipe_versions WHERE id = $1",
    )
    .bind(id(61))
    .fetch_one(&pool)
    .await?;
    assert_eq!(
        version_equipment, None,
        "the published version keeps no reference"
    );
    assert!(version_name.is_some(), "its own equipment snapshot stays");
    let batch_snapshot: serde_json::Value =
        sqlx::query_scalar("SELECT equipment_snapshot FROM batches WHERE id = $1")
            .bind(id(20))
            .fetch_one(&pool)
            .await?;
    assert_eq!(batch_snapshot["name"], "Kettle 1");
    let (device_id, device_snapshot): (Option<Uuid>, serde_json::Value) =
        sqlx::query_as("SELECT device_id, device_snapshot FROM measurements")
            .fetch_one(&pool)
            .await?;
    assert_eq!(device_id, None);
    assert_eq!(device_snapshot["name"], "Spindel 1");
    let line: Option<Uuid> = sqlx::query_scalar("SELECT ingredient_id FROM recipe_fermentables")
        .fetch_one(&pool)
        .await?;
    assert_eq!(line, None);

    // Deleting the batch keeps the ledger row with a snapshot of the batch.
    run(
        &worker,
        ws,
        None,
        &format!("DELETE FROM batches WHERE id = '{}'", id(20)),
    )
    .await?;
    let (batch_id, snapshot): (Option<Uuid>, serde_json::Value) =
        sqlx::query_as("SELECT batch_id, batch_snapshot FROM stock_movements")
            .fetch_one(&pool)
            .await?;
    assert_eq!(batch_id, None);
    assert_eq!(snapshot["code"], "B-001");
    Ok(())
}

// --- finding 10 ---------------------------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn the_api_cannot_read_event_payloads_and_a_processed_payload_is_blanked(
    pool: PgPool,
) -> sqlx::Result<()> {
    let api = pool_as(&pool, Role::Api).await;
    let worker = pool_as(&pool, Role::Worker).await;
    sqlx::query(
        "INSERT INTO external_events (id, source, delivery_id, payload)
         VALUES ($1, 'github', 'd-1', '{\"body\": \"customer text\"}')",
    )
    .bind(id(300))
    .execute(&api)
    .await?;
    let err = sqlx::query("SELECT payload FROM external_events")
        .execute(&api)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));
    let seen: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM external_events WHERE source = 'github' AND delivery_id = 'd-1'",
    )
    .fetch_one(&api)
    .await?;
    assert_eq!(seen, 1, "the duplicate check still works");

    sqlx::query("UPDATE external_events SET processed_at = now() WHERE id = $1")
        .bind(id(300))
        .execute(&worker)
        .await?;
    let payload: serde_json::Value = sqlx::query_scalar("SELECT payload FROM external_events")
        .fetch_one(&worker)
        .await?;
    assert_eq!(payload, serde_json::json!({}));
    Ok(())
}

// --- finding 12 ---------------------------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_measurement_of_2025_is_accepted(pool: PgPool) -> sqlx::Result<()> {
    let (app, ws, _) = with_batch(&pool).await;
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO measurements (id, workspace_id, batch_id, kind, value, measured_at, source)
             VALUES ('{}', '{ws}', '{}', 'sg', 1.048, '2025-06-01 10:00:00+00', 'paper')",
            id(70),
            id(20)
        ),
    )
    .await?;
    let part: String = sqlx::query_scalar(
        "SELECT tableoid::regclass::text FROM measurements WHERE measured_at < '2026-01-01+00'",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(part, "measurements_legacy");
    Ok(())
}

// --- finding 13 ---------------------------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_second_user_takes_over_the_push_token_of_a_shared_phone(
    pool: PgPool,
) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let (a, b) = (id(1), id(2));
    create_user(&app, a).await?;
    create_user(&app, b).await?;
    let register = |user: Uuid, n: u128| {
        let app = app.clone();
        async move {
            let mut tx = app.begin().await?;
            set_context(&mut tx, None, Some(user)).await?;
            let target: Uuid = sqlx::query_scalar(
                "SELECT register_push_target($1, 'production', 'apns-token-1', '1.0')",
            )
            .bind(id(n))
            .fetch_one(&mut *tx)
            .await?;
            tx.commit().await?;
            Ok::<Uuid, sqlx::Error>(target)
        }
    };
    register(a, 400).await?;
    // The same user registers again: no second row.
    register(a, 401).await?;
    let own: i64 = sqlx::query_scalar("SELECT count(*) FROM push_targets WHERE user_id = $1")
        .bind(a)
        .fetch_one(&pool)
        .await?;
    assert_eq!(own, 1);
    // User B signs in on the same phone: A's row goes, B's row exists.
    register(b, 402).await?;
    let owners: Vec<Uuid> = sqlx::query_scalar("SELECT user_id FROM push_targets")
        .fetch_all(&pool)
        .await?;
    assert_eq!(owners, vec![b]);

    // Without a user context the function refuses.
    let err = sqlx::query("SELECT register_push_target($1, 'production', 'x', NULL)")
        .bind(id(403))
        .execute(&app)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));
    Ok(())
}
