//! Business-mode production and tax tables (T2.7): vessel occupancy exclusion, scope kinds
//! `vessel` and `area`, workspace separation, append-only records, tax rate tables.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{
    Role, create_recipe, create_role, create_user, create_workspace, id, pool_as, run, sqlstate,
};
use sqlx::{PgPool, types::Uuid};
use std::fmt::Write as _;

const EXCLUSION_VIOLATION: &str = "23P01";
const CHECK_VIOLATION: &str = "23514";
const FOREIGN_KEY_VIOLATION: &str = "23503";
const INTEGRITY: &str = "23000";
const PERMISSION_DENIED: &str = "42501";

/// Workspace `ws` with a user and two batches (ids 20 and 21).
async fn with_batches(pool: &PgPool) -> (PgPool, Uuid, Uuid) {
    let app = pool_as(pool, Role::Api).await;
    let (ws, user) = (id(1), id(2));
    create_workspace(&app, ws, "A").await.unwrap();
    create_user(&app, user).await.unwrap();
    create_recipe(&app, ws, 10, "approved", Some(user))
        .await
        .unwrap();
    for (n, code) in [(20, "B-001"), (21, "B-002")] {
        run(
            &app,
            ws,
            None,
            &format!(
                "INSERT INTO batches (id, workspace_id, code, recipe_version_id)
                 VALUES ('{}', '{ws}', '{code}', '{}')",
                id(n),
                id(11)
            ),
        )
        .await
        .unwrap();
    }
    (app, ws, user)
}

async fn vessel(app: &PgPool, ws: Uuid, n: u128) {
    run(
        app,
        ws,
        None,
        &format!(
            "INSERT INTO vessels (id, workspace_id, code, kind, capacity_l)
             VALUES ('{}', '{ws}', 'FV-{n}', 'fermenter', 1000)",
            id(n)
        ),
    )
    .await
    .unwrap();
}

