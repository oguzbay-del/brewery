//! T2.9: fixes for the F2.1 data model review (`gelistirme/raporlar/F2.1-veri-modeli.md`).
//! Every test names the finding it closes; each one failed before the matching migration
//! (`0080`–`0085`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::time::Duration;

use common::{
    Role, create_recipe, create_user, create_workspace, id, pool_as, run, set_context, sqlstate,
};
use sqlx::{PgPool, types::Uuid};

const INTEGRITY: &str = "23000";
const CHECK_VIOLATION: &str = "23514";
const FOREIGN_KEY_VIOLATION: &str = "23503";
const PERMISSION_DENIED: &str = "42501";
const UNIQUE_VIOLATION: &str = "23505";
const NOT_SOFT_DELETED: &str = "DM001";
const UNDER_RETENTION: &str = "DM002";

/// Business workspace `ws` (id 1), user (id 2), approved recipe version (id 11), batch (id 20).
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
            "INSERT INTO batches (id, workspace_id, code, recipe_version_id)
             VALUES ('{}', '{ws}', 'B-001', '{}')",
            id(20),
            id(11)
        ),
    )
    .await
    .unwrap();
    (app, ws, user)
}

/// A shared library ingredient (id 100) and a stock lot on it (id 110).
async fn with_lot(pool: &PgPool, app: &PgPool, ws: Uuid) -> (Uuid, Uuid) {
    sqlx::query(
        "INSERT INTO ingredients (id, workspace_id, kind, name, source)
         VALUES ($1, NULL, 'fermentable', 'Pale Malt', 'library')",
    )
    .bind(id(100))
    .execute(pool)
    .await
    .unwrap();
    run(
        app,
        ws,
        None,
        &format!(
            "INSERT INTO stock_lots (id, workspace_id, ingredient_id)
             VALUES ('{}', '{ws}', '{}')",
            id(110),
            id(100)
        ),
    )
    .await
    .unwrap();
    (id(100), id(110))
}

async fn lot_quantity(pool: &PgPool) -> f64 {
    sqlx::query_scalar("SELECT quantity FROM stock_lots")
        .fetch_one(pool)
        .await
        .unwrap()
}

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

async fn count(pool: &PgPool, sql: &str) -> i64 {
    sqlx::query_scalar(sqlx::AssertSqlSafe(sql.to_owned()))
        .fetch_one(pool)
        .await
        .unwrap()
}

// --- 1: publish versus line insert ----------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn finding_1_a_line_insert_waits_for_a_concurrent_publish(pool: PgPool) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let (ws, user) = (id(1), id(2));
    create_workspace(&app, ws, "A").await?;
    create_user(&app, user).await?;
    create_recipe(&app, ws, 10, "draft", None).await?;
    let version = id(11);

    // Device A publishes (draft -> approved) and has not committed yet.
    let mut publish = app.begin().await?;
    set_context(&mut publish, Some(ws), None).await?;
    sqlx::query(
        "UPDATE recipe_versions SET status = 'approved', approved_by = $1, approved_at = now()",
    )
    .bind(user)
    .execute(&mut *publish)
    .await?;

    // Device B's autosave adds a line at the same time.
    let app_b = app.clone();
    let line = tokio::spawn(async move {
        run(
            &app_b,
            ws,
            None,
            &format!(
                "INSERT INTO recipe_hops (id, workspace_id, recipe_version_id, position, name, amount_kg, alpha_pct)
                 VALUES ('{}', '{ws}', '{version}', 0, 'Cascade', 0.03, 6.5)",
                id(20)
            ),
        )
        .await
    });
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(
        !line.is_finished(),
        "the line insert must wait for the publish"
    );
    publish.commit().await?;

    let err = line.await.unwrap().unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY));
    assert_eq!(count(&pool, "SELECT count(*) FROM recipe_hops").await, 0);
    Ok(())
}

// --- 2, 3: stock ledger ------------------------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn finding_2_a_lot_cannot_be_created_with_a_quantity(pool: PgPool) -> sqlx::Result<()> {
    let (app, ws, _) = with_batch(&pool).await;
    let (ingredient, _) = with_lot(&pool, &app, ws).await;
    let err = run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO stock_lots (id, workspace_id, ingredient_id, lot_code, quantity)
             VALUES ('{}', '{ws}', '{ingredient}', 'L2', 1000)",
            id(111)
        ),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY));
    // The opening quantity comes in as a `receive` movement.
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO stock_movements (id, workspace_id, lot_id, kind, quantity)
             VALUES ('{}', '{ws}', '{}', 'receive', 1000)",
            id(120),
            id(110)
        ),
    )
    .await?;
    assert!((lot_quantity(&pool).await - 1000.0).abs() < 1e-9);
    Ok(())
}

