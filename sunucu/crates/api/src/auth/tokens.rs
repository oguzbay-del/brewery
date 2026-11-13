//! Random tokens, their stored digests and the CSRF token (ADR-0009 rules 2.1, 3.1, 3.5).

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, KeyInit, Mac};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// `N` random bytes from the operating system.
///
/// # Panics
/// When the operating system cannot provide randomness (nothing safe can be done then).
#[must_use]
pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut buf = [0_u8; N];
    getrandom::fill(&mut buf).unwrap_or_else(|e| panic!("operating system randomness failed: {e}"));
    buf
}

/// A new opaque token: 32 random bytes, URL-safe base64 without padding.
#[must_use]
pub fn new_token() -> String {
    URL_SAFE_NO_PAD.encode(random_bytes::<32>())
}

/// The digest stored in the database for `token` (SHA-256 of its text).
#[must_use]
pub fn token_hash(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

/// Lower-case hex, for cache keys and the like.
#[must_use]
pub fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        })
}

fn csrf_mac(key: &[u8], session_id: Uuid) -> Hmac<Sha256> {
    // HMAC accepts any key length; `new_from_slice` cannot fail here.
    #[allow(clippy::expect_used)]
    let mut mac =
        <Hmac<Sha256> as KeyInit>::new_from_slice(key).expect("HMAC takes any key length");
    mac.update(b"csrf-v1:");
    mac.update(session_id.as_bytes());
    mac
}

/// The CSRF token of a session: an HMAC bound to the session id (ADR-0009 rule 3.5).
#[must_use]
pub fn csrf_token(key: &[u8], session_id: Uuid) -> String {
    URL_SAFE_NO_PAD.encode(csrf_mac(key, session_id).finalize().into_bytes())
}

/// Constant-time check of a presented CSRF token.
#[must_use]
pub fn csrf_valid(key: &[u8], session_id: Uuid, presented: &str) -> bool {
    URL_SAFE_NO_PAD
        .decode(presented.trim())
        .is_ok_and(|bytes| csrf_mac(key, session_id).verify_slice(&bytes).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_distinct_and_hash_is_stable() {
        let (a, b) = (new_token(), new_token());
        assert_ne!(a, b);
        assert_eq!(a.len(), 43);
        assert_eq!(token_hash(&a), token_hash(&a));
        assert_eq!(token_hash(&a).len(), 32);
    }

    #[test]
    fn csrf_token_is_bound_to_key_and_session() {
        let key = b"0123456789abcdef";
        let s1 = Uuid::now_v7();
        let s2 = Uuid::now_v7();
        let t = csrf_token(key, s1);
        assert!(csrf_valid(key, s1, &t));
        assert!(!csrf_valid(key, s2, &t));
        assert!(!csrf_valid(b"fedcba9876543210", s1, &t));
        assert!(!csrf_valid(key, s1, "not base64 !"));
        assert!(!csrf_valid(key, s1, ""));
    }
}
