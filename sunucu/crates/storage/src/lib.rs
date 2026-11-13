//! Object storage behind one interface (T2.18).
//!
//! [`ObjectStore`] is all the server code depends on. [`S3Store`] implements it for Amazon S3
//! and every S3 compatible store: Google Cloud Storage through its S3 interoperability API
//! (HMAC keys), `SeaweedFS` in the local Compose stack, and others. An Azure Blob
//! implementation is not written yet; it would implement the same trait (shared access
//! signatures cannot bind a size limit, which is why completing an upload always checks the
//! stored object, see the `api` crate).
//!
//! Files never pass through the API: clients upload and download directly with short lived
//! presigned requests ([`PresignedRequest`]).

mod config;
mod error;
mod s3;
pub mod sigv4;

use std::{future::Future, pin::Pin, time::Duration};

use chrono::{DateTime, Utc};
use serde::Serialize;

pub use config::S3Config;
pub use error::{StorageError, StorageResult};
pub use s3::S3Store;

/// Boxed future (keeps [`ObjectStore`] object safe).
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// What an upload must look like; the presigned request binds all three.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadLimits {
    /// Exact `Content-Type` the client must send.
    pub content_type: String,
    /// Exact body size in bytes.
    pub size_bytes: u64,
    /// SHA-256 of the body; the store verifies it while receiving.
    pub sha256: [u8; 32],
}

/// A request the client sends itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PresignedRequest {
    /// HTTP method (`PUT` or `GET`).
    pub method: String,
    /// Complete URL with the signature in its query string.
    pub url: String,
    /// Headers the client must send unchanged (name, value). `Host` and `Content-Length` are
    /// signed too but set by every HTTP client from the URL and the body.
    pub headers: Vec<(String, String)>,
    /// When the request stops being accepted.
    pub expires_at: DateTime<Utc>,
}

/// Size and SHA-256 of an object.
pub type Digest = (u64, [u8; 32]);

/// What the store reports about a stored object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectMeta {
    /// Size in bytes.
    pub size_bytes: u64,
    /// Stored `Content-Type`.
    pub content_type: Option<String>,
    /// SHA-256 when the store keeps it (S3 does for uploads that sent the checksum header).
    pub sha256: Option<[u8; 32]>,
}

/// The one interface to object storage.
pub trait ObjectStore: Send + Sync + 'static {
    /// A presigned `PUT` bound to `limits`, valid for `ttl`.
    fn presign_put(
        &self,
        key: &str,
        limits: &UploadLimits,
        ttl: Duration,
    ) -> StorageResult<PresignedRequest>;

    /// A presigned `GET`, valid for `ttl`.
    fn presign_get(&self, key: &str, ttl: Duration) -> StorageResult<PresignedRequest>;

    /// Metadata of an object; `None` when it does not exist.
    fn head<'a>(&'a self, key: &'a str) -> BoxFuture<'a, StorageResult<Option<ObjectMeta>>>;

    /// Size and SHA-256 of an object, computed by streaming it. Fails with
    /// [`StorageError::TooLarge`] past `max_bytes`; `None` when it does not exist.
    fn digest<'a>(
        &'a self,
        key: &'a str,
        max_bytes: u64,
    ) -> BoxFuture<'a, StorageResult<Option<Digest>>>;

    /// Stores a small object (server side writes, tests).
    fn put<'a>(
        &'a self,
        key: &'a str,
        content_type: &'a str,
        body: Vec<u8>,
    ) -> BoxFuture<'a, StorageResult<()>>;

    /// Deletes an object; deleting a missing one succeeds.
    fn delete<'a>(&'a self, key: &'a str) -> BoxFuture<'a, StorageResult<()>>;

    /// Creates the bucket when it is missing (local stack and tests).
    fn ensure_bucket(&self) -> BoxFuture<'_, StorageResult<()>>;
}