fn count_movement(ws: Uuid, n: u128, counted: f64) -> String {
    format!(
        "INSERT INTO stock_movements (id, workspace_id, lot_id, kind, counted_quantity)
         VALUES ('{}', '{ws}', '{}', 'count', {counted})",
        id(n),
        id(110)
    )
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn finding_3_a_count_carries_the_counted_value_not_a_stale_difference(
    pool: PgPool,
) -> sqlx::Result<()> {
    let (app, ws, _) = with_batch(&pool).await;
    with_lot(&pool, &app, ws).await;
    let movement = |n: u128, kind: &str, qty: f64| {
        format!(
            "INSERT INTO stock_movements (id, workspace_id, lot_id, kind, quantity)
             VALUES ('{}', '{ws}', '{}', '{kind}', {qty})",
            id(n),
            id(110)
        )
    };
    run(&app, ws, None, &movement(120, "receive", 15.0)).await?;

    // The storekeeper counts 12.0 kg; a brew consumes 2.0 kg before the count is written:
    // the result is the counted value, not 15 - 3 - 2.
    let mut consume = app.begin().await?;
    set_context(&mut consume, Some(ws), None).await?;
    sqlx::query(sqlx::AssertSqlSafe(movement(121, "consume", -2.0)))
        .execute(&mut *consume)
        .await?;
    let app_b = app.clone();
    let count =
        tokio::spawn(async move { run(&app_b, ws, None, &count_movement(ws, 122, 12.0)).await });
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(!count.is_finished(), "the count waits for the lot lock");
    consume.commit().await?;
    count.await.unwrap()?;
    assert!((lot_quantity(&pool).await - 12.0).abs() < 1e-9);

    // The stored difference is derived: counted 12 against 13 on the lot = -1.
    let diff: f64 = sqlx::query_scalar("SELECT quantity FROM stock_movements WHERE kind = 'count'")
        .fetch_one(&pool)
        .await?;
    assert!((diff - -1.0).abs() < 1e-9);

    // A count needs `counted_quantity`; others may not carry it.
    for sql in [
        format!(
            "INSERT INTO stock_movements (id, workspace_id, lot_id, kind, quantity)
             VALUES ('{}', '{ws}', '{}', 'count', 1)",
            id(123),
            id(110)
        ),
        format!(
            "INSERT INTO stock_movements (id, workspace_id, lot_id, kind, quantity, counted_quantity)
             VALUES ('{}', '{ws}', '{}', 'adjust', 1, 5)",
            id(124),
            id(110)
        ),
    ] {
        let err = run(&app, ws, None, &sql).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(CHECK_VIOLATION), "{sql}");
    }
    Ok(())
}

// --- 5, 6, 8, 40: permanent workspace deletion --------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn findings_5_6_8_only_the_worker_purges_and_leaves_a_tombstone(
    pool: PgPool,
) -> sqlx::Result<()> {
    let (app, ws, user) = with_batch(&pool).await;
    let worker = pool_as(&pool, Role::Worker).await;
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO audit_log (id, workspace_id, seq, actor_kind, action, record_kind, prev_hash, hash)
             VALUES ('{}', '{ws}', 1, 'system', 'workspace.create', 'workspace', decode(repeat('00', 32), 'hex'),
                     decode(repeat('ab', 32), 'hex'));
             INSERT INTO audit_heads (workspace_id, last_seq, last_hash)
             VALUES ('{ws}', 1, decode(repeat('ab', 32), 'hex'))",
            id(300),
        ),
    )
    .await?;
    sqlx::query(
        "INSERT INTO subscriptions (id, workspace_id, source, external_id, product_key, status)
         VALUES ($1, $2, 'app_store', 'orig-tx-1', 'business_monthly', 'active')",
    )
    .bind(id(301))
    .bind(ws)
    .execute(&pool)
    .await?;
    sqlx::query(
        "INSERT INTO scheduled_jobs (id, workspace_id, kind, run_at) VALUES ($1, $2, 'x', now())",
    )
    .bind(id(302))
    .bind(ws)
    .execute(&pool)
    .await?;

    // The API cannot delete a workspace or a user, nor call the purge.
    for sql in [
        format!("DELETE FROM workspaces WHERE id = '{ws}'"),
        format!("DELETE FROM users WHERE id = '{user}'"),
        format!("SELECT purge_workspace('{ws}')"),
        "SELECT * FROM list_active_workspaces()".to_owned(),
    ] {
        let err = run(&app, ws, Some(user), &sql).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED), "{sql}");
    }

    // The worker sees active workspaces without any context, but cannot purge an active one.
    let active: Vec<(Uuid,)> = sqlx::query_as("SELECT workspace_id FROM list_active_workspaces()")
        .fetch_all(&worker)
        .await?;
    assert_eq!(active, vec![(ws,)]);
    let err = sqlx::query("SELECT purge_workspace($1)")
        .bind(ws)
        .execute(&worker)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(NOT_SOFT_DELETED));

    soft_delete_workspace(&app, ws).await;
    let pending: Vec<(Uuid, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
        "SELECT workspace_id, blocked_until FROM list_workspaces_pending_purge(now() + interval '1 day')",
    )
    .fetch_all(&worker)
    .await?;
    assert_eq!(pending, vec![(ws, None)]);
    assert!(
        sqlx::query_as::<_, (Uuid,)>(
            "SELECT workspace_id FROM list_workspaces_pending_purge(now() - interval '1 day')"
        )
        .fetch_all(&worker)
        .await?
        .is_empty()
    );

    let purged: bool = sqlx::query_scalar("SELECT purge_workspace($1)")
        .bind(ws)
        .fetch_one(&worker)
        .await?;
    assert!(purged);
    // Idempotent: a second call finds nothing.
    let again: bool = sqlx::query_scalar("SELECT purge_workspace($1)")
        .bind(ws)
        .fetch_one(&worker)
        .await?;
    assert!(!again);

    // Everything is gone, the tombstone and the object purge job remain.
    for table in [
        "workspaces",
        "batches",
        "audit_log",
        "audit_heads",
        "subscriptions",
    ] {
        assert_eq!(
            count(&pool, &format!("SELECT count(*) FROM {table}")).await,
            0,
            "{table}"
        );
    }
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM scheduled_jobs WHERE kind = 'x'"
        )
        .await,
        0
    );
    let job: (String, serde_json::Value) = sqlx::query_as(
        "SELECT kind, payload FROM scheduled_jobs WHERE kind = 'purge_workspace_objects'",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(job.1["prefix"], format!("{ws}/"));

    let (seq, hash, refs, mode): (i64, Vec<u8>, serde_json::Value, String) = sqlx::query_as(
        "SELECT last_audit_seq, last_audit_hash, external_refs, mode
         FROM workspace_tombstones WHERE deleted_workspace_id = $1",
    )
    .bind(ws)
    .fetch_one(&worker)
    .await?;
    assert_eq!((seq, hash, mode.as_str()), (1, vec![0xab; 32], "business"));
    assert_eq!(refs[0]["external_id"], "orig-tx-1");

    // The tombstone is append-only and the API cannot touch it.
    for sql in [
        "UPDATE workspace_tombstones SET mode = 'home'",
        "DELETE FROM workspace_tombstones",
    ] {
        let err = sqlx::query(sql).execute(&pool).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY), "{sql}");
    }
    let err = sqlx::query("SELECT count(*) FROM workspace_tombstones")
        .execute(&app)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));
    Ok(())
}

