# sunucu: Rust workspace

## What it does
- Cargo workspace. The crates are in `crates/`: engine, engine-service, api, worker, ingest, common, db, events, storage.
- `rust-toolchain.toml` pins the version. Shared dependencies come into the crates with `workspace = true`.
- Layout decision: `adr/0001-depo-duzeni.md`.

## What it does not touch
- Do not edit `Cargo.lock` by hand. Cargo generates it.
- `sozlesme/` and `altyapi/` are outside this directory. Do not change them from here.
- Do not write client code (`web/`, `ios/`) here.

## How to test
- In `sunucu/`, run: `cargo clippy --workspace --all-targets -- -D warnings`
- One crate: `cargo test -p <crate>` (for example `cargo test -p engine`).
- The `db` tests need a real PostgreSQL (`DATABASE_URL`). In the repository root, `make test` gives the compose default. Run `make up` first, or start only postgres. Details: `crates/db/CLAUDE.md`.

## Rules
- Code is in English. Notes for Claude (this file, task lists) use ASD-STE100. Other documents are in Turkish (ADR-0001).
- Format with `rustfmt`. Do not accept warnings (`clippy -D warnings`).
- Start the commit message with the task number: `T0.x: ...`.

## How to add an endpoint module (`crates/api`)
- Write the module file. Put its `routes` function, its `OperationSpec` rows (`spec::OPERATIONS`) and its `#[derive(OpenApi)]` fragment in that file. Copy the block `registry entry` of `src/events.rs`.
- Define `pub const MODULE: crate::modules::Module` in the file. Add `MODULE` to `MODULES` in `src/modules.rs`. This is the only other change.
- Do not change `src/lib.rs`, `src/openapi.rs` or `src/main.rs`. `build()` serves the routes and `openapi_json()` merges the fragments and rows.
- A module that needs the auth services returns an empty `Router` when `state.auth_services` is `None`. See `src/auth/access.rs`.
- Put the tags of the module in its fragment. The order in `MODULES` sets the order of the tags.
- Run `make gen`. The file `sozlesme/openapi/openapi.json` must change only for the new paths.
