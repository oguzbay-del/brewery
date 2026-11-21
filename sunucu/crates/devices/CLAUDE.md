# devices

## What it does
- Device data intake (T2.25). `formats` parses the bodies of iSpindel, RAPT and Brewfather "Custom Stream" and checks the ranges (SG 0.980 to 1.200, temperature -10 to 100 C, pH 0 to 14, pressure 0 to 10 bar, battery 0 to 6 V, signal -130 to 0 dBm). A value out of range rejects the whole body.
- `intake::find` and `intake::store` (role `app_api`, called by `ingest`): find the device by the digest of its key, write `external_events` (source `device`) and the outbox event `device.reading_received`.
- `apply::ReadingHandler` (role `app_worker`, consumer `device-readings`): writes the measurements of the paired batch and the event `batch.measurement_added`. It drops a reading when the device has no batch or the batch is `completed` or `cancelled`.
- `new_key` and `key_hash`: a key has 32 random bytes in URL-safe base64. Only the SHA-256 digest is stored.

## What it does not touch
- It has no HTTP code. `ingest` serves the data address. `api/src/devices.rs` has the registration endpoints.
- Calibration (angle to SG curve) is not here. The formats are to be verified against real devices: each source is in the comment of `formats.rs`.

## How to test
- In `sunucu/`, run: `cargo test -p devices` (parser tests, no database). The end-to-end test is `crates/api/tests/devices.rs` (PostgreSQL and NATS).

## Rules
- Do not log a key or an address that contains a key.
- Code is in English. Notes for Claude (this file, task lists) use ASD-STE100. Other documents are in Turkish.
- Start the commit message with the task number: `T2.x: ...`.