// --- 11, 34: mandatory retention ----------------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn finding_11_retention_and_legal_hold_block_deletion(pool: PgPool) -> sqlx::Result<()> {
    let (app, ws, _user) = with_batch(&pool).await;
    let worker = pool_as(&pool, Role::Worker).await;
    let lab = |n: u128| {
        format!(
            "INSERT INTO lab_results (id, workspace_id, batch_id, parameter, value, source, retention_until)
             VALUES ('{}', '{ws}', '{}', 'ph', 4.3, 'lab', now() + interval '3 years')",
            id(n),
            id(20)
        )
    };
    run(&app, ws, None, &lab(130)).await?;
    // Default retention comes from the configuration (unverified until Aşama 6).
    let years: (i32, String) = sqlx::query_as(
        "SELECT retention_years, verification_status FROM retention_rules
         WHERE region = 'us' AND record_kind = 'lab_results'",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(years, (3, "unverified".to_owned()));

    // Retention cannot be shortened or removed by the application; extending is fine.
    for sql in [
        "UPDATE lab_results SET retention_until = now() + interval '1 day'",
        "UPDATE lab_results SET retention_until = NULL",
    ] {
        let err = run(&app, ws, None, sql).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY), "{sql}");
    }
    run(
        &app,
        ws,
        None,
        "UPDATE lab_results SET retention_until = now() + interval '5 years'",
    )
    .await?;

    // The batch that owns the retained result cannot be deleted either; the workspace purge
    // reports the blocker instead of failing halfway.
    let err = run(
        &worker,
        ws,
        None,
        &format!("DELETE FROM batches WHERE id = '{}'", id(20)),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY));
    soft_delete_workspace(&app, ws).await;
    let err = sqlx::query("SELECT purge_workspace($1)")
        .bind(ws)
        .execute(&worker)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(UNDER_RETENTION));
    let blocked: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        "SELECT blocked_until FROM list_workspaces_pending_purge(now() + interval '1 day')",
    )
    .fetch_one(&worker)
    .await?;
    assert!(blocked.is_some_and(|t| t > chrono::Utc::now() + chrono::Duration::days(365 * 4)));

    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn finding_11_a_legal_hold_is_lifted_only_through_the_function(
    pool: PgPool,
) -> sqlx::Result<()> {
    let (app, ws, user) = with_batch(&pool).await;
    let worker = pool_as(&pool, Role::Worker).await;
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO transfers (id, workspace_id, kind, source_vessel_id, volume_l, legal_hold)
             VALUES ('{}', '{ws}', 'disposal', NULL, 10, true)",
            id(140)
        ),
    )
    .await
    .unwrap_err(); // disposal needs a source vessel: keeps the test honest about the schema
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO vessels (id, workspace_id, code, kind, capacity_l)
             VALUES ('{}', '{ws}', 'FV-1', 'fermenter', 1000);
             INSERT INTO cleaning_records (id, workspace_id, vessel_id, legal_hold)
             VALUES ('{}', '{ws}', '{}', true)",
            id(150),
            id(151),
            id(150)
        ),
    )
    .await?;
    soft_delete_workspace(&app, ws).await;
    let infinite: bool = sqlx::query_scalar(
        "SELECT blocked_until = 'infinity' FROM list_workspaces_pending_purge(now() + interval '1 day')",
    )
    .fetch_one(&worker)
    .await?;
    assert!(infinite);

    // Direct edits, soft delete and delete are refused while the hold stands.
    for sql in [
        "UPDATE cleaning_records SET legal_hold = false",
        "UPDATE cleaning_records SET deleted_at = now()",
        "DELETE FROM cleaning_records",
    ] {
        let err = run(&app, ws, Some(user), sql).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY), "{sql}");
    }
    // The function needs a user context; the worker cannot call it.
    let err = run(
        &worker,
        ws,
        Some(user),
        &format!(
            "SELECT release_legal_hold('cleaning_records', '{}')",
            id(151)
        ),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));
    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(ws), Some(user)).await?;
    sqlx::query("SELECT set_config('app.actor_kind', 'user', true)")
        .execute(&mut *tx)
        .await?;
    let lifted: bool = sqlx::query_scalar("SELECT release_legal_hold('cleaning_records', $1)")
        .bind(id(151))
        .fetch_one(&mut *tx)
        .await?;
    assert!(lifted);
    tx.commit().await?;

    let purged: bool = sqlx::query_scalar("SELECT purge_workspace($1)")
        .bind(ws)
        .fetch_one(&worker)
        .await?;
    assert!(purged);
    Ok(())
}

