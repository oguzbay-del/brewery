//! Shared building blocks for all server crates: configuration, logging and errors.

pub mod catalog;
pub mod config;
pub mod error;
#[cfg(feature = "http")]
pub mod http;
pub mod shutdown;
pub mod telemetry;

pub use config::ServiceConfig;
pub use error::{Error, Result};
