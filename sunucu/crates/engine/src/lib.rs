//! Brewing calculation engine.
//!
//! Pure functions, no IO: no database, network, filesystem, clock or async runtime
//! dependencies are allowed in this crate. Every function is deterministic and
//! testable with plain inputs and outputs. Formulas arrive in Phase 1, following the
//! specifications and test vectors under `formuller/`.
//!
//! One module per formula section. Parity with the Python reference is checked by the
//! integration tests in `tests/parite_NN.rs` against `formuller/testler/vektorler/NN.json`
//! (ADR-0003).

pub mod brewery;
pub mod brewery_tax;
pub mod density;
pub mod error;
pub mod home;
pub mod hops;
pub mod manual;
pub mod mash;
pub mod recipe;
pub mod trigger;
pub mod units;
pub mod water;
pub mod yeast;

pub use error::{Error, Result};

/// Returns the engine crate version.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_matches_crate_version() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
        assert!(!version().is_empty());
    }
}
