//! W3C trace context (ADR-0008 rule 11.3). Lives in its own test binary: it installs a
//! thread-local tracing subscriber, which must not race other tests' span callsites.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::sync::Arc;

use api::redis_client::Redis;
use axum::body::Body;
use sqlx::PgPool;
use support::{
    Demo, FakeAuth, config, demo_router, insert_user, insert_workspace, req, send, state_with,
};
use uuid::Uuid;

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn traceparent_joins_the_incoming_trace(pool: PgPool) {
    use opentelemetry::trace::TracerProvider as _;
    use tracing_subscriber::layer::SubscriberExt;
    // The service installs this in `common::otel`; the test builds the same pieces locally.
    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder().build();
    opentelemetry::global::set_text_map_propagator(
        opentelemetry_sdk::propagation::TraceContextPropagator::new(),
    );
    let subscriber = tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("test")));
    let _guard = tracing::subscriber::set_default(subscriber);

    let (user, ws) = (Uuid::now_v7(), Uuid::now_v7());
    insert_user(&pool, user).await;
    insert_workspace(&pool, ws).await;
    let auth = Arc::new(FakeAuth::default());
    auth.add(user, ws, &[]);
    let state = state_with(&pool, Redis::disabled(), auth, config("postgres://unused")).await;
    let app = demo_router(&state, Demo::with_items(1));
    let trace_id = "4bf92f3577b34da6a3ce929d0e0e4736";
    let r = send(
        &app,
        req("GET", &format!("/v1/workspaces/{ws}/trace"), Some(user))
            .header("traceparent", format!("00-{trace_id}-00f067aa0ba902b7-01"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(r.body["trace_id"], trace_id);
    // Without a header a fresh trace id exists anyway.
    let r = send(
        &app,
        req("GET", &format!("/v1/workspaces/{ws}/trace"), Some(user))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(r.body["trace_id"].as_str().unwrap().len(), 32);
}
