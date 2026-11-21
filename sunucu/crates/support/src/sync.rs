//! Application to GitHub sync of one support request.
//!
//! [`sync_request`] is state based and safe to repeat: it creates the Issue when the request has
//! none, sends every customer or staff message that has no comment yet, sends edits and deletes
//! of messages, and writes the request status to the Issue (state and the label
//! `bekliyor:musteri`). Progress is saved after each step, so a later run continues where the
//! last one stopped.
//!
//! When GitHub is not available, [`SyncHandler`] schedules a job in `scheduled_jobs`
//! ([`schedule`]) and [`run_due`] retries it with back-off. The request is already open for the
//! customer.

use std::{sync::Arc, time::Duration};

use events::consumer::{BoxFuture, Event, Handler, HandlerError};
use serde_json::{Value, json};
use sqlx::{PgConnection, PgPool};
use tokio::sync::watch;
use uuid::Uuid;

use crate::{
    JOB_KIND, MARKER, Result, SYNC_TOPIC, WAITING_LABEL, enter_workspace,
    github::{GitHub, GitHubError},
};

/// Attempts before a job is marked failed.
const MAX_ATTEMPTS: i32 = 30;

/// What a sync did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Everything is in sync.
    Synced,
    /// GitHub is not available; try again after the delay (GitHub's own wish, if any).
    Later(Option<Duration>),
    /// GitHub refused: a retry does not help.
    Failed(String),
}

type RequestRow = (i64, String, String, String, Option<Value>, Option<i64>, Option<String>, String, String);

/// Labels of an Issue: topic, mode and, while the customer must answer, the waiting label.
#[must_use]
pub fn labels(topic: &str, mode: &str, status: &str) -> Vec<String> {
    let mode = if mode == "business" { "isletme" } else { "ev" };
    let mut labels = vec![format!("konu:{topic}"), format!("mod:{mode}")];
    if status == "answered" {
        labels.push(WAITING_LABEL.to_owned());
    }
    labels
}

fn later(e: &GitHubError) -> Option<Outcome> {
    match e {
        GitHubError::Unavailable { retry_after, .. } => {
            Some(Outcome::Later(retry_after.map(Duration::from_secs)))
        }
        GitHubError::Rejected(_) => None,
    }
}

fn failed(e: &GitHubError) -> Outcome {
    match later(e) {
        Some(o) => o,
        None => Outcome::Failed(e.to_string()),
    }
}

/// The text of a comment: the hidden marker, a header for the author and the message.
#[must_use]
pub fn comment_text(
    message_id: Uuid,
    author_kind: &str,
    visibility: &str,
    body: &str,
    attachments: i64,
) -> String {
    let header = match (author_kind, visibility) {
        ("customer", _) => "**Müşteri**",
        (_, "internal") => "**İç not (destek paneli)**",
        _ => "**Yanıt (destek paneli)**",
    };
    let files = if attachments > 0 {
        format!("\n\nEkler: {attachments} (destek panelinde açılır)")
    } else {
        String::new()
    };
    format!("{MARKER}{message_id} -->\n{header}\n\n{body}{files}")
}

