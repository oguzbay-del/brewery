# Status

Each session reads this file first. Keep it to one page. Update it after each task. Write it in ASD-STE100 (root `CLAUDE.md` › Yazım standardı).

**Stage:** 2, core API and data. Stage 1 is closed (ADR-0006).
**Next task:** T2.9 is open. T2.12 and T2.13 are done. The first task of the next session is a one-time refactor:
- Each endpoint module registers its route and its OpenAPI row in its own file. Then parallel tasks do not change `api/src/lib.rs`, `openapi.rs` or `main.rs`.
- Split the implementation notes at the end of ADR-0007 into topic files.

After that, give the chain T2.19 → T2.21 → T2.22 to one agent. Work rules: root `CLAUDE.md` › Token verimliliği.
**Method:** ADR-0003 (Python → JSON vectors → Rust parity test; `make vektor`).

## Environment

- Installed: Rust 1.97, Node 22, Docker client. Start `dockerd` manually at the start of each session.
- In scratchpad/bin: terraform 1.17.2, kind 0.33.0, kubectl 1.37.2, actionlint 1.7.12. If the scratchpad is empty in a new session, download them again (the README files give the steps).
- The tests need PostgreSQL. Start it with `docker compose -f altyapi/compose/compose.yaml up -d --wait postgres` or with `make up`. `make test` sets DATABASE_URL to the compose default.
- The disk allowance is small. In parallel worktrees, set `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`.
- Do not use a shared `CARGO_TARGET_DIR`. Cargo names workspace crates by relative path, so worktrees overwrite the test binaries of other worktrees. Delete the target directory of a worktree when its task is done.
- To build images in this environment, use `make up BUILD_CA_FILE=/root/.ccr/ca-bundle.crt` (TLS proxy).
- CI: GitHub Actions, 6 workflows. iOS runs on GitHub macOS machines. `altyapi.yml` also runs actionlint, kustomize and the kind smoke test. Leave the kind and compose end-to-end checks to CI. Locally, the test of the related crate is sufficient. After a push, check the CI result with `actions_list`.
- Database roles: `app_owner` (owner, NOLOGIN), `app_api`, `app_worker`. `migrate` sets their passwords (`APP_*_PASSWORD`).

## Blockers

- T2.14: if an attacker deletes both the rows and the head of the chain, the check does not find it. An external anchor is necessary: write a periodic digest to the object store. If the canonical field list changes, add a format version.
- T2.16: `EngineClient` is not connected to `AppState` yet (T2.20 and T2.21 connect it). The proto has no separate field for the intermediate values of "Nasıl hesaplandı".
- T2.11: the web redirect code flow and CORS come in stage 3. The Apple refresh token comes in T2.31. Session events are not written to the audit log yet.
- T2.15: the consumer is not registered yet (add `consumer::run` to `worker/src/background.rs`). Each SSE connection opens its own JetStream consumer. If scale makes it necessary, use a shared subscription.
- T2.18: the file limits (25 MiB; jpeg, png, heic, webp, pdf, text) are a product proposal, to be verified. No worker job deletes the object bytes of deleted or incomplete uploads. The object store for k8s and Terraform comes in T2.32. There is no Azure Blob implementation (the interface is ready).
- There is no seed data for tax rates. Rows start as `unverified`. Load them after a check against the official source (T2.8).
- T2.22: a batch starts only from an `approved` version (also in Home mode). To start a batch from a draft, publish the version first.
- T2.10 decisions (they apply until someone objects):
  - An unknown path and an incorrect method give 404 (the ADR code list has no 405).
  - Idempotency keeps only 2xx responses.
  - Rate limits: 300 requests per minute per IP, 600 per user, 1200 per workspace.
  - `/v1/meta/fields` has `max-age=3600`. The request ID is UUIDv7.
  - Each new endpoint adds a row to the `OPERATIONS` table in `api/src/openapi.rs`.
  - The `/internal` listener and SSE come in later tasks.
- T2.30: only `app_worker` writes `subscriptions` and `entitlements`. A decision is necessary about the path for manual corrections by support.
- T2.17: the field limits are a product proposal (`verify: true`, to be verified). The catalog uses exact unit definitions. The engine uses the rounded constants of BeerSmith (difference approximately 1e-7). The decision about alignment is open.
- T2.28: web payment waits for decision O6 (the App Store part has no blocker). Other open items are in ADR-0002 (legal and cost decisions for Terraform, cloud account O4, AppIcon).

## Recent decisions

- 2026-11-14 (F2.1 decisions, Oğuz):
  - Permanent delete: change an immutable reference to a snapshot, then delete the original record.
  - Audit log: for a personal field, write only "changed" (no digest of the value). The legal opinion is to be verified.
  - A device measurement without a batch goes to the tank only if the device is connected to a tank. Reject data from a device that is not connected.
  - Business mode: build the infrastructure for mandatory retention (retention end, legal hold) now. Keep the periods in configuration as "to be verified". Stage 6 sets them.
