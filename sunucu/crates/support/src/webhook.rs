//! GitHub webhooks: verification and storage (`ingest`), application (`worker`).
//!
//! `ingest` checks the signature (`X-Hub-Signature-256`), stores the event once for each
//! delivery id (`X-GitHub-Delivery`) in `external_events` and writes an outbox event. The worker
//! consumer ([`GithubHandler`]) applies it:
//!
//! - A comment that starts with `/yanit`, from a person in the support team list
//!   (`staff_users.github_login`), is a reply to the customer: a customer-visible message, status
//!   `answered`, notification (inbox, push, e-mail). From anyone else the command is ignored.
//! - Any other comment is an internal note. It never goes to the customer.
//! - A comment of the application itself (login of the App, or the hidden marker) is ignored.
//! - A changed comment changes the message (the customer sees `edited_at`); a deleted comment
//!   retracts it. Comments that the customer wrote stay as they are.
//! - Closing an Issue sets the request to `resolved`, reopening sets it to `open`, and the label
//!   `bekliyor:musteri` sets `answered`.

use std::sync::Arc;

use db::outbox::{Entity, NewOutboxEvent, enqueue};
use events::consumer::{BoxFuture, Event, Handler, HandlerError};
use ring::hmac;
use serde_json::{Value, json};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    EVENT_ENTITY, GITHUB_EVENT_TOPIC, MARKER, Result, WAITING_LABEL, enter_workspace,
    reply::{enqueue_sync, notify_customer},
};

/// Result of [`store`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stored {
    /// New delivery: stored.
    Queued,
    /// The same delivery arrived before.
    Duplicate,
}

/// Checks `X-Hub-Signature-256` (`sha256=<hex>`) of the raw body.
#[must_use]
pub fn verify(secret: &[u8], header: Option<&str>, body: &[u8]) -> bool {
    let Some(hex) = header.and_then(|h| h.strip_prefix("sha256=")) else {
        return false;
    };
    if hex.len() % 2 != 0 || !hex.is_ascii() {
        return false;
    }
    let Some(sig) = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect::<Option<Vec<u8>>>()
    else {
        return false;
    };
    hmac::verify(&hmac::Key::new(hmac::HMAC_SHA256, secret), body, &sig).is_ok()
}

/// Stores a verified delivery and writes the outbox event, in the caller's transaction.
///
/// # Errors
/// Database errors.
pub async fn store(
    conn: &mut PgConnection,
    delivery_id: &str,
    event: &str,
    payload: &Value,
) -> Result<Stored> {
    let id = Uuid::now_v7();
    let inserted: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO external_events (id, source, delivery_id, payload)
         VALUES ($1, 'github', $2, $3)
         ON CONFLICT (source, delivery_id) DO NOTHING RETURNING id",
    )
    .bind(id)
    .bind(delivery_id)
    .bind(json!({"event": event, "payload": payload}))
    .fetch_optional(&mut *conn)
    .await?;
    let Some(id) = inserted else {
        return Ok(Stored::Duplicate);
    };
    enqueue(
        conn,
        &NewOutboxEvent {
            topic: GITHUB_EVENT_TOPIC.to_owned(),
            workspace_id: None,
            entity: Some(Entity {
                kind: EVENT_ENTITY.to_owned(),
                id,
                version: 1,
            }),
            data: None,
        },
    )
    .await?;
    Ok(Stored::Queued)
}

/// What [`apply`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applied {
    /// A customer-visible reply was stored.
    Reply,
    /// An internal note was stored.
    Note,
    /// A message was edited or retracted, or the request status changed.
    Updated,
    /// Nothing to do (own comment, unknown Issue, repeated delivery, unaccepted command).
    Ignored,
}

/// The text after `/yanit`, if the comment is a `/yanit` command.
#[must_use]
pub fn reply_text(body: &str) -> Option<&str> {
    let rest = body.trim_start();
    let head = rest.get(..6)?;
    if !head.eq_ignore_ascii_case("/yanit") {
        return None;
    }
    let tail = &rest[6..];
    (tail.is_empty() || tail.starts_with(char::is_whitespace)).then(|| tail.trim())
}

async fn find_request(conn: &mut PgConnection, issue: i64) -> Result<Option<(Uuid, Uuid)>> {
    let Ok(issue) = i32::try_from(issue) else {
        return Ok(None);
    };
    Ok(
        sqlx::query_as("SELECT workspace_id, id FROM support_find_request($1, NULL)")
            .bind(issue)
            .fetch_optional(&mut *conn)
            .await?,
    )
}

