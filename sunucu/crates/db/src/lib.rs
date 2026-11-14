//! PostgreSQL access for the server crates (sqlx).
//!
//! - [`database_url`] / [`connect`] / [`connect_from_env`]: connection pool setup.
//! - [`migrate`]: applies `sunucu/migrations/` (embedded at compile time).
//! - [`provision_role`]: login and password for the application roles (after migrating).
//! - [`append_audit`], [`verify_chain`], [`verify_all_chains`]: tamper-evident audit log
//!   (hash chain per workspace, ADR-0007 rule 5).
//! - [`outbox::enqueue`]: transactional outbox writes (events for SSE and consumers).
//! - [`DbContext`], [`begin`], [`set_context`]: request context for row level security
//!   (ADR-0007).

mod audit;
mod context;
mod error;
mod measurement;
pub mod outbox;
mod pool;
mod roles;

pub use audit::{
    AuditEvent, AuditRecord, CanonicalFields, ChainBreak, ChainReport, GENESIS_HASH, NewAuditEntry,
    append_audit, canonical_row, chain_hash, changed_field, verify_all_chains, verify_chain,
};
pub use context::{
    ActorKind, DbContext, IDEMPOTENCY_KEY, begin, set_actor_kind, set_context, set_user_id,
    set_workspace_id,
};
pub use error::{Error, Result};
pub use measurement::{
    MAX_FUTURE as MAX_MEASURED_AT_FUTURE, MeasuredAtError, earliest_measured_at,
    validate_measured_at,
};
pub use pool::{
    DEFAULT_MAX_CONNECTIONS, ENV_DATABASE_URL, ENV_DATABASE_URL_FILE, ENV_MIGRATE_DATABASE_URL,
    ENV_MIGRATE_DATABASE_URL_FILE, connect, connect_from_env, connect_lazy, database_url,
    database_url_from_lookup, migrate_database_url, migrate_database_url_from_lookup,
};
pub use roles::{
    AppRole, password_from_env, password_from_lookup, password_variable, provision_role,
};
pub use sqlx::PgPool;

/// The migrations embedded from `sunucu/migrations/`.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations");

/// Applies all pending migrations. Safe to run concurrently: sqlx takes an advisory lock.
pub async fn migrate(pool: &PgPool) -> Result<()> {
    MIGRATOR.run(pool).await?;
    Ok(())
}
