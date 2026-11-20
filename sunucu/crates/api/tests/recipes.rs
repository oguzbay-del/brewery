//! Endpoint tests of T2.21: recipe, calculation, publish, immutable versions, the `412` body,
//! scaling, comparison, folders, tags, favorites, the edit session, mead. Needs PostgreSQL
//! (`#[sqlx::test]`); the engine runs in process.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines, clippy::float_cmp)]

mod domain;
mod support;

use axum::http::StatusCode;
use domain::{call_as, delete, get, ipa, patch, post, post_with, world};
use serde_json::{Value, json};
use sqlx::PgPool;

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn recipe_to_calculation_to_publish_keeps_old_version(pool: PgPool) {
    let w = world(&pool).await;
    let r = post(&w, "/recipes", ipa()).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let recipe = r.body["id"].as_str().unwrap().to_owned();
    let v1 = r.body["draft_version_id"].as_str().unwrap().to_owned();
    assert!(r.body["approved_version_id"].is_null());

    // Calculation of the draft.
    let c = post(&w, &format!("/recipe-versions/{v1}/calculate"), json!({})).await;
    assert_eq!(c.status, StatusCode::OK, "{}", c.body);
    let og = c.body["og_sg"].as_f64().unwrap();
    assert!(og > 1.040 && og < 1.080, "og {og}");
    assert!(c.body["ibu"].as_f64().unwrap() > 10.0);
    // 200 billion cells in 20 L = 10 million per mL.
    assert!((c.body["est_cells_ml"].as_f64().unwrap() - 10.0).abs() < 1e-9);

    // Publish.
    let draft = get(&w, &format!("/recipe-versions/{v1}")).await;
    assert_eq!(draft.body["status"], "draft");
    assert_eq!(draft.body["fermentables"].as_array().unwrap().len(), 2);
    let etag = draft.header("etag").to_owned();
    let p = post_with(
        &w,
        &format!("/recipe-versions/{v1}/publish"),
        json!({}),
        &[("if-match", &etag)],
    )
    .await;
    assert_eq!(p.status, StatusCode::OK, "{}", p.body);
    assert_eq!(p.body["status"], "approved");
    assert_eq!(p.body["calc_summary"]["og_sg"].as_f64().unwrap(), og);
    let published_v1 = get(&w, &format!("/recipe-versions/{v1}")).await.body;
    let rec = get(&w, &format!("/recipes/{recipe}")).await;
    assert_eq!(rec.body["approved_version_id"], json!(v1));
    assert!(rec.body["draft_version_id"].is_null());

    // A published version does not change.
    let etag1 = get(&w, &format!("/recipe-versions/{v1}"))
        .await
        .header("etag")
        .to_owned();
    let denied = patch(
        &w,
        &format!("/recipe-versions/{v1}"),
        json!({"efficiency_pct": 60.0}),
        &etag1,
    )
    .await;
    assert_eq!(denied.status, StatusCode::CONFLICT, "{}", denied.body);
    // The calculation of a published version is the stored summary.
    let stored = post(&w, &format!("/recipe-versions/{v1}/calculate"), json!({})).await;
    assert_eq!(stored.body["source"], "stored");

    // New draft, change, stale save gives 412 with the difference.
    let d = post(
        &w,
        &format!("/recipes/{recipe}/versions"),
        json!({"change_note": "more malt"}),
    )
    .await;
    assert_eq!(d.status, StatusCode::CREATED, "{}", d.body);
    assert_eq!(d.body["number"], 2);
    let v2 = d.body["id"].as_str().unwrap().to_owned();
    let again = post(&w, &format!("/recipes/{recipe}/versions"), json!({})).await;
    assert_eq!(again.status, StatusCode::CONFLICT);
    let stale = d.header("etag").to_owned();
    let mut fermentables = d.body["fermentables"].clone();
    fermentables[0]["amount_kg"] = json!(5.5);
    let ok = patch(
        &w,
        &format!("/recipe-versions/{v2}"),
        json!({"fermentables": fermentables}),
        &stale,
    )
    .await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.body);
    assert_eq!(ok.body["fermentables"][0]["amount_kg"], 5.5);
    let fresh = ok.header("etag").to_owned();
    assert_ne!(fresh, stale);
    let conflict = patch(
        &w,
        &format!("/recipe-versions/{v2}"),
        json!({"efficiency_pct": 65.0}),
        &stale,
    )
    .await;
    assert_eq!(
        conflict.status,
        StatusCode::PRECONDITION_FAILED,
        "{}",
        conflict.body
    );
    assert_eq!(conflict.body["code"], "version_mismatch");
    assert_eq!(conflict.body["current"]["version"], ok.body["version"]);
    assert_eq!(
        conflict.body["current"]["conflicting_fields"],
        json!(["efficiency_pct"])
    );
    let missing = call_as(
        &w,
        w.admin,
        "PATCH",
        &format!("/recipe-versions/{v2}"),
        Some(json!({"efficiency_pct": 65.0})),
        &[],
    )
    .await;
    assert_eq!(missing.status, StatusCode::PRECONDITION_REQUIRED);

    // Compare the two versions.
    let cmp = get(&w, &format!("/recipe-versions/{v1}/compare/{v2}")).await;
    assert_eq!(cmp.status, StatusCode::OK, "{}", cmp.body);
    let changed = &cmp.body["lines"]["fermentables"]["changed"];
    assert_eq!(changed.as_array().unwrap().len(), 1);
    assert_eq!(changed[0]["fields"][0]["field"], "amount_kg");

    // Publish v2; v1 is still the same.
    let p2 = post_with(
        &w,
        &format!("/recipe-versions/{v2}/publish"),
        json!({}),
        &[("if-match", &fresh)],
    )
    .await;
    assert_eq!(p2.status, StatusCode::OK, "{}", p2.body);
    assert!(p2.body["calc_summary"]["og_sg"].as_f64().unwrap() > og);
    let after = get(&w, &format!("/recipe-versions/{v1}")).await.body;
    assert_eq!(after, published_v1);
    let list = get(&w, &format!("/recipes/{recipe}/versions")).await;
    assert_eq!(list.body["items"].as_array().unwrap().len(), 2);
    assert_eq!(list.body["items"][0]["number"], 2);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn library_snapshot_and_mead(pool: PgPool) {
    let w = world(&pool).await;
    let honey = post(
        &w,
        "/ingredients",
        json!({"kind": "fermentable", "name": "Wildflower honey",
               "detail": {"grain_type": "honey", "yield_pct": 75.0, "color_srm": 3.0}}),
    )
    .await;
    assert_eq!(honey.status, StatusCode::CREATED, "{}", honey.body);
    let honey_id = honey.body["id"].as_str().unwrap().to_owned();
    let r = post(
        &w,
        "/recipes",
        json!({"name": "Traditional mead", "version": {
            "kind": "mead", "batch_volume_l": 10.0, "boil_time_min": 0.0, "efficiency_pct": 100.0,
            "fermentables": [{"ingredient_id": honey_id, "amount_kg": 3.0, "usage": "primary"}],
            "yeasts": [{"name": "Mead yeast", "form": "dry", "min_attenuation_pct": 75.0,
                        "max_attenuation_pct": 90.0, "viable_cells_billion": 100.0, "packs": 2.0}]
        }}),
    )
    .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    assert_eq!(r.body["kind"], "mead");
    let v = r.body["draft_version_id"].as_str().unwrap();
    let line = &get(&w, &format!("/recipe-versions/{v}")).await.body["fermentables"][0];
    // The line holds copies of the library properties.
    assert_eq!(line["name"], "Wildflower honey");
    assert_eq!(line["yield_pct"], 75.0);
    assert_eq!(line["grain_type"], "honey");
    // The library changes later; the line does not.
    let etag = get(&w, &format!("/ingredients/{honey_id}"))
        .await
        .header("etag")
        .to_owned();
    let up = patch(
        &w,
        &format!("/ingredients/{honey_id}"),
        json!({"detail": {"yield_pct": 60.0}}),
        &etag,
    )
    .await;
    assert_eq!(up.status, StatusCode::OK, "{}", up.body);
    let line = &get(&w, &format!("/recipe-versions/{v}")).await.body["fermentables"][0];
    assert_eq!(line["yield_pct"], 75.0);

    let c = post(&w, &format!("/recipe-versions/{v}/calculate"), json!({})).await;
    assert_eq!(c.status, StatusCode::OK, "{}", c.body);
    assert_eq!(c.body["honey_pct"], 100.0);
    // 2 packs of 100 billion in 10 L = 20 million per mL.
    assert!((c.body["est_cells_ml"].as_f64().unwrap() - 20.0).abs() < 1e-9);
    assert!(c.body["og_sg"].as_f64().unwrap() > 1.05);

    let bad = post(&w, "/recipes", json!({"name": "X", "version": {"kind": "beer", "batch_volume_l": 5.0, "efficiency_pct": 70.0}})).await;
    assert_eq!(bad.status, StatusCode::UNPROCESSABLE_ENTITY);
    let cider = post(&w, "/recipes", json!({"name": "Cider", "version": {"kind": "cider", "batch_volume_l": 5.0, "efficiency_pct": 100.0}})).await;
    assert_eq!(cider.status, StatusCode::CREATED);
    let seltzer = post(&w, "/recipes", json!({"name": "Seltzer", "version": {"kind": "seltzer", "batch_volume_l": 5.0, "efficiency_pct": 100.0}})).await;
    assert_eq!(seltzer.status, StatusCode::CREATED);
    let only_mead = get(&w, "/recipes?kind=mead").await;
    assert_eq!(only_mead.body["items"].as_array().unwrap().len(), 1);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn scaling_folders_tags_favorites(pool: PgPool) {
    let w = world(&pool).await;
    let folder = post(&w, "/folders", json!({"name": "IPA"})).await;
    assert_eq!(folder.status, StatusCode::CREATED);
    let folder_id = folder.body["id"].as_str().unwrap().to_owned();
    let tag = post(&w, "/tags", json!({"name": "Summer", "color": "#ffaa00"})).await;
    assert_eq!(tag.status, StatusCode::CREATED);
    let tag_id = tag.body["id"].as_str().unwrap().to_owned();
    let dup = post(&w, "/tags", json!({"name": "summer"})).await;
    assert_eq!(dup.status, StatusCode::CONFLICT);

    let mut body = ipa();
    body["folder_id"] = json!(folder_id);
    body["tag_ids"] = json!([tag_id]);
    let r = post(&w, "/recipes", body).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let recipe = r.body["id"].as_str().unwrap().to_owned();
    let v = r.body["draft_version_id"].as_str().unwrap().to_owned();
    assert_eq!(r.body["tag_ids"], json!([tag_id]));

    let fav = call_as(
        &w,
        w.admin,
        "PUT",
        &format!("/recipes/{recipe}/favorite"),
        None,
        &[],
    )
    .await;
    assert_eq!(fav.status, StatusCode::NO_CONTENT);
    let fav2 = call_as(
        &w,
        w.admin,
        "PUT",
        &format!("/recipes/{recipe}/favorite"),
        None,
        &[],
    )
    .await;
    assert_eq!(fav2.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(&w, &format!("/recipes/{recipe}")).await.body["is_favorite"],
        true
    );
    // The favorite is private to the user.
    let other = call_as(&w, w.other, "GET", &format!("/recipes/{recipe}"), None, &[]).await;
    assert_eq!(other.body["is_favorite"], false);
    let favs = get(&w, "/recipes?favorite=true").await;
    assert_eq!(favs.body["items"].as_array().unwrap().len(), 1);
    let none = call_as(&w, w.other, "GET", "/recipes?favorite=true", None, &[]).await;
    assert_eq!(none.body["items"].as_array().unwrap().len(), 0);
    let by_tag = get(
        &w,
        &format!("/recipes?tag_id={tag_id}&folder_id={folder_id}&q=ipa"),
    )
    .await;
    assert_eq!(by_tag.body["items"].as_array().unwrap().len(), 1);

    // A folder with a recipe cannot be deleted.
    let busy = delete(&w, &format!("/folders/{folder_id}")).await;
    assert_eq!(busy.status, StatusCode::CONFLICT);
    let untag = delete(&w, &format!("/recipes/{recipe}/tags/{tag_id}")).await;
    assert_eq!(untag.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(&w, &format!("/recipes/{recipe}")).await.body["tag_ids"],
        json!([])
    );

    // Scaling preview.
    let s = post(
        &w,
        &format!("/recipe-versions/{v}/scale"),
        json!({"factor": 2.0}),
    )
    .await;
    assert_eq!(s.status, StatusCode::OK, "{}", s.body);
    assert_eq!(s.body["batch_volume_l"], 40.0);
    assert_eq!(s.body["fermentables"][0]["amount_kg"], 9.0);
    assert!((s.body["hops"][0]["amount_kg"].as_f64().unwrap() - 0.06).abs() < 1e-12);
    assert!(s.body["summary"]["og_sg"].as_f64().unwrap() > 1.04);
    let by_volume = post(
        &w,
        &format!("/recipe-versions/{v}/scale"),
        json!({"batch_volume_l": 10.0, "calculate": false}),
    )
    .await;
    assert_eq!(by_volume.body["factor"], 0.5);
    assert!(by_volume.body["summary"].is_null());
    let both = post(&w, &format!("/recipe-versions/{v}/scale"), json!({})).await;
    assert_eq!(both.status, StatusCode::UNPROCESSABLE_ENTITY);

    // Lock and delete.
    let rec = get(&w, &format!("/recipes/{recipe}")).await;
    let etag = rec.header("etag").to_owned();
    let locked = patch(
        &w,
        &format!("/recipes/{recipe}"),
        json!({"is_locked": true}),
        &etag,
    )
    .await;
    assert_eq!(locked.status, StatusCode::OK, "{}", locked.body);
    let vetag = get(&w, &format!("/recipe-versions/{v}"))
        .await
        .header("etag")
        .to_owned();
    let denied = patch(
        &w,
        &format!("/recipe-versions/{v}"),
        json!({"efficiency_pct": 70.0}),
        &vetag,
    )
    .await;
    assert_eq!(denied.status, StatusCode::CONFLICT);
    let etag = locked.header("etag").to_owned();
    let del = call_as(
        &w,
        w.admin,
        "DELETE",
        &format!("/recipes/{recipe}"),
        None,
        &[("if-match", &etag)],
    )
    .await;
    assert_eq!(del.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(&w, &format!("/recipes/{recipe}")).await.status,
        StatusCode::NOT_FOUND
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn edit_session_is_a_soft_lock(pool: PgPool) {
    let w = world(&pool).await;
    let r = post(&w, "/recipes", ipa()).await;
    let recipe = r.body["id"].as_str().unwrap().to_owned();
    let path = format!("/recipes/{recipe}/edit-session");
    let none = get(&w, &path).await;
    assert_eq!(none.status, StatusCode::OK);
    assert_eq!(none.body, Value::Null);

    let a = post(&w, &path, json!({})).await;
    assert_eq!(a.status, StatusCode::OK, "{}", a.body);
    assert_eq!(a.body["user_id"], json!(w.admin));
    // The other user sees the holder and cannot start.
    let b = call_as(&w, w.other, "POST", &path, Some(json!({})), &[]).await;
    assert_eq!(b.status, StatusCode::CONFLICT, "{}", b.body);
    assert_eq!(b.body["current"]["user_id"], json!(w.admin));
    // Heartbeat of the holder works, of the other user does not.
    let renew = call_as(&w, w.admin, "PUT", &path, None, &[]).await;
    assert_eq!(renew.status, StatusCode::OK);
    let bad = call_as(&w, w.other, "PUT", &path, None, &[]).await;
    assert_eq!(bad.status, StatusCode::CONFLICT);
    // Take over on purpose.
    let t = call_as(
        &w,
        w.other,
        "POST",
        &path,
        Some(json!({"take_over": true})),
        &[],
    )
    .await;
    assert_eq!(t.status, StatusCode::OK, "{}", t.body);
    assert_eq!(t.body["user_id"], json!(w.other));
    // An expired lock is free.
    sqlx::query("UPDATE edit_sessions SET expires_at = now() - interval '1 second'")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(get(&w, &path).await.body, Value::Null);
    let again = post(&w, &path, json!({})).await;
    assert_eq!(again.status, StatusCode::OK);
    let rel = delete(&w, &path).await;
    assert_eq!(rel.status, StatusCode::NO_CONTENT);
    assert_eq!(delete(&w, &path).await.status, StatusCode::NO_CONTENT);
    // Each change went to the outbox (SSE reads it).
    let topics: Vec<String> = sqlx::query_scalar(
        "SELECT topic FROM outbox WHERE topic LIKE 'recipe.edit_session_%' ORDER BY id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        topics,
        vec![
            "recipe.edit_session_started",
            "recipe.edit_session_started",
            "recipe.edit_session_started",
            "recipe.edit_session_released"
        ]
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn permissions_and_isolation(pool: PgPool) {
    let w = world(&pool).await;
    let viewer = uuid::Uuid::now_v7();
    sqlx::query("INSERT INTO users (id, display_name) VALUES ($1, 'V')")
        .bind(viewer)
        .execute(&pool)
        .await
        .unwrap();
    w.auth.add(viewer, w.ws, &["recipe.view"]);
    let denied = call_as(
        &w,
        viewer,
        "POST",
        "/recipes",
        Some(ipa()),
        &[("idempotency-key", "k1")],
    )
    .await;
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
    let (recipe, version) = domain::published_recipe(&w, ipa()).await;
    let read = call_as(&w, viewer, "GET", &format!("/recipes/{recipe}"), None, &[]).await;
    assert_eq!(read.status, StatusCode::OK);
    let publish = call_as(
        &w,
        viewer,
        "POST",
        &format!("/recipe-versions/{version}/publish"),
        Some(json!({})),
        &[("idempotency-key", "k2"), ("if-match", "\"1\"")],
    )
    .await;
    assert_eq!(publish.status, StatusCode::FORBIDDEN);
    // A publish without a fermentable is refused.
    let empty = post(
        &w,
        "/recipes",
        json!({"name": "Empty", "version": {"batch_volume_l": 10.0, "efficiency_pct": 70.0}}),
    )
    .await;
    let v = empty.body["draft_version_id"].as_str().unwrap();
    let etag = get(&w, &format!("/recipe-versions/{v}"))
        .await
        .header("etag")
        .to_owned();
    let p = post_with(
        &w,
        &format!("/recipe-versions/{v}/publish"),
        json!({}),
        &[("if-match", &etag)],
    )
    .await;
    assert_eq!(p.status, StatusCode::UNPROCESSABLE_ENTITY);
}
