//! File formats for import and export (T2.26): `BeerXML`, `BeerJSON`, Brewfather JSON, escaped
//! CSV and a stored ZIP writer.
//!
//! The crate has no I/O and no database. It turns bytes into the neutral [`model`] and back.
//! The model uses the internal SI units: kg, L, degrees C, SRM, SG, minutes. The parsers read
//! untrusted files: see [`xml`] for the limits and the entity rules.
//!
//! TODO: `.bsmx` (`BeerSmith`) needs a sample file. None is available (T2.26).

#![allow(clippy::too_many_lines, clippy::similar_names)]

pub mod beerjson;
pub mod beerxml;
pub mod brewfather;
pub mod convert;
pub mod csv;
pub mod model;
pub mod xml;
pub mod zip;

pub use model::{Document, Equipment, Recipe};

/// Errors of the parsers.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    /// The input is larger than the limit.
    #[error("input is too large")]
    TooLarge,
    /// The XML has too many elements or is nested too deep.
    #[error("input is too complex")]
    TooComplex,
    /// The XML has a `DOCTYPE` (and so could have entities).
    #[error("a DOCTYPE is not allowed")]
    DoctypeNotAllowed,
    /// The XML refers to an entity that is not predefined.
    #[error("an entity is not allowed")]
    EntityNotAllowed,
    /// The XML is not well formed.
    #[error("invalid XML: {0}")]
    Xml(String),
    /// The JSON is not valid.
    #[error("invalid JSON: {0}")]
    Json(String),
    /// The file is valid but is not the expected format.
    #[error("unexpected format: {0}")]
    Format(String),
}

/// Result of a parse.
pub type Result<T> = std::result::Result<T, Error>;

/// Largest JSON input, bytes.
pub const MAX_JSON_BYTES: usize = 16 * 1024 * 1024;

/// Parses JSON with a size limit.
pub(crate) fn parse_json(input: &[u8]) -> Result<serde_json::Value> {
    if input.len() > MAX_JSON_BYTES {
        return Err(Error::TooLarge);
    }
    serde_json::from_slice(input).map_err(|e| Error::Json(e.to_string()))
}
