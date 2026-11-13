//! Compiles the engine protobuf contract into client stubs only (no server code, and no
//! dependency on the `engine` calculation library: the API links this crate, not
//! `engine-service`).

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let proto_root = "../../../sozlesme/proto";
    let proto_files = [
        "../../../sozlesme/proto/engine/v1/engine.proto",
        "../../../sozlesme/proto/engine/v1/tool.proto",
        "../../../sozlesme/proto/engine/v1/recipe.proto",
        "../../../sozlesme/proto/engine/v1/mash.proto",
        "../../../sozlesme/proto/engine/v1/water.proto",
        "../../../sozlesme/proto/engine/v1/yeast.proto",
        "../../../sozlesme/proto/engine/v1/fermentation.proto",
        "../../../sozlesme/proto/engine/v1/brewery.proto",
    ];
    for file in proto_files {
        println!("cargo:rerun-if-changed={file}");
    }
    let file_descriptors = protox::compile(proto_files, [proto_root])?;
    tonic_prost_build::configure()
        .build_server(false)
        .compile_fds(file_descriptors)?;
    Ok(())
}