/// Syncs one request. Call [`enter_workspace`] first.
///
/// # Errors
/// Database errors. GitHub failures are in the [`Outcome`].
#[allow(clippy::too_many_lines)]
pub async fn sync_request(
    conn: &mut PgConnection,
    gh: &GitHub,
    request_id: Uuid,
) -> Result<Outcome> {
    let row: Option<RequestRow> = sqlx::query_as(
        "SELECT r.public_number, r.topic, r.title, r.status, r.diagnostics, r.github_issue_number::bigint,
                r.github_synced_status, w.mode, w.support_pseudonym
         FROM support_requests r JOIN workspaces w ON w.id = r.workspace_id
         WHERE r.id = $1 AND r.deleted_at IS NULL FOR UPDATE OF r",
    )
    .bind(request_id)
    .fetch_optional(&mut *conn)
    .await?;
    let Some((number, topic, title, status, diagnostics, issue, mut synced_status, mode, alias)) = row
    else {
        return Ok(Outcome::Synced);
    };

    // 1. The Issue.
    let issue = if let Some(n) = issue {
        n
    } else {
        let version = diagnostics
            .as_ref()
            .and_then(|d| d.get("app_version"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let attachments: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM attachments a JOIN support_messages m
                 ON m.workspace_id = a.workspace_id AND m.id = a.record_id
             WHERE a.record_kind = 'support_message' AND m.request_id = $1
               AND a.deleted_at IS NULL",
        )
        .bind(request_id)
        .fetch_one(&mut *conn)
        .await?;
        let body = format!(
            "Talep: #{number}\nÇalışma alanı: {alias}\nKonu: {topic}\nMod: {mode}\n\
             Uygulama sürümü: {version}\nEk sayısı: {attachments}\n"
        );
        match gh
            .create_issue(
                &format!("#{number} {title}"),
                &body,
                &labels(&topic, &mode, "open"),
            )
            .await
        {
            Ok(n) => {
                sqlx::query(
                    "UPDATE support_requests SET github_issue_number = $2, github_synced_at = now(),
                         github_synced_status = 'open' WHERE id = $1",
                )
                .bind(request_id)
                .bind(i32::try_from(n).unwrap_or(i32::MAX))
                .execute(&mut *conn)
                .await?;
                synced_status = Some("open".to_owned());
                n
            }
            Err(e) => return Ok(failed(&e)),
        }
    };

    // 2. Messages: create, edit, delete.
    type MessageRow = (Uuid, String, String, String, Option<i64>, Option<i64>, i64, bool, bool);
    let messages: Vec<MessageRow> = sqlx::query_as(
        "SELECT id, author_kind, visibility, body, github_comment_id, github_synced_version,
                version, retracted_at IS NOT NULL, github_deleted_at IS NOT NULL
         FROM support_messages WHERE request_id = $1 AND deleted_at IS NULL
         ORDER BY created_at, id",
    )
    .bind(request_id)
    .fetch_all(&mut *conn)
    .await?;
    for (id, author, visibility, body, comment, synced, version, retracted, gh_deleted) in messages
    {
        let attachments: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM attachments WHERE record_kind = 'support_message'
               AND record_id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_one(&mut *conn)
        .await?;
        match (comment, retracted) {
            (None, false) => {
                let text = comment_text(id, &author, &visibility, &body, attachments);
                match gh.add_comment(issue, &text).await {
                    Ok(c) => {
                        sqlx::query(
                            "UPDATE support_messages
                             SET github_comment_id = $2, github_synced_version = version + 1
                             WHERE id = $1",
                        )
                        .bind(id)
                        .bind(c)
                        .execute(&mut *conn)
                        .await?;
                    }
                    Err(e) => return Ok(failed(&e)),
                }
            }
            (Some(c), false) if synced.is_none_or(|s| s < version) => {
                let text = comment_text(id, &author, &visibility, &body, attachments);
                match gh.edit_comment(c, &text).await {
                    Ok(()) => {
                        sqlx::query(
                            "UPDATE support_messages SET github_synced_version = version + 1
                             WHERE id = $1",
                        )
                        .bind(id)
                        .execute(&mut *conn)
                        .await?;
                    }
                    Err(e) => return Ok(failed(&e)),
                }
            }
            (Some(c), true) if !gh_deleted => match gh.delete_comment(c).await {
                Ok(()) => {
                    sqlx::query(
                        "UPDATE support_messages SET github_deleted_at = now() WHERE id = $1",
                    )
                    .bind(id)
                    .execute(&mut *conn)
                    .await?;
                }
                Err(e) => return Ok(failed(&e)),
            },
            _ => {}
        }
    }

    // 3. The status.
    if synced_status.as_deref() != Some(status.as_str()) {
        let closed = matches!(status.as_str(), "resolved" | "closed");
        match gh
            .update_issue(issue, closed, &labels(&topic, &mode, &status))
            .await
        {
            Ok(()) => {
                sqlx::query(
                    "UPDATE support_requests SET github_synced_status = $2 WHERE id = $1",
                )
                .bind(request_id)
                .bind(&status)
                .execute(&mut *conn)
                .await?;
            }
            Err(e) => return Ok(failed(&e)),
        }
    }
    sqlx::query("UPDATE support_requests SET github_synced_at = now() WHERE id = $1")
        .bind(request_id)
        .execute(&mut *conn)
        .await?;
    Ok(Outcome::Synced)
}

/// Back-off for attempt `attempts` (5 s, 10 s, 20 s, ... at most one hour).
#[must_use]
pub fn backoff(attempts: i32) -> Duration {
    Duration::from_secs((5u64 << attempts.clamp(0, 10).unsigned_abs()).min(3600))
}

/// Schedules a retry for a request (one pending job for each request).
///
/// # Errors
/// Database errors.
pub async fn schedule(
    conn: &mut PgConnection,
    workspace_id: Uuid,
    request_id: Uuid,
    delay: Duration,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO scheduled_jobs (id, workspace_id, kind, run_at, payload)
         VALUES ($1, $2, $3, now() + make_interval(secs => $4), $5)
         ON CONFLICT ((payload ->> 'request_id'))
             WHERE kind = 'support.github_sync' AND status IN ('pending', 'running')
         DO NOTHING",
    )
    .bind(Uuid::now_v7())
    .bind(workspace_id)
    .bind(JOB_KIND)
    .bind(delay.as_secs_f64())
    .bind(json!({"schema_version": 1, "request_id": request_id}))
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Consumer of `support.sync_requested`. Without a GitHub client the sync waits in a job.
#[derive(Debug, Clone)]
pub struct SyncHandler {
    github: Option<Arc<GitHub>>,
}

impl SyncHandler {
    /// A handler for `github` (`None`: only schedule).
    #[must_use]
    pub const fn new(github: Option<Arc<GitHub>>) -> Self {
        Self { github }
    }
}

impl Handler for SyncHandler {
    fn handle<'a>(
        &'a self,
        event: &'a Event,
        tx: &'a mut PgConnection,
    ) -> BoxFuture<'a, std::result::Result<(), HandlerError>> {
        Box::pin(async move {
            if event.topic != SYNC_TOPIC {
                return Ok(());
            }
            let (Some(ws), Some(id)) = (
                event.workspace_id,
                event
                    .body
                    .pointer("/entity/id")
                    .and_then(Value::as_str)
                    .and_then(|s| Uuid::parse_str(s).ok()),
            ) else {
                return Err(HandlerError::Permanent("event has no request".to_owned()));
            };
            enter_workspace(tx, ws).await?;
            let Some(gh) = &self.github else {
                schedule(tx, ws, id, Duration::from_mins(1)).await?;
                return Ok(());
            };
            match sync_request(tx, gh, id).await? {
                Outcome::Synced => {}
                Outcome::Later(wait) => {
                    schedule(tx, ws, id, wait.unwrap_or_else(|| backoff(0))).await?;
                }
                Outcome::Failed(reason) => {
                    tracing::error!(request = %id, %reason, "support sync rejected by GitHub");
                }
            }
            Ok(())
        })
    }
}

