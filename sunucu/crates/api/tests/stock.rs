//! Endpoint tests of T2.23 (stock, ledger, deduction, shopping list). Needs
//! PostgreSQL (`#[sqlx::test]`); the engine runs in process.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::too_many_lines,
    clippy::float_cmp
)]

mod domain;
mod support;

use axum::http::StatusCode;
use domain::{World, call_as, get, ipa, post, published_recipe, world};
use serde_json::{Value, json};
use sqlx::PgPool;

async fn stock_world(pool: &PgPool) -> World {
    let w = world(pool).await;
    let mut perms: Vec<&str> = domain::ALL.to_vec();
    perms.extend(["stock.move", "cost.view"]);
    w.auth.add(w.admin, w.ws, &perms);
    w.auth.add(w.other, w.ws, &["recipe.view"]);
    w
}

async fn ingredient(w: &World, kind: &str, name: &str, detail: Value) -> String {
    let r = post(
        w,
        "/ingredients",
        json!({"kind": kind, "name": name, "detail": detail}),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    r.body["id"].as_str().unwrap().to_owned()
}

async fn malt(w: &World, name: &str) -> String {
    ingredient(
        w,
        "fermentable",
        name,
        json!({"grain_type": "grain", "yield_pct": 80.0, "color_srm": 2.0}),
    )
    .await
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

async fn put(w: &World, id: &str, line: &str, amount: f64) -> support::Reply {
    call_as(
        w,
        w.admin,
        "PUT",
        &format!("/batches/{id}/actuals/fermentable/{line}"),
        Some(json!({"actual_amount_kg": amount})),
        &[],
    )
    .await
}

async fn movement(w: &World, lot: &str, body: Value) -> support::Reply {
    post(w, &format!("/stock/lots/{lot}/movements"), body).await
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn receipt_count_adjust_and_negative_warning(pool: PgPool) {
    let w = stock_world(&pool).await;
    let m = malt(&w, "Pale malt").await;
    let first = receive(&w, &m, 10.0).await;
    let lot = first["lot"]["id"].as_str().unwrap().to_owned();
    assert_eq!(first["lot"]["quantity"], 10.0);
    assert_eq!(first["lot"]["lot_code"], "default");
    // The same lot code adds to the same lot.
    let second = receive(&w, &m, 5.0).await;
    assert_eq!(second["lot"]["id"], json!(lot));
    assert_eq!(second["lot"]["quantity"], 15.0);
    // A different unit for the same lot is refused.
    let bad = post(
        &w,
        "/stock/receipts",
        json!({"ingredient_id": m, "quantity": 1.0, "unit": "l"}),
    )
    .await;
    assert_eq!(bad.status, StatusCode::CONFLICT, "{}", bad.body);

    // Stock count: the server calculates the difference.
    let c = movement(&w, &lot, json!({"kind": "count", "counted_quantity": 12.0})).await;
    assert_eq!(c.status, StatusCode::CREATED, "{}", c.body);
    assert_eq!(c.body["movement"]["quantity"], -3.0);
    assert_eq!(c.body["lot"]["quantity"], 12.0);
    assert_eq!(c.body["warnings"], json!([]));

    // An adjustment needs a reason.
    let a = movement(&w, &lot, json!({"kind": "adjust", "quantity": -1.0})).await;
    assert_eq!(a.status, StatusCode::UNPROCESSABLE_ENTITY);
    let a = movement(
        &w,
        &lot,
        json!({"kind": "adjust", "quantity": -1.0, "reason": "spilled"}),
    )
    .await;
    assert_eq!(a.body["lot"]["quantity"], 11.0);

    // A negative stock is allowed and warns.
    let n = movement(&w, &lot, json!({"kind": "consume", "quantity": 15.0})).await;
    assert_eq!(n.status, StatusCode::CREATED, "{}", n.body);
    assert_eq!(n.body["lot"]["quantity"], -4.0);
    assert_eq!(n.body["warnings"][0]["kind"], "negative_stock");
    let neg = get(&w, "/stock/lots?negative=true").await;
    assert_eq!(neg.body["items"].as_array().unwrap().len(), 1);
    let totals = get(&w, "/stock/totals").await;
    assert_eq!(totals.body["items"][0]["quantity"], -4.0);
    assert_eq!(totals.body["items"][0]["cached_quantity"], -4.0);
    assert_eq!(totals.body["warnings"][0]["kind"], "negative_stock");
    let ledger = get(&w, &format!("/stock/movements?lot_id={lot}&sort=id")).await;
    let kinds: Vec<&str> = ledger.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, ["receive", "receive", "count", "adjust", "consume"]);

    // A member without `stock.move` reads but does not write.
    let denied = call_as(
        &w,
        w.other,
        "POST",
        "/stock/receipts",
        Some(json!({"ingredient_id": m, "quantity": 1.0})),
        &[("idempotency-key", "k-denied")],
    )
    .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
    let read = call_as(&w, w.other, "GET", &format!("/stock/lots/{lot}"), None, &[]).await;
    assert_eq!(read.status, StatusCode::OK);
    assert!(read.body["unit_cost"].is_null());
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn two_movements_at_the_same_time_give_the_right_total(pool: PgPool) {
    let w = stock_world(&pool).await;
    let m = malt(&w, "Munich malt").await;
    let lot = receive(&w, &m, 100.0).await["lot"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut calls = Vec::new();
    for i in 0..20 {
        let body = if i % 2 == 0 {
            json!({"kind": "consume", "quantity": 2.0})
        } else {
            json!({"kind": "receive", "quantity": 1.0})
        };
        calls.push(movement(&w, &lot, body));
    }
    for r in futures_util::future::join_all(calls).await {
        assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    }
    // 100 - 10 * 2 + 10 * 1
    let l = get(&w, &format!("/stock/lots/{lot}")).await;
    assert_eq!(l.body["quantity"], 90.0);
    let totals = get(&w, "/stock/totals").await;
    assert_eq!(totals.body["items"][0]["quantity"], 90.0);
    assert_eq!(totals.body["items"][0]["cached_quantity"], 90.0);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn batch_deduction_is_safe_to_repeat(pool: PgPool) {
    let w = stock_world(&pool).await;
    let m = malt(&w, "Pale ale malt").await;
    let mut body = ipa();
    body["version"]["fermentables"][0]["ingredient_id"] = json!(m);
    let (_, version) = published_recipe(&w, body).await;
    let v = get(&w, &format!("/recipe-versions/{version}")).await.body;
    let line = v["fermentables"][0]["id"].as_str().unwrap().to_owned();
    let b = post(&w, "/batches", json!({"recipe_version_id": version})).await;
    let id = b.body["id"].as_str().unwrap().to_owned();
    assert_eq!(put(&w, &id, &line, 4.5).await.status, StatusCode::OK);

    // No lot yet: the line is skipped.
    let none = post(&w, &format!("/batches/{id}/stock-deduction"), json!({})).await;
    assert_eq!(none.status, StatusCode::OK, "{}", none.body);
    assert_eq!(none.body["skipped"][0]["reason"], "no_lot");

    let lot = receive(&w, &m, 10.0).await["lot"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let d = post(&w, &format!("/batches/{id}/stock-deduction"), json!({})).await;
    assert_eq!(d.body["movements"].as_array().unwrap().len(), 1, "{}", d.body);
    assert_eq!(d.body["movements"][0]["quantity"], -4.5);
    assert_eq!(
        get(&w, &format!("/stock/lots/{lot}")).await.body["quantity"],
        5.5
    );
    // Repeat: nothing moves.
    let again = post(&w, &format!("/batches/{id}/stock-deduction"), json!({})).await;
    assert_eq!(again.body["movements"], json!([]));
    assert_eq!(again.body["skipped"][0]["reason"], "already_deducted");
    // A larger actual amount deducts the difference, and a shortage warns.
    assert_eq!(put(&w, &id, &line, 16.0).await.status, StatusCode::OK);
    let more = post(&w, &format!("/batches/{id}/stock-deduction"), json!({})).await;
    assert_eq!(more.body["movements"][0]["quantity"], -11.5);
    assert_eq!(more.body["warnings"][0]["kind"], "negative_stock");
    let by_batch = get(&w, &format!("/stock/movements?batch_id={id}")).await;
    assert_eq!(by_batch.body["items"].as_array().unwrap().len(), 2);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn shopping_list_from_recipe_subtracts_stock(pool: PgPool) {
    let w = stock_world(&pool).await;
    let m = malt(&w, "Pilsner malt").await;
    let h = ingredient(&w, "hop", "Saaz", json!({"alpha_pct": 3.5})).await;
    let mut body = ipa();
    body["version"]["fermentables"][0]["ingredient_id"] = json!(m);
    body["version"]["hops"][0]["ingredient_id"] = json!(h);
    let (_, version) = published_recipe(&w, body).await;
    receive(&w, &m, 2.0).await;
    let r = post(
        &w,
        "/shopping-list/from-recipe",
        json!({"recipe_version_id": version}),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let items = r.body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    let malt_item = items
        .iter()
        .find(|i| i["ingredient_id"] == json!(m))
        .unwrap();
    assert!((malt_item["quantity"].as_f64().unwrap() - 2.5).abs() < 1e-9);
    assert!(r.body["unlinked"].as_array().unwrap().len() >= 2);
    // Repeat renews the items and adds none.
    let again = post(
        &w,
        "/shopping-list/from-recipe",
        json!({"recipe_version_id": version}),
    )
    .await;
    assert_eq!(again.body["items"].as_array().unwrap().len(), 2);
    let list = get(&w, "/shopping-list").await;
    assert_eq!(list.body["items"].as_array().unwrap().len(), 2);
    // Enough stock: the item leaves the list.
    receive(&w, &m, 10.0).await;
    let covered = post(
        &w,
        "/shopping-list/from-recipe",
        json!({"recipe_version_id": version}),
    )
    .await;
    assert_eq!(covered.body["covered"][0]["ingredient_id"], json!(m));
    let list = get(&w, "/shopping-list").await;
    assert_eq!(list.body["items"].as_array().unwrap().len(), 1);
    // Tick off the remaining item.
    let item = list.body["items"][0].clone();
    let id = item["id"].as_str().unwrap();
    let p = call_as(
        &w,
        w.admin,
        "PATCH",
        &format!("/shopping-list/{id}"),
        Some(json!({"checked": true})),
        &[("if-match", &format!("\"{}\"", item["version"]))],
    )
    .await;
    assert_eq!(p.status, StatusCode::OK, "{}", p.body);
    assert_eq!(p.body["checked"], true);
}
