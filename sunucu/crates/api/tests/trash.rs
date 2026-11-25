//! Endpoint tests of T2.37: recently deleted. Restore within 30 days, no restore after 30 days,
//! a record of another workspace gives 404. Needs PostgreSQL (`#[sqlx::test]`).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

mod domain;
mod support;

use axum::http::StatusCode;
use domain::{World, call_as, get, ipa, post, world};
use serde_json::json;
use sqlx::PgPool;

async fn delete_with_etag(w: &World, path: &str) {
    let etag = get(w, path).await.header("etag").to_owned();
    let r = call_as(w, w.admin, "DELETE", path, None, &[("if-match", &etag)]).await;
    assert_eq!(r.status, StatusCode::NO_CONTENT, "{}", r.body);
}

async fn restore(w: &World, kind: &str, id: &str) -> support::Reply {
    post(w, &format!("/trash/{kind}/{id}/restore"), json!({})).await
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn restore_works_within_30_days_and_not_after(pool: PgPool) {
    let w = world(&pool).await;
    let recipe = post(&w, "/recipes", ipa()).await.body["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let eq = post(
        &w,
        "/equipment",
        json!({"name": "Kettle", "batch_volume_l": 20.0, "efficiency_pct": 72.0}),
    )
    .await
    .body["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let ing = post(
        &w,
        "/ingredients",
        json!({"kind": "fermentable", "name": "My honey", "detail": {"grain_type": "honey", "yield_pct": 75.0, "color_srm": 3.0}}),
    )
    .await;
    assert_eq!(ing.status, StatusCode::CREATED, "{}", ing.body);
    let ing = ing.body["id"].as_str().unwrap().to_owned();

    delete_with_etag(&w, &format!("/recipes/{recipe}")).await;
    delete_with_etag(&w, &format!("/equipment/{eq}")).await;
    delete_with_etag(&w, &format!("/ingredients/{ing}")).await;
    assert_eq!(
        get(&w, &format!("/recipes/{recipe}")).await.status,
        StatusCode::NOT_FOUND
    );

    let list = get(&w, "/trash").await;
    assert_eq!(list.status, StatusCode::OK, "{}", list.body);
    let kinds: Vec<&str> = list.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds.len(), 3, "{kinds:?}");
    for k in ["recipe", "equipment", "ingredient"] {
        assert!(kinds.contains(&k), "{k} missing in {kinds:?}");
    }
    let only = get(&w, "/trash?kind=recipe").await;
    assert_eq!(only.body["items"].as_array().unwrap().len(), 1);
    assert_eq!(
        get(&w, "/trash?kind=nothing").await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );

    // Within 30 days: the restore works and the record is back.
    let ok = restore(&w, "recipe", &recipe).await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.body);
    assert_eq!(ok.body["name"], "House IPA");
    assert_eq!(
        get(&w, &format!("/recipes/{recipe}")).await.status,
        StatusCode::OK
    );
    assert_eq!(
        get(&w, "/trash?kind=recipe").await.body["items"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    // A second restore finds nothing in the trash.
    assert_eq!(
        restore(&w, "recipe", &recipe).await.status,
        StatusCode::NOT_FOUND
    );

    // After 30 days: no restore, and the record leaves the list.
    sqlx::query("UPDATE equipment SET deleted_at = now() - interval '31 days' WHERE id = $1::uuid")
        .bind(&eq)
        .execute(&pool)
        .await
        .unwrap();
    let late = restore(&w, "equipment", &eq).await;
    assert_eq!(late.status, StatusCode::CONFLICT, "{}", late.body);
    assert_eq!(
        get(&w, "/trash?kind=equipment").await.body["items"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let near = restore(&w, "ingredient", &ing).await;
    assert_eq!(near.status, StatusCode::OK, "{}", near.body);
    assert_eq!(
        restore(&w, "unknown", &ing).await.status,
        StatusCode::NOT_FOUND
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn a_record_of_another_workspace_gives_404(pool: PgPool) {
    let a = world(&pool).await;
    let b = world(&pool).await;
    let recipe = post(&b, "/recipes", ipa()).await.body["id"]
        .as_str()
        .unwrap()
        .to_owned();
    delete_with_etag(&b, &format!("/recipes/{recipe}")).await;
    let r = restore(&a, "recipe", &recipe).await;
    assert_eq!(r.status, StatusCode::NOT_FOUND, "{}", r.body);
    assert_eq!(
        get(&a, "/trash").await.body["items"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    // The owner workspace can still restore it.
    assert_eq!(restore(&b, "recipe", &recipe).await.status, StatusCode::OK);
}


#[sqlx::test(migrator = "db::MIGRATOR")]
async fn trash_cursor_returns_each_record_once_in_order(pool: PgPool) {
    let w = world(&pool).await;
    let mut expected = Vec::new();
    for n in 0..5 {
        let mut input = ipa();
        input["name"] = json!(format!("Cursor IPA {n}"));
        let created = post(&w, "/recipes", input).await;
        assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
        let id = created.body["id"].as_str().unwrap().to_owned();
        delete_with_etag(&w, &format!("/recipes/{id}")).await;
        expected.push(id);
    }

    let first = get(&w, "/trash?kind=recipe&limit=2").await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.body);
    assert_eq!(first.body["items"].as_array().unwrap().len(), 2);
    let cursor1 = first.body["next_cursor"].as_str().unwrap();
    let second = get(
        &w,
        &format!("/trash?kind=recipe&limit=2&cursor={cursor1}"),
    )
    .await;
    assert_eq!(second.status, StatusCode::OK, "{}", second.body);
    assert_eq!(second.body["items"].as_array().unwrap().len(), 2);
    let cursor2 = second.body["next_cursor"].as_str().unwrap();
    let third = get(
        &w,
        &format!("/trash?kind=recipe&limit=2&cursor={cursor2}"),
    )
    .await;
    assert_eq!(third.status, StatusCode::OK, "{}", third.body);
    assert_eq!(third.body["items"].as_array().unwrap().len(), 1);
    assert!(third.body["next_cursor"].is_null());

    let mut seen = Vec::new();
    for page in [&first, &second, &third] {
        for item in page.body["items"].as_array().unwrap() {
            seen.push(item["id"].as_str().unwrap().to_owned());
        }
    }
    seen.sort();
    expected.sort();
    assert_eq!(seen, expected);

    let times: Vec<&str> = [&first, &second, &third]
        .into_iter()
        .flat_map(|page| page.body["items"].as_array().unwrap())
        .map(|item| item["deleted_at"].as_str().unwrap())
        .collect();
    for pair in times.windows(2) {
        assert!(pair[0] >= pair[1], "deletions are not newest first: {times:?}");
    }
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn trash_cursor_validates_limits_and_exact_last_page(pool: PgPool) {
    let w = world(&pool).await;
    let mut input = ipa();
    input["name"] = json!("Single deleted IPA");
    let created = post(&w, "/recipes", input).await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    let id = created.body["id"].as_str().unwrap().to_owned();
    delete_with_etag(&w, &format!("/recipes/{id}")).await;

    let exact = get(&w, "/trash?kind=recipe&limit=1").await;
    assert_eq!(exact.status, StatusCode::OK, "{}", exact.body);
    assert_eq!(exact.body["items"].as_array().unwrap().len(), 1);
    assert!(exact.body["next_cursor"].is_null());

    let corrupt = get(&w, "/trash?cursor=not-a-valid-cursor").await;
    assert_eq!(corrupt.status, StatusCode::BAD_REQUEST, "{}", corrupt.body);
    assert_eq!(corrupt.body["errors"][0]["code"], "invalid_cursor");
    assert_eq!(
        get(&w, "/trash?limit=0").await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        get(&w, "/trash?limit=201").await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}
