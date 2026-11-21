//! Device data intake (T2.25).
//!
//! A device (iSpindel, RAPT Pill, Tilt bridge, Brewfather "Custom Stream" sender) posts JSON to its
//! own address `/v1/ingest/{key}`. The key is random, shown once, and only its SHA-256 digest is
//! stored (`devices.ingest_key_hash`). A key rotation replaces the digest, so the old address stops
//! working at once.
//!
//! Flow: `ingest` parses and range-checks the body ([`formats`]), then [`intake::store`] writes one
//! `external_events` row (source `device`) and one outbox event [`TOPIC`]. The worker consumer
//! ([`apply::ReadingHandler`]) writes the `measurements` rows (source `device`) for the batch that
//! the device is paired with, and writes the event `batch.measurement_added` for the SSE stream.

pub mod apply;
pub mod formats;
pub mod intake;

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Topic of the outbox event that `ingest` writes for a stored reading.
pub const TOPIC: &str = "device.reading_received";
/// Entity kind of that event (the entity id is the `external_events` id).
pub const ENTITY: &str = "device_reading";
/// Durable name of the worker consumer.
pub const CONSUMER_NAME: &str = "device-readings";
/// Schema version of the stored [`ReadingBatch`].
pub const SCHEMA_VERSION: u32 = 1;

/// NATS subject filter of the reading events (any workspace).
#[must_use]
pub fn reading_filter() -> String {
    format!("{}.*.{ENTITY}.>", events::SUBJECT_PREFIX)
}

/// One normalized value of a device. `kind` is a `measurements.kind`; the value is in SI units
/// (SG, degrees Celsius, pH, bar, volt, dBm).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reading {
    /// `sg`, `temp_c`, `ph`, `pressure_bar`, `battery_v` or `signal_dbm`.
    pub kind: String,
    /// Value in SI units.
    pub value: f64,
}

/// What `ingest` stores in `external_events.payload`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadingBatch {
    /// [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// The device that sent the values.
    pub device_id: uuid::Uuid,
    /// Workspace of the device.
    pub workspace_id: uuid::Uuid,
    /// Receive time at the server (devices send no reliable clock).
    pub received_at: chrono::DateTime<chrono::Utc>,
    /// The values.
    pub readings: Vec<Reading>,
}

/// A new device key: 32 random bytes, URL-safe base64 without padding (43 characters).
///
/// # Panics
/// When the operating system cannot provide randomness.
#[must_use]
pub fn new_key() -> String {
    let mut buf = [0_u8; 32];
    getrandom::fill(&mut buf).unwrap_or_else(|e| panic!("operating system randomness failed: {e}"));
    URL_SAFE_NO_PAD.encode(buf)
}

/// The digest stored for a key (SHA-256 of its text).
#[must_use]
pub fn key_hash(key: &str) -> Vec<u8> {
    Sha256::digest(key.as_bytes()).to_vec()
}

/// Whether `key` has the shape of a key from [`new_key`]. A cheap check before the database.
#[must_use]
pub fn is_key_shape(key: &str) -> bool {
    key.len() == 43
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_unique_and_shaped() {
        let (a, b) = (new_key(), new_key());
        assert_ne!(a, b);
        assert!(is_key_shape(&a));
        assert!(!is_key_shape("short"));
        assert_eq!(key_hash(&a).len(), 32);
        assert_ne!(key_hash(&a), key_hash(&b));
    }
}
