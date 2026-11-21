//! Permanent account deletion (T2.31, ADR-0007 rule 6.2).
//!
//! A user asks for the deletion (`POST /v1/me/deletion`). After the retention time
//! ([`db::account::RETENTION_DAYS`]) this job deletes the account for good, in this order:
//!
//! 1. Revoke the Apple refresh token at Apple (trait [`account::AppleClient`]).
//! 2. Collect the outside records with `account_collect()`: GitHub Issues of the user and the
//!    object store files of the data that goes.
//! 3. Scrub the Issues and delete the files. A missing client for a record that exists is an
//!    error: the job stops and tries again later, so no outside copy stays by accident.
//! 4. Run `account_purge()` (SECURITY DEFINER, one transaction): deletes identities, passkeys,
//!    sessions and push targets, blanks the `users` row (tombstone), deletes Home workspaces
//!    where the user is the only administrator, deletes the support requests of the user, and
//!    closes the membership in the other workspaces. The audit log is not changed: it names
//!    the user by UUID only, so the chains stay valid.
//! 5. Write one audit row in each workspace that stays (`account.delete`, state `purged`).

use std::{sync::Arc, time::Duration};

use account::{AppleClient, Envelope};
use db::{ActorKind, AuditEvent, DbContext, NewAuditEntry};
use serde_json::json;
use sqlx::PgPool;
use storage::ObjectStore;
use support::github::IssueScrubber;
use tokio::sync::watch;
use uuid::Uuid;

use crate::{Error, Result};

const POLL: Duration = Duration::from_mins(5);
const BATCH: i32 = 20;

/// What the job needs from the outside. A missing part fails only the users that need it.
#[derive(Clone, Default)]
pub struct Deps {
    /// Object store of the files.
    pub store: Option<Arc<dyn ObjectStore>>,
    /// Sign in with Apple client.
    pub apple: Option<Arc<dyn AppleClient>>,
    /// Key that opens the stored Apple token.
    pub envelope: Option<Arc<Envelope>>,
    /// GitHub Issue client.
    pub issues: Option<Arc<dyn IssueScrubber>>,
}

impl std::fmt::Debug for Deps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Deps")
            .field("store", &self.store.is_some())
            .field("apple", &self.apple.is_some())
            .field("envelope", &self.envelope.is_some())
            .field("issues", &self.issues.is_some())
            .finish()
    }
}

fn dberr(e: sqlx::Error) -> Error {
    Error::from(db::Error::from(e))
}

/// Revokes the stored Apple refresh tokens of `user`.
async fn revoke_apple(pool: &PgPool, deps: &Deps, user: Uuid) -> Result<usize> {
    let tokens: Vec<(Uuid, Vec<u8>)> =
        sqlx::query_as("SELECT identity_id, token_enc FROM account_apple_tokens($1)")
            .bind(user)
            .fetch_all(pool)
            .await
            .map_err(dberr)?;
    if tokens.is_empty() {
        return Ok(0);
    }
    let (Some(apple), Some(envelope)) = (&deps.apple, &deps.envelope) else {
        return Err(Error::Format(
            "an Apple token exists but the Apple client or the account key is not configured"
                .into(),
        ));
    };
    for (identity, blob) in &tokens {
        let plain = envelope
            .open(blob)
            .map_err(|e| Error::Format(e.to_string()))?;
        let token = String::from_utf8(plain).map_err(|e| Error::Format(e.to_string()))?;
        apple
            .revoke(&token)
            .await
            .map_err(|e| Error::Format(e.to_string()))?;
        sqlx::query("SELECT account_apple_token_cleared($1)")
            .bind(identity)
            .execute(pool)
            .await
            .map_err(dberr)?;
    }
    Ok(tokens.len())
}

#[derive(sqlx::FromRow)]
struct Collected {
    kind: String,
    workspace_id: Uuid,
    issue_number: Option<i32>,
    object_key: Option<String>,
}

/// What `purge_user` did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Purged {
    /// Apple tokens revoked.
    pub apple_tokens: usize,
    /// Issues scrubbed.
    pub issues: usize,
    /// Files deleted.
    pub objects: usize,
    /// Home workspaces deleted with all content.
    pub home_workspaces: usize,
}

