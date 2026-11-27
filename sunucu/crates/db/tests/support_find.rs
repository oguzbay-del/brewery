//! `support_find_workspace` checks the staff context itself (T2.55, F2.2 finding G7).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{Role, create_user, create_workspace, id, pool_as};
use db::ActorKind;
use sqlx::{PgPool, types::Uuid};

struct Fixture {
    app: PgPool,
    ws: Uuid,
    number: i64,
    staff: Uuid,
}

async fn add_staff(base: &PgPool, n: u128, status: &str, deleted: bool) -> Uuid {
    let staff = id(n);
    sqlx::query(
        "INSERT INTO staff_users (id, google_subject, email, role, status, deleted_at)
         VALUES ($1, $2, $3, 'support', $4, CASE WHEN $5 THEN now() END)",
    )
    .bind(staff)
    .bind(format!("sub-{n}"))
    .bind(format!("s{n}@company.example"))
    .bind(status)
    .bind(deleted)
    .execute(base)
    .await
    .unwrap();
    staff
}

async fn fixture(base: &PgPool) -> Fixture {
    let app = pool_as(base, Role::Api).await;
    let (ws, user) = (id(1), id(2));
    create_workspace(&app, ws, "Alpha").await.unwrap();
    create_user(&app, user).await.unwrap();
    let number: i64 = sqlx::query_scalar(
        "INSERT INTO support_requests (id, workspace_id, topic, title, created_by)
         VALUES ($1, $2, 'other', 'Help', $3) RETURNING public_number",
    )
    .bind(id(3))
    .bind(ws)
    .bind(user)
    .fetch_one(base)
    .await
    .unwrap();
    let staff = add_staff(base, 10, "active", false).await;
    Fixture {
        app,
        ws,
        number,
        staff,
    }
}

/// Calls the function in a transaction with the given settings. `None` leaves a setting unset.
async fn find(
    app: &PgPool,
    actor_kind: Option<&str>,
    staff_id: Option<Uuid>,
    user_id: Option<Uuid>,
    query: &str,
) -> usize {
    let mut tx = app.begin().await.unwrap();
    for (name, value) in [
        ("app.actor_kind", actor_kind.map(str::to_owned)),
        ("app.staff_id", staff_id.map(|s| s.to_string())),
        ("app.user_id", user_id.map(|u| u.to_string())),
    ] {
        if let Some(value) = value {
            sqlx::query("SELECT set_config($1, $2, true)")
                .bind(name)
                .bind(value)
                .execute(&mut *tx)
                .await
                .unwrap();
        }
    }
    let rows: Vec<(Uuid,)> = sqlx::query_as("SELECT workspace_id FROM support_find_workspace($1)")
        .bind(query)
        .fetch_all(&mut *tx)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    rows.len()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn staff_context_finds_by_alias_id_and_number(base: PgPool) {
    let f = fixture(&base).await;
    let staff = Some(f.staff);
    let kind = Some("staff");
    assert_eq!(
        find(&f.app, kind, staff, None, &format!("pseudonym-{}", f.ws)).await,
        1
    );
    assert_eq!(find(&f.app, kind, staff, None, &f.ws.to_string()).await, 1);
    assert_eq!(
        find(&f.app, kind, staff, None, &format!("#{}", f.number)).await,
        1
    );
    assert_eq!(
        find(&f.app, kind, staff, None, &f.number.to_string()).await,
        1
    );
    assert_eq!(
        find(&f.app, kind, staff, None, "no-such-workspace").await,
        0
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn the_begin_staff_helper_gives_a_working_read_only_context(base: PgPool) {
    let f = fixture(&base).await;
    let mut tx = db::begin_staff(&f.app, f.staff).await.unwrap();
    let rows: Vec<(Uuid,)> = sqlx::query_as("SELECT workspace_id FROM support_find_workspace($1)")
        .bind(f.ws.to_string())
        .fetch_all(&mut *tx)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert!(
        sqlx::query("UPDATE workspaces SET name = 'x'")
            .execute(&mut *tx)
            .await
            .is_err(),
        "the staff transaction is read-only"
    );
    assert_eq!(ActorKind::Staff.as_str(), "staff");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_user_context_gets_no_row(base: PgPool) {
    let f = fixture(&base).await;
    let query = f.ws.to_string();
    // A user context, with and without a staff id.
    assert_eq!(
        find(&f.app, Some("user"), None, Some(id(2)), &query).await,
        0
    );
    assert_eq!(
        find(&f.app, Some("user"), Some(f.staff), Some(id(2)), &query).await,
        0
    );
    // No context at all.
    assert_eq!(find(&f.app, None, None, None, &query).await, 0);
    // The staff id alone, without the actor kind.
    assert_eq!(find(&f.app, None, Some(f.staff), None, &query).await, 0);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_wrong_actor_kind_gets_no_row(base: PgPool) {
    let f = fixture(&base).await;
    let query = f.ws.to_string();
    for kind in ["support", "system", "device", "Staff", ""] {
        assert_eq!(
            find(&f.app, Some(kind), Some(f.staff), None, &query).await,
            0,
            "actor kind {kind:?}"
        );
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_missing_or_inactive_staff_user_gets_no_row(base: PgPool) {
    let f = fixture(&base).await;
    let query = f.ws.to_string();
    let disabled = add_staff(&base, 11, "disabled", false).await;
    let deleted = add_staff(&base, 12, "active", true).await;
    for staff in [Some(disabled), Some(deleted), Some(id(99)), None] {
        assert_eq!(
            find(&f.app, Some("staff"), staff, None, &query).await,
            0,
            "staff {staff:?}"
        );
    }
    // A value that is not a UUID is an error of the database, not a row.
    let mut tx = f.app.begin().await.unwrap();
    sqlx::query(
        "SELECT set_config('app.actor_kind', 'staff', true), set_config('app.staff_id', 'x', true)",
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    assert!(
        sqlx::query("SELECT * FROM support_find_workspace($1)")
            .bind(&query)
            .fetch_all(&mut *tx)
            .await
            .is_err()
    );
}
