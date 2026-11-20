//! Apple Push Notification service client (HTTP/2, token based authentication).

use std::{sync::Mutex, time::Instant};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use ring::{
    rand::SystemRandom,
    signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair},
};
use serde_json::{Value, json};

/// A provider token is reused this long (Apple wants 20 to 60 minutes).
const TOKEN_TTL_SECS: u64 = 40 * 60;

/// APNs settings.
#[derive(Debug)]
pub struct ApnsConfig {
    /// Base URL for both environments (tests, mock); `None` uses the Apple hosts.
    pub base_url: Option<String>,
    /// `apns-topic`: the bundle id of the app.
    pub topic: String,
    /// Provider token signer; `None` sends no `authorization` header (mock).
    pub signer: Option<Signer>,
}

/// Signs provider tokens (JWT, ES256).
#[derive(Debug)]
pub struct Signer {
    key: EcdsaKeyPair,
    key_id: String,
    team_id: String,
    cached: Mutex<Option<(Instant, String)>>,
}

impl Signer {
    /// Builds a signer from the PEM text of the `.p8` key (PKCS#8).
    ///
    /// # Errors
    /// A text that says what is wrong with the key.
    pub fn from_pem(pem: &str, key_id: &str, team_id: &str) -> Result<Self, String> {
        let body: String = pem.lines().filter(|l| !l.starts_with("-----")).collect();
        let der = base64::engine::general_purpose::STANDARD
            .decode(body.trim())
            .map_err(|e| format!("APNs key is not base64: {e}"))?;
        let key =
            EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &der, &SystemRandom::new())
                .map_err(|e| format!("APNs key is not a P-256 PKCS#8 key: {e}"))?;
        Ok(Self {
            key,
            key_id: key_id.to_owned(),
            team_id: team_id.to_owned(),
            cached: Mutex::new(None),
        })
    }

    /// The signed token (cached).
    ///
    /// # Errors
    /// A text when signing fails.
    pub fn token(&self, now_unix: i64) -> Result<String, String> {
        let mut guard = self
            .cached
            .lock()
            .map_err(|_| "token cache poisoned".to_owned())?;
        if let Some((at, token)) = guard.as_ref()
            && at.elapsed().as_secs() < TOKEN_TTL_SECS
        {
            return Ok(token.clone());
        }
        let token = self.sign(now_unix)?;
        *guard = Some((Instant::now(), token.clone()));
        Ok(token)
    }

    fn sign(&self, now_unix: i64) -> Result<String, String> {
        let header = json!({"alg": "ES256", "kid": self.key_id});
        let claims = json!({"iss": self.team_id, "iat": now_unix});
        let head = URL_SAFE_NO_PAD.encode(header.to_string());
        let body = URL_SAFE_NO_PAD.encode(claims.to_string());
        let signing_input = format!("{head}.{body}");
        let sig = self
            .key
            .sign(&SystemRandom::new(), signing_input.as_bytes())
            .map_err(|e| format!("signing failed: {e}"))?;
        Ok(format!(
            "{signing_input}.{}",
            URL_SAFE_NO_PAD.encode(sig.as_ref())
        ))
    }
}

/// What APNs answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sent {
    /// 200.
    Accepted,
    /// 410 or `BadDeviceToken`: the token is dead and must be removed.
    Unregistered,
    /// Another 4xx: the request is wrong; a retry does not help.
    Rejected(String),
    /// 5xx, 429 or no connection: try again later.
    Transient(String),
}

/// Sends one push to one device token.
pub async fn send(
    http: &reqwest::Client,
    cfg: &ApnsConfig,
    environment: &str,
    device_token: &str,
    priority: i16,
    payload: &Value,
) -> Sent {
    let base = cfg.base_url.clone().unwrap_or_else(|| {
        if environment == "sandbox" {
            "https://api.sandbox.push.apple.com".to_owned()
        } else {
            "https://api.push.apple.com".to_owned()
        }
    });
    let mut request = http
        .post(format!(
            "{}/3/device/{device_token}",
            base.trim_end_matches('/')
        ))
        .header("apns-topic", &cfg.topic)
        .header("apns-push-type", "alert")
        .header("apns-priority", if priority >= 3 { "5" } else { "10" })
        .json(payload);
    if let Some(signer) = &cfg.signer {
        match signer.token(chrono::Utc::now().timestamp()) {
            Ok(t) => request = request.bearer_auth(t),
            Err(e) => return Sent::Transient(e),
        }
    }
    match request.send().await {
        Ok(r) if r.status().is_success() => Sent::Accepted,
        Ok(r) => {
            let status = r.status();
            let reason = r.text().await.unwrap_or_default();
            if status.as_u16() == 410 || reason.contains("BadDeviceToken") {
                Sent::Unregistered
            } else if status.is_server_error() || status.as_u16() == 429 {
                Sent::Transient(format!("{status} {reason}"))
            } else {
                Sent::Rejected(format!("{status} {reason}"))
            }
        }
        Err(e) => Sent::Transient(e.to_string()),
    }
}

/// The APNs body of a notification: the client renders the text from the key.
#[must_use]
pub fn payload(priority: i16, text_key: &str, args: &[String], notification_id: &str) -> Value {
    let level = match priority {
        1 => "time-sensitive",
        2 => "active",
        _ => "passive",
    };
    json!({
        "aps": {
            "alert": {"title-loc-key": text_key, "loc-args": args},
            "sound": "default",
            "interruption-level": level,
        },
        "notification_id": notification_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signer_makes_a_three_part_token() {
        let pkcs8 =
            EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &SystemRandom::new())
                .unwrap();
        let pem = format!(
            "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----\n",
            base64::engine::general_purpose::STANDARD.encode(pkcs8.as_ref())
        );
        let signer = Signer::from_pem(&pem, "KEY1", "TEAM1").unwrap();
        let token = signer.token(1_700_000_000).unwrap();
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
        let header: Value =
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(parts[0]).unwrap()).unwrap();
        assert_eq!(header["kid"], "KEY1");
        assert_eq!(URL_SAFE_NO_PAD.decode(parts[2]).unwrap().len(), 64);
        // Cached: the same token again.
        assert_eq!(signer.token(1_700_000_999).unwrap(), token);
    }

    #[test]
    fn level_follows_priority() {
        assert_eq!(
            payload(1, "k", &[], "n")["aps"]["interruption-level"],
            "time-sensitive"
        );
        assert_eq!(
            payload(3, "k", &[], "n")["aps"]["interruption-level"],
            "passive"
        );
    }
}
