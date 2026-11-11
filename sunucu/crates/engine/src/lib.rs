//! Brewing calculation engine.
//!
//! Pure functions, no IO: no database, network, filesystem, clock or async runtime
//! dependencies are allowed in this crate. Every function is deterministic and
//! testable with plain inputs and outputs. Formulas arrive in Phase 1, following the
//! specifications and test vectors under `formuller/`.

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
