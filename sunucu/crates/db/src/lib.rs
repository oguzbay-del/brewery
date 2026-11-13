//! PostgreSQL access for the server crates (sqlx).
//!
//! - [`database_url`] / [`connect`] / [`connect_from_env`]: connection pool setup.
//! - [`migrate`]: applies `sunucu/migrations/` (embedded at compile time).
//! - [`provision_role`]: login and password for the application roles (after migrating).
//! - [`DbContext`], [`begin`], [`set_context`]: request context for row level security
//!   (ADR-0007).

mod context;
mod error;
mod pool;
mod roles;

pub use context::{
    ActorKind, DbContext, begin, set_actor_kind, set_context, set_user_id, set_workspace_id,
};
pub use error::{Error, Result};
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
