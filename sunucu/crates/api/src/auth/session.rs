//! Sessions (ADR-0009 rule 3): opaque token, stored as a SHA-256 digest, with a five minute
//! Redis cache of the record. Revoking deletes the cache entry; without Redis every request
//! reads PostgreSQL.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use super::tokens::{hex, new_token, token_hash};
use crate::{
    error::{ApiError, ApiResult},
    redis_client::Redis,
};

/// How long the Redis copy of a session record lives.
const CACHE_TTL_SECS: u64 = 300;
/// `last_activity_at` is written at most this often (ADR-0009 rule 3.3).
pub const ACTIVITY_WRITE_EVERY: Duration = Duration::seconds(60);

/// Which client holds the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientKind {
    /// The iOS app (bearer token).
    Ios,
    /// A browser (cookie).
    Web,
    /// A member signed in on a shared device (T2.12).
    SharedDevice,
}

impl ClientKind {
    /// Value of `sessions.client_kind`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ios => "ios",
            Self::Web => "web",
            Self::SharedDevice => "shared_device",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "ios" => Some(Self::Ios),
            "web" => Some(Self::Web),
            "shared_device" => Some(Self::SharedDevice),
            _ => None,
        }
    }
}

/// Idle timeout and absolute lifetime (ADR-0009 rule 3.3; values still to be confirmed).
#[must_use]
pub fn lifetimes(kind: ClientKind, remember: bool) -> (Duration, Duration) {
    match (kind, remember) {
        (ClientKind::Ios, _) => (Duration::days(30), Duration::days(90)),
        (ClientKind::Web, true) => (Duration::days(7), Duration::days(30)),
        (ClientKind::Web, false) => (Duration::hours(12), Duration::hours(12)),
        (ClientKind::SharedDevice, _) => (Duration::minutes(5), Duration::hours(12)),
    }
}

/// A session row as the API uses it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRecord {
    /// Session id.
    pub id: Uuid,
    /// Owner.
    pub user_id: Uuid,
    /// Client.
    pub client_kind: ClientKind,
    /// Shared device, for `shared_device` sessions.
    pub shared_device_id: Option<Uuid>,
    /// "Remember this computer" was ticked.
    pub remember: bool,
    /// Creation (sign-in) time.
    pub created_at: DateTime<Utc>,
    /// Last write of the activity time.
    pub last_activity_at: DateTime<Utc>,
    /// Absolute end.
    pub expires_at: DateTime<Utc>,
    /// Time of the last second factor.
    pub step_up_at: Option<DateTime<Utc>>,
    /// Set when closed.
    pub revoked_at: Option<DateTime<Utc>>,
}

impl SessionRecord {
    /// Whether the session may still be used at `now`: not closed, inside its absolute
    /// lifetime and not idle for longer than the client kind allows.
    #[must_use]
    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        let (idle, _) = lifetimes(self.client_kind, self.remember);
        self.revoked_at.is_none() && now < self.expires_at && now - self.last_activity_at < idle
    }
}

#[derive(sqlx::FromRow)]
struct Row {
    id: Uuid,
    user_id: Uuid,
    client_kind: String,
    shared_device_id: Option<Uuid>,
    remember: bool,
    created_at: DateTime<Utc>,
    last_activity_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    step_up_at: Option<DateTime<Utc>>,
    revoked_at: Option<DateTime<Utc>>,
}

impl TryFrom<Row> for SessionRecord {
    type Error = ApiError;

    fn try_from(r: Row) -> Result<Self, ApiError> {
        let client_kind = ClientKind::parse(&r.client_kind).ok_or_else(|| {
            tracing::error!(kind = %r.client_kind, "unknown session client kind");
            ApiError::new(crate::ErrorCode::Internal)
        })?;
        Ok(Self {
            id: r.id,
            user_id: r.user_id,
            client_kind,
            shared_device_id: r.shared_device_id,
            remember: r.remember,
            created_at: r.created_at,
            last_activity_at: r.last_activity_at,
            expires_at: r.expires_at,
            step_up_at: r.step_up_at,
            revoked_at: r.revoked_at,
        })
    }
}

// Fixed text, no user input: safe to splice into the queries below.
const COLUMNS: &str = "id, user_id, client_kind, shared_device_id, remember, created_at, \
                       last_activity_at, expires_at, step_up_at, revoked_at";

/// Creates a session inside the caller's transaction (which must run with the user's context)
/// and returns its token, shown once, and the record.
pub async fn insert(
    conn: &mut PgConnection,
    user_id: Uuid,
    kind: ClientKind,
    remember: bool,
    device_name: &str,
    approx_location: Option<&str>,
) -> sqlx::Result<(String, SessionRecord)> {
    let token = new_token();
    let (_, max) = lifetimes(kind, remember);
    let now = Utc::now();
    let record = SessionRecord {
        id: Uuid::now_v7(),
        user_id,
        client_kind: kind,
        shared_device_id: None,
        remember,
        created_at: now,
        last_activity_at: now,
        expires_at: now + max,
        step_up_at: None,
        revoked_at: None,
    };
    sqlx::query(
        "INSERT INTO sessions (id, user_id, token_hash, client_kind, device_name, approx_location, \
                               remember, last_activity_at, expires_at, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $8)",
    )
    .bind(record.id)
    .bind(user_id)
    .bind(token_hash(&token))
    .bind(kind.as_str())
    .bind(device_name)
    .bind(approx_location)
    .bind(remember)
    .bind(now)
    .bind(record.expires_at)
    .execute(conn)
    .await?;
    Ok((token, record))
}

