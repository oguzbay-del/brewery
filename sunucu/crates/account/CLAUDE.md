# account

## What it does
- Shared code of account deletion (T2.31) for `api` and `worker`.
- `envelope.rs`: AES-256-GCM envelope for the Apple refresh token (`user_identities.provider_refresh_token_enc`). Settings: `ACCOUNT_KEY` (base64, 32 bytes) or `ACCOUNT_KEY_FILE`. Locally the key is a fixed development key. TODO: KMS key wrapping in the cloud (T2.32).
- `apple.rs`: the trait `AppleClient` (`exchange_code`, `revoke`), the HTTP client `HttpApple` and the fake `FakeApple`. Settings: `APPLE_TEAM_ID`, `APPLE_KEY_ID`, `APPLE_PRIVATE_KEY` (PEM, or `APPLE_PRIVATE_KEY_FILE`), `APPLE_CLIENT_IDS` (the first value is the client id), `APPLE_TOKEN_URL`, `APPLE_REVOKE_URL`. `APPLE_LOCAL_MODE=true` selects the fake client for local runs.

## What it does not touch
- It has no database code. The purge is in `worker/src/account.rs` and in migration `0220`.

## Rules
- Do not use `APPLE_LOCAL_MODE` in production. The fake client does not call Apple.
- Code is in English. Notes for Claude use ASD-STE100.
- Start the commit message with the task number: `T2.x: ...`.

## How to test
- In `sunucu/`, run: `cargo test -p account`.
