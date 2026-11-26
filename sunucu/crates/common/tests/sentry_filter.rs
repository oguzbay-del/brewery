#![cfg(feature = "sentry")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;
use std::sync::Arc;

use common::sentry::{FILTERED, init_with, scrub};
use sentry::protocol::{Breadcrumb, Event, Exception, IpAddress, Request, User};
use sentry::{ClientOptions, Transport, TransportFactory};

fn request_event() -> Event<'static> {
    let mut headers = BTreeMap::new();
    for name in [
        "Authorization",
        "Cookie",
        "Set-Cookie",
        "Proxy-Authorization",
        "X-Api-Key",
        "Idempotency-Key",
        "Paddle-Signature",
        "X-Hub-Signature-256",
        "x-custom-token",
        "accept",
    ] {
        headers.insert(name.to_owned(), "value-secret-1".to_owned());
    }
    headers.insert("accept".to_owned(), "application/json".to_owned());
    let request = Request {
        url: "https://api.example.com/v1/x?token=abc&page=2".parse().ok(),
        data: Some("{\"password\":\"hunter2\"}".to_owned()),
        cookies: Some("sid=abc".to_owned()),
        query_string: Some("token=abc&page=2".to_owned()),
        headers,
        env: BTreeMap::from([("REMOTE_ADDR".to_owned(), "10.0.0.1".to_owned())]),
        ..Default::default()
    };
    let mut event = Event {
        message: Some("boom".to_owned()),
        request: Some(request),
        user: Some(User {
            id: Some("u1".to_owned()),
            email: Some("a@b.example".to_owned()),
            username: Some("alice".to_owned()),
            ip_address: Some(IpAddress::Auto),
            ..Default::default()
        }),
        ..Default::default()
    };
    event.tags.insert("service".to_owned(), "api".to_owned());
    event.exception.values.push(Exception {
        ty: "Error".to_owned(),
        value: Some("failed".to_owned()),
        ..Default::default()
    });
    event.extra.insert("api_key".into(), "k".into());
    event.extra.insert("order".into(), "42".into());
    event.breadcrumbs.values.push(Breadcrumb {
        message: Some("GET /cb?code=zzz&x=1".to_owned()),
        ..Default::default()
    });
    event
}

#[test]
fn secrets_are_removed_and_page_stays() {
    let event = scrub(request_event()).unwrap();
    let dump = format!("{event:?}");
    for secret in [
        "abc",
        "hunter2",
        "sid=",
        "10.0.0.1",
        "zzz",
        "a@b.example",
        "alice",
    ] {
        assert!(!dump.contains(secret), "leaked {secret}");
    }
    let request = event.request.unwrap();
    assert!(request.url.unwrap().as_str().contains("page=2"));
    assert!(request.query_string.unwrap().contains("page=2"));
    assert!(request.data.is_none() && request.cookies.is_none() && request.env.is_empty());
    assert!(
        event.breadcrumbs.values[0]
            .message
            .as_deref()
            .unwrap()
            .contains("x=1")
    );
    assert!(!event.extra.contains_key("api_key"));
    assert!(event.extra.contains_key("order"));
}

#[test]
fn headers_are_filtered() {
    let event = scrub(request_event()).unwrap();
    let headers = event.request.unwrap().headers;
    for (name, value) in &headers {
        if name == "accept" {
            assert_eq!(value, "application/json");
        } else {
            assert_eq!(value, FILTERED, "{name}");
        }
    }
    assert_eq!(headers.len(), 10);
}

#[test]
fn user_pii_is_removed() {
    let user = scrub(request_event()).unwrap().user.unwrap();
    assert!(user.email.is_none() && user.username.is_none() && user.ip_address.is_none());
    assert_eq!(user.id.as_deref(), Some("u1"));
}

#[test]
fn message_exception_and_tags_stay() {
    let event = scrub(request_event()).unwrap();
    assert_eq!(event.message.as_deref(), Some("boom"));
    assert_eq!(event.exception.values[0].value.as_deref(), Some("failed"));
    assert_eq!(event.tags.get("service").map(String::as_str), Some("api"));
}

struct Drop_;
impl Transport for Drop_ {
    fn send_envelope(&self, _envelope: sentry::Envelope) {}
}
impl TransportFactory for Drop_ {
    fn create_transport(&self, _options: &ClientOptions) -> Arc<dyn Transport> {
        Arc::new(Drop_)
    }
}

fn init_with_env(dsn: Option<&str>) -> bool {
    let dsn = dsn.map(str::to_owned);
    let lookup = move |n: &str| if n == "SENTRY_DSN" { dsn.clone() } else { None };
    let base = ClientOptions::new().transport(Drop_);
    init_with(lookup, "test", "0.0.0", base).is_some()
}

#[test]
fn dsn_reading() {
    assert!(!init_with_env(None));
    assert!(!init_with_env(Some("not a dsn")));
    assert!(init_with_env(Some("https://public@example.invalid/1")));
}
