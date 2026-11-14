# ingest

## What it does
- Intake of external events (webhooks): device, App Store, payment and GitHub sources.
- For now it has only `GET /healthz` and clean shutdown. Later tasks add the source connections.
- Port `PORT` (default 8082).

## What it does not touch
- It does not touch the database or calculation code.
- `worker` owns event processing. This crate only receives the events.

## How to test
- In `sunucu/`, run: `cargo test -p ingest`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`

## Rules
- Code is in English. Notes for Claude (this file, task lists) use ASD-STE100. Other documents are in Turkish.
- Start the commit message with the task number: `T0.x: ...`.
