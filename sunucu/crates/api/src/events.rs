//! Live updates over SSE (ADR-0008 rule 9): `GET /v1/workspaces/{ws}/events`.
//!
//! Every connection reads the workspace's subjects of the `EVENTS` `JetStream` stream with its own
//! ordered consumer, so any API pod serves any client (pods share nothing but NATS). Only
//! workspace members connect (the authentication layer answers `401` / `404`), the consumer's
//! subject filter is the workspace and every event is checked against the workspace again. Events
//! are notifications without record content. `Last-Event-ID` replays from the stream (one hour
//! retention) and answers `event: reset` when the event is no longer there. Membership is
//! re-checked every 30 seconds and the stream closes when it ends; a `: ping` comment goes out
//! every 25 seconds.

use std::{collections::BTreeSet, convert::Infallible, time::Duration};

use async_nats::jetstream::{
    self,
    consumer::{DeliverPolicy, push::OrderedConfig},
};
use axum::{
    Router,
    extract::{Path, Query, State},
    http::{HeaderMap, header::HeaderName},
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
};
use futures_util::{Stream, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

use crate::{
    ApiError, ApiResult, AppState, RequestContext, authenticated,
    error::{ErrorCode, FieldIssue},
};

/// Keep-alive comment interval (ADR-0008 rule 9.5).
pub const PING_INTERVAL: Duration = Duration::from_secs(25);
/// How often the stream re-checks the caller's session and membership.
pub const REVALIDATE_INTERVAL: Duration = Duration::from_secs(30);
/// Replay starts this long before the `Last-Event-ID` event, to survive clock skew.
const REPLAY_SLACK: Duration = Duration::from_secs(5);

/// Query of the events endpoint.
#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    /// `batch:{id},recipe:{id}`; absent means every event of the workspace.
    pub topics: Option<String>,
}

/// Routes of this module (wrapped with the authentication layer).
pub fn routes(state: &AppState) -> Router<AppState> {
    authenticated(
        state,
        Router::new().route("/v1/workspaces/{ws}/events", get(workspace_events)),
    )
}

fn parse_topics(raw: Option<&str>) -> ApiResult<Option<BTreeSet<(String, String)>>> {
    let Some(raw) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    let mut set = BTreeSet::new();
    for item in raw.split(',') {
        let valid = item
            .split_once(':')
            .filter(|(kind, id)| db::outbox::is_token(kind) && Uuid::parse_str(id).is_ok());
        let Some((kind, id)) = valid else {
            return Err(ApiError::validation(vec![FieldIssue::new(
                "topics",
                "invalid_topic",
            )]));
        };
        set.insert((kind.to_owned(), id.to_ascii_lowercase()));
    }
    Ok(Some(set))
}

/// Start time of a replay for `last`, or `None` when the id is not a `UUIDv7` inside the stream's
/// retention (the client must reset).
fn replay_start(last: &str, now: std::time::SystemTime) -> Option<time::OffsetDateTime> {
    let ts = Uuid::parse_str(last).ok()?.get_timestamp()?;
    let (secs, nanos) = ts.to_unix();
    let at = std::time::UNIX_EPOCH + Duration::new(secs, nanos);
    if now.duration_since(at).ok()? >= events::RETENTION {
        return None;
    }
    let start = at
        .checked_sub(REPLAY_SLACK)
        .unwrap_or(std::time::UNIX_EPOCH);
    let secs = start.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    time::OffsetDateTime::from_unix_timestamp(i64::try_from(secs).ok()?).ok()
}

fn unavailable(step: &str, e: &dyn std::fmt::Display) -> ApiError {
    tracing::warn!(step, error = %e, "SSE setup failed");
    ApiError::unavailable(5)
}

fn reset_event() -> Event {
    Event::default().event("reset").data("{}")
}