/// Runs the due retry jobs once. Returns the number of jobs handled.
///
/// # Errors
/// Database errors.
pub async fn run_due(pool: &PgPool, gh: &GitHub) -> Result<usize> {
    let mut handled = 0;
    loop {
        let mut tx = pool.begin().await?;
        let job: Option<(Uuid, Uuid, Value, i32)> = sqlx::query_as(
            "SELECT id, workspace_id, payload, attempts FROM scheduled_jobs
             WHERE kind = $1 AND status = 'pending' AND run_at <= now()
             ORDER BY run_at LIMIT 1 FOR UPDATE SKIP LOCKED",
        )
        .bind(JOB_KIND)
        .fetch_optional(&mut *tx)
        .await?;
        let Some((job_id, ws, payload, attempts)) = job else {
            return Ok(handled);
        };
        let request = payload
            .get("request_id")
            .and_then(Value::as_str)
            .and_then(|s| Uuid::parse_str(s).ok());
        let outcome = match request {
            Some(r) => {
                enter_workspace(&mut tx, ws).await?;
                sync_request(&mut tx, gh, r).await?
            }
            None => Outcome::Failed("job has no request".to_owned()),
        };
        match outcome {
            Outcome::Synced => {
                sqlx::query("UPDATE scheduled_jobs SET status = 'done' WHERE id = $1")
                    .bind(job_id)
                    .execute(&mut *tx)
                    .await?;
            }
            Outcome::Later(_) if attempts + 1 >= MAX_ATTEMPTS => {
                sqlx::query(
                    "UPDATE scheduled_jobs SET status = 'failed', attempts = attempts + 1,
                         last_error = 'GitHub stayed unavailable' WHERE id = $1",
                )
                .bind(job_id)
                .execute(&mut *tx)
                .await?;
            }
            Outcome::Later(wait) => {
                let delay = wait.unwrap_or_else(|| backoff(attempts + 1)).max(Duration::from_secs(1));
                sqlx::query(
                    "UPDATE scheduled_jobs SET attempts = attempts + 1, last_error = 'unavailable',
                         run_at = now() + make_interval(secs => $2) WHERE id = $1",
                )
                .bind(job_id)
                .bind(delay.as_secs_f64())
                .execute(&mut *tx)
                .await?;
            }
            Outcome::Failed(reason) => {
                sqlx::query(
                    "UPDATE scheduled_jobs SET status = 'failed', attempts = attempts + 1,
                         last_error = $2 WHERE id = $1",
                )
                .bind(job_id)
                .bind(reason.chars().take(500).collect::<String>())
                .execute(&mut *tx)
                .await?;
            }
        }
        tx.commit().await?;
        handled += 1;
        if handled >= 50 {
            return Ok(handled);
        }
    }
}

/// Runs [`run_due`] every 10 seconds until `stop` turns `true`.
pub async fn run_jobs(pool: PgPool, gh: Option<Arc<GitHub>>, mut stop: watch::Receiver<bool>) {
    let Some(gh) = gh else {
        tracing::warn!("no GitHub settings; support sync jobs wait");
        return;
    };
    loop {
        if let Err(e) = run_due(&pool, &gh).await {
            tracing::warn!(error = %e, "support sync jobs failed");
        }
        tokio::select! {
            r = stop.changed() => if r.is_err() || *stop.borrow() { return; },
            () = tokio::time::sleep(Duration::from_secs(10)) => {}
        }
    }
}
