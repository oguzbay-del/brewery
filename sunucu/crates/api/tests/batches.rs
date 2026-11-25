//! Endpoint tests of T2.22: batch from a recipe, steps, actual amounts, measurements (safe to
//! repeat), stage transitions, summary, calendar, and the signed brew day package. Needs
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
use chrono::Utc;
use domain::{World, call_as, get, ipa, patch, post, post_with, published_recipe, world};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

static PACKAGE_KEY_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn key() {
    api::brew_day::set_signing_key(b"test-signing-key-of-the-brew-day-package", "k-test");
}

async fn start_batch(w: &World, version: &str, extra: Value) -> Value {
    let mut body = json!({"recipe_version_id": version});
    if let (Some(b), Some(e)) = (body.as_object_mut(), extra.as_object()) {
        b.extend(e.clone());
    }
    let r = post(w, "/batches", body).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    r.body
}

async fn make_package(w: &World, batch: &str) -> Value {
    let r = post(w, &format!("/batches/{batch}/brew-day-package"), json!({})).await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    r.body
}

fn measurement(id: Uuid, value: f64, package: Option<&str>) -> Value {
    json!({"items": [{"id": id, "kind": "sg", "value": value,
        "measured_at": Utc::now().to_rfc3339(), "package_id": package}]})
}

async fn put_actual(w: &World, id: &str, line: &str, amount: f64) -> support::Reply {
    let path = format!("/batches/{id}/actuals/fermentable/{line}");
    call_as(
        w,
        w.admin,
        "PUT",
        &path,
        Some(json!({"actual_amount_kg": amount})),
        &[],
    )
    .await
}

async fn post_m(
    w: &World,
    id: &str,
    kind: &str,
    value: f64,
    at: chrono::DateTime<Utc>,
) -> support::Reply {
    let path = format!("/batches/{id}/measurements");
    let body = json!({"items": [
        {"id": Uuid::now_v7(), "kind": kind, "value": value, "measured_at": at.to_rfc3339()}]});
    post(w, &path, body).await
}

