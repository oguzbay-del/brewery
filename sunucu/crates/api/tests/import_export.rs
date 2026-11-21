//! Endpoint tests of T2.26: recipe import (`BeerXML`, `BeerJSON`, Brewfather) with the mapping
//! decisions and the calculated values, the safety limits, and the export job rules. Needs
//! PostgreSQL (`#[sqlx::test]`); the engine runs in process. The files the worker writes are
//! tested in `crates/worker/tests/export.rs`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::too_many_lines,
    clippy::float_cmp
)]

mod domain;
mod support;

use axum::{body::Body, http::StatusCode};
use domain::{World, call_as, get, ipa, post, published_recipe, world};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{Reply, req, send};

const BEERXML: &str = include_str!("../../interchange/tests/fixtures/sample.beerxml");
const BREWFATHER: &str = include_str!("../../interchange/tests/fixtures/brewfather.json");
const BOMB: &str = include_str!("../../interchange/tests/fixtures/xml_bomb.xml");
const XXE: &str = include_str!("../../interchange/tests/fixtures/xml_external_entity.xml");

async fn exporter_world(pool: &PgPool) -> World {
    let w = world(pool).await;
    let mut perms: Vec<&str> = domain::ALL.to_vec();
    perms.push("reports.export");
    w.auth.add(w.admin, w.ws, &perms);
    w
}

