//! Worker maintenance primitives (T2.39): audit chain heads across workspaces, job claims,
//! and the retention of external events (F2.1 decisions 11 and 12).
//!
//! Only `app_worker` calls these. Cross-workspace reads go through the narrow SECURITY DEFINER
//! functions of migration 0160. The queries are plain `sqlx::query` calls (no query cache).

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row, types::Uuid};

use crate::{
    Error, Result,
    outbox::{Entity, NewOutboxEvent, enqueue},
    set_workspace_id,
};

/// Head of one workspace's audit chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditHead {
    /// The workspace.
    pub workspace_id: Uuid,
    /// Sequence number of the last row (0 when the chain has no row).
    pub last_seq: i64,
    /// Hash of the last row.
    pub last_hash: [u8; 32],
}

/// Deletion record of a purged workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tombstone {
    /// The deleted workspace.
    pub former_workspace_id: Uuid,
    /// When it was purged.
    pub deleted_at: DateTime<Utc>,
    /// Sequence number of the last audit row at purge time.
    pub audit_last_seq: i64,
    /// Hash of the last audit row at purge time.
    pub audit_last_hash: [u8; 32],
}

fn to_hash(bytes: Vec<u8>) -> Result<[u8; 32]> {
    bytes
        .try_into()
        .map_err(|_| Error::AuditCanonical("hash is not 32 bytes".into()))
}

/// The head of every audit chain, across workspaces.
pub async fn list_audit_heads(pool: &PgPool) -> Result<Vec<AuditHead>> {
    let rows = sqlx::query("SELECT workspace_id, last_seq, last_hash FROM list_audit_heads()")
        .fetch_all(pool)
        .await?;
    rows.iter()
        .map(|r| {
            Ok(AuditHead {
                workspace_id: r.try_get("workspace_id")?,
                last_seq: r.try_get("last_seq")?,
                last_hash: to_hash(r.try_get("last_hash")?)?,
            })
        })
        .collect()
}

/// Every workspace id, soft-deleted ones included.
pub async fn list_workspace_ids(pool: &PgPool) -> Result<Vec<Uuid>> {
    Ok(sqlx::query_scalar("SELECT id FROM list_workspace_ids()")
        .fetch_all(pool)
        .await?)
}

/// Every workspace deletion record.
pub async fn list_tombstones(pool: &PgPool) -> Result<Vec<Tombstone>> {
    let rows = sqlx::query(
        "SELECT former_workspace_id, deleted_at, audit_last_seq, audit_last_hash
         FROM list_workspace_tombstones()",
    )
    .fetch_all(pool)
    .await?;
    rows.iter()
        .map(|r| {
            Ok(Tombstone {
                former_workspace_id: r.try_get("former_workspace_id")?,
                deleted_at: r.try_get("deleted_at")?,
                audit_last_seq: r.try_get("audit_last_seq")?,
                audit_last_hash: to_hash(r.try_get("audit_last_hash")?)?,
            })
        })
        .collect()
}

/// The stored hash of audit row `seq` of a workspace; `None` when the row does not exist.
pub async fn audit_hash_at(
    pool: &PgPool,
    workspace_id: Uuid,
    seq: i64,
) -> Result<Option<[u8; 32]>> {
    let mut tx = pool.begin().await?;
    set_workspace_id(&mut tx, workspace_id).await?;
    let hash: Option<Vec<u8>> =
        sqlx::query_scalar("SELECT hash FROM audit_log WHERE workspace_id = $1 AND seq = $2")
            .bind(workspace_id)
            .bind(seq)
            .fetch_optional(&mut *tx)
            .await?;
    tx.rollback().await?;
    hash.map(to_hash).transpose()
}

/// Final state of a maintenance run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    /// The job finished and found nothing wrong.
    Ok,
    /// The job finished and reports findings (for example a broken chain).
    Findings,
    /// The job could not finish; the next tick tries again.
    Error,
}

impl RunStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Findings => "findings",
            Self::Error => "error",
        }
    }
}

