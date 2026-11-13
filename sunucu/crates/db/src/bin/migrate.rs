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
    let url = db::migrate_database_url()?;
    // A single connection is enough; the migrator serialises runs with an advisory lock.
    let pool = db::connect(&url, 1).await?;
    db::migrate(&pool).await?;
    tracing::info!(
        migrations = db::MIGRATOR.migrations.len(),
        "migrations applied"
    );
    // Logins of the application roles: skipped (with a note) where no password is configured,
    // e.g. when only the schema is needed.
    for role in [db::AppRole::Api, db::AppRole::Worker] {
        if let Some(password) = db::password_from_env(role)? {
            db::provision_role(&pool, role, &password).await?;
            tracing::info!(role = role.name(), "role login provisioned");
        } else {
            tracing::info!(
                role = role.name(),
                variable = db::password_variable(role),
                "no password configured, role login left unchanged"
            );
        }
    }
    pool.close().await;
    Ok(())
}
