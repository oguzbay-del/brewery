//! Outbox -> NATS `JetStream` relay (runs in the worker as `app_worker`).
//!
//! One pass ([`run_once`]) opens a transaction, takes a transaction-level advisory lock so that
//! only one relay publishes at a time (keeps the order when several workers run), selects the
//! oldest unpublished rows `FOR UPDATE SKIP LOCKED`, publishes them in order with the outbox id as
//! `Nats-Msg-Id` and waits for the `JetStream` acknowledgements. Only the acknowledged prefix is
//! marked `published_at`; the transaction then commits. A crash between publish and commit
//! publishes the rows again: delivery is at least once, and `JetStream` drops the repeat inside its
//! duplicate window (consumers also de-duplicate by event id).
//!
//! Order is the commit-independent `created_at, id` order of the rows. A row whose transaction
//! commits late can be published after a younger one; events are notifications carrying record
//! versions, so clients re-read and the order of two events is never load-bearing.

use std::time::Duration;

use async_nats::{HeaderMap, jetstream};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Row, types::Uuid};
use tokio::time::sleep;

use crate::{MSG_ID_HEADER, Result, envelope, nats_err, subject};

/// Advisory lock key of the relay ("outbox").
const LOCK_KEY: i64 = 0x0000_6f75_7462_6f78;

/// Relay settings.
#[derive(Debug, Clone)]
pub struct RelayConfig {
    /// Rows per pass.
    pub batch: i64,
    /// Sleep after an empty pass.
    pub idle: Duration,
    /// First sleep after a failed pass; doubles up to `max_backoff`.
    pub backoff: Duration,
    /// Longest sleep after failures.
    pub max_backoff: Duration,
    /// Published rows are deleted after this age.
    pub keep_published: Duration,
    /// How often old rows are deleted.
    pub cleanup_every: Duration,
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            batch: 200,
            idle: Duration::from_millis(250),
            backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(15),
            keep_published: Duration::from_hours(24),
            cleanup_every: Duration::from_mins(5),
        }
    }
}

/// What one pass did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PassReport {
    /// Rows published and marked.
    pub published: usize,
    /// Rows skipped as malformed (marked, never published).
    pub quarantined: usize,
    /// Another relay holds the lock.
    pub busy: bool,
}

struct Row_ {
    id: Uuid,
    topic: String,
    workspace_id: Option<Uuid>,
    payload: Value,
    created_at: DateTime<Utc>,
}

/// Where to publish and the body, or `None` for a malformed row.
fn message_for(row: &Row_) -> Option<(String, Vec<u8>)> {
    if !db::outbox::is_valid_topic(&row.topic) {
        return None;
    }
    let (kind, id) = match row.payload.get("entity") {
        Some(e) => {
            let kind = e.get("kind")?.as_str()?;
            let id = e.get("id")?.as_str()?;
            if !db::outbox::is_token(kind) || Uuid::parse_str(id).is_err() {
                return None;
            }
            (Some(kind), Some(id))
        }
        None => (None, None),
    };
    let subject = subject(row.workspace_id, kind, id, &row.topic);
    let body = envelope(&row.topic, row.workspace_id, &row.payload, row.created_at);
    Some((subject, serde_json::to_vec(&body).ok()?))
}

/// One relay pass.
pub async fn run_once(pool: &PgPool, js: &jetstream::Context, batch: i64) -> Result<PassReport> {
    let mut tx = pool.begin().await?;
    let locked: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
        .bind(LOCK_KEY)
        .fetch_one(&mut *tx)
        .await?;
    if !locked {
        return Ok(PassReport {
            busy: true,
            ..PassReport::default()
        });
    }
    let rows = sqlx::query(
        "SELECT id, topic, workspace_id, payload, created_at FROM outbox
         WHERE published_at IS NULL ORDER BY created_at, id LIMIT $1 FOR UPDATE SKIP LOCKED",
    )
    .bind(batch)
    .fetch_all(&mut *tx)
    .await?
    .into_iter()
    .map(|r| {
        Ok(Row_ {
            id: r.try_get("id")?,
            topic: r.try_get("topic")?,
            workspace_id: r.try_get("workspace_id")?,
            payload: r.try_get("payload")?,
            created_at: r.try_get("created_at")?,
        })
    })
    .collect::<sqlx::Result<Vec<_>>>()?;
    if rows.is_empty() {
        return Ok(PassReport::default());
    }

    // Send everything in order first, then await the acknowledgements in order.
    let mut done: Vec<Uuid> = Vec::with_capacity(rows.len());
    let mut quarantined = 0;
    let mut pending = Vec::with_capacity(rows.len());
    let mut failure = None;
    for row in &rows {
        let Some((subject, body)) = message_for(row) else {
            tracing::error!(outbox_id = %row.id, topic = %row.topic, "malformed outbox row skipped");
            pending.push((row.id, None));
            continue;
        };
        let mut headers = HeaderMap::new();
        headers.insert(MSG_ID_HEADER, row.id.to_string().as_str());
        match js.publish_with_headers(subject, headers, body.into()).await {
            Ok(ack) => pending.push((row.id, Some(ack))),
            Err(e) => {
                failure = Some(nats_err(e));
                break;
            }
        }
    }
    for (id, ack) in pending {
        if failure.is_none() {
            if let Some(ack) = ack {
                match ack.await {
                    Ok(_) => done.push(id),
                    Err(e) => failure = Some(nats_err(e)),
                }
            } else {
                done.push(id);
                quarantined += 1;
            }
        }
    }
    if !done.is_empty() {
        sqlx::query("UPDATE outbox SET published_at = now() WHERE id = ANY($1)")
            .bind(&done)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    if let Some(e) = failure {
        if done.is_empty() {
            return Err(e);
        }
        tracing::warn!(error = %e, published = done.len(), "relay pass stopped early");
    }
    Ok(PassReport {
        published: done.len() - quarantined,
        quarantined,
        busy: false,
    })
}

/// Deletes published rows older than `keep`; returns the count.
pub async fn cleanup(pool: &PgPool, keep: Duration) -> Result<u64> {
    let r = sqlx::query(
        "DELETE FROM outbox WHERE published_at IS NOT NULL
         AND published_at < now() - make_interval(secs => $1)",
    )
    .bind(keep.as_secs_f64())
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}

/// Runs the relay until `shutdown` resolves. Failures back off and retry; nothing is lost
/// because unpublished rows stay in the outbox.
pub async fn run(
    pool: PgPool,
    js: jetstream::Context,
    config: RelayConfig,
    shutdown: impl Future<Output = ()>,
) {
    tokio::pin!(shutdown);
    let mut backoff = config.backoff;
    let mut last_cleanup = tokio::time::Instant::now();
    loop {
        let wait = match run_once(&pool, &js, config.batch).await {
            Ok(r) if r.published + r.quarantined == usize::try_from(config.batch).unwrap_or(0) => {
                backoff = config.backoff;
                Duration::ZERO
            }
            Ok(_) => {
                backoff = config.backoff;
                config.idle
            }
            Err(e) => {
                tracing::warn!(error = %e, "outbox relay pass failed");
                let w = backoff;
                backoff = (backoff * 2).min(config.max_backoff);
                w
            }
        };
        if last_cleanup.elapsed() >= config.cleanup_every {
            last_cleanup = tokio::time::Instant::now();
            if let Err(e) = cleanup(&pool, config.keep_published).await {
                tracing::warn!(error = %e, "outbox cleanup failed");
            }
        }
        tokio::select! {
            () = &mut shutdown => return,
            () = sleep(wait) => {}
        }
    }
}
