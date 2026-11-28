//! Endpoint tests of T3.2: the tools API on a real engine in process. Needs PostgreSQL
//! (`#[sqlx::test]`).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

mod domain;
mod support;

use axum::http::StatusCode;
use domain::{call_as, post, world};
use serde_json::{Value, json};
use sqlx::PgPool;

async fn ok(w: &domain::World, tool: &str, body: Value) -> Value {
    let r = post(w, &format!("/tools/{tool}"), body).await;
    assert_eq!(r.status, StatusCode::OK, "{tool}: {}", r.body);
    assert!(r.body["engine_version"].is_string(), "{tool}: stamp");
    r.body
}

fn near(v: &Value, key: &str, expected: f64, tol: f64) {
    let got = v[key].as_f64().unwrap_or_else(|| panic!("{key} in {v}"));
    assert!((got - expected).abs() <= tol, "{key}: {got} vs {expected}");
}

fn tun() -> Value {
    json!({"mass_kg": 4.0, "specific_heat_cal": 0.12, "volume_l": 30.0})
}

fn ions(ca: f64, mg: f64, na: f64, so4: f64, cl: f64, hco3: f64) -> Value {
    json!({"calcium_ppm": ca, "magnesium_ppm": mg, "sodium_ppm": na,
        "sulfate_ppm": so4, "chloride_ppm": cl, "bicarbonate_ppm": hco3})
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn gravity_tools(pool: PgPool) {
    let w = world(&pool).await;
    let v = ok(
        &w,
        "gravity-convert",
        json!({"value": 1.050, "from": "sg", "to": "plato"}),
    )
    .await;
    near(&v, "value", 12.39, 0.05);
    let v = ok(
        &w,
        "hydrometer-correct",
        json!({"sg": 1.050, "sample_temp_c": 30.0, "calibration_temp_c": 20.0}),
    )
    .await;
    assert!(v["corrected_sg"].as_f64().unwrap() > 1.050);
    let v = ok(&w, "refractometer-unfermented", json!({"brix": 12.0})).await;
    near(&v, "og_sg", 1.046, 0.01);
    let v = ok(
        &w,
        "refractometer-fermenting",
        json!({"og_sg": 1.060, "brix": 8.0}),
    )
    .await;
    assert!(v["fg_sg"].as_f64().unwrap() < 1.060);
    let v = ok(&w, "alcohol", json!({"og_sg": 1.050, "fg_sg": 1.010})).await;
    near(&v, "abv_pct", 5.25, 0.1);
    assert!(v["abw_pct"].as_f64().unwrap() > 3.5);
    let v = ok(
        &w,
        "dilution",
        json!({"sg": 1.070, "volume_l": 20.0, "added_volume_l": 5.0, "added_is_wort": false}),
    )
    .await;
    near(&v, "total_volume_l", 25.0, 1e-9);
    near(&v, "sg", 1.056, 0.001);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn carbonation_tools(pool: PgPool) {
    let w = world(&pool).await;
    let v = ok(
        &w,
        "carbonation",
        json!({"type": "bottle", "co2_volumes": 2.4, "temp_c": 20.0, "volume_l": 20.0,
            "agent": "corn_sugar"}),
    )
    .await;
    assert!(v["priming_g"].as_f64().unwrap() > 50.0);
    assert!(v["dme_g"].as_f64().unwrap() > v["corn_sugar_g"].as_f64().unwrap());
    let v = ok(
        &w,
        "keg-priming",
        json!({"co2_vol": 2.4, "fermentation_max_c": 20.0, "conditioning_c": 20.0,
            "serving_c": 4.0, "beer_l": 19.0, "headspace_l": 1.0, "sugar": "dextrose",
            "purged": true, "mode": "full"}),
    )
    .await;
    assert!(v["sugar_g"].as_f64().unwrap() > 0.0);
    assert!(v["warnings"].is_array());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn mash_temperature_tools(pool: PgPool) {
    let w = world(&pool).await;
    let v = ok(
        &w,
        "strike-temp",
        json!({"water_l": 15.0, "step_c": 66.0, "grain_kg": 5.0, "grain_c": 20.0, "tun": tun()}),
    )
    .await;
    let strike = v["strike_c"].as_f64().unwrap();
    assert!(strike > 66.0 && strike < 100.0, "{strike}");
    let v = ok(
        &w,
        "infusion-temp",
        json!({"added_l": 5.0, "target_c": 72.0, "mash_water_l": 15.0, "mash_c": 66.0,
            "grain_kg": 5.0, "tun": tun()}),
    )
    .await;
    assert!(v["infusion_c"].as_f64().unwrap() > 72.0);
    assert_eq!(v["above_boiling"], json!(false));
    let v = ok(
        &w,
        "infusion-amount",
        json!({"infusion_c": 95.0, "target_c": 72.0, "mash_water_l": 15.0, "mash_c": 66.0,
            "grain_kg": 5.0, "tun": tun()}),
    )
    .await;
    assert!(v["added_l"].as_f64().unwrap() > 0.0);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn water_tools(pool: PgPool) {
    let w = world(&pool).await;
    let volumes = json!({"mash_water_l": 15.0, "sparge_l": 10.0, "mash_available_l": 14.0,
        "boil_l": 25.0, "post_boil_l": 22.0, "trub_loss_l": 1.0, "batch_l": 21.0,
        "has_boil_ingredients": true});
    let v = ok(
        &w,
        "water-calculate",
        json!({"kind": "all_grain",
            "waters": [{"amount_l": 25.0, "ph": 7.4, "ions": ions(50.0, 10.0, 15.0, 60.0, 20.0, 120.0)}],
            "salts": [{"salt": "gypsum", "amount_g": 3.0, "usage": "mash"}],
            "volumes": volumes,
            "mash_ph": {"grains": [{"amount_kg": 5.0, "color_srm": 3.0, "kind": "grain"}],
                "mash_thickness_l_per_kg": 3.0, "target_ph": 5.4}}),
    )
    .await;
    near(&v["base"], "calcium_ppm", 50.0, 1e-6);
    // Gypsum adds calcium and sulfate to the mash water.
    assert!(v["mash"]["calcium_ppm"].as_f64().unwrap() > 50.0);
    assert!(v["mash_stats"]["residual_alkalinity_ppm"].is_number());
    let ph = v["mash_ph"]["estimated_ph"].as_f64().unwrap();
    assert!(ph > 5.0 && ph < 6.5, "{ph}");

    let v = ok(
        &w,
        "acid-adjust",
        json!({"acid": "lactic", "concentration_pct": 88.0, "mash_ph": 5.8, "target_ph": 5.4,
            "sparge_target_ph": 6.0, "grain_kg": 5.0, "mash_thickness_l_per_kg": 3.0,
            "sparge_bicarbonate_ppm": 120.0, "sparge_water_ph": 7.4, "sparge_l": 10.0}),
    )
    .await;
    assert!(v["mash_amount"].as_f64().unwrap() > 0.0);

    let v = ok(
        &w,
        "profile-match",
        json!({"kind": "all_grain",
            "waters": [{"amount_l": 25.0, "ph": 7.0, "ions": ions(10.0, 2.0, 5.0, 5.0, 5.0, 20.0)}],
            "target": ions(100.0, 10.0, 10.0, 150.0, 60.0, 20.0),
            "volumes": volumes, "excluded": ["chalk"]}),
    )
    .await;
    assert!(!v["mash"].as_array().unwrap().is_empty());
    assert!(v["best_fit"]["calcium_ppm"].as_f64().unwrap() > 50.0);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn out_of_range_and_bad_input_are_field_errors(pool: PgPool) {
    let w = world(&pool).await;
    let r = post(&w, "/tools/alcohol", json!({"og_sg": 3.0, "fg_sg": 1.010})).await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", r.body);
    assert_eq!(r.body["code"], "validation_failed");
    assert_eq!(r.body["errors"][0]["field"], "/og_sg");
    assert_eq!(r.body["errors"][0]["code"], "out_of_range");
    // Missing field, unknown choice, wrong type: all reported together.
    let r = post(
        &w,
        "/tools/gravity-convert",
        json!({"value": "x", "from": "kelvin"}),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", r.body);
    let codes: Vec<&str> = r.body["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["code"].as_str().unwrap())
        .collect();
    assert!(
        codes.contains(&"invalid_choice") && codes.contains(&"invalid_type"),
        "{codes:?}"
    );
    // The engine rejects what the catalog does not limit.
    let r = post(
        &w,
        "/tools/infusion-temp",
        json!({"added_l": 0.0, "target_c": 72.0, "mash_water_l": 15.0, "mash_c": 66.0,
            "grain_kg": 5.0, "tun": tun()}),
    )
    .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", r.body);
    assert_eq!(r.body["code"], "calculation_failed");
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn other_workspace_user_gets_404(pool: PgPool) {
    let w = world(&pool).await;
    let stranger = uuid::Uuid::now_v7();
    support::insert_user(&pool, stranger).await;
    let r = call_as(
        &w,
        stranger,
        "POST",
        "/tools/alcohol",
        Some(json!({"og_sg": 1.050, "fg_sg": 1.010})),
        &[],
    )
    .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND, "{}", r.body);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn engine_down_is_503(pool: PgPool) {
    let mut w = world(&pool).await;
    if let Some(stop) = w.stop.take() {
        let _ = stop.send(());
    }
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let r = post(
        &w,
        "/tools/alcohol",
        json!({"og_sg": 1.050, "fg_sg": 1.010}),
    )
    .await;
    assert_eq!(r.status, StatusCode::SERVICE_UNAVAILABLE, "{}", r.body);
}