async fn count_measurements(pool: &PgPool, id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM measurements WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn package_verification_uses_previous_key_and_new_packages_use_current_key(pool: PgPool) {
    let _key_guard = PACKAGE_KEY_TEST_LOCK.lock().await;
    const OLD_KEY: &[u8] = b"old-brew-package-signing-key";
    const NEW_KEY: &[u8] = b"new-brew-package-signing-key";

    api::brew_day::set_signing_key(OLD_KEY, "k1");
    api::brew_day::set_previous_keys(&[]);
    let w = world(&pool).await;
    let (_, version) = published_recipe(&w, ipa()).await;
    let batch = start_batch(&w, &version, json!({})).await;
    let batch_id = batch["id"].as_str().unwrap();
    let old_package = make_package(&w, batch_id).await;
    let old_package_id = old_package["id"].as_str().unwrap();
    assert_eq!(old_package["key_id"], "k1");

    api::brew_day::set_signing_key(NEW_KEY, "k2");
    api::brew_day::set_previous_keys(&[("k1", OLD_KEY)]);

    let accepted = post(
        &w,
        &format!("/batches/{batch_id}/measurements"),
        measurement(Uuid::now_v7(), 1.05, Some(old_package_id)),
    )
    .await;
    assert_eq!(accepted.status, StatusCode::OK, "{}", accepted.body);

    api::brew_day::set_previous_keys(&[]);
    let missing_previous = post(
        &w,
        &format!("/batches/{batch_id}/measurements"),
        measurement(Uuid::now_v7(), 1.05, Some(old_package_id)),
    )
    .await;
    assert_eq!(
        missing_previous.body["errors"][0]["code"],
        "package_signature_invalid"
    );

    api::brew_day::set_previous_keys(&[("k1", OLD_KEY)]);
    sqlx::query("UPDATE brew_day_packages SET key_id = 'k2' WHERE id = $1")
        .bind(Uuid::parse_str(old_package_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    let changed_id = post(
        &w,
        &format!("/batches/{batch_id}/measurements"),
        measurement(Uuid::now_v7(), 1.05, Some(old_package_id)),
    )
    .await;
    assert_eq!(
        changed_id.body["errors"][0]["code"],
        "package_signature_invalid"
    );

    let new_package = make_package(&w, batch_id).await;
    assert_eq!(new_package["key_id"], "k2");
    let new_package_id = new_package["id"].as_str().unwrap();
    let accepted_new = post(
        &w,
        &format!("/batches/{batch_id}/measurements"),
        measurement(Uuid::now_v7(), 1.05, Some(new_package_id)),
    )
    .await;
    assert_eq!(accepted_new.status, StatusCode::OK, "{}", accepted_new.body);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn batch_starts_only_from_an_approved_version(pool: PgPool) {
    let w = world(&pool).await;
    let draft = post(&w, "/recipes", ipa()).await;
    let draft_version = draft.body["draft_version_id"].as_str().unwrap();
    let denied = post(&w, "/batches", json!({"recipe_version_id": draft_version})).await;
    assert_eq!(denied.status, StatusCode::CONFLICT, "{}", denied.body);
    let missing = post(&w, "/batches", json!({"recipe_version_id": Uuid::now_v7()})).await;
    assert_eq!(missing.status, StatusCode::UNPROCESSABLE_ENTITY);

    let (recipe, version) = published_recipe(&w, ipa()).await;
    let b = start_batch(&w, &version, json!({"planned_date": "2027-01-10"})).await;
    assert_eq!(b["code"], "B-0001");
    assert_eq!(b["stage"], "planned");
    assert_eq!(b["name"], "House IPA");
    assert_eq!(b["recipe_id"], json!(recipe));
    assert_eq!(b["target_volume_l"], 20.0);
    let second = start_batch(&w, &version, json!({})).await;
    assert_eq!(second["code"], "B-0002");

    let id = b["id"].as_str().unwrap();
    let steps = get(&w, &format!("/batches/{id}/steps")).await;
    let kinds: Vec<&str> = steps.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        ["prepare", "boil", "cool", "pitch", "ferment", "package"]
    );
    assert!(steps.body["items"][0]["planned_start_at"].is_string());
    let list = get(&w, &format!("/batches?recipe_id={recipe}")).await;
    assert_eq!(list.body["items"].as_array().unwrap().len(), 2);
    assert_eq!(list.body["items"][0]["code"], "B-0002");

    // The backward plan: ready on 2027-02-07 with 14 + 14 days.
    let plan = post(
        &w,
        "/calendar/backward-plan",
        json!({"recipe_version_id": version, "ready_date": "2027-02-07"}),
    )
    .await;
    assert_eq!(plan.status, StatusCode::OK, "{}", plan.body);
    assert_eq!(plan.body["brew_date"], "2027-01-10");
    let by_ready = start_batch(&w, &version, json!({"ready_date": "2027-02-07"})).await;
    assert_eq!(by_ready["planned_date"], "2027-01-10");
    let cal = get(&w, "/calendar?from=2027-01-01&to=2027-01-31").await;
    assert_eq!(cal.status, StatusCode::OK, "{}", cal.body);
    assert_eq!(cal.body["items"].as_array().unwrap().len(), 2);
    assert_eq!(cal.body["items"][0]["steps"].as_array().unwrap().len(), 6);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn same_measurement_twice_gives_one_record(pool: PgPool) {
    let _key_guard = PACKAGE_KEY_TEST_LOCK.lock().await;
    key();
    let w = world(&pool).await;
    let (_, version) = published_recipe(&w, ipa()).await;
    let b = start_batch(&w, &version, json!({})).await;
    let id = b["id"].as_str().unwrap();
    let package = make_package(&w, id).await;
    let pid = package["id"].as_str().unwrap();

    let mid = Uuid::now_v7();
    let body = measurement(mid, 1.052, Some(pid));
    let first = post(&w, &format!("/batches/{id}/measurements"), body.clone()).await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.body);
    assert_eq!(first.body["created"], 1);
    assert_eq!(first.body["items"][0]["status"], "created");
    let again = post(&w, &format!("/batches/{id}/measurements"), body).await;
    assert_eq!(again.status, StatusCode::OK, "{}", again.body);
    assert_eq!(again.body["created"], 0);
    assert_eq!(again.body["duplicates"], 1);
    assert_eq!(again.body["items"][0]["status"], "duplicate");
    assert_eq!(count_measurements(&pool, mid).await, 1);
    // The same ID with another time does not make a second record either.
    let moved = json!({"items": [{"id": mid, "kind": "sg", "value": 1.052,
        "measured_at": (Utc::now() - chrono::Duration::days(40)).to_rfc3339()}]});
    let third = post(&w, &format!("/batches/{id}/measurements"), moved).await;
    assert_eq!(third.status, StatusCode::OK, "{}", third.body);
    assert_eq!(count_measurements(&pool, mid).await, 1);

    let list = get(&w, &format!("/batches/{id}/measurements?kind=sg")).await;
    assert_eq!(list.body["items"].as_array().unwrap().len(), 1);
    assert_eq!(list.body["items"][0]["package_id"], json!(pid));
    // One outbox event for the one stored measurement.
    let events: i64 =
        sqlx::query_scalar("SELECT count(*) FROM outbox WHERE topic = 'batch.measurement_added'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(events, 1);

    // A clock that was reset and a time far ahead are refused (db::validate_measured_at).
    let old = json!({"items": [{"id": Uuid::now_v7(), "kind": "sg", "value": 1.0,
        "measured_at": "1970-01-01T00:00:00Z"}]});
    let r = post(&w, &format!("/batches/{id}/measurements"), old).await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(r.body["errors"][0]["field"], "/items/0/measured_at");
    assert_eq!(r.body["errors"][0]["code"], "too_old");
    let future = json!({"items": [{"id": Uuid::now_v7(), "kind": "sg", "value": 1.0,
        "measured_at": (Utc::now() + chrono::Duration::days(3)).to_rfc3339()}]});
    let r = post(&w, &format!("/batches/{id}/measurements"), future).await;
    assert_eq!(r.body["errors"][0]["code"], "in_future");
    let bad_kind = json!({"items": [{"id": Uuid::now_v7(), "kind": "battery_v", "value": 3.0,
        "measured_at": Utc::now().to_rfc3339()}]});
    assert_eq!(
        post(&w, &format!("/batches/{id}/measurements"), bad_kind)
            .await
            .status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn package_of_another_batch_expired_or_changed_is_rejected(pool: PgPool) {
    let _key_guard = PACKAGE_KEY_TEST_LOCK.lock().await;
    key();
    let w = world(&pool).await;
    let (_, version) = published_recipe(&w, ipa()).await;
    let a = start_batch(&w, &version, json!({})).await;
    let b = start_batch(&w, &version, json!({})).await;
    let (a_id, b_id) = (a["id"].as_str().unwrap(), b["id"].as_str().unwrap());
    let pkg_a = make_package(&w, a_id).await;
    let pkg_b = make_package(&w, b_id).await;
    let (pa, pb) = (pkg_a["id"].as_str().unwrap(), pkg_b["id"].as_str().unwrap());
    assert_eq!(pkg_a["key_id"], "k-test");
    assert!(pkg_a["signature"].as_str().unwrap().len() > 20);
    assert_eq!(pkg_a["content"]["batch"]["code"], "B-0001");
    assert!(pkg_a["content"]["targets"]["og_sg"].as_f64().unwrap() > 1.04);
    let boil = pkg_a["content"]["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["kind"] == "boil")
        .unwrap();
    assert_eq!(boil["timers"].as_array().unwrap().len(), 2);
    assert_eq!(boil["timers"][1]["at_min"], 55.0);
    let latest = get(&w, &format!("/batches/{a_id}/brew-day-package")).await;
    assert_eq!(latest.body["id"], json!(pa));

    // Package of a different batch.
    let mismatch = post(
        &w,
        &format!("/batches/{a_id}/measurements"),
        measurement(Uuid::now_v7(), 1.05, Some(pb)),
    )
    .await;
    assert_eq!(
        mismatch.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{}",
        mismatch.body
    );
    assert_eq!(mismatch.body["errors"][0]["code"], "package_batch_mismatch");
    assert_eq!(mismatch.body["errors"][0]["field"], "/items/0/package_id");
    // An unknown package.
    let unknown = post(
        &w,
        &format!("/batches/{a_id}/measurements"),
        measurement(Uuid::now_v7(), 1.05, Some(&Uuid::now_v7().to_string())),
    )
    .await;
    assert_eq!(unknown.body["errors"][0]["code"], "package_not_found");
    let rejected: i64 = sqlx::query_scalar("SELECT count(*) FROM measurements")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rejected, 0);

    // A changed package is not accepted: the signature does not match.
    sqlx::query("UPDATE brew_day_packages SET content = jsonb_set(content, '{targets,og_sg}', '1.2') WHERE id = $1")
        .bind(Uuid::parse_str(pb).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    let tampered = post(
        &w,
        &format!("/batches/{b_id}/measurements"),
        measurement(Uuid::now_v7(), 1.05, Some(pb)),
    )
    .await;
    assert_eq!(
        tampered.body["errors"][0]["code"],
        "package_signature_invalid"
    );

    // Expired package.
    sqlx::query(
        "UPDATE brew_day_packages SET expires_at = now() - interval '1 minute' WHERE id = $1",
    )
    .bind(Uuid::parse_str(pa).unwrap())
    .execute(&pool)
    .await
    .unwrap();
    let expired = post(
        &w,
        &format!("/batches/{a_id}/measurements"),
        measurement(Uuid::now_v7(), 1.05, Some(pa)),
    )
    .await;
    assert_eq!(
        expired.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{}",
        expired.body
    );
    assert_eq!(expired.body["errors"][0]["code"], "package_expired");

    // A new package works; a revoked one does not.
    let fresh = make_package(&w, a_id).await;
    assert_eq!(fresh["content_version"], 2);
    let fid = fresh["id"].as_str().unwrap();
    let ok = post(
        &w,
        &format!("/batches/{a_id}/measurements"),
        measurement(Uuid::now_v7(), 1.05, Some(fid)),
    )
    .await;
    assert_eq!(ok.status, StatusCode::OK, "{}", ok.body);
    let revoke = post(&w, &format!("/brew-day-packages/{fid}/revoke"), json!({})).await;
    assert_eq!(revoke.status, StatusCode::OK, "{}", revoke.body);
    assert!(revoke.body["revoked_at"].is_string());
    let revoked = post(
        &w,
        &format!("/batches/{a_id}/measurements"),
        measurement(Uuid::now_v7(), 1.05, Some(fid)),
    )
    .await;
    assert_eq!(revoked.body["errors"][0]["code"], "package_revoked");
    // No package at all is fine.
    let none = post(
        &w,
        &format!("/batches/{a_id}/measurements"),
        measurement(Uuid::now_v7(), 1.05, None),
    )
    .await;
    assert_eq!(none.status, StatusCode::OK);
    let long = post(
        &w,
        &format!("/batches/{a_id}/brew-day-package"),
        json!({"valid_hours": 1000}),
    )
    .await;
    assert_eq!(long.status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn stages_steps_actuals_and_summary(pool: PgPool) {
    let _key_guard = PACKAGE_KEY_TEST_LOCK.lock().await;
    key();
    let w = world(&pool).await;
    let (_, version) = published_recipe(&w, ipa()).await;
    let v = get(&w, &format!("/recipe-versions/{version}")).await.body;
    let malt = v["fermentables"][0]["id"].as_str().unwrap().to_owned();
    let b = start_batch(&w, &version, json!({"planned_date": "2027-03-01"})).await;
    let id = b["id"].as_str().unwrap();

    // Actual amounts: a deviation above 5 percent opens a deviation record.
    let a = put_actual(&w, id, &malt, 4.95).await;
    assert_eq!(a.status, StatusCode::OK, "{}", a.body);
    assert_eq!(a.body["planned_amount_kg"], 4.5);
    assert!((a.body["deviation_kg"].as_f64().unwrap() - 0.45).abs() < 1e-9);
    assert!((a.body["deviation_pct"].as_f64().unwrap() - 10.0).abs() < 1e-9);
    let open: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM deviations WHERE kind = 'ingredient_amount' AND status = 'open'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(open, 1);
    let a = put_actual(&w, id, &malt, 4.6).await;
    assert!((a.body["deviation_pct"].as_f64().unwrap() - 2.2222).abs() < 1e-3);
    let open: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM deviations WHERE kind = 'ingredient_amount' AND status = 'open'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(open, 0);
    let list = get(&w, &format!("/batches/{id}/actuals")).await;
    assert_eq!(list.body["items"].as_array().unwrap().len(), 1);
    let stranger = call_as(
        &w,
        w.admin,
        "PUT",
        &format!("/batches/{id}/actuals/hop/{malt}"),
        Some(json!({"actual_amount_kg": 0.1})),
        &[],
    )
    .await;
    assert_eq!(
        stranger.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{}",
        stranger.body
    );

    // Steps.
    let steps = get(&w, &format!("/batches/{id}/steps")).await;
    let step = steps.body["items"][0]["id"].as_str().unwrap();
    let s = call_as(
        &w,
        w.admin,
        "PATCH",
        &format!("/batches/{id}/steps/{step}"),
        Some(json!({"status": "running"})),
        &[],
    )
    .await;
    assert_eq!(s.status, StatusCode::OK, "{}", s.body);
    assert_eq!(s.body["status"], "running");
    assert!(s.body["actual_start_at"].is_string());
    let s = call_as(
        &w,
        w.admin,
        "PATCH",
        &format!("/batches/{id}/steps/{step}"),
        Some(json!({"status": "done", "input_source": "timer"})),
        &[],
    )
    .await;
    assert_eq!(s.body["status"], "done");
    assert!(s.body["actual_end_at"].is_string());

    // Stage transitions need If-Match and follow the order.
    let etag = get(&w, &format!("/batches/{id}"))
        .await
        .header("etag")
        .to_owned();
    let no_header = post(
        &w,
        &format!("/batches/{id}/transitions"),
        json!({"to_stage": "brewing"}),
    )
    .await;
    assert_eq!(no_header.status, StatusCode::PRECONDITION_REQUIRED);
    let skip = post_with(
        &w,
        &format!("/batches/{id}/transitions"),
        json!({"to_stage": "fermenting"}),
        &[("if-match", &etag)],
    )
    .await;
    assert_eq!(skip.status, StatusCode::CONFLICT, "{}", skip.body);
    assert_eq!(skip.body["errors"][0]["code"], "invalid_transition");
    let t = post_with(
        &w,
        &format!("/batches/{id}/transitions"),
        json!({"to_stage": "brewing"}),
        &[("if-match", &etag)],
    )
    .await;
    assert_eq!(t.status, StatusCode::OK, "{}", t.body);
    assert_eq!(t.body["stage"], "brewing");
    assert!(t.body["started_at"].is_string());
    let stale = post_with(
        &w,
        &format!("/batches/{id}/transitions"),
        json!({"to_stage": "fermenting"}),
        &[("if-match", &etag)],
    )
    .await;
    assert_eq!(stale.status, StatusCode::PRECONDITION_FAILED);
    let mut etag = t.header("etag").to_owned();
    for stage in ["fermenting", "conditioning", "packaging"] {
        let t = post_with(
            &w,
            &format!("/batches/{id}/transitions"),
            json!({"to_stage": stage}),
            &[("if-match", &etag)],
        )
        .await;
        assert_eq!(t.status, StatusCode::OK, "{}", t.body);
        etag = t.header("etag").to_owned();
    }

    // Measurements and the summary: plan against actual.
    let now = Utc::now();
    assert_eq!(
        post_m(&w, id, "sg", 1.060, now - chrono::Duration::days(10))
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        post_m(&w, id, "sg", 1.011, now - chrono::Duration::days(1))
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        post_m(&w, id, "volume_l", 19.5, now).await.status,
        StatusCode::OK
    );
    let sum = get(&w, &format!("/batches/{id}/summary")).await;
    assert_eq!(sum.status, StatusCode::OK, "{}", sum.body);
    let plan_og = sum.body["plan"]["og_sg"].as_f64().unwrap();
    assert!(plan_og > 1.04);
    assert_eq!(sum.body["actual"]["og_sg"], 1.06);
    assert_eq!(sum.body["actual"]["fg_sg"], 1.011);
    assert!((sum.body["difference"]["og_sg"].as_f64().unwrap() - (1.06 - plan_og)).abs() < 1e-9);
    assert_eq!(sum.body["actual"]["volume_l"], 19.5);
    assert_eq!(sum.body["difference"]["volume_l"], -0.5);
    assert_eq!(sum.body["ingredients"].as_array().unwrap().len(), 1);
    assert_eq!(sum.body["steps"]["done"], 1);
    assert_eq!(sum.body["stage_events"].as_array().unwrap().len(), 5);

    // Close the batch; a closed batch refuses changes.
    let t = post_with(
        &w,
        &format!("/batches/{id}/transitions"),
        json!({"to_stage": "completed"}),
        &[("if-match", &etag)],
    )
    .await;
    assert_eq!(t.status, StatusCode::OK, "{}", t.body);
    assert!(t.body["completed_at"].is_string());
    let etag = t.header("etag").to_owned();
    let closed = patch(
        &w,
        &format!("/batches/{id}"),
        json!({"notes": "late"}),
        &etag,
    )
    .await;
    assert_eq!(closed.status, StatusCode::CONFLICT);
    let again = post_with(
        &w,
        &format!("/batches/{id}/transitions"),
        json!({"to_stage": "cancelled"}),
        &[("if-match", &etag)],
    )
    .await;
    assert_eq!(again.status, StatusCode::CONFLICT);
    assert_eq!(
        put_actual(&w, id, &malt, 5.0).await.status,
        StatusCode::CONFLICT
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn batch_update_and_permissions(pool: PgPool) {
    let _key_guard = PACKAGE_KEY_TEST_LOCK.lock().await;
    key();
    let w = world(&pool).await;
    let (_, version) = published_recipe(&w, ipa()).await;
    let b = start_batch(&w, &version, json!({})).await;
    let id = b["id"].as_str().unwrap();
    let etag = get(&w, &format!("/batches/{id}"))
        .await
        .header("etag")
        .to_owned();
    let up = patch(
        &w,
        &format!("/batches/{id}"),
        json!({"name": "Renamed", "planned_date": "2027-05-01"}),
        &etag,
    )
    .await;
    assert_eq!(up.status, StatusCode::OK, "{}", up.body);
    assert_eq!(up.body["name"], "Renamed");
    let stale = patch(
        &w,
        &format!("/batches/{id}"),
        json!({"name": "Other"}),
        &etag,
    )
    .await;
    assert_eq!(stale.status, StatusCode::PRECONDITION_FAILED);
    assert_eq!(stale.body["current"]["name"], "Renamed");

    // A user with only recipe.view reads but does not record or plan.
    let viewer = Uuid::now_v7();
    sqlx::query("INSERT INTO users (id, display_name) VALUES ($1, 'V')")
        .bind(viewer)
        .execute(&pool)
        .await
        .unwrap();
    w.auth.add(viewer, w.ws, &["recipe.view"]);
    assert_eq!(
        call_as(&w, viewer, "GET", &format!("/batches/{id}"), None, &[])
            .await
            .status,
        StatusCode::OK
    );
    let m = call_as(
        &w,
        viewer,
        "POST",
        &format!("/batches/{id}/measurements"),
        Some(measurement(Uuid::now_v7(), 1.0, None)),
        &[],
    )
    .await;
    assert_eq!(m.status, StatusCode::FORBIDDEN);
    let p = call_as(
        &w,
        viewer,
        "POST",
        &format!("/batches/{id}/brew-day-package"),
        Some(json!({})),
        &[("idempotency-key", "k9")],
    )
    .await;
    assert_eq!(p.status, StatusCode::FORBIDDEN);

    let del = call_as(
        &w,
        w.admin,
        "DELETE",
        &format!("/batches/{id}"),
        None,
        &[("if-match", up.header("etag"))],
    )
    .await;
    assert_eq!(del.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(&w, &format!("/batches/{id}")).await.status,
        StatusCode::NOT_FOUND
    );
}