/// Session lookups and changes.
#[derive(Debug, Clone)]
pub struct SessionStore {
    db: PgPool,
    redis: Redis,
}

/// One row of the session list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionListItem {
    /// The record.
    pub record: SessionRecord,
    /// Device name given at sign-in.
    pub device_name: String,
    /// Approximate location (country), if known.
    pub approx_location: Option<String>,
}

impl SessionStore {
    /// A store over `db` and `redis`.
    #[must_use]
    pub const fn new(db: PgPool, redis: Redis) -> Self {
        Self { db, redis }
    }

    fn cache_key(hash: &[u8]) -> String {
        format!("sess:{}", hex(hash))
    }

    /// Finds the session of `hash` (cache first, then the `SECURITY DEFINER` lookup).
    pub async fn lookup(&self, hash: &[u8]) -> ApiResult<Option<SessionRecord>> {
        let key = Self::cache_key(hash);
        let cached = self
            .redis
            .run(|mut c| {
                let key = key.clone();
                async move {
                    redis::cmd("GET")
                        .arg(key)
                        .query_async::<Option<String>>(&mut c)
                        .await
                }
            })
            .await
            .flatten();
        if let Some(record) = cached.and_then(|s| serde_json::from_str::<SessionRecord>(&s).ok()) {
            return Ok(Some(record));
        }
        let row: Option<Row> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
            "SELECT {COLUMNS} FROM find_session_by_token_hash($1)"
        )))
        .bind(hash)
        .fetch_optional(&self.db)
        .await?;
        let Some(record) = row.map(SessionRecord::try_from).transpose()? else {
            return Ok(None);
        };
        self.cache_put(hash, &record).await;
        Ok(Some(record))
    }

    async fn cache_put(&self, hash: &[u8], record: &SessionRecord) {
        if record.revoked_at.is_some() {
            return;
        }
        let Ok(json) = serde_json::to_string(record) else {
            return;
        };
        let key = Self::cache_key(hash);
        self.redis
            .run(|mut c| async move {
                redis::cmd("SET")
                    .arg(key)
                    .arg(json)
                    .arg("EX")
                    .arg(CACHE_TTL_SECS)
                    .query_async::<()>(&mut c)
                    .await
            })
            .await;
    }

    async fn cache_drop(&self, hashes: &[Vec<u8>]) {
        for hash in hashes {
            let key = Self::cache_key(hash);
            self.redis
                .run(|mut c| async move { redis::cmd("DEL").arg(key).query_async::<i64>(&mut c).await })
                .await;
        }
    }

    /// Writes the activity time (at most once a minute per session; ADR-0009 rule 3.3) and
    /// refreshes the cached record. A failure only means a later retry.
    pub async fn touch(
        &self,
        record: &SessionRecord,
        hash: &[u8],
        now: DateTime<Utc>,
    ) -> SessionRecord {
        if now - record.last_activity_at < ACTIVITY_WRITE_EVERY {
            return record.clone();
        }
        let result = async {
            let mut tx = db::begin(&self.db, &db::DbContext::user(record.user_id, None)).await?;
            sqlx::query(
                "UPDATE sessions SET last_activity_at = $2 WHERE id = $1 AND revoked_at IS NULL",
            )
            .bind(record.id)
            .bind(now)
            .execute(&mut *tx)
            .await?;
            tx.commit().await.map_err(db::Error::from)
        }
        .await;
        match result {
            Ok(()) => {
                let mut updated = record.clone();
                updated.last_activity_at = now;
                self.cache_put(hash, &updated).await;
                updated
            }
            Err(e) => {
                tracing::warn!(error = %e, "could not write session activity");
                record.clone()
            }
        }
    }

    /// One session of `user_id`.
    pub async fn get(&self, user_id: Uuid, session_id: Uuid) -> ApiResult<Option<SessionRecord>> {
        let mut tx = db::begin(&self.db, &db::DbContext::user(user_id, None)).await?;
        let row: Option<Row> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
            "SELECT {COLUMNS} FROM sessions WHERE id = $1"
        )))
        .bind(session_id)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        row.map(SessionRecord::try_from).transpose()
    }

    /// The user's open sessions, newest sign-in first.
    pub async fn list(&self, user_id: Uuid, now: DateTime<Utc>) -> ApiResult<Vec<SessionListItem>> {
        #[derive(sqlx::FromRow)]
        struct ListRow {
            #[sqlx(flatten)]
            row: Row,
            device_name: String,
            approx_location: Option<String>,
        }
        let mut tx = db::begin(&self.db, &db::DbContext::user(user_id, None)).await?;
        let rows: Vec<ListRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
            "SELECT {COLUMNS}, device_name, approx_location FROM sessions \
             WHERE revoked_at IS NULL AND expires_at > $1 ORDER BY created_at DESC, id DESC LIMIT 200"
        )))
        .bind(now)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        let mut out = Vec::new();
        for r in rows {
            let record = SessionRecord::try_from(r.row)?;
            if record.is_active(now) {
                out.push(SessionListItem {
                    record,
                    device_name: r.device_name,
                    approx_location: r.approx_location,
                });
            }
        }
        Ok(out)
    }

    /// Replaces `old` by a new session of the same client, device name and location, and closes
    /// `old` (after linking an identity; ADR-0009 rule 3.4).
    pub async fn rotate(&self, old: &SessionRecord) -> ApiResult<(String, SessionRecord)> {
        let mut tx = db::begin(&self.db, &db::DbContext::user(old.user_id, None)).await?;
        let (device_name, location): (String, Option<String>) =
            sqlx::query_as("SELECT device_name, approx_location FROM sessions WHERE id = $1")
                .bind(old.id)
                .fetch_one(&mut *tx)
                .await?;
        let (token, record) = insert(
            &mut tx,
            old.user_id,
            old.client_kind,
            old.remember,
            &device_name,
            location.as_deref(),
        )
        .await?;
        let hashes: Vec<(Vec<u8>,)> = sqlx::query_as(
            "UPDATE sessions SET revoked_at = now() WHERE id = $1 AND revoked_at IS NULL RETURNING token_hash",
        )
        .bind(old.id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        let hashes: Vec<Vec<u8>> = hashes.into_iter().map(|h| h.0).collect();
        self.cache_drop(&hashes).await;
        Ok((token, record))
    }

    /// Closes one session of `user_id`; `false` when there is no such open session.
    pub async fn revoke(&self, user_id: Uuid, session_id: Uuid) -> ApiResult<bool> {
        let mut tx = db::begin(&self.db, &db::DbContext::user(user_id, None)).await?;
        let hashes: Vec<(Vec<u8>,)> = sqlx::query_as(
            "UPDATE sessions SET revoked_at = now() WHERE id = $1 AND revoked_at IS NULL RETURNING token_hash",
        )
        .bind(session_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        let hashes: Vec<Vec<u8>> = hashes.into_iter().map(|h| h.0).collect();
        self.cache_drop(&hashes).await;
        Ok(!hashes.is_empty())
    }

    /// Closes every open session of `user_id`, except `keep` (sign-out everywhere, rotation).
    /// Returns how many were closed.
    pub async fn revoke_all(&self, user_id: Uuid, keep: Option<Uuid>) -> ApiResult<usize> {
        let mut tx = db::begin(&self.db, &db::DbContext::user(user_id, None)).await?;
        let hashes: Vec<(Vec<u8>,)> = sqlx::query_as(
            "UPDATE sessions SET revoked_at = now() \
             WHERE revoked_at IS NULL AND ($1::uuid IS NULL OR id <> $1) RETURNING token_hash",
        )
        .bind(keep)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        let hashes: Vec<Vec<u8>> = hashes.into_iter().map(|h| h.0).collect();
        self.cache_drop(&hashes).await;
        Ok(hashes.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(kind: ClientKind, remember: bool, now: DateTime<Utc>) -> SessionRecord {
        let (_, max) = lifetimes(kind, remember);
        SessionRecord {
            id: Uuid::now_v7(),
            user_id: Uuid::now_v7(),
            client_kind: kind,
            shared_device_id: None,
            remember,
            created_at: now,
            last_activity_at: now,
            expires_at: now + max,
            step_up_at: None,
            revoked_at: None,
        }
    }

    #[test]
    fn idle_and_absolute_limits_follow_the_table() {
        let t0 = Utc::now();
        let ios = record(ClientKind::Ios, false, t0);
        assert!(ios.is_active(t0 + Duration::days(29)));
        assert!(!ios.is_active(t0 + Duration::days(31)));
        let web = record(ClientKind::Web, false, t0);
        assert!(web.is_active(t0 + Duration::hours(11)));
        assert!(!web.is_active(t0 + Duration::hours(13)));
        let mut remembered = record(ClientKind::Web, true, t0);
        remembered.last_activity_at = t0 + Duration::days(29);
        assert!(remembered.is_active(t0 + Duration::days(29) + Duration::hours(1)));
        assert!(
            !remembered.is_active(t0 + Duration::days(31)),
            "past the 30 day maximum"
        );
        let mut closed = record(ClientKind::Ios, false, t0);
        closed.revoked_at = Some(t0);
        assert!(!closed.is_active(t0));
    }
}