// --- 7: permanent delete of referenced records ----------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn finding_7_deleting_a_referenced_record_leaves_a_snapshot_behind(
    pool: PgPool,
) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let worker = pool_as(&pool, Role::Worker).await;
    let (ws, user) = (id(1), id(2));
    create_workspace(&app, ws, "A").await?;
    create_user(&app, user).await?;
    // Equipment, a private ingredient, a published recipe version using both.
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO equipment (id, workspace_id, name, batch_volume_l, efficiency_pct)
             VALUES ('{}', '{ws}', 'Brew Kettle 40', 40, 72);
             INSERT INTO ingredients (id, workspace_id, kind, name, source)
             VALUES ('{}', '{ws}', 'hop', 'My Hop', 'user');
             INSERT INTO recipes (id, workspace_id, name) VALUES ('{}', '{ws}', 'Pale Ale');
             INSERT INTO recipe_versions (id, workspace_id, recipe_id, number, status, batch_volume_l,
                                          efficiency_pct, equipment_id)
             VALUES ('{}', '{ws}', '{}', 1, 'draft', 20, 72, '{}');
             INSERT INTO recipe_hops (id, workspace_id, recipe_version_id, position, ingredient_id, name,
                                      amount_kg, alpha_pct)
             VALUES ('{}', '{ws}', '{}', 0, '{}', 'My Hop', 0.03, 6.5);
             UPDATE recipe_versions SET status = 'approved', approved_by = '{user}', approved_at = now();",
            id(10),
            id(11),
            id(12),
            id(13),
            id(12),
            id(10),
            id(14),
            id(13),
            id(11)
        ),
    )
    .await?;
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO batches (id, workspace_id, code, recipe_version_id, equipment_id)
             VALUES ('{}', '{ws}', 'B-001', '{}', '{}');
             INSERT INTO devices (id, workspace_id, kind, name, ingest_key_hash, batch_id)
             VALUES ('{}', '{ws}', 'tilt', 'Red Tilt', decode(repeat('01', 32), 'hex'), '{}');
             INSERT INTO measurements (id, workspace_id, batch_id, kind, value, measured_at, source, device_id,
                                       device_name)
             VALUES ('{}', '{ws}', '{}', 'sg', 1.05, '2026-11-14 10:00:00+00', 'device', '{}', 'Red Tilt');",
            id(20),
            id(13),
            id(10),
            id(30),
            id(20),
            id(31),
            id(20),
            id(30)
        ),
    )
    .await?;
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM batches WHERE equipment_name = 'Brew Kettle 40'"
        )
        .await,
        1,
        "the batch copied the equipment name"
    );

    // Permanent delete (the worker's job) of equipment, ingredient and device succeeds.
    run(
        &worker,
        ws,
        None,
        &format!("DELETE FROM equipment WHERE id = '{}'", id(10)),
    )
    .await?;
    run(
        &worker,
        ws,
        None,
        &format!("DELETE FROM ingredients WHERE id = '{}'", id(11)),
    )
    .await?;
    run(
        &worker,
        ws,
        None,
        &format!("DELETE FROM devices WHERE id = '{}'", id(30)),
    )
    .await?;
    let version: (Option<Uuid>, serde_json::Value, String) =
        sqlx::query_as("SELECT equipment_id, equipment_snapshot, status FROM recipe_versions")
            .fetch_one(&pool)
            .await?;
    assert_eq!((version.0, version.2.as_str()), (None, "approved"));
    let hop: (Option<Uuid>, String) = sqlx::query_as("SELECT ingredient_id, name FROM recipe_hops")
        .fetch_one(&pool)
        .await?;
    assert_eq!(hop, (None, "My Hop".to_owned()));
    let batch: (Option<Uuid>, Option<String>) =
        sqlx::query_as("SELECT equipment_id, equipment_name FROM batches")
            .fetch_one(&pool)
            .await?;
    assert_eq!(batch, (None, Some("Brew Kettle 40".to_owned())));
    let measurement: (Option<Uuid>, Option<String>, String) =
        sqlx::query_as("SELECT device_id, device_name, source FROM measurements")
            .fetch_one(&pool)
            .await?;
    assert_eq!(
        measurement,
        (None, Some("Red Tilt".to_owned()), "device".to_owned())
    );

    // The published version is still immutable for everything else.
    let err = run(
        &app,
        ws,
        None,
        "UPDATE recipe_versions SET batch_volume_l = 30",
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY));

    // Stock: a movement keeps the batch code after the batch is gone; the batch's own
    // dependants (measurements) go with it.
    let (ingredient, lot) = with_lot(&pool, &app, ws).await;
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO stock_movements (id, workspace_id, lot_id, kind, quantity, batch_id)
             VALUES ('{}', '{ws}', '{lot}', 'receive', 5, '{}');
             INSERT INTO batch_ingredient_actuals (id, workspace_id, batch_id, actual_amount_kg, lot_id,
                                                   substitute_ingredient_id)
             VALUES ('{}', '{ws}', '{}', 1, '{lot}', '{ingredient}');
             INSERT INTO scanned_documents (id, workspace_id, batch_id, form_type, object_key, content_type,
                                            size_bytes, sha256)
             VALUES ('{}', '{ws}', '{}', 'brew_log', 'k', 'image/png', 1, decode(repeat('02', 32), 'hex'))",
            id(40),
            id(20),
            id(41),
            id(20),
            id(42),
            id(20)
        ),
    )
    .await?;
    run(
        &worker,
        ws,
        None,
        &format!("DELETE FROM batches WHERE id = '{}'", id(20)),
    )
    .await?;
    let movement: (Option<Uuid>, Option<String>) =
        sqlx::query_as("SELECT batch_id, batch_code FROM stock_movements")
            .fetch_one(&pool)
            .await?;
    assert_eq!(movement, (None, Some("B-001".to_owned())));
    let doc: (Option<Uuid>, Option<String>) =
        sqlx::query_as("SELECT batch_id, batch_code FROM scanned_documents")
            .fetch_one(&pool)
            .await?;
    assert_eq!(doc, (None, Some("B-001".to_owned())));
    assert_eq!(count(&pool, "SELECT count(*) FROM measurements").await, 0);
    assert!((lot_quantity(&pool).await - 5.0).abs() < 1e-9);

    // The lot's actual was deleted with its batch; deleting a lot with a surviving reference
    // blanks the reference but keeps the code.
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO batches (id, workspace_id, code, recipe_version_id) VALUES ('{}', '{ws}', 'B-002', '{}');
             INSERT INTO batch_ingredient_actuals (id, workspace_id, batch_id, actual_amount_kg, lot_id,
                                                   substitute_ingredient_id)
             VALUES ('{}', '{ws}', '{}', 1, '{lot}', '{ingredient}')",
            id(50),
            id(13),
            id(51),
            id(50)
        ),
    )
    .await?;
    run(
        &worker,
        ws,
        None,
        &format!("DELETE FROM stock_lots WHERE id = '{lot}'"),
    )
    .await?;
    let actual: (Option<Uuid>, Option<String>) =
        sqlx::query_as("SELECT lot_id, lot_code FROM batch_ingredient_actuals")
            .fetch_one(&pool)
            .await?;
    assert_eq!(actual, (None, Some("default".to_owned())));
    // Still restricted: an ingredient that stock lots depend on is not deleted from under them.
    let (_, lot2) = (ingredient, id(112));
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO ingredients (id, workspace_id, kind, name, source) VALUES ('{}', '{ws}', 'misc', 'Salt', 'user');
             INSERT INTO stock_lots (id, workspace_id, ingredient_id) VALUES ('{lot2}', '{ws}', '{}')",
            id(113),
            id(113)
        ),
    )
    .await?;
    let err = run(
        &worker,
        ws,
        None,
        &format!("DELETE FROM ingredients WHERE id = '{}'", id(113)),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(FOREIGN_KEY_VIOLATION));
    Ok(())
}

