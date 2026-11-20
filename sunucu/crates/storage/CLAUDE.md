# storage

## What it does
- Object store interface `ObjectStore` and S3 implementation `S3Store` (S3, GCS interoperability, SeaweedFS): signed `PUT`/`GET` URL (SigV4, query signature), `head`, `digest` (streamed SHA-256), `put`, `delete`, `ensure_bucket`.
- `S3Config`: `S3_*` environment variables. If `S3_ENDPOINT` is not set, the store is off.
- `get` reads a small object. `put_locked` writes an object with an object lock (`S3Store::with_object_lock`, T2.39). Without a configured lock it writes a plain object. TODO: enable the lock in the cluster (T2.32).
- Details: `README.md`. Decision: `adr/0008-api-kurallari.md` (T2.18 notes).

## What it does not touch
- It has no business rules (size, type and authorization are in the `api` crate).
- It has no Azure Blob implementation. If you add one, it must implement `ObjectStore`.

## How to test
- In `sunucu/`, run: `cargo test -p storage`. `tests/s3_live.rs` connects to `objectstore` in compose (127.0.0.1:8333). Start it with `docker compose -f altyapi/compose/compose.yaml up -d --wait objectstore`. The `S3_*` variables change the address and the key.

## Rules
- A change to the signing code must pass the AWS example tests in `sigv4.rs`.
- Do not write secrets to the log or to `Debug` output.
- Start the commit message with the task number: `T2.x: ...`.
