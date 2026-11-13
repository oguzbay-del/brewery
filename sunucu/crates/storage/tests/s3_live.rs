//! `S3Store` against the S3 compatible service of the Compose stack (`objectstore`, port 8333;
//! `S3_*` variables override, as in CI).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Duration;

use chrono::Utc;
use sha2::{Digest, Sha256};
use storage::{ObjectStore, PresignedRequest, S3Config, S3Store, StorageError, UploadLimits};

fn env(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_owned())
}

async fn store() -> S3Store {
    let endpoint = env(
        "S3_PUBLIC_ENDPOINT",
        &env("S3_ENDPOINT", "http://127.0.0.1:8333"),
    );
    let store = S3Store::new(S3Config {
        public_endpoint: endpoint.clone(),
        endpoint,
        bucket: env("S3_BUCKET", "demleme-files"),
        region: env("S3_REGION", "us-east-1"),
        access_key: env("S3_ACCESS_KEY", "demleme-yerel"),
        secret_key: env("S3_SECRET_KEY", "demleme-yerel-gizli"),
        path_style: true,
        create_bucket: true,
        presign_ttl: Duration::from_mins(1),
    })
    .unwrap();
    store.ensure_bucket().await.unwrap();
    store
}

fn key(name: &str) -> String {
    format!("tests/{}/{name}", uuid_like())
}

fn uuid_like() -> String {
    let nanos = Utc::now().timestamp_nanos_opt().unwrap();
    format!("{nanos:x}")
}

fn limits_for(body: &[u8], content_type: &str) -> UploadLimits {
    UploadLimits {
        content_type: content_type.to_owned(),
        size_bytes: body.len() as u64,
        sha256: Sha256::digest(body).into(),
    }
}

async fn upload(request: &PresignedRequest, body: Vec<u8>) -> reqwest::StatusCode {
    let mut builder = reqwest::Client::new().put(&request.url);
    for (k, v) in &request.headers {
        builder = builder.header(k.as_str(), v.as_str());
    }
    builder.body(body).send().await.unwrap().status()
}

#[tokio::test]
async fn presigned_upload_head_download_and_delete() {
    let store = store().await;
    let key = key("photo.png");
    let body = b"hello object store".to_vec();
    let put = store
        .presign_put(
            &key,
            &limits_for(&body, "image/png"),
            Duration::from_mins(1),
        )
        .unwrap();
    assert!(upload(&put, body.clone()).await.is_success());

    let meta = store.head(&key).await.unwrap().expect("object exists");
    assert_eq!(meta.size_bytes, body.len() as u64);
    assert_eq!(meta.content_type.as_deref(), Some("image/png"));

    let (size, sha) = store.digest(&key, 1024).await.unwrap().unwrap();
    assert_eq!(size, body.len() as u64);
    assert_eq!(sha, <[u8; 32]>::from(Sha256::digest(&body)));
    assert!(matches!(
        store.digest(&key, 3).await,
        Err(StorageError::TooLarge(3))
    ));

    let get = store.presign_get(&key, Duration::from_mins(1)).unwrap();
    let downloaded = reqwest::get(&get.url).await.unwrap();
    assert!(downloaded.status().is_success());
    assert_eq!(downloaded.bytes().await.unwrap().as_ref(), body.as_slice());

    store.delete(&key).await.unwrap();
    assert!(store.head(&key).await.unwrap().is_none());
    store.delete(&key).await.unwrap();
    assert!(store.digest(&key, 10).await.unwrap().is_none());
}

#[tokio::test]
async fn expired_links_are_refused() {
    let store = store().await;
    let key = key("expired.png");
    let body = b"late".to_vec();
    let long_ago = Utc::now() - chrono::Duration::minutes(10);
    let put = store
        .presign_put_at(
            &key,
            &limits_for(&body, "image/png"),
            Duration::from_mins(1),
            long_ago,
        )
        .unwrap();
    assert_eq!(upload(&put, body).await, reqwest::StatusCode::FORBIDDEN);
    assert!(store.head(&key).await.unwrap().is_none());

    store.put(&key, "image/png", b"x".to_vec()).await.unwrap();
    let get = store
        .presign_get_at(&key, Duration::from_mins(1), long_ago)
        .unwrap();
    let status = reqwest::get(&get.url).await.unwrap().status();
    assert_eq!(status, reqwest::StatusCode::FORBIDDEN);
    store.delete(&key).await.unwrap();
}

#[tokio::test]
async fn upload_must_match_the_signed_limits() {
    let store = store().await;
    let key = key("limits.png");
    let body = b"twelve bytes".to_vec();
    let put = store
        .presign_put(
            &key,
            &limits_for(&body, "image/png"),
            Duration::from_mins(1),
        )
        .unwrap();
    // Wrong size.
    assert!(
        !upload(&put, b"much longer body than signed".to_vec())
            .await
            .is_success()
    );
    // Wrong content type.
    let mut wrong_type = put.clone();
    for (k, v) in &mut wrong_type.headers {
        if k == "content-type" {
            "text/html".clone_into(v);
        }
    }
    assert!(!upload(&wrong_type, body.clone()).await.is_success());
    // Tampered URL (another key).
    let tampered = PresignedRequest {
        url: put.url.replace("limits.png", "other.png"),
        ..put.clone()
    };
    assert!(!upload(&tampered, body.clone()).await.is_success());
    assert!(store.head(&key).await.unwrap().is_none());
    // The right request works.
    assert!(upload(&put, body).await.is_success());
    store.delete(&key).await.unwrap();
}
