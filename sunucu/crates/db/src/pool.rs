//! Connection pool setup.

use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::{Error, Result};

/// Environment variable holding the connection URL.
pub const ENV_DATABASE_URL: &str = "DATABASE_URL";
/// Environment variable naming a file that holds the connection URL (mounted secret).
pub const ENV_DATABASE_URL_FILE: &str = "DATABASE_URL_FILE";
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
    let direct = lookup(ENV_DATABASE_URL).filter(|v| !v.trim().is_empty());
    let file = lookup(ENV_DATABASE_URL_FILE).filter(|v| !v.trim().is_empty());
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
    fn unreadable_file_is_an_error() {
        let result = database_url_from_lookup(lookup(&[(
            ENV_DATABASE_URL_FILE,
            "/nonexistent/db-url-file",
        )]));
        assert!(matches!(result, Err(Error::UrlFile { .. })));
    }
}
