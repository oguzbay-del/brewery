//! End to end tests of T2.13 (ADR-0009 rules 6 and 8): preset roles against the plan's permission
//! matrix, role editing and copying, assignments and scopes, the last-administrator and
//! self-lockout protections, invitations, Home sharing, cost field hiding and audit rows.
//! Needs PostgreSQL (`#[sqlx::test]`), the compose Redis and, for one test, mailpit.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::items_after_statements, clippy::too_many_lines)]

mod support;

use std::{collections::BTreeSet, sync::Arc};

use api::{
    ApiConfig, AppState, RequestContext,
    auth::{
        AuthServices,
        email::{Mail, MailError, Mailer, MemoryMailer, SmtpMailer},
    },
    authz::{Permission, Target},
    context::BoxFuture,
    redis_client::Redis,
};
use axum::{
    Json, Router,
    body::Body,
    extract::Path,
    http::{Request, StatusCode, header},
    routing::get,
};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{Reply, app_pool, redis_url, send};
use uuid::Uuid;

const ORIGIN: &str = "https://app.example.com";

struct World {
    app: Router,
    mail: Arc<MemoryMailer>,
}

/// Sends every mail to memory and, when set, to a real SMTP relay as well.
struct Tee {
    memory: Arc<MemoryMailer>,
    smtp: Option<SmtpMailer>,
}

impl Mailer for Tee {
    fn send<'a>(&'a self, mail: &'a Mail) -> BoxFuture<'a, Result<(), MailError>> {
        Box::pin(async move {
            self.memory.send(mail).await?;
            if let Some(s) = &self.smtp {
                s.send(mail).await?;
            }
            Ok(())
        })
    }
}

async fn world_with(pool: &PgPool, redis: Redis, smtp: Option<SmtpMailer>) -> World {
    let map: std::collections::HashMap<String, String> = [
        ("DATABASE_URL", "postgres://unused"),
        ("TRUST_FORWARDED_FOR", "true"),
        ("WEB_BASE_URL", ORIGIN),
        ("CSRF_HMAC_KEY", "test-csrf-key-0123456789"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v.to_owned()))
    .collect();
    let cfg = ApiConfig::from_lookup(|n| map.get(n).cloned()).unwrap();
    let app_db = app_pool(pool).await;
    let mail = Arc::new(MemoryMailer::default());
    let tee = Arc::new(Tee {
        memory: mail.clone(),
        smtp,
    });
    let svc = Arc::new(AuthServices::new(
        app_db.clone(),
        redis.clone(),
        cfg.auth.clone(),
        tee,
    ));
    let state = AppState::new(app_db, redis, svc.clone(), cfg);
    // Probe routes stand in for resource endpoints: they show what the authenticator found and
    // how scopes and cost hiding behave.
    let probes = api::authenticated(
        &state,
        Router::new()
            .route(
                "/v1/workspaces/{ws}/probe",
                get(|ctx: RequestContext| async move {
                    Json(json!({"permissions": ctx.permissions, "scoped": ctx.scoped.len()}))
                }),
            )
            .route(
                "/v1/workspaces/{ws}/recipes/{id}/view",
                get(
                    |ctx: RequestContext, Path((_, id)): Path<(Uuid, Uuid)>| async move {
                        ctx.require_for(Permission::RecipeView, &Target::recipe(id))
                            .map(|()| Json(json!({"ok": true})))
                    },
                ),
            )
            .route(
                "/v1/workspaces/{ws}/recipes/{id}/edit",
                get(
                    |ctx: RequestContext, Path((_, id)): Path<(Uuid, Uuid)>| async move {
                        ctx.require_for(Permission::RecipeEdit, &Target::recipe(id))
                            .map(|()| Json(json!({"ok": true})))
                    },
                ),
            )
            .route(
                "/v1/workspaces/{ws}/costs",
                get(|ctx: RequestContext| async move {
                    ctx.redacted_json(
                        &json!({"name": "Malt", "unit_cost": 1.5, "cost_currency": "USD",
                        "lines": [{"kg": 2, "price": 3}]}),
                    )
                    .map(Json)
                }),
            ),
    );
    let app = api::build(state.clone(), svc.routes(&state).merge(probes));
    World { app, mail }
}

async fn world(pool: &PgPool) -> World {
    world_with(pool, Redis::disabled(), None).await
}

// --- helpers --------------------------------------------------------------------------------

struct User {
    token: String,
    id: Uuid,
}