/// Deletes the account of `user` for good. The request must be due.
pub async fn purge_user(pool: &PgPool, deps: &Deps, user: Uuid) -> Result<Purged> {
    let mut done = Purged {
        apple_tokens: revoke_apple(pool, deps, user).await?,
        ..Purged::default()
    };
    let rows: Vec<Collected> = sqlx::query_as(
        "SELECT kind, workspace_id, issue_number, object_key FROM account_collect($1)",
    )
    .bind(user)
    .fetch_all(pool)
    .await
    .map_err(dberr)?;
    let homes: Vec<Uuid> = rows
        .iter()
        .filter(|r| r.kind == "home_workspace")
        .map(|r| r.workspace_id)
        .collect();
    let issues: Vec<i32> = rows.iter().filter_map(|r| r.issue_number).collect();
    let keys: Vec<&str> = rows
        .iter()
        .filter_map(|r| r.object_key.as_deref())
        .collect();
    if !issues.is_empty() {
        let Some(client) = &deps.issues else {
            return Err(Error::Format(
                "support Issues exist but no GitHub client is configured".into(),
            ));
        };
        for n in &issues {
            client
                .scrub(i64::from(*n))
                .await
                .map_err(|e| Error::Format(e.to_string()))?;
        }
        done.issues = issues.len();
    }
    if !keys.is_empty() {
        let Some(store) = &deps.store else {
            return Err(Error::Format(
                "files exist but no object store is configured".into(),
            ));
        };
        for key in &keys {
            store.delete(key).await?;
        }
        done.objects = keys.len();
    }
    // Workspaces that stay: they get an audit row after the purge.
    let staying: Vec<Uuid> = sqlx::query_as::<_, (Uuid,)>(
        "SELECT workspace_id FROM account_export_workspaces($1) \
         WHERE NOT (mode = 'home' AND sole_admin)",
    )
    .bind(user)
    .fetch_all(pool)
    .await
    .map_err(dberr)?
    .into_iter()
    .map(|r| r.0)
    .collect();
    sqlx::query("SELECT account_purge($1, $2)")
        .bind(user)
        .bind(&homes)
        .execute(pool)
        .await
        .map_err(dberr)?;
    done.home_workspaces = homes.len();
    for ws in staying {
        if let Err(e) = audit_purged(pool, ws, user).await {
            tracing::warn!(workspace = %ws, error = %e, "account deletion audit row was not written");
        }
    }
    Ok(done)
}

/// One audit row that names the user by UUID only.
async fn audit_purged(pool: &PgPool, ws: Uuid, user: Uuid) -> Result<()> {
    let ctx = DbContext {
        user_id: None,
        workspace_id: Some(ws),
        actor_kind: ActorKind::System,
    };
    let mut tx = db::begin(pool, &ctx).await?;
    db::append_audit(
        &mut tx,
        ws,
        &NewAuditEntry {
            actor_kind: ActorKind::System,
            actor_id: None,
            event: AuditEvent::AccountDelete,
            record_kind: "user".to_owned(),
            record_id: Some(user),
            previous_value: None,
            new_value: Some(json!({"state": "purged"})),
            reason: None,
        },
    )
    .await?;
    tx.commit().await.map_err(dberr)?;
    Ok(())
}

/// Purges every account whose waiting time is over. Returns the number of accounts deleted.
pub async fn purge_due(pool: &PgPool, deps: &Deps) -> Result<usize> {
    let due: Vec<(Uuid,)> = sqlx::query_as("SELECT user_id FROM account_deletions_due($1)")
        .bind(BATCH)
        .fetch_all(pool)
        .await
        .map_err(dberr)?;
    let mut count = 0;
    for (user,) in due {
        match purge_user(pool, deps, user).await {
            Ok(done) => {
                count += 1;
                tracing::info!(user_id = %user, ?done, "account deleted for good");
            }
            Err(e) => {
                tracing::warn!(user_id = %user, error = %e, "account deletion failed; will retry");
                let _ = sqlx::query("SELECT account_deletion_failed($1, $2)")
                    .bind(user)
                    .bind(e.to_string())
                    .execute(pool)
                    .await;
            }
        }
    }
    Ok(count)
}

/// Runs the job until `stop` turns `true`.
pub async fn run(pool: PgPool, deps: Deps, mut stop: watch::Receiver<bool>) {
    loop {
        if let Err(e) = purge_due(&pool, &deps).await {
            tracing::warn!(error = %e, "account deletion pass failed");
        }
        tokio::select! {
            _ = stop.wait_for(|stopped| *stopped) => return,
            () = tokio::time::sleep(POLL) => {}
        }
    }
}