/// Applies one stored delivery. Call it with the worker context (no workspace yet).
///
/// # Errors
/// Database errors.
#[allow(clippy::too_many_lines)]
pub async fn apply(
    conn: &mut PgConnection,
    bot_login: &str,
    event: &str,
    payload: &Value,
) -> Result<Applied> {
    let Some(issue) = payload.pointer("/issue/number").and_then(Value::as_i64) else {
        return Ok(Applied::Ignored);
    };
    let action = payload
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let sender = payload
        .pointer("/sender/login")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let is_bot = sender == bot_login;
    let Some((ws, request)) = find_request(conn, issue).await? else {
        return Ok(Applied::Ignored);
    };
    enter_workspace(conn, ws).await?;

    match (event, action) {
        ("issue_comment", "created") => {
            let comment = payload.get("comment").unwrap_or(&Value::Null);
            let body = comment
                .get("body")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let Some(cid) = comment.get("id").and_then(Value::as_i64) else {
                return Ok(Applied::Ignored);
            };
            let author = comment
                .pointer("/user/login")
                .and_then(Value::as_str)
                .map_or_else(|| sender.clone(), str::to_ascii_lowercase);
            if is_bot || author == bot_login || body.contains(MARKER) {
                return Ok(Applied::Ignored);
            }
            let known: Option<Uuid> =
                sqlx::query_scalar("SELECT id FROM support_messages WHERE github_comment_id = $1")
                    .bind(cid)
                    .fetch_optional(&mut *conn)
                    .await?;
            if known.is_some() {
                return Ok(Applied::Ignored);
            }
            let staff: Option<Uuid> = sqlx::query_scalar(
                "SELECT id FROM staff_users
                 WHERE github_login = $1 AND status = 'active' AND deleted_at IS NULL",
            )
            .bind(&author)
            .fetch_optional(&mut *conn)
            .await?;
            let command = reply_text(body);
            if command.is_some() && staff.is_none() {
                tracing::warn!(%author, issue, "/yanit from a person outside the support team");
                return Ok(Applied::Ignored);
            }
            let (kind, visibility, text) = match (command, staff) {
                (Some(t), Some(_)) if !t.is_empty() => ("staff", "customer", t),
                (Some(_), _) => return Ok(Applied::Ignored),
                (None, Some(_)) => ("staff", "internal", body),
                (None, None) => ("system", "internal", body),
            };
            let id = Uuid::now_v7();
            sqlx::query(
                "INSERT INTO support_messages
                     (id, workspace_id, request_id, author_kind, author_staff_id, body, visibility,
                      github_comment_id, github_synced_version)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 1)",
            )
            .bind(id)
            .bind(ws)
            .bind(request)
            .bind(kind)
            .bind(if kind == "staff" { staff } else { None })
            .bind(text)
            .bind(visibility)
            .bind(cid)
            .execute(&mut *conn)
            .await?;
            if visibility == "customer" {
                sqlx::query(
                    "UPDATE support_requests SET status = 'answered'
                     WHERE id = $1 AND status IN ('open', 'answered', 'resolved')",
                )
                .bind(request)
                .execute(&mut *conn)
                .await?;
                notify_customer(conn, ws, request, id).await?;
                // The label `bekliyor:musteri` goes to the Issue.
                enqueue_sync(conn, ws, request).await?;
                Ok(Applied::Reply)
            } else {
                Ok(Applied::Note)
            }
        }
        ("issue_comment", "edited" | "deleted") => {
            let comment = payload.get("comment").unwrap_or(&Value::Null);
            let body = comment
                .get("body")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let Some(cid) = comment.get("id").and_then(Value::as_i64) else {
                return Ok(Applied::Ignored);
            };
            if is_bot || body.contains(MARKER) {
                return Ok(Applied::Ignored);
            }
            // The customer's own messages are not changed from the GitHub side.
            let n = if action == "edited" {
                let text = if sqlx::query_scalar::<_, String>(
                    "SELECT visibility FROM support_messages WHERE github_comment_id = $1",
                )
                .bind(cid)
                .fetch_optional(&mut *conn)
                .await?
                .as_deref()
                    == Some("customer")
                {
                    reply_text(body).unwrap_or(body)
                } else {
                    body
                };
                sqlx::query(
                    "UPDATE support_messages
                     SET body = $2, edited_at = now(), github_synced_version = version + 1
                     WHERE github_comment_id = $1 AND author_kind <> 'customer'
                       AND retracted_at IS NULL",
                )
                .bind(cid)
                .bind(text)
                .execute(&mut *conn)
                .await?
                .rows_affected()
            } else {
                sqlx::query(
                    "UPDATE support_messages SET retracted_at = now(), github_deleted_at = now()
                     WHERE github_comment_id = $1 AND author_kind <> 'customer'
                       AND retracted_at IS NULL",
                )
                .bind(cid)
                .execute(&mut *conn)
                .await?
                .rows_affected()
            };
            Ok(if n > 0 {
                Applied::Updated
            } else {
                Applied::Ignored
            })
        }
        ("issues", "closed" | "reopened" | "labeled" | "unlabeled") if !is_bot => {
            let label = payload.pointer("/label/name").and_then(Value::as_str);
            let (from, to): (&[&str], &str) = match (action, label) {
                ("closed", _) => (&["open", "answered"], "resolved"),
                ("reopened", _) => (&["resolved", "closed"], "open"),
                ("labeled", Some(WAITING_LABEL)) => (&["open"], "answered"),
                ("unlabeled", Some(WAITING_LABEL)) => (&["answered"], "open"),
                _ => return Ok(Applied::Ignored),
            };
            let from: Vec<String> = from.iter().map(|s| (*s).to_owned()).collect();
            let n = sqlx::query(
                "UPDATE support_requests
                 SET status = $2, github_synced_status = $2,
                     resolved_at = CASE WHEN $2 = 'resolved' THEN now() ELSE NULL END
                 WHERE id = $1 AND status = ANY($3)",
            )
            .bind(request)
            .bind(to)
            .bind(&from)
            .execute(&mut *conn)
            .await?
            .rows_affected();
            Ok(if n > 0 {
                Applied::Updated
            } else {
                Applied::Ignored
            })
        }
        _ => Ok(Applied::Ignored),
    }
}

