//! Account deletion request lifecycle and anonymisation (T2.31a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use chrono::Utc;
use common::{Role, create_user, create_workspace, id, pool_as, set_context};
use db::{
    AccountDeletionRequest, ActorKind, AuditEvent, NewAuditEntry, anonymize_user, append_audit,
    cancel_account_deletion, due_account_deletions, request_account_deletion, verify_chain,
};
use serde_json::json;
use sqlx::{PgPool, types::Uuid};

async fn request_as(app: &PgPool, user: Uuid, grace_days: i32) -> AccountDeletionRequest {
    let mut tx = app.begin().await.unwrap();
    set_context(&mut tx, None, Some(user)).await.unwrap();
    let request = request_account_deletion(&mut tx, grace_days).await.unwrap();
    tx.commit().await.unwrap();
    request
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn request_is_idempotent_and_visible_only_to_its_user(pool: PgPool) {
    let app = pool_as(&pool, Role::Api).await;
    let (u1, u2) = (id(21), id(22));
    create_user(&app, u1).await.unwrap();
    create_user(&app, u2).await.unwrap();

    let first = request_as(&app, u1, 30).await;
    let duplicate = request_as(&app, u1, 30).await;
    let other = request_as(&app, u2, 30).await;
    assert_eq!(first.id, duplicate.id);
    assert_eq!(first.purge_at, duplicate.purge_at);
    assert_eq!(first.status, "pending");
    assert!(first.purge_at >= Utc::now() + chrono::Duration::days(29));
    assert_ne!(first.id, other.id);

    let mut tx = app.begin().await.unwrap();
    set_context(&mut tx, None, Some(u1)).await.unwrap();
    let ids: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM account_deletion_requests ORDER BY id")
        .fetch_all(&mut *tx)
        .await
        .unwrap();
    assert_eq!(ids, vec![first.id]);
    tx.commit().await.unwrap();

    let worker = pool_as(&pool, Role::Worker).await;
    let all: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM account_deletion_requests ORDER BY id")
        .fetch_all(&worker)
        .await
        .unwrap();
    assert_eq!(all.len(), 2);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn cancellation_is_user_scoped_and_records_restoration(pool: PgPool) {
    let app = pool_as(&pool, Role::Api).await;
    let (u1, u2) = (id(31), id(32));
    create_user(&app, u1).await.unwrap();
    create_user(&app, u2).await.unwrap();
    let r1 = request_as(&app, u1, 30).await;
    let r2 = request_as(&app, u2, 30).await;

    let mut tx = app.begin().await.unwrap();
    set_context(&mut tx, None, Some(u1)).await.unwrap();
    let cancelled = cancel_account_deletion(&mut tx).await.unwrap().unwrap();
    assert_eq!(cancelled.id, r1.id);
    assert_eq!(cancelled.status, "cancelled");
    assert!(cancelled.restored_at.is_some());
    tx.commit().await.unwrap();

    let mut tx = app.begin().await.unwrap();
    set_context(&mut tx, None, Some(u1)).await.unwrap();
    assert!(cancel_account_deletion(&mut tx).await.unwrap().is_none());
    tx.commit().await.unwrap();

    let status: String =
        sqlx::query_scalar("SELECT status FROM account_deletion_requests WHERE id = $1")
            .bind(r2.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "pending");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn due_requests_are_locked_and_skipped_by_another_worker(pool: PgPool) {
    let app = pool_as(&pool, Role::Api).await;
    let user = id(41);
    create_user(&app, user).await.unwrap();
    let request = request_as(&app, user, 1).await;
    sqlx::query(
        "UPDATE account_deletion_requests SET purge_at = now() - interval '1 second' WHERE id = $1",
    )
    .bind(request.id)
    .execute(&pool)
    .await
    .unwrap();

    let worker = pool_as(&pool, Role::Worker).await;
    let mut first_tx = worker.begin().await.unwrap();
    let first = due_account_deletions(&mut first_tx, 10).await.unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].request_id, request.id);
    assert_eq!(first[0].user_id, user);

    let mut second_tx = worker.begin().await.unwrap();
    let second = due_account_deletions(&mut second_tx, 10).await.unwrap();
    assert!(
        second.is_empty(),
        "a second worker must skip the locked request"
    );

    second_tx.commit().await.unwrap();
    first_tx.commit().await.unwrap();
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn anonymisation_removes_credentials_and_preserves_audit_chain(pool: PgPool) {
    let app = pool_as(&pool, Role::Api).await;
    let worker = pool_as(&pool, Role::Worker).await;
    let (user, ws) = (id(51), id(1));
    create_user(&app, user).await.unwrap();
    create_workspace(&app, ws, "Audit workspace").await.unwrap();

    let mut tx = app.begin().await.unwrap();
    set_context(&mut tx, Some(ws), Some(user)).await.unwrap();
    sqlx::query(
        "UPDATE users SET display_name = 'Personal Name', contact_email = 'person@example.com',
            contact_email_verified_at = now() WHERE id = $1",
    )
    .bind(user)
    .execute(&mut *tx)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO user_identities (id, user_id, provider, subject, provider_refresh_token_enc)
         VALUES ($1, $2, 'apple', 'apple-subject', decode('aabb', 'hex')),
                ($3, $2, 'google', 'google-subject', decode('ccdd', 'hex'))",
    )
    .bind(id(61))
    .bind(user)
    .bind(id(62))
    .execute(&mut *tx)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO passkeys (id, user_id, credential_id, public_key, name)
         VALUES ($1, $2, decode('0102', 'hex'), decode('0304', 'hex'), 'Personal key')",
    )
    .bind(id(63))
    .bind(user)
    .execute(&mut *tx)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO sessions (id, user_id, token_hash, client_kind, expires_at)
         VALUES ($1, $2, decode(repeat('11', 32), 'hex'), 'ios', now() + interval '1 day')",
    )
    .bind(id(64))
    .bind(user)
    .execute(&mut *tx)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO push_targets (id, user_id, environment, token)
         VALUES ($1, $2, 'sandbox', 'push-secret-token')",
    )
    .bind(id(65))
    .bind(user)
    .execute(&mut *tx)
    .await
    .unwrap();
    append_audit(
        &mut tx,
        ws,
        &NewAuditEntry {
            actor_kind: ActorKind::User,
            actor_id: Some(user),
            event: AuditEvent::AccountDelete,
            record_kind: "user".into(),
            record_id: Some(user),
            previous_value: Some(
                json!({"name": db::changed_field(), "email": db::changed_field()}),
            ),
            new_value: Some(json!({"deleted": true})),
            reason: None,
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let before = {
        let mut conn = pool.acquire().await.unwrap();
        verify_chain(&mut conn, ws).await.unwrap()
    };
    assert!(before.is_ok(), "{before:?}");

    let mut tx = worker.begin().await.unwrap();
    anonymize_user(&mut tx, user).await.unwrap();
    // A repeated call is safe.
    anonymize_user(&mut tx, user).await.unwrap();
    tx.commit().await.unwrap();

    let user_row: (String, Option<String>, Option<chrono::DateTime<Utc>>) =
        sqlx::query_as("SELECT display_name, contact_email, deleted_at FROM users WHERE id = $1")
            .bind(user)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(user_row.0, "Deleted user");
    assert!(user_row.1.is_none());
    assert!(user_row.2.is_some());

    let identities: Vec<(String, Option<Vec<u8>>)> = sqlx::query_as(
        "SELECT subject, provider_refresh_token_enc FROM user_identities WHERE user_id = $1 ORDER BY provider",
    )
    .bind(user)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(identities.len(), 2);
    assert!(
        identities
            .iter()
            .all(|(subject, _)| subject.starts_with("deleted:"))
    );
    assert_eq!(identities[0].1, Some(vec![0xaa, 0xbb]));
    assert_eq!(identities[1].1, None);

    let sessions: i64 = sqlx::query_scalar("SELECT count(*) FROM sessions WHERE user_id = $1")
            .bind(user)
            .fetch_one(&pool)
            .await
            .unwrap();
    let passkeys: i64 = sqlx::query_scalar("SELECT count(*) FROM passkeys WHERE user_id = $1")
            .bind(user)
            .fetch_one(&pool)
            .await
            .unwrap();
    let push_targets: i64 =
        sqlx::query_scalar("SELECT count(*) FROM push_targets WHERE user_id = $1")
            .bind(user)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((sessions, passkeys, push_targets), (0, 0, 0));

    let after = {
        let mut conn = pool.acquire().await.unwrap();
        verify_chain(&mut conn, ws).await.unwrap()
    };
    assert!(after.is_ok(), "{after:?}");
    assert_eq!(before.rows, after.rows);
}
