//! gRPC front for the calculation engine.

/// Generated protobuf and gRPC code for `demleme.engine.v1`.
#[allow(clippy::pedantic, clippy::all, missing_docs)]
pub mod proto {
    tonic::include_proto!("demleme.engine.v1");
}

mod check;
pub mod convert;
pub mod mash;
pub mod recipe;
pub mod service;
pub mod status;
pub mod tool;
pub mod yeast;
