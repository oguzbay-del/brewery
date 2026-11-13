//! Request context for row level security (ADR-0007, rule 3).

use sqlx::{PgConnection, types::Uuid};

use crate::Result;

/// Sets `app.workspace_id` for the current transaction (`SET LOCAL` semantics).
///
/// RLS policies read the value through `current_setting('app.workspace_id', true)`. It must be
/// called on a connection inside a transaction: outside one, the setting is dropped at the end
/// of the statement. `set_config(.., true)` is used because `SET LOCAL` cannot take bind
/// parameters.
///
/// The full context (`app.user_id`, `app.actor_kind`) arrives with T2.10.
pub async fn set_workspace_id(tx: &mut PgConnection, workspace_id: Uuid) -> Result<()> {
    sqlx::query("SELECT set_config('app.workspace_id', $1, true)")
        .bind(workspace_id.to_string())
        .execute(tx)
        .await?;
    Ok(())
}
