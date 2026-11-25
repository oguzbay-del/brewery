//! Authenticated encryption for provider refresh tokens (ADR-0009 rule 1.5).
//!
//! Stored format: version byte, 96-bit random nonce, AES-256-GCM ciphertext and tag.
//! The key is supplied by configuration; this module never reads environment variables.

use ring::{
    aead::{self, Aad, LessSafeKey, Nonce, UnboundKey},
    rand::{SecureRandom, SystemRandom},
};

const FORMAT_VERSION: u8 = 1;
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

/// A fixed-size AES-256-GCM key. Its contents are never exposed through Debug.
#[derive(Clone)]
pub struct TokenCipher {
    key: [u8; 32],
}

/// Failure while encrypting or decrypting a provider token.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TokenCipherError {
    #[error("provider token ciphertext is truncated")]
    Truncated,
    #[error("provider token ciphertext version is not supported")]
    UnsupportedVersion,
    #[error("secure random source is unavailable")]
    RandomUnavailable,
    #[error("provider token encryption failed")]
    EncryptionFailed,
    #[error("provider token authentication failed")]
    AuthenticationFailed,
}

impl TokenCipher {
    /// Creates a cipher from a 32-byte key.
    #[must_use]
    pub const fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    fn key(&self) -> Result<LessSafeKey, TokenCipherError> {
        let key = UnboundKey::new(&aead::AES_256_GCM, &self.key)
            .map_err(|_| TokenCipherError::EncryptionFailed)?;
        Ok(LessSafeKey::new(key))
    }

    /// Encrypts a provider refresh token.
    ///
    /// The returned bytes are safe to store in the encrypted refresh-token column.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, TokenCipherError> {
        let key = self.key()?;
        let rng = SystemRandom::new();
        let mut nonce_bytes = [0_u8; NONCE_LEN];
        rng.fill(&mut nonce_bytes)
            .map_err(|_| TokenCipherError::RandomUnavailable)?;
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let mut ciphertext = plaintext.to_vec();
        key.seal_in_place_append_tag(nonce, Aad::empty(), &mut ciphertext)
            .map_err(|_| TokenCipherError::EncryptionFailed)?;

        let mut stored = Vec::with_capacity(1 + NONCE_LEN + ciphertext.len());
        stored.push(FORMAT_VERSION);
        stored.extend_from_slice(&nonce_bytes);
        stored.extend_from_slice(&ciphertext);
        Ok(stored)
    }

    /// Decrypts a provider refresh token previously returned by encrypt.
    pub fn decrypt(&self, stored: &[u8]) -> Result<Vec<u8>, TokenCipherError> {
        if stored.len() < 1 + NONCE_LEN + TAG_LEN {
            return Err(TokenCipherError::Truncated);
        }
        if stored[0] != FORMAT_VERSION {
            return Err(TokenCipherError::UnsupportedVersion);
        }
        let nonce_bytes: [u8; NONCE_LEN] = stored[1..1 + NONCE_LEN]
            .try_into()
            .map_err(|_| TokenCipherError::Truncated)?;
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let key = self.key()?;
        let mut ciphertext = stored[1 + NONCE_LEN..].to_vec();
        let plaintext = key
            .open_in_place(nonce, Aad::empty(), &mut ciphertext)
            .map_err(|_| TokenCipherError::AuthenticationFailed)?;
        Ok(plaintext.to_vec())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    const KEY: [u8; 32] = [0x5a; 32];

    #[test]
    fn encrypts_and_decrypts_provider_refresh_tokens() {
        let cipher = TokenCipher::new(KEY);
        let plaintext = b"apple-refresh-token";
        let encrypted = cipher.encrypt(plaintext).expect("encryption succeeds");
        assert_eq!(encrypted[0], FORMAT_VERSION);
        assert_ne!(encrypted, plaintext);
        assert_eq!(
            cipher.decrypt(&encrypted).expect("decryption succeeds"),
            plaintext
        );
    }

    #[test]
    fn each_encryption_uses_a_fresh_nonce() {
        let cipher = TokenCipher::new(KEY);
        let first = cipher.encrypt(b"same token").expect("first encryption succeeds");
        let second = cipher.encrypt(b"same token").expect("second encryption succeeds");
        assert_ne!(&first[1..1 + NONCE_LEN], &second[1..1 + NONCE_LEN]);
        assert_ne!(first, second);
    }

    #[test]
    fn refuses_tampered_ciphertext_and_wrong_keys() {
        let cipher = TokenCipher::new(KEY);
        let mut encrypted = cipher.encrypt(b"secret").expect("encryption succeeds");
        let last = encrypted.len() - 1;
        encrypted[last] ^= 1;
        assert_eq!(
            cipher.decrypt(&encrypted),
            Err(TokenCipherError::AuthenticationFailed)
        );

        let other = TokenCipher::new([0xa5; 32]);
        let encrypted = cipher.encrypt(b"secret").expect("encryption succeeds");
        assert_eq!(
            other.decrypt(&encrypted),
            Err(TokenCipherError::AuthenticationFailed)
        );
    }

    #[test]
    fn rejects_unknown_versions_and_truncated_values() {
        let cipher = TokenCipher::new(KEY);
        assert_eq!(cipher.decrypt(&[]), Err(TokenCipherError::Truncated));
        assert_eq!(
            cipher.decrypt(&[2; 1 + NONCE_LEN + TAG_LEN]),
            Err(TokenCipherError::UnsupportedVersion)
        );
    }
}
