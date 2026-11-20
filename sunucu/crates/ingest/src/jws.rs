//! JWS verification for App Store Server Notifications v2.
//!
//! Apple signs each JWS with ES256. The header carries the certificate chain in `x5c` (leaf,
//! intermediate, root). The verifier:
//! 1. requires `alg = ES256` and a chain of 2 to 4 certificates,
//! 2. checks that each certificate is valid now and signed by the next one,
//! 3. checks that the last certificate is a trusted root, or is signed by one,
//! 4. checks the Apple marker extensions (leaf and intermediate) when the policy sets them,
//! 5. verifies the JWS signature with the public key of the leaf.

use base64::{
    Engine,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use ring::signature::{ECDSA_P256_SHA256_FIXED, UnparsedPublicKey};
use serde_json::Value;
use x509_parser::{
    certificate::X509Certificate, der_parser::oid::Oid, prelude::FromDer, time::ASN1Time,
};

/// The Apple Root CA - G3 certificate (DER). SHA-256 fingerprint
/// `63:34:3A:BF:B8:9A:6A:03:EB:B5:7E:9B:3F:5F:A7:BE:7C:4F:5C:75:6F:30:17:B3:A8:C4:88:C3:65:3E:91:79`.
pub const APPLE_ROOT_CA_G3: &[u8] = include_bytes!("../certs/AppleRootCA-G3.cer");

/// Marker extension of the leaf certificate (Mac App Store receipt signing).
const OID_LEAF: &[u64] = &[1, 2, 840, 113_635, 100, 6, 11, 1];
/// Marker extension of the intermediate certificate (Worldwide Developer Relations).
const OID_INTERMEDIATE: &[u64] = &[1, 2, 840, 113_635, 100, 6, 2, 1];

/// Why a JWS was rejected.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum JwsError {
    /// The token is not three base64url parts, or the header or payload is not JSON.
    #[error("malformed JWS: {0}")]
    Malformed(&'static str),
    /// The algorithm is not ES256.
    #[error("unsupported algorithm")]
    Algorithm,
    /// The certificate chain is missing, broken, expired or not anchored in a trusted root.
    #[error("bad certificate chain: {0}")]
    Chain(&'static str),
    /// The signature does not match.
    #[error("bad signature")]
    Signature,
}

/// Verifier settings.
#[derive(Debug, Clone)]
pub struct JwsVerifier {
    roots: Vec<Vec<u8>>,
    apple_markers: bool,
}

impl JwsVerifier {
    /// Verifier for the real Apple root, with the Apple marker extensions required.
    #[must_use]
    pub fn apple() -> Self {
        Self {
            roots: vec![APPLE_ROOT_CA_G3.to_vec()],
            apple_markers: true,
        }
    }

    /// Verifier for other roots (tests). `apple_markers` requires the Apple marker extensions.
    #[must_use]
    pub const fn with_roots(roots: Vec<Vec<u8>>, apple_markers: bool) -> Self {
        Self {
            roots,
            apple_markers,
        }
    }

    /// Verifies `jws` at `now` (Unix seconds) and returns its payload as JSON.
    ///
    /// # Errors
    /// [`JwsError`] for a malformed token, a bad chain or a bad signature.
    pub fn verify(&self, jws: &str, now_unix: i64) -> Result<Value, JwsError> {
        let mut parts = jws.split('.');
        let (Some(h), Some(p), Some(s), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(JwsError::Malformed("expected three parts"));
        };
        let decode = |part: &str| {
            URL_SAFE_NO_PAD
                .decode(part)
                .map_err(|_| JwsError::Malformed("bad base64url"))
        };
        let header: Value = serde_json::from_slice(&decode(h)?)
            .map_err(|_| JwsError::Malformed("header is not JSON"))?;
        if header.get("alg").and_then(Value::as_str) != Some("ES256") {
            return Err(JwsError::Algorithm);
        }
        let chain: Vec<Vec<u8>> = header
            .get("x5c")
            .and_then(Value::as_array)
            .ok_or(JwsError::Chain("no x5c"))?
            .iter()
            .map(|c| {
                c.as_str()
                    .and_then(|c| STANDARD.decode(c).ok())
                    .ok_or(JwsError::Chain("bad x5c entry"))
            })
            .collect::<Result<_, _>>()?;
        let leaf_key = self.verify_chain(&chain, now_unix)?;

        let signature = decode(s)?;
        let signed = format!("{h}.{p}");
        UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED, leaf_key)
            .verify(signed.as_bytes(), &signature)
            .map_err(|_| JwsError::Signature)?;
        serde_json::from_slice(&decode(p)?).map_err(|_| JwsError::Malformed("payload is not JSON"))
    }

    /// Verifies the chain and returns the public key bytes of the leaf.
    fn verify_chain(&self, chain: &[Vec<u8>], now_unix: i64) -> Result<Vec<u8>, JwsError> {
        if !(2..=4).contains(&chain.len()) {
            return Err(JwsError::Chain("chain length"));
        }
        let certs: Vec<X509Certificate<'_>> = chain
            .iter()
            .map(|der| {
                X509Certificate::from_der(der)
                    .map(|(rest, c)| (rest.is_empty()).then_some(c))
                    .ok()
                    .flatten()
                    .ok_or(JwsError::Chain("unparsable certificate"))
            })
            .collect::<Result<_, _>>()?;
        let now = ASN1Time::from_timestamp(now_unix).map_err(|_| JwsError::Chain("clock"))?;

        for (i, cert) in certs.iter().enumerate() {
            if !cert.validity().is_valid_at(now) {
                return Err(JwsError::Chain("certificate not valid now"));
            }
            if i > 0 {
                let ca = cert
                    .basic_constraints()
                    .ok()
                    .flatten()
                    .is_some_and(|bc| bc.value.ca);
                if !ca {
                    return Err(JwsError::Chain("issuer is not a CA"));
                }
            }
        }
        for pair in certs.windows(2) {
            pair[0]
                .verify_signature(Some(pair[1].public_key()))
                .map_err(|_| JwsError::Chain("certificate signature"))?;
        }

        // Anchor: the last certificate is a trusted root, or a trusted root signed it.
        let last_index = certs.len() - 1;
        let last = &certs[last_index];
        let anchored = self.roots.iter().any(|root_der| {
            if root_der.as_slice() == chain[last_index].as_slice() {
                return true;
            }
            X509Certificate::from_der(root_der).is_ok_and(|(_, root)| {
                root.validity().is_valid_at(now)
                    && last.verify_signature(Some(root.public_key())).is_ok()
            })
        });
        if !anchored {
            return Err(JwsError::Chain("not anchored in a trusted root"));
        }

        if self.apple_markers {
            let has = |cert: &X509Certificate<'_>, arcs: &[u64]| {
                Oid::from(arcs).is_ok_and(|oid| cert.extensions().iter().any(|e| e.oid == oid))
            };
            if !has(&certs[0], OID_LEAF) {
                return Err(JwsError::Chain("leaf marker extension missing"));
            }
            if !has(&certs[1], OID_INTERMEDIATE) && certs.len() > 2 {
                return Err(JwsError::Chain("intermediate marker extension missing"));
            }
        }
        Ok(certs[0].public_key().subject_public_key.data.to_vec())
    }
}