// --- 9: audit rows for personal fields ---------------------------------------------------------------
// (Rust side: `db::changed_marker`; see tests/audit_chain.rs.)

// --- 10, 13, 41: account and webhook tables -----------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn finding_10_webhook_payloads_are_not_readable_by_the_api(pool: PgPool) -> sqlx::Result<()> {
    let api = pool_as(&pool, Role::Api).await;
    let worker = pool_as(&pool, Role::Worker).await;
    sqlx::query("INSERT INTO external_events (id, source, delivery_id, payload) VALUES ($1, 'github', 'd-1', '{\"comment\": \"my beer is flat\"}')")
        .bind(id(1))
        .execute(&api)
        .await?;
    let err = sqlx::query("SELECT payload FROM external_events")
        .execute(&api)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));
    let seen = |source: &'static str, delivery: &'static str| {
        let api = api.clone();
        async move {
            sqlx::query_scalar::<_, bool>("SELECT external_event_seen($1, $2)")
                .bind(source)
                .bind(delivery)
                .fetch_one(&api)
                .await
                .unwrap()
        }
    };
    assert!(seen("github", "d-1").await);
    assert!(!seen("github", "d-9").await);

    // The worker blanks the payload of processed events after a week and deletes them later.
    sqlx::query("UPDATE external_events SET processed_at = now() - interval '8 days'")
        .execute(&worker)
        .await?;
    let n: i64 = sqlx::query_scalar("SELECT purge_external_events()")
        .fetch_one(&worker)
        .await?;
    assert_eq!(n, 1);
    let payload: serde_json::Value = sqlx::query_scalar("SELECT payload FROM external_events")
        .fetch_one(&worker)
        .await?;
    assert_eq!(payload, serde_json::json!({"purged": true}));
    sqlx::query("UPDATE external_events SET processed_at = now() - interval '100 days'")
        .execute(&worker)
        .await?;
    let n: i64 = sqlx::query_scalar("SELECT purge_external_events()")
        .fetch_one(&worker)
        .await?;
    assert_eq!(n, 1);
    assert_eq!(
        count(&pool, "SELECT count(*) FROM external_events").await,
        0
    );
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn finding_13_a_second_user_takes_over_the_device_token(pool: PgPool) -> sqlx::Result<()> {
    let api = pool_as(&pool, Role::Api).await;
    let (a, b) = (id(2), id(3));
    create_user(&api, a).await?;
    create_user(&api, b).await?;
    assert_eq!(claim(&api, a, 10).await?, id(10));
    // Same user again: the existing row is kept.
    assert_eq!(claim(&api, a, 11).await?, id(10));
    // B logs in on the same phone: A's row goes, B's is created.
    assert_eq!(claim(&api, b, 12).await?, id(12));
    let owners: Vec<(Uuid,)> = sqlx::query_as("SELECT user_id FROM push_targets")
        .fetch_all(&pool)
        .await?;
    assert_eq!(owners, vec![(b,)]);
    // A user can have the same token only once, but two users can (different rows) when
    // inserted directly under their own context.
    let mut tx = api.begin().await?;
    set_context(&mut tx, None, Some(a)).await?;
    sqlx::query(
        "INSERT INTO push_targets (id, user_id, environment, token) VALUES ($1, $2, 'production', 'apns-token-1')",
    )
    .bind(id(13))
    .bind(a)
    .execute(&mut *tx)
    .await?;
    let dup = sqlx::query(
        "INSERT INTO push_targets (id, user_id, environment, token) VALUES ($1, $2, 'production', 'apns-token-1')",
    )
    .bind(id(14))
    .bind(a)
    .execute(&mut *tx)
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&dup).as_deref(), Some(UNIQUE_VIOLATION));
    // Without a user context the function refuses.
    let mut tx = api.begin().await?;
    let err = sqlx::query("SELECT claim_push_target($1, 'production', 'x')")
        .bind(id(15))
        .execute(&mut *tx)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn finding_41_account_events_belong_to_the_user_and_are_append_only(
    pool: PgPool,
) -> sqlx::Result<()> {
    let api = pool_as(&pool, Role::Api).await;
    let (a, b) = (id(2), id(3));
    create_user(&api, a).await?;
    create_user(&api, b).await?;
    for (user, n) in [(a, 10_u128), (b, 11)] {
        let mut tx = api.begin().await?;
        set_context(&mut tx, None, Some(user)).await?;
        sqlx::query(
            "INSERT INTO account_events (id, user_id, kind, actor_kind) VALUES ($1, $2, 'passkey.added', 'user')",
        )
        .bind(id(n))
        .bind(user)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
    }
    let mut tx = api.begin().await?;
    set_context(&mut tx, None, Some(a)).await?;
    let seen: Vec<(Uuid,)> = sqlx::query_as("SELECT user_id FROM account_events")
        .fetch_all(&mut *tx)
        .await?;
    assert_eq!(seen, vec![(a,)]);
    // Writing for someone else is refused by RLS; the log cannot be edited.
    let err = sqlx::query(
        "INSERT INTO account_events (id, user_id, kind, actor_kind) VALUES ($1, $2, 'login', 'user')",
    )
    .bind(id(12))
    .bind(b)
    .execute(&mut *tx)
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some("42501"));
    tx.rollback().await?;
    for sql in [
        "UPDATE account_events SET kind = 'x'",
        "DELETE FROM account_events",
    ] {
        let err = sqlx::query(sql).execute(&api).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED), "{sql}");
        let err = sqlx::query(sql).execute(&pool).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY), "{sql}");
    }
    Ok(())
}

