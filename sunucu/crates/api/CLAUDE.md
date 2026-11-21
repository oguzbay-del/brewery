# api

## What it does
- Core REST API: the middleware layers of ADR-0008 (error envelope, request context, idempotency key, ETag/If-Match, cursor pagination, rate limit, request id and trace, version compatibility) and OpenAPI generation. Details: `README.md`.
- Endpoints: `/healthz`, `/readyz`, `/v1/version`, `/v1/meta/fields`, `/v1/meta/client-requirements`.
- Sign-in and session (`auth` module, ADR-0009, T2.11): `/v1/auth/*`, `/v1/me/session(s)`, `/v1/me/identities`, `POST /v1/workspaces`.
- Authorization (`authz.rs`, `auth/access.rs`, T2.13): endpoints for roles, members, assignments, invitations and Home sharing.
- `GET /v1/workspaces/{ws}/events` (SSE, `events.rs`, T2.15): each connection reads from NATS JetStream with its own ordered consumer. The API checks membership every 30 seconds. If `NATS_URL` is not set, it returns 503.
- Domain endpoints come in later tasks. File endpoints (`files.rs`: attachments, scanned-documents; T2.18).
- Passkey and shared device (T2.12): passkey and second step (`auth/passkey.rs`, `auth/step_up.rs`, T2.12): `/v1/me/passkeys*`, `/v1/me/step-up/*`. Shared device (`auth/shared_device.rs`, `auth/pin.rs`, T2.12): `/v1/workspaces/{ws}/shared-devices*`, `.../me/pin`, `.../members/{m}/pin`, and `/v1/shared-devices/{id}/{members,switch,lock}` with the device key.
- It accesses the database through the `db` crate (role `app_api`). Each request context runs in a transaction where `RequestContext::begin` did `SET LOCAL`.
- Engine connection (T2.16): `src/engine.rs`. It has a tonic channel from `ENGINE_URL`, the engine version stamp, a Redis cache keyed by the input digest (canonical protobuf bytes + SHA-256), the `calc_results` record, and the mapping of the engine status to the error envelope. An endpoint that needs a calculation uses `EngineClient::calculate`. The output is base64 protobuf in `calc_results.output`.
- Subscription (`subscription.rs`, T2.28): `GET /v1/workspaces/{ws}/subscription` reads the access state. `require_paid_write(conn, ws)` is the guard for a write to a paid feature: it returns `403 entitlement_required` one month after workspace creation when no entitlement is valid. Do not call it in export or read endpoints. The API does not write `subscriptions` or `entitlements`.
- Recipes (`recipes.rs`, `recipe_org.rs`, T2.21): recipe and version endpoints, calculation with the engine, publish (the version becomes immutable and keeps `calc_summary`), scaling preview, comparison, folders, tags, favorites, the edit session (soft lock, events through the outbox). A stale `If-Match` on a draft gives `412` with `current.conflicting_fields`.
- Batches (`batches.rs`, `brew_day.rs`, T2.22): batch from an `approved` version, steps, actual amounts with deviation, measurements with a client UUID (safe to repeat), stage transitions, summary, calendar, backward plan, and the signed brew day package. The package key is `BREW_PACKAGE_KEY` (or `_FILE`) and `BREW_PACKAGE_KEY_ID`. Without a key, the package endpoints answer `503`.
- Stock (`stock.rs`, T2.23): lots, the movement ledger (`receive`, `consume`, `adjust`, `count`), totals from the ledger, `negative_stock` warning, deduction for a batch (safe to repeat, oldest lot first), and the shopping list from a recipe version minus stock. Write permission is `stock.move`. The module does not add migrations (tables are in `0010` and `0100`).
- Search (`search.rs`, T2.27): `GET /v1/workspaces/{ws}/search?q=&types=&limit=`. Full-text over recipes, batches, ingredients and stock lots (`search` columns, `simple`). Each branch filters on `workspace_id`; library ingredients are included. Words match as prefix; the whole text also runs as a web search query.
- Recently deleted (`trash.rs`, T2.37): `GET .../trash` lists the soft-deleted recipes, batches, ingredients and equipment of the last 30 days. `POST .../trash/{kind}/{id}/restore` restores one. After 30 days restore answers `409`. Each kind needs the write permission of its own endpoints.
- Devices (`devices.rs`, T2.25): `/v1/workspaces/{ws}/devices*` (register, pair with a batch, rotate key, delete). The key is shown once on create and rotate. Only its SHA-256 digest is stored. Rotation needs the second step when the auth services run, and writes the audit event `device_key.renew`. `INGEST_PUBLIC_URL` (optional) gives the full data address in the answer.
- Notifications (`notifications.rs`, T2.24): inbox, actions (approve, postpone, ignore), notification preferences (quiet hours, daily summary) and alert rules. The worker creates and delivers notifications (`notifier` crate).
- Support requests (`support.rs`, T2.29): open a request, messages, edit, retract, resolve. Each change writes the outbox event `support.sync_requested`; the worker syncs with the issue tracker (`support` crate). The customer sees only messages with visibility `customer` and only own requests. Attachments use the file endpoints with `record_kind` `support_message`.
- Support access (`support_access.rs`, T2.30): the customer gives a grant (72 hours at the most), revokes it and reads the access history. Permission `support.grant`.
- Support panel (`internal.rs`, T2.30): `/internal/v1/*`. These routes are not in the OpenAPI document and not on the public gateway. `INTERNAL_PROXY_TOKEN` is the shared token of the OIDC reverse proxy; the proxy sets `X-Auth-Request-Email`. Without the token, the routes answer 503. Each operation needs a `reason` and writes `support_actions` and an audit row. Content (`content/*`) needs a valid grant (403 otherwise, and the attempt is logged); it is read in a read-only transaction as `support`. A correction of a subscription or entitlement is a request (`support_corrections`); the worker applies it. Emergency access needs two different approvals (the second from `support_admin`) and tells the customer.
- Import and export (`import_export.rs`, T2.26): `POST .../imports` takes a file body (BeerXML, BeerJSON, Brewfather JSON; 8 MiB limit) and parses it with the `interchange` crate. `POST .../imports/{id}/apply` creates the recipes (a draft each), matches equipment and ingredients by name, calculates with the engine and returns the source values and the calculated values side by side. The decisions go to `import_sources.mapping_decisions`. `POST .../exports` queues a job in `export_jobs` (the worker builds the file). One full export (`scope` `workspace`) for each user in 24 hours; recipe and batch exports have no limit. `GET .../exports/{id}` gives a presigned download link. Permissions: `recipe.edit` (import), `reports.export` (export). `.bsmx` is open (needs a sample file).
- Port `PORT` (default 8080).

