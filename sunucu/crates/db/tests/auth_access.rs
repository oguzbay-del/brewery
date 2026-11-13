//! Account tables and the pre-login lookups (ADR-0007, rules 2.1 and 2.6).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{
    Role, create_role, create_user, create_workspace, id, pool_as, set_context, sqlstate,
};
use sqlx::{PgPool, types::Uuid};

const INSUFFICIENT_PRIVILEGE: &str = "42501";

async fn add_identity(app: &PgPool, user: Uuid, identity: Uuid, subject: &str) -> sqlx::Result<()> {
    let mut tx = app.begin().await?;
    set_context(&mut tx, None, Some(user)).await?;
    sqlx::query(
        "INSERT INTO user_identities (id, user_id, provider, subject) VALUES ($1, $2, 'apple', $3)",
    )
    .bind(identity)
    .bind(user)
    .bind(subject)
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_user_sees_only_their_own_account_rows(pool: PgPool) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let (u1, u2) = (id(21), id(22));
    create_user(&app, u1).await?;
    create_user(&app, u2).await?;
    add_identity(&app, u1, id(51), "apple-1").await?;
    add_identity(&app, u2, id(52), "apple-2").await?;

    let mut tx = app.begin().await?;
    set_context(&mut tx, None, Some(u1)).await?;
    let users: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM users")
        .fetch_all(&mut *tx)
        .await?;
    assert_eq!(users, vec![u1]);
    let identities: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM user_identities")
        .fetch_all(&mut *tx)
        .await?;
    assert_eq!(identities, vec![id(51)]);
    // Writing a row for somebody else is refused.
    let err = sqlx::query("INSERT INTO user_identities (id, user_id, provider, subject) VALUES ($1, $2, 'google', 'g')")
        .bind(id(53))
        .bind(u2)
        .execute(&mut *tx)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some("42501"));
    drop(tx);

    // No context, no rows.
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&app)
        .await?;
    assert_eq!(n, 0);
    // A provider identity is unique across users.
    let mut tx = app.begin().await?;
    set_context(&mut tx, None, Some(u2)).await?;
    let err = sqlx::query("INSERT INTO user_identities (id, user_id, provider, subject) VALUES ($1, $2, 'apple', 'apple-1')")
        .bind(id(54))
        .bind(u2)
        .execute(&mut *tx)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some("23505"));
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn identity_lookup_works_without_context_and_returns_one_row(
    pool: PgPool,
) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let u1 = id(21);
    create_user(&app, u1).await?;
    add_identity(&app, u1, id(51), "apple-1").await?;

    let found: Option<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT user_id, identity_id FROM find_user_by_identity('apple', 'apple-1')",
    )
    .fetch_optional(&app)
    .await?;
    assert_eq!(found, Some((u1, id(51))));
    let none: Option<(Uuid, Uuid)> =
        sqlx::query_as("SELECT user_id, identity_id FROM find_user_by_identity('apple', 'nobody')")
            .fetch_optional(&app)
            .await?;
    assert!(none.is_none());

    // The table itself stays closed without context.
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM user_identities")
        .fetch_one(&app)
        .await?;
    assert_eq!(n, 0);
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn session_lookup_by_token_hash(pool: PgPool) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let u1 = id(21);
    create_user(&app, u1).await?;
    let hash = vec![7u8; 32];
    let mut tx = app.begin().await?;
    set_context(&mut tx, None, Some(u1)).await?;
    sqlx::query(
        "INSERT INTO sessions (id, user_id, token_hash, client_kind, expires_at)
         VALUES ($1, $2, $3, 'ios', now() + interval '90 days')",
    )
    .bind(id(61))
    .bind(u1)
    .bind(&hash)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    let found: Option<(Uuid, Uuid, String)> =
        sqlx::query_as("SELECT id, user_id, client_kind FROM find_session_by_token_hash($1)")
            .bind(&hash)
            .fetch_optional(&app)
            .await?;
    assert_eq!(found, Some((id(61), u1, "ios".to_owned())));
    let miss: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM find_session_by_token_hash($1)")
        .bind(vec![8u8; 32])
        .fetch_optional(&app)
        .await?;
    assert!(miss.is_none());
    // A shared-device session needs its device (and vice versa).
    let mut tx = app.begin().await?;
    set_context(&mut tx, None, Some(u1)).await?;
    let err = sqlx::query(
        "INSERT INTO sessions (id, user_id, token_hash, client_kind, expires_at)
         VALUES ($1, $2, $3, 'shared_device', now() + interval '12 hours')",
    )
    .bind(id(62))
    .bind(u1)
    .bind(vec![9u8; 32])
    .execute(&mut *tx)
    .await
    .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some("23514"));
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn email_tokens_and_auth_requests_are_single_use(pool: PgPool) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;

    // The tables are closed to the application role; only the functions work.
    for table in ["email_login_tokens", "auth_requests"] {
        let sql = if table == "email_login_tokens" {
            "SELECT count(*) FROM email_login_tokens"
        } else {
            "SELECT count(*) FROM auth_requests"
        };
        let err = sqlx::query_scalar::<_, i64>(sql)
            .fetch_one(&app)
            .await
            .unwrap_err();
        assert_eq!(
            sqlstate(&err).as_deref(),
            Some(INSUFFICIENT_PRIVILEGE),
            "{table}"
        );
    }

    let hash = vec![3u8; 32];
    sqlx::query(
        "SELECT email_login_token_create($1, $2, 'a@example.com', now() + interval '15 minutes')",
    )
    .bind(id(71))
    .bind(&hash)
    .execute(&app)
    .await?;
    let first: Option<(String,)> =
        sqlx::query_as("SELECT email FROM email_login_token_consume($1)")
            .bind(&hash)
            .fetch_optional(&app)
            .await?;
    assert_eq!(first, Some(("a@example.com".to_owned(),)));
    let again: Option<(String,)> =
        sqlx::query_as("SELECT email FROM email_login_token_consume($1)")
            .bind(&hash)
            .fetch_optional(&app)
            .await?;
    assert!(again.is_none(), "token consumed twice");

    // Expired tokens cannot be consumed.
    sqlx::query(
        "SELECT email_login_token_create($1, $2, 'b@example.com', now() - interval '1 minute')",
    )
    .bind(id(72))
    .bind(vec![4u8; 32])
    .execute(&app)
    .await?;
    let expired: Option<(String,)> =
        sqlx::query_as("SELECT email FROM email_login_token_consume($1)")
            .bind(vec![4u8; 32])
            .fetch_optional(&app)
            .await?;
    assert!(expired.is_none());

    // Auth requests: by nonce or by state, once.
    sqlx::query("SELECT auth_request_create($1, 'nonce-1', 'state-1', 'verifier', now() + interval '10 minutes')")
        .bind(id(73))
        .execute(&app)
        .await?;
    let by_state: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT nonce, pkce_verifier FROM auth_request_consume(NULL, 'state-1')")
            .fetch_optional(&app)
            .await?;
    assert_eq!(
        by_state,
        Some(("nonce-1".to_owned(), Some("verifier".to_owned())))
    );
    let reuse: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT nonce, pkce_verifier FROM auth_request_consume('nonce-1', NULL)")
            .fetch_optional(&app)
            .await?;
    assert!(reuse.is_none(), "nonce reused");
    // Exactly one key must be given.
    let neither: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT nonce, pkce_verifier FROM auth_request_consume(NULL, NULL)")
            .fetch_optional(&app)
            .await?;
    assert!(neither.is_none());
    Ok(())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn invitation_and_device_lookups_return_only_valid_rows(pool: PgPool) -> sqlx::Result<()> {
    let app = pool_as(&pool, Role::Api).await;
    let (ws, role, user) = (id(1), id(11), id(21));
    create_workspace(&app, ws, "A").await?;
    create_role(&app, ws, role, "Viewer").await?;
    create_user(&app, user).await?;

    let mut tx = app.begin().await?;
    set_context(&mut tx, Some(ws), Some(user)).await?;
    for (n, offset, revoked) in [
        (1u8, "7 days", false),
        (2, "-1 minute", false),
        (3, "7 days", true),
    ] {
        sqlx::query(
            "INSERT INTO invitations (id, workspace_id, token_hash, role_id, expires_at, revoked_at, created_by)
             VALUES ($1, $2, $3, $4, now() + $5::interval, CASE WHEN $6 THEN now() END, $7)",
        )
        .bind(id(80 + u128::from(n)))
        .bind(ws)
        .bind(vec![n; 32])
        .bind(role)
        .bind(offset)
        .bind(revoked)
        .bind(user)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query(
        "INSERT INTO shared_devices (id, workspace_id, name, opened_by, device_key_hash)
         VALUES ($1, $2, 'Bar iPad', $3, $4)",
    )
    .bind(id(90))
    .bind(ws)
    .bind(user)
    .bind(vec![9u8; 32])
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    // Without any context: valid invitation found, expired and revoked ones are not.
    for (n, valid) in [(1u8, true), (2, false), (3, false)] {
        let found: Option<(Uuid, Uuid)> =
            sqlx::query_as("SELECT id, workspace_id FROM find_invitation_by_token_hash($1)")
                .bind(vec![n; 32])
                .fetch_optional(&app)
                .await?;
        assert_eq!(found.is_some(), valid, "invitation {n}");
    }
    let device: Option<(Uuid, Uuid, i32)> = sqlx::query_as(
        "SELECT id, workspace_id, idle_timeout_min FROM find_shared_device_by_key_hash($1)",
    )
    .bind(vec![9u8; 32])
    .fetch_optional(&app)
    .await?;
    assert_eq!(device, Some((id(90), ws, 5)));

    // The tables stay invisible without context.
    let n: i64 = sqlx::query_scalar(
        "SELECT (SELECT count(*) FROM invitations) + (SELECT count(*) FROM shared_devices)",
    )
    .fetch_one(&app)
    .await?;
    assert_eq!(n, 0);
    // The worker may not use the pre-login functions.
    let worker = pool_as(&pool, Role::Worker).await;
    let err = sqlx::query("SELECT * FROM find_invitation_by_token_hash($1)")
        .bind(vec![1u8; 32])
        .execute(&worker)
        .await
        .unwrap_err();
    assert_eq!(sqlstate(&err).as_deref(), Some(INSUFFICIENT_PRIVILEGE));
    Ok(())
}
