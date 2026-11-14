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
- Port `PORT` (default 8080).

## What it does not touch
- It has no formulas. When it needs a calculation, it requests it from `engine-service` over gRPC.
- The audit log read endpoint (T2.14) is not here. It comes in later tasks.
- The object store client is in the `storage` crate. Here are only the endpoints and the validation.
- It does not add migrations (exceptions: T2.18 `0070`, T2.12 `0090`). Table decisions: `adr/0007-veri-modeli.md`.

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