## What it does not touch
- It has no formulas. When it needs a calculation, it requests it from `engine-service` over gRPC.
- The audit log read endpoint (T2.14) is not here. It comes in later tasks.
- The object store client is in the `storage` crate. Here are only the endpoints and the validation.
- It does not add migrations (exceptions: T2.18 `0070`, T2.12 `0090`, T2.28 `0130`, T2.21 `0140`, T2.22 `0141`, T2.40 `0190`, T2.37 `0191`, T2.25 `0200`, T2.26 `0210` in `db` migrations). Table decisions: `adr/0007-veri-modeli.md`.

## How to test
- In `sunucu/`, run: `DATABASE_URL=... cargo test -p api`. You need PostgreSQL and Redis. `tests/sse.rs` needs NATS JetStream. `tests/files.rs` needs the object store. `tests/auth.rs` needs mailpit. Start them with `docker compose -f altyapi/compose/compose.yaml up -d --wait postgres redis nats objectstore mailpit`.
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- Contract: `bash sozlesme/scripts/check.sh`

## Rules
- Do not edit the OpenAPI file in `sozlesme/openapi/` by hand. Generate it again with `gen-openapi` (`bash sozlesme/scripts/gen.sh`).
- A new endpoint adds a row to the `OPERATIONS` table in `openapi.rs` (error codes, `x-permission`, `x-concurrency`). A test finds a missing row.
- Always return an error with `ApiError`. Do not build your own JSON format. A Redis error does not fail the request (`redis_client` returns `None`; the rate limit fails open).
- Code is in English. Notes for Claude (this file, task lists) use ASD-STE100. Other documents are in Turkish.
- Start the commit message with the task number: `T2.x: ...`.

## Passkey settings (T2.12)
- `WEBAUTHN_RP_ID` (default: the domain of `WEB_BASE_URL`), `WEBAUTHN_RP_NAME` (`Demleme`), `WEBAUTHN_ORIGINS` (default: the web origins and `https://<rp id>`; the iOS origin). If the RP id and the origin do not match, the passkey endpoints return `503`.
- A protected action calls `AuthServices::require_step_up(&ctx, ProtectedAction::...)` (`auth/step_up.rs`). This is separate from the allow list.
