//! One-shot migration runner (compose service and Kubernetes Job).

use std::process::ExitCode;

use common::telemetry;

#[tokio::main]
async fn main() -> ExitCode {
    if let Err(e) = telemetry::init_tracing("migrate") {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(error = %e, "migration failed");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> db::Result<()> {
    let url = db::database_url()?;
    // A single connection is enough; the migrator serialises runs with an advisory lock.
    let pool = db::connect(&url, 1).await?;
    db::migrate(&pool).await?;
    tracing::info!(
        migrations = db::MIGRATOR.migrations.len(),
        "migrations applied"
    );
    pool.close().await;
    Ok(())
}
