//! Envelope encryption of small secrets (the Apple refresh token), AES-256-GCM.
//!
//! Blob layout: `version (1 byte) || nonce (12 bytes) || ciphertext || tag`.

use base64::{Engine, engine::general_purpose::STANDARD};
use ring::{
    aead::{AES_256_GCM, Aad, LessSafeKey, NONCE_LEN, Nonce, UnboundKey},
    rand::{SecureRandom, SystemRandom},
};

const VERSION: u8 = 1;
const AAD: &[u8] = b"gristora:account:provider_refresh_token";

/// Why a blob cannot be opened.
#[derive(Debug, thiserror::Error)]
pub enum EnvelopeError {
    /// The key is not 32 bytes of base64.
    #[error("account key must be 32 bytes in base64")]
    BadKey,
    /// The blob is damaged or the key is not the one that sealed it.
    #[error("cannot decrypt the value")]
    Decrypt,
    /// The random source failed.
    #[error("random source failed")]
    Random,
}

/// The sealing key.
pub struct Envelope {
    key: LessSafeKey,
}

impl std::fmt::Debug for Envelope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Envelope").finish_non_exhaustive()
    }
}

impl Envelope {
    /// From 32 raw key bytes.
    ///
    /// # Errors
    /// [`EnvelopeError::BadKey`] when the length is wrong.
    pub fn from_bytes(key: &[u8]) -> Result<Self, EnvelopeError> {
        let unbound = UnboundKey::new(&AES_256_GCM, key).map_err(|_| EnvelopeError::BadKey)?;
        Ok(Self {
            key: LessSafeKey::new(unbound),
        })
    }

    /// From a base64 key.
    ///
    /// # Errors
    /// [`EnvelopeError::BadKey`] when it is not 32 bytes of base64.
    pub fn from_base64(text: &str) -> Result<Self, EnvelopeError> {
        let bytes = STANDARD
            .decode(text.trim())
            .map_err(|_| EnvelopeError::BadKey)?;
        Self::from_bytes(&bytes)
    }

    /// From `ACCOUNT_KEY` or `ACCOUNT_KEY_FILE`. `None` when neither is set or the key is bad
    /// (a bad key is logged).
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let raw = std::env::var("ACCOUNT_KEY")
            .ok()
            .or_else(|| {
                std::env::var("ACCOUNT_KEY_FILE")
                    .ok()
                    .and_then(|p| std::fs::read_to_string(p).ok())
            })
            .filter(|v| !v.trim().is_empty())?;
        match Self::from_base64(&raw) {
            Ok(e) => Some(e),
            Err(e) => {
                tracing::error!(error = %e, "ACCOUNT_KEY is invalid");
                None
            }
        }
    }

    /// Seals `plain`.
    ///
    /// # Errors
    /// [`EnvelopeError::Random`] when no nonce can be made.
    pub fn seal(&self, plain: &[u8]) -> Result<Vec<u8>, EnvelopeError> {
        let mut nonce = [0u8; NONCE_LEN];
        SystemRandom::new()
            .fill(&mut nonce)
            .map_err(|_| EnvelopeError::Random)?;
        let mut buf = plain.to_vec();
        self.key
            .seal_in_place_append_tag(
                Nonce::assume_unique_for_key(nonce),
                Aad::from(AAD),
                &mut buf,
            )
            .map_err(|_| EnvelopeError::Decrypt)?;
        let mut out = Vec::with_capacity(1 + NONCE_LEN + buf.len());
        out.push(VERSION);
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&buf);
        Ok(out)
    }

    /// Opens a blob made by [`Envelope::seal`].
    ///
    /// # Errors
    /// [`EnvelopeError::Decrypt`] when the blob or the key is wrong.
    pub fn open(&self, blob: &[u8]) -> Result<Vec<u8>, EnvelopeError> {
        if blob.len() < 1 + NONCE_LEN || blob[0] != VERSION {
            return Err(EnvelopeError::Decrypt);
        }
        let nonce: [u8; NONCE_LEN] = blob[1..=NONCE_LEN]
            .try_into()
            .map_err(|_| EnvelopeError::Decrypt)?;
        let mut buf = blob[1 + NONCE_LEN..].to_vec();
        let plain = self
            .key
            .open_in_place(
                Nonce::assume_unique_for_key(nonce),
                Aad::from(AAD),
                &mut buf,
            )
            .map_err(|_| EnvelopeError::Decrypt)?;
        Ok(plain.to_vec())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn seal_and_open_round_trip() {
        let e = Envelope::from_bytes(&[7u8; 32]).unwrap();
        let blob = e.seal(b"refresh-token").unwrap();
        assert!(!blob.windows(13).any(|w| w == b"refresh-token"));
        assert_eq!(e.open(&blob).unwrap(), b"refresh-token");
    }

    #[test]
    fn wrong_key_or_damage_is_refused() {
        let a = Envelope::from_bytes(&[1u8; 32]).unwrap();
        let b = Envelope::from_bytes(&[2u8; 32]).unwrap();
        let mut blob = a.seal(b"x").unwrap();
        assert!(b.open(&blob).is_err());
        let last = blob.len() - 1;
        blob[last] ^= 1;
        assert!(a.open(&blob).is_err());
        assert!(a.open(&[1, 2, 3]).is_err());
        assert!(Envelope::from_bytes(&[0u8; 5]).is_err());
    }
}
