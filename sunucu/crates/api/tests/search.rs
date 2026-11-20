//! Endpoint test of T2.27: global search over recipes, batches, ingredients and stock, limited to
//! the workspace. Needs PostgreSQL (`#[sqlx::test]`); the engine runs in process.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::too_many_lines,
    clippy::float_cmp
)]

mod domain;
mod support;

use axum::http::StatusCode;
use domain::{World, get, ipa, post, published_recipe, world};
use serde_json::{Value, json};
use sqlx::PgPool;

async fn stock_world(pool: &PgPool) -> World {
    let w = world(pool).await;
    let mut perms: Vec<&str> = domain::ALL.to_vec();
    perms.push("stock.move");
    w.auth.add(w.admin, w.ws, &perms);
    w
}

async fn malt(w: &World, name: &str) -> String {
    let r = post(
        w,
        "/ingredients",
        json!({"kind": "fermentable", "name": name,
               "detail": {"grain_type": "grain", "yield_pct": 80.0, "color_srm": 2.0}}),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    r.body["id"].as_str().unwrap().to_owned()
}

async fn receive(w: &World, ingredient: &str, quantity: f64) -> Value {
    let r = post(
        w,
        "/stock/receipts",
        json!({"ingredient_id": ingredient, "quantity": quantity}),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    r.body
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn search_finds_four_kinds_and_stays_in_the_workspace(pool: PgPool) {
    let w = stock_world(&pool).await;
    let w2 = stock_world(&pool).await;
    let m = malt(&w, "Zythumalt").await;
    receive(&w, &m, 1.0).await;
    let mut body = ipa();
    body["name"] = json!("Zythum IPA");
    let (_, version) = published_recipe(&w, body).await;
    let b = post(
        &w,
        "/batches",
        json!({"recipe_version_id": version, "name": "Zythum batch"}),
    )
    .await;
    assert_eq!(b.status, StatusCode::CREATED, "{}", b.body);
    // The other workspace has the same names.
    let m2 = malt(&w2, "Zythumalt").await;
    receive(&w2, &m2, 1.0).await;
    let mut body2 = ipa();
    body2["name"] = json!("Zythum IPA");
    published_recipe(&w2, body2).await;

    let r = get(&w, "/search?q=zyth").await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    let hits = r.body["items"].as_array().unwrap();
    let mut kinds: Vec<&str> = hits.iter().map(|h| h["type"].as_str().unwrap()).collect();
    kinds.sort_unstable();
    assert_eq!(
        kinds,
        ["batch", "ingredient", "recipe", "stock"],
        "{}",
        r.body
    );
    let foreign: Vec<String> = sqlx::query_scalar(
        "SELECT id::text FROM recipes WHERE workspace_id = $1 \
         UNION SELECT id::text FROM ingredients WHERE workspace_id = $1 \
         UNION SELECT id::text FROM stock_lots WHERE workspace_id = $1",
    )
    .bind(w2.ws)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(foreign.len(), 3);
    for h in hits {
        assert!(!foreign.contains(&h["id"].as_str().unwrap().to_owned()));
    }
    let only = get(&w, "/search?q=zythum&types=recipe").await;
    assert_eq!(only.body["items"].as_array().unwrap().len(), 1);
    // Batch code with a hyphen.
    let code = get(&w, "/search?q=B-0001").await;
    assert_eq!(code.body["items"][0]["type"], "batch", "{}", code.body);
    assert_eq!(
        get(&w, "/search?q=nomatchxyz").await.body["items"],
        json!([])
    );
    assert_eq!(
        get(&w, "/search?q=").await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        get(&w, "/search?q=a&types=foo").await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}
