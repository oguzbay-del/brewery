//! Cursor pagination, sorting and list filters (ADR-0008 rule 4).

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult, ErrorCode, FieldIssue};

/// Page size when `limit` is absent.
pub const DEFAULT_LIMIT: u32 = 50;
/// Largest accepted `limit`.
pub const MAX_LIMIT: u32 = 200;

/// Query parameters every list endpoint accepts.
#[derive(Debug, Default, Clone, Deserialize, ToSchema)]
pub struct ListParams {
    /// Page size (1..=200, default 50).
    pub limit: Option<u32>,
    /// Opaque cursor from the previous page's `next_cursor`.
    pub cursor: Option<String>,
    /// Sort field, `-` prefix for descending (`-updated_at`).
    pub sort: Option<String>,
    /// Only records changed since this instant, deleted ones included (cache refresh).
    pub updated_since: Option<DateTime<Utc>>,
    /// Include soft-deleted records (trash view).
    pub include_deleted: Option<bool>,
}

impl ListParams {
    /// The page size.
    ///
    /// # Errors
    /// `422 validation_failed` when `limit` is outside `1..=200`.
    pub fn limit(&self) -> ApiResult<u32> {
        let limit = self.limit.unwrap_or(DEFAULT_LIMIT);
        if (1..=MAX_LIMIT).contains(&limit) {
            Ok(limit)
        } else {
            Err(ApiError::validation(vec![
                FieldIssue::new("limit", "out_of_range")
                    .with_params(serde_json::json!({"min": 1, "max": MAX_LIMIT})),
            ]))
        }
    }

    /// The decoded cursor for the given sort, if one was sent.
    ///
    /// # Errors
    /// `400 malformed_request` for a corrupt cursor or one made for another sort.
    pub fn cursor(&self, sort: &SortSpec) -> ApiResult<Option<Cursor>> {
        self.cursor
            .as_deref()
            .map(|raw| Cursor::decode(raw, sort))
            .transpose()
    }
}

/// A validated sort: field and direction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortSpec {
    /// Field name (one of the endpoint's allowed fields).
    pub field: &'static str,
    /// Descending order.
    pub descending: bool,
}

