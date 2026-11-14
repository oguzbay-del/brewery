//! Catalog scan (ADR-0007, rule 2.8): isolation must be enforced for every table, not only
//! for the ones the other tests happen to touch. T2.6 and T2.7 add tables and are covered by
//! the same scan without changes here (T2.6 added the library, recipe, batch, stock,
//! notification and support tables, partitions of `measurements` included).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use sqlx::PgPool;

/// Cross-workspace tables (ADR-0007, rule 2.5): protected by privileges instead of RLS.
const CROSS_WORKSPACE: &[&str] = &[
    "outbox",
    "scheduled_jobs",
    "external_events",
    "processed_events",
    // Written only by `purge_workspace` (T2.9); the id column is not `workspace_id` on purpose.
    "workspace_tombstones",
];
/// Shared tables written only by migrations and seeds; no RLS, read-only for the API.
const SHARED: &[&str] = &[
    "staff_users",
    "tax_jurisdictions",
    "tax_rate_tables",
    "tax_rate_bands",
    "retention_rules",
];
/// Tables whose `app_owner` policies serve the pre-login SECURITY DEFINER functions.
const OWNER_LOOKUP: &[&str] = &[
    "users",
    "user_identities",
    "sessions",
    "email_login_tokens",
    "auth_requests",
    "shared_devices",
    "invitations",
    "idempotency_keys",
    "push_targets",
    "workspaces",
];

/// Shared-library tables (`workspace_id` NULL = shared row): `app_owner` seeds them through the
/// `owner_seed` policy (migrations and seed jobs); the application roles cannot write NULL rows.
const OWNER_SEED: &[&str] = &[
    "ingredients",
    "ingredient_fermentables",
    "ingredient_hops",
    "ingredient_yeasts",
    "ingredient_miscs",
    "style_guides",
    "styles",
    "style_mappings",
    "package_types",
];

fn names(rows: Vec<(String,)>) -> Vec<String> {
    rows.into_iter().map(|(n,)| n).collect()
}

