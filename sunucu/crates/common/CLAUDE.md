# common

## What it does
- Shared parts of all server crates: configuration from the environment variables `PORT` and `SERVICE_NAME`, JSON log (`tracing`), shared error type, clean shutdown signal.
- When you enable the `http` feature, it gives `/healthz` and the server starter for axum.
- `config::secret_from_lookup` (secrets `NAME` / `NAME_FILE`) and the `otel` feature (OpenTelemetry trace, OTLP optional).
- `catalog`: field definition, unit and text catalog (from the `sozlesme/katalog` data; `validate`, `limits`). It is the only source of field limits. Server validation reads from it (T2.17).

## What it does not touch
- It has no business logic, no database code and no calculation code.
- Do not put behavior for only one crate here.

## How to test
- In `sunucu/`, run: `cargo test -p common`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`

## Rules
- When you change a shared type or feature, the whole workspace must pass clippy.
- Code is in English. Notes for Claude (this file, task lists) use ASD-STE100. Other documents are in Turkish.
- Start the commit message with the task number: `T0.x: ...`.
