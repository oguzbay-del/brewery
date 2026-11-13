//! Request context for row level security (ADR-0007, rule 3).

use sqlx::{PgConnection, PgPool, Postgres, Transaction, types::Uuid};

use crate::Result;

/// Who acts, as stored in `app.actor_kind` (ADR-0007 rule 2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorKind {
    /// A signed-in customer user.
    User,
    /// A support staff member with a valid grant (read-only transactions).
    Support,
    /// The system itself (jobs, ingest).
    System,
    /// A paired shared device acting for a member.
    Device,
}

impl ActorKind {
    /// The value written to `app.actor_kind`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Support => "support",
            Self::System => "system",
            Self::Device => "device",
        }
    }
}

/// The full per-request database context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DbContext {
    /// `app.user_id`; `None` leaves it unset (no account row matches).
    pub user_id: Option<Uuid>,
    /// `app.workspace_id`; `None` leaves it unset (no workspace row matches).
    pub workspace_id: Option<Uuid>,
    /// `app.actor_kind`.
    pub actor_kind: ActorKind,
}

impl DbContext {
    /// Context of a signed-in user, optionally inside a workspace.
    #[must_use]
    pub const fn user(user_id: Uuid, workspace_id: Option<Uuid>) -> Self {
        Self {
            user_id: Some(user_id),
            workspace_id,
            actor_kind: ActorKind::User,
        }
    }
}

/// Sets `app.workspace_id` for the current transaction (`SET LOCAL` semantics).
///
/// RLS policies read the value through `current_setting('app.workspace_id', true)`. It must be
/// called on a connection inside a transaction: outside one, the setting is dropped at the end
/// of the statement. `set_config(.., true)` is used because `SET LOCAL` cannot take bind
/// parameters.
pub async fn set_workspace_id(tx: &mut PgConnection, workspace_id: Uuid) -> Result<()> {
    set_local(tx, "app.workspace_id", &workspace_id.to_string()).await
}

/// Sets `app.user_id` for the current transaction.
pub async fn set_user_id(tx: &mut PgConnection, user_id: Uuid) -> Result<()> {
    set_local(tx, "app.user_id", &user_id.to_string()).await
}

/// Sets `app.actor_kind` for the current transaction.
pub async fn set_actor_kind(tx: &mut PgConnection, kind: ActorKind) -> Result<()> {
    set_local(tx, "app.actor_kind", kind.as_str()).await
}

/// Applies the whole [`DbContext`] to the current transaction. A support actor also makes the
/// transaction read-only (ADR-0007 rule 2.3).
pub async fn set_context(tx: &mut PgConnection, ctx: &DbContext) -> Result<()> {
    if ctx.actor_kind == ActorKind::Support {
        sqlx::query("SET TRANSACTION READ ONLY")
            .execute(&mut *tx)
            .await?;
    }
    set_actor_kind(tx, ctx.actor_kind).await?;
    if let Some(user_id) = ctx.user_id {
        set_user_id(tx, user_id).await?;
    }
    if let Some(workspace_id) = ctx.workspace_id {
        set_workspace_id(tx, workspace_id).await?;
    }
    Ok(())
}

/// Starts a transaction with the request context applied; the API runs every request in one.
pub async fn begin(pool: &PgPool, ctx: &DbContext) -> Result<Transaction<'static, Postgres>> {
    let mut tx = pool.begin().await?;
    set_context(&mut tx, ctx).await?;
    Ok(tx)
}

async fn set_local(tx: &mut PgConnection, name: &str, value: &str) -> Result<()> {
    sqlx::query("SELECT set_config($1, $2, true)")
        .bind(name)
        .bind(value)
        .execute(tx)
        .await?;
    Ok(())
}