fn occupancy(ws: Uuid, n: u128, batch: u128, vessel: u128, status: &str, period: &str) -> String {
    format!(
        "INSERT INTO vessel_occupancies (id, workspace_id, batch_id, vessel_id, status, period, volume_l)
         VALUES ('{}', '{ws}', '{}', '{}', '{status}', '{period}', 500)",
        id(n),
        id(batch),
        id(vessel)
    )
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn actual_vessel_occupancies_cannot_overlap(pool: PgPool) -> sqlx::Result<()> {
    let (app, ws, _) = with_batches(&pool).await;
    vessel(&app, ws, 30).await;
    vessel(&app, ws, 31).await;
    let jan = "[2026-11-01 00:00+00,2026-11-10 00:00+00)";
    run(&app, ws, None, &occupancy(ws, 40, 20, 30, "actual", jan)).await?;

    // Overlap in the same vessel is refused, whichever batch it belongs to.
    for (n, batch, period) in [
        (41, 21, "[2026-11-05 00:00+00,2026-11-12 00:00+00)"),
        (42, 20, "[2026-10-25 00:00+00,2026-11-02 00:00+00)"),
        (43, 21, "[2026-11-03 00:00+00,)"),
    ] {
        let err = run(
            &app,
            ws,
            None,
            &occupancy(ws, n, batch, 30, "actual", period),
        )
        .await
        .unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(EXCLUSION_VIOLATION), "{n}");
    }
    // Touching ranges, another vessel and planned occupancies (a planning warning only) pass.
    run(
        &app,
        ws,
        None,
        &occupancy(ws, 44, 21, 30, "actual", "[2026-11-10 00:00+00,)"),
    )
    .await?;
    run(&app, ws, None, &occupancy(ws, 45, 21, 31, "actual", jan)).await?;
    run(&app, ws, None, &occupancy(ws, 46, 21, 30, "planned", jan)).await?;
    run(&app, ws, None, &occupancy(ws, 47, 20, 30, "planned", jan)).await?;
    // Promoting a planned row into a taken slot is refused; a soft-deleted actual row frees it.
    let promote = format!(
        "UPDATE vessel_occupancies SET status = 'actual' WHERE id = '{}'",
        id(46)
    );
    let err = run(&app, ws, None, &promote).await.unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(EXCLUSION_VIOLATION));
    run(
        &app,
        ws,
        None,
        &format!(
            "UPDATE vessel_occupancies SET deleted_at = now() WHERE id = '{}'",
            id(40)
        ),
    )
    .await?;
    run(&app, ws, None, &promote).await?;

    // Shape checks: empty, unbounded-below ranges and an actual row without a volume.
    for (n, status, period, volume) in [
        (
            50,
            "planned",
            "[2026-11-01 00:00+00,2026-11-01 00:00+00)",
            "1",
        ),
        (51, "planned", "(,2026-11-01 00:00+00)", "1"),
        (
            52,
            "actual",
            "[2027-01-01 00:00+00,2027-01-02 00:00+00)",
            "NULL",
        ),
        (
            53,
            "planned",
            "[2027-01-01 00:00+00,2027-01-02 00:00+00)",
            "0",
        ),
    ] {
        let sql = occupancy(ws, n, 20, 31, status, period).replace("500)", &format!("{volume})"));
        let err = run(&app, ws, None, &sql).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(CHECK_VIOLATION), "{n}");
    }
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn vessel_and_area_scopes_work_and_follow_their_targets(pool: PgPool) -> sqlx::Result<()> {
    let (app, a, _) = with_batches(&pool).await;
    let (b, member_user, role, member) = (id(3), id(4), id(5), id(6));
    create_workspace(&app, b, "B").await?;
    create_user(&app, member_user).await?;
    create_role(&app, a, role, "Operator").await?;
    run(
        &app,
        a,
        None,
        &format!(
            "INSERT INTO workspace_members (id, workspace_id, user_id) VALUES ('{member}', '{a}', '{member_user}')"
        ),
    )
    .await?;
    run(
        &app,
        a,
        None,
        &format!(
            "INSERT INTO production_areas (id, workspace_id, name) VALUES ('{}', '{a}', 'Cellar')",
            id(60)
        ),
    )
    .await?;
    vessel(&app, a, 30).await;
    vessel(&app, b, 31).await;

    let assign = |n: u128, kind: &str, target: Uuid| {
        format!(
            "INSERT INTO role_assignments (id, workspace_id, member_id, role_id, scope_kind, scope_id)
             VALUES ('{}', '{a}', '{member}', '{role}', '{kind}', '{target}')",
            id(n)
        )
    };
    run(&app, a, None, &assign(90, "vessel", id(30))).await?;
    run(&app, a, None, &assign(91, "area", id(60))).await?;
    // Another workspace's vessel, a missing area and a vessel id used as an area are refused.
    for (n, kind, target) in [
        (92, "vessel", id(31)),
        (93, "area", id(999)),
        (94, "area", id(30)),
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
    // An alert rule may be scoped to a vessel as well.
    run(
        &app,
        a,
        None,
        &format!(
            "INSERT INTO alert_rules (id, workspace_id, name, condition, scope_kind, scope_id)
             VALUES ('{}', '{a}', 'Too warm', '{{\"schema_version\": 1}}', 'vessel', '{}')",
            id(95),
            id(30)
        ),
    )
    .await?;

    // devices.vessel_id is a real foreign key: unknown or foreign vessels are refused, and
    // deleting the vessel clears only the link.
    let device = |n: u128, vessel: Uuid| {
        format!(
            "INSERT INTO devices (id, workspace_id, kind, name, ingest_key_hash, vessel_id)
             VALUES ('{}', '{a}', 'tilt', 'T', decode(repeat('{n:02x}', 32), 'hex'), '{vessel}')",
            id(n)
        )
    };
    for target in [id(999), id(31)] {
        let err = run(&app, a, None, &device(70, target)).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(FOREIGN_KEY_VIOLATION));
    }
    run(&app, a, None, &device(71, id(30))).await?;

    let worker = pool_as(&pool, Role::Worker).await;
    for (table, target) in [("vessels", id(30)), ("production_areas", id(60))] {
        run(
            &worker,
            a,
            None,
            &format!("DELETE FROM {table} WHERE id = '{target}'"),
        )
        .await?;
    }
    let left: (i64, i64, Option<Uuid>, Uuid) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM role_assignments WHERE scope_kind IN ('vessel', 'area')),
                (SELECT count(*) FROM alert_rules WHERE scope_kind = 'vessel'),
                (SELECT vessel_id FROM devices WHERE id = $1),
                (SELECT workspace_id FROM devices WHERE id = $1)",
    )
    .bind(id(71))
    .fetch_one(&pool)
    .await?;
    assert_eq!((left.0, left.1, left.2, left.3), (0, 0, None, a));
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn business_tables_are_separated_by_workspace(pool: PgPool) -> sqlx::Result<()> {
    let (app, a, _) = with_batches(&pool).await;
    let b = id(3);
    create_workspace(&app, b, "B").await?;
    vessel(&app, a, 30).await;
    run(
        &app,
        a,
        None,
        &occupancy(a, 40, 20, 30, "actual", "[2026-11-01 00:00+00,)"),
    )
    .await?;

    // B sees nothing of A's rows.
    for table in ["vessels", "vessel_occupancies", "batches"] {
        let mut tx = app.begin().await?;
        common::set_context(&mut tx, Some(b), None).await?;
        let n: i64 =
            sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT count(*) FROM {table}")))
                .fetch_one(&mut *tx)
                .await?;
        assert_eq!(n, 0, "{table}");
    }
    // B cannot write rows into A, nor point its own rows at A's vessel.
    let err = run(
        &app,
        b,
        None,
        &occupancy(a, 41, 20, 30, "planned", "[2027-01-01 00:00+00,)"),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));
    let err = run(
        &app,
        b,
        None,
        &format!(
            "INSERT INTO cleaning_records (id, workspace_id, vessel_id) VALUES ('{}', '{b}', '{}')",
            id(42),
            id(30)
        ),
    )
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(FOREIGN_KEY_VIOLATION));
    // The exclusion constraint does not leak across workspaces either: B's own vessel with
    // the same period is its own business (a different vessel id, no overlap).
    vessel(&app, b, 31).await;
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn records_are_append_only_and_checked(pool: PgPool) -> sqlx::Result<()> {
    let (app, ws, user) = with_batches(&pool).await;
    vessel(&app, ws, 30).await;
    vessel(&app, ws, 31).await;
    let ex = |sql: String| {
        let app = app.clone();
        async move { run(&app, ws, None, &sql).await }
    };
    let expect = |r: sqlx::Result<()>, code: &str, what: &str| {
        assert_eq!(sqlstate(&r.unwrap_err()).as_deref(), Some(code), "{what}");
    };

    // Transfers: kind and vessels agree; loss cannot exceed volume.
    let transfer = |n: u128, kind: &str, src: &str, dst: &str, loss: &str| {
        format!(
            "INSERT INTO transfers (id, workspace_id, kind, source_vessel_id, target_vessel_id, volume_l, loss_l)
             VALUES ('{}', '{ws}', '{kind}', {src}, {dst}, 100, {loss})",
            id(n)
        )
    };
    let (v30, v31) = (format!("'{}'", id(30)), format!("'{}'", id(31)));
    ex(transfer(80, "transfer", &v30, &v31, "2")).await?;
    ex(transfer(81, "disposal", &v30, "NULL", "0")).await?;
    ex(transfer(82, "receipt", "NULL", &v31, "0")).await?;
    expect(
        ex(transfer(83, "transfer", &v30, &v30, "0")).await,
        CHECK_VIOLATION,
        "same vessel",
    );
    expect(
        ex(transfer(84, "transfer", &v30, "NULL", "0")).await,
        CHECK_VIOLATION,
        "no target",
    );
    expect(
        ex(transfer(85, "disposal", &v30, &v31, "0")).await,
        CHECK_VIOLATION,
        "disposal target",
    );
    expect(
        ex(transfer(86, "transfer", &v30, &v31, "101")).await,
        CHECK_VIOLATION,
        "loss",
    );
    ex(format!(
        "INSERT INTO transfer_batches (id, workspace_id, transfer_id, batch_id, volume_l)
         VALUES ('{}', '{ws}', '{}', '{}', 50)",
        id(87),
        id(80),
        id(20)
    ))
    .await?;

    // Releases and lab results are append-only; rejection needs a reason.
    let release = |n: u128, decision: &str, reason: &str| {
        format!(
            "INSERT INTO releases (id, workspace_id, batch_id, decision, decided_by, reason)
             VALUES ('{}', '{ws}', '{}', '{decision}', '{user}', {reason})",
            id(n),
            id(20)
        )
    };
    ex(release(100, "released", "NULL")).await?;
    ex(release(101, "rejected", "'diacetyl'")).await?;
    expect(
        ex(release(102, "rejected", "NULL")).await,
        CHECK_VIOLATION,
        "reason",
    );
    expect(
        ex(release(103, "held", "'  '")).await,
        CHECK_VIOLATION,
        "blank reason",
    );
    expect(
        ex(format!(
            "UPDATE releases SET decision = 'released' WHERE id = '{}'",
            id(101)
        ))
        .await,
        PERMISSION_DENIED,
        "update release",
    );
    ex(format!(
        "INSERT INTO lab_results (id, workspace_id, batch_id, parameter, value, source)
         VALUES ('{}', '{ws}', '{}', 'ph', 4.3, 'lab')",
        id(110),
        id(20)
    ))
    .await?;
    ex(format!(
        "INSERT INTO lab_results (id, workspace_id, batch_id, parameter, value, source, corrects_id)
         VALUES ('{}', '{ws}', '{}', 'ph', 4.2, 'lab', '{}')",
        id(111),
        id(20),
        id(110)
    ))
    .await?;
    for sql in [
        format!("UPDATE lab_results SET value = 1 WHERE id = '{}'", id(110)),
        format!("DELETE FROM lab_results WHERE id = '{}'", id(110)),
    ] {
        expect(ex(sql).await, PERMISSION_DENIED, "lab result mutation");
    }

    // Quality specs: one per recipe and parameter, range ordered.
    let spec = |n: u128, param: &str, min: &str, max: &str| {
        format!(
            "INSERT INTO quality_specs (id, workspace_id, recipe_id, parameter, min_value, max_value)
             VALUES ('{}', '{ws}', '{}', '{param}', {min}, {max})",
            id(n),
            id(9)
        )
    };
    let recipe: Option<Uuid> =
        sqlx::query_scalar("SELECT recipe_id FROM recipe_versions WHERE id = $1")
            .bind(id(11))
            .fetch_optional(&pool)
            .await?;
    let recipe = recipe.expect("recipe of the version");
    let spec = |n: u128, param: &str, min: &str, max: &str| {
        spec(n, param, min, max).replace(&id(9).to_string(), &recipe.to_string())
    };
    ex(spec(120, "abv_pct", "4.5", "5.5")).await?;
    expect(
        ex(spec(121, "abv_pct", "4", "6")).await,
        "23505",
        "duplicate parameter",
    );
    expect(
        ex(spec(122, "ph", "5", "4")).await,
        CHECK_VIOLATION,
        "inverted range",
    );
    expect(
        ex(spec(123, "ph", "NULL", "NULL")).await,
        CHECK_VIOLATION,
        "no bounds",
    );

    // Cleaning verification travels together.
    let clean = |n: u128, extra: &str| {
        format!(
            "INSERT INTO cleaning_records (id, workspace_id, vessel_id, chemical, concentration_pct{extra})
             VALUES ('{}', '{ws}', '{}', 'PAA', 0.5{})",
            id(n),
            id(30),
            if extra.is_empty() { "" } else { ", now(), 'atp'" }
        )
    };
    ex(clean(130, "")).await?;
    expect(
        ex(clean(131, ", verified_at, verification_method")).await,
        CHECK_VIOLATION,
        "verified_at without verified_by",
    );

    // Yeast generations chain by exactly one generation and the same yeast.
    sqlx::query(
        "INSERT INTO ingredients (id, workspace_id, kind, name, source)
         VALUES ($1, NULL, 'yeast', 'US-05', 'library'), ($2, NULL, 'yeast', 'S-04', 'library')",
    )
    .bind(id(140))
    .bind(id(141))
    .execute(&pool)
    .await?;
    let generation = |n: u128, yeast: u128, generation: i32, parent: Option<u128>| {
        format!(
            "INSERT INTO yeast_generations (id, workspace_id, yeast_ingredient_id, generation, parent_id, source_batch_id)
             VALUES ('{}', '{ws}', '{}', {generation}, {}, '{}')",
            id(n),
            id(yeast),
            parent.map_or("NULL".to_owned(), |p| format!("'{}'", id(p))),
            id(20)
        )
    };
    ex(generation(150, 140, 0, None)).await?;
    ex(generation(151, 140, 1, Some(150))).await?;
    expect(
        ex(generation(152, 140, 3, Some(151))).await,
        INTEGRITY,
        "skipped generation",
    );
    expect(
        ex(generation(153, 141, 1, Some(150))).await,
        INTEGRITY,
        "other yeast",
    );
    Ok(())
}

