//! Endpoint tests of K2 (containers, scan codes, fills, recall, labels). Needs PostgreSQL.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

mod domain;
mod support;

use axum::{body::Body, http::StatusCode};
use domain::{World, call_as, get, ipa, post, published_recipe, world};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

async fn container_world(pool: &PgPool) -> World {
    let w = world(pool).await;
    sqlx::query("UPDATE workspaces SET mode = 'business' WHERE id = $1")
        .bind(w.ws)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO entitlements (id, workspace_id, feature, source, valid_from, valid_until)
         VALUES ($1, $2, $3, 'promo', now() - interval '1 day', now() + interval '1 day')",
    )
    .bind(Uuid::now_v7())
    .bind(w.ws)
    .bind(billing::PAID_FEATURE)
    .execute(pool)
    .await
    .unwrap();
    let mut perms: Vec<&str> = domain::ALL.to_vec();
    perms.extend([
        "stock.move",
        "field.view",
        "container.status",
        "container.manage",
    ]);
    w.auth.add(w.admin, w.ws, &perms);
    w.auth.add(w.other, w.ws, &["field.view"]);
    w
}

async fn package_type(pool: &PgPool, ws: Uuid, name: &str) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO package_types (id, workspace_id, kind, name, volume_l)
         VALUES ($1, $2, 'keg', $3, 30)",
    )
    .bind(id)
    .bind(ws)
    .bind(name)
    .execute(pool)
    .await
    .unwrap();
    id
}

