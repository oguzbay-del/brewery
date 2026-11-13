//! One or more tests per rule of ADR-0008, against the real layers and a demo resource.
//! Needs PostgreSQL (`DATABASE_URL`, `#[sqlx::test]`) and, for rate limit tests, the compose
//! Redis on 127.0.0.1:6379.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::sync::{Arc, atomic::Ordering};

use api::{ApiConfig, redis_client::Redis};
use axum::{Router, body::Body, http::header};
use serde_json::json;
use sqlx::PgPool;
use support::{
    Demo, FakeAuth, RedisProxy, config, demo_router, insert_user, insert_workspace, json_post,
    redis_url, req, send, state_with,
};
use uuid::Uuid;

struct World {
    app: Router,
    demo: Arc<Demo>,
    auth: Arc<FakeAuth>,
    user: Uuid,
    ws: Uuid,
}

async fn world_with(pool: &PgPool, redis: Redis, cfg: ApiConfig) -> World {
    let user = Uuid::now_v7();
    let ws = Uuid::now_v7();
    insert_user(pool, user).await;
    insert_workspace(pool, ws).await;
    let auth = Arc::new(FakeAuth::default());
    auth.add(user, ws, &["recipe.view"]);
    let state = state_with(pool, redis, auth.clone(), cfg).await;
    let demo = Demo::with_items(7);
    World {
        app: demo_router(&state, demo.clone()),
        demo,
        auth,
        user,
        ws,
    }
}

async fn world(pool: &PgPool) -> World {
    world_with(pool, Redis::disabled(), config("postgres://unused")).await
}

fn cfg_with(extra: &'static [(&'static str, &'static str)]) -> ApiConfig {
    ApiConfig::from_lookup(move |n| match n {
        "DATABASE_URL" => Some("postgres://unused".to_owned()),
        "TRUST_FORWARDED_FOR" => Some("true".to_owned()),
        _ => extra
            .iter()
            .find(|(k, _)| *k == n)
            .map(|(_, v)| (*v).to_owned()),
    })
    .unwrap()
}

fn items_path(w: &World) -> String {
    format!("/v1/workspaces/{}/demo-items", w.ws)
}

