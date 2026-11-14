//! Member PINs of shared devices (ADR-0009 rule 7.2): six digits, kept as Argon2id (PHC
//! string, library defaults: m = 19 MiB, t = 2, p = 1). Hashing is slow on purpose and runs on
//! the blocking pool.

use std::sync::OnceLock;

use argon2::{Argon2, PasswordHasher, PasswordVerifier};

use super::tokens::random_bytes;
use crate::error::{ApiError, ApiResult, ErrorCode};

/// Digits of a PIN.
pub const PIN_LEN: usize = 6;
/// Failed attempts in a row that lock a member (rule 7.2).
pub const MAX_FAILED_ATTEMPTS: i32 = 5;
/// Length of the lock, minutes.
pub const LOCK_MINUTES: i64 = 15;
/// Attempts per device and minute.
pub const DEVICE_ATTEMPTS_PER_MINUTE: u32 = 20;

/// Six ASCII digits.
#[must_use]
pub fn is_valid_pin(pin: &str) -> bool {
    pin.len() == PIN_LEN && pin.bytes().all(|b| b.is_ascii_digit())
}

fn hash_blocking(pin: &str) -> Option<String> {
    let salt = random_bytes::<16>();
    Argon2::default()
        .hash_password_with_salt(pin.as_bytes(), &salt)
        .ok()
        .map(|h| h.to_string())
}

/// Hashes a PIN.
pub async fn hash_pin(pin: &str) -> ApiResult<String> {
    let pin = pin.to_owned();
    tokio::task::spawn_blocking(move || hash_blocking(&pin))
        .await
        .ok()
        .flatten()
        .ok_or_else(|| ApiError::new(ErrorCode::Internal))
}

/// Checks a PIN against a stored hash.
pub async fn verify_pin(pin: &str, hash: &str) -> bool {
    let (pin, hash) = (pin.to_owned(), hash.to_owned());
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .verify_password(pin.as_bytes(), hash.as_str())
            .is_ok()
    })
    .await
    .unwrap_or(false)
}

/// A hash of no PIN anyone has, verified when the member or the PIN row does not exist so the
/// answer takes as long as a real check.
pub async fn dummy_verify(pin: &str) {
    static DUMMY: OnceLock<String> = OnceLock::new();
    let hash = DUMMY.get_or_init(|| hash_blocking("000000").unwrap_or_default());
    let _ = verify_pin(pin, hash).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_six_digits_are_a_pin() {
        assert!(is_valid_pin("012345"));
        assert!(!is_valid_pin("12345"));
        assert!(!is_valid_pin("1234567"));
        assert!(!is_valid_pin("12a456"));
        assert!(!is_valid_pin("١٢٣٤٥٦"));
    }

    #[tokio::test]
    async fn hash_verifies_and_is_argon2id_with_a_fresh_salt() {
        let a = hash_pin("123456").await.unwrap();
        let b = hash_pin("123456").await.unwrap();
        assert!(a.starts_with("$argon2id$"));
        assert_ne!(a, b);
        assert!(verify_pin("123456", &a).await);
        assert!(!verify_pin("123457", &a).await);
    }
}
