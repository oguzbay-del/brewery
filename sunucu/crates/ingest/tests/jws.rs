//! JWS verification with a test root: good chain, bad signature, bad chain.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use billing::UpdateKind::{Active, Expired, Grace, Refunded};
use ingest::{
    apple::{AppleError, Parsed, parse_notification},
    jws::{APPLE_ROOT_CA_G3, JwsError, JwsVerifier},
};
use serde_json::json;
use support::{Notification, Pki};

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

fn note(kind: &'static str) -> Notification {
    Notification {
        id: "11111111-1111-1111-1111-111111111111".into(),
        kind,
        subtype: None,
        workspace: "22222222-2222-2222-2222-222222222222".into(),
        original_transaction_id: "1000000000001".into(),
        signed_ms: 1_800_000_000_000,
        expires_ms: 1_800_000_000_000 + 30 * 86_400_000,
    }
}

#[test]
fn apple_root_is_embedded() {
    assert_eq!(APPLE_ROOT_CA_G3.first(), Some(&0x30), "DER sequence");
    assert!(APPLE_ROOT_CA_G3.len() > 400);
}

#[test]
fn good_chain_verifies() {
    let pki = Pki::new(true);
    let v = JwsVerifier::with_roots(vec![pki.root_der.clone()], true);
    let jws = pki.sign(&json!({"a": 1}));
    assert_eq!(v.verify(&jws, now()).unwrap()["a"], 1);
}

#[test]
fn chain_without_root_entry_is_anchored_by_the_trusted_root() {
    let pki = Pki::new(true);
    let v = JwsVerifier::with_roots(vec![pki.root_der.clone()], true);
    let jws = pki.sign_with(&[&pki.leaf_der, &pki.intermediate_der], &json!({"a": 1}));
    assert!(v.verify(&jws, now()).is_ok());
}

#[test]
fn tampered_payload_is_rejected() {
    let pki = Pki::new(true);
    let v = JwsVerifier::with_roots(vec![pki.root_der.clone()], true);
    let jws = pki.sign(&json!({"a": 1}));
    let other = pki.sign(&json!({"a": 2}));
    let mut parts: Vec<&str> = jws.split('.').collect();
    parts[1] = other.split('.').nth(1).unwrap();
    assert_eq!(v.verify(&parts.join("."), now()), Err(JwsError::Signature));
}

#[test]
fn other_root_is_rejected() {
    let pki = Pki::new(true);
    let stranger = Pki::new(true);
    let v = JwsVerifier::with_roots(vec![stranger.root_der.clone()], true);
    let jws = pki.sign(&json!({"a": 1}));
    assert!(matches!(v.verify(&jws, now()), Err(JwsError::Chain(_))));
}

#[test]
fn real_apple_root_rejects_a_test_chain() {
    let pki = Pki::new(true);
    let jws = pki.sign(&json!({"a": 1}));
    assert!(matches!(
        JwsVerifier::apple().verify(&jws, now()),
        Err(JwsError::Chain(_))
    ));
}

#[test]
fn leaf_from_another_chain_is_rejected() {
    let pki = Pki::new(true);
    let stranger = Pki::new(true);
    let v = JwsVerifier::with_roots(vec![pki.root_der.clone()], true);
    // The leaf of the stranger does not chain to our intermediate.
    let jws = pki.sign_with(
        &[&stranger.leaf_der, &pki.intermediate_der, &pki.root_der],
        &json!({"a": 1}),
    );
    assert!(matches!(v.verify(&jws, now()), Err(JwsError::Chain(_))));
}

#[test]
fn signature_with_another_leaf_key_is_rejected() {
    let pki = Pki::new(true);
    let other_key = Pki::new(true);
    let v = JwsVerifier::with_roots(vec![pki.root_der.clone()], true);
    // Signed by another key, but the header shows our valid chain.
    let jws = other_key.sign_with(
        &[&pki.leaf_der, &pki.intermediate_der, &pki.root_der],
        &json!({"a": 1}),
    );
    assert_eq!(v.verify(&jws, now()), Err(JwsError::Signature));
}

#[test]
fn missing_apple_markers_are_rejected_under_the_apple_policy() {
    let pki = Pki::new(false);
    let strict = JwsVerifier::with_roots(vec![pki.root_der.clone()], true);
    let jws = pki.sign(&json!({"a": 1}));
    assert!(matches!(
        strict.verify(&jws, now()),
        Err(JwsError::Chain(_))
    ));
    let lax = JwsVerifier::with_roots(vec![pki.root_der.clone()], false);
    assert!(lax.verify(&jws, now()).is_ok());
}

#[test]
fn expired_chain_is_rejected() {
    let pki = Pki::new(true);
    let v = JwsVerifier::with_roots(vec![pki.root_der.clone()], true);
    let jws = pki.sign(&json!({"a": 1}));
    // Year 5000, after the test certificates expire.
    assert!(matches!(
        v.verify(&jws, 95_000_000_000),
        Err(JwsError::Chain(_))
    ));
}

#[test]
fn malformed_tokens_and_algorithms_are_rejected() {
    let pki = Pki::new(true);
    let v = JwsVerifier::with_roots(vec![pki.root_der.clone()], true);
    for bad in ["", "a.b", "a.b.c.d", "!!.!!.!!"] {
        assert!(
            matches!(v.verify(bad, now()), Err(JwsError::Malformed(_))),
            "{bad}"
        );
    }
    let none = pki.sign_header(&json!({"alg": "none"}), &json!({}));
    assert_eq!(v.verify(&none, now()), Err(JwsError::Algorithm));
    let no_chain = pki.sign_header(&json!({"alg": "ES256"}), &json!({}));
    assert!(matches!(
        v.verify(&no_chain, now()),
        Err(JwsError::Chain(_))
    ));
}

#[test]
fn notification_types_map_to_updates() {
    let pki = Pki::new(true);
    let v = JwsVerifier::with_roots(vec![pki.root_der.clone()], true);
    let kind = |t: &'static str, sub: Option<&'static str>| {
        let mut n = note(t);
        n.subtype = sub;
        match parse_notification(&v, &pki.notification(&n), now(), Some("app.demleme")).unwrap() {
            Parsed::Update(u) => Some(u.kind),
            Parsed::Ignored { .. } => None,
        }
    };
    assert_eq!(kind("SUBSCRIBED", Some("INITIAL_BUY")), Some(Active));
    assert_eq!(kind("DID_RENEW", None), Some(Active));
    assert_eq!(kind("DID_FAIL_TO_RENEW", Some("GRACE_PERIOD")), Some(Grace));
    assert_eq!(kind("DID_FAIL_TO_RENEW", None), Some(Expired));
    assert_eq!(kind("EXPIRED", Some("VOLUNTARY")), Some(Expired));
    assert_eq!(kind("REFUND", None), Some(Refunded));
    assert_eq!(kind("REVOKE", None), Some(Refunded));
    assert_eq!(kind("DID_CHANGE_RENEWAL_STATUS", None), None);
    assert_eq!(kind("TEST", None), None);
}

#[test]
fn bundle_mismatch_is_rejected() {
    let pki = Pki::new(true);
    let v = JwsVerifier::with_roots(vec![pki.root_der.clone()], true);
    let jws = pki.notification(&note("DID_RENEW"));
    assert_eq!(
        parse_notification(&v, &jws, now(), Some("app.other")),
        Err(AppleError::Bundle)
    );
}
