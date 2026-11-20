//! Endpoint tests of T2.20: equipment, profiles, vessels, workspace and user settings, the field
//! level last-writer-wins rule, the change history and restore, the equipment wizard and the
//! calibration proposal. Needs PostgreSQL (`#[sqlx::test]`).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

mod support;

use std::sync::Arc;

use api::{
    ErrorCode,
    engine::{EngineClient, EngineConfig},
    redis_client::Redis,
};
use axum::{
    Router,
    body::Body,
    http::{StatusCode, header},
};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{FakeAuth, Reply, config, insert_user, insert_workspace, req, send, state_with};
use tokio::{net::TcpListener, sync::oneshot};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use uuid::Uuid;

struct World {
    app: Router,
    auth: Arc<FakeAuth>,
    admin: Uuid,
    ws: Uuid,
}

const ALL: &[&str] = &["recipe.view", "settings.equipment", "workspace.manage"];

async fn world_with_engine(pool: &PgPool, engine: Option<EngineClient>) -> World {
    let admin = Uuid::now_v7();
    let ws = Uuid::now_v7();
    insert_user(pool, admin).await;
    insert_workspace(pool, ws).await;
    let auth = Arc::new(FakeAuth::default());
    auth.add(admin, ws, ALL);
    let mut state = state_with(
        pool,
        Redis::disabled(),
        auth.clone(),
        config("postgres://unused"),
    )
    .await;
    if let Some(e) = engine {
        state = state.with_engine(e);
    }
    World {
        app: api::router(state),
        auth,
        admin,
        ws,
    }
}

async fn world(pool: &PgPool) -> World {
    world_with_engine(pool, None).await
}

async fn call(
    w: &World,
    method: &str,
    path: &str,
    user: Uuid,
    body: Option<Value>,
    extra: &[(&str, &str)],
) -> Reply {
    let mut b = req(
        method,
        &format!("/v1/workspaces/{}{path}", w.ws),
        Some(user),
    );
    for (k, v) in extra {
        b = b.header(*k, *v);
    }
    let request = match body {
        Some(v) => {
            let ct = if method == "PATCH" {
                "application/merge-patch+json"
            } else {
                "application/json"
            };
            b.header(header::CONTENT_TYPE, ct)
                .body(Body::from(v.to_string()))
                .unwrap()
        }
        None => b.body(Body::empty()).unwrap(),
    };
    send(&w.app, request).await
}

async fn post(w: &World, path: &str, body: Value) -> Reply {
    let key = Uuid::now_v7().to_string();
    call(
        w,
        "POST",
        path,
        w.admin,
        Some(body),
        &[("idempotency-key", &key)],
    )
    .await
}

async fn patch(w: &World, path: &str, body: Value) -> Reply {
    call(w, "PATCH", path, w.admin, Some(body), &[]).await
}

async fn get(w: &World, path: &str) -> Reply {
    call(w, "GET", path, w.admin, None, &[]).await
}

fn equipment() -> Value {
    json!({"name": "Kettle", "batch_volume_l": 20.0, "efficiency_pct": 72.0, "trub_loss_l": 1.0})
}