// ---- tax tables -------------------------------------------------------------------------

async fn jurisdiction(pool: &PgPool) {
    sqlx::query("INSERT INTO tax_jurisdictions (id, code, name, currency) VALUES ($1, 'GB', 'United Kingdom', 'GBP')")
        .bind(id(1))
        .execute(pool)
        .await
        .unwrap();
}

fn rate_table(n: u128, position: i16, period: &str) -> String {
    format!(
        "INSERT INTO tax_rate_tables (id, jurisdiction_id, chain_position, method, valid_during, taxable_event,
             abv_source, input_rounding, input_dp, bounds_inclusive)
         VALUES ('{}', '{}', {position}, 'saf_alkol', '{period}', 'excise_exit', 'measured', 'down', 1, false)",
        id(n),
        id(1)
    )
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn tax_validity_ranges_cannot_overlap(pool: PgPool) -> sqlx::Result<()> {
    jurisdiction(&pool).await;
    let q = |sql: String| {
        let pool = pool.clone();
        async move {
            sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
                .execute(&pool)
                .await
                .map(|_| ())
        }
    };
    q(rate_table(10, 1, "[2025-02-01,2026-02-01)")).await?;
    q(rate_table(11, 1, "[2026-02-01,)")).await?; // adjacent: fine
    for (n, position, period) in [
        (12, 1, "[2026-06-01,2027-01-01)"),
        (13, 1, "[2024-01-01,2025-02-02)"),
    ] {
        let err = q(rate_table(n, position, period)).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(EXCLUSION_VIOLATION), "{n}");
    }
    // Another chain step (or another rate class) may run in parallel; so may another country.
    q(rate_table(14, 2, "[2026-02-01,)")).await?;
    q(rate_table(15, 1, "[2026-02-01,)")
        .replace("chain_position,", "rate_class, chain_position,")
        .replace(
            &format!("'{}', 1,", id(1)),
            &format!("'{}', 'draught', 1,", id(1)),
        ))
    .await?;
    sqlx::query("INSERT INTO tax_jurisdictions (id, code, name, currency) VALUES ($1, 'DE', 'Germany', 'EUR')")
        .bind(id(2))
        .execute(&pool)
        .await?;
    q(rate_table(16, 1, "[2026-02-01,)")
        .replace(&format!("'{}', 1,", id(1)), &format!("'{}', 1,", id(2))))
    .await?;
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn tax_rows_are_validated_and_read_only_for_the_application(
    pool: PgPool,
) -> sqlx::Result<()> {
    jurisdiction(&pool).await;
    let q = |sql: String| {
        let pool = pool.clone();
        async move {
            sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
                .execute(&pool)
                .await
                .map(|_| ())
        }
    };
    q(rate_table(10, 1, "[2026-02-01,)")).await?;

    // Bands: strictly increasing, open-ended last, checked when the transaction commits.
    let bands = |rows: &[(i16, &str, &str)]| {
        let mut sql = String::new();
        for (pos, upper, rate) in rows {
            let _ = write!(
                sql,
                "INSERT INTO tax_rate_bands (id, table_id, position, upper_bound, rate)
                 VALUES ('{}', '{}', {pos}, {upper}, {rate});",
                id(200 + u128::try_from(*pos).unwrap()),
                id(10)
            );
        }
        sql
    };
    for bad in [
        bands(&[(1, "3.5", "9.27"), (2, "8.5", "21.78")]), // last closed
        bands(&[(1, "8.5", "9.27"), (2, "3.5", "21.78"), (3, "NULL", "29")]), // decreasing
        bands(&[(1, "NULL", "9.27"), (2, "NULL", "21.78")]), // open before last
        bands(&[(1, "3.5", "9.27"), (3, "NULL", "29.54")]), // gap in positions
    ] {
        let err = q(bad).await.unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(CHECK_VIOLATION));
    }
    q(bands(&[
        (1, "3.5", "9.27"),
        (2, "8.5", "21.78"),
        (3, "NULL", "29.54"),
    ]))
    .await?;
    // Deleting the middle band later breaks the chain and is refused too.
    let err = q(format!(
        "DELETE FROM tax_rate_bands WHERE id = '{}'",
        id(202)
    ))
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(CHECK_VIOLATION));

    // Table-level rules: undefined input rounding, verified row without an official source.
    let err = q(rate_table(11, 2, "[2026-02-01,)").replace("'down', 1", "NULL, NULL"))
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(CHECK_VIOLATION));
    q(rate_table(12, 3, "[2026-02-01,)").replace("'down', 1", "'none', NULL")).await?;
    let err = q(format!(
        "UPDATE tax_rate_tables SET status = 'verified' WHERE id = '{}'",
        id(10)
    ))
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(CHECK_VIOLATION));
    q(format!(
        "UPDATE tax_rate_tables SET status = 'verified', source_official = true, source_name = 'gov.uk',
             source_url = 'https://www.gov.uk/guidance/alcohol-duty-rates', source_date = '2026-02-01'
         WHERE id = '{}'",
        id(10)
    ))
    .await?;

    // The application reads but never writes (shared reference data).
    for role in [Role::Api, Role::Worker] {
        let app = pool_as(&pool, role).await;
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM tax_rate_bands")
            .fetch_one(&app)
            .await?;
        assert_eq!(n, 3);
        let err = sqlx::query("UPDATE tax_rate_tables SET notes = 'x'")
            .execute(&app)
            .await
            .unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));
        let err = sqlx::query("DELETE FROM tax_jurisdictions")
            .execute(&app)
            .await
            .unwrap_err();
        assert_eq!(sqlstate(&err).as_deref(), Some(PERMISSION_DENIED));
    }
    Ok(())
}
