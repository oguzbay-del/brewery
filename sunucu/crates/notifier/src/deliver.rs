//! Delivery of a notification: push (APNs) and e-mail, with the preferences of the recipient.
//!
//! Rules:
//! - Quiet hours defer priority 2 and 3 to the end of the quiet time. Priority 1 ignores them.
//! - Priority 3 with the daily summary on is not pushed. It stays in the inbox and in the summary.
//! - E-mail goes out only for the kinds in [`EMAIL_KINDS`]. Alerts never go out by e-mail.
//! - A dead APNs token is deleted.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use events::consumer::{BoxFuture, Event, Handler, HandlerError};
use serde_json::Value;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    CREATED_TOPIC, Error, Result,
    apns::{self, ApnsConfig, Sent, Signer},
    enter_workspace,
    smtp::{self, Mail, SmtpConfig},
};

/// Notification kinds that also go out by e-mail. The mail has no content, only a pointer to the
/// app.
pub const EMAIL_KINDS: &[&str] = &["support.reply", "support.emergency_access"];

/// What [`deliver_one`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The row is not open, already delivered, not due or locked by another worker.
    Skipped,
    /// Quiet hours: delivery moved to this time.
    Deferred(DateTime<Utc>),
    /// Done: the number of pushes accepted and whether a mail went out.
    Delivered {
        /// Pushes accepted by APNs.
        pushes: usize,
        /// A mail was sent.
        email: bool,
    },
}

/// Push and mail channels.
#[derive(Debug)]
pub struct Deliverer {
    http: reqwest::Client,
    apns: Option<ApnsConfig>,
    smtp: Option<SmtpConfig>,
}

impl Deliverer {
    /// A deliverer with the given channels. `None` switches a channel off.
    #[must_use]
    pub fn new(apns: Option<ApnsConfig>, smtp: Option<SmtpConfig>) -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            apns,
            smtp,
        }
    }

    /// Reads `APNS_URL`, `APNS_TOPIC`, `APNS_KEY_FILE`, `APNS_KEY_ID`, `APNS_TEAM_ID` and
    /// `SMTP_HOST`, `SMTP_PORT`, `SMTP_FROM`. A channel without settings is off.
    #[must_use]
    pub fn from_env() -> Self {
        let var = |n: &str| std::env::var(n).ok().filter(|v| !v.is_empty());
        let signer = match (
            var("APNS_KEY_FILE"),
            var("APNS_KEY_ID"),
            var("APNS_TEAM_ID"),
        ) {
            (Some(file), Some(kid), Some(team)) => std::fs::read_to_string(&file)
                .map_err(|e| e.to_string())
                .and_then(|pem| Signer::from_pem(&pem, &kid, &team))
                .map_err(|e| tracing::error!(error = %e, "APNs key unusable; push is off"))
                .ok(),
            _ => None,
        };
        let base_url = var("APNS_URL");
        let apns = (base_url.is_some() || signer.is_some()).then(|| ApnsConfig {
            base_url,
            topic: var("APNS_TOPIC").unwrap_or_else(|| "app.demleme".to_owned()),
            signer,
        });
        let smtp = var("SMTP_HOST").map(|host| SmtpConfig {
            host,
            port: var("SMTP_PORT")
                .and_then(|p| p.parse().ok())
                .unwrap_or(1025),
            from: var("SMTP_FROM").unwrap_or_else(|| "noreply@demleme.local".to_owned()),
        });
        if apns.is_none() {
            tracing::warn!("no APNs settings; push is off");
        }
        if smtp.is_none() {
            tracing::warn!("no SMTP_HOST; e-mail is off");
        }
        Self::new(apns, smtp)
    }

    /// Whether mail can go out.
    #[must_use]
    pub const fn can_mail(&self) -> bool {
        self.smtp.is_some()
    }

    /// Sends a mail. `Ok(false)` when the channel is off.
    ///
    /// # Errors
    /// A text when the relay fails.
    pub async fn mail(
        &self,
        to: &str,
        subject: &str,
        text: &str,
    ) -> std::result::Result<bool, String> {
        let Some(cfg) = &self.smtp else {
            return Ok(false);
        };
        smtp::send(
            cfg,
            &Mail {
                to: to.to_owned(),
                subject: subject.to_owned(),
                text: text.to_owned(),
            },
        )
        .await?;
        Ok(true)
    }
}