/// Creates a keg package type over HTTP.
async fn package_type_http(w: &World, name: &str) -> Uuid {
    let r = post(
        w,
        "/package-types",
        json!({"kind": "keg", "name": name, "volume_l": 30.0}),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    r.body["id"].as_str().unwrap().parse().unwrap()
}

/// Creates containers `KEG-0001..` and returns (ids, tokens).
async fn create(w: &World, pt: Uuid, count: u32) -> (Vec<String>, Vec<String>) {
    let r = post(
        w,
        "/containers",
        json!({"package_type_id": pt, "prefix": "KEG-", "count": count}),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let items = r.body["items"].as_array().unwrap();
    (
        items
            .iter()
            .map(|i| i["container"]["id"].as_str().unwrap().to_owned())
            .collect(),
        items
            .iter()
            .map(|i| i["scan_token"].as_str().unwrap().to_owned())
            .collect(),
    )
}

/// A lot (via the stock API) and a packaging run line (via SQL) for a batch.
async fn lot_and_line(w: &World, pt: Uuid, quantity: i32) -> (String, Uuid, Uuid) {
    let ing = post(
        w,
        "/ingredients",
        json!({"kind": "fermentable", "name": "Pale malt", "detail":
            {"grain_type": "grain", "yield_pct": 80.0, "color_srm": 2.0}}),
    )
    .await;
    let rec = post(
        w,
        "/stock/receipts",
        json!({"ingredient_id": ing.body["id"], "quantity": 5.0, "lot_code": "L-1"}),
    )
    .await;
    assert_eq!(rec.status, StatusCode::CREATED, "{}", rec.body);
    let lot = rec.body["lot"]["id"].as_str().unwrap().to_owned();
    let (_, version) = published_recipe(w, ipa()).await;
    let batch = post(w, "/batches", json!({"recipe_version_id": version})).await;
    let batch_id: Uuid = batch.body["id"].as_str().unwrap().parse().unwrap();
    let run = Uuid::now_v7();
    let line = Uuid::now_v7();
    sqlx::query("INSERT INTO packaging_runs (id, workspace_id) VALUES ($1, $2)")
        .bind(run)
        .bind(w.ws)
        .execute(&w.pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO packaging_run_lines (id, workspace_id, run_id, batch_id, package_type_id,
             quantity, volume_l, lot_code) VALUES ($1, $2, $3, $4, $5, $6, 30, 'P-1')",
    )
    .bind(line)
    .bind(w.ws)
    .bind(run)
    .bind(batch_id)
    .bind(pt)
    .bind(quantity)
    .execute(&w.pool)
    .await
    .unwrap();
    (lot, line, batch_id)
}

async fn scan(w: &World, user: Uuid, path: &str, body: Value) -> support::Reply {
    call_as(w, user, "POST", path, Some(body), &[]).await
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn create_list_counts_detail_and_retire(pool: PgPool) {
    let w = container_world(&pool).await;
    let pt = package_type_http(&w, "Keg 30 L").await;
    let (ids, _) = create(&w, pt, 3).await;
    assert_eq!(ids.len(), 3);
    // A used serial number is a conflict.
    let dup = post(
        &w,
        "/containers",
        json!({"package_type_id": pt, "serial_numbers": ["KEG-0002"]}),
    )
    .await;
    assert_eq!(dup.status, StatusCode::CONFLICT, "{}", dup.body);
    // Neither list nor prefix: validation.
    let bad = post(&w, "/containers", json!({"package_type_id": pt})).await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY);

    let list = get(&w, "/containers").await;
    assert_eq!(list.status, StatusCode::OK, "{}", list.body);
    assert_eq!(list.body["items"].as_array().unwrap().len(), 3);
    let counts = list.body["status_counts"].as_array().unwrap();
    assert_eq!(counts.len(), 6);
    assert_eq!(counts[0], json!({"status": "clean", "count": 3}));

    let r = call_as(
        &w,
        w.admin,
        "POST",
        &format!("/containers/{}/retire", ids[0]),
        Some(json!({"note": "bent"})),
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert_eq!(r.body["status"], "retired");
    let filtered = get(&w, "/containers?status=retired").await;
    assert_eq!(filtered.body["items"].as_array().unwrap().len(), 1);
    assert_eq!(filtered.body["status_counts"][5]["count"], 1);
    let detail = get(&w, &format!("/containers/{}", ids[0])).await;
    assert_eq!(detail.body["events"][0]["new_status"], "retired");
    // A user without container.manage cannot write.
    let denied = call_as(
        &w,
        w.other,
        "POST",
        &format!("/containers/{}/retire", ids[1]),
        Some(json!({})),
        &[],
    )
    .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn unknown_and_revoked_codes_are_404_and_permissions_shape_actions(pool: PgPool) {
    let w = container_world(&pool).await;
    let pt = package_type(&pool, w.ws, "Keg 30 L").await;
    let (ids, tokens) = create(&w, pt, 1).await;

    let unknown = scan(
        &w,
        w.admin,
        "/scan/resolve",
        json!({"code": "no-such-token"}),
    )
    .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);

    let ok = scan(&w, w.admin, "/scan/resolve", json!({"code": tokens[0]})).await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.body);
    assert_eq!(ok.body["kind"], "container");
    assert_eq!(ok.body["id"], json!(ids[0]));
    let actions: Vec<&str> = ok.body["actions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a.as_str().unwrap())
        .collect();
    assert!(actions.contains(&"fill") && actions.contains(&"change_status"));
    // The QR text form resolves too.
    let qr = scan(
        &w,
        w.admin,
        "/scan/resolve",
        json!({"code": format!("gristora://scan/{}", tokens[0])}),
    )
    .await;
    assert_eq!(qr.status, StatusCode::OK);

    // A read-only user gets no write action.
    let ro = scan(&w, w.other, "/scan/resolve", json!({"code": tokens[0]})).await;
    assert_eq!(ro.body["actions"], json!(["view"]));
    assert_eq!(ok.body["label"], "KEG-0001");
    assert_eq!(tokens[0].len(), 26);
    let denied = scan(
        &w,
        w.other,
        "/scan/status",
        json!({"code": tokens[0], "status": "dirty"}),
    )
    .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);

    // A revoked code is 404.
    sqlx::query("UPDATE scan_code SET revoked_at = now()")
        .execute(&pool)
        .await
        .unwrap();
    let revoked = scan(&w, w.admin, "/scan/resolve", json!({"code": tokens[0]})).await;
    assert_eq!(revoked.status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn status_from_scan_and_batch_scan(pool: PgPool) {
    let w = container_world(&pool).await;
    let pt = package_type(&pool, w.ws, "Keg 30 L").await;
    let (ids, tokens) = create(&w, pt, 2).await;
    let one = scan(
        &w,
        w.admin,
        "/scan/status",
        json!({"code": tokens[0], "status": "out", "note": "pub"}),
    )
    .await;
    assert_eq!(one.status, StatusCode::OK, "{}", one.body);
    assert_eq!(one.body["status"], "out");
    assert_eq!(one.body["out_note"], "pub");
    // Filled needs a fill.
    let filled = scan(
        &w,
        w.admin,
        "/scan/status",
        json!({"code": tokens[0], "status": "filled"}),
    )
    .await;
    assert_eq!(filled.status, StatusCode::UNPROCESSABLE_ENTITY);

    let batch = scan(
        &w,
        w.admin,
        "/scan/status-batch",
        json!({"codes": [tokens[0], tokens[1], "bogus"], "status": "dirty"}),
    )
    .await;
    assert_eq!(batch.status, StatusCode::OK, "{}", batch.body);
    assert_eq!(batch.body["changed"], 2);
    assert_eq!(batch.body["failed"], 1);
    assert_eq!(batch.body["results"][2]["error"], "not_found");
    // A repeat changes nothing.
    let again = scan(
        &w,
        w.admin,
        "/scan/status-batch",
        json!({"codes": [tokens[0]], "status": "dirty"}),
    )
    .await;
    assert_eq!(again.body["changed"], 0);
    assert_eq!(again.body["results"][0]["unchanged"], true);
    let detail = get(&w, &format!("/containers/{}", ids[0])).await;
    assert_eq!(detail.body["events"].as_array().unwrap().len(), 2);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn fill_recall_and_dirty_conflict(pool: PgPool) {
    let w = container_world(&pool).await;
    let pt = package_type(&pool, w.ws, "Keg 30 L").await;
    let (ids, tokens) = create(&w, pt, 3).await;
    let (lot, line, _) = lot_and_line(&w, pt, 3).await;

    // A dirty container cannot be filled.
    scan(
        &w,
        w.admin,
        "/scan/status",
        json!({"code": tokens[2], "status": "dirty"}),
    )
    .await;
    let dirty = post(
        &w,
        "/containers/fill",
        json!({"packaging_run_line_id": line, "lot_id": lot, "container_ids": [ids[2]]}),
    )
    .await;
    assert_eq!(dirty.status, StatusCode::CONFLICT, "{}", dirty.body);
    assert_eq!(dirty.body["errors"][0]["code"], "not_fillable");

    // A clean one and one by scan code are filled.
    let ok = post(
        &w,
        "/containers/fill",
        json!({"packaging_run_line_id": line, "lot_id": lot,
               "container_ids": [ids[0]], "codes": [tokens[1]]}),
    )
    .await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.body);
    assert_eq!(ok.body["items"][0]["status"], "filled");
    assert_eq!(ok.body["items"][1]["last_lot_id"], json!(lot));
    // A filled container cannot be filled again.
    let twice = post(
        &w,
        "/containers/fill",
        json!({"packaging_run_line_id": line, "container_ids": [ids[0]]}),
    )
    .await;
    assert_eq!(twice.status, StatusCode::CONFLICT);

    // Recall: each filled container of the lot.
    let recall = get(&w, &format!("/recall/lots/{lot}/containers")).await;
    assert_eq!(recall.status, StatusCode::OK, "{}", recall.body);
    let items = recall.body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    let mut got: Vec<&str> = items.iter().map(|i| i["id"].as_str().unwrap()).collect();
    got.sort_unstable();
    let mut want = vec![ids[0].as_str(), ids[1].as_str()];
    want.sort_unstable();
    assert_eq!(got, want);
    assert!(items[0]["filled_at"].is_string());
    let missing = get(&w, &format!("/recall/lots/{}/containers", Uuid::now_v7())).await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
}

async fn labels_pdf(w: &World, uri: &str, body: Value) -> (StatusCode, Vec<u8>) {
    let request = support::req("POST", uri, Some(w.admin))
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = w.app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, bytes.to_vec())
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn labels_return_a_pdf_keep_old_codes_and_retire_revokes_them(pool: PgPool) {
    let w = container_world(&pool).await;
    let pt = package_type(&pool, w.ws, "Keg 30 L").await;
    let (ids, tokens) = create(&w, pt, 30).await;
    let (lot, _, _) = lot_and_line(&w, pt, 1).await;
    let base = format!("/v1/workspaces/{}/labels", w.ws);
    let mut targets: Vec<Value> = ids
        .iter()
        .map(|id| json!({"kind": "container", "id": id}))
        .collect();
    targets.push(json!({"kind": "lot", "id": lot}));
    // 31 labels: two pages on A4.
    let (status, bytes) = labels_pdf(&w, &base, json!({"targets": targets})).await;
    assert_eq!(status, StatusCode::OK);
    assert!(bytes.starts_with(b"%PDF-"));
    assert!(bytes.ends_with(b"%%EOF\n"));
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("/Count 2"));
    assert!(text.contains("/MediaBox [0 0 595.28 841.89]"));
    let (_, letter) = labels_pdf(
        &w,
        &format!("{base}?paper=letter"),
        json!({"targets": targets}),
    )
    .await;
    assert!(String::from_utf8_lossy(&letter).contains("/MediaBox [0 0 612.00 792.00]"));
    let (bad, _) = labels_pdf(&w, &format!("{base}?paper=a3"), json!({"targets": targets})).await;
    assert_eq!(bad, StatusCode::UNPROCESSABLE_ENTITY);
    // A reprint does not revoke the earlier code.
    let old = scan(&w, w.admin, "/scan/resolve", json!({"code": tokens[0]})).await;
    assert_eq!(old.status, StatusCode::OK);
    // One audit row and one outbox event for each of the three label prints that succeeded.
    let audit: i64 =
        sqlx::query_scalar("SELECT count(*) FROM audit_log WHERE action = 'label.issue'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(audit, 2);
    let outbox: i64 =
        sqlx::query_scalar("SELECT count(*) FROM outbox WHERE topic = 'label.issued'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(outbox, 2);
    // Retire revokes every code of the container.
    let r = call_as(
        &w,
        w.admin,
        "POST",
        &format!("/containers/{}/retire", ids[0]),
        Some(json!({})),
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    let gone = scan(&w, w.admin, "/scan/resolve", json!({"code": tokens[0]})).await;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);
    let audit: i64 =
        sqlx::query_scalar("SELECT count(*) FROM audit_log WHERE action = 'container.retire'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(audit, 1);
    // An unknown target is a validation error; a retired container has no label.
    let bad = post(
        &w,
        "/labels",
        json!({"targets": [{"kind": "container", "id": Uuid::now_v7()}]}),
    )
    .await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", bad.body);
    let retired = post(
        &w,
        "/labels",
        json!({"targets": [{"kind": "container", "id": ids[0]}]}),
    )
    .await;
    assert_eq!(retired.status, StatusCode::CONFLICT, "{}", retired.body);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn recall_finds_containers_by_the_batch_that_consumed_the_lot(pool: PgPool) {
    let w = container_world(&pool).await;
    let pt = package_type(&pool, w.ws, "Keg 30 L").await;
    let (ids, _) = create(&w, pt, 3).await;
    let (lot, line, batch) = lot_and_line(&w, pt, 3).await;
    // ids[0] names the lot; ids[1] has no lot but its batch consumed the lot (the second path);
    // ids[2] is not filled.
    let a = post(
        &w,
        "/containers/fill",
        json!({"packaging_run_line_id": line, "lot_id": lot, "container_ids": [ids[0]]}),
    )
    .await;
    assert_eq!(a.status, StatusCode::OK, "{}", a.body);
    let b = post(
        &w,
        "/containers/fill",
        json!({"packaging_run_line_id": line, "container_ids": [ids[1]]}),
    )
    .await;
    assert_eq!(b.status, StatusCode::OK, "{}", b.body);
    let consume = post(
        &w,
        &format!("/stock/lots/{lot}/movements"),
        json!({"kind": "consume", "quantity": 1.0, "batch_id": batch}),
    )
    .await;
    assert_eq!(consume.status, StatusCode::CREATED, "{}", consume.body);
    let recall = get(&w, &format!("/recall/lots/{lot}/containers")).await;
    assert_eq!(recall.status, StatusCode::OK, "{}", recall.body);
    let mut got: Vec<&str> = recall.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["id"].as_str().unwrap())
        .collect();
    got.sort_unstable();
    let mut want = vec![ids[0].as_str(), ids[1].as_str()];
    want.sort_unstable();
    assert_eq!(got, want, "each container once, by both paths");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_code_of_another_workspace_is_404_and_actions_follow_permissions(pool: PgPool) {
    let w = container_world(&pool).await;
    let pt = package_type(&pool, w.ws, "Keg 30 L").await;
    let (_, tokens) = create(&w, pt, 1).await;
    // The same user is a member of a second workspace and scans the code there.
    let other_ws = Uuid::now_v7();
    support::insert_workspace(&pool, other_ws).await;
    w.auth.add(
        w.admin,
        other_ws,
        &["field.view", "container.status", "container.manage"],
    );
    sqlx::query("UPDATE workspaces SET mode = 'business' WHERE id = $1")
        .bind(other_ws)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO entitlements (id, workspace_id, feature, source, valid_from, valid_until)
         VALUES ($1, $2, $3, 'promo', now() - interval '1 day', now() + interval '1 day')",
    )
    .bind(Uuid::now_v7())
    .bind(other_ws)
    .bind(billing::PAID_FEATURE)
    .execute(&pool)
    .await
    .unwrap();
    let request = support::req(
        "POST",
        &format!("/v1/workspaces/{other_ws}/scan/resolve"),
        Some(w.admin),
    )
    .header("content-type", "application/json")
    .body(Body::from(json!({"code": tokens[0]}).to_string()))
    .unwrap();
    let reply = support::send(&w.app, request).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND, "{}", reply.body);
    // A caller with only container.status sees change and fill, but not retire or labels.
    let mid = Uuid::now_v7();
    support::insert_user(&pool, mid).await;
    w.auth.add(mid, w.ws, &["field.view", "container.status"]);
    let r = scan(&w, mid, "/scan/resolve", json!({"code": tokens[0]})).await;
    assert_eq!(r.body["actions"], json!(["view", "change_status", "fill"]));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn package_types_list_create_validate_gate_and_replay(pool: PgPool) {
    let w = container_world(&pool).await;
    // A system row, and a row of another workspace.
    let system = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO package_types (id, workspace_id, kind, name, volume_l)
         VALUES ($1, NULL, 'bottle', 'Z system bottle', 0.5)",
    )
    .bind(system)
    .execute(&pool)
    .await
    .unwrap();
    let foreign_ws = Uuid::now_v7();
    support::insert_workspace(&pool, foreign_ws).await;
    package_type(&pool, foreign_ws, "Foreign keg").await;

    let r = post(
        &w,
        "/package-types",
        json!({"kind": "cask", "name": "A cask", "volume_l": 40.5, "single_use": true,
            "material": "oak", "tare_kg": 12.0}),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    assert_eq!(r.body["system"], false);
    assert_eq!(r.body["single_use"], true);
    assert_eq!(r.body["material"], "oak");
    let created = r.body["id"].as_str().unwrap().to_owned();

    let list = get(&w, "/package-types").await;
    assert_eq!(list.status, StatusCode::OK, "{}", list.body);
    let items = list.body["items"].as_array().unwrap();
    let names: Vec<&str> = items.iter().map(|i| i["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["Z system bottle", "A cask"]);
    assert_eq!(items[0]["system"], true);
    assert_eq!(items[1]["id"], created.as_str());

    // A soft-deleted row is not listed.
    sqlx::query("UPDATE package_types SET deleted_at = now() WHERE id = $1")
        .bind(created.parse::<Uuid>().unwrap())
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        get(&w, "/package-types").await.body["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    // Audit row and outbox event.
    let audits: i64 =
        sqlx::query_scalar("SELECT count(*) FROM audit_log WHERE action = 'package_type.create'")
            .fetch_one(&pool)
            .await
            .unwrap();
    let events: i64 =
        sqlx::query_scalar("SELECT count(*) FROM outbox WHERE topic = 'package_type.created'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((audits, events), (1, 1));

    // Permissions: field.view lists, but cannot create.
    let denied = call_as(
        &w,
        w.other,
        "POST",
        "/package-types",
        Some(json!({"kind": "keg", "name": "X", "volume_l": 1})),
        &[("idempotency-key", &Uuid::now_v7().to_string())],
    )
    .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
    let seen = call_as(&w, w.other, "GET", "/package-types", None, &[]).await;
    assert_eq!(seen.status, StatusCode::OK);

    // Validation.
    for body in [
        json!({"kind": "tank", "name": "X", "volume_l": 1}),
        json!({"kind": "keg", "name": "", "volume_l": 1}),
        json!({"kind": "keg", "name": "X", "volume_l": 0}),
        json!({"kind": "keg", "name": "X", "volume_l": 1, "tare_kg": -1}),
        json!({"kind": "keg", "name": "X", "volume_l": 1, "material": "m".repeat(101)}),
    ] {
        let bad = post(&w, "/package-types", body).await;
        assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", bad.body);
    }

    // Idempotent replay returns the same row.
    let key = [("idempotency-key", "pt-replay-1")];
    let body = json!({"kind": "can", "name": "Can 0.44 L", "volume_l": 0.44});
    let first = call_as(
        &w,
        w.admin,
        "POST",
        "/package-types",
        Some(body.clone()),
        &key,
    )
    .await;
    let again = call_as(&w, w.admin, "POST", "/package-types", Some(body), &key).await;
    assert_eq!(first.status, StatusCode::CREATED, "{}", first.body);
    assert_eq!(again.status, StatusCode::CREATED, "{}", again.body);
    assert_eq!(first.body["id"], again.body["id"]);
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM package_types WHERE name = 'Can 0.44 L'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 1);

    // Module gate: Production is off in a Home workspace.
    sqlx::query("UPDATE workspaces SET mode = 'home' WHERE id = $1")
        .bind(w.ws)
        .execute(&pool)
        .await
        .unwrap();
    for gated in [
        get(&w, "/package-types").await,
        get(&w, "/containers").await,
    ] {
        assert_eq!(gated.status, StatusCode::FORBIDDEN);
        assert_eq!(gated.body["code"], "module_disabled");
    }
}