/// Live updates of one workspace (SSE).
#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws}/events",
    tag = "events",
    operation_id = "events_stream",
    params(
        ("ws" = Uuid, Path, description = "Workspace id"),
        ("topics" = Option<String>, Query, description = "Comma separated `kind:id` filters, e.g. `batch:{id},recipe:{id}`"),
        ("Last-Event-ID" = Option<String>, Header, description = "Last event id the client saw; the server replays what followed (one hour)")
    ),
    responses(
        (
            status = 200,
            description = "`text/event-stream`. Events carry the outbox id as `id`, the event type as `event` and a JSON notification (`type`, `workspace_id`, `entity`, `occurred_at`, optional `data`) as `data`. `event: reset` asks the client to re-read open screens. `: ping` comments keep the connection alive.",
            content_type = "text/event-stream",
            body = String,
            example = json!("id: 0192f1c0-7a3b-7000-8000-000000000001\nevent: batch.measurement_added\ndata: {\"type\":\"batch.measurement_added\",\"workspace_id\":\"0192f1c0-7a3b-7000-8000-000000000002\",\"entity\":{\"kind\":\"batch\",\"id\":\"0192f1c0-7a3b-7000-8000-000000000003\",\"version\":12},\"occurred_at\":\"2026-11-13T09:30:00Z\"}\n\n")
        )
    )
)]
pub async fn workspace_events(
    State(state): State<AppState>,
    ctx: RequestContext,
    Path(ws): Path<Uuid>,
    Query(query): Query<EventsQuery>,
    headers: HeaderMap,
) -> ApiResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    if ctx.workspace_id != Some(ws) {
        return Err(ApiError::new(ErrorCode::NotFound));
    }
    let topics = parse_topics(query.topics.as_deref())?;
    let js = state
        .events
        .clone()
        .ok_or_else(|| ApiError::unavailable(5))?;
    let last_event_id = headers
        .get(HeaderName::from_static("last-event-id"))
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned);

    let (start, mut reset_first) = match last_event_id.as_deref() {
        None => (DeliverPolicy::New, false),
        Some(last) => match replay_start(last, std::time::SystemTime::now()) {
            Some(start_time) => (DeliverPolicy::ByStartTime { start_time }, false),
            None => (DeliverPolicy::New, true),
        },
    };
    let stream = js
        .get_stream(events::STREAM)
        .await
        .map_err(|e| unavailable("stream lookup", &e))?;
    let consumer = stream
        .create_consumer(OrderedConfig {
            deliver_subject: format!("_INBOX.sse.{}", Uuid::new_v4().simple()),
            filter_subject: events::workspace_filter(ws),
            deliver_policy: start,
            ..Default::default()
        })
        .await
        .map_err(|e| unavailable("consumer creation", &e))?;
    // Messages already waiting at start: the replay window.
    let replay_left = if last_event_id.is_some() && !reset_first {
        consumer.cached_info().num_pending.min(u64::from(u32::MAX))
    } else {
        0
    };
    let mut skipping = last_event_id.filter(|_| !reset_first);
    if skipping.is_some() && replay_left == 0 {
        reset_first = true;
        skipping = None;
    }
    let messages = consumer
        .messages()
        .await
        .map_err(|e| unavailable("consumer start", &e))?;

    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(64);
    tokio::spawn(
        Pump {
            messages,
            tx,
            state: state.clone(),
            headers,
            user: ctx.identity.user_id,
            ws,
            topics,
            replay_left,
            skipping,
            reset_first,
        }
        .run(),
    );

    Ok(Sse::new(ReceiverStream::new(rx))
        .keep_alive(KeepAlive::new().interval(PING_INTERVAL).text("ping")))
}

/// One connection's forwarding task: stream messages in, SSE events out.
struct Pump {
    messages: jetstream::consumer::push::Ordered,
    tx: mpsc::Sender<Result<Event, Infallible>>,
    state: AppState,
    headers: HeaderMap,
    user: Uuid,
    ws: Uuid,
    topics: Option<BTreeSet<(String, String)>>,
    /// Replay messages still to come (those waiting when the consumer started).
    replay_left: u64,
    /// The client's last event id while it is still being skipped over.
    skipping: Option<String>,
    reset_first: bool,
}

