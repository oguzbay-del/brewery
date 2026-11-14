# db

## What it does
- The only gate to PostgreSQL access: sqlx pool (`DATABASE_URL` or `DATABASE_URL_FILE`), migrations, and request context helpers (`DbContext`, `begin`: `SET LOCAL app.*` for each transaction).
- The migrations are in `sunucu/migrations/`. `migrate()` (or the `migrate` binary) applies them. They are embedded in the image.
- The `migrate` binary runs one time in compose and in Kubernetes. Application services do not run migrations.

## What it does not touch
- Table and column decisions are in `adr/0007-veri-modeli.md`. The Home tables (T2.6) are in migrations `0007` to `0013`. The Business tables (T2.7) come in their own migration.
- Do not write business logic of application services here.

## Roles and migration rules
- Three roles: `app_owner` (table owner, cannot sign in), `app_api` and `app_worker` (under RLS, not owner, no BYPASSRLS). `0002_roller.sql` creates the roles if they do not exist (NOLOGIN). Migrations have no password.
- The `migrate` binary gives sign-in and password after the migration (`APP_API_PASSWORD`, `APP_WORKER_PASSWORD`, or `*_FILE`). A second run renews the password. The migration role connects with `MIGRATE_DATABASE_URL` (or `DATABASE_URL`). It is a superuser, or a CREATEROLE owner and database owner. Application services connect with `DATABASE_URL` as `app_api` (worker: `app_worker`).
- Each table migration starts with `SET LOCAL ROLE app_owner;` and ends with `RESET ROLE;`. The objects belong to `app_owner`. The migration role writes the sqlx record. Each table gets `ENABLE` + `FORCE ROW LEVEL SECURITY`, at least one policy and an explicit `GRANT`. Do not grant rights to `PUBLIC`.
- Policies use the functions `app_workspace_id()` and `app_user_id()` in the form `(SELECT ...)`. The functions treat an empty string as NULL. After `SET LOCAL` ends, the setting returns to an empty string.
- Lookups before sign-in are narrow `SECURITY DEFINER` functions (owner `app_owner`, fixed `search_path`, `PUBLIC` cannot call them). These tables have only an `owner_lookup` policy open to `app_owner`. If you add a new one, update the `OWNER_LOOKUP` list in `tests/catalog.rs`.
- A table with `workspace_id` has a composite foreign key to the parent record (`(workspace_id, parent_id)`). The catalog test enforces this. Cross-workspace tables (`outbox`, `scheduled_jobs`, `external_events`, `processed_events`) have no RLS and rights protect them. If you leave a table that is not in the list without RLS, the test fails.
- The audit chain is in `src/audit.rs` (digest and verification are in Rust, no migration). Add rows only with `append_audit`, in the request transaction. If the row fields change, the `canonical_row` format changes and the old chain breaks (a new format version is necessary). Test: `tests/audit_chain.rs`.
- Only the worker deletes a workspace: `purge_workspace(id)` (SECURITY DEFINER, `app_worker` only). `app_api` has no DELETE on `workspaces` and `users`. The function writes `workspace_tombstones` first. A retention or legal hold trigger stops the purge and rolls the tombstone back.
- Do not add a NO ACTION foreign key from an immutable or ledger row to a record that a job deletes. Use `ON DELETE SET NULL` and a snapshot column filled by `snapshot_references()`.
- `audit_log` is append-only. The application roles have no UPDATE/DELETE/TRUNCATE right. A trigger also rejects these actions for the owner role (except the foreign key chain when a workspace is deleted).

## How to test
- You need a real PostgreSQL: `make up` (or only `docker compose -f altyapi/compose/compose.yaml up -d postgres`). Then run `make test` in the repository root. By hand: `DATABASE_URL=postgres://demleme:demleme-yerel@127.0.0.1:5432/demleme cargo test -p db`.
- `#[sqlx::test]` creates a temporary database for each test, applies the migrations and deletes the database at the end. `pool_as` in `tests/common` tests application behavior: the connections open with `SET ROLE app_api` (or `app_worker`, `app_owner`). After `SET ROLE`, the RLS exemption of the superuser does not apply. A superuser session can use `SET ROLE` to change to any role. For this reason, test claims such as "an application role cannot be the owner" from `pg_roles` or the catalog.
- Tests: `migrations.rs` (migration), `rls.rs` (workspace separation), `catalog.rs` (catalog scan, rights), `audit_log.rs`, `auth_access.rs` (account tables and pre-sign-in functions), `infra.rs` (outbox, repeat safety), `library.rs` (shared library), `recipes.rs` (immutable version), `batches.rs` (batch, measurement partitions, stock, scope trigger, write constraints). `f21.rs` (F2.1 fixes: line race, stock count, workspace purge and tombstone, snapshots, retention, push token).
- Query cache: run `make sqlx-prepare` in the repository root. Commit `sunucu/.sqlx/`. The build runs with `SQLX_OFFLINE=true`.

- A new table migration calls `secure_workspace_table(table, mode, rights)` (0007). A table that holds shared rows uses the `shared` mode. It needs a `shared_ref_<column>` trigger for a reference to a shared record. Add the table to the `OWNER_SEED` list in `tests/catalog.rs`. Each partition of a partitioned table (for example measurements) also has its own RLS and policy.

## Rules
- Migration files are `NNNN_name.sql` (four-digit sequence, lowercase, underscore). Do not change an applied migration. Add a new migration.
- Code is in English. Notes for Claude (this file, task lists) use ASD-STE100. Other documents are in Turkish.
- Start the commit message with the task number: `T2.x: ...`.
