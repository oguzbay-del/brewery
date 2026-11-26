# Status

Each session reads this file first. Keep it to one page. Update it after each task. Write it in ASD-STE100 (root `CLAUDE.md` › Yazım standardı).

**Stage:** 2, core API and data. Stage 1 is closed (ADR-0006).
**Next task (new session):** stage 2 tasks T2.23–T2.30, T2.37 and T2.40 are done (2026-11-21). Read the open stage 2 rows in `gelistirme/ISLER.md` for the next task.
- Done (2026-11-21): the `.bsmx` import (format `beersmith`). The sample fixture comes from `cadwallion/brewscribe` (code read only, not copied).
- Done on 2026-11-20: T2.19, T2.20, T2.21, T2.22, T2.28, T2.38, T2.39. Full api tests: 168 passed (with mailpit, NATS and the object store running).
- 2026-11-14: the module registry is done. A new endpoint module adds one line to `api/src/modules.rs` only (`sunucu/CLAUDE.md` gives the steps). The ADR-0007 implementation notes are in `adr/0007-notlar/`.
- 2026-11-14: T2.9 is done (migrations 0100–0102). New decision items 9–12 are in the F2.1 report › Karar gerektiren konular.
- 2026-11-22: ChatGPT is a second contributor (owner decision). The shared board is the GitHub labels. Before you start a task, read the open issues and PRs. Review a ChatGPT PR before the next task. ChatGPT issues: #193 (G2), #194 (T2.36 draft), #195 (T2.33 scenarios), #196 (P2). Claude issues: #197 (T2.32), #198 (T2.31). Rules: `marketing-site/CHATGPT-SCOPE.md`.
- 2026-11-22: Claude merged PR #201 (P2 draft; #196 closed). Claude merged PR #199 (G2 report, 9 product-name matches) and PR #200 (T2.33 scenarios; Claude fixed the path names and the 503 precondition). The message channel is active (`coord/*` branches). T2.31 (#198) moved to ChatGPT (owner request: save tokens); Claude reviews and tests the PR. T2.32 (#197) also moved to ChatGPT in six parts: #203 → #204 → #205 → #206 in order; #207 (mTLS design note) and #208 (Terraform) in parallel. Claude reviews, tests and merges each PR. Claude has no own product task now. 2026-11-22: #209 (mTLS note) is merged. ADR-0009 already selects Istio ambient; #207 step 2 implements it after #206. The channel test is closed. Claude cannot delete branches from this environment (HTTP 403). ChatGPT deletes closed `coord/*` branches (owner decision, 2026-11-22). T2.31 is split into #212–#217 (order: #212, then #213 and #214, then #215 and #216; #217 now). ChatGPT has no local Rust build: CI `sunucu` is the test runner, and Claude runs `gen.sh` before a merge when the OpenAPI file changes. Owner decisions (Oğuz, 2026-11-22): the grace period for account deletion is 30 days (`ACCOUNT_DELETION_GRACE_DAYS` default 30). GitHub Issues of a deleted account: remove the content and close the issue (no delete). #220 (BeerXML) and #222 (yeast count deduction) moved to ChatGPT. #221 is closed: the equipment efficiency fields are already done (migration 0190, test `equipment_split.rs`); the T2.38 note below is out of date. Migration number 0230 is free again.
- 2026-11-23: Claude merged PR #226 (T2.32b). #205 is ready. Claude sent the database contract for #214 (migration 0221; 0222 is reserved for #215) in `coord/t2-31b-214-worker-contract`. Claude regenerated the contract on PR #227. The T2.33 test code moved to ChatGPT (#229).
- 2026-11-23: Claude merged the ChatGPT PRs #210 (T2.32a), #211 (T2.32f), #218 (T2.31f) and #219 (T2.31a). The CI was green on all four heads. Claude did a code review; Claude did not run the tests locally. Claude also merged PR #224 (yeast count deduction, #222) and PR #225 (BeerXML 1.0, #220) after code review; CI was green. Note: the BeerXML reader now rejects a file without a required field (spec #220 item 3). Some exporters omit fields, so watch the import errors in beta.
- 2026-11-24 (owner decisions): ChatGPT has permission classes A, B and C (root `CLAUDE.md`, `marketing-site/CHATGPT-SCOPE.md` › Permission classes). Class B is the same as class C until the CI guard for protected paths is merged (#256). Vercel is not used; the W12 guard is cleared. mTLS uses Istio ambient (#207 step 2 after #206; Claude writes its scope). #216: edit and close the issues. #146: ChatGPT prepares the domain setup; Oğuz applies DNS. #255: ChatGPT deletes the branches of merged or closed PRs. #145: the logo work continues. O8: Oğuz reviews `gelistirme/raporlar/P2-kademe-matrisi.md` now.
- 2026-11-24: new ChatGPT tasks: #250 (T3.a API client), #251 (T3.b text catalog), #252 (T3.c app shell), #253 (local guide), #254 (API summary), #255 (branches), #256 (CI guards). #205 scope now includes `veri.yaml` (Redis: `api` only).
- 2026-11-24: Claude merged #257 (guard step 1, with fixes), #227 (T2.31b, contract regenerated), #228 (T2.31c, migration 0221) and #230 (T2.33). Local tests: api `account_deletion`, db, worker (needs NATS and mailpit), e2e (skip without `E2E_BASE_URL`), fmt and clippy are clean. #215 and #216 can start now. Open follow-ups: in T2.31b, `revoke_all` runs after the commit, so a failure leaves the sessions active until the next call; a user with no workspace gets no audit row. T2.33 scenario 6 is ignored (no HTTP seam). The oasdiff check is not installed locally (#256 step 4).
- 2026-11-25 (Oğuz): #266 (T2.41) moved from an earlier assignee to ChatGPT. No branch and no PR existed. The task is class C: Claude reviews, tests and merges.
- 2026-11-25: Claude merged #281 (CI guard step 2). Claude closed #257 (step 1 is already on main) and #252 (#262 is merged). ChatGPT merged #262 itself, but `web/` is class C until #256 is done. Changes requested: #260 and #261 (prettier; #261 also needs a Vite fix), #278 (`kind-test.sh` lost its executable bit), #280 (the API summary is not up to date). Unblocked: #215 (Claude gave the Apple token contract; Oğuz can confirm the setting names), #194. Still blocked: #216 (waits for #215), #206 (waits for #278), #207 (waits for #206), #255 (Oğuz must delete the branches).
- 2026-11-25: new ChatGPT issues: #282–#287 (T2.42–T2.47; migrations 0240 and 0241). #283, #286 and #287 change `maintenance.rs`, so do them in that order.
- 2026-11-25: P1 (#335) and K1 (#333, migration 0250) moved to ChatGPT. Claude reviews, tests and merges. K2 follows K1 and P1.
- 2026-11-25 (Oğuz): #146 clearance reviewed; GRISTORA stays. O7: the owner buys `gristora.com`. O4: the owner opens the Google Cloud organization and billing. G7 (DNS, email, redirect URIs) can start when the domain exists.
- 2026-11-25 (Oğuz): development loop v2 runs only in the cloud. Rules: `gelistirme/dongu/README.md`; Codex reads `AGENTS.md`; ADR-0011. The Claude loop session is `session_01DL3udGsAD478rdQFh8h2an` (Sonnet, repository attached). The hourly task `trig_012BVZoxD9sjwKa7SRopZZbt` (`:17` UTC) fires into it. The old tasks "ChatGPT iş atama A/B" had no repository and did nothing (GitHub 403); the owner must disable them. Stop switch: label `loop:pause` on issue #344 (LOOP-CONTROL). Pilot issues: #345 (Claude), #346 (Codex). Owner steps: README › 9 (Codex cloud environment, ChatGPT tasks, optional Claude GitHub trigger).
- 2026-11-25 (main session): pilots #345 (PR #350) and #346 (PR #347) are merged. The Codex review path works (Codex reviewed #350 with no findings). Owner instructions: README › 7a (finish work in the same run, no wait for "devam") and › 7b (lowest sufficient model). Claude unblocked #285 (dependency `sentry` =0.49.5 approved) and #314 (scope: catalog group `ui`). G1 is done: `adr/0012-urun-markasi.md`. New ChatGPT issues: #351 (G4, server texts) and #352 (G5a, web title, favicon, manifest). G3 (Claude) changes the design system tokens by ADR-0012 › 3. The header logo (rest of G5) comes after #314.
- 2026-11-25: Claude merged #337 (K1, migration 0250). Fix requests (round 1): #336 (`Cargo.lock`, two tests), #341 (two race windows; follow-up #355), #338 (trial test dates, read-only gate after the trial, e2e fixture). Codex tasks still answer "create an environment": the owner must publish the Codex environment. Codex reviews work.
- Owner decision (Oğuz, 2026-11-25, #338): in Home mode, the preset roles (admin, editor, viewer) and recipe sharing are off. The Team module stays Business only. Follow-up for Claude: update plan › Yetki matrisi and plan › Mod ayrımı, the concept and the design system (Home has no role or share screens).
- 2026-11-25 (Oğuz, decisions): (1) the Codex task path is off; fix requests go without `@codex` (README › 6.4 note). (2) O5: the owner turns on "Automatically delete head branches". (3) Trial: Home 3 months, Business 1 month (issue #357, after #356). (4) Prices: ChatGPT does competitor research (#358), then Claude brings 2–3 options to the owner. (5) O10: the owner opens the Paddle account now. Contract check: a new enum value in a response is info, not error (ADR-0008 rule 8.2, `breaking.sh`, in #356).
- Next for Claude: review the #256 steps and the T3 PRs (#250–#252) when they open.
- 2026-11-26 (Oğuz): ChatGPT has no limit now. Claude does the open ChatGPT product issues with Sonnet subagents. Marketing issues stay with ChatGPT.
- 2026-11-26: merged to `main`: #362 (T2.44 follow-up, #355), #359 (T2.46), G5a (#352), T3.d (#314, catalog group `ui`), #357 (trial: Home 3 months, Business 1 month; `billing::access::trial_end`), G4 (#351; also the BeerJSON author and the notifier sender), T2.47 (#285; `sentry` 0.49.5 pinned, CI runs `--features sentry`), T2.31d (#215; migration 0222, new crate `apple-auth`), T2.32d (#206).
- T2.32d: the edge uses the Gateway API, with Istio 1.28.0 in kind. Do not use ingress-nginx; the project was retired in March 2026. The oauth2-proxy alpha config owns the provider and the listen address. The local kind test passed (all edge checks). Open: the `test` and `canli` overlays need the real host, the Secret `demleme-oauth2-proxy` and the production `gatewayClassName`. #207 (mTLS, Istio ambient) can start now.
- T2.31d open items: the AAD for the ciphertext (`TODO` in `apple-auth`); the Apple answer `invalid_grant` counts as a failure (to be verified with the real Apple service). Three attempts run on three hourly maintenance ticks.
- Local kind in this environment: the kind node cannot pull images through the TLS proxy. Pull on the host, then use `docker save | kind load image-archive /dev/stdin`.
- 2026-11-26: T2.31e (#216) is merged; T2.31 (#198) is done. Migration 0251. The purge saves the Issue numbers, then the worker replaces the Issue title, the body and the customer comments with a fixed text and closes the Issue. A GitHub failure is retried on each hourly tick (no limit). A Sonnet subagent wrote the code; Claude reviewed it and ran the worker, support and db tests.
- 2026-11-26 (Oğuz): a second Claude account works as the contributor "Claude-2". Rules: `gelistirme/CLAUDE2-SCOPE.md`; label `assigned-claude-2`; branches `claude2/...`. Claude-2 does not merge. Claude reviews, tests and merges its PRs before the next task. Issues: #363 (K2), #364 (F2.2 report), #365 (K3, after #363). P1 was merged in #356; the ISLER row is now done.
- Next for Claude: #207 (mTLS, a Sonnet subagent is on it). Blocked: #255 (Oğuz), #316 and #326 (P3).
**Method:** ADR-0003 (Python → JSON vectors → Rust parity test; `make vektor`).

## Environment

- Installed: Rust 1.97, Node 22, Docker client. Start `dockerd` manually at the start of each session.
- In scratchpad/bin: terraform 1.17.2, kind 0.33.0, kubectl 1.37.2, actionlint 1.7.12. If the scratchpad is empty in a new session, download them again (the README files give the steps).
- The tests need PostgreSQL. Start it with `docker compose -f altyapi/compose/compose.yaml up -d --wait postgres` or with `make up`. `make test` sets DATABASE_URL to the compose default (`postgres://demleme:demleme-yerel@127.0.0.1:5432/demleme`). If Docker Hub gives 429, use the local PostgreSQL 16 (`pg_ctlcluster`) and `redis-server`.
- The disk allowance is small. In parallel worktrees, set `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`.
- Do not use a shared `CARGO_TARGET_DIR`. Cargo names workspace crates by relative path, so worktrees overwrite the test binaries of other worktrees. Delete the target directory of a worktree when its task is done.
- To build images in this environment, use `make up BUILD_CA_FILE=/root/.ccr/ca-bundle.crt` (TLS proxy).
- CI: GitHub Actions, 6 workflows. iOS runs on GitHub macOS machines. `altyapi.yml` also runs actionlint, kustomize and the kind smoke test. Leave the kind and compose end-to-end checks to CI. Locally, the test of the related crate is sufficient. After a push, check the CI result with `actions_list`.
- Database roles: `app_owner` (owner, NOLOGIN), `app_api`, `app_worker`. `migrate` sets their passwords (`APP_*_PASSWORD`).

## Blockers

- T2.21: `est_cells_ml` and `honey_pct` are calculated in the API (`TODO`: move them into the engine). Publish is one step (`recipe.edit` in Home, `recipe.approve` in Business); submit and review come in stage 6. `calculate` on a published version returns the stored `calc_summary`. `scale` is a preview only. The 412 body has `current.conflicting_fields` as the difference.
- T2.22: the actual ABV comes from an `abv_pct` measurement (the engine needs a call for measured OG/FG). The brew day package key is `BREW_PACKAGE_KEY` (or `_FILE`) and `BREW_PACKAGE_KEY_ID`; without a key the package endpoints give 503. Only the current key verifies, so a key change makes open packages invalid (`TODO`: key rotation). The `device` measurement source comes in T2.25. The SSE delivery of edit session events is not tested end to end.

- T2.39: object lock is off locally (`S3_OBJECT_LOCK_MODE` not set); T2.32 configures a lock-enabled bucket (`S3_DIGEST_BUCKET`) in the cluster and in Terraform. The digest retention is 400 days (approved by Oğuz, 2026-11-20). A finding of the weekly check goes to the log and to `maintenance_runs` only; there is no alert yet.

- T2.38: the engine accepts `conversion_efficiency_pct` and `lauter_efficiency_pct` (proto fields 14, 15). The equipment endpoints (T2.20) do not have these fields yet and do not send them to the engine. Add them in T2.21 or in a small follow-up task. The scripts in `karsilastirma/testler/` still use the rounded BeerSmith constants (not part of the parity tests).

- T2.19: ingredient writes use `recipe.edit` (there is no `library.edit`). Style guides, styles and mappings are read-only. The sample seed (`db` binary `seed-sample`) is not in compose or the Makefile. The source of real library data is open.
- T2.20: the engine has no calibration function. The API calls `BrewhouseEfficiency` for each observation and averages the last 3 values if their spread is 5 points or less (`TODO`: move the rule into the engine; to be verified: brewhouse yield against `efficiency_pct`). The wizard defaults are to be verified. Boil values are equipment columns (no `boil_profiles` table). Workspace settings use `workspaces.version` as the ETag, so an unrelated update can make an optional `If-Match` fail.
- T2.28: the iOS client must set `appAccountToken` to the workspace ID at purchase. The Paddle adapter is not written; the web payment route gives 503 in production. A grace period of 16 days is used when Apple sends no `gracePeriodExpiresDate`. Set `APPLE_BUNDLE_ID` in production. The trial is `workspaces.created_at` plus 1 month (no row).

- T2.9: the measurement endpoint (T2.22) must call `db::validate_measured_at`. `purge_workspace` has no worker job yet. F2.1 items 8–12 are decided (2026-11-20, see Recent decisions). Their implementation is not done yet.

- T2.14: if an attacker deletes both the rows and the head of the chain, the check does not find it. An external anchor is necessary: write a periodic digest to the object store. If the canonical field list changes, add a format version.
- T2.16: `EngineClient` is not connected to `AppState` yet (T2.20 and T2.21 connect it). The proto has no separate field for the intermediate values of "Nasıl hesaplandı".
- T2.11: the web redirect code flow and CORS come in stage 3. The Apple refresh token comes in T2.31. Session events are not written to the audit log yet.
- T2.15: the consumer is not registered yet (add `consumer::run` to `worker/src/background.rs`). Each SSE connection opens its own JetStream consumer. If scale makes it necessary, use a shared subscription.
- T2.18: the file limits (25 MiB; jpeg, png, heic, webp, pdf, text) are approved (2026-11-20). No worker job deletes the object bytes of deleted or incomplete uploads. The object store for k8s and Terraform comes in T2.32. There is no Azure Blob implementation (the interface is ready).
- There is no seed data for tax rates. Rows start as `unverified`. Load them after a check against the official source (T2.8).
- T2.22: a batch starts only from an `approved` version (also in Home mode). To start a batch from a draft, publish the version first.
- T2.10 decisions (they apply until someone objects):
  - An unknown path and an incorrect method give 404 (the ADR code list has no 405).
  - Idempotency keeps only 2xx responses.
  - Rate limits: 300 requests per minute per IP, 600 per user, 1200 per workspace.
  - `/v1/meta/fields` has `max-age=3600`. The request ID is UUIDv7.
  - Each new endpoint adds a row to the `OPERATIONS` table in `api/src/openapi.rs`.
  - The `/internal` listener and SSE come in later tasks.
- T2.30: only `app_worker` writes `subscriptions` and `entitlements`. Support corrections go through the worker (decision 2026-11-20).
- T2.17: the field limits are a product proposal (`verify: true`, to be verified). Done in T2.38 (2026-11-20): the engine and the Python references use the exact unit definitions; 235 vectors were generated again; the tolerance did not change.
- T2.28: the web payment provider is Paddle (O6). The Paddle adapter (webhook signature, events) is not written yet. Other open items are in ADR-0002 (legal and cost decisions for Terraform, cloud account O4, AppIcon).

- T2.23: a yeast line in `count` units is not deducted (`skipped`, `unit_mismatch`). Add a pack amount to the batch actual record. The stock endpoints do not publish SSE events yet. Search has no paging (`limit` 1–50).
- T2.37: `trash_list` and `trash_restore` have `x-permission: none`, because the permission depends on the kind (to be decided). Restore writes no audit row and no outbox event (to be decided). `require_for` scoped permissions do not apply to trash. The list has no cursor.

- T2.24, T2.29, T2.30: APNs and GitHub App tokens are to be verified against the real services. The alert scope `vessel` has no effect. No job sets a request to `closed` after 14 days. `subscription_reverify` only writes an outbox event (no store client). `deletion_request` stays `pending`. The grant note in the Issue is not written. The `INTERNAL_PROXY_TOKEN` and oauth2-proxy header names are an assumption; T2.32 must confirm them. Compose and Helm do not have the `GITHUB_*`, `GITHUB_WEBHOOK_SECRET`, `APNS_*` and `/internal` settings yet (T2.32).
- T2.25: the device body formats are to be verified with real devices (sources in `devices/src/formats.rs`). The iSpindel gravity unit is auto (1.2 or more is Plato) unless `calibration.gravity_unit` is set. ingest has no rate limit for each device. RAPT battery, aux and external temperature, bpm and iSpindel angle are not stored. Tank pairing (`vessel_id`) needs the Business tank endpoints. Set `INGEST_PUBLIC_URL` in production.
- T2.26: the Brewfather parser assumptions (hop grams, `potential` in SG) are to be verified with a real export. The full export ZIP builds in memory (limit 512 MiB). Water profiles, devices, support requests, documents and photos are not in the full export (T2.31). The worker export loop needs `S3_ENDPOINT`. BeerJSON has no first wort or whirlpool use.

- T2.26 `.bsmx`: the BeerSmith index tables (grain type, hop use, yeast form) and the dry hop time unit (days) are to be verified with files from other BeerSmith versions. Misc lines are not imported. Source OG, IBU and color come from the `DESIRED` fields.
- BeerXML 1.0 check: the reader reads fermentable `COLOR` as SRM, but the specification says Lovibond. The writer omits `BREWER`, the `STYLE` sub-fields and empty `ALPHA`, `YIELD`, `COLOR`. The reader does not check required fields and sets yeast `packs` to 1. Decide if the writer must make a fully valid BeerXML 1.0 file.

## Recent decisions

- 2026-11-24 (O9, Oğuz): Sentry receives errors and crash reports from the server, the web and iOS (ADR-0010). Traces, metrics and logs stay in OpenTelemetry. Do not send personal data to Sentry. Write the Sentry tasks into the task lists for stages 3, 4 and 5. The web payment provider stays Paddle (O6).

- 2026-11-20 (Oğuz): the iPhone shows data and does short tasks in the field. Design, planning and management are on the iPad and the web only (plan › iPhone kapsamı). In Home mode, the recipe wizard, the equipment wizard and the quick batch start stay on the iPhone. Business version 2 gets a field view on the iPhone: one scan screen for tank, lot and container QR codes, with the new role "Saha" and three new permissions. The concept has 64 screens. Tasks K1, K2 and K4 changed.

- 2026-11-20 (Oğuz): Business version 2 gets light keg and cask tracking in the Production module. It has a QR label for each container, a status from a scan, and a link to the lot at fill. Customers, deposits and overdue alerts stay out of scope (ERP). Tasks K1–K4 are in stage 6. The concept has the new screen "Fıçılar" (63 screens). The marketing site now says that only customer keg tracking and keg deposits are out of scope.

- 2026-11-20 (Oğuz): the product has modules (plan › Modüller; tasks P1–P3 in stage 5). The marketing site shows the same modules (work package W13 in `web-stratejisi/46-*`).

- 2026-11-20: competitor review of Breww (Business mode) is in `karsilastirma/breww-2026-11.md`. Oğuz approved the plan additions (2026-11-20): stock reservation and alerts, CIP state in the tank proposal (Business core); equipment maintenance, stock count freeze, month-end valuation and xlsx export, light purchasing without invoices or accounting (Business version 2); contract packaging later (product decision).

- 2026-11-20 (Oğuz): we do not train an own model now. Added to the plan: a target-to-recipe solver in the engine (clone a beer uses it; no AI) in Home version 2, and style statistics from open recipes (simple statistics, not a model) later. A model on our own measurement data stays an idea in plan › Analitik modeller.

- 2026-11-20 (Oğuz): no AI recipe generator of our own. The user can connect an own agent through a remote MCP server (OAuth 2.1; the engine calculates the numbers; no publish, approve or delete). ADR in stage 5, release in Home version 2. Tasks M1–M5.

- 2026-11-20 (Oğuz): the daily audit digests stay retention-locked in the object store for 400 days (T2.39).

- 2026-11-20 (competitor review, Oğuz approved Claude proposals; report `karsilastirma/rakipler-2026-11.md`):
  - Import of recipes and equipment from Brewfather JSON and `.bsmx` in Home MVP (T2.26). Batches and stock in Home version 2.
  - Device intake accepts the Brewfather "Custom Stream" format (T2.25).
  - "Recently deleted" screen with restore for 30 days in Home MVP (T2.37).
  - Rule-based "what can I brew from stock" in Home version 2. A recipe draft from a language model comes later, after a separate decision.
  - Conversion and lauter efficiency split also in Home mode, as an advanced setting (T2.38).
  - Claude writes the tier matrix draft before the beta; Oğuz approves it (O8).

- 2026-11-20 (Oğuz): the product name is GRISTORA and the domain is gristora.com. The app design and the marketing site design are separate; only the name, the logo, the app icon and the accent color are shared. Tasks G1–G8 in `gelistirme/ISLER.md` (stages 3–5).

- 2026-11-20 (F2.1 items 8–12, Oğuz approved Claude proposals):
  - 8: the deletion reason is a value from a fixed list, with an optional short note. The client shows a warning: do not write personal data in the note.
  - 9: an ingredient, a lot or a recipe version that is in use cannot be deleted permanently. It can only be archived. When it is not in use, a permanent delete is possible, and references change to a snapshot.
  - 10: only the workspace owner can shorten or remove a legal hold or a retention end. The operation needs step-up and writes an audit log row. A delete request for a workspace with mandatory retention is postponed to the retention end, and the user gets a notification.
  - 11: the payload of an external event that is not processed is kept for 30 days, then cleared. The event ID is kept for 13 months for the duplicate check.
  - 12: each day, the worker writes the digests of the chain heads to an immutable (retention-locked) area of the object store. A weekly job verifies the chain against these digests.
- 2026-11-20 (Oğuz approved Claude proposals):
  - T2.30: support corrects a subscription or an entitlement through a request with a reason in the support panel. The worker applies the request and writes an audit log row. The API does not write these tables.
  - T2.17: the engine will use the exact unit definitions of the catalog. The BeerSmith parity tests keep their tolerance. This is a task for the engine (not started).
  - T2.18: the file limits are approved: 25 MiB; jpeg, png, heic, webp, pdf, text.
- 2026-11-20 (O6, Oğuz): the web payment provider is Paddle (merchant of record).

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
