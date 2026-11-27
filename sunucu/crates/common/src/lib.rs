//! Shared building blocks for all server crates: configuration, logging and errors.

pub mod catalog;
pub mod config;
pub mod error;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "otel")]
pub mod otel;
#[cfg(feature = "redis")]
pub mod redis_client;
pub mod retention;
#[cfg(feature = "sentry")]
pub mod sentry;
pub mod shutdown;
pub mod telemetry;

pub use config::ServiceConfig;
pub use error::{Error, Result};