/// Claims a job: `true` when the last start is older than `min_interval` (or the job never
/// ran) and this caller now owns the run. Replicas race on one atomic statement.
pub async fn claim_job(
    pool: &PgPool,
    job: &str,
    now: DateTime<Utc>,
    min_interval: std::time::Duration,
) -> Result<bool> {
    let claimed = sqlx::query(
        "INSERT INTO maintenance_runs (job, last_started_at, last_status) VALUES ($1, $2, 'running')
         ON CONFLICT (job) DO UPDATE
            SET last_started_at = EXCLUDED.last_started_at, last_finished_at = NULL,
                last_status = 'running'
            WHERE maintenance_runs.last_started_at <= $2 - make_interval(secs => $3)
         RETURNING job",
    )
    .bind(job)
    .bind(now)
    .bind(min_interval.as_secs_f64())
    .fetch_optional(pool)
    .await?;
    Ok(claimed.is_some())
}

/// Records the result of a claimed run. After [`RunStatus::Error`] the claim is released, so
/// the next tick retries.
pub async fn finish_job(
    pool: &PgPool,
    job: &str,
    now: DateTime<Utc>,
    status: RunStatus,
    detail: &str,
) -> Result<()> {
    sqlx::query(
        "UPDATE maintenance_runs
         SET last_finished_at = $2, last_status = $3, last_detail = $4,
             last_started_at = CASE WHEN $3 = 'error' THEN to_timestamp(0) ELSE last_started_at END
         WHERE job = $1",
    )
    .bind(job)
    .bind(now)
    .bind(status.as_str())
    .bind(detail)
    .execute(pool)
    .await?;
    Ok(())
}

/// Clears the payload of external events that nobody processed and that arrived before
/// `cutoff` (decision 11: 30 days). Returns the number of rows cleared; a second run clears
/// nothing more.
pub async fn clear_stale_external_payloads(
    pool: &PgPool,
    cutoff: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<u64> {
    let done = sqlx::query(
        "UPDATE external_events SET payload = '{}'::jsonb, payload_cleared_at = $2
         WHERE processed_at IS NULL AND payload_cleared_at IS NULL AND received_at < $1",
    )
    .bind(cutoff)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(done.rows_affected())
}

/// Deletes external events (and with them their delivery IDs) received before `cutoff`
/// (decision 11: 13 months). Returns the number of rows deleted.
pub async fn delete_old_external_events(pool: &PgPool, cutoff: DateTime<Utc>) -> Result<u64> {
    let done = sqlx::query("DELETE FROM external_events WHERE received_at < $1")
        .bind(cutoff)
        .execute(pool)
        .await?;
    Ok(done.rows_affected())
}

/// Closes resolved support requests older than `cutoff` in every workspace and enqueues the
/// normal support sync event in the same transaction. Returns the number of requests closed.
/// The SECURITY DEFINER function exposes only changed row identities and versions to the worker.
pub async fn close_resolved_support_requests(pool: &PgPool, cutoff: DateTime<Utc>) -> Result<u64> {
    let mut tx = pool.begin().await?;
    let rows: Vec<(Uuid, Uuid, i64)> =
        sqlx::query_as("SELECT workspace_id, id, version FROM close_resolved_support_requests($1)")
            .bind(cutoff)
            .fetch_all(&mut *tx)
            .await?;
    for (workspace_id, request_id, version) in &rows {
        enqueue(
            &mut tx,
            &NewOutboxEvent {
                topic: "support.sync_requested".to_owned(),
                workspace_id: Some(*workspace_id),
                entity: Some(Entity {
                    kind: "support_request".to_owned(),
                    id: *request_id,
                    version: *version,
                }),
                data: None,
            },
        )
        .await?;
    }
    tx.commit().await?;
    Ok(rows.iter().fold(0_u64, |count, _| count.saturating_add(1)))
}

/// Result of [`purge_expired_soft_deletes`] for one kind of record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurgeCount {
    /// `batch`, `recipe`, `equipment` or `ingredient`.
    pub kind: String,
    /// Records deleted for good.
    pub purged: i64,
    /// Expired records that stay soft-deleted because something still uses them
    /// (F2.1 decision 9).
    pub kept: i64,
}