/// Workspace tables (a `workspace_id` column) lacking RLS, FORCE or a policy.
async fn unprotected_workspace_tables(pool: &PgPool) -> sqlx::Result<Vec<String>> {
    Ok(names(
        sqlx::query_as(
            "SELECT c.relname::text
             FROM pg_class c
             JOIN pg_namespace n ON n.oid = c.relnamespace AND n.nspname = 'public'
             JOIN pg_attribute a ON a.attrelid = c.oid AND a.attname = 'workspace_id' AND NOT a.attisdropped
             WHERE c.relkind IN ('r', 'p') AND c.relname <> ALL ($1)
               AND (NOT c.relrowsecurity OR NOT c.relforcerowsecurity
                    OR NOT EXISTS (SELECT 1 FROM pg_policy p WHERE p.polrelid = c.oid))
             ORDER BY 1",
        )
        .bind(CROSS_WORKSPACE)
        .fetch_all(pool)
        .await?,
    ))
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn every_workspace_table_has_forced_rls_and_a_policy(pool: PgPool) -> sqlx::Result<()> {
    let missing = unprotected_workspace_tables(&pool).await?;
    assert!(missing.is_empty(), "no RLS/FORCE/policy: {missing:?}");

    // The same holds for every other table except the declared exceptions: account tables
    // (user_id policies), `workspaces` and `audit_heads` included.
    let unprotected = names(
        sqlx::query_as(
            "SELECT c.relname::text
             FROM pg_class c
             JOIN pg_namespace n ON n.oid = c.relnamespace AND n.nspname = 'public'
             WHERE c.relkind IN ('r', 'p') AND c.relname <> '_sqlx_migrations'
               AND c.relname <> ALL ($1) AND c.relname <> ALL ($2)
               AND (NOT c.relrowsecurity OR NOT c.relforcerowsecurity
                    OR NOT EXISTS (SELECT 1 FROM pg_policy p WHERE p.polrelid = c.oid))
             ORDER BY 1",
        )
        .bind(CROSS_WORKSPACE)
        .bind(SHARED)
        .fetch_all(&pool)
        .await?,
    );
    assert!(
        unprotected.is_empty(),
        "tables without RLS: {unprotected:?}"
    );
    Ok(())
}

/// The scan is not vacuous: each way of getting it wrong is reported.
#[sqlx::test(migrator = "db::MIGRATOR")]
async fn the_scan_detects_unprotected_tables(pool: PgPool) -> sqlx::Result<()> {
    for ddl in [
        "CREATE TABLE leak_none (workspace_id uuid NOT NULL)",
        "CREATE TABLE leak_no_force (workspace_id uuid NOT NULL);
         ALTER TABLE leak_no_force ENABLE ROW LEVEL SECURITY;
         CREATE POLICY p ON leak_no_force USING (true)",
        "CREATE TABLE leak_no_policy (workspace_id uuid NOT NULL);
         ALTER TABLE leak_no_policy ENABLE ROW LEVEL SECURITY;
         ALTER TABLE leak_no_policy FORCE ROW LEVEL SECURITY",
    ] {
        sqlx::raw_sql(ddl).execute(&pool).await?;
    }
    let found = unprotected_workspace_tables(&pool).await?;
    assert_eq!(found, ["leak_no_force", "leak_no_policy", "leak_none"]);
    Ok(())
}

/// Single-column foreign keys between two workspace tables (referential checks bypass RLS, so
/// such a key would let one workspace point at another's row). References to shared-capable
/// tables (nullable `workspace_id`) are exempt: a trigger checks them (see below).
async fn single_column_references(pool: &PgPool) -> sqlx::Result<Vec<String>> {
    Ok(names(
        sqlx::query_as(
            "SELECT con.conrelid::regclass::text || '.' || con.conname
             FROM pg_constraint con
             WHERE con.contype = 'f'
               AND EXISTS (SELECT 1 FROM pg_attribute a WHERE a.attrelid = con.conrelid
                           AND a.attname = 'workspace_id' AND NOT a.attisdropped)
               AND EXISTS (SELECT 1 FROM pg_attribute a WHERE a.attrelid = con.confrelid
                           AND a.attname = 'workspace_id' AND NOT a.attisdropped)
               AND NOT EXISTS (SELECT 1 FROM pg_attribute a WHERE a.attrelid = con.conrelid
                               AND a.attname = 'workspace_id' AND a.attnum = ANY (con.conkey))
               AND EXISTS (SELECT 1 FROM pg_attribute a WHERE a.attrelid = con.confrelid
                           AND a.attname = 'workspace_id' AND a.attnotnull)
             ORDER BY 1",
        )
        .fetch_all(pool)
        .await?,
    ))
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn workspace_references_use_composite_foreign_keys(pool: PgPool) -> sqlx::Result<()> {
    let offenders = single_column_references(&pool).await?;
    assert!(
        offenders.is_empty(),
        "single-column references: {offenders:?}"
    );
    Ok(())
}

/// The scans also cover partitions and references added later (negative tests).
#[sqlx::test(migrator = "db::MIGRATOR")]
async fn the_scans_detect_an_unprotected_partition_and_a_single_column_reference(
    pool: PgPool,
) -> sqlx::Result<()> {
    sqlx::raw_sql(
        "CREATE TABLE measurements_y2099m01 PARTITION OF measurements
             FOR VALUES FROM ('2099-01-01+00') TO ('2099-02-01+00');
         CREATE TABLE leak_fk (id uuid PRIMARY KEY, workspace_id uuid NOT NULL,
                               recipe_id uuid REFERENCES recipes (id))",
    )
    .execute(&pool)
    .await?;
    assert_eq!(
        unprotected_workspace_tables(&pool).await?,
        ["leak_fk", "measurements_y2099m01"]
    );
    assert_eq!(
        single_column_references(&pool).await?,
        ["leak_fk.leak_fk_recipe_id_fkey"]
    );
    Ok(())
}

/// Foreign keys from a workspace table to a shared-capable table (nullable `workspace_id`)
/// that lack the checking trigger: the referenced row could belong to another workspace.
async fn unchecked_shared_references(pool: &PgPool) -> sqlx::Result<Vec<String>> {
    Ok(names(
        sqlx::query_as(
            "SELECT con.conrelid::regclass::text || '.' || a.attname
             FROM pg_constraint con
             JOIN pg_attribute a ON a.attrelid = con.conrelid AND a.attnum = con.conkey[1]
             WHERE con.contype = 'f' AND array_length(con.conkey, 1) = 1
               AND EXISTS (SELECT 1 FROM pg_attribute w WHERE w.attrelid = con.confrelid
                           AND w.attname = 'workspace_id' AND NOT w.attnotnull)
               AND EXISTS (SELECT 1 FROM pg_attribute w WHERE w.attrelid = con.conrelid
                           AND w.attname = 'workspace_id' AND NOT w.attisdropped)
               AND NOT EXISTS (
                   SELECT 1 FROM pg_trigger t
                   WHERE t.tgrelid = con.conrelid AND NOT t.tgisinternal
                     AND t.tgname = 'shared_ref_' || a.attname
                     AND t.tgfoid = 'check_shared_ref'::regproc)
             ORDER BY 1",
        )
        .fetch_all(pool)
        .await?,
    ))
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn references_to_shared_tables_are_checked_by_trigger(pool: PgPool) -> sqlx::Result<()> {
    let unchecked = unchecked_shared_references(&pool).await?;
    assert!(unchecked.is_empty(), "unchecked: {unchecked:?}");

    // Not vacuous: a reference without the trigger is reported, the trigger clears it.
    sqlx::raw_sql(
        "CREATE TABLE leak_ref (id uuid PRIMARY KEY, workspace_id uuid NOT NULL,
                                ingredient_id uuid REFERENCES ingredients (id))",
    )
    .execute(&pool)
    .await?;
    assert_eq!(
        unchecked_shared_references(&pool).await?,
        ["leak_ref.ingredient_id"]
    );
    sqlx::raw_sql(
        "CREATE TRIGGER shared_ref_ingredient_id BEFORE INSERT ON leak_ref
         FOR EACH ROW EXECUTE FUNCTION check_shared_ref('ingredients', 'ingredient_id')",
    )
    .execute(&pool)
    .await?;
    assert!(unchecked_shared_references(&pool).await?.is_empty());
    Ok(())
}

/// Every table the application can update carries the common columns (ADR-0007, rule 1.2).
/// Exceptions are the T2.5 tables keyed by their natural key and the cross-workspace tables.
#[sqlx::test(migrator = "db::MIGRATOR")]
async fn mutable_tables_carry_the_common_columns(pool: PgPool) -> sqlx::Result<()> {
    let missing = names(
        sqlx::query_as(
            "SELECT c.relname::text || '.' || col.name
             FROM pg_class c
             JOIN pg_namespace n ON n.oid = c.relnamespace AND n.nspname = 'public'
             CROSS JOIN (VALUES ('created_at'), ('updated_at'), ('deleted_at'), ('version'),
                                ('created_by'), ('updated_by')) col(name)
             WHERE c.relkind IN ('r', 'p') AND c.relname <> ALL ($1)
               AND (has_table_privilege('app_api', c.oid, 'UPDATE')
                    OR has_table_privilege('app_worker', c.oid, 'UPDATE'))
               AND EXISTS (SELECT 1 FROM pg_attribute a WHERE a.attrelid = c.oid
                           AND a.attname = 'workspace_id' AND NOT a.attisdropped)
               AND NOT EXISTS (SELECT 1 FROM pg_attribute a WHERE a.attrelid = c.oid
                               AND a.attname = col.name AND NOT a.attisdropped)
             ORDER BY 1",
        )
        .bind(
            [
                "audit_heads",
                "role_permissions",
                "outbox",
                "scheduled_jobs",
                "external_events",
                "idempotency_keys",
            ]
            .as_slice(),
        )
        .fetch_all(&pool)
        .await?,
    );
    assert!(missing.is_empty(), "missing common columns: {missing:?}");
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn measurement_partitions_exist_and_are_protected(pool: PgPool) -> sqlx::Result<()> {
    let parts: Vec<(String, bool, bool, i64)> = sqlx::query_as(
        "SELECT c.relname::text, c.relrowsecurity, c.relforcerowsecurity,
                (SELECT count(*) FROM pg_policy p WHERE p.polrelid = c.oid)
         FROM pg_inherits i JOIN pg_class c ON c.oid = i.inhrelid
         WHERE i.inhparent = 'measurements'::regclass ORDER BY 1",
    )
    .fetch_all(&pool)
    .await?;
    assert!(parts.len() >= 14, "partitions: {}", parts.len());
    assert_eq!(parts[0].0, "measurements_legacy");
    assert_eq!(parts[1].0, "measurements_y2026m01");
    for (name, rls, force, policies) in parts {
        assert!(rls && force && policies >= 1, "{name} is not protected");
    }
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn application_roles_hold_only_the_intended_rights(pool: PgPool) -> sqlx::Result<()> {
    // No application role may own, truncate, reference or add triggers to anything.
    let too_much = names(
        sqlx::query_as(
            "SELECT r.rolname || ' ' || p.priv || ' ' || c.relname
             FROM pg_class c
             JOIN pg_namespace n ON n.oid = c.relnamespace AND n.nspname = 'public'
             CROSS JOIN (VALUES ('app_api'), ('app_worker')) r(rolname)
             CROSS JOIN (VALUES ('TRUNCATE'), ('REFERENCES'), ('TRIGGER')) p(priv)
             WHERE c.relkind IN ('r', 'p')
               AND has_table_privilege(r.rolname, c.oid, p.priv)
             ORDER BY 1",
        )
        .fetch_all(&pool)
        .await?,
    );
    assert!(too_much.is_empty(), "excess privileges: {too_much:?}");

    // The API only inserts into the cross-workspace tables (plus its own dedupe read).
    for (table, role, privilege, expected) in [
        ("outbox", "app_api", "INSERT", true),
        ("outbox", "app_api", "SELECT", false),
        ("outbox", "app_api", "UPDATE", false),
        ("outbox", "app_api", "DELETE", false),
        ("outbox", "app_worker", "SELECT", true),
        ("outbox", "app_worker", "UPDATE", true),
        ("outbox", "app_worker", "DELETE", true),
        ("scheduled_jobs", "app_api", "SELECT", false),
        ("scheduled_jobs", "app_api", "INSERT", true),
        ("scheduled_jobs", "app_worker", "DELETE", true),
        ("external_events", "app_api", "INSERT", true),
        // The API asks `external_event_seen()` instead of reading webhook payloads (T2.9).
        ("external_events", "app_api", "SELECT", false),
        ("external_events", "app_api", "DELETE", false),
        ("audit_log", "app_api", "UPDATE", false),
        ("audit_log", "app_api", "DELETE", false),
        ("audit_log", "app_worker", "UPDATE", false),
        ("audit_log", "app_worker", "DELETE", false),
        ("audit_log", "app_api", "INSERT", true),
        ("email_login_tokens", "app_api", "SELECT", false),
        ("auth_requests", "app_api", "INSERT", false),
        ("staff_users", "app_api", "INSERT", false),
        ("staff_users", "app_api", "SELECT", true),
        // Append-only ledgers: no UPDATE or DELETE for the application (T2.6).
        ("measurements", "app_api", "INSERT", true),
        ("measurements", "app_api", "UPDATE", false),
        ("measurements", "app_worker", "DELETE", false),
        ("stage_events", "app_api", "UPDATE", false),
        ("stock_movements", "app_api", "INSERT", true),
        ("stock_movements", "app_api", "UPDATE", false),
        ("stock_movements", "app_worker", "DELETE", false),
        ("notification_events", "app_api", "DELETE", false),
        ("support_actions", "app_api", "INSERT", true),
        ("support_actions", "app_api", "UPDATE", false),
        ("calc_results", "app_api", "UPDATE", false),
        // Written by the server side only (ingest to worker): the API reads.
        ("subscriptions", "app_api", "SELECT", true),
        ("subscriptions", "app_api", "INSERT", false),
        ("subscriptions", "app_worker", "UPDATE", true),
        ("entitlements", "app_api", "UPDATE", false),
        ("entitlements", "app_worker", "INSERT", true),
        // Library tables: read and write are policy-restricted, not privilege-restricted.
        ("ingredients", "app_api", "INSERT", true),
    ] {
        let has: bool = sqlx::query_scalar("SELECT has_table_privilege($1, $2, $3)")
            .bind(role)
            .bind(table)
            .bind(privilege)
            .fetch_one(&pool)
            .await?;
        assert_eq!(has, expected, "{role} {privilege} on {table}");
    }
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn owner_policies_exist_only_where_a_definer_function_needs_them(
    pool: PgPool,
) -> sqlx::Result<()> {
    let tables = names(
        sqlx::query_as(
            "SELECT DISTINCT c.relname::text
             FROM pg_policy p
             JOIN pg_class c ON c.oid = p.polrelid
             WHERE p.polroles <> '{0}'::oid[]
               AND NOT EXISTS (SELECT 1 FROM unnest(p.polroles) r(oid) WHERE r.oid <> 'app_owner'::regrole)
             ORDER BY 1",
        )
        .fetch_all(&pool)
        .await?,
    );
    let mut expected: Vec<String> = OWNER_LOOKUP
        .iter()
        .chain(OWNER_SEED)
        .map(|s| (*s).to_owned())
        .collect();
    expected.sort();
    assert_eq!(tables, expected);

    // Every SECURITY DEFINER function belongs to app_owner, pins its search_path and is not
    // callable by PUBLIC.
    let loose = names(
        sqlx::query_as(
            "SELECT p.proname::text
             FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace AND n.nspname = 'public'
             WHERE p.prosecdef
               AND (p.proowner <> 'app_owner'::regrole
                    OR NOT EXISTS (SELECT 1 FROM unnest(coalesce(p.proconfig, '{}')) c
                                   WHERE c LIKE 'search_path=%')
                    OR has_function_privilege('public', p.oid, 'EXECUTE'))
             ORDER BY 1",
        )
        .fetch_all(&pool)
        .await?,
    );
    assert!(
        loose.is_empty(),
        "unsafe SECURITY DEFINER functions: {loose:?}"
    );
    Ok(())
}
