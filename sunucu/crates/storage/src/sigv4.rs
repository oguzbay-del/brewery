//! AWS Signature Version 4 for S3 (also accepted by GCS' S3 interoperability API).
//!
//! Hand written on purpose: only `hmac` and `sha2` are needed, and the signing code stays small
//! enough to check against the published AWS examples (see the tests).

use chrono::{DateTime, Utc};
use hmac::{Hmac, KeyInit as _, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

/// `x-amz-content-sha256` value of a request without a signed body.
pub const UNSIGNED_PAYLOAD: &str = "UNSIGNED-PAYLOAD";
/// SHA-256 of the empty body.
pub const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
const ALGORITHM: &str = "AWS4-HMAC-SHA256";

/// Lower-case hex.
#[must_use]
pub fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut acc, b| {
        let _ = write!(acc, "{b:02x}");
        acc
    })
}

/// SHA-256 as hex.
#[must_use]
pub fn sha256_hex(data: &[u8]) -> String {
    hex(&Sha256::digest(data))
}

fn hmac(key: &[u8], data: &[u8]) -> Vec<u8> {
    // HMAC accepts keys of any length, so this cannot fail.
    HmacSha256::new_from_slice(key).map_or_else(
        |_| Vec::new(),
        |mut mac| {
            mac.update(data);
            mac.finalize().into_bytes().to_vec()
        },
    )
}

/// Percent-encodes everything but the unreserved characters; `/` stays when `keep_slash`.
#[must_use]
pub fn encode(input: &str, keep_slash: bool) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(input.len());
    for b in input.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(b));
        } else if b == b'/' && keep_slash {
            out.push('/');
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

/// The `yyyymmdd'T'hhmmss'Z'` timestamp.
#[must_use]
pub fn amz_date(now: DateTime<Utc>) -> String {
    now.format("%Y%m%dT%H%M%SZ").to_string()
}

/// Scope `<date>/<region>/s3/aws4_request`.
#[must_use]
pub fn scope(now: DateTime<Utc>, region: &str) -> String {
    format!("{}/{region}/s3/aws4_request", now.format("%Y%m%d"))
}

/// A request to sign. `path` is already percent-encoded; `query` and `headers` are raw.
#[derive(Debug)]
pub struct Canonical<'a> {
    /// HTTP method.
    pub method: &'a str,
    /// Encoded absolute path.
    pub path: &'a str,
    /// Query parameters (any order, unencoded).
    pub query: &'a [(String, String)],
    /// Signed headers (any order; names are lower-cased here).
    pub headers: &'a [(String, String)],
    /// `x-amz-content-sha256` value or `UNSIGNED-PAYLOAD`.
    pub payload_hash: &'a str,
}

impl Canonical<'_> {
    /// Sorted, encoded query string.
    #[must_use]
    pub fn query_string(&self) -> String {
        let mut pairs: Vec<(String, String)> = self
            .query
            .iter()
            .map(|(k, v)| (encode(k, false), encode(v, false)))
            .collect();
        pairs.sort();
        pairs
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("&")
    }

    /// Signed header names, `;`-joined.
    #[must_use]
    pub fn signed_headers(&self) -> String {
        let mut names: Vec<String> = self
            .headers
            .iter()
            .map(|(k, _)| k.to_ascii_lowercase())
            .collect();
        names.sort();
        names.join(";")
    }

    fn request_text(&self) -> String {
        let mut headers: Vec<(String, String)> = self
            .headers
            .iter()
            .map(|(k, v)| (k.to_ascii_lowercase(), v.trim().to_owned()))
            .collect();
        headers.sort();
        let block = headers.iter().fold(String::new(), |mut acc, (k, v)| {
            use std::fmt::Write as _;
            let _ = writeln!(acc, "{k}:{v}");
            acc
        });
        format!(
            "{}\n{}\n{}\n{block}\n{}\n{}",
            self.method,
            self.path,
            self.query_string(),
            self.signed_headers(),
            self.payload_hash
        )
    }

    /// The hex signature.
    #[must_use]
    pub fn signature(&self, secret: &str, region: &str, now: DateTime<Utc>) -> String {
        let to_sign = format!(
            "{ALGORITHM}\n{}\n{}\n{}",
            amz_date(now),
            scope(now, region),
            sha256_hex(self.request_text().as_bytes())
        );
        let k_date = hmac(
            format!("AWS4{secret}").as_bytes(),
            now.format("%Y%m%d").to_string().as_bytes(),
        );
        let k_region = hmac(&k_date, region.as_bytes());
        let k_service = hmac(&k_region, b"s3");
        let k_signing = hmac(&k_service, b"aws4_request");
        hex(&hmac(&k_signing, to_sign.as_bytes()))
    }
}

