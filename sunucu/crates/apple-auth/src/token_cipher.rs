//! Encryption for provider refresh tokens (ADR-0009 rule 1.5).
//!
//! The stored format is version byte 1, a 12-byte random nonce, then AES-256-GCM
//! ciphertext and its authentication tag. The trait keeps a later KMS implementation
//! behind the same API.
//!
//! TODO: Bind the ciphertext to `user_identities.id` with AAD.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use common::{Error, Result as ConfigResult, config::secret_from_lookup};
use ring::{
    aead::{self, Aad, LessSafeKey, Nonce, UnboundKey},
    rand::{SecureRandom, SystemRandom},
};

const VERSION: u8 = 1;
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

/// Errors returned by a provider-token cipher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TokenCipherError {
    /// The stored format version is not supported.
    #[error("unsupported provider-token format")]
    UnsupportedVersion,
    /// The ciphertext is too short or malformed.
    #[error("invalid provider-token ciphertext")]
    InvalidCiphertext,
    /// Secure random generation failed.
    #[error("secure random generation failed")]
    Random,
    /// Encryption failed.
    #[error("provider-token encryption failed")]
    Encrypt,
    /// Authentication or decryption failed.
    #[error("provider-token authentication failed")]
    Decrypt,
}

/// Encryption interface for provider refresh tokens.
pub trait TokenCipher: Send + Sync {
    /// Encrypts plaintext and returns the versioned stored representation.
    ///
    /// # Errors
    /// Returns an error if secure randomness or encryption fails.
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, TokenCipherError>;

    /// Decrypts a versioned stored representation.
    ///
    /// # Errors
    /// Returns an error if the format, authentication tag or ciphertext is invalid.
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, TokenCipherError>;
}

/// Local AES-256-GCM implementation. Cloud KMS can implement the trait later.
#[derive(Clone)]
pub struct Aes256GcmTokenCipher {
    key: [u8; 32],
}

impl std::fmt::Debug for Aes256GcmTokenCipher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Aes256GcmTokenCipher")
            .field("key", &"<redacted>")
            .finish()
    }
}

impl Aes256GcmTokenCipher {
    /// Creates a cipher from exactly 32 key bytes.
    #[must_use]
    pub const fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Reads `PROVIDER_TOKEN_KEY` (or `PROVIDER_TOKEN_KEY_FILE`) from `lookup`.
    ///
    /// Returns `None` when no key is set and `required` is false.
    ///
    /// # Errors
    /// Returns an error when the key is invalid, or missing while `required` is true.
    pub fn from_lookup<F>(lookup: &F, required: bool) -> ConfigResult<Option<Self>>
    where
        F: Fn(&str) -> Option<String>,
    {
        match secret_from_lookup(lookup, "PROVIDER_TOKEN_KEY", "PROVIDER_TOKEN_KEY_FILE")? {
            Some(raw) => {
                let decoded = STANDARD.decode(raw.trim()).map_err(|_| Error::InvalidEnv {
                    name: "PROVIDER_TOKEN_KEY",
                    reason: "must be base64".to_owned(),
                })?;
                let key: [u8; 32] = decoded.try_into().map_err(|_| Error::InvalidEnv {
                    name: "PROVIDER_TOKEN_KEY",
                    reason: "must decode to exactly 32 bytes".to_owned(),
                })?;
                Ok(Some(Self::new(key)))
            }
            None if required => Err(Error::InvalidEnv {
                name: "PROVIDER_TOKEN_KEY",
                reason: "required when Apple token exchange is configured".to_owned(),
            }),
            None => Ok(None),
        }
    }

    fn key(&self) -> Result<LessSafeKey, TokenCipherError> {
        let key = UnboundKey::new(&aead::AES_256_GCM, &self.key)
            .map_err(|_| TokenCipherError::Encrypt)?;
        Ok(LessSafeKey::new(key))
    }
}

impl TokenCipher for Aes256GcmTokenCipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, TokenCipherError> {
        let mut nonce_bytes = [0_u8; NONCE_LEN];
        SystemRandom::new()
            .fill(&mut nonce_bytes)
            .map_err(|_| TokenCipherError::Random)?;
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let mut encrypted = plaintext.to_vec();
        self.key()?
            .seal_in_place_append_tag(nonce, Aad::empty(), &mut encrypted)
            .map_err(|_| TokenCipherError::Encrypt)?;

        let mut stored = Vec::with_capacity(1 + NONCE_LEN + encrypted.len());
        stored.push(VERSION);
        stored.extend_from_slice(&nonce_bytes);
        stored.extend_from_slice(&encrypted);
        Ok(stored)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, TokenCipherError> {
        if ciphertext.len() < 1 + NONCE_LEN + TAG_LEN {
            return Err(TokenCipherError::InvalidCiphertext);
        }
        if ciphertext[0] != VERSION {
            return Err(TokenCipherError::UnsupportedVersion);
        }
        let nonce = Nonce::try_assume_unique_for_key(&ciphertext[1..=NONCE_LEN])
            .map_err(|_| TokenCipherError::InvalidCiphertext)?;
        let mut encrypted = ciphertext[1 + NONCE_LEN..].to_vec();
        let plaintext = self
            .key()
            .map_err(|_| TokenCipherError::Decrypt)?
            .open_in_place(nonce, Aad::empty(), &mut encrypted)
            .map_err(|_| TokenCipherError::Decrypt)?;
        Ok(plaintext.to_vec())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn round_trip_uses_version_nonce_and_authenticated_ciphertext() {
        let cipher = Aes256GcmTokenCipher::new([7; 32]);
        let encrypted = cipher.encrypt(b"refresh-token-value").unwrap();
        assert_eq!(encrypted[0], VERSION);
        assert_eq!(
            encrypted.len(),
            1 + NONCE_LEN + b"refresh-token-value".len() + TAG_LEN
        );
        assert_eq!(
            cipher.decrypt(&encrypted).unwrap().as_slice(),
            b"refresh-token-value"
        );
    }

    #[test]
    fn each_encryption_uses_a_fresh_nonce() {
        let cipher = Aes256GcmTokenCipher::new([7; 32]);
        let first = cipher.encrypt(b"same").unwrap();
        let second = cipher.encrypt(b"same").unwrap();
        assert_ne!(&first[1..=NONCE_LEN], &second[1..=NONCE_LEN]);
        assert_ne!(first, second);
    }

    #[test]
    fn rejects_unsupported_version_and_tampering() {
        let cipher = Aes256GcmTokenCipher::new([7; 32]);
        let mut encrypted = cipher.encrypt(b"refresh-token-value").unwrap();
        encrypted[0] = 2;
        assert_eq!(
            cipher.decrypt(&encrypted),
            Err(TokenCipherError::UnsupportedVersion)
        );

        let mut encrypted = cipher.encrypt(b"refresh-token-value").unwrap();
        let last = encrypted.len() - 1;
        encrypted[last] ^= 1;
        assert_eq!(cipher.decrypt(&encrypted), Err(TokenCipherError::Decrypt));
    }

    #[test]
    fn rejects_short_ciphertext() {
        let cipher = Aes256GcmTokenCipher::new([7; 32]);
        assert_eq!(
            cipher.decrypt(&[VERSION; 4]),
            Err(TokenCipherError::InvalidCiphertext)
        );
    }
}
