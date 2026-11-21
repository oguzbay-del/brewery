# worker

## What it does
- `GET /healthz` and clean shutdown.
- Background work (`background.rs`, T2.15): the relay from the outbox to NATS JetStream (`events::relay`) and the cleanup of `processed_events`. The consumer `subscription-notifications` (`billing::apply::NotificationHandler`, T2.28) applies verified subscription notifications. The consumers of later tasks (`events::consumer::run`) register here. If `DATABASE_URL` (`app_worker`) and `NATS_URL` are not set, only the health endpoint runs.
- The consumer `device-readings` (`devices::apply::ReadingHandler`, T2.25) writes `measurements` with source `device` for the batch that the device is paired with, and writes the event `batch.measurement_added`.
- Notification consumers (T2.24): `notifier-alerts` (`notifier::alert::AlertHandler`) and `notifier-delivery` (`notifier::deliver::DeliveryHandler`). The task `notifier::tick::run` delivers deferred notifications, wakes snoozed ones, fires reminders and sends the daily summary.
- Support consumers (T2.29): `support-sync` (`support::sync::SyncHandler`), `support-github` (`support::webhook::GithubHandler`) and the retry task `support::sync::run_jobs` (every 10 seconds). Settings: `GITHUB_REPO` and the credentials (see `support/CLAUDE.md`).
- Port `PORT` (default 8081).
- Maintenance jobs (`maintenance.rs`, T2.39). A tick runs each hour. `maintenance_runs` lets each job run once for each interval, also with many replicas.
  - Daily: `audit_anchor::write_daily_digest` writes the audit chain heads and the workspace deletion records to `audit-digests/YYYY-MM-DD.json` with an object lock.
  - Weekly: `audit_anchor::verify_against_digests` verifies each chain and compares it with the digests of the last 8 days. A mismatch is a finding (`last_status = 'findings'` and an error log).
  - Daily: `retention::run` clears the payload of unprocessed `external_events` after 30 days and deletes the rows after 13 months.
  - Daily: `retention::purge_soft_deletes` deletes recipes, batches, equipment and user ingredients that were soft-deleted more than 30 days ago (T2.37, `db::maintenance::purge_expired_soft_deletes`). A record that is in use stays soft-deleted (F2.1 decision 9).
  - Settings: `S3_*`, `S3_DIGEST_BUCKET` (separate bucket), `S3_OBJECT_LOCK_MODE` (`governance` or `compliance`). Without `S3_ENDPOINT`, only the retention job runs.

## What it does not touch
- It does not touch calculation code. It accesses the database only with the `app_worker` role and only through `events` and `db`.
- `ingest` owns the intake of external events (webhooks).

## How to test
- In `sunucu/`, run: `cargo test -p worker`. The relay and consumer tests are in `crates/events/tests/` (they need PostgreSQL and NATS).
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`

## Rules
- Code is in English. Notes for Claude (this file, task lists) use ASD-STE100. Other documents are in Turkish.
- Start the commit message with the task number: `T0.x: ...`.