// --- 12, 31, 22: measurements ----------------------------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn findings_12_31_22_measurement_targets_and_range(pool: PgPool) -> sqlx::Result<()> {
    let (app, ws, _) = with_batch(&pool).await;
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO vessels (id, workspace_id, code, kind, capacity_l)
             VALUES ('{}', '{ws}', 'FV-1', 'fermenter', 1000);
             INSERT INTO devices (id, workspace_id, kind, name, ingest_key_hash)
             VALUES ('{}', '{ws}', 'tilt', 'Tilt', decode(repeat('01', 32), 'hex'));",
            id(50),
            id(60)
        ),
    )
    .await?;
    let measurement = |n: u128,
                       at: &str,
                       batch: Option<Uuid>,
                       vessel: Option<Uuid>,
                       extra: &str| {
        let q = |v: Option<Uuid>| v.map_or("NULL".to_owned(), |v| format!("'{v}'"));
        format!(
            "INSERT INTO measurements (id, workspace_id, batch_id, vessel_id, kind, value, measured_at, source,
                                      device_id {})
             VALUES ('{}', '{ws}', {}, {}, 'temp_c', 18.5, '{at}', 'device', '{}' {})",
            if extra.is_empty() { "" } else { ", package_id" },
            id(n),
            q(batch),
            q(vessel),
            id(60),
            extra
        )
    };
    // 12: a paper record from 2025 lands in the legacy partition; a clock stuck in 1970 does not.
    run(
        &app,
        ws,
        None,
        &measurement(70, "2025-06-01 12:00:00+00", Some(id(20)), None, ""),
    )
    .await?;
    let part: String =
        sqlx::query_scalar("SELECT tableoid::regclass::text FROM measurements WHERE id = $1")
            .bind(id(70))
            .fetch_one(&pool)
            .await?;
    assert_eq!(part, "measurements_legacy");
    let err = run(
        &app,
        ws,
        None,
        &measurement(71, "1970-01-01 00:00:00+00", Some(id(20)), None, ""),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(CHECK_VIOLATION));

    // 31: a device that is on neither a batch nor a vessel is refused; a tank is enough.
    let err = run(
        &app,
        ws,
        None,
        &measurement(72, "2026-11-14 10:00:00+00", None, None, ""),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(CHECK_VIOLATION));
    run(
        &app,
        ws,
        None,
        &measurement(73, "2026-11-14 10:00:00+00", None, Some(id(50)), ""),
    )
    .await?;

    // 22: the package must belong to the measurement's own batch.
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO batches (id, workspace_id, code, recipe_version_id) VALUES ('{}', '{ws}', 'B-002', '{}');
             INSERT INTO brew_day_packages (id, workspace_id, batch_id, content_version, content, signature, key_id)
             VALUES ('{}', '{ws}', '{}', 1, '{{\"schema_version\": 1}}', decode('00', 'hex'), 'k1')",
            id(21),
            id(11),
            id(80),
            id(21)
        ),
    )
    .await?;
    let err = run(
        &app,
        ws,
        None,
        &measurement(
            74,
            "2026-11-14 10:00:00+00",
            Some(id(20)),
            None,
            &format!(", '{}'", id(80)),
        ),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(FOREIGN_KEY_VIOLATION));
    run(
        &app,
        ws,
        None,
        &measurement(
            75,
            "2026-11-14 10:00:00+00",
            Some(id(21)),
            None,
            &format!(", '{}'", id(80)),
        ),
    )
    .await?;
    Ok(())
}

