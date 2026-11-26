# apple-auth

## What it does
- Shared Sign in with Apple code for `api` and `worker` (ADR-0009 rule 1.5).
- `AppleTokenSettings` and the environment parser for `APPLE_*` (`settings.rs`).
- `AppleClient` (`client.rs`): signs the ES256 client secret and calls `/auth/revoke`. `revoke_stored` decrypts a stored payload and tries the client IDs in order.
- `TokenCipher` and `Aes256GcmTokenCipher` (`token_cipher.rs`): encrypt and decrypt the refresh token with `PROVIDER_TOKEN_KEY`.
- Payload codec (`payload.rs`): the new payload is JSON `{"v":1,"client_id":"...","refresh_token":"..."}`. The old payload is the raw token bytes.

## What it does not touch
- It does not verify ID tokens and does not exchange authorization codes. `api` does both (`auth/apple_token.rs`).
- It does not read or write the database.

## How to test
- In `sunucu/`, run: `cargo test -p apple-auth`. The tests use a local fake HTTP server and need no other service.

## Rules
- Never log the token, the ciphertext or the client secret.
- Do not add a dependency that is not in the workspace.
- Code is in English. Notes for Claude use ASD-STE100.
- Start the commit message with the task number: `T0.x: ...`.
