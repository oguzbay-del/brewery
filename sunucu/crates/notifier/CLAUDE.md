# notifier

## What it does
- Notifications (T2.24). `create::create` inserts a notification and writes the outbox event `notification.created`. `create::recipients` lists the members that get an alert.
- Alert rules (`alert.rs`). `AlertHandler` is the worker consumer of `batch.measurement_added`. It reads the newest measurement of each kind, applies each enabled rule (`Condition`) and creates one notification for each recipient. The dedupe key holds the rule, the cooldown window and the member, so a repeated event does not repeat the notification.
- Delivery (`deliver.rs`). `DeliveryHandler` is the worker consumer of `notification.created`. `deliver_one` applies the preferences of the recipient: quiet hours (SQL function `notify_quiet_until`), daily summary, push (`apns.rs`) and e-mail (`smtp.rs`, only the kinds in `EMAIL_KINDS`).
- Periodic work (`tick.rs`). `run_due` calls `notify_due_work` and then delivers deferred notifications, wakes snoozed notifications, fires reminders and sends the daily summary. The worker runs it every 30 seconds.
- Priority: 1 critical (ignores quiet hours, time-sensitive push), 2 normal, 3 low (summary only when the summary is on).
- Settings (`Deliverer::from_env`): `APNS_URL`, `APNS_TOPIC`, `APNS_KEY_FILE`, `APNS_KEY_ID`, `APNS_TEAM_ID`, `SMTP_HOST`, `SMTP_PORT`, `SMTP_FROM`. A channel without settings is off.

## What it does not touch
- It has no HTTP routes. The inbox, the actions, the preferences and the rules are in `api/src/notifications.rs`.
- It does not write `subscriptions`, `entitlements` or support tables.

## How to test
- In `sunucu/`, run: `DATABASE_URL=... cargo test -p notifier` (unit tests). The end-to-end test is `crates/worker/tests/notifications.rs`. It needs PostgreSQL and NATS. It uses a fake APNs server in the test process.

## Rules
- Code is in English. Notes for Claude (this file, task lists) use ASD-STE100. Other documents are in Turkish.
- Do not store system text in the database. Store the text key and the parameters.
- Start the commit message with the task number: `T2.x: ...`.

## Open items
- TODO: the APNs provider token and the HTTP/2 path are not tested against the real APNs (to be verified).
- TODO: the alert scope `vessel` has no effect until the vessel tables exist (T2.7 tables are there, the match is not written).
