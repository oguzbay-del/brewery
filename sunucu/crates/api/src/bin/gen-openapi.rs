//! Writes the `OpenAPI` document to `sozlesme/openapi/openapi.json`.
//!
//! Usage: `cargo run -p api --bin gen-openapi [-- <output-path>]`
//! (default path is resolved relative to the repository root).

use std::{path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    let path = std::env::args_os().nth(1).map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../sozlesme/openapi/openapi.json"),
        PathBuf::from,
    );
    let json = match api::openapi_json() {
        Ok(json) => json,
        Err(err) => {
            eprintln!("cannot render OpenAPI document: {err}");
            return ExitCode::FAILURE;
        }
    };
    match std::fs::write(&path, json) {
        Ok(()) => {
            println!("wrote {}", path.display());
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("cannot write {}: {err}", path.display());
            ExitCode::FAILURE
        }
    }
}