/// Query-string parameters of a presigned URL (everything but the signature).
#[must_use]
pub fn presign_params(
    access_key: &str,
    region: &str,
    now: DateTime<Utc>,
    expires_secs: u64,
    signed_headers: &str,
) -> Vec<(String, String)> {
    vec![
        ("X-Amz-Algorithm".to_owned(), ALGORITHM.to_owned()),
        (
            "X-Amz-Credential".to_owned(),
            format!("{access_key}/{}", scope(now, region)),
        ),
        ("X-Amz-Date".to_owned(), amz_date(now)),
        ("X-Amz-Expires".to_owned(), expires_secs.to_string()),
        ("X-Amz-SignedHeaders".to_owned(), signed_headers.to_owned()),
    ]
}

/// The `Authorization` header value of a header-signed request.
#[must_use]
pub fn authorization(
    access_key: &str,
    secret: &str,
    region: &str,
    now: DateTime<Utc>,
    request: &Canonical<'_>,
) -> String {
    format!(
        "{ALGORITHM} Credential={access_key}/{}, SignedHeaders={}, Signature={}",
        scope(now, region),
        request.signed_headers(),
        request.signature(secret, region, now)
    )
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;

    use super::*;

    const SECRET: &str = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";

    #[test]
    fn matches_the_aws_presigned_get_example() {
        // https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-query-string-auth.html
        let now = Utc.with_ymd_and_hms(2013, 5, 24, 0, 0, 0).unwrap();
        let mut query = presign_params("AKIAIOSFODNN7EXAMPLE", "us-east-1", now, 86400, "host");
        let headers = [(
            "host".to_owned(),
            "examplebucket.s3.amazonaws.com".to_owned(),
        )];
        let canonical = Canonical {
            method: "GET",
            path: "/test.txt",
            query: &query,
            headers: &headers,
            payload_hash: UNSIGNED_PAYLOAD,
        };
        let signature = canonical.signature(SECRET, "us-east-1", now);
        assert_eq!(
            signature,
            "aeeed9bbccd4d02ee5c0109b86d86835f995330da4c265957d157751f604d404"
        );
        query.push(("X-Amz-Signature".to_owned(), signature));
    }

    #[test]
    fn matches_the_aws_header_signed_get_example() {
        // https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-header-based-auth.html
        let now = Utc.with_ymd_and_hms(2013, 5, 24, 0, 0, 0).unwrap();
        let headers = [
            (
                "host".to_owned(),
                "examplebucket.s3.amazonaws.com".to_owned(),
            ),
            ("range".to_owned(), "bytes=0-9".to_owned()),
            ("x-amz-content-sha256".to_owned(), EMPTY_SHA256.to_owned()),
            ("x-amz-date".to_owned(), "20130524T000000Z".to_owned()),
        ];
        let canonical = Canonical {
            method: "GET",
            path: "/test.txt",
            query: &[],
            headers: &headers,
            payload_hash: EMPTY_SHA256,
        };
        assert_eq!(
            canonical.signature(SECRET, "us-east-1", now),
            "f0e8bdb87c964420e857bd35b5d6ed310bd44f0170aba48dd91039c6036bdb41"
        );
    }

    #[test]
    fn encodes_like_s3() {
        assert_eq!(encode("a b/ü~", true), "a%20b/%C3%BC~");
        assert_eq!(encode("a/b", false), "a%2Fb");
    }
}
