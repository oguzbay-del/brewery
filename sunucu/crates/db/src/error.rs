//! Error type of the `db` crate.

/// Convenience result alias using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Errors raised while configuring, connecting to or migrating the database.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Neither `DATABASE_URL` nor `DATABASE_URL_FILE` is set.
    #[error("neither DATABASE_URL nor DATABASE_URL_FILE is set")]
    MissingUrl,
    /// Both `DATABASE_URL` and `DATABASE_URL_FILE` are set; the choice would be ambiguous.
    #[error("DATABASE_URL and DATABASE_URL_FILE are both set; set exactly one")]
    AmbiguousUrl,
    /// The file named by `DATABASE_URL_FILE` could not be read or is empty.
    #[error("cannot read database URL from {path}: {reason}")]
    UrlFile {
        /// Path taken from `DATABASE_URL_FILE`.
        path: String,
        /// Human-readable reason.
        reason: String,
    },
    /// A secret is set both directly and through its `_FILE` variable.
    #[error("{variable} and {variable}_FILE are both set; set exactly one")]
    AmbiguousSecret {
        /// Name of the direct variable.
        variable: String,
    },
    /// The file named by a `*_FILE` secret variable could not be read or is empty.
    #[error("cannot read secret from {path}: {reason}")]
    SecretFile {
        /// Path taken from the `*_FILE` variable.
        path: String,
        /// Human-readable reason.
        reason: String,
    },
    /// Query, connection or pool failure.
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    /// A migration failed to apply (or the applied history does not match the files).
    #[error(transparent)]
    Migrate(#[from] sqlx::migrate::MigrateError),
}