/// Consumer of `support.github_event`.
#[derive(Debug, Clone)]
pub struct GithubHandler {
    bot_login: Arc<str>,
}

impl GithubHandler {
    /// A handler that ignores the events of `bot_login` (the login of the GitHub App).
    #[must_use]
    pub fn new(bot_login: &str) -> Self {
        Self {
            bot_login: bot_login.to_ascii_lowercase().into(),
        }
    }
}

impl Handler for GithubHandler {
    fn handle<'a>(
        &'a self,
        event: &'a Event,
        tx: &'a mut PgConnection,
    ) -> BoxFuture<'a, std::result::Result<(), HandlerError>> {
        Box::pin(async move {
            if event.topic != GITHUB_EVENT_TOPIC {
                return Ok(());
            }
            let id = event
                .body
                .pointer("/entity/id")
                .and_then(Value::as_str)
                .and_then(|s| Uuid::parse_str(s).ok())
                .ok_or_else(|| HandlerError::Permanent("event has no external event".to_owned()))?;
            let row: Option<(Value, bool)> = sqlx::query_as(
                "SELECT payload, processed_at IS NOT NULL FROM external_events
                 WHERE id = $1 FOR UPDATE",
            )
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(crate::Error::from)?;
            let Some((stored, processed)) = row else {
                return Err(HandlerError::Permanent(
                    "external event not found".to_owned(),
                ));
            };
            if processed {
                return Ok(());
            }
            let name = stored
                .get("event")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let payload = stored.get("payload").unwrap_or(&Value::Null);
            let result = apply(tx, &self.bot_login, name, payload).await?;
            tracing::info!(?result, event = name, "GitHub event applied");
            sqlx::query("UPDATE external_events SET processed_at = now() WHERE id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(crate::Error::from)?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::format_collect)]
    use super::*;

    #[test]
    fn signature_check() {
        let key = hmac::Key::new(hmac::HMAC_SHA256, b"secret");
        let tag = hmac::sign(&key, b"{}");
        let hex: String = tag.as_ref().iter().map(|b| format!("{b:02x}")).collect();
        assert!(verify(b"secret", Some(&format!("sha256={hex}")), b"{}"));
        assert!(!verify(b"other", Some(&format!("sha256={hex}")), b"{}"));
        assert!(!verify(b"secret", Some(&format!("sha256={hex}")), b"{ }"));
        assert!(!verify(b"secret", Some("sha256=zz"), b"{}"));
        assert!(!verify(b"secret", Some("sha1=00"), b"{}"));
        assert!(!verify(b"secret", None, b"{}"));
    }

    #[test]
    fn yanit_command() {
        assert_eq!(reply_text("/yanit Merhaba"), Some("Merhaba"));
        assert_eq!(reply_text("  /YANIT\nSatir"), Some("Satir"));
        assert_eq!(reply_text("/yanit"), Some(""));
        assert_eq!(reply_text("/yanitla"), None);
        assert_eq!(reply_text("not /yanit"), None);
        assert_eq!(reply_text("ü"), None);
    }
}