/// Deletes for good the batches, recipes, equipment and user ingredients of every workspace
/// that were soft-deleted before `cutoff` (T2.37: 30 days). A record that is in use stays and
/// counts as `kept`. Running it again deletes nothing more. Returns the totals by kind.
pub async fn purge_expired_soft_deletes(
    pool: &PgPool,
    cutoff: DateTime<Utc>,
) -> Result<Vec<PurgeCount>> {
    let mut totals: Vec<PurgeCount> = Vec::new();
    for workspace_id in list_workspace_ids(pool).await? {
        let mut tx = pool.begin().await?;
        set_workspace_id(&mut tx, workspace_id).await?;
        let rows = sqlx::query("SELECT kind, purged, kept FROM purge_expired_soft_deletes($1)")
            .bind(cutoff)
            .fetch_all(&mut *tx)
            .await?;
        tx.commit().await?;
        for r in &rows {
            let kind: String = r.try_get("kind")?;
            let (purged, kept): (i64, i64) = (r.try_get("purged")?, r.try_get("kept")?);
            match totals.iter_mut().find(|t| t.kind == kind) {
                Some(t) => {
                    t.purged += purged;
                    t.kept += kept;
                }
                None => totals.push(PurgeCount { kind, purged, kept }),
            }
        }
    }
    Ok(totals)
}

/// One upload whose object bytes can be removed by the worker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurgeableUpload {
    /// `attachment` or `scanned_document`.
    pub kind: String,
    /// Upload record identifier.
    pub id: Uuid,
    /// Server-generated object-store key.
    pub object_key: String,
    /// True when the upload never reached its complete step.
    pub incomplete: bool,
    /// True when the row already has a deletion timestamp.
    pub deleted: bool,
}

/// Lists at most `limit` purgeable uploads in one workspace.
pub async fn list_purgeable_uploads(
    pool: &PgPool,
    workspace_id: Uuid,
    incomplete_before: DateTime<Utc>,
    limit: i32,
) -> Result<Vec<PurgeableUpload>> {
    let mut tx = pool.begin().await?;
    set_workspace_id(&mut tx, workspace_id).await?;
    let rows = sqlx::query(
        "SELECT p.kind, p.id, p.object_key,
                CASE WHEN p.kind = 'attachment' THEN a.uploaded_at IS NULL
                     ELSE d.uploaded_at IS NULL END AS incomplete,
                CASE WHEN p.kind = 'attachment' THEN a.deleted_at IS NOT NULL
                     ELSE d.deleted_at IS NOT NULL END AS deleted
         FROM list_purgeable_uploads($1, $2) AS p
         LEFT JOIN attachments AS a ON p.kind = 'attachment' AND a.id = p.id
         LEFT JOIN scanned_documents AS d ON p.kind = 'scanned_document' AND d.id = p.id",
    )
    .bind(incomplete_before)
    .bind(limit)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    rows.iter()
        .map(|row| {
            Ok(PurgeableUpload {
                kind: row.try_get("kind")?,
                id: row.try_get("id")?,
                object_key: row.try_get("object_key")?,
                incomplete: row.try_get("incomplete")?,
                deleted: row.try_get("deleted")?,
            })
        })
        .collect()
}

/// Closes a stale upload before deletion. A concurrent completion wins if it commits first.
pub async fn close_incomplete_upload(
    pool: &PgPool,
    workspace_id: Uuid,
    kind: &str,
    id: Uuid,
) -> Result<bool> {
    let statement = match kind {
        "attachment" => {
            "UPDATE attachments SET deleted_at = now()
             WHERE id = $1 AND uploaded_at IS NULL AND deleted_at IS NULL AND object_purged_at IS NULL"
        }
        "scanned_document" => {
            "UPDATE scanned_documents SET deleted_at = now()
             WHERE id = $1 AND uploaded_at IS NULL AND deleted_at IS NULL AND object_purged_at IS NULL
               AND (NOT legal_hold OR (retention_until IS NOT NULL AND retention_until <= now()))"
        }
        _ => return Ok(false),
    };
    let mut tx = pool.begin().await?;
    set_workspace_id(&mut tx, workspace_id).await?;
    let result = sqlx::query(statement).bind(id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(result.rows_affected() == 1)
}

/// Marks one upload after its object bytes have been deleted.
pub async fn mark_upload_purged(
    pool: &PgPool,
    workspace_id: Uuid,
    kind: &str,
    id: Uuid,
) -> Result<bool> {
    let mut tx = pool.begin().await?;
    set_workspace_id(&mut tx, workspace_id).await?;
    let changed: bool = sqlx::query_scalar("SELECT mark_upload_purged($1, $2)")
        .bind(kind)
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(changed)
}