- 2026-11-14 (F2.1 decisions, Claude):
  - Only the worker deletes a workspace permanently, through a narrow function (`app_api` loses the DELETE privilege).
  - After a delete, keep a small deletion record and the last digest of the audit chain outside the workspace.
  - Account-level events go to an account log without a chain.
  - Lot-specific raw material values come in stage 6.
- T2.8: add the steps of the tax chain. The effective date is the first day of the period. A change inside a period gives an error (split the period).
- 2026-11-11: full development. The interviews and the golden test are the gate of stage 5.
- ADR-0001: repository layout. ADR-0002: stage 0 closure and deviations.
- F1: Fable did an independent check of the formulas. We processed 22 findings and 14 decisions. Floating-point rule: gravity thresholds use integer units (`round(SG*100000)`); continuous quantities use 1e-9 (formuller/00-OKU-BENI.md).
- Open a new session for each stage.
- ADR-0006: stage 1 closed (2026-11-13). T1.17 engine gaps.
- ADR-0008: API rules. Workspace in the path, RFC 9457 errors, cursor pagination, Merge Patch, ETag/If-Match (optimistic, field_lww, append), Idempotency-Key, SSE events without content, `/internal` on a separate port, oasdiff check for breaking changes.
- ADR-0009: identity, session, authorization.
  - Apple and Google tokens: JWKS and a single-use nonce.
  - Opaque session: digest in PostgreSQL, Redis cache.
  - Web: cookie and CSRF. iOS: Bearer.
  - Passkey as second factor (mandatory for administrators).
  - Shared device: PIN only. Face ID does not identify the person (the plan and the concept are updated).
  - The permission list is in code; `api::authz` is the only place.
  - Istio ambient mTLS. Secrets come from the environment or from `*_FILE`.
- ADR-0003: engine parity; one module per section; engine functions use the units of the specification; money uses `rust_decimal`.

## Stage 2 (active)

T2.0 task list · T2.1 data model (ADR-0007) · T2.2 API rules (ADR-0008) · T2.3 identity and authorization (ADR-0009) · T2.4 `db` crate, sqlx 0.9, migration service and Job · T2.17 field, unit and text catalog (`sozlesme/katalog`, `make gen`, `common::catalog`) · T2.5 core migrations (23 tables, FORCE RLS, three roles, SECURITY DEFINER functions before login, RLS scan) · T2.6 Home MVP domain migrations (0007–0013, `secure_workspace_table`, partitioned measurements, immutable version, scope trigger) · T2.10 API core (context and `SET LOCAL`, RFC 9457, idempotency, ETag/If-Match, cursor, Redis rate limit fail-open, OTel, OpenAPI and oasdiff, `/v1/meta/fields`) · T2.7 Business and tax tables (0014–0015) · T2.14 audit chain (RFC 8785, `audit-verify`) · T2.16 engine client (`engine-client` crate, Redis and `calc_results` cache, error mapping) · T2.15 outbox → NATS relay, consumer framework, SSE · T2.18 object store (`storage` crate, SigV4, SeaweedFS, file endpoints) · T2.11 login (Apple/Google JWKS, email link, opaque session, cookie and CSRF, `fake-oidc`) · T2.8 tax chain (09: 459 parity cases) · F2.1 report · T2.13 authorization (`api::authz`, standard roles from the matrix, scope, cost field removal, roles and members, last administrator protection, 7-day invitation, Home sharing, permission cache in Redis) · T2.12 passkey (webauthn-rs 0.6.2), step-up for protected actions, shared device (6-digit Argon2id PIN, lock for 15 minutes after 5 failures, migration 0090). 421 tests passed before T2.12 and T2.13.

Order: decisions → data and F2.1 → API core → domain endpoints → support → cluster and end-to-end → F2.2 → gate. The data model also contains the Business tables. Business endpoints come in stage 6.

## Stage 1 (closed)

T1.0 ADR-0003 · T1.1–T1.10 ports · T1.14 calculations without a reference · T1.12 Fable review (5 reports) · T1.15 finding fixes · T1.16 common model, error type, names (ADR-0004) · T1.11 8 gRPC services (Tool, Recipe, Mash, Water, Yeast, Fermentation, Brewery, Engine), endpoint tests with vectors and an end-to-end test over TCP; 203 tests pass in the workspace · T1.17 engine gaps · T1.13 gate (ADR-0006). 10 269 parity cases, all equal.

## Stage 0 (closed)

T0.1 repository layout · T0.2 Rust skeleton · T0.3 Docker Compose · T0.4 API contract · T0.5 web · T0.6 iOS · T0.7 Kubernetes and kind · T0.8 Terraform · T0.9 CI · T0.10 module documents · T0.11 gate · T0.12 dark high contrast · F1, F1.1, F1.2 formula check
