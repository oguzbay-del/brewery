/// Object store failures.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// The store could not be reached or answered with a server error.
    #[error("object store unavailable: {0}")]
    Unavailable(String),
    /// The store refused the request.
    #[error("object store refused the request ({status}): {body}")]
    Rejected {
        /// HTTP status.
        status: u16,
        /// Start of the response body.
        body: String,
    },
    /// The object is larger than the caller allows.
    #[error("object is larger than {0} bytes")]
    TooLarge(u64),
    /// Settings are missing or invalid.
    #[error("invalid storage configuration: {0}")]
    Config(String),
}

/// Result of a storage call.
pub type StorageResult<T> = Result<T, StorageError>;
