//! Test PKI and App Store notification builders. The chain imitates Apple's: root, intermediate
//! (marker extension), leaf (marker extension), all P-256.

#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use base64::{
    Engine,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use rcgen::{
    BasicConstraints, CertificateParams, CustomExtension, IsCa, Issuer, KeyPair,
    PKCS_ECDSA_P256_SHA256,
};
use ring::{
    rand::SystemRandom,
    signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair},
};
use serde_json::{Value, json};

const OID_LEAF: &[u64] = &[1, 2, 840, 113_635, 100, 6, 11, 1];
const OID_INTERMEDIATE: &[u64] = &[1, 2, 840, 113_635, 100, 6, 2, 1];
/// DER of ASN.1 NULL, a valid extension value.
const NULL_DER: &[u8] = &[0x05, 0x00];

/// A test chain.
pub struct Pki {
    pub root_der: Vec<u8>,
    pub intermediate_der: Vec<u8>,
    pub leaf_der: Vec<u8>,
    leaf_pkcs8: Vec<u8>,
}

fn params(ca: bool, marker: Option<&[u64]>) -> CertificateParams {
    let mut p = CertificateParams::new(Vec::<String>::new()).unwrap();
    if ca {
        p.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    }
    if let Some(oid) = marker {
        p.custom_extensions
            .push(CustomExtension::from_oid_content(oid, NULL_DER.to_vec()));
    }
    p
}

impl Pki {
    /// A new chain. `markers` adds the Apple marker extensions.
    pub fn new(markers: bool) -> Self {
        let root_key = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256).unwrap();
        let root_params = params(true, None);
        let root = root_params.self_signed(&root_key).unwrap();
        let root_issuer = Issuer::new(root_params, root_key);

        let inter_key = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256).unwrap();
        let inter_params = params(true, markers.then_some(OID_INTERMEDIATE));
        let inter = inter_params.signed_by(&inter_key, &root_issuer).unwrap();
        let inter_issuer = Issuer::new(inter_params, inter_key);

        let leaf_key = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256).unwrap();
        let leaf_params = params(false, markers.then_some(OID_LEAF));
        let leaf = leaf_params.signed_by(&leaf_key, &inter_issuer).unwrap();

        Self {
            root_der: root.der().to_vec(),
            intermediate_der: inter.der().to_vec(),
            leaf_der: leaf.der().to_vec(),
            leaf_pkcs8: leaf_key.serialize_der(),
        }
    }

    /// Signs `payload` as an ES256 JWS with the given `x5c` chain.
    pub fn sign_with(&self, chain: &[&[u8]], payload: &Value) -> String {
        let header = json!({
            "alg": "ES256",
            "x5c": chain.iter().map(|c| STANDARD.encode(c)).collect::<Vec<_>>(),
        });
        self.sign_header(&header, payload)
    }

    pub fn sign_header(&self, header: &Value, payload: &Value) -> String {
        let h = URL_SAFE_NO_PAD.encode(header.to_string());
        let p = URL_SAFE_NO_PAD.encode(payload.to_string());
        let rng = SystemRandom::new();
        let key =
            EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &self.leaf_pkcs8, &rng)
                .unwrap();
        let sig = key.sign(&rng, format!("{h}.{p}").as_bytes()).unwrap();
        format!("{h}.{p}.{}", URL_SAFE_NO_PAD.encode(sig.as_ref()))
    }

    /// Signs with the full chain (leaf, intermediate, root).
    pub fn sign(&self, payload: &Value) -> String {
        self.sign_with(
            &[&self.leaf_der, &self.intermediate_der, &self.root_der],
            payload,
        )
    }

    /// A `signedPayload` for a notification.
    pub fn notification(&self, n: &Notification) -> String {
        self.notification_for(n, "app.demleme", "Production")
    }

    /// A `signedPayload` for another bundle id and environment.
    pub fn notification_for(&self, n: &Notification, bundle: &str, environment: &str) -> String {
        let tx = self.sign(&json!({
            "originalTransactionId": n.original_transaction_id,
            "transactionId": format!("{}-tx", n.id),
            "productId": "pro.monthly",
            "appAccountToken": n.workspace,
            "expiresDate": n.expires_ms,
            "bundleId": bundle,
            "environment": environment,
        }));
        self.sign(&json!({
            "notificationType": n.kind,
            "subtype": n.subtype,
            "notificationUUID": n.id,
            "signedDate": n.signed_ms,
            "version": "2.0",
            "data": {
                "environment": environment,
                "bundleId": bundle,
                "signedTransactionInfo": tx,
            },
        }))
    }
}

/// Fields of a test notification.
pub struct Notification {
    pub id: String,
    pub kind: &'static str,
    pub subtype: Option<&'static str>,
    pub workspace: String,
    pub original_transaction_id: String,
    pub signed_ms: i64,
    pub expires_ms: i64,
}
