//! Request id, W3C trace context and the request span (ADR-0008 rule 11.3).

use axum::{
    extract::Request,
    http::{HeaderMap, HeaderName, HeaderValue, header},
    middleware::Next,
    response::Response,
};
use opentelemetry::{
    propagation::Extractor,
    trace::{TraceContextExt, TraceId},
};
use tracing::Instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::error::REQUEST_ID;

/// Request id header (accepted from the client, returned on every response).
pub static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

/// Longest client-supplied request id that is accepted.
const MAX_REQUEST_ID_LEN: usize = 64;

struct HeaderExtractor<'a>(&'a HeaderMap);

impl Extractor for HeaderExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(HeaderName::as_str).collect()
    }
}

fn valid_request_id(raw: &str) -> bool {
    !raw.is_empty()
        && raw.len() <= MAX_REQUEST_ID_LEN
        && raw
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}

/// The trace id of the current span, if a tracer provider is installed.
#[must_use]
pub fn current_trace_id() -> Option<String> {
    let context = tracing::Span::current().context();
    let span = context.span();
    let id = span.span_context().trace_id();
    (id != TraceId::INVALID).then(|| id.to_string())
}

/// Middleware: takes or makes the request id, opens the request span (child of an incoming
/// `traceparent`), and returns the id in `X-Request-Id`. Also makes `/v1` responses
/// `Cache-Control: private, no-cache` unless the handler chose otherwise (rule 5.2).
pub async fn layer(req: Request, next: Next) -> Response {
    let request_id = req
        .headers()
        .get(&X_REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .filter(|v| valid_request_id(v))
        .map_or_else(|| uuid::Uuid::now_v7().simple().to_string(), str::to_owned);
    let path = req.uri().path().to_owned();

    let span = tracing::info_span!(
        "http_request",
        request_id = %request_id,
        method = %req.method(),
        path = %path,
        status = tracing::field::Empty,
        trace_id = tracing::field::Empty,
    );
    let parent = opentelemetry::global::get_text_map_propagator(|p| {
        p.extract(&HeaderExtractor(req.headers()))
    });
    // An unusable parent only means a fresh trace.
    let _ = span.set_parent(parent);
    {
        let _enter = span.enter();
        if let Some(trace_id) = current_trace_id() {
            span.record("trace_id", trace_id.as_str());
        }
    }

    let mut response = REQUEST_ID
        .scope(request_id.clone(), next.run(req).instrument(span.clone()))
        .await;
    span.record("status", response.status().as_u16());
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(X_REQUEST_ID.clone(), value);
    }
    if path.starts_with("/v1") && !response.headers().contains_key(header::CACHE_CONTROL) {
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, no-cache"),
        );
    }
    tracing::info!(parent: &span, "request finished");
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_id_validation() {
        assert!(valid_request_id("01JC-abc_1.2"));
        assert!(!valid_request_id(""));
        assert!(!valid_request_id("has space"));
        assert!(!valid_request_id(&"a".repeat(65)));
    }
}