type Row = (Uuid, i16, String, String, Value, Uuid);

async fn record(
    conn: &mut PgConnection,
    ws: Uuid,
    notification: Uuid,
    kind: &str,
    target: Option<Uuid>,
    result: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO notification_events
             (id, workspace_id, notification_id, kind, push_target_id, push_result)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(Uuid::now_v7())
    .bind(ws)
    .bind(notification)
    .bind(kind)
    .bind(target)
    .bind(result)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

fn loc_args(params: &Value) -> Vec<String> {
    let mut args = Vec::new();
    for key in ["rule_name", "value", "request_number"] {
        match params.get(key) {
            Some(Value::String(s)) => args.push(s.clone()),
            Some(v @ Value::Number(_)) => args.push(v.to_string()),
            _ => {}
        }
    }
    args
}

/// Delivers one notification in the context of its workspace (call [`enter_workspace`] first).
///
/// # Errors
/// Database errors, or [`Error::Delivery`] when every push failed in a way a retry can fix.
pub async fn deliver_one(
    conn: &mut PgConnection,
    d: &Deliverer,
    workspace_id: Uuid,
    id: Uuid,
    now: DateTime<Utc>,
) -> Result<Outcome> {
    let row: Option<Row> = sqlx::query_as(
        "SELECT n.id, n.priority, n.kind, n.text_key, n.params, m.user_id
         FROM notifications n
         JOIN workspace_members m ON m.workspace_id = n.workspace_id AND m.id = n.recipient_member_id
         WHERE n.id = $1 AND n.status = 'open' AND n.delivered_at IS NULL AND n.deleted_at IS NULL
           AND n.deliver_after <= $2
         FOR UPDATE OF n SKIP LOCKED",
    )
    .bind(id)
    .bind(now)
    .fetch_optional(&mut *conn)
    .await?;
    let Some((id, priority, kind, text_key, params, user_id)) = row else {
        return Ok(Outcome::Skipped);
    };
    db::set_user_id(conn, user_id).await?;
    let (push_enabled, email_enabled, summary, quiet_until): (
        bool,
        bool,
        bool,
        Option<DateTime<Utc>>,
    ) = sqlx::query_as(
        "SELECT COALESCE(p.push_enabled, true), COALESCE(p.email_enabled, true),
                    COALESCE(p.daily_summary, false),
                    notify_quiet_until($2, COALESCE(p.timezone, w.time_zone),
                                       p.quiet_start, p.quiet_end)
             FROM workspaces w
             LEFT JOIN notification_preferences p ON p.workspace_id = w.id AND p.user_id = $3
                  AND p.deleted_at IS NULL
             WHERE w.id = $1",
    )
    .bind(workspace_id)
    .bind(now)
    .bind(user_id)
    .fetch_one(&mut *conn)
    .await?;

    if priority > 1
        && let Some(until) = quiet_until
    {
        sqlx::query("UPDATE notifications SET deliver_after = $2 WHERE id = $1")
            .bind(id)
            .bind(until)
            .execute(&mut *conn)
            .await?;
        return Ok(Outcome::Deferred(until));
    }

    let mut pushes = 0;
    let mut transient: Option<String> = None;
    let summary_only = priority == 3 && summary;
    if summary_only {
        record(
            conn,
            workspace_id,
            id,
            "suppressed",
            None,
            Some("daily_summary"),
        )
        .await?;
    } else if let (true, Some(cfg)) = (push_enabled, &d.apns) {
        let targets: Vec<(Uuid, String, String)> =
            sqlx::query_as("SELECT id, environment, token FROM push_targets WHERE user_id = $1")
                .bind(user_id)
                .fetch_all(&mut *conn)
                .await?;
        let body = apns::payload(priority, &text_key, &loc_args(&params), &id.to_string());
        for (target, env, token) in targets {
            match apns::send(&d.http, cfg, &env, &token, priority, &body).await {
                Sent::Accepted => {
                    pushes += 1;
                    record(
                        conn,
                        workspace_id,
                        id,
                        "sent",
                        Some(target),
                        Some("accepted"),
                    )
                    .await?;
                    sqlx::query(
                        "UPDATE push_targets SET last_success_at = now(), last_error = NULL
                         WHERE id = $1",
                    )
                    .bind(target)
                    .execute(&mut *conn)
                    .await?;
                }
                Sent::Unregistered => {
                    record(conn, workspace_id, id, "sent", None, Some("unregistered")).await?;
                    sqlx::query("DELETE FROM push_targets WHERE id = $1")
                        .bind(target)
                        .execute(&mut *conn)
                        .await?;
                }
                Sent::Rejected(reason) => {
                    record(
                        conn,
                        workspace_id,
                        id,
                        "sent",
                        Some(target),
                        Some(&reason.chars().take(190).collect::<String>()),
                    )
                    .await?;
                    sqlx::query(
                        "UPDATE push_targets SET last_error = $2, last_error_at = now()
                         WHERE id = $1",
                    )
                    .bind(target)
                    .bind(reason.chars().take(200).collect::<String>())
                    .execute(&mut *conn)
                    .await?;
                }
                Sent::Transient(reason) => transient = Some(reason),
            }
        }
    }
    if pushes == 0
        && let Some(reason) = transient
    {
        return Err(Error::Delivery(reason));
    }

    let mut emailed = false;
    if email_enabled && !summary_only && EMAIL_KINDS.contains(&kind.as_str()) && d.can_mail() {
        let to: Option<Option<String>> = sqlx::query_scalar(
            "SELECT contact_email FROM users
             WHERE id = $1 AND contact_email_verified_at IS NOT NULL",
        )
        .bind(user_id)
        .fetch_optional(&mut *conn)
        .await?;
        if let Some(Some(to)) = to {
            let (subject, text) = mail_text(&kind, &params);
            match d.mail(&to, &subject, &text).await {
                Ok(sent) => {
                    emailed = sent;
                    if sent {
                        record(conn, workspace_id, id, "email_sent", None, None).await?;
                    }
                }
                Err(e) => return Err(Error::Delivery(e)),
            }
        }
    }
    sqlx::query("UPDATE notifications SET delivered_at = now() WHERE id = $1")
        .bind(id)
        .execute(&mut *conn)
        .await?;
    Ok(Outcome::Delivered {
        pushes,
        email: emailed,
    })
}

/// Subject and body of a notification mail: a pointer to the app, never the content.
fn mail_text(kind: &str, params: &Value) -> (String, String) {
    match kind {
        "support.reply" => {
            let number = params.get("request_number").map(ToString::to_string).unwrap_or_default();
            (
                format!("You have a reply to your support request #{number}"),
                "Our support team replied to your request.\nOpen the app and go to Help and support to read the reply.\n".to_owned(),
            )
        }
        _ => (
            "Support access to your workspace".to_owned(),
            "Support staff opened emergency access to your workspace.\nOpen the app and go to Help and support, then Support access history, for details.\n".to_owned(),
        ),
    }
}

/// Consumer of `notification.created`.
#[derive(Debug, Clone)]
pub struct DeliveryHandler {
    deliverer: Arc<Deliverer>,
}

impl DeliveryHandler {
    /// A handler that delivers with `deliverer`.
    #[must_use]
    pub const fn new(deliverer: Arc<Deliverer>) -> Self {
        Self { deliverer }
    }
}

impl Handler for DeliveryHandler {
    fn handle<'a>(
        &'a self,
        event: &'a Event,
        tx: &'a mut PgConnection,
    ) -> BoxFuture<'a, std::result::Result<(), HandlerError>> {
        Box::pin(async move {
            if event.topic != CREATED_TOPIC {
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
                return Err(HandlerError::Permanent(
                    "event has no notification".to_owned(),
                ));
            };
            enter_workspace(tx, ws).await?;
            deliver_one(tx, &self.deliverer, ws, id, Utc::now()).await?;
            Ok(())
        })
    }
}
