//! Endpoint test of T2.40: the equipment efficiency split reaches the calculation. Needs
//! PostgreSQL (`#[sqlx::test]`); the engine runs in process.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

mod domain;
mod support;

use axum::http::StatusCode;
use domain::{call_as, ipa, post, world};
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn equipment_efficiency_split_reaches_the_calculation(pool: PgPool) {
    let w = world(&pool).await;
    let base = json!({"name": "Plain", "batch_volume_l": 20.0, "efficiency_pct": 72.0});
    let split = json!({
        "name": "Split", "batch_volume_l": 20.0, "efficiency_pct": 72.0,
        "conversion_efficiency_pct": 90.0, "lauter_efficiency_pct": 70.0
    });
    let plain_eq = post(&w, "/equipment", base).await;
    assert_eq!(plain_eq.status, StatusCode::CREATED, "{}", plain_eq.body);
    let split_eq = post(&w, "/equipment", split).await;
    assert_eq!(split_eq.status, StatusCode::CREATED, "{}", split_eq.body);
    assert_eq!(split_eq.body["conversion_efficiency_pct"], 90.0);

    let og_with = |eq: String| {
        let w = &w;
        async move {
            let mut body = ipa();
            body["version"]["equipment_id"] = json!(eq);
            let r = post(w, "/recipes", body).await;
            assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
            let v = r.body["draft_version_id"].as_str().unwrap().to_owned();
            let c = post(w, &format!("/recipe-versions/{v}/calculate"), json!({})).await;
            assert_eq!(c.status, StatusCode::OK, "{}", c.body);
            c.body["og_sg"].as_f64().unwrap()
        }
    };
    let og_plain = og_with(plain_eq.body["id"].as_str().unwrap().to_owned()).await;
    let og_split = og_with(split_eq.body["id"].as_str().unwrap().to_owned()).await;
    // 90 % x 70 % = 63 % brewhouse efficiency, below the plain 72 %.
    assert!(og_split < og_plain, "{og_split} < {og_plain}");

    // One value alone is rejected on create and on update.
    let one = post(
        &w,
        "/equipment",
        json!({"name": "Half", "batch_volume_l": 20.0, "efficiency_pct": 72.0,
               "conversion_efficiency_pct": 90.0}),
    )
    .await;
    assert_eq!(one.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", one.body);
    let id = plain_eq.body["id"].as_str().unwrap();
    let up = call_as(
        &w,
        w.admin,
        "PATCH",
        &format!("/equipment/{id}"),
        Some(json!({"lauter_efficiency_pct": 80.0})),
        &[],
    )
    .await;
    assert_eq!(up.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", up.body);
}
