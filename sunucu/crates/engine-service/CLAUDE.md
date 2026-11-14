# engine-service

## What it does
- It serves the `engine` library over gRPC: `EngineService`, `ToolService`, `RecipeService`, `MashService`, `WaterService`, `YeastService`, `FermentationService`, `BreweryService` (`demleme.engine.v1`, ADR-0005) and the standard health service.
- Unit conversion is only in `convert.rs`. Shared input checks are in `check.rs`. The end-to-end test is `tests/uc_uca.rs`.
- The contract is `sozlesme/proto/engine/v1/*.proto`. `build.rs` compiles it with `protox` (the system does not need protoc).
- Port `PORT` (default 50051).

## What it does not touch
- It is only for the internal network. Do not expose it to the outside.
- Do not write calculation code. The calculation is in `engine`. This crate calls it.
- Change the proto contract in `sozlesme/`. Do not change it in this crate.

## How to test
- In `sunucu/`, run: `cargo test -p engine-service`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`

## Rules
- When the proto changes, update the contract first. Then build the service.
- Code is in English. Notes for Claude (this file, task lists) use ASD-STE100. Other documents are in Turkish.
- Start the commit message with the task number: `T0.x: ...`.
