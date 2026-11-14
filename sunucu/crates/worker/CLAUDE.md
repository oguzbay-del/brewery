# worker

## What it does
- `GET /healthz` and clean shutdown.
- Background work (`background.rs`, T2.15): the relay from the outbox to NATS JetStream (`events::relay`) and the cleanup of `processed_events`. The consumers of later tasks (`events::consumer::run`) register here. If `DATABASE_URL` (`app_worker`) and `NATS_URL` are not set, only the health endpoint runs.
- Port `PORT` (default 8081).

## What it does not touch
- It does not touch calculation code. It accesses the database only with the `app_worker` role and only through `events` and `db`.
- `ingest` owns the intake of external events (webhooks).

## How to test
- In `sunucu/`, run: `cargo test -p worker`. The relay and consumer tests are in `crates/events/tests/` (they need PostgreSQL and NATS).
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`

## Rules
- Code is in English. Notes for Claude (this file, task lists) use ASD-STE100. Other documents are in Turkish.
- Start the commit message with the task number: `T0.x: ...`.
