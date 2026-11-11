//! Common error type.

/// Convenience result alias using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Errors shared across services.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An environment variable holds a value that cannot be parsed.
    #[error("invalid value for environment variable {name}: {reason}")]
    InvalidEnv {
        /// Variable name.
        name: &'static str,
        /// Human-readable reason.
        reason: String,
    },
    /// Logging could not be initialised.
    #[error("failed to initialise telemetry: {0}")]
    Telemetry(String),
    /// I/O failure (binding sockets and similar).
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
