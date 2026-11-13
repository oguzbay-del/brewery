//! Verifies the audit log hash chains (ADR-0007 rule 5.3).
//!
//! Usage: `audit-verify --all` or `audit-verify --workspace <uuid>`. Connects with the
//! migration URL (`MIGRATE_DATABASE_URL`, else `DATABASE_URL`), which must bypass row level
//! security for `--all`. Exit code 0: every chain intact; 1: a chain is broken; 2: usage or
//! connection error.

use std::process::ExitCode;

use db::{ChainReport, Error};
use sqlx::types::Uuid;

enum Target {
    All,
    Workspace(Uuid),
}

fn parse_args(args: &[String]) -> Option<Target> {
    match args {
        [flag] if flag == "--all" => Some(Target::All),
        [flag, id] if flag == "--workspace" => id.parse().ok().map(Target::Workspace),
        _ => None,
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(target) = parse_args(&args) else {
        eprintln!("usage: audit-verify (--all | --workspace <uuid>)");
        return ExitCode::from(2);
    };
    match run(&target).await {
        Ok(reports) => report(&reports),
        Err(e) => {
            eprintln!("audit-verify failed: {e}");
            ExitCode::from(2)
        }
    }
}

async fn run(target: &Target) -> db::Result<Vec<ChainReport>> {
    let url = db::migrate_database_url()?;
    let pool = db::connect(&url, 1).await?;
    let mut conn = pool.acquire().await.map_err(Error::from)?;
    let reports = match target {
        Target::All => db::verify_all_chains(&mut conn).await?,
        Target::Workspace(id) => vec![db::verify_chain(&mut conn, *id).await?],
    };
    drop(conn);
    pool.close().await;
    Ok(reports)
}

fn report(reports: &[ChainReport]) -> ExitCode {
    let mut broken = 0usize;
    for r in reports {
        match &r.broken {
            None => println!("ok      {} ({} rows)", r.workspace_id, r.rows),
            Some(b) => {
                broken += 1;
                println!("BROKEN  {} ({} rows checked): {b}", r.workspace_id, r.rows);
            }
        }
    }
    println!("{} chains, {broken} broken", reports.len());
    if broken == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
