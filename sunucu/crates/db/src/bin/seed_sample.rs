//! Writes the sample library data (development only; every row is marked `SAMPLE DATA`).

use std::process::ExitCode;

use common::telemetry;

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(e) = telemetry::init_tracing("seed-sample") {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(error = %e, "seed failed");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> db::Result<()> {
    // Shared rows need the owner role, so the migration connection is used.
    let pool = db::connect(&db::migrate_database_url()?, 1).await?;
    db::seed::seed_sample_library(&pool).await?;
    tracing::info!("sample library data written");
    pool.close().await;
    Ok(())
}
