//! Sentry error reporting (feature `sentry`, ADR-0010).
//!
//! Sentry is off when `SENTRY_DSN` is not set. Every event passes through [`scrub`], which
//! removes tokens and personal data before the event leaves the process.

use sentry::protocol::{Event, Request};
use sentry::{ClientInitGuard, ClientOptions};
use sentry_tracing::{BreadcrumbLayer, ErrorLayer};
use tracing::Subscriber;
use tracing_subscriber::filter::{LevelFilter, filter_fn};
use tracing_subscriber::layer::Layer;
use tracing_subscriber::registry::LookupSpan;

use crate::config::secret_from_lookup;

/// Environment variable with the Sentry DSN.
pub const ENV_DSN: &str = "SENTRY_DSN";
/// Environment variable with the path of a file that holds the Sentry DSN.
pub const ENV_DSN_FILE: &str = "SENTRY_DSN_FILE";
/// Environment variable with the deployment environment name.
pub const ENV_ENVIRONMENT: &str = "SENTRY_ENVIRONMENT";

/// Replacement for each removed value.
pub const FILTERED: &str = "[Filtered]";

const SECRET_HEADERS: &[&str] = &[
    "authorization",
    "cookie",
    "set-cookie",
    "proxy-authorization",
    "x-api-key",
    "idempotency-key",
    "paddle-signature",
    "x-hub-signature-256",
];
const HEADER_FRAGMENTS: &[&str] = &["token", "secret", "key"];
const QUERY_FRAGMENTS: &[&str] = &["token", "secret", "key", "code", "password", "signature"];

/// Starts Sentry. Returns `None` when no DSN is set or when the DSN is not valid.
///
/// Keep the returned guard alive for the life of the process. It flushes events on drop.
#[must_use]
pub fn init(service_name: &str, release: &str) -> Option<ClientInitGuard> {
    init_with(
        |name| std::env::var(name).ok(),
        service_name,
        release,
        ClientOptions::default(),
    )
}

/// Like [`init`], with an explicit environment lookup and base options (a test replaces the
/// transport with them).
pub fn init_with<F>(
    lookup: F,
    service_name: &str,
    release: &str,
    base: ClientOptions,
) -> Option<ClientInitGuard>
where
    F: Fn(&str) -> Option<String>,
{
    let dsn = match secret_from_lookup(&lookup, ENV_DSN, ENV_DSN_FILE) {
        Ok(Some(dsn)) => dsn,
        Ok(None) => {
            tracing::info!("sentry disabled: {ENV_DSN} is not set");
            return None;
        }
        Err(e) => {
            tracing::error!(error = %e, "sentry disabled: cannot read the DSN");
            return None;
        }
    };
    let Ok(dsn) = dsn.parse::<sentry::types::Dsn>() else {
        tracing::error!("sentry disabled: {ENV_DSN} is not a valid DSN");
        return None;
    };
    let environment = lookup(ENV_ENVIRONMENT)
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "local".to_owned());
    let mut options = base
        .sample_rate(1.0)
        .traces_sample_rate(0.0)
        .attach_stacktrace(true)
        .send_default_pii(false)
        .server_name("")
        .release(release.to_owned())
        .environment(environment)
        .before_send(scrub);
    options.dsn = Some(dsn);
    let guard = sentry::init(options);
    let service = service_name.to_owned();
    sentry::configure_scope(|scope| scope.set_tag("service", service));
    Some(guard)
}

/// Tracing layer: only `ERROR` events create Sentry events; `WARN` becomes a breadcrumb;
/// `INFO` and lower are not sent.
#[must_use]
pub fn layer<S>() -> impl Layer<S> + Send + Sync
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let errors = ErrorLayer::new().with_filter(LevelFilter::ERROR);
    let crumbs = BreadcrumbLayer::new().with_filter(filter_fn(|metadata| {
        *metadata.level() == tracing::Level::WARN
    }));
    errors.and_then(crumbs)
}

fn contains_any(name: &str, fragments: &[&str]) -> bool {
    let lower = name.to_ascii_lowercase();
    fragments.iter().any(|f| lower.contains(f))
}

fn is_secret_header(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    SECRET_HEADERS.contains(&lower.as_str()) || contains_any(&lower, HEADER_FRAGMENTS)
}

/// Replaces the value of each sensitive query parameter in `text`. Works on a full URL, on a
/// bare query string and on a message. Text without `=` stays as it is.
fn scrub_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(eq) = rest.find('=') {
        let (head, tail) = rest.split_at(eq);
        // The parameter name is the run of name characters just before `=`.
        let name_start = head
            .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.'))
            .map_or(0, |i| i + 1);
        let name = &head[name_start..];
        out.push_str(head);
        out.push('=');
        let value_part = &tail[1..];
        let value_end = value_part
            .find(['&', '#', ' ', '"', '\'', '\n'])
            .unwrap_or(value_part.len());
        if !name.is_empty() && contains_any(name, QUERY_FRAGMENTS) {
            out.push_str(FILTERED);
        } else {
            out.push_str(&value_part[..value_end]);
        }
        rest = &value_part[value_end..];
    }
    out.push_str(rest);
    out
}

fn scrub_request(request: &mut Request) {
    request.data = None;
    request.cookies = None;
    request.env.clear();
    if let Some(query) = request.query_string.take() {
        request.query_string = Some(scrub_text(&query));
    }
    for (name, value) in &mut request.headers {
        if is_secret_header(name) {
            FILTERED.clone_into(value);
        }
    }
    if let Some(url) = request.url.take() {
        let cleaned = scrub_text(url.as_str());
        request.url = cleaned.parse().ok().or(Some(url));
    }
}

/// The `before_send` filter: removes tokens and personal data from an event.
///
/// It keeps the message, the exception, the stack trace, the tags and the trace id. It never
/// drops an event; the return type fits the Sentry `before_send` hook.
#[must_use]
pub fn scrub(mut event: Event<'static>) -> Option<Event<'static>> {
    if let Some(request) = event.request.as_mut() {
        scrub_request(request);
    }
    if let Some(user) = event.user.as_mut() {
        user.email = None;
        user.username = None;
        user.ip_address = None;
    }
    event
        .extra
        .retain(|key, _| !contains_any(key, QUERY_FRAGMENTS));
    for crumb in &mut event.breadcrumbs.values {
        if let Some(message) = crumb.message.take() {
            crumb.message = Some(scrub_text(&message));
        }
        for (key, value) in &mut crumb.data {
            if contains_any(key, QUERY_FRAGMENTS) {
                *value = FILTERED.into();
            } else if let Some(text) = value.as_str() {
                *value = scrub_text(text).into();
            }
        }
    }
    Some(event)
}