// --- rule 1: paths ---------------------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn unversioned_probes_and_v1_paths(pool: PgPool) {
    let w = world(&pool).await;
    let r = send(
        &w.app,
        req("GET", "/healthz", None).body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(r.body["status"], "ok");
    let r = send(
        &w.app,
        req("GET", "/readyz", None).body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(r.body["database"], "ok");
    assert_eq!(r.body["redis"], "disabled");
    let r = send(
        &w.app,
        req("GET", "/v1/version", None).body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(r.body["api"], env!("CARGO_PKG_VERSION"));
    // An unversioned business path does not exist.
    let r = send(
        &w.app,
        req("GET", "/version", None).body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(r.body["code"], "not_found");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn merge_patch_content_type_is_required(pool: PgPool) {
    let w = world(&pool).await;
    let id = Uuid::from_u128(1);
    let path = format!("{}/{id}", items_path(&w));
    let r = send(
        &w.app,
        req("PATCH", &path, Some(w.user))
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::IF_MATCH, "\"1\"")
            .body(Body::from("{\"name\":\"x\"}"))
            .unwrap(),
    )
    .await;
    assert_eq!(r.status, 415);
    assert_eq!(r.body["code"], "unsupported_media_type");
}

// --- rule 3: errors --------------------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn error_envelope_is_rfc9457_with_request_id(pool: PgPool) {
    let w = world(&pool).await;
    let r = send(
        &w.app,
        req("GET", "/v1/nothing", None).body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(r.status, 404);
    assert_eq!(r.header("content-type"), "application/problem+json");
    assert_eq!(r.body["type"], "/errors/not_found");
    assert_eq!(r.body["title"], "Not found");
    assert_eq!(r.body["status"], 404);
    assert_eq!(r.body["code"], "not_found");
    assert_eq!(r.body["request_id"], r.header("x-request-id"));
    // A wrong method is also the problem envelope, not axum's empty 405.
    let r = send(
        &w.app,
        req("DELETE", "/v1/version", None)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(r.body["code"], "not_found");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn malformed_json_and_wrong_media_type(pool: PgPool) {
    let w = world(&pool).await;
    let path = items_path(&w);
    let post = |ct: &str, body: &str| {
        req("POST", &path, Some(w.user))
            .header(header::CONTENT_TYPE, ct.to_owned())
            .header("idempotency-key", Uuid::new_v4().to_string())
            .body(Body::from(body.to_owned()))
            .unwrap()
    };
    let r = send(&w.app, post("application/json", "{not json")).await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (400, Some("malformed_request"))
    );
    let r = send(&w.app, post("application/json", "{\"name\": 5}")).await;
    assert_eq!(r.body["code"], "malformed_request");
    let r = send(&w.app, post("text/plain", "{\"name\":\"x\"}")).await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (415, Some("unsupported_media_type"))
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn validation_conflict_and_unavailable_shapes(pool: PgPool) {
    let w = world(&pool).await;
    w.auth.add(w.user, w.ws, &[]);
    let r = send(
        &w.app,
        json_post(
            &items_path(&w),
            w.user,
            Some(&Uuid::new_v4().to_string()),
            &json!({"name": "invalid"}),
        ),
    )
    .await;
    assert_eq!(r.status, 422);
    assert_eq!(r.body["code"], "validation_failed");
    assert_eq!(r.body["errors"][0]["field"], "/name");
    assert_eq!(r.body["errors"][0]["code"], "invalid_value");

    let base = format!("/v1/workspaces/{}", w.ws);
    let r = send(
        &w.app,
        req("GET", &format!("{base}/conflict"), Some(w.user))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (409, Some("conflict"))
    );
    let r = send(
        &w.app,
        req("GET", &format!("{base}/unavailable"), Some(w.user))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (503, Some("unavailable"))
    );
    assert_eq!(r.header("retry-after"), "7");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn internal_errors_do_not_leak_details(pool: PgPool) {
    let w = world(&pool).await;
    let r = send(
        &w.app,
        req(
            "GET",
            &format!("/v1/workspaces/{}/broken", w.ws),
            Some(w.user),
        )
        .body(Body::empty())
        .unwrap(),
    )
    .await;
    assert_eq!(r.status, 500);
    assert_eq!(r.body["code"], "internal");
    let text = r.body.to_string();
    assert!(
        !text.contains("secret") && !text.contains("SELECT"),
        "{text}"
    );
    assert!(r.body.get("stack").is_none());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn other_workspace_is_404_own_missing_permission_is_403(pool: PgPool) {
    let w = world(&pool).await;
    // Not a member of this workspace: 404, never 403 (rule 3.3).
    let stranger = Uuid::now_v7();
    insert_user(&pool, stranger).await;
    let r = send(
        &w.app,
        req("GET", &items_path(&w), Some(stranger))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (404, Some("not_found"))
    );
    // Member without the permission: 403.
    let r = send(
        &w.app,
        req(
            "GET",
            &format!("/v1/workspaces/{}/needs-permission", w.ws),
            Some(w.user),
        )
        .body(Body::empty())
        .unwrap(),
    )
    .await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (403, Some("forbidden"))
    );
    // No session: 401.
    let r = send(
        &w.app,
        req("GET", &items_path(&w), None)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (401, Some("unauthenticated"))
    );
    // A workspace id that is no UUID is just not found.
    let r = send(
        &w.app,
        req("GET", "/v1/workspaces/abc/demo-items", Some(w.user))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(r.body["code"], "not_found");
}

// --- ADR-0007 rule 2.3: request context ------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn request_runs_in_a_transaction_with_full_context(pool: PgPool) {
    let w = world(&pool).await;
    let r = send(
        &w.app,
        req(
            "GET",
            &format!("/v1/workspaces/{}/context", w.ws),
            Some(w.user),
        )
        .body(Body::empty())
        .unwrap(),
    )
    .await;
    assert_eq!(r.body["workspace_id"], w.ws.to_string());
    assert_eq!(r.body["user_id"], w.user.to_string());
    assert_eq!(r.body["actor_kind"], "user");
    assert_eq!(r.body["read_only"], false);
    // Outside a workspace only the user is set.
    let r = send(
        &w.app,
        req("GET", "/v1/me/context", Some(w.user))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert!(r.body["workspace_id"].is_null());
    assert_eq!(r.body["user_id"], w.user.to_string());
    // A support actor gets a read-only transaction.
    *w.auth.actor.lock().unwrap() = Some(db::ActorKind::Support);
    let r = send(
        &w.app,
        req(
            "GET",
            &format!("/v1/workspaces/{}/context", w.ws),
            Some(w.user),
        )
        .body(Body::empty())
        .unwrap(),
    )
    .await;
    assert_eq!(r.body["actor_kind"], "support");
    assert_eq!(r.body["read_only"], true);
}

// --- rule 4: pagination ----------------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn cursor_pages_cover_all_rows_once(pool: PgPool) {
    let w = world(&pool).await;
    for sort in ["name", "-name"] {
        let mut seen = Vec::new();
        let mut cursor: Option<String> = None;
        let mut pages = 0;
        loop {
            let mut path = format!("{}?limit=3&sort={sort}", items_path(&w));
            if let Some(c) = &cursor {
                path = format!("{path}&cursor={c}");
            }
            let r = send(
                &w.app,
                req("GET", &path, Some(w.user)).body(Body::empty()).unwrap(),
            )
            .await;
            assert_eq!(r.status, 200, "{}", r.body);
            for item in r.body["items"].as_array().unwrap() {
                seen.push(item["name"].as_str().unwrap().to_owned());
            }
            pages += 1;
            match r.body["next_cursor"].as_str() {
                Some(c) => cursor = Some(c.to_owned()),
                None => break,
            }
        }
        assert_eq!(pages, 3);
        let mut expected: Vec<String> = (1..=7).map(|i| format!("item-{i:02}")).collect();
        if sort == "-name" {
            expected.reverse();
        }
        assert_eq!(seen, expected, "{sort}");
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn bad_cursor_limit_and_sort_are_rejected(pool: PgPool) {
    let w = world(&pool).await;
    let get = |q: &str| {
        let path = format!("{}?{q}", items_path(&w));
        req("GET", &path, Some(w.user)).body(Body::empty()).unwrap()
    };
    let r = send(&w.app, get("cursor=%25%25bogus")).await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (400, Some("malformed_request"))
    );
    let r = send(&w.app, get("limit=201")).await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (422, Some("validation_failed"))
    );
    assert_eq!(r.body["errors"][0]["field"], "limit");
    let r = send(&w.app, get("sort=secret")).await;
    assert_eq!(r.body["errors"][0]["code"], "invalid_sort");
    // A cursor made for one sort is not valid for another.
    let page = send(&w.app, get("limit=2&sort=name")).await;
    let c = page.body["next_cursor"].as_str().unwrap().to_owned();
    let r = send(&w.app, get(&format!("limit=2&sort=-name&cursor={c}"))).await;
    assert_eq!(r.status, 400);
    let r = send(&w.app, get("limit=notanumber")).await;
    assert_eq!(r.body["code"], "malformed_request");
}

// --- rule 5: ETag and If-Match ---------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn etag_conditional_read_and_optimistic_write(pool: PgPool) {
    let w = world(&pool).await;
    let path = format!("{}/{}", items_path(&w), Uuid::from_u128(1));
    let r = send(
        &w.app,
        req("GET", &path, Some(w.user)).body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(r.header("etag"), "\"1\"");
    assert_eq!(r.header("cache-control"), "private, no-cache");
    // If-None-Match: 304.
    let r = send(
        &w.app,
        req("GET", &path, Some(w.user))
            .header(header::IF_NONE_MATCH, "\"1\"")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(r.status, 304);
    assert_eq!(r.header("etag"), "\"1\"");

    let patch = |if_match: Option<&str>, name: &str| {
        let mut b = req("PATCH", &path, Some(w.user))
            .header(header::CONTENT_TYPE, "application/merge-patch+json");
        if let Some(v) = if_match {
            b = b.header(header::IF_MATCH, v.to_owned());
        }
        b.body(Body::from(json!({"name": name}).to_string()))
            .unwrap()
    };
    // No If-Match on an optimistic resource: 428.
    let r = send(&w.app, patch(None, "a")).await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (428, Some("precondition_required"))
    );
    // Current version: accepted, new ETag.
    let r = send(&w.app, patch(Some("\"1\""), "a")).await;
    assert_eq!(r.status, 200);
    assert_eq!(r.header("etag"), "\"2\"");
    // Stale version: 412 with the current record next to the problem.
    let r = send(&w.app, patch(Some("\"1\""), "b")).await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (412, Some("version_mismatch"))
    );
    assert_eq!(r.body["current"]["version"], 2);
    assert_eq!(r.body["current"]["name"], "a");
    // The old ETag no longer gives 304.
    let r = send(
        &w.app,
        req("GET", &path, Some(w.user))
            .header(header::IF_NONE_MATCH, "\"1\"")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(r.status, 200);
}

// --- rule 6: idempotency ---------------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn idempotency_key_is_required(pool: PgPool) {
    let w = world(&pool).await;
    let r = send(
        &w.app,
        json_post(&items_path(&w), w.user, None, &json!({"name": "x"})),
    )
    .await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (428, Some("precondition_required"))
    );
    assert_eq!(w.demo.creates.load(Ordering::SeqCst), 0);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn same_key_and_body_replays_the_stored_response(pool: PgPool) {
    let w = world(&pool).await;
    let key = Uuid::new_v4().to_string();
    let body = json!({"name": "ale"});
    let first = send(
        &w.app,
        json_post(&items_path(&w), w.user, Some(&key), &body),
    )
    .await;
    assert_eq!(first.status, 201);
    assert!(first.header("idempotent-replayed").is_empty());
    let second = send(
        &w.app,
        json_post(&items_path(&w), w.user, Some(&key), &body),
    )
    .await;
    assert_eq!(second.status, 201);
    assert_eq!(second.header("idempotent-replayed"), "true");
    assert_eq!(second.body, first.body);
    assert_eq!(second.header("location"), first.header("location"));
    assert_eq!(second.header("etag"), first.header("etag"));
    assert_eq!(w.demo.creates.load(Ordering::SeqCst), 1, "handler ran once");
    let stored: i64 = sqlx::query_scalar("SELECT count(*) FROM idempotency_keys WHERE key = $1")
        .bind(&key)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, 1);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn same_key_with_another_body_is_rejected(pool: PgPool) {
    let w = world(&pool).await;
    let key = Uuid::new_v4().to_string();
    let first = send(
        &w.app,
        json_post(&items_path(&w), w.user, Some(&key), &json!({"name": "a"})),
    )
    .await;
    assert_eq!(first.status, 201);
    let other = send(
        &w.app,
        json_post(&items_path(&w), w.user, Some(&key), &json!({"name": "b"})),
    )
    .await;
    assert_eq!(
        (other.status.as_u16(), other.body["code"].as_str()),
        (422, Some("idempotency_mismatch"))
    );
    assert_eq!(w.demo.creates.load(Ordering::SeqCst), 1);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn request_in_progress_gives_409_and_failures_release_the_key(pool: PgPool) {
    let w = Arc::new(world(&pool).await);
    let key = Uuid::new_v4().to_string();
    let slow = json!({"name": "slow"});
    let first = {
        let (w, key, slow) = (w.clone(), key.clone(), slow.clone());
        tokio::spawn(async move {
            send(
                &w.app,
                json_post(&items_path(&w), w.user, Some(&key), &slow),
            )
            .await
        })
    };
    w.demo.entered.notified().await;
    let second = send(
        &w.app,
        json_post(&items_path(&w), w.user, Some(&key), &slow),
    )
    .await;
    assert_eq!(
        (second.status.as_u16(), second.body["code"].as_str()),
        (409, Some("idempotency_in_progress"))
    );
    assert_eq!(second.header("retry-after"), "1");
    w.demo.gate.notify_one();
    let first = first.await.unwrap();
    assert_eq!(first.status, 201);
    let replay = send(
        &w.app,
        json_post(&items_path(&w), w.user, Some(&key), &slow),
    )
    .await;
    assert_eq!(replay.header("idempotent-replayed"), "true");

    // A failed request (422) releases its key, so the same key can be used again.
    let key2 = Uuid::new_v4().to_string();
    let bad = json!({"name": "invalid"});
    for _ in 0..2 {
        let r = send(
            &w.app,
            json_post(&items_path(&w), w.user, Some(&key2), &bad),
        )
        .await;
        assert_eq!(r.status, 422);
        assert!(r.header("idempotent-replayed").is_empty());
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn keys_are_scoped_to_the_workspace(pool: PgPool) {
    let w = world(&pool).await;
    let ws2 = Uuid::now_v7();
    insert_workspace(&pool, ws2).await;
    w.auth.add(w.user, ws2, &[]);
    let key = Uuid::new_v4().to_string();
    let body = json!({"name": "a"});
    let a = send(
        &w.app,
        json_post(&items_path(&w), w.user, Some(&key), &body),
    )
    .await;
    let b = send(
        &w.app,
        json_post(
            &format!("/v1/workspaces/{ws2}/demo-items"),
            w.user,
            Some(&key),
            &body,
        ),
    )
    .await;
    assert_eq!((a.status.as_u16(), b.status.as_u16()), (201, 201));
    assert!(b.header("idempotent-replayed").is_empty());
    assert_eq!(w.demo.creates.load(Ordering::SeqCst), 2);
}

// --- rule 8: app version ---------------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn old_clients_get_426_but_can_read_requirements(pool: PgPool) {
    let cfg = cfg_with(&[("MIN_APP_VERSION_IOS", "1.4.0")]);
    let w = world_with(&pool, Redis::disabled(), cfg).await;
    let get = |path: &str, version: Option<&str>| {
        let mut b = req("GET", path, None);
        if let Some(v) = version {
            b = b.header("x-app-version", v.to_owned());
        }
        b.body(Body::empty()).unwrap()
    };
    let r = send(&w.app, get("/v1/version", Some("ios/1.3.9+80"))).await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (426, Some("upgrade_required"))
    );
    assert_eq!(
        send(&w.app, get("/v1/version", Some("ios/1.4.0+1")))
            .await
            .status,
        200
    );
    assert_eq!(send(&w.app, get("/v1/version", None)).await.status, 200);
    let r = send(&w.app, get("/v1/meta/client-requirements", Some("ios/0.1"))).await;
    assert_eq!(r.status, 200);
    assert_eq!(r.body["min_ios_version"], "1.4.0");
    assert!(r.body["min_web_version"].is_null());
    assert_eq!(
        send(&w.app, get("/healthz", Some("ios/0.1"))).await.status,
        200
    );
}

// --- rule 7: field definitions ---------------------------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn meta_fields_serves_the_catalog_with_etag(pool: PgPool) {
    let w = world(&pool).await;
    let r = send(
        &w.app,
        req("GET", "/v1/meta/fields", None)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(r.status, 200);
    assert!(r.body["fields"].as_object().unwrap().len() > 30);
    assert!(r.body["units"].is_object());
    let etag = r.header("etag").to_owned();
    assert!(etag.starts_with('"'));
    assert!(r.header("cache-control").contains("max-age"));
    let r = send(
        &w.app,
        req("GET", "/v1/meta/fields", None)
            .header(header::IF_NONE_MATCH, etag)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(r.status, 304);
}

// --- rule 11: limits, rate limit, request id, trace ------------------------------------

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn body_over_1_mib_is_413(pool: PgPool) {
    let w = world(&pool).await;
    let big = json!({"name": "x".repeat(1024 * 1024 + 1)});
    let r = send(
        &w.app,
        json_post(
            &items_path(&w),
            w.user,
            Some(&Uuid::new_v4().to_string()),
            &big,
        ),
    )
    .await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (413, Some("payload_too_large"))
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn bulk_requests_are_limited_to_500_items(pool: PgPool) {
    let w = world(&pool).await;
    let path = format!("/v1/workspaces/{}/demo-bulk", w.ws);
    let post = |n: usize| {
        json_post(
            &path,
            w.user,
            Some(&Uuid::new_v4().to_string()),
            &json!({"items": vec![0; n]}),
        )
    };
    assert_eq!(send(&w.app, post(500)).await.status, 204);
    let r = send(&w.app, post(501)).await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (422, Some("validation_failed"))
    );
    assert_eq!(r.body["errors"][0]["code"], "too_many_items");
    assert_eq!(r.body["errors"][0]["params"]["max"], 500);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn rate_limit_answers_429_with_headers(pool: PgPool) {
    let cfg = cfg_with(&[("RATE_LIMIT_IP_PER_MINUTE", "3")]);
    let redis = Redis::connect_lazy(&redis_url()).unwrap();
    let w = world_with(&pool, redis, cfg).await;
    let ip = Uuid::new_v4().to_string();
    let get = || {
        let mut r = req("GET", "/v1/version", None).body(Body::empty()).unwrap();
        r.headers_mut()
            .insert("x-forwarded-for", ip.parse().unwrap());
        r
    };
    let first = send(&w.app, get()).await;
    assert_eq!(first.status, 200);
    assert_eq!(first.header("ratelimit-policy"), "\"ip\";q=3;w=60");
    assert!(
        first.header("ratelimit").starts_with("\"ip\";r=2;"),
        "{}",
        first.header("ratelimit")
    );
    assert_eq!(send(&w.app, get()).await.status, 200);
    assert_eq!(send(&w.app, get()).await.status, 200);
    let blocked = send(&w.app, get()).await;
    assert_eq!(
        (blocked.status.as_u16(), blocked.body["code"].as_str()),
        (429, Some("rate_limited"))
    );
    assert!(blocked.header("retry-after").parse::<u64>().unwrap() >= 1);
    assert!(blocked.header("ratelimit").contains("r=0"));
    // Another client is unaffected.
    assert_eq!(
        send(
            &w.app,
            req("GET", "/v1/version", None).body(Body::empty()).unwrap()
        )
        .await
        .status,
        200
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn user_and_workspace_limits_apply_after_sign_in(pool: PgPool) {
    let cfg = cfg_with(&[("RATE_LIMIT_USER_PER_MINUTE", "2")]);
    let redis = Redis::connect_lazy(&redis_url()).unwrap();
    let w = world_with(&pool, redis, cfg).await;
    let get = || {
        req("GET", &items_path(&w), Some(w.user))
            .body(Body::empty())
            .unwrap()
    };
    assert_eq!(send(&w.app, get()).await.status, 200);
    assert_eq!(send(&w.app, get()).await.status, 200);
    let r = send(&w.app, get()).await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (429, Some("rate_limited"))
    );
    assert!(r.header("ratelimit-policy").starts_with("\"user\""));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn api_keeps_working_without_redis(pool: PgPool) {
    // 1. Redis configured but nothing listens there.
    let dead = Redis::connect_lazy("redis://127.0.0.1:1").unwrap();
    let w = world_with(&pool, dead, cfg_with(&[("RATE_LIMIT_IP_PER_MINUTE", "1")])).await;
    for _ in 0..3 {
        let r = send(
            &w.app,
            req("GET", "/v1/version", None).body(Body::empty()).unwrap(),
        )
        .await;
        assert_eq!(r.status, 200, "fail open: the limit is not applied");
        assert!(r.header("ratelimit").is_empty());
    }
    let r = send(
        &w.app,
        req("GET", "/readyz", None).body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(r.status, 200);
    assert_eq!(r.body["redis"], "unavailable");
    let r = send(
        &w.app,
        json_post(
            &items_path(&w),
            w.user,
            Some(&Uuid::new_v4().to_string()),
            &json!({"name": "x"}),
        ),
    )
    .await;
    assert_eq!(r.status, 201, "writes with idempotency work too");

    // 2. Redis dies while the API is running.
    let mut proxy = RedisProxy::start().await;
    let live = Redis::connect_lazy(&proxy.url).unwrap();
    let w = world_with(
        &pool,
        live,
        cfg_with(&[("RATE_LIMIT_IP_PER_MINUTE", "100")]),
    )
    .await;
    let ok = send(
        &w.app,
        req("GET", "/v1/version", None).body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(ok.status, 200);
    assert!(
        !ok.header("ratelimit").is_empty(),
        "limit active while Redis lives"
    );
    proxy.cut().await;
    for _ in 0..3 {
        let r = send(
            &w.app,
            req("GET", &items_path(&w), Some(w.user))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(r.status, 200);
        assert!(r.header("ratelimit").is_empty());
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn request_id_is_taken_or_made_and_always_returned(pool: PgPool) {
    let w = world(&pool).await;
    let r = send(
        &w.app,
        req("GET", "/v1/version", None)
            .header("x-request-id", "client-id-42")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(r.header("x-request-id"), "client-id-42");
    let r = send(
        &w.app,
        req("GET", "/v1/version", None).body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(r.header("x-request-id").len(), 32);
    // An unusable client id is replaced, also on errors.
    let r = send(
        &w.app,
        req("GET", "/v1/missing", None)
            .header("x-request-id", "bad id!")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_ne!(r.header("x-request-id"), "bad id!");
    assert_eq!(r.body["request_id"], r.header("x-request-id"));
}
