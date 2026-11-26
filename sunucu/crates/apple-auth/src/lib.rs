//! Shared Sign in with Apple code for `api` and `worker` (ADR-0009 rule 1.5).

pub mod client;
pub mod payload;
pub mod settings;
pub mod token_cipher;

pub use client::{AppleClient, ClientSecretError, RevokeOutcome, RevokeResult};
pub use payload::{ApplePayload, PayloadError};
pub use settings::{AppleTokenSettings, client_ids_from_lookup};
pub use token_cipher::{Aes256GcmTokenCipher, TokenCipher, TokenCipherError};
