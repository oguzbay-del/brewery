# ingest

## What it does
- Intake of external events (webhooks): device, App Store, payment and GitHub sources.
- `GET /healthz` and clean shutdown.
- App Store Server Notifications v2 (T2.28): `POST /v1/webhooks/app-store`. `jws.rs` verifies the JWS: ES256, `x5c` chain to the Apple root (`certs/AppleRootCA-G3.cer`), Apple marker extensions. `apple.rs` maps the notification to a `billing::SubscriptionUpdate`. `APPLE_BUNDLE_ID` limits the accepted app.
- Web payment provider (Paddle, decision O6): `POST /v1/webhooks/web-payment` calls the trait `billing::provider::WebPaymentProvider`. The Paddle adapter is not written yet, so the route answers 503 in production.
- The service stores a verified update in `external_events` and one outbox event in one transaction (role `app_api`). A repeated notification gets 200 `duplicate` and is not stored again.
- Device data (T2.25): `POST /v1/ingest/{key}` and `/v1/ingest/{key}/{format}` (`device.rs`). The key in the address is the credential. An unknown, rotated or deleted key gives `401`. A value out of range gives `422`. The parsers are in the `devices` crate. The body limit is 8 KiB. TODO: rate limit for each device.
- Port `PORT` (default 8082).

## What it does not touch
- It does not touch calculation code. It writes only `external_events` and `outbox`.
- `worker` owns event processing (`billing::apply::NotificationHandler`). This crate only receives and verifies the events.
- Do not use `JwsVerifier::with_roots` in production code. It is for tests with a test root.

## How to test
- In `sunucu/`, run: `DATABASE_URL=... NATS_URL=nats://127.0.0.1:4222 cargo test -p ingest`. `tests/subscription.rs` needs PostgreSQL and NATS. `tests/jws.rs` needs neither. The tests build a test PKI with `rcgen`.
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`

## Rules
- Code is in English. Notes for Claude (this file, task lists) use ASD-STE100. Other documents are in Turkish.
- Start the commit message with the task number: `T0.x: ...`.
