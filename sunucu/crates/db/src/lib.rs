//! PostgreSQL access for the server crates (sqlx).
//!
//! - [`database_url`] / [`connect`] / [`connect_from_env`]: connection pool setup.
//! - [`migrate`]: applies `sunucu/migrations/` (embedded at compile time).
//! - [`set_workspace_id`]: request context for row level security (ADR-0007).

mod context;
mod error;
mod pool;

pub use context::set_workspace_id;
pub use error::{Error, Result};
pub use pool::{
    DEFAULT_MAX_CONNECTIONS, ENV_DATABASE_URL, ENV_DATABASE_URL_FILE, connect, connect_from_env,
    database_url, database_url_from_lookup,
};
pub use sqlx::PgPool;

/// The migrations embedded from `sunucu/migrations/`.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations");

/// Applies all pending migrations. Safe to run concurrently: sqlx takes an advisory lock.
pub async fn migrate(pool: &PgPool) -> Result<()> {
    MIGRATOR.run(pool).await?;
    Ok(())
}
