//! Login credentials for the application roles (ADR-0007, rule 2.4).
//!
//! Migrations create `app_owner`, `app_api` and `app_worker` without login or password. After
//! migrating, the `migrate` binary gives the two application roles `LOGIN` and the password
//! from the environment; running it again rotates the password. `app_owner` never logs in.

use sqlx::PgPool;

use crate::{Error, Result};

/// Roles that can be given a login.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppRole {
    /// `app_api`: API and ingest.
    Api,
    /// `app_worker`: background jobs.
    Worker,
}

impl AppRole {
    /// The PostgreSQL role name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Api => "app_api",
            Self::Worker => "app_worker",
        }
    }
}

/// Environment variable holding the password of `role` (`APP_API_PASSWORD`, ...).
#[must_use]
pub const fn password_variable(role: AppRole) -> &'static str {
    match role {
        AppRole::Api => "APP_API_PASSWORD",
        AppRole::Worker => "APP_WORKER_PASSWORD",
    }
}

/// Reads the password of `role` from its variable or, if unset, from the file named by the
/// same variable plus `_FILE` (mounted secret). `Ok(None)` when neither is set.
pub fn password_from_env(role: AppRole) -> Result<Option<String>> {
    password_from_lookup(role, |name| std::env::var(name).ok())
}

/// Like [`password_from_env`] with an injectable lookup function.
pub fn password_from_lookup<F>(role: AppRole, lookup: F) -> Result<Option<String>>
where
    F: Fn(&str) -> Option<String>,
{
    let var = password_variable(role);
    let file_var = format!("{var}_FILE");
    let direct = lookup(var).filter(|v| !v.is_empty());
    let file = lookup(&file_var).filter(|v| !v.trim().is_empty());
    match (direct, file) {
        (Some(_), Some(_)) => Err(Error::AmbiguousSecret {
            variable: var.to_owned(),
        }),
        (Some(password), None) => Ok(Some(password)),
        (None, Some(path)) => {
            let content = std::fs::read_to_string(&path).map_err(|e| Error::SecretFile {
                path: path.clone(),
                reason: e.to_string(),
            })?;
            let password = content.trim_end_matches(['\n', '\r']);
            if password.is_empty() {
                return Err(Error::SecretFile {
                    path,
                    reason: "file is empty".to_owned(),
                });
            }
            Ok(Some(password.to_owned()))
        }
        (None, None) => Ok(None),
    }
}

/// Gives `role` `LOGIN` and `password`. Needs a role that may alter it (the migration role).
///
/// The statement is assembled by the server (`format(%I, %L)`), so the password is quoted
/// there and never interpolated into SQL text on the client.
pub async fn provision_role(pool: &PgPool, role: AppRole, password: &str) -> Result<()> {
    let mut conn = pool.acquire().await?;
    let statement: String = sqlx::query_scalar(
        "SELECT format('ALTER ROLE %I WITH LOGIN PASSWORD %L', $1::text, $2::text)",
    )
    .bind(role.name())
    .bind(password)
    .fetch_one(&mut *conn)
    .await?;
    // SAFETY of the text: built by the server with %I/%L quoting (see above).
    sqlx::raw_sql(sqlx::AssertSqlSafe(statement))
        .execute(&mut *conn)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_comes_from_the_variable_or_nothing() {
        let set = |name: &str| (name == "APP_API_PASSWORD").then(|| "s3cret".to_owned());
        assert_eq!(
            password_from_lookup(AppRole::Api, set).unwrap().as_deref(),
            Some("s3cret")
        );
        assert_eq!(password_from_lookup(AppRole::Worker, set).unwrap(), None);
    }

    #[test]
    fn both_variable_and_file_is_ambiguous() {
        let both = |name: &str| match name {
            "APP_WORKER_PASSWORD" => Some("a".to_owned()),
            "APP_WORKER_PASSWORD_FILE" => Some("/x".to_owned()),
            _ => None,
        };
        assert!(password_from_lookup(AppRole::Worker, both).is_err());
    }
}