impl Pump {
    async fn run(mut self) {
        if self.reset_first && self.tx.send(Ok(reset_event())).await.is_err() {
            return;
        }
        let mut recheck = tokio::time::interval(REVALIDATE_INTERVAL);
        recheck.tick().await;
        loop {
            tokio::select! {
                () = self.tx.closed() => return,
                _ = recheck.tick() => {
                    if !still_member(&self.state, &self.headers, self.user, self.ws).await {
                        return;
                    }
                }
                next = self.messages.next() => {
                    let Some(Ok(msg)) = next else { return };
                    if !self.forward(&msg.message).await {
                        return;
                    }
                }
            }
        }
    }

    /// Handles one message; `false` ends the task (client gone).
    async fn forward(&mut self, msg: &async_nats::Message) -> bool {
        let in_replay = self.replay_left > 0;
        self.replay_left = self.replay_left.saturating_sub(1);
        let id = msg
            .headers
            .as_ref()
            .and_then(|h| h.get(events::MSG_ID_HEADER))
            .map(ToString::to_string);
        if let Some(last) = &self.skipping {
            if id.as_deref() == Some(last.as_str()) {
                self.skipping = None;
            } else if in_replay && self.replay_left == 0 {
                // The replay window ended without the client's last event: it is gone (or never
                // was); everything may have changed.
                self.skipping = None;
                return self.tx.send(Ok(reset_event())).await.is_ok();
            }
            return true;
        }
        match to_sse(msg, self.ws, id, self.topics.as_ref()) {
            Some(event) => self.tx.send(Ok(event)).await.is_ok(),
            None => true,
        }
    }
}

/// The SSE event for a stream message, or `None` when it must not reach this client.
fn to_sse(
    msg: &async_nats::Message,
    ws: Uuid,
    id: Option<String>,
    topics: Option<&BTreeSet<(String, String)>>,
) -> Option<Event> {
    let (subject_ws, kind, entity_id) = events::parse_subject(&msg.subject)?;
    if subject_ws != ws.to_string() {
        return None;
    }
    let body: serde_json::Value = serde_json::from_slice(&msg.payload).ok()?;
    if body.get("workspace_id").and_then(|v| v.as_str()) != Some(subject_ws) {
        return None;
    }
    if let Some(topics) = topics
        && !topics.contains(&(kind.to_owned(), entity_id.to_owned()))
    {
        return None;
    }
    let name = body.get("type")?.as_str()?.to_owned();
    let mut event = Event::default().event(name).data(body.to_string());
    if let Some(id) = id {
        event = event.id(id);
    }
    Some(event)
}

async fn still_member(state: &AppState, headers: &HeaderMap, user: Uuid, ws: Uuid) -> bool {
    let Ok(identity) = state.auth.authenticate(headers).await else {
        return false;
    };
    if identity.user_id != user {
        return false;
    }
    matches!(state.auth.membership(&identity, ws).await, Ok(Some(_)))
}

/// Re-exported for the NATS handle type in [`AppState`].
pub type JetStream = jetstream::Context;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topics_parse() {
        let id = Uuid::now_v7();
        let t = parse_topics(Some(&format!("batch:{id},recipe:{id}")))
            .unwrap()
            .unwrap();
        assert_eq!(t.len(), 2);
        assert!(parse_topics(None).unwrap().is_none());
        assert!(parse_topics(Some("batch")).is_err());
        assert!(parse_topics(Some("batch:nope")).is_err());
        assert!(parse_topics(Some("*:x")).is_err());
    }

    #[test]
    fn replay_window() {
        let now = std::time::SystemTime::now();
        let recent = Uuid::now_v7().to_string();
        assert!(replay_start(&recent, now).is_some());
        assert!(replay_start("not-an-id", now).is_none());
        assert!(replay_start(&Uuid::new_v4().to_string(), now).is_none());
        let later = now + Duration::from_hours(2);
        assert!(replay_start(&recent, later).is_none());
    }
}