// --- small findings ------------------------------------------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn small_findings_16_17_19_21_23_26_32_39(pool: PgPool) -> sqlx::Result<()> {
    let (app, ws, user) = with_batch(&pool).await;
    let worker = pool_as(&pool, Role::Worker).await;

    // 16: an internal support note is invisible to the customer context.
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO support_requests (id, workspace_id, topic, title) VALUES ('{}', '{ws}', 'billing', 'Help');
             INSERT INTO support_messages (id, workspace_id, request_id, author_kind, body, visibility)
             VALUES ('{}', '{ws}', '{}', 'system', 'public', 'customer'),
                    ('{}', '{ws}', '{}', 'system', 'internal note', 'internal')",
            id(200),
            id(201),
            id(200),
            id(202),
            id(200)
        ),
    )
    .await?;
    let visible = |actor: &'static str| {
        let app = app.clone();
        async move {
            let mut tx = app.begin().await.unwrap();
            set_context(&mut tx, Some(ws), None).await.unwrap();
            sqlx::query("SELECT set_config('app.actor_kind', $1, true)")
                .bind(actor)
                .execute(&mut *tx)
                .await
                .unwrap();
            let n: i64 = sqlx::query_scalar("SELECT count(*) FROM support_messages")
                .fetch_one(&mut *tx)
                .await
                .unwrap();
            n
        }
    };
    assert_eq!(visible("user").await, 1);
    assert_eq!(visible("support").await, 2);

    // 17: account data exports are visible to the requester only.
    run(
        &app,
        ws,
        Some(user),
        &format!(
            "INSERT INTO export_jobs (id, workspace_id, scope, format, created_by)
             VALUES ('{}', '{ws}', 'account_data', 'zip', '{user}')",
            id(210)
        ),
    )
    .await?;
    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(ws), Some(id(9))).await?;
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM export_jobs")
        .fetch_one(&mut *tx)
        .await?;
    assert_eq!(n, 0);
    tx.rollback().await?;

    // 19: the API cannot queue an event for a workspace outside its context.
    let err = sqlx::query(
        "INSERT INTO outbox (id, topic, workspace_id, payload) VALUES ($1, 't', $2, '{\"schema_version\": 1}')",
    )
    .bind(id(220))
    .bind(ws)
    .execute(&app)
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));
    sqlx::query("INSERT INTO outbox (id, topic, workspace_id, payload) VALUES ($1, 't', $2, '{\"schema_version\": 1}')")
        .bind(id(221))
        .bind(ws)
        .execute(&worker)
        .await?;

    // 21: a recipe's approved version belongs to that recipe.
    create_recipe(&app, ws, 30, "approved", Some(user)).await?;
    let err = run(
        &app,
        ws,
        None,
        &format!(
            "UPDATE recipes SET approved_version_id = '{}' WHERE id = '{}'",
            id(11),
            id(30)
        ),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(FOREIGN_KEY_VIOLATION));
    run(
        &app,
        ws,
        None,
        &format!(
            "UPDATE recipes SET approved_version_id = '{}' WHERE id = '{}'",
            id(31),
            id(30)
        ),
    )
    .await?;

    // 23: an ingredient's kind is fixed.
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO ingredients (id, workspace_id, kind, name, source) VALUES ('{}', '{ws}', 'hop', 'H', 'user')",
            id(240)
        ),
    )
    .await?;
    let err = run(&app, ws, None, "UPDATE ingredients SET kind = 'misc'")
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY));

    // 26: one correction per lab result.
    let lab = |n: u128, corrects: Option<u128>| {
        format!(
            "INSERT INTO lab_results (id, workspace_id, batch_id, parameter, value, source, corrects_id)
             VALUES ('{}', '{ws}', '{}', 'ph', 4.3, 'lab', {})",
            id(n),
            id(20),
            corrects.map_or("NULL".to_owned(), |c| format!("'{}'", id(c)))
        )
    };
    run(&app, ws, None, &lab(250, None)).await?;
    run(&app, ws, None, &lab(251, Some(250))).await?;
    let err = run(&app, ws, None, &lab(252, Some(250))).await.unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(UNIQUE_VIOLATION));

    // 32: a brew day package keeps its content and signature.
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO brew_day_packages (id, workspace_id, batch_id, content_version, content, signature, key_id)
             VALUES ('{}', '{ws}', '{}', 1, '{{\"schema_version\": 1}}', decode('00', 'hex'), 'k1')",
            id(260),
            id(20)
        ),
    )
    .await?;
    for sql in [
        "UPDATE brew_day_packages SET content = '{\"schema_version\": 2}'",
        "UPDATE brew_day_packages SET signature = decode('ff', 'hex')",
    ] {
        let err = run(&app, ws, None, sql).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED), "{sql}");
    }
    run(
        &app,
        ws,
        None,
        "UPDATE brew_day_packages SET revoked_at = now()",
    )
    .await?;

    // 39: a deleted account keeps no personal data.
    let tombstone = |sql: &str| {
        let app = app.clone();
        let sql = sql.to_owned();
        async move {
            let mut tx = app.begin().await.unwrap();
            set_context(&mut tx, None, Some(id(2))).await.unwrap();
            sqlx::query(sqlx::AssertSqlSafe(sql))
                .execute(&mut *tx)
                .await
        }
    };
    let err = tombstone("UPDATE users SET deleted_at = now()")
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(CHECK_VIOLATION));
    tombstone("UPDATE users SET deleted_at = now(), display_name = '', contact_email = NULL")
        .await?;
    Ok(())
}

