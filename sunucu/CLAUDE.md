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