async fn call(
    w: &World,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
    extra: &[(&str, String)],
) -> Reply {
    let mut b = Request::builder()
        .method(method)
        .uri(path)
        .header("x-forwarded-for", Uuid::new_v4().to_string());
    if let Some(t) = token {
        b = b.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    for (k, v) in extra {
        b = b.header(*k, v.as_str());
    }
    let req = match body {
        Some(v) => b
            .header(
                header::CONTENT_TYPE,
                if method == "PATCH" {
                    "application/merge-patch+json"
                } else {
                    "application/json"
                },
            )
            .body(Body::from(v.to_string()))
            .unwrap(),
        None => b.body(Body::empty()).unwrap(),
    };
    send(&w.app, req).await
}

async fn get_as(w: &World, path: &str, u: &User) -> Reply {
    call(w, "GET", path, Some(&u.token), None, &[]).await
}

async fn post_as(w: &World, path: &str, u: &User, body: Value) -> Reply {
    let key = ("idempotency-key", Uuid::new_v4().to_string());
    call(w, "POST", path, Some(&u.token), Some(body), &[key]).await
}

async fn delete_as(w: &World, path: &str, u: &User) -> Reply {
    call(w, "DELETE", path, Some(&u.token), None, &[]).await
}

async fn patch_as(w: &World, path: &str, u: &User, version: Option<i64>, body: Value) -> Reply {
    let extra: Vec<_> = version
        .map(|v| ("if-match", format!("\"{v}\"")))
        .into_iter()
        .collect();
    call(w, "PATCH", path, Some(&u.token), Some(body), &extra).await
}

fn unique_email() -> String {
    format!("friend-{}@example.com", Uuid::new_v4().simple())
}

async fn login(w: &World) -> User {
    let email = unique_email();
    let r = call(
        w,
        "POST",
        "/v1/auth/email/start",
        None,
        Some(json!({"email": email, "client": "ios"})),
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::ACCEPTED, "{:?}", r.body);
    let mail = w.mail.sent().into_iter().rfind(|m| m.to == email).unwrap();
    let token = mail.text.split("token=").nth(1).unwrap();
    let token = token.split_whitespace().next().unwrap();
    let r = call(
        w,
        "POST",
        "/v1/auth/email/verify",
        None,
        Some(json!({"token": token, "client": "ios"})),
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
    User {
        token: r.body["session"]["token"].as_str().unwrap().to_owned(),
        id: r.body["user"]["id"].as_str().unwrap().parse().unwrap(),
    }
}

async fn workspace(w: &World, owner: &User, mode: &str) -> Uuid {
    let r = post_as(
        w,
        "/v1/workspaces",
        owner,
        json!({"name": "Brewery", "mode": mode, "region": "us", "country": "US",
               "time_zone": "America/New_York"}),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{:?}", r.body);
    r.body["id"].as_str().unwrap().parse().unwrap()
}

async fn roles(w: &World, ws: Uuid, admin: &User) -> Vec<Value> {
    let r = get_as(w, &format!("/v1/workspaces/{ws}/roles"), admin).await;
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
    r.body["items"].as_array().unwrap().clone()
}

fn role_id(roles: &[Value], template: &str) -> Uuid {
    roles
        .iter()
        .find(|r| r["template_key"] == template)
        .unwrap_or_else(|| panic!("role {template}"))["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

/// Invites a fresh user with `body` (role and scope), who accepts. Returns the user and the
/// member id.
async fn join(w: &World, ws: Uuid, admin: &User, mut body: Value) -> (User, Uuid) {
    let email = unique_email();
    body["email"] = json!(email);
    let r = post_as(w, &format!("/v1/workspaces/{ws}/invitations"), admin, body).await;
    assert_eq!(r.status, StatusCode::CREATED, "{:?}", r.body);
    let guest = login(w).await;
    let r = accept(w, &guest, &token_for(w, &email)).await;
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
    let member = r.body["member_id"].as_str().unwrap().parse().unwrap();
    (guest, member)
}

fn token_for(w: &World, email: &str) -> String {
    let mail = w.mail.sent().into_iter().rfind(|m| m.to == email).unwrap();
    let t = mail.text.split("token=").nth(1).unwrap();
    t.split_whitespace().next().unwrap().to_owned()
}

async fn accept(w: &World, u: &User, token: &str) -> Reply {
    call(
        w,
        "POST",
        "/v1/invitations/accept",
        Some(&u.token),
        Some(json!({"token": token})),
        &[],
    )
    .await
}

async fn probe(w: &World, ws: Uuid, u: &User) -> Reply {
    get_as(w, &format!("/v1/workspaces/{ws}/probe"), u).await
}

fn perms_of(r: &Reply) -> BTreeSet<String> {
    r.body["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_str().unwrap().to_owned())
        .collect()
}

fn code(r: &Reply) -> &str {
    r.body["errors"][0]["code"].as_str().unwrap_or_default()
}

async fn insert_recipe(pool: &PgPool, ws: Uuid, name: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO recipes (id, workspace_id, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(ws)
        .bind(name)
        .execute(pool)
        .await
        .unwrap();
    id
}

async fn audit_actions(pool: &PgPool, ws: Uuid) -> Vec<String> {
    sqlx::query_scalar("SELECT action FROM audit_log WHERE workspace_id = $1 ORDER BY seq")
        .bind(ws)
        .fetch_all(pool)
        .await
        .unwrap()
}

// --- the matrix -----------------------------------------------------------------------------

/// The plan's matrix (plan > Yetki matrisi), written out independently of the code: rows are
/// permissions, the string says which roles have it. a admin, b brewmaster, o operator,
/// q quality, d warehouse, v viewer.
const MATRIX: [(&str, &str); 17] = [
    ("recipe.view", "aboqdv"),
    ("recipe.edit", "ab"),
    ("recipe.approve", "ab"),
    ("batch.plan", "ab"),
    ("batch.record", "abo"),
    ("batch.correct_closed", "abq"),
    ("quality.record", "aboq"),
    ("release.approve", "aq"),
    ("stock.move", "abd"),
    ("cost.view", "abd"),
    ("reports.export", "abqdv"),
    ("settings.equipment", "ab"),
    ("users.manage", "a"),
    // Not in the matrix: administrator only (ADR-0009 rule 8.1).
    ("workspace.manage", "a"),
    ("billing.manage", "a"),
    ("support.grant", "a"),
    ("devices.manage", "a"),
];

const MATRIX_ROLES: [(char, &str); 6] = [
    ('a', "admin"),
    ('b', "brewmaster"),
    ('o', "operator"),
    ('q', "quality"),
    ('d', "warehouse"),
    ('v', "viewer"),
];

#[sqlx::test(migrations = "../../migrations")]
async fn every_cell_of_the_matrix_holds_for_every_preset_role(pool: PgPool) {
    let w = world(&pool).await;
    let owner = login(&w).await;
    let ws = workspace(&w, &owner, "business").await;
    let all = roles(&w, ws, &owner).await;
    assert_eq!(all.len(), 6);

    for (letter, key) in MATRIX_ROLES {
        let (member, _) = join(&w, ws, &owner, json!({"role_id": role_id(&all, key)})).await;
        let probe_reply = probe(&w, ws, &member).await;
        assert_eq!(probe_reply.status, StatusCode::OK);
        let held = perms_of(&probe_reply);
        for (permission, who) in MATRIX {
            assert_eq!(
                held.contains(permission),
                who.contains(letter),
                "{key} x {permission}"
            );
        }
        // Managing users needs `users.manage`: 200 for the administrator, 403 for the rest.
        let r = get_as(&w, &format!("/v1/workspaces/{ws}/roles"), &member).await;
        assert_eq!(
            r.status,
            if letter == 'a' {
                StatusCode::OK
            } else {
                StatusCode::FORBIDDEN
            },
            "{key}"
        );
        for path in ["members", "invitations"] {
            let r = get_as(&w, &format!("/v1/workspaces/{ws}/{path}"), &member).await;
            assert_eq!(r.status == StatusCode::OK, letter == 'a', "{key} {path}");
        }
        // Cost fields are in the response only with `cost.view` (absent, never null).
        let costs = get_as(&w, &format!("/v1/workspaces/{ws}/costs"), &member).await;
        let has = costs.body.get("unit_cost").is_some();
        assert_eq!(has, "abd".contains(letter), "{key} cost field");
        assert_eq!(costs.body["lines"][0].get("price").is_some(), has);
        assert_eq!(costs.body["name"], "Malt");
    }
    // The code's preset table agrees with the independent matrix.
    for (letter, key) in MATRIX_ROLES {
        for (permission, who) in MATRIX {
            let p = Permission::parse(permission).unwrap();
            assert_eq!(
                api::authz::preset(key).unwrap().permissions.contains(&p),
                who.contains(letter),
                "{key} x {permission}"
            );
        }
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn home_workspaces_get_admin_editor_and_viewer_only(pool: PgPool) {
    let w = world(&pool).await;
    let owner = login(&w).await;
    let ws = workspace(&w, &owner, "home").await;
    let all = roles(&w, ws, &owner).await;
    let keys: BTreeSet<_> = all
        .iter()
        .map(|r| r["template_key"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        keys,
        ["admin", "share_editor", "share_viewer"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect()
    );
    let (editor, _) = join(
        &w,
        ws,
        &owner,
        json!({"role_id": role_id(&all, "share_editor")}),
    )
    .await;
    assert_eq!(
        perms_of(&probe(&w, ws, &editor).await),
        ["batch.record", "recipe.edit", "recipe.view"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect()
    );
    let (viewer, _) = join(
        &w,
        ws,
        &owner,
        json!({"role_id": role_id(&all, "share_viewer")}),
    )
    .await;
    assert_eq!(
        perms_of(&probe(&w, ws, &viewer).await),
        ["recipe.view".to_owned()].into_iter().collect()
    );
}

// --- 403 and 404 ----------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn missing_permission_is_403_and_foreign_workspace_records_are_404(pool: PgPool) {
    let w = world(&pool).await;
    let owner = login(&w).await;
    let ws = workspace(&w, &owner, "business").await;
    let other_owner = login(&w).await;
    let other_ws = workspace(&w, &other_owner, "business").await;
    let all = roles(&w, ws, &owner).await;
    let other_roles = roles(&w, other_ws, &other_owner).await;

    let (viewer, _) = join(&w, ws, &owner, json!({"role_id": role_id(&all, "viewer")})).await;
    let base = format!("/v1/workspaces/{ws}");
    // A member without `users.manage`: 403 on every management endpoint.
    assert_eq!(
        get_as(&w, &format!("{base}/roles"), &viewer).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        post_as(
            &w,
            &format!("{base}/roles"),
            &viewer,
            json!({"name": "X", "permissions": []})
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        post_as(
            &w,
            &format!("{base}/invitations"),
            &viewer,
            json!({"email": unique_email(), "role_id": role_id(&all, "viewer")})
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );
    // A stranger sees 404 for everything, as if the workspace did not exist.
    assert_eq!(
        get_as(&w, &format!("{base}/roles"), &other_owner)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        probe(&w, ws, &other_owner).await.status,
        StatusCode::NOT_FOUND
    );
    // A role of another workspace is 404 in this one, for reading, copying, editing, deleting.
    let foreign = role_id(&other_roles, "operator");
    let foreign_etag = other_roles
        .iter()
        .find(|r| r["id"] == foreign.to_string())
        .unwrap()["version"]
        .as_i64()
        .unwrap();
    assert_eq!(
        post_as(
            &w,
            &format!("{base}/roles/{foreign}/copy"),
            &owner,
            json!({"name": "Mine"})
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        patch_as(
            &w,
            &format!("{base}/roles/{foreign}"),
            &owner,
            Some(foreign_etag),
            json!({"name": "Mine"})
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        delete_as(&w, &format!("{base}/roles/{foreign}"), &owner)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    // Assigning a foreign role is a validation error, not a leak.
    let owner_member = get_as(&w, &format!("{base}/members"), &owner).await.body["items"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let r = post_as(
        &w,
        &format!("{base}/members/{owner_member}/assignments"),
        &owner,
        json!({"role_id": foreign}),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    // And the other workspace is untouched.
    assert_eq!(roles(&w, other_ws, &other_owner).await.len(), 6);
}

// --- last administrator and self protection ------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn the_last_administrator_cannot_be_removed_and_nobody_locks_themselves_out(pool: PgPool) {
    let w = world(&pool).await;
    let a = login(&w).await;
    let ws = workspace(&w, &a, "business").await;
    let base = format!("/v1/workspaces/{ws}");
    let all = roles(&w, ws, &a).await;
    let admin_role = role_id(&all, "admin");
    let members = get_as(&w, &format!("{base}/members"), &a).await.body["items"].clone();
    let a_member = members[0]["id"].as_str().unwrap().to_owned();
    let a_assignment = members[0]["assignments"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();

    // The only administrator: every way out is refused with `last_admin`.
    let r = delete_as(
        &w,
        &format!("{base}/members/{a_member}/assignments/{a_assignment}"),
        &a,
    )
    .await;
    assert_eq!((r.status, code(&r)), (StatusCode::CONFLICT, "last_admin"));
    let r = patch_as(
        &w,
        &format!("{base}/members/{a_member}"),
        &a,
        None,
        json!({"status": "disabled"}),
    )
    .await;
    assert_eq!(r.status, StatusCode::CONFLICT);
    let r = delete_as(&w, &format!("{base}/members/{a_member}"), &a).await;
    assert_eq!(r.status, StatusCode::CONFLICT);
    let admin_etag = all
        .iter()
        .find(|r| r["id"] == admin_role.to_string())
        .unwrap()["version"]
        .as_i64()
        .unwrap();
    let without: Vec<_> = Permission::ALL
        .iter()
        .filter(|p| **p != Permission::UsersManage)
        .map(|p| p.as_str())
        .collect();
    let r = patch_as(
        &w,
        &format!("{base}/roles/{admin_role}"),
        &a,
        Some(admin_etag),
        json!({"permissions": without}),
    )
    .await;
    assert_eq!((r.status, code(&r)), (StatusCode::CONFLICT, "last_admin"));
    // Nothing changed (the transaction rolled back).
    assert!(perms_of(&probe(&w, ws, &a).await).contains("users.manage"));

    // With a second administrator the first still cannot drop their own `users.manage`.
    let (b, b_member) = join(&w, ws, &a, json!({"role_id": admin_role})).await;
    let r = delete_as(
        &w,
        &format!("{base}/members/{a_member}/assignments/{a_assignment}"),
        &a,
    )
    .await;
    assert_eq!((r.status, code(&r)), (StatusCode::CONFLICT, "self_lockout"));
    let r = patch_as(
        &w,
        &format!("{base}/members/{a_member}"),
        &a,
        None,
        json!({"status": "disabled"}),
    )
    .await;
    assert_eq!((r.status, code(&r)), (StatusCode::CONFLICT, "self_lockout"));
    let r = delete_as(&w, &format!("{base}/members/{a_member}"), &a).await;
    assert_eq!((r.status, code(&r)), (StatusCode::CONFLICT, "self_lockout"));
    let r = patch_as(
        &w,
        &format!("{base}/roles/{admin_role}"),
        &a,
        Some(admin_etag),
        json!({"permissions": without}),
    )
    .await;
    assert_eq!(r.status, StatusCode::CONFLICT);

    // The second administrator can demote the first; then they are the last one.
    let r = delete_as(
        &w,
        &format!("{base}/members/{a_member}/assignments/{a_assignment}"),
        &b,
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT, "{:?}", r.body);
    assert_eq!(probe(&w, ws, &a).await.status, StatusCode::OK);
    assert!(perms_of(&probe(&w, ws, &a).await).is_empty());
    let b_assignment = get_as(&w, &format!("{base}/members"), &b).await.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["id"] == b_member.to_string())
        .unwrap()["assignments"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let r = delete_as(
        &w,
        &format!("{base}/members/{b_member}/assignments/{b_assignment}"),
        &b,
    )
    .await;
    assert_eq!((r.status, code(&r)), (StatusCode::CONFLICT, "last_admin"));
    let _ = a;
}

// --- roles ----------------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn roles_can_be_created_copied_edited_and_deleted_and_changes_apply_at_once(pool: PgPool) {
    // Real Redis: the permission cache must retire itself when authz_version moves.
    let w = world_with(&pool, Redis::connect_lazy(&redis_url()).unwrap(), None).await;
    let owner = login(&w).await;
    let ws = workspace(&w, &owner, "business").await;
    let base = format!("/v1/workspaces/{ws}");
    let all = roles(&w, ws, &owner).await;

    // Copy a preset, then edit the copy.
    let operator = role_id(&all, "operator");
    let r = post_as(
        &w,
        &format!("{base}/roles/{operator}/copy"),
        &owner,
        json!({"name": "Night shift"}),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{:?}", r.body);
    assert_eq!(r.body["template_key"], "operator");
    assert_eq!(
        r.body["permissions"],
        json!(["batch.record", "quality.record", "recipe.view"])
    );
    let copy = r.body["id"].as_str().unwrap().to_owned();
    // Names are unique ignoring case.
    let r = post_as(
        &w,
        &format!("{base}/roles/{operator}/copy"),
        &owner,
        json!({"name": "night SHIFT"}),
    )
    .await;
    assert_eq!((r.status, code(&r)), (StatusCode::CONFLICT, "name_taken"));
    // Unknown permission codes and bad names are validation errors.
    let r = post_as(
        &w,
        &format!("{base}/roles"),
        &owner,
        json!({"name": " ", "permissions": ["recipe.view", "nope"]}),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    let fields: Vec<_> = r.body["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["field"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(fields, ["/name", "/permissions/1"]);

    let (worker, _) = join(&w, ws, &owner, json!({"role_id": copy})).await;
    assert!(!perms_of(&probe(&w, ws, &worker).await).contains("stock.move"));

    // PATCH needs If-Match (428), a stale one is 412, a fresh one works.
    let path = format!("{base}/roles/{copy}");
    let r = patch_as(
        &w,
        &path,
        &owner,
        None,
        json!({"permissions": ["recipe.view", "stock.move"]}),
    )
    .await;
    assert_eq!(r.status, StatusCode::PRECONDITION_REQUIRED);
    let r = patch_as(&w, &path, &owner, Some(99), json!({"name": "x"})).await;
    assert_eq!(r.status, StatusCode::PRECONDITION_FAILED);
    let r = patch_as(
        &w,
        &path,
        &owner,
        Some(1),
        json!({"permissions": ["recipe.view", "stock.move"], "landing_screen": "stock"}),
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
    assert_eq!(r.body["permissions"], json!(["recipe.view", "stock.move"]));
    assert_eq!(r.body["landing_screen"], "stock");
    assert_eq!(r.body["version"], 2);
    // The holder sees the change on the very next request, cache or not.
    let held = perms_of(&probe(&w, ws, &worker).await);
    assert_eq!(
        held,
        ["recipe.view", "stock.move"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect()
    );
    let r = patch_as(
        &w,
        &path,
        &owner,
        Some(2),
        json!({"landing_screen": null, "bogus": 1}),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    let r = patch_as(&w, &path, &owner, Some(2), json!({"landing_screen": null})).await;
    assert!(r.body["landing_screen"].is_null());

    // A role in use cannot be deleted; an unused one can (and its invitations die with it).
    let r = call(
        &w,
        "DELETE",
        &path,
        Some(&owner.token),
        None,
        &[("if-match", "\"3\"".to_owned())],
    )
    .await;
    assert_eq!((r.status, code(&r)), (StatusCode::CONFLICT, "role_in_use"));
    let members = get_as(&w, &format!("{base}/members"), &owner).await.body["items"].clone();
    let worker_member = members
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["user_id"] == worker.id.to_string())
        .unwrap()
        .clone();
    let aid = worker_member["assignments"][0]["id"].as_str().unwrap();
    let wm = worker_member["id"].as_str().unwrap();
    assert_eq!(
        delete_as(
            &w,
            &format!("{base}/members/{wm}/assignments/{aid}"),
            &owner
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    assert!(perms_of(&probe(&w, ws, &worker).await).is_empty());
    let r = post_as(
        &w,
        &format!("{base}/invitations"),
        &owner,
        json!({"email": unique_email(), "role_id": copy}),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED);
    let r = call(
        &w,
        "DELETE",
        &path,
        Some(&owner.token),
        None,
        &[("if-match", "\"3\"".to_owned())],
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT, "{:?}", r.body);
    assert_eq!(
        get_as(&w, &format!("{base}/invitations"), &owner)
            .await
            .body["items"],
        json!([])
    );
    assert_eq!(roles(&w, ws, &owner).await.len(), 6);

    // Role, assignment and invitation changes are in the audit chain.
    let actions = audit_actions(&pool, ws).await;
    for expected in [
        "role.create",
        "role.permissions_set",
        "role.update",
        "role.assign",
        "role.unassign",
        "role.delete",
        "invitation.create",
        "member.status_change",
    ] {
        assert!(
            actions.iter().any(|a| a == expected),
            "{expected} in {actions:?}"
        );
    }
    let mut conn = pool.acquire().await.unwrap();
    assert!(db::verify_chain(&mut conn, ws).await.unwrap().is_ok());
}

// --- scope ----------------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn a_scoped_assignment_only_holds_inside_its_scope(pool: PgPool) {
    let w = world(&pool).await;
    let owner = login(&w).await;
    let ws = workspace(&w, &owner, "business").await;
    let base = format!("/v1/workspaces/{ws}");
    let all = roles(&w, ws, &owner).await;
    let (recipe_a, recipe_b) = (
        insert_recipe(&pool, ws, "A").await,
        insert_recipe(&pool, ws, "B").await,
    );
    let brewmaster = role_id(&all, "brewmaster");

    // A scoped invitation: the role holds for recipe A only.
    let (guest, member) = join(
        &w,
        ws,
        &owner,
        json!({"role_id": brewmaster, "scope_kind": "recipe", "scope_id": recipe_a}),
    )
    .await;
    let p = probe(&w, ws, &guest).await;
    assert_eq!(p.status, StatusCode::OK);
    assert!(perms_of(&p).is_empty(), "no workspace-wide permission");
    assert_eq!(p.body["scoped"], 1);
    let edit = |id: Uuid| format!("{base}/recipes/{id}/edit");
    assert_eq!(
        get_as(&w, &edit(recipe_a), &guest).await.status,
        StatusCode::OK
    );
    assert_eq!(
        get_as(&w, &format!("{base}/recipes/{recipe_a}/view"), &guest)
            .await
            .status,
        StatusCode::OK
    );
    // Outside the scope the record does not exist for the guest.
    assert_eq!(
        get_as(&w, &edit(recipe_b), &guest).await.status,
        StatusCode::NOT_FOUND
    );
    // Management stays closed.
    assert_eq!(
        get_as(&w, &format!("{base}/roles"), &guest).await.status,
        StatusCode::FORBIDDEN
    );
    // The same guest, also scoped to B with another role, now reaches B too.
    let viewer = role_id(&all, "viewer");
    let r = post_as(
        &w,
        &format!("{base}/members/{member}/assignments"),
        &owner,
        json!({"role_id": viewer, "scope_kind": "recipe", "scope_id": recipe_b}),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{:?}", r.body);
    assert_eq!(
        get_as(&w, &format!("{base}/recipes/{recipe_b}/view"), &guest)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        get_as(&w, &edit(recipe_b), &guest).await.status,
        StatusCode::FORBIDDEN,
        "viewer cannot edit B"
    );
    // Same assignment twice is a conflict.
    let r = post_as(
        &w,
        &format!("{base}/members/{member}/assignments"),
        &owner,
        json!({"role_id": viewer, "scope_kind": "recipe", "scope_id": recipe_b}),
    )
    .await;
    assert_eq!(
        (r.status, code(&r)),
        (StatusCode::CONFLICT, "already_assigned")
    );
    // A workspace-wide role (administrator) cannot be scoped; unknown targets are refused;
    // a half-given scope is a validation error.
    let admin = role_id(&all, "admin");
    let r = post_as(
        &w,
        &format!("{base}/members/{member}/assignments"),
        &owner,
        json!({"role_id": admin, "scope_kind": "recipe", "scope_id": recipe_a}),
    )
    .await;
    assert_eq!(
        (r.status, code(&r)),
        (StatusCode::UNPROCESSABLE_ENTITY, "role_not_scopable")
    );
    let r = post_as(
        &w,
        &format!("{base}/members/{member}/assignments"),
        &owner,
        json!({"role_id": viewer, "scope_kind": "recipe", "scope_id": Uuid::now_v7()}),
    )
    .await;
    assert_eq!(
        (r.status, code(&r)),
        (StatusCode::UNPROCESSABLE_ENTITY, "scope_target_not_found")
    );
    let r = post_as(
        &w,
        &format!("{base}/members/{member}/assignments"),
        &owner,
        json!({"role_id": viewer, "scope_kind": "recipe"}),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    // Another workspace's recipe cannot be a scope here.
    let other = login(&w).await;
    let other_ws = workspace(&w, &other, "business").await;
    let foreign = insert_recipe(&pool, other_ws, "Z").await;
    let r = post_as(
        &w,
        &format!("{base}/members/{member}/assignments"),
        &owner,
        json!({"role_id": viewer, "scope_kind": "recipe", "scope_id": foreign}),
    )
    .await;
    assert_eq!(
        (r.status, code(&r)),
        (StatusCode::UNPROCESSABLE_ENTITY, "scope_target_not_found")
    );
}

// --- invitations ----------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn an_invitation_works_once_for_seven_days_and_can_be_revoked(pool: PgPool) {
    let w = world(&pool).await;
    let owner = login(&w).await;
    let ws = workspace(&w, &owner, "business").await;
    let base = format!("/v1/workspaces/{ws}");
    let all = roles(&w, ws, &owner).await;
    let role = role_id(&all, "operator");
    let (wr, ow, bs) = (&w, &owner, &base);
    let invite = |email: String| async move {
        let body = json!({"email": email, "role_id": role});
        post_as(wr, &format!("{bs}/invitations"), ow, body).await
    };

    // Created with a 7 day life; the mail carries the link; only a digest of the token is stored.
    let email = unique_email();
    let r = invite(email.clone()).await;
    assert_eq!(r.status, StatusCode::CREATED, "{:?}", r.body);
    assert_eq!(r.body["email_sent"], true);
    let expires: chrono::DateTime<chrono::Utc> =
        r.body["expires_at"].as_str().unwrap().parse().unwrap();
    let days = (expires - chrono::Utc::now()).num_hours();
    assert!((167..=168).contains(&days), "{days}");
    let mail = w.mail.sent().into_iter().rfind(|m| m.to == email).unwrap();
    assert!(mail.text.contains("/invite?token="));
    assert!(mail.text.contains("7 days"));
    let token = token_for(&w, &email);
    let stored: i64 =
        sqlx::query_scalar("SELECT count(*) FROM invitations WHERE token_hash = sha256($1::bytea)")
            .bind(token.as_bytes())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stored, 1, "token_hash is the SHA-256 of the token");
    assert_eq!(
        get_as(&w, &format!("{base}/invitations"), &owner)
            .await
            .body["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    // Bad addresses are refused; the address is not stored anywhere.
    assert_eq!(
        invite("not an address".into()).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );

    // Accepting needs a sign-in; garbage and unknown tokens are 404.
    let guest = login(&w).await;
    let r = call(
        &w,
        "POST",
        "/v1/invitations/accept",
        None,
        Some(json!({"token": token})),
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        accept(&w, &guest, "garbage").await.status,
        StatusCode::NOT_FOUND
    );
    // The first use works, the second (same or another user) is 404.
    let r = accept(&w, &guest, &token).await;
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
    assert_eq!(r.body["workspace_id"], ws.to_string());
    assert_eq!(
        accept(&w, &guest, &token).await.status,
        StatusCode::NOT_FOUND
    );
    let second = login(&w).await;
    assert_eq!(
        accept(&w, &second, &token).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(probe(&w, ws, &second).await.status, StatusCode::NOT_FOUND);
    assert_eq!(perms_of(&probe(&w, ws, &guest).await).len(), 3);
    assert!(
        get_as(&w, &format!("{base}/invitations"), &owner)
            .await
            .body["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    // Concurrent use of one token joins exactly one account.
    let email = unique_email();
    invite(email.clone()).await;
    let token = token_for(&w, &email);
    let (u1, u2) = (login(&w).await, login(&w).await);
    let (a, b) = tokio::join!(accept(&w, &u1, &token), accept(&w, &u2, &token));
    let mut statuses = [a.status, b.status];
    statuses.sort();
    assert_eq!(statuses, [StatusCode::OK, StatusCode::NOT_FOUND]);

    // After 7 days the token is dead (and `expires_at` is what decides).
    let email = unique_email();
    invite(email.clone()).await;
    let token = token_for(&w, &email);
    sqlx::query(
        "UPDATE invitations SET expires_at = now() - interval '1 second' WHERE accepted_at IS NULL",
    )
    .execute(&pool)
    .await
    .unwrap();
    let late = login(&w).await;
    assert_eq!(
        accept(&w, &late, &token).await.status,
        StatusCode::NOT_FOUND
    );
    assert!(
        get_as(&w, &format!("{base}/invitations"), &owner)
            .await
            .body["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    // A revoked invitation is dead too; revoking twice is 404; strangers cannot revoke.
    let email = unique_email();
    let r = invite(email.clone()).await;
    let id = r.body["id"].as_str().unwrap().to_owned();
    let token = token_for(&w, &email);
    assert_eq!(
        delete_as(&w, &format!("{base}/invitations/{id}"), &guest)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        delete_as(&w, &format!("{base}/invitations/{id}"), &owner)
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        delete_as(&w, &format!("{base}/invitations/{id}"), &owner)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        accept(&w, &late, &token).await.status,
        StatusCode::NOT_FOUND
    );

    // A disabled member cannot rejoin through a new invitation; a removed one can.
    let members = get_as(&w, &format!("{base}/members"), &owner).await.body["items"].clone();
    let guest_member = members
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["user_id"] == guest.id.to_string())
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let r = patch_as(
        &w,
        &format!("{base}/members/{guest_member}"),
        &owner,
        None,
        json!({"status": "disabled"}),
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{:?}", r.body);
    assert_eq!(
        probe(&w, ws, &guest).await.status,
        StatusCode::NOT_FOUND,
        "disabled loses access at once"
    );
    let email = unique_email();
    invite(email.clone()).await;
    let r = accept(&w, &guest, &token_for(&w, &email)).await;
    assert_eq!(
        (r.status, code(&r)),
        (StatusCode::CONFLICT, "member_disabled")
    );
    // The conflict rolled back: the token is still usable by someone else.
    let r = accept(&w, &late, &token_for(&w, &email)).await;
    assert_eq!(r.status, StatusCode::OK);
    let r = patch_as(
        &w,
        &format!("{base}/members/{guest_member}"),
        &owner,
        None,
        json!({"status": "active"}),
    )
    .await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(probe(&w, ws, &guest).await.status, StatusCode::OK);
    assert_eq!(
        delete_as(&w, &format!("{base}/members/{guest_member}"), &owner)
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(probe(&w, ws, &guest).await.status, StatusCode::NOT_FOUND);
    let email = unique_email();
    invite(email.clone()).await;
    assert_eq!(
        accept(&w, &guest, &token_for(&w, &email)).await.status,
        StatusCode::OK
    );
    assert_eq!(probe(&w, ws, &guest).await.status, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn invitation_mail_reaches_mailpit(pool: PgPool) {
    let smtp = SmtpMailer::new("127.0.0.1", 1025, "no-reply@demleme.local");
    let w = world_with(&pool, Redis::disabled(), Some(smtp)).await;
    let owner = login(&w).await;
    let ws = workspace(&w, &owner, "business").await;
    let all = roles(&w, ws, &owner).await;
    let to = unique_email();
    let r = post_as(
        &w,
        &format!("/v1/workspaces/{ws}/invitations"),
        &owner,
        json!({"email": to, "role_id": role_id(&all, "viewer")}),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{:?}", r.body);
    assert_eq!(r.body["email_sent"], true);
    let client = reqwest::Client::new();
    let mut found = None;
    for _ in 0..30 {
        let listed: Value = client
            .get(format!(
                "http://127.0.0.1:8025/api/v1/search?query=to%3A{}",
                to.replace('@', "%40")
            ))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        if let Some(m) = listed["messages"].as_array().and_then(|m| m.first()) {
            found = Some(m.clone());
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    let found = found.expect("invitation arrived in mailpit");
    assert_eq!(found["Subject"], "You have been invited to a brewery");
}

// --- Home sharing ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../../migrations")]
async fn home_owner_shares_a_recipe_as_editor_or_viewer(pool: PgPool) {
    let w = world(&pool).await;
    let owner = login(&w).await;
    let ws = workspace(&w, &owner, "home").await;
    let base = format!("/v1/workspaces/{ws}");
    let (mine, other) = (
        insert_recipe(&pool, ws, "IPA").await,
        insert_recipe(&pool, ws, "Stout").await,
    );

    let (wr, ow, bs) = (&w, &owner, &base);
    let share = |access: &'static str, email: String| async move {
        let body = json!({"kind": "recipe", "id": mine, "access": access, "email": email});
        post_as(wr, &format!("{bs}/shares"), ow, body).await
    };
    // Viewer: sees the shared recipe, cannot edit it, does not know the other one.
    let email = unique_email();
    let r = share("viewer", email.clone()).await;
    assert_eq!(r.status, StatusCode::CREATED, "{:?}", r.body);
    assert_eq!(r.body["scope_kind"], "recipe");
    let friend = login(&w).await;
    assert_eq!(
        accept(&w, &friend, &token_for(&w, &email)).await.status,
        StatusCode::OK
    );
    assert_eq!(
        get_as(&w, &format!("{base}/recipes/{mine}/view"), &friend)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        get_as(&w, &format!("{base}/recipes/{mine}/edit"), &friend)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        get_as(&w, &format!("{base}/recipes/{other}/view"), &friend)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get_as(&w, &format!("{base}/roles"), &friend).await.status,
        StatusCode::FORBIDDEN
    );

    // Editor: can edit the shared recipe only.
    let email = unique_email();
    assert_eq!(
        share("editor", email.clone()).await.status,
        StatusCode::CREATED
    );
    let friend2 = login(&w).await;
    assert_eq!(
        accept(&w, &friend2, &token_for(&w, &email)).await.status,
        StatusCode::OK
    );
    assert_eq!(
        get_as(&w, &format!("{base}/recipes/{mine}/edit"), &friend2)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        get_as(&w, &format!("{base}/recipes/{other}/edit"), &friend2)
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    // Removing the assignment ends the share at once.
    let members = get_as(&w, &format!("{base}/members"), &owner).await.body["items"].clone();
    let m = members
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["user_id"] == friend2.id.to_string())
        .unwrap()
        .clone();
    let (mid, aid) = (
        m["id"].as_str().unwrap(),
        m["assignments"][0]["id"].as_str().unwrap(),
    );
    assert_eq!(
        delete_as(
            &w,
            &format!("{base}/members/{mid}/assignments/{aid}"),
            &owner
        )
        .await
        .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        get_as(&w, &format!("{base}/recipes/{mine}/edit"), &friend2)
            .await
            .status,
        StatusCode::NOT_FOUND
    );

    // Bad input: unknown kind or access, a target that is not there, a friend (no users.manage).
    let r = post_as(
        &w,
        &format!("{base}/shares"),
        &owner,
        json!({"kind": "tank", "id": mine, "access": "viewer", "email": unique_email()}),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    let r = post_as(
        &w,
        &format!("{base}/shares"),
        &owner,
        json!({"kind": "recipe", "id": mine, "access": "owner", "email": unique_email()}),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    let r = post_as(
        &w,
        &format!("{base}/shares"),
        &owner,
        json!({"kind": "batch", "id": Uuid::now_v7(), "access": "viewer", "email": unique_email()}),
    )
    .await;
    assert_eq!(
        (r.status, code(&r)),
        (StatusCode::UNPROCESSABLE_ENTITY, "scope_target_not_found")
    );
    let r = post_as(
        &w,
        &format!("{base}/shares"),
        &friend,
        json!({"kind": "recipe", "id": mine, "access": "viewer", "email": unique_email()}),
    )
    .await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);

    // Business workspaces have no sharing endpoint effect.
    let biz_owner = login(&w).await;
    let biz = workspace(&w, &biz_owner, "business").await;
    let recipe = insert_recipe(&pool, biz, "Lager").await;
    let r = post_as(
        &w,
        &format!("/v1/workspaces/{biz}/shares"),
        &biz_owner,
        json!({"kind": "recipe", "id": recipe, "access": "viewer", "email": unique_email()}),
    )
    .await;
    assert_eq!(
        (r.status, code(&r)),
        (StatusCode::CONFLICT, "home_mode_only")
    );
}
