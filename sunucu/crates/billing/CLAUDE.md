# billing

## What it does
- Subscription and entitlement logic (T2.28). `SubscriptionUpdate` is the verified, normalized change of a subscription (source: App Store or web).
- `apply::store` (role `app_api`, called by `ingest`): inserts `external_events` (unique by source and delivery id) and one outbox event. It returns `Duplicate` for a repeated notification.
- `apply::NotificationHandler` (role `app_worker`, consumer `subscription-notifications`): upserts `subscriptions` and the entitlement `paid_features`, marks `external_events.processed_at`, and writes the outbox event `subscription.changed`.
- `access::access` computes the access level: full during the trial (one month from `workspaces.created_at`) or with a valid entitlement; otherwise read-only. Export is never blocked.
- `provider::WebPaymentProvider` is the interface for the web payment provider (Paddle, decision O6). `FakeProvider` is for tests. The Paddle adapter is not written yet.

## What it does not touch
- It does not verify the App Store JWS. `ingest` does.
- It has no HTTP code. The read endpoint is in `api/src/subscription.rs`.
- The client sets the App Store `appAccountToken` to the workspace id at purchase. A notification without it goes to the dead letter (permanent error).

## How to test
- In `sunucu/`, run: `cargo test -p billing`. The end-to-end tests are in `crates/ingest/tests/subscription.rs` (PostgreSQL and NATS).

## Rules
- Do not make the API role write `subscriptions` or `entitlements`. Only `app_worker` writes them.
- A change older than `subscriptions.last_verified_at` is skipped. Do not remove this check. The store can deliver out of order.
- Code is in English. Notes for Claude (this file, task lists) use ASD-STE100. Other documents are in Turkish.
- Start the commit message with the task number: `T2.x: ...`.
