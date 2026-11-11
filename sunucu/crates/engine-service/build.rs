//! Compiles the engine protobuf contract with the pure-Rust `protox` compiler
//! (no system `protoc` required).

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let proto_root = "../../../sozlesme/proto";
    let proto_file = "../../../sozlesme/proto/engine/v1/engine.proto";

    println!("cargo:rerun-if-changed={proto_file}");

    let file_descriptors = protox::compile([proto_file], [proto_root])?;
    tonic_prost_build::configure().compile_fds(file_descriptors)?;
    Ok(())
}
