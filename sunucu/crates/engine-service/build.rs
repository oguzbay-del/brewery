//! Compiles the engine protobuf contract with the pure-Rust `protox` compiler
//! (no system `protoc` required).

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let proto_root = "../../../sozlesme/proto";
    let proto_files = [
        "../../../sozlesme/proto/engine/v1/engine.proto",
        "../../../sozlesme/proto/engine/v1/tool.proto",
        "../../../sozlesme/proto/engine/v1/recipe.proto",
    ];

    for file in proto_files {
        println!("cargo:rerun-if-changed={file}");
    }

    let file_descriptors = protox::compile(proto_files, [proto_root])?;
    tonic_prost_build::configure().compile_fds(file_descriptors)?;
    Ok(())
}
