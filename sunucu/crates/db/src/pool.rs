//! Connection pool setup.

use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::{Error, Result};

/// Environment variable holding the connection URL.
pub const ENV_DATABASE_URL: &str = "DATABASE_URL";
/// Environment variable naming a file that holds the connection URL (mounted secret).
pub const ENV_DATABASE_URL_FILE: &str = "DATABASE_URL_FILE";
/// Environment variable holding the migration connection URL (owner-capable role). The
/// `migrate` binary prefers it over [`ENV_DATABASE_URL`], which then stays free for the
/// application role.
pub const ENV_MIGRATE_DATABASE_URL: &str = "MIGRATE_DATABASE_URL";
/// Environment variable naming a file that holds the migration connection URL.
pub const ENV_MIGRATE_DATABASE_URL_FILE: &str = "MIGRATE_DATABASE_URL_FILE";
/// Pool size used by [`connect_from_env`].
pub const DEFAULT_MAX_CONNECTIONS: u32 = 10;

/// Reads the connection URL from `DATABASE_URL` or, if that is unset, from the file named by
/// `DATABASE_URL_FILE`. Setting both is an error.
pub fn database_url() -> Result<String> {
    database_url_from_lookup(|name| std::env::var(name).ok())
}

/// Like [`database_url`] but with an injectable lookup function (testable without touching
/// the process environment).
pub fn database_url_from_lookup<F>(lookup: F) -> Result<String>
where
    F: Fn(&str) -> Option<String>,
{
    url_from_lookup(&lookup, ENV_DATABASE_URL, ENV_DATABASE_URL_FILE)
}

/// Connection URL for running migrations: `MIGRATE_DATABASE_URL` / `MIGRATE_DATABASE_URL_FILE`
/// when either is set, otherwise the ordinary [`database_url`].
pub fn migrate_database_url() -> Result<String> {
    migrate_database_url_from_lookup(|name| std::env::var(name).ok())
}

/// Like [`migrate_database_url`] with an injectable lookup function.
pub fn migrate_database_url_from_lookup<F>(lookup: F) -> Result<String>
where
    F: Fn(&str) -> Option<String>,
{
    let set = |name: &str| lookup(name).is_some_and(|v| !v.trim().is_empty());
    if set(ENV_MIGRATE_DATABASE_URL) || set(ENV_MIGRATE_DATABASE_URL_FILE) {
        url_from_lookup(
            &lookup,
            ENV_MIGRATE_DATABASE_URL,
            ENV_MIGRATE_DATABASE_URL_FILE,
        )
    } else {
        database_url_from_lookup(lookup)
    }
}

fn url_from_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
    var: &str,
    file_var: &str,
) -> Result<String> {
    let direct = lookup(var).filter(|v| !v.trim().is_empty());
    let file = lookup(file_var).filter(|v| !v.trim().is_empty());
    match (direct, file) {
        (Some(_), Some(_)) => Err(Error::AmbiguousUrl),
        (Some(url), None) => Ok(url.trim().to_owned()),
        (None, Some(path)) => {
            let content = std::fs::read_to_string(&path).map_err(|e| Error::UrlFile {
                path: path.clone(),
                reason: e.to_string(),
            })?;
            let url = content.trim();
            if url.is_empty() {
                return Err(Error::UrlFile {
                    path,
                    reason: "file is empty".to_owned(),
                });
            }
            Ok(url.to_owned())
        }
        (None, None) => Err(Error::MissingUrl),
    }
}

/// Opens a pool to `url` with at most `max_connections` connections.
pub async fn connect(url: &str, max_connections: u32) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(url)
        .await?;
    Ok(pool)
}

/// Builds a pool that connects on first use, so the process starts (and reports `503` on
/// `/readyz`) even while the database is still unavailable.
pub fn connect_lazy(url: &str, max_connections: u32) -> Result<PgPool> {
    Ok(PgPoolOptions::new()
        .max_connections(max_connections)
        .connect_lazy(url)?)
}

/// Opens a pool using the URL from the environment ([`database_url`]).
pub async fn connect_from_env() -> Result<PgPool> {
    connect(&database_url()?, DEFAULT_MAX_CONNECTIONS).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |name| {
            pairs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| (*v).to_owned())
        }
    }

    #[test]
    fn url_is_read_from_variable() {
        let url = database_url_from_lookup(lookup(&[(ENV_DATABASE_URL, " postgres://x/y \n")]));
        assert_eq!(url.unwrap(), "postgres://x/y");
    }

    #[test]
    fn url_is_read_from_file() {
        let dir = std::env::temp_dir().join(format!("db-url-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("url");
        std::fs::write(&path, "postgres://file/db\n").unwrap();
        let path_str = path.to_str().unwrap();
        let url = database_url_from_lookup(lookup(&[(ENV_DATABASE_URL_FILE, path_str)]));
        assert_eq!(url.unwrap(), "postgres://file/db");
        std::fs::write(&path, "  \n").unwrap();
        let empty = database_url_from_lookup(lookup(&[(ENV_DATABASE_URL_FILE, path_str)]));
        assert!(matches!(empty, Err(Error::UrlFile { .. })));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_and_ambiguous_are_errors() {
        assert!(matches!(
            database_url_from_lookup(lookup(&[])),
            Err(Error::MissingUrl)
        ));
        let both = lookup(&[(ENV_DATABASE_URL, "a"), (ENV_DATABASE_URL_FILE, "b")]);
        assert!(matches!(
            database_url_from_lookup(both),
            Err(Error::AmbiguousUrl)
        ));
    }

    #[test]
    fn migrate_url_is_preferred_over_the_application_url() {
        let both = lookup(&[
            (ENV_DATABASE_URL, "postgres://app/db"),
            (ENV_MIGRATE_DATABASE_URL, "postgres://owner/db"),
        ]);
        assert_eq!(
            migrate_database_url_from_lookup(both).unwrap(),
            "postgres://owner/db"
        );
        let only_app = lookup(&[(ENV_DATABASE_URL, "postgres://app/db")]);
        assert_eq!(
            migrate_database_url_from_lookup(only_app).unwrap(),
            "postgres://app/db"
        );
        assert!(matches!(
            migrate_database_url_from_lookup(lookup(&[])),
            Err(Error::MissingUrl)
        ));
    }

    #[test]
    fn unreadable_file_is_an_error() {
        let result = database_url_from_lookup(lookup(&[(
            ENV_DATABASE_URL_FILE,
            "/nonexistent/db-url-file",
        )]));
        assert!(matches!(result, Err(Error::UrlFile { .. })));
    }
}
