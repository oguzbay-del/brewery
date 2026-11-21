# support

## What it does
- Support requests (T2.29). The record is in the database (`support_requests`, `support_messages`). The support team works in GitHub Issues in a private repository. The two sides sync both ways.
- `sync.rs`: `sync_request` sends the request, the customer messages, edits, deletes and the status to GitHub. It is state based and safe to repeat. `SyncHandler` is the worker consumer of the outbox event `support.sync_requested`. When GitHub is not available, `schedule` writes a retry job to `scheduled_jobs` (one pending job for each request) and `run_due` retries it with back-off. The request is open for the customer all the time.
- `webhook.rs`: `verify` checks `X-Hub-Signature-256`. `store` keeps a delivery once in `external_events` (`ingest` calls both). `GithubHandler` is the worker consumer of `support.github_event`. It applies `/yanit` replies, internal notes, edits, deletes, and Issue close, reopen and label events.
- `reply.rs`: `staff_message` writes a message of the support team (the support panel calls it). `notify_customer` creates the notification `support.reply` (inbox, push, e-mail without the text).
- `github.rs`: the REST client. Settings: `GITHUB_REPO`, `GITHUB_API_URL`, `GITHUB_TOKEN` or `GITHUB_APP_ID`, `GITHUB_APP_INSTALLATION_ID`, `GITHUB_APP_KEY_FILE`, `GITHUB_BOT_LOGIN`. `GITHUB_WEBHOOK_SECRET` is for `ingest`.

- `panel.rs` (T2.30): `active_grant`, `record_action` (support_actions and audit row), `request_correction` and `CorrectionHandler` (worker consumer of `support.correction_requested`; only the worker writes `subscriptions` and `entitlements`), `open_emergency` and `approve_emergency` (two different staff members; the grant lasts 24 hours; the customer gets a priority 1 notification).

## Rules
- `/yanit` is valid only from a person in `staff_users.github_login`. A comment with the marker `<!-- demleme:msg:` or from the bot login is ignored.
- Do not write the name, the e-mail or the workspace name to GitHub. Only the pseudonym of the workspace and the request number go there.
- An internal note (`visibility = 'internal'`) never goes to the customer, to a notification or to an e-mail.
- Code is in English. Notes for Claude (this file, task lists) use ASD-STE100. Other documents are in Turkish.
- Start the commit message with the task number: `T2.x: ...`.

## How to test
- In `sunucu/`, run: `DATABASE_URL=... cargo test -p support` (unit tests). The end-to-end tests are in `crates/worker/tests/support.rs` (PostgreSQL, NATS, mailpit) and `crates/ingest/tests/github.rs` (PostgreSQL). They use a fake GitHub server in the test process.

## Open items
- TODO: the GitHub App token flow is not tested against real GitHub (to be verified).
- TODO: the request status `closed` (14 days after `resolved`) has no job yet.
- TODO: the correction kind `deletion_request` stays pending. A person must decide and the purge call is not connected.
- TODO: the correction kind `subscription_reverify` only queues the outbox event `subscription.reverify_requested`. The store client does not exist yet.
- The device key rotation of the panel uses the key helpers of the `devices` crate. The old key stops working and staff do not see a new key.
- TODO: the grant note in the Issue ("izin var, bitiş: ...") is not written.
- Account deletion (T2.31): `GitHub::scrub_issue` deletes all comments, blanks the title and the body and closes the Issue (trait `IssueScrubber`, used by `worker/src/account.rs`). TODO: real Issue delete needs the GraphQL `deleteIssue` mutation and an administrator right of the App (to be verified).
