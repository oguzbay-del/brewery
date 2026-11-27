//! `StoreKit` transaction sync (B3): a signed transaction becomes a subscription update.
//! The JWS verifier moved to `billing`; the test PKI is here. Needs no database.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::collections::HashMap;

use billing::{
    PlanKey, UpdateKind,
    apple_sync::{AppleSyncSettings, SyncError, parse_transaction},
    jws::JwsVerifier,
};
use chrono::Utc;
use serde_json::{Value, json};
use support::Pki;
use uuid::Uuid;

const DAY_MS: i64 = 86_400_000;

fn settings(pki: &Pki) -> AppleSyncSettings {
    AppleSyncSettings {
        verifier: JwsVerifier::with_roots(vec![pki.root_der.clone()], true),
        bundle_id: "app.demleme".to_owned(),
        environment: "Production".to_owned(),
        product_ids: vec!["home.monthly".to_owned()],
        plans: HashMap::from([("home.monthly".to_owned(), PlanKey::Home)]),
    }
}

fn transaction(ws: Uuid, expires_in_days: i64) -> Value {
    let now_ms = Utc::now().timestamp_millis();
    json!({
        "bundleId": "app.demleme",
        "environment": "Production",
        "productId": "home.monthly",
        "originalTransactionId": "1000000000001",
        "transactionId": "2000000000007",
        "appAccountToken": ws.to_string(),
        "signedDate": now_ms - 1000,
        "expiresDate": now_ms + expires_in_days * DAY_MS,
    })
}

fn parse(pki: &Pki, ws: Uuid, tx: &Value) -> Result<billing::SubscriptionUpdate, SyncError> {
    parse_transaction(&settings(pki), &pki.sign(tx), Utc::now().timestamp(), ws)
}

#[test]
fn active_transaction_becomes_an_active_update_with_the_plan() {
    let (pki, ws) = (Pki::new(true), Uuid::now_v7());
    let update = parse(&pki, ws, &transaction(ws, 30)).unwrap();
    assert_eq!(update.kind, UpdateKind::Active);
    assert_eq!(update.workspace_id, Some(ws));
    assert_eq!(update.external_id, "1000000000001");
    assert_eq!(update.notification_id, "sync:2000000000007");
    assert_eq!(update.plan_key, Some(PlanKey::Home));
    assert!(update.period_ends_at.is_some());
}

#[test]
fn ended_and_revoked_transactions() {
    let (pki, ws) = (Pki::new(true), Uuid::now_v7());
    let expired = parse(&pki, ws, &transaction(ws, -2)).unwrap();
    assert_eq!(expired.kind, UpdateKind::Expired);
    let mut tx = transaction(ws, 30);
    tx["revocationDate"] = json!(Utc::now().timestamp_millis());
    assert_eq!(parse(&pki, ws, &tx).unwrap().kind, UpdateKind::Refunded);
}

#[test]
fn wrong_workspace_app_environment_product_and_signature_are_rejected() {
    let (pki, ws) = (Pki::new(true), Uuid::now_v7());
    assert_eq!(
        parse(&pki, Uuid::now_v7(), &transaction(ws, 30)).unwrap_err(),
        SyncError::Workspace
    );
    let mut tx = transaction(ws, 30);
    tx["bundleId"] = json!("app.other");
    assert_eq!(parse(&pki, ws, &tx).unwrap_err(), SyncError::Bundle);
    let mut tx = transaction(ws, 30);
    tx["environment"] = json!("Sandbox");
    assert_eq!(parse(&pki, ws, &tx).unwrap_err(), SyncError::Environment);
    let mut tx = transaction(ws, 30);
    tx["productId"] = json!("other.product");
    assert_eq!(parse(&pki, ws, &tx).unwrap_err(), SyncError::Product);
    // A chain from another root.
    let stranger = Pki::new(true);
    let jws = stranger.sign(&transaction(ws, 30));
    assert!(matches!(
        parse_transaction(&settings(&pki), &jws, Utc::now().timestamp(), ws),
        Err(SyncError::Jws(_))
    ));
}

#[test]
fn unknown_product_gives_no_plan_when_the_list_is_open() {
    let (pki, ws) = (Pki::new(true), Uuid::now_v7());
    let mut s = settings(&pki);
    s.product_ids.clear();
    let mut tx = transaction(ws, 30);
    tx["productId"] = json!("new.product");
    let update = parse_transaction(&s, &pki.sign(&tx), Utc::now().timestamp(), ws).unwrap();
    assert_eq!(update.plan_key, None);
}
