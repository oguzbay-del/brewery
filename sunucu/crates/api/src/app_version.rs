//! Client app version (`X-App-Version: ios/1.4.2+87`) and the minimum-version check
//! (ADR-0008 rule 8.3).

use std::cmp::Ordering;

use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    AppState,
    error::{ApiError, ErrorCode, FieldIssue},
};

/// Header the clients send on every request.
pub const HEADER: &str = "x-app-version";

/// A dotted numeric version with an optional `+build` suffix; compared numerically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppVersion {
    parts: Vec<u64>,
    build: u64,
    text: String,
}

impl AppVersion {
    /// Parses `1.4.2`, `1.4.2+87` or `20261113`. `None` for anything else.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        let (version, build) = match raw.split_once('+') {
            Some((v, b)) => (v, b.parse().ok()?),
            None => (raw, 0),
        };
        let parts = version
            .split('.')
            .map(|p| p.parse::<u64>().ok())
            .collect::<Option<Vec<_>>>()?;
        if parts.is_empty() {
            return None;
        }
        Some(Self {
            parts,
            build,
            text: raw.to_owned(),
        })
    }

    /// The text as given.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    fn compare(&self, other: &Self) -> Ordering {
        let len = self.parts.len().max(other.parts.len());
        for i in 0..len {
            let a = self.parts.get(i).copied().unwrap_or(0);
            let b = other.parts.get(i).copied().unwrap_or(0);
            match a.cmp(&b) {
                Ordering::Equal => {}
                ord => return ord,
            }
        }
        self.build.cmp(&other.build)
    }
}

/// Lowest supported version per platform; `None` accepts every version.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MinVersions {
    /// iOS app.
    pub ios: Option<AppVersion>,
    /// Web app (build number or dotted build id).
    pub web: Option<AppVersion>,
}

/// Outcome of checking a request header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    /// Header absent (tools, curl) or platform without a minimum.
    Allowed,
    /// Below the minimum.
    TooOld,
    /// Header present but not `platform/version`.
    Malformed,
}

impl MinVersions {
    /// Checks the `X-App-Version` header.
    #[must_use]
    pub fn check(&self, headers: &HeaderMap) -> Check {
        let Some(raw) = headers.get(HEADER) else {
            return Check::Allowed;
        };
        let Some((platform, version)) = raw.to_str().ok().and_then(|v| v.split_once('/')) else {
            return Check::Malformed;
        };
        let Some(version) = AppVersion::parse(version.trim()) else {
            return Check::Malformed;
        };
        let min = match platform.trim() {
            "ios" => &self.ios,
            "web" => &self.web,
            _ => &None,
        };
        match min {
            Some(min) if version.compare(min) == Ordering::Less => Check::TooOld,
            _ => Check::Allowed,
        }
    }
}

/// Response of `GET /v1/meta/client-requirements`.
#[derive(Debug, Serialize, ToSchema)]
pub struct ClientRequirements {
    /// Lowest supported iOS version, or `null`.
    pub min_ios_version: Option<String>,
    /// Lowest supported web build, or `null`.
    pub min_web_version: Option<String>,
}

impl From<&MinVersions> for ClientRequirements {
    fn from(m: &MinVersions) -> Self {
        Self {
            min_ios_version: m.ios.as_ref().map(|v| v.as_str().to_owned()),
            min_web_version: m.web.as_ref().map(|v| v.as_str().to_owned()),
        }
    }
}

/// Middleware: `426 upgrade_required` for clients below the minimum version. Paths that must
/// stay readable by an outdated client (`/healthz`, `/readyz`, client requirements) are exempt.
pub async fn enforce(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let path = req.uri().path();
    let exempt = matches!(
        path,
        "/healthz" | "/readyz" | "/v1/meta/client-requirements"
    );
    if !exempt {
        match state.config.min_versions.check(req.headers()) {
            Check::Allowed => {}
            Check::TooOld => {
                return axum::response::IntoResponse::into_response(ApiError::new(
                    ErrorCode::UpgradeRequired,
                ));
            }
            Check::Malformed => {
                return axum::response::IntoResponse::into_response(
                    ApiError::new(ErrorCode::MalformedRequest)
                        .with_errors(vec![FieldIssue::new(HEADER, "invalid_header")]),
                );
            }
        }
    }
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(v: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(HEADER, v.parse().unwrap());
        h
    }

    #[test]
    fn compares_numerically_with_build() {
        let min = MinVersions {
            ios: AppVersion::parse("1.4.2+87"),
            web: AppVersion::parse("120"),
        };
        assert_eq!(min.check(&HeaderMap::new()), Check::Allowed);
        assert_eq!(min.check(&headers("ios/1.4.2+87")), Check::Allowed);
        assert_eq!(min.check(&headers("ios/1.4.2+86")), Check::TooOld);
        assert_eq!(min.check(&headers("ios/1.10.0+1")), Check::Allowed);
        assert_eq!(min.check(&headers("ios/1.4")), Check::TooOld);
        assert_eq!(min.check(&headers("web/99")), Check::TooOld);
        assert_eq!(min.check(&headers("web/121")), Check::Allowed);
        assert_eq!(min.check(&headers("android/1")), Check::Allowed);
        assert_eq!(min.check(&headers("nonsense")), Check::Malformed);
        assert_eq!(min.check(&headers("ios/abc")), Check::Malformed);
    }
}
