//! The HTTP client against a fake Apple server in the test process.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::{Arc, Mutex};

use account::{AppleClient, AppleError, HttpApple};
use axum::{Router, extract::State, http::StatusCode, routing::post};
use base64::{Engine, engine::general_purpose::STANDARD};
use ring::{
    rand::SystemRandom,
    signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair},
};

type Seen = Arc<Mutex<Vec<(String, String)>>>;

async fn token(State(seen): State<Seen>, body: String) -> (StatusCode, String) {
    seen.lock().unwrap().push(("token".into(), body));
    (
        StatusCode::OK,
        r#"{"refresh_token":"r-1","access_token":"a"}"#.into(),
    )
}

async fn revoke(State(seen): State<Seen>, body: String) -> (StatusCode, String) {
    let unknown = body.contains("token=gone");
    seen.lock().unwrap().push(("revoke".into(), body));
    if unknown {
        (
            StatusCode::BAD_REQUEST,
            r#"{"error":"invalid_grant"}"#.into(),
        )
    } else if seen.lock().unwrap().len() > 2 {
        (StatusCode::BAD_GATEWAY, String::new())
    } else {
        (StatusCode::OK, String::new())
    }
}

async fn client() -> (HttpApple, Seen) {
    let seen: Seen = Arc::default();
    let app = Router::new()
        .route("/auth/token", post(token))
        .route("/auth/revoke", post(revoke))
        .with_state(seen.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let der = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &SystemRandom::new())
        .unwrap();
    let pem = format!(
        "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----\n",
        STANDARD.encode(der.as_ref())
    );
    let apple = HttpApple::new(
        "TEAM123456",
        "KEY1234567",
        "com.example.brewery",
        &pem,
        &format!("{base}/auth/token"),
        &format!("{base}/auth/revoke"),
    )
    .unwrap();
    (apple, seen)
}

#[tokio::test]
async fn code_exchange_and_revoke_follow_the_apple_protocol() {
    let (apple, seen) = client().await;
    assert_eq!(apple.exchange_code("c o/de").await.unwrap(), "r-1");
    apple.revoke("tok").await.unwrap();
    // A token Apple does not know counts as revoked.
    apple.revoke("gone").await.unwrap();
    // A server error is a retryable failure.
    let err = apple.revoke("again").await.unwrap_err();
    assert!(matches!(err, AppleError::Unavailable(_)), "{err:?}");

    let seen = seen.lock().unwrap();
    let (_, exchange) = &seen[0];
    assert!(exchange.contains("grant_type=authorization_code"));
    assert!(exchange.contains("code=c%20o%2Fde"));
    assert!(exchange.contains("client_id=com.example.brewery"));
    // The client secret is a three part ES256 token.
    let secret = exchange
        .split('&')
        .find_map(|p| p.strip_prefix("client_secret="))
        .unwrap();
    assert_eq!(secret.split('.').count(), 3);
    assert!(seen[1].1.contains("token=tok"));
    assert!(seen[1].1.contains("token_type_hint=refresh_token"));
}
