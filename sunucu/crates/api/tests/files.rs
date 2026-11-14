//! File endpoints end to end (T2.18): real PostgreSQL (`#[sqlx::test]`) and the S3 compatible
//! service of the Compose stack (`objectstore`, port 8333; `S3_*` variables override). Uploads
//! and downloads go straight to the object store with the presigned requests, as a client does.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::{sync::Arc, time::Duration};

use api::{ApiConfig, redis_client::Redis};
use axum::{Router, body::Body, http::StatusCode};
use db::{ActorKind, DbContext};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use storage::{ObjectStore, S3Config, S3Store};
use support::{FakeAuth, Reply, insert_user, insert_workspace, json_post, req, send, state_with};
use uuid::Uuid;

fn env(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn s3_config(ttl_secs: u64) -> S3Config {
    let endpoint = env(
        "S3_PUBLIC_ENDPOINT",
        &env("S3_ENDPOINT", "http://127.0.0.1:8333"),
    );
    S3Config::from_lookup(|n| match n {
        "S3_ENDPOINT" => Some(endpoint.clone()),
        "S3_BUCKET" => Some(env("S3_BUCKET", "demleme-files")),
        "S3_ACCESS_KEY" => Some(env("S3_ACCESS_KEY", "demleme-yerel")),
        "S3_SECRET_KEY" => Some(env("S3_SECRET_KEY", "demleme-yerel-gizli")),
        "S3_PRESIGN_TTL_SECS" => Some(ttl_secs.to_string()),
        _ => None,
    })
    .unwrap()
    .unwrap()
}

fn api_config(ttl_secs: u64) -> ApiConfig {
    let s3 = s3_config(ttl_secs);
    ApiConfig::from_lookup(|n| match n {
        "DATABASE_URL" => Some("postgres://unused".to_owned()),
        "TRUST_FORWARDED_FOR" => Some("true".to_owned()),
        "S3_ENDPOINT" => Some(s3.endpoint.clone()),
        "S3_BUCKET" => Some(s3.bucket.clone()),
        "S3_ACCESS_KEY" => Some(s3.access_key.clone()),
        "S3_SECRET_KEY" => Some(s3.secret_key.clone()),
        "S3_PRESIGN_TTL_SECS" => Some(ttl_secs.to_string()),
        _ => None,
    })
    .unwrap()
}

struct World {
    app: Router,
    store: Arc<S3Store>,
    ws: Uuid,
    batch: Uuid,
    note: Uuid,
    /// `batch.record`.
    writer: Uuid,
    /// `batch.record` and `workspace.manage`.
    manager: Uuid,
    /// Member without permissions.
    reader: Uuid,
    /// Not a member.
    outsider: Uuid,
}

async fn world(pool: &PgPool, ttl_secs: u64, with_store: bool) -> World {
    let ws = Uuid::now_v7();
    let (writer, manager, reader, outsider) = (
        Uuid::now_v7(),
        Uuid::now_v7(),
        Uuid::now_v7(),
        Uuid::now_v7(),
    );
    insert_workspace(pool, ws).await;
    for u in [writer, manager, reader, outsider] {
        insert_user(pool, u).await;
    }
    let auth = Arc::new(FakeAuth::default());
    auth.add(writer, ws, &["batch.record"]);
    auth.add(manager, ws, &["batch.record", "workspace.manage"]);
    auth.add(reader, ws, &[]);

    let store = Arc::new(S3Store::new(s3_config(ttl_secs)).unwrap());
    store.ensure_bucket().await.unwrap();
    let mut state = state_with(pool, Redis::disabled(), auth, api_config(ttl_secs)).await;
    if with_store {
        state = state.with_store(store.clone());
    }

    // A published recipe version, a batch and a tasting note (as the workspace's writer).
    let (batch, note) = (Uuid::now_v7(), Uuid::now_v7());
    let (recipe, version) = (Uuid::now_v7(), Uuid::now_v7());
    let mut tx = db::begin(
        &state.db,
        &DbContext {
            user_id: Some(manager),
            workspace_id: Some(ws),
            actor_kind: ActorKind::User,
        },
    )
    .await
    .unwrap();
    for sql in [
        format!(
            "INSERT INTO recipes (id, workspace_id, name) VALUES ('{recipe}', '{ws}', 'Pale Ale')"
        ),
        format!(
            "INSERT INTO recipe_versions (id, workspace_id, recipe_id, number, status, \
                 batch_volume_l, efficiency_pct, approved_by, approved_at) \
             VALUES ('{version}', '{ws}', '{recipe}', 1, 'approved', 20, 72, '{manager}', now())"
        ),
        format!(
            "INSERT INTO batches (id, workspace_id, code, recipe_version_id) \
             VALUES ('{batch}', '{ws}', 'B-001', '{version}')"
        ),
        format!(
            "INSERT INTO tasting_notes (id, workspace_id, batch_id) VALUES ('{note}', '{ws}', '{batch}')"
        ),
    ] {
        sqlx::query(sqlx::AssertSqlSafe(sql))
            .execute(&mut *tx)
            .await
            .unwrap();
    }
    tx.commit().await.unwrap();

    World {
        app: api::router(state.clone()),
        store,
        ws,
        batch,
        note,
        writer,
        manager,
        reader,
        outsider,
    }
}

fn sha_hex(bytes: &[u8]) -> String {
    storage::sigv4::hex(&Sha256::digest(bytes))
}

fn path(w: &World, tail: &str) -> String {
    format!("/v1/workspaces/{}/{tail}", w.ws)
}

async fn get(w: &World, user: Uuid, tail: &str) -> Reply {
    send(
        &w.app,
        req("GET", &path(w, tail), Some(user))
            .body(Body::empty())
            .unwrap(),
    )
    .await
}

async fn post_empty(w: &World, user: Uuid, tail: &str) -> Reply {
    send(
        &w.app,
        req("POST", &path(w, tail), Some(user))
            .body(Body::empty())
            .unwrap(),
    )
    .await
}

async fn delete(w: &World, user: Uuid, tail: &str) -> Reply {
    send(
        &w.app,
        req("DELETE", &path(w, tail), Some(user))
            .body(Body::empty())
            .unwrap(),
    )
    .await
}

fn new_attachment(w: &World, bytes: &[u8]) -> Value {
    json!({
        "kind": "photo", "record_kind": "batch", "record_id": w.batch,
        "content_type": "image/png", "size_bytes": bytes.len(), "sha256": sha_hex(bytes),
        "caption": "day 2"
    })
}

async fn create_attachment(w: &World, user: Uuid, body: &Value) -> Reply {
    send(
        &w.app,
        json_post(
            &path(w, "attachments"),
            user,
            Some(&Uuid::now_v7().to_string()),
            body,
        ),
    )
    .await
}

/// Sends the presigned request the way a client does.
async fn put_to_store(upload: &Value, bytes: Vec<u8>) -> StatusCode {
    let mut request = reqwest::Client::new().put(upload["url"].as_str().unwrap());
    for (k, v) in upload["headers"].as_object().unwrap() {
        request = request.header(k.as_str(), v.as_str().unwrap());
    }
    StatusCode::from_u16(request.body(bytes).send().await.unwrap().status().as_u16()).unwrap()
}

fn id_of(r: &Reply, key: &str) -> String {
    r.body[key]["id"].as_str().unwrap().to_owned()
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn upload_and_download_end_to_end(pool: PgPool) {
    let w = world(&pool, 600, true).await;
    let bytes = b"\x89PNG not really a png but bytes".to_vec();

    let created = create_attachment(&w, w.writer, &new_attachment(&w, &bytes)).await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    assert!(created.header("location").contains("/attachments/"));
    assert_eq!(created.body["attachment"]["uploaded_at"], Value::Null);
    let id = id_of(&created, "attachment");

    // Pending: visible by id, not in the list, no download link yet.
    assert_eq!(
        get(&w, w.reader, &format!("attachments/{id}")).await.status,
        StatusCode::OK
    );
    assert_eq!(
        get(&w, w.reader, "attachments").await.body["items"],
        json!([])
    );
    let early = get(&w, w.reader, &format!("attachments/{id}/download")).await;
    assert_eq!(early.status, StatusCode::CONFLICT);
    assert_eq!(early.body["errors"][0]["code"], "not_uploaded");
    let early = post_empty(&w, w.writer, &format!("attachments/{id}/complete")).await;
    assert_eq!(early.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(early.body["errors"][0]["code"], "not_uploaded");

    assert!(
        put_to_store(&created.body["upload"], bytes.clone())
            .await
            .is_success()
    );
    let done = post_empty(&w, w.writer, &format!("attachments/{id}/complete")).await;
    assert_eq!(done.status, StatusCode::OK, "{}", done.body);
    assert!(done.body["uploaded_at"].is_string());
    // Completing twice is harmless.
    let again = post_empty(&w, w.writer, &format!("attachments/{id}/complete")).await;
    assert_eq!(again.status, StatusCode::OK);
    assert_eq!(again.body["uploaded_at"], done.body["uploaded_at"]);

    let listed = get(
        &w,
        w.reader,
        &format!("attachments?record_kind=batch&record_id={}", w.batch),
    )
    .await;
    assert_eq!(listed.body["items"].as_array().unwrap().len(), 1);
    let other = get(&w, w.reader, &format!("attachments?record_id={}", w.note)).await;
    assert_eq!(other.body["items"], json!([]));

    let link = get(&w, w.reader, &format!("attachments/{id}/download")).await;
    assert_eq!(link.status, StatusCode::OK);
    assert_eq!(link.header("cache-control"), "no-store");
    let downloaded = reqwest::get(link.body["url"].as_str().unwrap())
        .await
        .unwrap();
    assert!(downloaded.status().is_success());
    assert_eq!(downloaded.bytes().await.unwrap().as_ref(), bytes.as_slice());

    // Delete: soft, then gone from reads and the list.
    assert_eq!(
        delete(&w, w.reader, &format!("attachments/{id}"))
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        delete(&w, w.writer, &format!("attachments/{id}"))
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        get(&w, w.reader, &format!("attachments/{id}")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&w, w.reader, "attachments").await.body["items"],
        json!([])
    );
    let trash = get(&w, w.reader, "attachments?include_deleted=true").await;
    assert_eq!(trash.body["items"].as_array().unwrap().len(), 1);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn expired_links_are_refused(pool: PgPool) {
    let w = world(&pool, 1, true).await;
    let bytes = b"short lived".to_vec();

    // Upload link expires before use.
    let created = create_attachment(&w, w.writer, &new_attachment(&w, &bytes)).await;
    assert_eq!(created.status, StatusCode::CREATED);
    tokio::time::sleep(Duration::from_millis(2500)).await;
    assert_eq!(
        put_to_store(&created.body["upload"], bytes.clone()).await,
        StatusCode::FORBIDDEN
    );
    let id = id_of(&created, "attachment");
    let late = post_empty(&w, w.writer, &format!("attachments/{id}/complete")).await;
    assert_eq!(late.body["errors"][0]["code"], "not_uploaded");

    // Download link expires before use.
    let created = create_attachment(&w, w.writer, &new_attachment(&w, &bytes)).await;
    let id = id_of(&created, "attachment");
    assert!(
        put_to_store(&created.body["upload"], bytes)
            .await
            .is_success()
    );
    assert_eq!(
        post_empty(&w, w.writer, &format!("attachments/{id}/complete"))
            .await
            .status,
        StatusCode::OK
    );
    let link = get(&w, w.reader, &format!("attachments/{id}/download")).await;
    assert_eq!(link.status, StatusCode::OK);
    tokio::time::sleep(Duration::from_millis(2500)).await;
    let status = reqwest::get(link.body["url"].as_str().unwrap())
        .await
        .unwrap()
        .status();
    assert_eq!(status.as_u16(), 403);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn complete_checks_the_stored_object(pool: PgPool) {
    let w = world(&pool, 600, true).await;
    let declared = b"declared content".to_vec();

    // Another object of another size under the key: refused and removed.
    let created = create_attachment(&w, w.writer, &new_attachment(&w, &declared)).await;
    let id = id_of(&created, "attachment");
    let key = format!("{}/attachments/{id}", w.ws);
    w.store
        .put(&key, "image/png", b"too long to match".to_vec())
        .await
        .unwrap();
    let r = post_empty(&w, w.writer, &format!("attachments/{id}/complete")).await;
    assert_eq!(
        (r.status, r.body["errors"][0]["code"].clone()),
        (StatusCode::UNPROCESSABLE_ENTITY, json!("size_mismatch"))
    );
    assert!(w.store.head(&key).await.unwrap().is_none());

    // Same size, other bytes: checksum mismatch.
    let created = create_attachment(&w, w.writer, &new_attachment(&w, &declared)).await;
    let id = id_of(&created, "attachment");
    let key = format!("{}/attachments/{id}", w.ws);
    w.store
        .put(&key, "image/png", b"DECLARED CONTENT".to_vec())
        .await
        .unwrap();
    let r = post_empty(&w, w.writer, &format!("attachments/{id}/complete")).await;
    assert_eq!(r.body["errors"][0]["code"], "checksum_mismatch");
    assert!(w.store.head(&key).await.unwrap().is_none());

    // Same bytes, other type.
    let created = create_attachment(&w, w.writer, &new_attachment(&w, &declared)).await;
    let id = id_of(&created, "attachment");
    let key = format!("{}/attachments/{id}", w.ws);
    w.store
        .put(&key, "text/html", declared.clone())
        .await
        .unwrap();
    let r = post_empty(&w, w.writer, &format!("attachments/{id}/complete")).await;
    assert_eq!(r.body["errors"][0]["code"], "content_type_mismatch");

    // The right object passes even when put by the server (no presigned request involved).
    let created = create_attachment(&w, w.writer, &new_attachment(&w, &declared)).await;
    let id = id_of(&created, "attachment");
    let key = format!("{}/attachments/{id}", w.ws);
    w.store.put(&key, "image/png", declared).await.unwrap();
    let r = post_empty(&w, w.writer, &format!("attachments/{id}/complete")).await;
    assert_eq!(r.status, StatusCode::OK);
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn requests_are_validated_and_authorized(pool: PgPool) {
    let w = world(&pool, 600, true).await;
    let bytes = b"abc".to_vec();
    let ok = new_attachment(&w, &bytes);

    // Size, type and checksum limits; every problem is listed.
    let mut bad = ok.clone();
    bad["content_type"] = json!("text/html");
    bad["size_bytes"] = json!(30 * 1024 * 1024);
    bad["sha256"] = json!("nothex");
    let r = create_attachment(&w, w.writer, &bad).await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    let fields: Vec<&str> = r.body["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["field"].as_str().unwrap())
        .collect();
    assert_eq!(fields, ["/content_type", "/size_bytes", "/sha256"]);
    let mut zero = ok.clone();
    zero["size_bytes"] = json!(0);
    assert_eq!(
        create_attachment(&w, w.writer, &zero).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );

    // Unknown record, unsupported record kind, bad kind.
    let mut unknown = ok.clone();
    unknown["record_id"] = json!(Uuid::now_v7());
    let r = create_attachment(&w, w.writer, &unknown).await;
    assert_eq!(r.body["errors"][0]["field"], "/record_id");
    let mut kind = ok.clone();
    kind["record_kind"] = json!("support_message");
    assert_eq!(
        create_attachment(&w, w.writer, &kind).await.body["errors"][0]["field"],
        "/record_kind"
    );
    kind = ok.clone();
    kind["kind"] = json!("video");
    assert_eq!(
        create_attachment(&w, w.writer, &kind).await.body["errors"][0]["field"],
        "/kind"
    );

    // Permissions and membership.
    assert_eq!(
        create_attachment(&w, w.reader, &ok).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        create_attachment(&w, w.outsider, &ok).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&w, w.outsider, "attachments").await.status,
        StatusCode::NOT_FOUND
    );
    let no_key = send(
        &w.app,
        json_post(&path(&w, "attachments"), w.writer, None, &ok),
    )
    .await;
    assert_eq!(no_key.status, StatusCode::PRECONDITION_REQUIRED);

    // A replay with the same key returns the same record.
    let key = Uuid::now_v7().to_string();
    let first = send(
        &w.app,
        json_post(&path(&w, "attachments"), w.writer, Some(&key), &ok),
    )
    .await;
    let second = send(
        &w.app,
        json_post(&path(&w, "attachments"), w.writer, Some(&key), &ok),
    )
    .await;
    assert_eq!(second.header("idempotent-replayed"), "true");
    assert_eq!(
        first.body["attachment"]["id"],
        second.body["attachment"]["id"]
    );

    // Files of a tasting note.
    let mut note = ok.clone();
    note["record_kind"] = json!("tasting_note");
    note["record_id"] = json!(w.note);
    assert_eq!(
        create_attachment(&w, w.writer, &note).await.status,
        StatusCode::CREATED
    );

    // Listing validates paging parameters like every list.
    assert_eq!(
        get(&w, w.reader, "attachments?limit=0").await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        get(&w, w.reader, "attachments?sort=caption").await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn lists_page_with_a_cursor(pool: PgPool) {
    let w = world(&pool, 600, true).await;
    let mut ids = Vec::new();
    for n in 0..3 {
        let bytes = format!("file {n}").into_bytes();
        let created = create_attachment(&w, w.writer, &new_attachment(&w, &bytes)).await;
        let id = id_of(&created, "attachment");
        assert!(
            put_to_store(&created.body["upload"], bytes)
                .await
                .is_success()
        );
        post_empty(&w, w.writer, &format!("attachments/{id}/complete")).await;
        ids.push(id);
    }
    let first = get(&w, w.reader, "attachments?limit=2").await;
    assert_eq!(first.body["items"].as_array().unwrap().len(), 2);
    let cursor = first.body["next_cursor"].as_str().unwrap().to_owned();
    let second = get(
        &w,
        w.reader,
        &format!("attachments?limit=2&cursor={cursor}"),
    )
    .await;
    assert_eq!(second.body["items"].as_array().unwrap().len(), 1);
    assert_eq!(second.body["next_cursor"], Value::Null);
    // Newest first: the last created is the first listed.
    assert_eq!(first.body["items"][0]["id"], json!(ids[2]));
    assert_eq!(second.body["items"][0]["id"], json!(ids[0]));
    let asc = get(&w, w.reader, "attachments?sort=created_at").await;
    assert_eq!(asc.body["items"][0]["id"], json!(ids[0]));
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn scanned_documents_and_legal_hold(pool: PgPool) {
    let w = world(&pool, 600, true).await;
    let bytes = b"%PDF-1.4 scanned brew log".to_vec();
    let body = |hold: bool, until: Option<&str>| {
        json!({
            "batch_id": w.batch, "form_type": "brew_log", "content_type": "application/pdf",
            "size_bytes": bytes.len(), "sha256": sha_hex(&bytes),
            "legal_hold": hold, "retention_until": until
        })
    };
    let create = |user: Uuid, b: Value| {
        let app = w.app.clone();
        let p = path(&w, "scanned-documents");
        async move {
            send(
                &app,
                json_post(&p, user, Some(&Uuid::now_v7().to_string()), &b),
            )
            .await
        }
    };

    // Hold and retention need `workspace.manage`.
    assert_eq!(
        create(w.writer, body(true, None)).await.status,
        StatusCode::FORBIDDEN
    );
    let mut bad = body(false, None);
    bad["form_type"] = json!("Brew Log");
    assert_eq!(
        create(w.writer, bad).await.body["errors"][0]["field"],
        "/form_type"
    );
    let mut text = body(false, None);
    text["content_type"] = json!("text/plain");
    assert_eq!(
        create(w.writer, text).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let mut nobatch = body(false, None);
    nobatch["batch_id"] = json!(Uuid::now_v7());
    assert_eq!(
        create(w.writer, nobatch).await.body["errors"][0]["field"],
        "/batch_id"
    );

    // A plain document: full cycle.
    let plain = create(w.writer, body(false, None)).await;
    assert_eq!(plain.status, StatusCode::CREATED, "{}", plain.body);
    let plain_id = id_of(&plain, "document");
    assert!(
        put_to_store(&plain.body["upload"], bytes.clone())
            .await
            .is_success()
    );
    let done = post_empty(
        &w,
        w.writer,
        &format!("scanned-documents/{plain_id}/complete"),
    )
    .await;
    assert_eq!(done.status, StatusCode::OK, "{}", done.body);
    let link = get(
        &w,
        w.reader,
        &format!("scanned-documents/{plain_id}/download"),
    )
    .await;
    let downloaded = reqwest::get(link.body["url"].as_str().unwrap())
        .await
        .unwrap();
    assert_eq!(downloaded.bytes().await.unwrap().as_ref(), bytes.as_slice());
    let by_batch = get(
        &w,
        w.reader,
        &format!("scanned-documents?batch_id={}&form_type=brew_log", w.batch),
    )
    .await;
    assert_eq!(by_batch.body["items"].as_array().unwrap().len(), 1);
    assert_eq!(
        delete(&w, w.writer, &format!("scanned-documents/{plain_id}"))
            .await
            .status,
        StatusCode::NO_CONTENT
    );

    // Under legal hold: cannot be deleted.
    let held = create(w.manager, body(true, None)).await;
    assert_eq!(held.status, StatusCode::CREATED, "{}", held.body);
    let held_id = id_of(&held, "document");
    assert!(
        put_to_store(&held.body["upload"], bytes.clone())
            .await
            .is_success()
    );
    post_empty(
        &w,
        w.manager,
        &format!("scanned-documents/{held_id}/complete"),
    )
    .await;
    let refused = delete(&w, w.manager, &format!("scanned-documents/{held_id}")).await;
    assert_eq!(refused.status, StatusCode::CONFLICT);
    assert_eq!(refused.body["errors"][0]["code"], "legal_hold");
    assert_eq!(
        get(&w, w.reader, &format!("scanned-documents/{held_id}"))
            .await
            .status,
        StatusCode::OK
    );

    // Hold with a retention end in the past no longer protects.
    let expired = create(w.manager, body(true, Some("2020-01-01T00:00:00Z"))).await;
    let expired_id = id_of(&expired, "document");
    assert_eq!(
        delete(&w, w.manager, &format!("scanned-documents/{expired_id}"))
            .await
            .status,
        StatusCode::NO_CONTENT
    );
}

#[sqlx::test(migrator = "db::MIGRATOR")]
async fn without_an_object_store_the_endpoints_answer_503(pool: PgPool) {
    let w = world(&pool, 600, false).await;
    let bytes = b"abc".to_vec();
    let r = create_attachment(&w, w.writer, &new_attachment(&w, &bytes)).await;
    assert_eq!(r.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(r.body["code"], "unavailable");
}
