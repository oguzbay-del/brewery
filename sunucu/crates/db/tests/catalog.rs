//! Catalog scan (ADR-0007, rule 2.8): isolation must be enforced for every table, not only
//! for the ones the other tests happen to touch. T2.6 and T2.7 add tables and are covered by
//! the same scan without changes here.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use sqlx::PgPool;

/// Cross-workspace tables (ADR-0007, rule 2.5): protected by privileges instead of RLS.
const CROSS_WORKSPACE: &[&str] = &["outbox", "scheduled_jobs", "external_events"];
/// Shared tables written only by migrations and seeds; no RLS, read-only for the API.
const SHARED: &[&str] = &["staff_users"];
/// Tables whose `app_owner` policies serve the pre-login SECURITY DEFINER functions.
const OWNER_LOOKUP: &[&str] = &[
    "user_identities",
    "sessions",
    "email_login_tokens",
    "auth_requests",
    "shared_devices",
    "invitations",
    "idempotency_keys",
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

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn workspace_references_use_composite_foreign_keys(pool: PgPool) -> sqlx::Result<()> {
    // A foreign key between two tables that both carry `workspace_id` must include it:
    // referential checks bypass RLS, so a single-column key would let one workspace point at
    // another's row.
    let offenders = names(
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
             ORDER BY 1",
        )
        .fetch_all(&pool)
        .await?,
    );
    assert!(
        offenders.is_empty(),
        "single-column references: {offenders:?}"
    );
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
        ("external_events", "app_api", "SELECT", true),
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
    let mut expected: Vec<String> = OWNER_LOOKUP.iter().map(|s| (*s).to_owned()).collect();
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
