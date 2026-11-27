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
    /// A support staff member who works outside a workspace (panel search). The transaction is
    /// read-only and `app.staff_id` names the staff user (see [`begin_staff`]).
    Staff,
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
            Self::Staff => "staff",
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

/// Sets the audit channel of the current transaction: `via` (`app`, `mcp`, `api`) and, for an
/// OAuth call, the client row, the grant and the scope of the call. [`crate::append_audit`] copies
/// them into each audit row of the transaction (ADR-0015 rule 5.2).
pub async fn set_audit_channel(
    tx: &mut PgConnection,
    via: &str,
    oauth_client_id: Option<Uuid>,
    oauth_grant_id: Option<Uuid>,
    oauth_scope: Option<&str>,
) -> Result<()> {
    set_local(tx, "app.audit_via", via).await?;
    set_local(
        tx,
        "app.oauth_client_id",
        &oauth_client_id.map(|u| u.to_string()).unwrap_or_default(),
    )
    .await?;
    set_local(
        tx,
        "app.oauth_grant_id",
        &oauth_grant_id.map(|u| u.to_string()).unwrap_or_default(),
    )
    .await?;
    set_local(tx, "app.oauth_scope", oauth_scope.unwrap_or_default()).await
}

/// Sets `app.staff_id` for the current transaction. The ID must come from the authenticated
/// request context, never from a caller parameter. `support_find_workspace` reads it.
pub async fn set_staff_id(tx: &mut PgConnection, staff_id: Uuid) -> Result<()> {
    set_local(tx, "app.staff_id", &staff_id.to_string()).await
}

/// Starts a read-only transaction for a staff member (`app.actor_kind = 'staff'`,
/// `app.staff_id`). No user or workspace is set, so no workspace row matches an RLS policy.
pub async fn begin_staff(pool: &PgPool, staff_id: Uuid) -> Result<Transaction<'static, Postgres>> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION READ ONLY")
        .execute(&mut *tx)
        .await?;
    set_actor_kind(&mut tx, ActorKind::Staff).await?;
    set_staff_id(&mut tx, staff_id).await?;
    Ok(tx)
}

/// Applies the whole [`DbContext`] to the current transaction. A support or staff actor also
/// makes the transaction read-only (ADR-0007 rule 2.3).
pub async fn set_context(tx: &mut PgConnection, ctx: &DbContext) -> Result<()> {
    if matches!(ctx.actor_kind, ActorKind::Support | ActorKind::Staff) {
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

tokio::task_local! {
    /// The `idempotency_keys` row of the request that runs in this task. The idempotency layer
    /// scopes the handler future with it; [`begin`] then marks the row in every transaction the
    /// handler opens, so the mark commits together with the business work (F2.1 finding 4).
    pub static IDEMPOTENCY_KEY: Uuid;
}

/// Starts a transaction with the request context applied; the API runs every request in one.
///
/// Inside an idempotent request (see [`IDEMPOTENCY_KEY`]) the transaction also sets
/// `idempotency_keys.work_committed_at`. If the process dies after the commit but before the
/// stored response, a retry sees the mark and does not run the work again.
pub async fn begin(pool: &PgPool, ctx: &DbContext) -> Result<Transaction<'static, Postgres>> {
    let mut tx = pool.begin().await?;
    set_context(&mut tx, ctx).await?;
    if !matches!(ctx.actor_kind, ActorKind::Support | ActorKind::Staff)
        && let Ok(key) = IDEMPOTENCY_KEY.try_with(|k| *k)
    {
        sqlx::query(
            "UPDATE idempotency_keys SET work_committed_at = now() \
             WHERE id = $1 AND work_committed_at IS NULL",
        )
        .bind(key)
        .execute(&mut *tx)
        .await?;
    }
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