/// Finding 24: foreign key columns that deletes and "where is this ingredient used" scan.
#[sqlx::test(migrator = "db::MIGRATOR")]
async fn finding_24_listed_foreign_keys_are_indexed(pool: PgPool) -> sqlx::Result<()> {
    let columns = [
        ("calc_results", "recipe_version_id"),
        ("calc_results", "batch_id"),
        ("measurements", "package_id"),
        ("measurements", "vessel_id"),
        ("tasting_notes", "batch_id"),
        ("recipe_fermentables", "ingredient_id"),
        ("recipe_hops", "ingredient_id"),
        ("recipe_yeasts", "ingredient_id"),
        ("recipe_miscs", "ingredient_id"),
        ("devices", "batch_id"),
        ("lab_results", "spec_id"),
        ("lab_results", "corrects_id"),
        ("releases", "packaging_run_id"),
        ("yeast_generations", "source_batch_id"),
        ("role_assignments", "role_id"),
        ("invitations", "role_id"),
    ];
    for (table, column) in columns {
        let indexed: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                 SELECT 1 FROM pg_index i
                 JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attname = $2
                 WHERE i.indrelid = $1::regclass AND (i.indkey::int2[])[0:1] @> ARRAY[a.attnum])",
        )
        .bind(table)
        .bind(column)
        .fetch_one(&pool)
        .await?;
        assert!(indexed, "{table}.{column} has no index");
    }
    Ok(())
}

/// Finding 20: only the movement trigger may write the stock cache or delete versions from a
/// trigger; a new trigger function touching them must be added to this list on purpose (the
/// `pg_trigger_depth()` gates cannot tell a foreign key cascade from any other trigger).
#[sqlx::test(migrator = "db::MIGRATOR")]
async fn finding_20_trigger_functions_touching_guarded_tables_are_listed(
    pool: PgPool,
) -> sqlx::Result<()> {
    let found: Vec<(String,)> = sqlx::query_as(
        "SELECT p.proname::text FROM pg_proc p
         JOIN pg_namespace n ON n.oid = p.pronamespace AND n.nspname = 'public'
         WHERE p.prorettype = 'trigger'::regtype
           AND (p.prosrc ~* '(update|delete from|insert into)\\s+(stock_lots|recipe_versions|audit_log)\\y'
                OR p.prosrc ~* '(update|delete from|insert into)\\s+recipe_(fermentables|hops|yeasts|miscs|water_additions)\\y')
         ORDER BY 1",
    )
    .fetch_all(&pool)
    .await?;
    let names: Vec<&str> = found.iter().map(|(n,)| n.as_str()).collect();
    assert_eq!(
        names,
        ["stock_movements_apply"],
        "review the new trigger function against the depth gates"
    );
    Ok(())
}

async fn claim(api: &PgPool, user: Uuid, n: u128) -> sqlx::Result<Uuid> {
    let mut tx = api.begin().await?;
    set_context(&mut tx, None, Some(user)).await?;
    let row: Uuid =
        sqlx::query_scalar("SELECT claim_push_target($1, 'production', 'apns-token-1', '1.0')")
            .bind(id(n))
            .fetch_one(&mut *tx)
            .await?;
    tx.commit().await?;
    Ok(row)
}