impl SortSpec {
    /// Parses `?sort=` against the endpoint's allowed fields; `default` (such as
    /// `-updated_at`) applies when absent. Ties are always broken by `id`.
    ///
    /// # Errors
    /// `422 validation_failed` for a field that is not allowed.
    pub fn parse(
        raw: Option<&str>,
        allowed: &[&'static str],
        default: &'static str,
    ) -> ApiResult<Self> {
        let raw = raw.unwrap_or(default);
        let (descending, name) = raw
            .strip_prefix('-')
            .map_or((false, raw), |name| (true, name));
        let field = allowed.iter().find(|f| **f == name).ok_or_else(|| {
            ApiError::validation(vec![
                FieldIssue::new("sort", "invalid_sort")
                    .with_params(serde_json::json!({"allowed": allowed})),
            ])
        })?;
        Ok(Self { field, descending })
    }

    /// The canonical `?sort=` text.
    #[must_use]
    pub fn to_param(&self) -> String {
        if self.descending {
            format!("-{}", self.field)
        } else {
            self.field.to_owned()
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CursorBody {
    /// Sort parameter the cursor was made for.
    s: String,
    /// Sort key of the last row.
    k: Value,
    /// `id` of the last row (tie breaker).
    id: Uuid,
}

/// Position after the last row of a page.
#[derive(Debug, Clone, PartialEq)]
pub struct Cursor {
    /// Value of the sort field in the last row.
    pub key: Value,
    /// `id` of the last row.
    pub id: Uuid,
}

impl Cursor {
    /// Encodes as an opaque base64url string bound to `sort`.
    #[must_use]
    pub fn encode(&self, sort: &SortSpec) -> String {
        let body = CursorBody {
            s: sort.to_param(),
            k: self.key.clone(),
            id: self.id,
        };
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&body).unwrap_or_default())
    }

    /// Decodes and checks a cursor.
    ///
    /// # Errors
    /// `400 malformed_request` (`invalid_cursor`) when it is corrupt or belongs to another sort.
    pub fn decode(raw: &str, sort: &SortSpec) -> ApiResult<Self> {
        let invalid = || {
            ApiError::new(ErrorCode::MalformedRequest)
                .with_errors(vec![FieldIssue::new("cursor", "invalid_cursor")])
        };
        let bytes = URL_SAFE_NO_PAD.decode(raw).map_err(|_| invalid())?;
        let body: CursorBody = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
        if body.s != sort.to_param() {
            return Err(invalid());
        }
        Ok(Self {
            key: body.k,
            id: body.id,
        })
    }
}

/// A page of a list endpoint: `{ "items": [...], "next_cursor": "..." | null }`.
#[derive(Debug, Serialize, ToSchema)]
pub struct Page<T> {
    /// The rows of this page.
    pub items: Vec<T>,
    /// Cursor of the next page; `null` on the last page.
    pub next_cursor: Option<String>,
}

impl<T> Page<T> {
    /// Builds a page from rows fetched with `LIMIT limit + 1`: the extra row only signals that
    /// more exist. `position` gives the sort key and id of a row.
    pub fn from_rows(
        mut rows: Vec<T>,
        limit: u32,
        sort: &SortSpec,
        position: impl Fn(&T) -> Cursor,
    ) -> Self {
        let limit = limit as usize;
        let next_cursor = if rows.len() > limit {
            rows.truncate(limit);
            rows.last().map(|row| position(row).encode(sort))
        } else {
            None
        };
        Self {
            items: rows,
            next_cursor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALLOWED: [&str; 2] = ["updated_at", "name"];

    #[test]
    fn limit_bounds() {
        let p = |limit| ListParams {
            limit,
            ..ListParams::default()
        };
        assert_eq!(p(None).limit().unwrap(), 50);
        assert_eq!(p(Some(200)).limit().unwrap(), 200);
        assert!(p(Some(0)).limit().is_err());
        assert!(p(Some(201)).limit().is_err());
    }

    #[test]
    fn sort_parsing() {
        let s = SortSpec::parse(None, &ALLOWED, "-updated_at").unwrap();
        assert!(s.descending && s.field == "updated_at");
        let s = SortSpec::parse(Some("name"), &ALLOWED, "-updated_at").unwrap();
        assert!(!s.descending && s.to_param() == "name");
        assert!(SortSpec::parse(Some("password"), &ALLOWED, "name").is_err());
    }

    #[test]
    fn cursor_roundtrip_and_tamper() {
        let sort = SortSpec::parse(Some("name"), &ALLOWED, "name").unwrap();
        let c = Cursor {
            key: serde_json::json!("Pale Ale"),
            id: Uuid::now_v7(),
        };
        let raw = c.encode(&sort);
        assert_eq!(Cursor::decode(&raw, &sort).unwrap(), c);
        let other = SortSpec::parse(Some("-name"), &ALLOWED, "name").unwrap();
        assert!(Cursor::decode(&raw, &other).is_err());
        assert!(Cursor::decode("%%%", &sort).is_err());
        assert!(Cursor::decode("e30", &sort).is_err());
    }

    #[test]
    fn page_signals_more_rows() {
        let sort = SortSpec::parse(None, &ALLOWED, "name").unwrap();
        let pos = |n: &i32| Cursor {
            key: serde_json::json!(n),
            id: Uuid::from_u128(u128::from(n.unsigned_abs())),
        };
        let page = Page::from_rows(vec![1, 2, 3], 2, &sort, pos);
        assert_eq!(page.items, vec![1, 2]);
        assert!(page.next_cursor.is_some());
        assert!(
            Page::from_rows(vec![1, 2], 2, &sort, pos)
                .next_cursor
                .is_none()
        );
    }
}