async fn upload(w: &World, query: &str, body: Vec<u8>) -> Reply {
    let request = req(
        "POST",
        &format!("/v1/workspaces/{}/imports{query}", w.ws),
        Some(w.admin),
    )
    .body(Body::from(body))
    .unwrap();
    send(&w.app, request).await
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn beerxml_import_shows_source_and_calculated_values(pool: PgPool) {
    let w = exporter_world(&pool).await;
    let up = upload(&w, "?file_name=sample.beerxml", BEERXML.as_bytes().to_vec()).await;
    assert_eq!(up.status, StatusCode::CREATED, "{}", up.body);
    assert_eq!(up.body["format"], "beerxml");
    assert_eq!(up.body["status"], "parsed");
    assert_eq!(up.body["recipes"][0]["name"], "Test Pale & Hoppy");
    assert_eq!(up.body["recipes"][0]["source"]["og_sg"], 1.052);
    assert_eq!(up.body["recipes"][0]["lines"], json!([2, 2, 1, 1]));
    let id = up.body["id"].as_str().unwrap().to_owned();

    // A library ingredient with the same name as a line is matched.
    let m = post(
        &w,
        "/ingredients",
        json!({"kind": "fermentable", "name": "test pale malt",
               "detail": {"grain_type": "grain", "yield_pct": 79.0, "color_srm": 2.0}}),
    )
    .await;
    assert_eq!(m.status, StatusCode::CREATED, "{}", m.body);

    let apply = post(&w, &format!("/imports/{id}/apply"), json!({})).await;
    assert_eq!(apply.status, StatusCode::CREATED, "{}", apply.body);
    let r = &apply.body["recipes"][0];
    assert_eq!(r["source"]["og_sg"], 1.052);
    let calc = &r["calculated"];
    assert!(calc.is_object(), "no calculated values: {}", apply.body);
    let og = calc["og_sg"].as_f64().unwrap();
    assert!((1.03..1.08).contains(&og), "og {og}");
    assert!(calc["ibu"].as_f64().unwrap() > 10.0);
    assert!(calc["color_srm"].as_f64().unwrap() > 1.0);

    // The decisions are saved in the import source.
    let d = &apply.body["mapping_decisions"];
    assert_eq!(d["recipes"][0]["equipment"]["decision"], "created");
    let matches = d["recipes"][0]["ingredients"].as_array().unwrap();
    let malt = matches
        .iter()
        .find(|m| m["name"] == "Test Pale Malt")
        .unwrap();
    assert_eq!(malt["decision"], "matched");
    let hop = matches.iter().find(|m| m["name"] == "Test Hop A").unwrap();
    assert_eq!(hop["decision"], "kept_file_values");
    let stored = get(&w, &format!("/imports/{id}")).await;
    assert_eq!(stored.body["status"], "applied");
    assert_eq!(stored.body["mapping_decisions"]["format"], "beerxml");

    // The recipe exists with a draft that holds the lines and the targets.
    let recipe_id = r["recipe_id"].as_str().unwrap();
    let rec = get(&w, &format!("/recipes/{recipe_id}")).await;
    assert_eq!(rec.status, StatusCode::OK, "{}", rec.body);
    let v = get(
        &w,
        &format!("/recipe-versions/{}", r["version_id"].as_str().unwrap()),
    )
    .await;
    assert_eq!(v.body["fermentables"].as_array().unwrap().len(), 2);
    assert_eq!(v.body["target_og_sg"], 1.052);

    // A second apply is refused.
    let again = post(&w, &format!("/imports/{id}/apply"), json!({})).await;
    assert_eq!(again.status, StatusCode::CONFLICT, "{}", again.body);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn brewfather_import_creates_recipe_and_equipment(pool: PgPool) {
    let w = exporter_world(&pool).await;
    let up = upload(&w, "?format=brewfather", BREWFATHER.as_bytes().to_vec()).await;
    assert_eq!(up.status, StatusCode::CREATED, "{}", up.body);
    assert_eq!(up.body["format"], "brewfather");
    assert_eq!(up.body["recipes"][0]["source"]["ibu"], 55.0);
    let id = up.body["id"].as_str().unwrap();
    let apply = post(&w, &format!("/imports/{id}/apply"), json!({})).await;
    assert_eq!(apply.status, StatusCode::CREATED, "{}", apply.body);
    assert!(apply.body["recipes"][0]["equipment_id"].is_string());
    let eq: i64 = sqlx::query_scalar("SELECT count(*) FROM equipment WHERE name = 'Test Rig'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(eq, 1);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn beerjson_import_round_trips_a_beerxml_file(pool: PgPool) {
    let w = exporter_world(&pool).await;
    let doc = interchange::beerxml::parse(BEERXML.as_bytes()).unwrap();
    let json_file = interchange::beerjson::write(&doc.recipes);
    let up = upload(&w, "", json_file.into_bytes()).await;
    assert_eq!(up.status, StatusCode::CREATED, "{}", up.body);
    assert_eq!(up.body["format"], "beerjson");
    assert_eq!(up.body["recipes"][0]["source"]["og_sg"], 1.052);
    assert_eq!(up.body["recipes"][0]["lines"], json!([2, 2, 1, 1]));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn unsafe_xml_and_big_files_are_rejected(pool: PgPool) {
    let w = exporter_world(&pool).await;
    for (file, code) in [
        (BOMB, "xml_doctype_not_allowed"),
        (XXE, "xml_doctype_not_allowed"),
    ] {
        let r = upload(&w, "?format=beerxml", file.as_bytes().to_vec()).await;
        assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", r.body);
        assert_eq!(r.body["errors"][0]["code"], code);
    }
    let r = upload(
        &w,
        "?format=beerxml",
        b"<RECIPES><RECIPE><NAME>&xxe;</NAME></RECIPE></RECIPES>".to_vec(),
    )
    .await;
    assert_eq!(r.body["errors"][0]["code"], "xml_entity_not_allowed");
    // Larger than the 8 MiB limit of the route.
    let big = vec![b' '; 9 * 1024 * 1024];
    let r = upload(&w, "?format=beerxml", big).await;
    assert_eq!(r.status, StatusCode::PAYLOAD_TOO_LARGE, "{}", r.body);
    // Nothing was stored.
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM import_sources")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 0);
    let r = upload(&w, "?format=beerxml", b"not xml at all".to_vec()).await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", r.body);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn import_needs_the_edit_permission(pool: PgPool) {
    let w = exporter_world(&pool).await;
    w.auth.add(w.other, w.ws, &["recipe.view"]);
    let request = req(
        "POST",
        &format!("/v1/workspaces/{}/imports", w.ws),
        Some(w.other),
    )
    .body(Body::from(BEERXML))
    .unwrap();
    assert_eq!(send(&w.app, request).await.status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn full_export_is_limited_to_one_per_day_but_batch_export_is_not(pool: PgPool) {
    let w = exporter_world(&pool).await;
    let (recipe, version) = published_recipe(&w, ipa()).await;
    let batch = post(
        &w,
        "/batches",
        json!({"recipe_version_id": version, "name": "Export batch"}),
    )
    .await;
    assert_eq!(batch.status, StatusCode::CREATED, "{}", batch.body);
    let batch_id = batch.body["id"].as_str().unwrap().to_owned();

    let first = post(
        &w,
        "/exports",
        json!({"scope": "workspace", "format": "zip"}),
    )
    .await;
    assert_eq!(first.status, StatusCode::ACCEPTED, "{}", first.body);
    assert_eq!(first.body["status"], "queued");
    let second = post(
        &w,
        "/exports",
        json!({"scope": "workspace", "format": "zip"}),
    )
    .await;
    assert_eq!(
        second.status,
        StatusCode::TOO_MANY_REQUESTS,
        "{}",
        second.body
    );
    assert!(second.headers.contains_key("retry-after"));
    assert_eq!(second.body["errors"][0]["code"], "full_export_once_per_day");

    // A batch export (also a date range) works after the full export, as often as needed.
    for _ in 0..3 {
        let b = post(
            &w,
            "/exports",
            json!({"scope": "batch", "scope_id": batch_id, "format": "csv",
                   "from": "2026-01-01T00:00:00Z", "to": "2026-12-31T00:00:00Z"}),
        )
        .await;
        assert_eq!(b.status, StatusCode::ACCEPTED, "{}", b.body);
    }
    let r = post(
        &w,
        "/exports",
        json!({"scope": "recipe", "scope_id": recipe, "format": "beerjson"}),
    )
    .await;
    assert_eq!(r.status, StatusCode::ACCEPTED, "{}", r.body);
    let got = get(&w, &format!("/exports/{}", r.body["id"].as_str().unwrap())).await;
    assert_eq!(got.status, StatusCode::OK);
    assert!(got.body["download"].is_null());

    // A failed full export does not block a new one; another user has their own allowance.
    sqlx::query("UPDATE export_jobs SET status = 'failed' WHERE scope = 'workspace'")
        .execute(&pool)
        .await
        .unwrap();
    let retry = post(
        &w,
        "/exports",
        json!({"scope": "workspace", "format": "zip"}),
    )
    .await;
    assert_eq!(retry.status, StatusCode::ACCEPTED, "{}", retry.body);
    let mut perms: Vec<&str> = domain::ALL.to_vec();
    perms.push("reports.export");
    w.auth.add(w.other, w.ws, &perms);
    let other = call_as(
        &w,
        w.other,
        "POST",
        "/exports",
        Some(json!({"scope": "workspace", "format": "zip"})),
        &[],
    )
    .await;
    assert_eq!(other.status, StatusCode::ACCEPTED, "{}", other.body);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn export_requests_are_validated(pool: PgPool) {
    let w = exporter_world(&pool).await;
    for body in [
        json!({"scope": "nope", "format": "zip"}),
        json!({"scope": "workspace", "format": "csv"}),
        json!({"scope": "recipe", "format": "beerjson"}),
        json!({"scope": "recipe", "scope_id": uuid::Uuid::now_v7(), "format": "beerjson"}),
        json!({"scope": "batch", "scope_id": uuid::Uuid::now_v7(), "format": "beerxml"}),
        json!({"scope": "workspace", "format": "zip", "from": "2026-01-01T00:00:00Z"}),
    ] {
        let r = post(&w, "/exports", body.clone()).await;
        assert_eq!(
            r.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{body}: {}",
            r.body
        );
    }
    // Without the permission.
    w.auth.add(w.other, w.ws, &["recipe.view"]);
    let r = call_as(
        &w,
        w.other,
        "POST",
        "/exports",
        Some(json!({"scope": "workspace", "format": "zip"})),
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    let _: Value = r.body;
}
