//! Batches, measurements (monthly partitions), stock ledger, scope integrity, legal hold,
//! write-restricted tables (T2.6).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{
    Role, create_recipe, create_role, create_user, create_workspace, id, pool_as, run, set_context,
    sqlstate,
};
use sqlx::{PgPool, types::Uuid};

const INTEGRITY: &str = "23000";
const CHECK_VIOLATION: &str = "23514";
const FOREIGN_KEY_VIOLATION: &str = "23503";
const PERMISSION_DENIED: &str = "42501";

/// Workspace `ws` (business) with user, an approved recipe version (id 11) and batch (id 20).
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

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_batch_needs_a_published_recipe_version(pool: PgPool) -> sqlx::Result<()> {
    let (app, ws, _) = with_batch(&pool).await;
    let batch = |n: u128, version: Uuid, code: &str| {
        format!(
            "INSERT INTO batches (id, workspace_id, code, recipe_version_id)
             VALUES ('{}', '{ws}', '{code}', '{version}')",
            id(n)
        )
    };
    // Draft versions are mutable: refused.
    create_recipe(&app, ws, 40, "draft", None).await?;
    let err = run(&app, ws, None, &batch(41, id(41), "B-002"))
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY));

    // Archived: business refuses, home accepts. (The archived version is reached the legal way.)
    run(
        &app,
        ws,
        None,
        &format!(
            "UPDATE recipe_versions SET status = 'archived', archived_at = now() WHERE id = '{}'",
            id(11)
        ),
    )
    .await?;
    let err = run(&app, ws, None, &batch(42, id(11), "B-003"))
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY));

    let home = id(5);
    let user = id(2);
    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(home), None).await?;
    sqlx::query(
        "INSERT INTO workspaces (id, name, mode, region, country, time_zone, support_pseudonym)
         VALUES ($1, 'Home', 'home', 'us', 'US', 'America/New_York', 'pseudonym-home-ws')",
    )
    .bind(home)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    create_recipe(&app, home, 50, "approved", Some(user)).await?;
    run(
        &app,
        home,
        None,
        &format!(
            "UPDATE recipe_versions SET status = 'archived', archived_at = now();
             INSERT INTO batches (id, workspace_id, code, recipe_version_id)
             VALUES ('{}', '{home}', 'H-1', '{}')",
            id(52),
            id(51)
        ),
    )
    .await?;

    // Code is unique per workspace.
    let err = run(&app, ws, None, &batch(43, id(11), "B-001"))
        .await
        .unwrap_err();
    assert!(sqlstate(&err).is_some());
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn measurements_land_in_monthly_partitions_with_rls(pool: PgPool) -> sqlx::Result<()> {
    let (app, a, _) = with_batch(&pool).await;
    let b = id(3);
    create_workspace(&app, b, "B").await?;
    let m = id(60);
    let insert = format!(
        "INSERT INTO measurements (id, workspace_id, batch_id, kind, value, measured_at, source)
         VALUES ('{m}', '{a}', '{}', 'sg', 1.052, '2026-11-14 10:00:00+00', 'manual')
         ON CONFLICT DO NOTHING",
        id(20)
    );
    run(&app, a, None, &insert).await?;
    // The client resends the same measurement (same UUID and time): no second row.
    run(&app, a, None, &insert).await?;
    let in_partition: i64 = sqlx::query_scalar("SELECT count(*) FROM measurements_y2026m11")
        .fetch_one(&pool)
        .await?;
    assert_eq!(in_partition, 1);
    // Same time in another month is a different partition entirely.
    run(
        &app,
        a,
        None,
        &format!(
            "INSERT INTO measurements (id, workspace_id, batch_id, kind, value, measured_at, source)
             VALUES ('{}', '{a}', '{}', 'temp_c', 18.5, '2026-12-01 00:00:00+00', 'manual')",
            id(61),
            id(20)
        ),
    )
    .await?;
    let december: i64 = sqlx::query_scalar("SELECT count(*) FROM measurements_y2026m12")
        .fetch_one(&pool)
        .await?;
    assert_eq!(december, 1);

    // B sees nothing through the parent and cannot insert into A's batch.
    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(b), None).await?;
    let seen: i64 = sqlx::query_scalar("SELECT count(*) FROM measurements")
        .fetch_one(&mut *tx)
        .await?;
    assert_eq!(seen, 0);
    tx.rollback().await?;
    let err = run(
        &app,
        b,
        None,
        &insert
            .replace(&a.to_string(), &b.to_string())
            .replace(&m.to_string(), &id(62).to_string()),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(FOREIGN_KEY_VIOLATION));
    // Writing A's workspace id from B's context is refused by the policy.
    let err = run(
        &app,
        b,
        None,
        &insert.replace(&m.to_string(), &id(63).to_string()),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));

    // The partition itself is protected too: a role that may read it still sees one workspace.
    let owner = pool_as(&pool, Role::Owner).await;
    let mut tx = owner.begin().await?;
    set_context(&mut tx, Some(b), None).await?;
    let direct: i64 = sqlx::query_scalar("SELECT count(*) FROM measurements_y2026m11")
        .fetch_one(&mut *tx)
        .await?;
    assert_eq!(direct, 0);
    tx.rollback().await?;

    // Append-only for the application; partitions are not directly accessible.
    for sql in [
        "UPDATE measurements SET value = 0",
        "DELETE FROM measurements",
        "SELECT 1 FROM measurements_y2026m11",
    ] {
        let err = run(&app, a, None, sql).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED), "{sql}");
    }
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn partitions_are_opened_ahead_by_the_worker_only(pool: PgPool) -> sqlx::Result<()> {
    let (app, a, _) = with_batch(&pool).await;
    let far = format!(
        "INSERT INTO measurements (id, workspace_id, batch_id, kind, value, measured_at, source)
         VALUES ('{}', '{a}', '{}', 'sg', 1.01, '2031-03-05 00:00:00+00', 'manual')",
        id(70),
        id(20)
    );
    // No partition yet: the insert fails (the worker keeps them open ahead).
    let err = run(&app, a, None, &far).await.unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(CHECK_VIOLATION));

    let err = sqlx::query("SELECT create_measurement_partition('2031-03-01')")
        .execute(&app)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));
    let err = sqlx::query("SELECT ensure_measurement_partitions()")
        .execute(&app)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));

    // Worker: idempotent, covers this month and the next two (already there from the migration).
    let worker = pool_as(&pool, Role::Worker).await;
    let created: i32 = sqlx::query_scalar("SELECT ensure_measurement_partitions(2)")
        .fetch_one(&worker)
        .await?;
    assert_eq!(created, 0);
    // A new month is created with its protection.
    let owner = pool_as(&pool, Role::Owner).await;
    let name: String = sqlx::query_scalar("SELECT create_measurement_partition('2031-03-17')")
        .fetch_one(&owner)
        .await?;
    assert_eq!(name, "measurements_y2031m03");
    let name2: String = sqlx::query_scalar("SELECT create_measurement_partition('2031-03-01')")
        .fetch_one(&owner)
        .await?;
    assert_eq!(name2, name);
    run(&app, a, None, &far).await
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn stock_quantity_follows_the_movement_ledger(pool: PgPool) -> sqlx::Result<()> {
    let (app, ws, _) = with_batch(&pool).await;
    let owner = pool_as(&pool, Role::Owner).await;
    let ingredient = id(100);
    sqlx::query(
        "INSERT INTO ingredients (id, workspace_id, kind, name, source)
         VALUES ($1, NULL, 'fermentable', 'Pale Malt', 'library')",
    )
    .bind(ingredient)
    .execute(&owner)
    .await?;
    let lot = id(110);
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO stock_lots (id, workspace_id, ingredient_id) VALUES ('{lot}', '{ws}', '{ingredient}')"
        ),
    )
    .await?;
    let movement = |n: u128, kind: &str, qty: f64| {
        format!(
            "INSERT INTO stock_movements (id, workspace_id, lot_id, kind, quantity, batch_id)
             VALUES ('{}', '{ws}', '{lot}', '{kind}', {qty}, '{}')",
            id(n),
            id(20)
        )
    };
    run(&app, ws, None, &movement(120, "receive", 25.0)).await?;
    run(&app, ws, None, &movement(121, "consume", -4.5)).await?;
    // Negative stock is allowed (the app warns).
    run(&app, ws, None, &movement(122, "consume", -30.0)).await?;
    let quantity: f64 = sqlx::query_scalar("SELECT quantity FROM stock_lots")
        .fetch_one(&pool)
        .await?;
    assert!((quantity - -9.5).abs() < 1e-9);

    // The cache cannot be edited by hand; the ledger cannot be edited at all.
    for (sql, code) in [
        ("UPDATE stock_lots SET quantity = 100".to_owned(), INTEGRITY),
        (
            "UPDATE stock_movements SET quantity = 1".to_owned(),
            PERMISSION_DENIED,
        ),
        ("DELETE FROM stock_movements".to_owned(), PERMISSION_DENIED),
        (movement(123, "receive", -1.0), CHECK_VIOLATION),
        (movement(124, "consume", 1.0), CHECK_VIOLATION),
    ] {
        let err = run(&app, ws, None, &sql).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(code), "{sql}");
    }
    // Other columns of the lot still change (and bump `version`).
    run(&app, ws, None, "UPDATE stock_lots SET supplier = 'Acme'").await?;
    let version: i64 = sqlx::query_scalar("SELECT version FROM stock_lots")
        .fetch_one(&pool)
        .await?;
    assert!(version >= 4);
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn scope_ids_must_point_at_a_row_of_their_kind_in_the_workspace(
    pool: PgPool,
) -> sqlx::Result<()> {
    let (app, a, _) = with_batch(&pool).await;
    let b = id(3);
    let (member_user, role) = (id(4), id(5));
    create_workspace(&app, b, "B").await?;
    create_user(&app, member_user).await?;
    create_role(&app, a, role, "Viewer").await?;
    let member = id(6);
    run(
        &app,
        a,
        None,
        &format!(
            "INSERT INTO workspace_members (id, workspace_id, user_id) VALUES ('{member}', '{a}', '{member_user}')"
        ),
    )
    .await?;
    create_recipe(&app, b, 80, "draft", None).await?;
    create_recipe(&app, a, 85, "draft", None).await?;

    let assign = |n: u128, kind: &str, target: Uuid| {
        format!(
            "INSERT INTO role_assignments (id, workspace_id, member_id, role_id, scope_kind, scope_id)
             VALUES ('{}', '{a}', '{member}', '{role}', '{kind}', '{target}')",
            id(n)
        )
    };
    // Own batch: fine. Missing target, another workspace's recipe, a kind without a table yet
    // (here a batch id used as a vessel or area): foreign key violation. Real vessel and area
    // scopes are covered in business.rs.
    run(&app, a, None, &assign(90, "batch", id(20))).await?;
    run(&app, a, None, &assign(91, "recipe", id(85))).await?;
    for (n, kind, target) in [
        (92, "batch", id(999)),
        (93, "recipe", id(80)),
        (94, "batch", id(85)),
        (95, "vessel", id(20)),
        (96, "area", id(20)),
    ] {
        let err = run(&app, a, None, &assign(n, kind, target))
            .await
            .unwrap_err();
        assert_eq!(
            sqlstate(&err).as_deref(),
            Some(FOREIGN_KEY_VIOLATION),
            "{kind}"
        );
    }
    // Same rule on invitations; updating the target re-checks.
    let invite = format!(
        "INSERT INTO invitations (id, workspace_id, token_hash, role_id, scope_kind, scope_id, expires_at)
         VALUES ('{}', '{a}', decode(repeat('ab', 32), 'hex'), '{role}', 'recipe', '{}', now() + interval '7 days')",
        id(97),
        id(80)
    );
    let err = run(&app, a, None, &invite).await.unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(FOREIGN_KEY_VIOLATION));
    run(
        &app,
        a,
        None,
        &invite.replace(&id(80).to_string(), &id(85).to_string()),
    )
    .await?;
    let err = run(
        &app,
        a,
        None,
        &format!(
            "UPDATE role_assignments SET scope_id = '{}' WHERE id = '{}'",
            id(80),
            id(91)
        ),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(FOREIGN_KEY_VIOLATION));

    // Deleting the target removes the scoped assignment and invitation (no dangling scope).
    let worker = pool_as(&pool, Role::Worker).await;
    run(
        &worker,
        a,
        None,
        &format!("DELETE FROM recipes WHERE id = '{}'", id(85)),
    )
    .await?;
    let left: (i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM role_assignments WHERE scope_kind = 'recipe'),
                (SELECT count(*) FROM invitations WHERE scope_kind = 'recipe')",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(left, (0, 0));
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn legal_hold_blocks_deletion_until_retention_ends(pool: PgPool) -> sqlx::Result<()> {
    let (app, ws, _) = with_batch(&pool).await;
    let doc = |n: u128, until: &str| {
        format!(
            "INSERT INTO scanned_documents (id, workspace_id, batch_id, form_type, object_key,
                 content_type, size_bytes, sha256, legal_hold, retention_until)
             VALUES ('{}', '{ws}', '{}', 'brew_log', 'k/{n}', 'image/jpeg', 10,
                     decode(repeat('cd', 32), 'hex'), true, {until})",
            id(n),
            id(20)
        )
    };
    run(&app, ws, None, &doc(130, "now() + interval '1 day'")).await?;
    run(&app, ws, None, &doc(131, "now() - interval '1 day'")).await?;
    let err = run(
        &app,
        ws,
        None,
        &format!("DELETE FROM scanned_documents WHERE id = '{}'", id(130)),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY));
    run(
        &app,
        ws,
        None,
        &format!("DELETE FROM scanned_documents WHERE id = '{}'", id(131)),
    )
    .await?;
    // Workspace deletion is blocked as long as a hold is active (adr/0007 › 6.3).
    let err = sqlx::query("DELETE FROM workspaces WHERE id = $1")
        .bind(ws)
        .execute(&pool)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INTEGRITY));
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn server_written_and_user_scoped_tables(pool: PgPool) -> sqlx::Result<()> {
    let (app, ws, user) = with_batch(&pool).await;
    let other = id(7);
    create_user(&app, other).await?;
    let worker = pool_as(&pool, Role::Worker).await;

    // Subscriptions: the API reads, the worker writes.
    let sub = format!(
        "INSERT INTO subscriptions (id, workspace_id, source, external_id, product_key, status)
         VALUES ('{}', '{ws}', 'app_store', 'tx-1', 'pro_monthly', 'active')",
        id(140)
    );
    let err = run(&app, ws, None, &sub).await.unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));
    run(&worker, ws, None, &sub).await?;
    let err = run(
        &app,
        ws,
        None,
        "UPDATE subscriptions SET status = 'expired'",
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));
    // A manual entitlement needs a reason and a support employee.
    let err = run(
        &worker,
        ws,
        None,
        &format!(
            "INSERT INTO entitlements (id, workspace_id, feature, source)
             VALUES ('{}', '{ws}', 'business_mode', 'manual')",
            id(141)
        ),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(CHECK_VIOLATION));

    // Favorites belong to one user inside the workspace.
    run(
        &app,
        ws,
        Some(user),
        &format!(
            "INSERT INTO favorites (id, workspace_id, user_id, batch_id) VALUES ('{}', '{ws}', '{user}', '{}')",
            id(150),
            id(20)
        ),
    )
    .await?;
    for (viewer, expected) in [(user, 1_i64), (other, 0)] {
        let mut tx = app.begin().await?;
        set_context(&mut tx, Some(ws), Some(viewer)).await?;
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM favorites")
            .fetch_one(&mut *tx)
            .await?;
        assert_eq!(n, expected);
        tx.rollback().await?;
    }
    let err = run(
        &app,
        ws,
        Some(other),
        &format!(
            "INSERT INTO favorites (id, workspace_id, user_id, batch_id) VALUES ('{}', '{ws}', '{user}', '{}')",
            id(151),
            id(20)
        ),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));

    // Support grants last at most 72 hours.
    let member_user = id(8);
    create_user(&app, member_user).await?;
    run(
        &app,
        ws,
        None,
        &format!(
            "INSERT INTO workspace_members (id, workspace_id, user_id) VALUES ('{}', '{ws}', '{member_user}')",
            id(9)
        ),
    )
    .await?;
    let grant = |hours: u32| {
        format!(
            "INSERT INTO support_grants (id, workspace_id, granted_by_member_id, starts_at, ends_at)
             VALUES ('{}', '{ws}', '{}', now(), now() + interval '{hours} hours')",
            id(160 + u128::from(hours)),
            id(9)
        )
    };
    run(&app, ws, None, &grant(72)).await?;
    let err = run(&app, ws, None, &grant(73)).await.unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(CHECK_VIOLATION));
    Ok(())
}