async fn make_equipment(w: &World) -> String {
    let r = post(w, "/equipment", equipment()).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    r.body["id"].as_str().unwrap().to_owned()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn equipment_create_read_and_validation(pool: PgPool) {
    let w = world(&pool).await;
    let r = post(&w, "/equipment", equipment()).await;
    assert_eq!(r.status, StatusCode::CREATED);
    assert_eq!(r.header("etag"), "\"1\"");
    // The first equipment of a workspace becomes the default; database defaults fill the rest.
    assert_eq!(r.body["is_default"], true);
    assert_eq!(r.body["boil_time_min"], 60.0);
    let id = r.body["id"].as_str().unwrap();

    let g = get(&w, &format!("/equipment/{id}")).await;
    assert_eq!(g.status, StatusCode::OK);
    let cached = call(
        &w,
        "GET",
        &format!("/equipment/{id}"),
        w.admin,
        None,
        &[("if-none-match", "\"1\"")],
    )
    .await;
    assert_eq!(cached.status, StatusCode::NOT_MODIFIED);

    let list = get(&w, "/equipment").await;
    assert_eq!(list.body["items"].as_array().unwrap().len(), 1);

    let bad = post(
        &w,
        "/equipment",
        json!({"name": "", "batch_volume_l": -1.0, "bogus": 1}),
    )
    .await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(bad.body["code"], ErrorCode::ValidationFailed.as_str());
    let codes: Vec<String> = bad.body["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            format!(
                "{}:{}",
                e["field"].as_str().unwrap(),
                e["code"].as_str().unwrap()
            )
        })
        .collect();
    for want in [
        "/name:too_short",
        "/batch_volume_l:out_of_range",
        "/bogus:unknown_field",
        "/efficiency_pct:required",
    ] {
        assert!(codes.contains(&want.to_owned()), "{want} in {codes:?}");
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn last_writer_wins_for_each_field(pool: PgPool) {
    let w = world(&pool).await;
    let id = make_equipment(&w).await;
    let path = format!("/equipment/{id}");
    // Two writers change different fields from the same old version: both changes stay.
    let a = call(
        &w,
        "PATCH",
        &path,
        w.admin,
        Some(json!({"efficiency_pct": 80.0})),
        &[("if-match", "\"1\"")],
    )
    .await;
    assert_eq!(a.status, StatusCode::OK, "{}", a.body);
    assert_eq!(a.header("etag"), "\"2\"");
    let b = patch(&w, &path, json!({"name": "Big kettle"})).await;
    assert_eq!(b.status, StatusCode::OK);
    assert_eq!(b.body["efficiency_pct"], 80.0);
    assert_eq!(b.body["name"], "Big kettle");
    assert_eq!(b.body["version"], 3);
    // Two writers change the same field: the last one wins.
    patch(&w, &path, json!({"efficiency_pct": 70.0})).await;
    let c = patch(&w, &path, json!({"efficiency_pct": 75.0})).await;
    assert_eq!(c.body["efficiency_pct"], 75.0);
    // A stale `If-Match` is refused with the current record.
    let stale = call(
        &w,
        "PATCH",
        &path,
        w.admin,
        Some(json!({"name": "X"})),
        &[("if-match", "\"1\"")],
    )
    .await;
    assert_eq!(stale.status, StatusCode::PRECONDITION_FAILED);
    assert_eq!(stale.body["current"]["name"], "Big kettle");
    // A value that does not change makes no new version.
    let same = patch(&w, &path, json!({"efficiency_pct": 75.0})).await;
    assert_eq!(same.body["version"], c.body["version"]);
    // Clearing an optional field with null; a required field refuses null.
    assert_eq!(
        patch(&w, &path, json!({"notes": null})).await.status,
        StatusCode::OK
    );
    assert_eq!(
        patch(&w, &path, json!({"name": null})).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        patch(&w, &path, json!({"efficiency_pct": 101.0}))
            .await
            .status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_value_comes_back_from_the_history(pool: PgPool) {
    let w = world(&pool).await;
    let id = make_equipment(&w).await;
    let path = format!("/equipment/{id}");
    patch(&w, &path, json!({"efficiency_pct": 80.0, "name": "Second"})).await; // version 2
    patch(&w, &path, json!({"efficiency_pct": 85.0})).await; // version 3

    let h = get(&w, &format!("{path}/history")).await;
    let items = h.body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    // Newest first: version 3 replaced 80, version 2 replaced 72 (and the name).
    assert_eq!(items[0]["record_version"], 3);
    assert_eq!(items[0]["previous_value"].as_object().unwrap().len(), 1);
    assert_eq!(
        items[0]["previous_value"]["efficiency_pct"].as_f64(),
        Some(80.0)
    );
    assert_eq!(items[1]["previous_value"]["efficiency_pct"], 72.0);
    assert_eq!(items[1]["previous_value"]["name"], "Kettle");

    // Restore the oldest row: efficiency is 72 and the name is "Kettle" again.
    let hid = items[1]["id"].as_str().unwrap();
    let r = call(
        &w,
        "POST",
        &format!("{path}/history/{hid}/restore"),
        w.admin,
        None,
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert_eq!(r.body["efficiency_pct"], 72.0);
    assert_eq!(r.body["name"], "Kettle");
    assert_eq!(r.body["version"], 4);
    // The restore is a change with its own row, so it can be undone too.
    let h2 = get(&w, &format!("{path}/history")).await;
    assert_eq!(h2.body["items"].as_array().unwrap().len(), 3);
    assert_eq!(
        h2.body["items"][0]["previous_value"]["efficiency_pct"],
        85.0
    );
    // A row of another record is not found.
    let other = make_equipment(&w).await;
    let wrong = call(
        &w,
        "POST",
        &format!("/equipment/{other}/history/{hid}/restore"),
        w.admin,
        None,
        &[],
    )
    .await;
    assert_eq!(wrong.status, StatusCode::NOT_FOUND);
    // Paging: one row per page.
    let page = get(&w, &format!("{path}/history?limit=1")).await;
    assert_eq!(page.body["items"].as_array().unwrap().len(), 1);
    let cursor = page.body["next_cursor"].as_str().unwrap();
    let next = get(&w, &format!("{path}/history?limit=1&cursor={cursor}")).await;
    assert_eq!(next.body["items"][0]["record_version"], 3);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn one_default_equipment_and_restorable_delete(pool: PgPool) {
    let w = world(&pool).await;
    let first = make_equipment(&w).await;
    let second = post(
        &w,
        "/equipment",
        json!({"name": "B", "batch_volume_l": 10.0, "efficiency_pct": 70.0, "is_default": true}),
    )
    .await;
    let second = second.body["id"].as_str().unwrap().to_owned();
    assert_eq!(
        get(&w, &format!("/equipment/{first}")).await.body["is_default"],
        false
    );
    assert_eq!(
        get(&w, &format!("/equipment/{second}")).await.body["is_default"],
        true
    );

    let d = call(
        &w,
        "DELETE",
        &format!("/equipment/{second}"),
        w.admin,
        None,
        &[],
    )
    .await;
    assert_eq!(d.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(&w, &format!("/equipment/{second}")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&w, "/equipment").await.body["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        get(&w, "/equipment?include_deleted=true").await.body["items"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let h = get(&w, &format!("/equipment/{second}/history")).await;
    let hid = h.body["items"][0]["id"].as_str().unwrap();
    let r = call(
        &w,
        "POST",
        &format!("/equipment/{second}/history/{hid}/restore"),
        w.admin,
        None,
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert_eq!(r.body["deleted_at"], Value::Null);
    assert_eq!(r.body["is_default"], true);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn profiles_with_steps_and_other_resources(pool: PgPool) {
    let w = world(&pool).await;
    let m = post(
        &w,
        "/mash-profiles",
        json!({
        "name": "Two rests",
        "steps": [
            {"kind": "temperature", "temp_c": 52.0, "time_min": 10.0},
            {"kind": "infusion", "temp_c": 66.0, "time_min": 60.0, "infusion_l": 8.0}
        ]}),
    )
    .await;
    assert_eq!(m.status, StatusCode::CREATED, "{}", m.body);
    assert_eq!(m.body["steps"].as_array().unwrap().len(), 2);
    assert_eq!(m.body["steps"][0]["ramp_min"], 0.0);
    let id = m.body["id"].as_str().unwrap();
    // The steps are replaced as a whole and the old list is in the history.
    let p = patch(
        &w,
        &format!("/mash-profiles/{id}"),
        json!({"steps": [{"kind": "temperature", "temp_c": 67.0, "time_min": 60.0}]}),
    )
    .await;
    assert_eq!(p.status, StatusCode::OK, "{}", p.body);
    assert_eq!(p.body["steps"].as_array().unwrap().len(), 1);
    let h = get(&w, &format!("/mash-profiles/{id}/history")).await;
    let hid = h.body["items"][0]["id"].as_str().unwrap();
    let r = call(
        &w,
        "POST",
        &format!("/mash-profiles/{id}/history/{hid}/restore"),
        w.admin,
        None,
        &[],
    )
    .await;
    assert_eq!(r.body["steps"].as_array().unwrap().len(), 2);
    assert_eq!(r.body["steps"][1]["infusion_l"], 8.0);
    let bad = post(
        &w,
        "/mash-profiles",
        json!({"name": "x", "steps": [{"kind": "boil", "temp_c": 1.0}]}),
    )
    .await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY);

    let f = post(
        &w,
        "/fermentation-profiles",
        json!({"name": "Ale", "steps": [{"temp_c": 18.0, "duration_min": 10080.0}]}),
    )
    .await;
    assert_eq!(f.status, StatusCode::CREATED, "{}", f.body);
    let c = post(&w, "/carbonation-profiles", json!({"name": "Bottle", "method": "priming", "target_co2_vols": 2.4, "priming_agent": "corn_sugar"})).await;
    assert_eq!(c.status, StatusCode::CREATED, "{}", c.body);
    let wp = post(
        &w,
        "/water-profiles",
        json!({"name": "Tap", "kind": "source", "calcium_mg_l": 40.0}),
    )
    .await;
    assert_eq!(wp.status, StatusCode::CREATED, "{}", wp.body);
    let missing_doc = post(
        &w,
        "/water-profiles",
        json!({"name": "T", "kind": "source", "source_document_id": Uuid::now_v7()}),
    )
    .await;
    assert_eq!(missing_doc.status, StatusCode::UNPROCESSABLE_ENTITY);
    let v = post(
        &w,
        "/vessels",
        json!({"code": "FV-1", "kind": "fermenter", "capacity_l": 30.0}),
    )
    .await;
    assert_eq!(v.status, StatusCode::CREATED, "{}", v.body);
    assert_eq!(v.body["cleaning_status"], "dirty");
    let dup = post(
        &w,
        "/vessels",
        json!({"code": "FV-1", "kind": "kettle", "capacity_l": 40.0}),
    )
    .await;
    assert_eq!(dup.status, StatusCode::CONFLICT);
    assert_eq!(dup.body["errors"][0]["field"], "/code");
    // Sorting and cursor paging.
    post(
        &w,
        "/vessels",
        json!({"code": "FV-2", "kind": "fermenter", "capacity_l": 30.0}),
    )
    .await;
    post(
        &w,
        "/vessels",
        json!({"code": "FV-3", "kind": "fermenter", "capacity_l": 30.0}),
    )
    .await;
    let page = get(&w, "/vessels?limit=2&sort=-code").await;
    let codes: Vec<&str> = page.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["code"].as_str().unwrap())
        .collect();
    assert_eq!(codes, ["FV-3", "FV-2"]);
    let cursor = page.body["next_cursor"].as_str().unwrap();
    let rest = get(&w, &format!("/vessels?limit=2&sort=-code&cursor={cursor}")).await;
    assert_eq!(rest.body["items"][0]["code"], "FV-1");
    assert_eq!(
        get(&w, "/vessels?sort=password").await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn permissions_and_workspace_isolation(pool: PgPool) {
    let w = world(&pool).await;
    let id = make_equipment(&w).await;
    let viewer = Uuid::now_v7();
    insert_user(&pool, viewer).await;
    w.auth.add(viewer, w.ws, &["recipe.view"]);
    let nobody = Uuid::now_v7();
    insert_user(&pool, nobody).await;
    w.auth.add(nobody, w.ws, &[]);

    assert_eq!(
        call(&w, "GET", &format!("/equipment/{id}"), viewer, None, &[])
            .await
            .status,
        StatusCode::OK
    );
    let denied = call(
        &w,
        "PATCH",
        &format!("/equipment/{id}"),
        viewer,
        Some(json!({"name": "Z"})),
        &[],
    )
    .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
    assert_eq!(
        call(&w, "DELETE", &format!("/equipment/{id}"), viewer, None, &[])
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(&w, "GET", "/equipment", nobody, None, &[])
            .await
            .status,
        StatusCode::FORBIDDEN
    );

    // A member of another workspace sees nothing here (404), and its records stay apart.
    let other_ws = Uuid::now_v7();
    insert_workspace(&pool, other_ws).await;
    let stranger = Uuid::now_v7();
    insert_user(&pool, stranger).await;
    w.auth.add(stranger, other_ws, ALL);
    let r = call(&w, "GET", &format!("/equipment/{id}"), stranger, None, &[]).await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    let own = send(
        &w.app,
        req(
            "GET",
            &format!("/v1/workspaces/{other_ws}/equipment/{id}"),
            Some(stranger),
        )
        .body(Body::empty())
        .unwrap(),
    )
    .await;
    assert_eq!(own.status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn workspace_and_user_settings_with_history(pool: PgPool) {
    let w = world(&pool).await;
    let g = get(&w, "/settings").await;
    assert_eq!(g.status, StatusCode::OK, "{}", g.body);
    assert_eq!(g.body["settings"], json!({}));
    let p = patch(
        &w,
        "/settings",
        json!({"unit_system": "us", "ibu_equation": "tinseth", "steep_efficiency_pct": 20.0}),
    )
    .await;
    assert_eq!(p.status, StatusCode::OK, "{}", p.body);
    assert_eq!(p.body["settings"]["unit_system"], "us");
    let p2 = patch(&w, "/settings", json!({"unit_system": "metric"})).await;
    assert_eq!(p2.body["settings"]["ibu_equation"], "tinseth");
    assert_eq!(
        patch(&w, "/settings", json!({"unit_system": "furlongs"}))
            .await
            .status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        patch(&w, "/settings", json!({"schema_version": 9}))
            .await
            .status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    // Restore the first change: the unit system is "us" again; a key that did not exist is removed.
    let h = get(&w, "/settings/history").await;
    let items = h.body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    let hid = items[0]["id"].as_str().unwrap();
    let r = call(
        &w,
        "POST",
        &format!("/settings/history/{hid}/restore"),
        w.admin,
        None,
        &[],
    )
    .await;
    assert_eq!(r.body["settings"]["unit_system"], "us");
    let first = items[1]["id"].as_str().unwrap();
    let r2 = call(
        &w,
        "POST",
        &format!("/settings/history/{first}/restore"),
        w.admin,
        None,
        &[],
    )
    .await;
    assert!(r2.body["settings"].get("ibu_equation").is_none());
    assert!(r2.body["settings"].get("unit_system").is_none());
    // Only `workspace.manage` writes workspace settings; every member has own settings.
    let member = Uuid::now_v7();
    insert_user(&pool, member).await;
    w.auth.add(member, w.ws, &["recipe.view"]);
    assert_eq!(
        call(
            &w,
            "PATCH",
            "/settings",
            member,
            Some(json!({"unit_system": "us"})),
            &[]
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        call(&w, "GET", "/settings", member, None, &[]).await.status,
        StatusCode::OK
    );

    let mine = call(&w, "GET", "/me/settings", member, None, &[]).await;
    assert_eq!(mine.body["version"], 0);
    let set = call(
        &w,
        "PATCH",
        "/me/settings",
        member,
        Some(json!({"temperature_unit": "fahrenheit"})),
        &[],
    )
    .await;
    assert_eq!(set.status, StatusCode::OK, "{}", set.body);
    assert_eq!(set.body["version"], 1);
    // The other member does not see it.
    let admins = get(&w, "/me/settings").await;
    assert_eq!(admins.body["settings"], json!({}));
    let hist = call(&w, "GET", "/me/settings/history", member, None, &[]).await;
    assert_eq!(hist.body["items"].as_array().unwrap().len(), 1);
    let hid = hist.body["items"][0]["id"].as_str().unwrap();
    let back = call(
        &w,
        "POST",
        &format!("/me/settings/history/{hid}/restore"),
        member,
        None,
        &[],
    )
    .await;
    assert_eq!(back.body["settings"], json!({}));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn equipment_wizard(pool: PgPool) {
    let w = world(&pool).await;
    let r = post(
        &w,
        "/equipment/wizard",
        json!({"method": "biab", "batch_volume_l": 20.0, "kettle_volume_l": 30.0}),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    assert_eq!(r.body["batch_volume_l"], 20.0);
    assert_eq!(r.body["boil_volume_l"], 24.0);
    assert_eq!(r.body["mash_tun_volume_l"], 30.0);
    assert_eq!(r.body["is_default"], true);
    let small = post(
        &w,
        "/equipment/wizard",
        json!({"method": "all_grain", "batch_volume_l": 20.0, "kettle_volume_l": 22.0}),
    )
    .await;
    assert_eq!(small.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(small.body["errors"][0]["field"], "/kettle_volume_l");
    let missing = post(&w, "/equipment/wizard", json!({"method": "all_grain"})).await;
    assert_eq!(missing.status, StatusCode::UNPROCESSABLE_ENTITY);
}

async fn start_engine() -> (String, oneshot::Sender<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (stop, rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        Server::builder()
            .add_service(engine_service::service::server())
            .add_service(engine_service::brewery::server())
            .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                let _ = rx.await;
            })
            .await
    });
    (url, stop)
}

fn observation(volume: f64) -> Value {
    json!({"wort_volume_l": volume, "wort_extract_p": 12.0, "malt_kg": 5.0})
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn calibration_proposal_uses_the_engine(pool: PgPool) {
    let (url, stop) = start_engine().await;
    let mut cfg = EngineConfig::new(url);
    cfg.version_ttl = std::time::Duration::ZERO;
    let engine = EngineClient::connect_lazy(cfg, Redis::disabled()).unwrap();
    let w = world_with_engine(&pool, Some(engine)).await;
    let id = make_equipment(&w).await;
    let path = format!("/equipment/{id}/calibration-proposal");

    let few = call(
        &w,
        "POST",
        &path,
        w.admin,
        Some(json!({"observations": [observation(20.0)]})),
        &[],
    )
    .await;
    assert_eq!(few.status, StatusCode::OK, "{}", few.body);
    assert_eq!(few.body["proposal"], Value::Null);
    assert_eq!(few.body["reason"], "too_few_observations");
    assert!(
        few.body["observations"][0]["brewhouse_yield_pct"]
            .as_f64()
            .unwrap()
            > 0.0
    );
    assert_eq!(few.body["current_efficiency_pct"], 72.0);

    let three: Vec<Value> = [20.0, 20.5, 21.0].map(observation).into();
    let ok = call(
        &w,
        "POST",
        &path,
        w.admin,
        Some(json!({"observations": three})),
        &[],
    )
    .await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.body);
    let yields: Vec<f64> = ok.body["observations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["brewhouse_yield_pct"].as_f64().unwrap())
        .collect();
    let mean = yields.iter().sum::<f64>() / 3.0;
    let proposed = ok.body["proposal"]["efficiency_pct"].as_f64().unwrap();
    assert!(
        (proposed - mean).abs() <= 0.05 + 1e-9,
        "{proposed} vs {mean}"
    );
    // The proposal changes nothing.
    assert_eq!(
        get(&w, &format!("/equipment/{id}")).await.body["efficiency_pct"],
        72.0
    );

    let spread: Vec<Value> = [10.0, 20.0, 30.0].map(observation).into();
    let odd = call(
        &w,
        "POST",
        &path,
        w.admin,
        Some(json!({"observations": spread})),
        &[],
    )
    .await;
    assert_eq!(odd.body["reason"], "inconsistent_observations");

    let bad = call(
        &w,
        "POST",
        &path,
        w.admin,
        Some(json!({"observations": []})),
        &[],
    )
    .await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY);
    let unknown = call(
        &w,
        "POST",
        &format!("/equipment/{}/calibration-proposal", Uuid::now_v7()),
        w.admin,
        Some(json!({"observations": [observation(20.0)]})),
        &[],
    )
    .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    let _ = stop.send(());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn calibration_without_an_engine_answers_503(pool: PgPool) {
    let w = world(&pool).await;
    let id = make_equipment(&w).await;
    let r = call(
        &w,
        "POST",
        &format!("/equipment/{id}/calibration-proposal"),
        w.admin,
        Some(json!({"observations": [observation(20.0)]})),
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::SERVICE_UNAVAILABLE);
}
