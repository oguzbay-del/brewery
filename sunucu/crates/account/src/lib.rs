//! Shared code of account deletion (T2.31): envelope encryption of the Apple refresh token and
//! the client for Sign in with Apple (code exchange, token revoke).

pub mod apple;
pub mod envelope;

pub use apple::{AppleClient, AppleError, FakeApple, HttpApple};
pub use envelope::{Envelope, EnvelopeError};
