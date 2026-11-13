//! Shared helpers for the database tests. `#[sqlx::test]` hands every test a throwaway
//! database and a pool connected as the owner of `DATABASE_URL` (a superuser locally). The
//! application is simulated by a second pool whose connections run `SET ROLE app_api` (or
//! `app_worker`): after `SET ROLE` PostgreSQL checks privileges and row level security
//! against that role, so the superuser's own bypass does not apply.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(dead_code)] // each test binary uses a subset

use sqlx::{PgConnection, PgPool, postgres::PgPoolOptions, types::Uuid};

/// Roles a test can act as.
#[derive(Clone, Copy)]
pub enum Role {
    Api,
    Worker,
    Owner,
}

impl Role {
    fn set_role_sql(self) -> &'static str {
        match self {
            Self::Api => "SET ROLE app_api",
            Self::Worker => "SET ROLE app_worker",
            Self::Owner => "SET ROLE app_owner",
        }
    }
}

/// A pool whose every connection acts as `role`.
pub async fn pool_as(base: &PgPool, role: Role) -> PgPool {
    PgPoolOptions::new()
        .max_connections(4)
        .after_connect(move |conn, _| {
            Box::pin(async move {
                sqlx::query(role.set_role_sql()).execute(conn).await?;
                Ok(())
            })
        })
        .connect_with((*base.connect_options()).clone())
        .await
        .expect("connect as application role")
}

/// Deterministic UUID that looks like a v7 one.
pub fn id(n: u128) -> Uuid {
    Uuid::from_u128(0x0192_f1c0_7a3b_7000_8000_0000_0000_0000 + n)
}

/// Sets `app.workspace_id` and/or `app.user_id` for the transaction.
pub async fn set_context(
    conn: &mut PgConnection,
    workspace: Option<Uuid>,
    user: Option<Uuid>,
) -> sqlx::Result<()> {
    if let Some(ws) = workspace {
        db::set_workspace_id(conn, ws).await.expect("set workspace");
    }
    if let Some(u) = user {
        sqlx::query("SELECT set_config('app.user_id', $1, true)")
            .bind(u.to_string())
            .execute(conn)
            .await?;
    }
    Ok(())
}

/// SQLSTATE of a database error, `None` for other errors.
pub fn sqlstate(error: &sqlx::Error) -> Option<String> {
    let code = error.as_database_error()?.code()?;
    Some(code.into_owned())
}

/// Inserts a workspace as the application (context = the new workspace).
pub async fn create_workspace(app: &PgPool, ws: Uuid, name: &str) -> sqlx::Result<()> {
    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(ws), None).await?;
    sqlx::query(
        "INSERT INTO workspaces (id, name, mode, region, country, time_zone, support_pseudonym)
         VALUES ($1, $2, 'business', 'us', 'US', 'America/New_York', $3)",
    )
    .bind(ws)
    .bind(name)
    .bind(format!("pseudonym-{ws}"))
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

/// Inserts a user as the application (context = the new user).
pub async fn create_user(app: &PgPool, user: Uuid) -> sqlx::Result<()> {
    let mut tx = app.begin().await?;
    set_context(&mut tx, None, Some(user)).await?;
    sqlx::query("INSERT INTO users (id, display_name) VALUES ($1, 'Test')")
        .bind(user)
        .execute(&mut *tx)
        .await?;
    tx.commit().await
}

/// Inserts a role into a workspace as the application.
pub async fn create_role(app: &PgPool, ws: Uuid, role: Uuid, name: &str) -> sqlx::Result<()> {
    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(ws), None).await?;
    sqlx::query("INSERT INTO roles (id, workspace_id, name) VALUES ($1, $2, $3)")
        .bind(role)
        .bind(ws)
        .bind(name)
        .execute(&mut *tx)
        .await?;
    tx.commit().await
}
