//! Plaintext of the stored Apple refresh token.
//!
//! The new format is JSON `{"v":1,"client_id":"...","refresh_token":"..."}`. The old format is
//! the raw token bytes without a version. No data migration exists (issue 215).

use serde::{Deserialize, Serialize};

/// Errors from the payload codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PayloadError {
    /// The payload is neither a supported JSON version nor valid UTF-8 token text.
    #[error("invalid Apple token payload")]
    Invalid,
    /// The JSON version is not supported.
    #[error("unsupported Apple token payload version")]
    UnsupportedVersion,
}

#[derive(Serialize)]
struct EncodeV1<'a> {
    v: u8,
    client_id: &'a str,
    refresh_token: &'a str,
}

#[derive(Deserialize)]
struct DecodeV1 {
    v: u8,
    client_id: String,
    refresh_token: String,
}

/// A decoded refresh-token payload.
#[derive(Clone, PartialEq, Eq)]
pub struct ApplePayload {
    /// The client ID that issued the token. `None` for the old format.
    pub client_id: Option<String>,
    /// The Apple refresh token.
    pub refresh_token: String,
}

impl std::fmt::Debug for ApplePayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplePayload")
            .field("client_id", &self.client_id)
            .field("refresh_token", &"<redacted>")
            .finish()
    }
}

impl ApplePayload {
    /// Encodes the version 1 payload.
    ///
    /// # Errors
    /// Returns an error if JSON encoding fails.
    pub fn encode_v1(client_id: &str, refresh_token: &str) -> Result<Vec<u8>, PayloadError> {
        serde_json::to_vec(&EncodeV1 {
            v: 1,
            client_id,
            refresh_token,
        })
        .map_err(|_| PayloadError::Invalid)
    }

    /// Decodes a version 1 payload or an old raw token.
    ///
    /// # Errors
    /// Returns an error for an unsupported version, an empty token or invalid UTF-8.
    pub fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        if let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes)
            && value.is_object()
        {
            let v1: DecodeV1 = serde_json::from_value(value).map_err(|_| PayloadError::Invalid)?;
            if v1.v != 1 {
                return Err(PayloadError::UnsupportedVersion);
            }
            if v1.client_id.is_empty() || v1.refresh_token.is_empty() {
                return Err(PayloadError::Invalid);
            }
            return Ok(Self {
                client_id: Some(v1.client_id),
                refresh_token: v1.refresh_token,
            });
        }
        let token = std::str::from_utf8(bytes).map_err(|_| PayloadError::Invalid)?;
        if token.is_empty() {
            return Err(PayloadError::Invalid);
        }
        Ok(Self {
            client_id: None,
            refresh_token: token.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn v1_payload_round_trips() {
        let bytes = ApplePayload::encode_v1("com.example.ios", "r.token.value").unwrap();
        let payload = ApplePayload::decode(&bytes).unwrap();
        assert_eq!(payload.client_id.as_deref(), Some("com.example.ios"));
        assert_eq!(payload.refresh_token, "r.token.value");
    }

    #[test]
    fn old_payload_is_the_raw_token() {
        let payload = ApplePayload::decode(b"r.old.token").unwrap();
        assert_eq!(payload.client_id, None);
        assert_eq!(payload.refresh_token, "r.old.token");
    }

    #[test]
    fn rejects_unknown_version_and_empty_input() {
        assert_eq!(
            ApplePayload::decode(br#"{"v":2,"client_id":"a","refresh_token":"b"}"#),
            Err(PayloadError::UnsupportedVersion)
        );
        assert_eq!(ApplePayload::decode(b""), Err(PayloadError::Invalid));
        assert_eq!(
            ApplePayload::decode(&[0xff, 0xfe]),
            Err(PayloadError::Invalid)
        );
    }
}
