# engine

## What it does
- Calculation engine library. Write the formulas here as pure functions.
- The formulas follow the specifications and test vectors in `formuller/` (Stage 1).
- One module for each section (`units`, `density`, `hops`, ...). The parity method is in ADR-0003.

## What it does not touch
- It has no IO. Do not add a dependency on a database, a network, a clock or async. You can add only a pure calculation crate (for example `rust_decimal`). The test dependency is `serde_json`.
- `engine-service` owns the gRPC and server code.

## How to test
- In `sunucu/`, run: `cargo test -p engine` (parity: `tests/parite_NN.rs`; `make vektor` generates the vectors)
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`

## Rules
- Code is in English. Notes for Claude (this file, task lists) use ASD-STE100. Other documents are in Turkish.
- Change a formula in the `formuller/` specification first. Then change the code and the test vector.
- Start the commit message with the task number: `T0.x: ...`.
