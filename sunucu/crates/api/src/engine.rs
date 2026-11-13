//! Client of `engine-service` (ADR-0005) with a version stamp, a Redis cache and the
//! `calc_results` record (ADR-0007).
//!
//! Every calculation goes through [`EngineClient::calculate`]:
//! 1. the engine version is read (`GetVersion`, remembered for a short time);
//! 2. the request is serialized canonically (protobuf wire bytes: fixed field order, default
//!    values omitted) and hashed with SHA-256;
//! 3. Redis is asked first (key holds workspace, kind, engine version and hash), then
//!    `calc_results`; only a miss in both calls the engine. Redis may be down: the call then
//!    goes straight on to PostgreSQL / the engine;
//! 4. a fresh result is stored in `calc_results` (unchanging record) and in Redis.
//!
//! Because the engine version is part of both keys, a new engine version never sees old
//! results. Engine errors become the API error envelope (see [`map_status`]).

use std::{
    fmt::Write as _,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use common::{Error, Result as CommonResult};
use prost::Message;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgConnection;
use tonic::{Code, Status, transport::Channel, transport::Endpoint};
use tonic_types::StatusExt;
use uuid::Uuid;

pub use engine_client::proto;

use crate::{ApiError, ApiResult, ErrorCode, error::FieldIssue, redis_client::Redis};

/// `calc_results.output` schema version written by this module.
pub const OUTPUT_SCHEMA_VERSION: u64 = 1;
/// `Retry-After` (seconds) sent with `503` when the engine does not answer.
pub const RETRY_AFTER_SECS: u64 = 5;

/// Engine connection settings.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// `ENGINE_URL`, e.g. `http://engine-service:50051`.
    pub url: String,
    /// Longest wait to connect.
    pub connect_timeout: Duration,
    /// Longest wait for one call.
    pub request_timeout: Duration,
    /// How long the engine version is remembered (`0` asks every time).
    pub version_ttl: Duration,
    /// Lifetime of a Redis cache entry.
    pub cache_ttl: Duration,
}

impl EngineConfig {
    /// Settings for `url` with default timeouts (connect 2 s, call 10 s, version 60 s, cache 24 h).
    #[must_use]
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            connect_timeout: Duration::from_secs(2),
            request_timeout: Duration::from_secs(10),
            version_ttl: Duration::from_mins(1),
            cache_ttl: Duration::from_hours(24),
        }
    }

    /// Reads `ENGINE_URL` (and `ENGINE_REQUEST_TIMEOUT_MS`); `None` when the URL is unset.
    pub fn from_env() -> CommonResult<Option<Self>> {
        Self::from_lookup(|n| std::env::var(n).ok())
    }

    /// Like [`Self::from_env`] with an injectable lookup.
    pub fn from_lookup<F: Fn(&str) -> Option<String>>(lookup: F) -> CommonResult<Option<Self>> {
        let Some(url) = lookup("ENGINE_URL").filter(|v| !v.trim().is_empty()) else {
            return Ok(None);
        };
        let mut cfg = Self::new(url.trim());
        if let Some(raw) = lookup("ENGINE_REQUEST_TIMEOUT_MS").filter(|v| !v.trim().is_empty()) {
            let ms: u64 =
                raw.trim()
                    .parse()
                    .map_err(|e: std::num::ParseIntError| Error::InvalidEnv {
                        name: "ENGINE_REQUEST_TIMEOUT_MS",
                        reason: e.to_string(),
                    })?;
            cfg.request_timeout = Duration::from_millis(ms);
        }
        Ok(Some(cfg))
    }
}

/// Where a result came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Redis cache.
    Redis,
    /// `calc_results`.
    Database,
    /// A fresh engine call.
    Engine,
}

/// What a result belongs to (`calc_results.recipe_version_id` / `batch_id`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalcTarget {
    /// Not tied to a record (tools, previews).
    None,
    /// A recipe version.
    RecipeVersion(Uuid),
    /// A batch.
    Batch(Uuid),
}

/// Who asks and where the result is kept.
#[derive(Debug, Clone)]
pub struct CalcScope {
    /// Workspace (RLS context must be the same).
    pub workspace_id: Uuid,
    /// Acting user, stored as `created_by`.
    pub user_id: Option<Uuid>,
    /// Record the result belongs to.
    pub target: CalcTarget,
    /// JSON Pointer prefix of the engine request inside the HTTP body (`""` when the body is the
    /// engine request; `"/recipe"` when nested), used for field errors.
    pub field_prefix: String,
}

impl CalcScope {
    /// A scope without target and prefix.
    #[must_use]
    pub fn new(workspace_id: Uuid, user_id: Option<Uuid>) -> Self {
        Self {
            workspace_id,
            user_id,
            target: CalcTarget::None,
            field_prefix: String::new(),
        }
    }
}

/// A calculation result with its provenance stamp.
#[derive(Debug, Clone)]
pub struct Calculated<T> {
    /// The engine response (all intermediate values the proto carries are in it).
    pub output: T,
    /// Engine version that produced it.
    pub engine_version: String,
    /// SHA-256 of the canonical request.
    pub input_hash: [u8; 32],
    /// Where it came from.
    pub source: Source,
}

struct Inner {
    channel: Channel,
    redis: Redis,
    config: EngineConfig,
    version: RwLock<Option<(String, Instant)>>,
}

/// Engine client; cheap to clone.
#[derive(Clone)]
pub struct EngineClient {
    inner: Option<Arc<Inner>>,
}

impl std::fmt::Debug for EngineClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineClient")
            .field("configured", &self.inner.is_some())
            .finish_non_exhaustive()
    }
}

/// Lower-case hex.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// SHA-256 of the canonical (protobuf wire) form of `message`.
#[must_use]
pub fn input_hash<M: Message>(message: &M) -> [u8; 32] {
    Sha256::digest(message.encode_to_vec()).into()
}

impl EngineClient {
    /// A client without an engine: every call answers `503 unavailable`.
    #[must_use]
    pub const fn disabled() -> Self {
        Self { inner: None }
    }

    /// Prepares a lazily connecting channel (reconnects by itself, multiplexes calls over one
    /// HTTP/2 connection). Fails only on an unparsable URL. Call inside the Tokio runtime.
    pub fn connect_lazy(config: EngineConfig, redis: Redis) -> CommonResult<Self> {
        let endpoint = Endpoint::from_shared(config.url.clone())
            .map_err(|e| Error::InvalidEnv {
                name: "ENGINE_URL",
                reason: e.to_string(),
            })?
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .tcp_nodelay(true)
            .http2_keep_alive_interval(Duration::from_secs(30))
            .keep_alive_while_idle(true);
        Ok(Self {
            inner: Some(Arc::new(Inner {
                channel: endpoint.connect_lazy(),
                redis,
                config,
                version: RwLock::new(None),
            })),
        })
    }

    /// The channel, for building `proto::*_client::*Client`s; `None` when disabled.
    #[must_use]
    pub fn channel(&self) -> Option<Channel> {
        self.inner.as_ref().map(|i| i.channel.clone())
    }

    fn inner(&self) -> ApiResult<&Arc<Inner>> {
        self.inner
            .as_ref()
            .ok_or_else(|| ApiError::unavailable(RETRY_AFTER_SECS))
    }

    /// The engine version. Remembered for `version_ttl`; when the engine does not answer, the
    /// last known version is used (so cached results stay readable), otherwise `503`.
    pub async fn engine_version(&self) -> ApiResult<String> {
        let inner = self.inner()?;
        let known = inner.version.read().ok().and_then(|g| g.clone());
        if let Some((v, at)) = &known
            && at.elapsed() < inner.config.version_ttl
        {
            return Ok(v.clone());
        }
        let mut client =
            proto::engine_service_client::EngineServiceClient::new(inner.channel.clone());
        match client.get_version(proto::GetVersionRequest {}).await {
            Ok(resp) => {
                let v = resp.into_inner().version;
                if let Ok(mut g) = inner.version.write() {
                    *g = Some((v.clone(), Instant::now()));
                }
                Ok(v)
            }
            Err(status) => match known {
                Some((v, _)) => {
                    tracing::warn!(%status, "engine version check failed; using last known");
                    Ok(v)
                }
                None => Err(map_status(&status, "")),
            },
        }
    }

    /// Runs (or fetches) one calculation. `call` performs the engine RPC with the channel;
    /// it is invoked only when neither Redis nor `calc_results` holds the result.
    ///
    /// `tx` must be the request transaction (RLS context of `scope.workspace_id`).
    pub async fn calculate<Req, Resp, F, Fut>(
        &self,
        tx: &mut PgConnection,
        scope: &CalcScope,
        kind: &str,
        request: Req,
        call: F,
    ) -> ApiResult<Calculated<Resp>>
    where
        Req: Message,
        Resp: Message + Default,
        F: FnOnce(Channel, Req) -> Fut,
        Fut: Future<Output = Result<tonic::Response<Resp>, Status>>,
    {
        if !valid_kind(kind) {
            tracing::error!(kind, "invalid calculation kind");
            return Err(ApiError::new(ErrorCode::Internal));
        }
        let inner = self.inner()?;
        let version = self.engine_version().await?;
        let hash = input_hash(&request);
        let key = format!(
            "calc:{}:{kind}:{version}:{}",
            scope.workspace_id,
            hex(&hash)
        );
        let hit = |output, source| Calculated {
            output,
            engine_version: version.clone(),
            input_hash: hash,
            source,
        };

        // 1. Redis.
        let cached = inner
            .redis
            .run(|mut c| {
                let key = key.clone();
                async move {
                    redis::cmd("GET")
                        .arg(key)
                        .query_async::<Option<Vec<u8>>>(&mut c)
                        .await
                }
            })
            .await
            .flatten();
        if let Some(bytes) = cached {
            match Resp::decode(bytes.as_slice()) {
                Ok(output) => return Ok(hit(output, Source::Redis)),
                Err(e) => tracing::warn!(error = %e, "undecodable calc cache entry; ignoring"),
            }
        }

        // 2. calc_results.
        let stored: Option<(Value,)> = sqlx::query_as(
            "SELECT output FROM calc_results
             WHERE workspace_id = $1 AND kind = $2 AND input_hash = $3 AND engine_version = $4",
        )
        .bind(scope.workspace_id)
        .bind(kind)
        .bind(hash.as_slice())
        .bind(&version)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db::Error::from)?;
        if let Some((output,)) = stored {
            if let Some(msg) = decode_output::<Resp>(&output) {
                self.cache_put(inner, &key, &msg).await;
                return Ok(hit(msg, Source::Database));
            }
            tracing::warn!(kind, "undecodable calc_results row; recomputing");
        }

        // 3. The engine.
        let response = call(inner.channel.clone(), request)
            .await
            .map_err(|s| map_status(&s, &scope.field_prefix))?
            .into_inner();
        let (recipe_version_id, batch_id) = match scope.target {
            CalcTarget::None => (None, None),
            CalcTarget::RecipeVersion(id) => (Some(id), None),
            CalcTarget::Batch(id) => (None, Some(id)),
        };
        sqlx::query(
            "INSERT INTO calc_results
                 (id, workspace_id, kind, input_hash, engine_version, output,
                  recipe_version_id, batch_id, created_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT (workspace_id, kind, input_hash, engine_version) DO NOTHING",
        )
        .bind(Uuid::now_v7())
        .bind(scope.workspace_id)
        .bind(kind)
        .bind(hash.as_slice())
        .bind(&version)
        .bind(encode_output(&response, &version))
        .bind(recipe_version_id)
        .bind(batch_id)
        .bind(scope.user_id)
        .execute(&mut *tx)
        .await
        .map_err(db::Error::from)?;
        self.cache_put(inner, &key, &response).await;
        Ok(hit(response, Source::Engine))
    }

    async fn cache_put<M: Message>(&self, inner: &Inner, key: &str, message: &M) {
        let bytes = message.encode_to_vec();
        let ttl = inner.config.cache_ttl.as_secs().max(1);
        inner
            .redis
            .run(|mut c| {
                let key = key.to_owned();
                async move {
                    redis::cmd("SET")
                        .arg(key)
                        .arg(bytes)
                        .arg("EX")
                        .arg(ttl)
                        .query_async::<()>(&mut c)
                        .await
                }
            })
            .await;
    }
}

fn valid_kind(kind: &str) -> bool {
    let mut chars = kind.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// `calc_results.output`: the response as base64 protobuf plus the stamp. The wire form keeps
/// every value of the engine response, including intermediate ("how it was calculated") ones.
fn encode_output<M: Message>(message: &M, engine_version: &str) -> Value {
    json!({
        "schema_version": OUTPUT_SCHEMA_VERSION,
        "encoding": "protobuf_base64",
        "engine_version": engine_version,
        "message": B64.encode(message.encode_to_vec()),
    })
}

fn decode_output<M: Message + Default>(output: &Value) -> Option<M> {
    if output.get("schema_version")?.as_u64()? != OUTPUT_SCHEMA_VERSION {
        return None;
    }
    let bytes = B64.decode(output.get("message")?.as_str()?).ok()?;
    M::decode(bytes.as_slice()).ok()
}

/// Converts an engine field name (`fermentables[0].amount_kg`) into a JSON Pointer under
/// `prefix` (`/fermentables/0/amount_kg`).
#[must_use]
pub fn field_pointer(prefix: &str, field: &str) -> String {
    let mut path = String::from(prefix);
    let mut cur = String::new();
    let flush = |cur: &mut String, path: &mut String| {
        if !cur.is_empty() {
            path.push('/');
            path.push_str(&cur.replace('~', "~0").replace('/', "~1"));
            cur.clear();
        }
    };
    for ch in field.chars() {
        match ch {
            '.' | '[' => flush(&mut cur, &mut path),
            ']' => {}
            c => cur.push(c),
        }
    }
    flush(&mut cur, &mut path);
    path
}

/// Maps an engine gRPC status to the API error (ADR-0005 item 5, ADR-0008 error table):
/// `INVALID_ARGUMENT` -> `422 calculation_failed` with one `errors[]` entry per field
/// violation (`invalid_input`, reason in `params.reason`); `FAILED_PRECONDITION` ->
/// `422 calculation_failed` (`no_convergence`); engine down, slow or overloaded -> `503
/// unavailable` with `Retry-After`; anything else -> `500 internal`.
#[must_use]
pub fn map_status(status: &Status, field_prefix: &str) -> ApiError {
    match status.code() {
        Code::InvalidArgument => {
            let details = status.get_error_details();
            let mut issues: Vec<FieldIssue> = details
                .bad_request()
                .map(|b| {
                    b.field_violations
                        .iter()
                        .map(|v| {
                            FieldIssue::new(field_pointer(field_prefix, &v.field), "invalid_input")
                                .with_params(json!({"reason": v.description}))
                        })
                        .collect()
                })
                .unwrap_or_default();
            if issues.is_empty() {
                issues.push(
                    FieldIssue::new(field_prefix, "invalid_input")
                        .with_params(json!({"reason": status.message()})),
                );
            }
            ApiError::new(ErrorCode::CalculationFailed).with_errors(issues)
        }
        Code::FailedPrecondition => ApiError::new(ErrorCode::CalculationFailed).with_errors(vec![
            FieldIssue::new(field_prefix, "no_convergence")
                .with_params(json!({"reason": status.message()})),
        ]),
        Code::Unavailable
        | Code::DeadlineExceeded
        | Code::Cancelled
        | Code::ResourceExhausted
        | Code::Aborted => {
            tracing::warn!(code = ?status.code(), message = status.message(), "engine unavailable");
            ApiError::unavailable(RETRY_AFTER_SECS)
        }
        code => {
            tracing::error!(?code, message = status.message(), "engine call failed");
            ApiError::new(ErrorCode::Internal)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_conversion() {
        assert_eq!(field_pointer("", "sg"), "/sg");
        assert_eq!(
            field_pointer("/recipe", "fermentables[2].amount_kg"),
            "/recipe/fermentables/2/amount_kg"
        );
        assert_eq!(field_pointer("", ""), "");
    }

    #[test]
    fn kind_names() {
        assert!(valid_kind("recipe_summary"));
        assert!(!valid_kind("Recipe"));
        assert!(!valid_kind(""));
    }

    #[test]
    fn status_mapping() {
        let e = map_status(&Status::unavailable("down"), "");
        assert_eq!(e.code(), ErrorCode::Unavailable);
        let e = map_status(&Status::deadline_exceeded("slow"), "");
        assert_eq!(e.code(), ErrorCode::Unavailable);
        let e = map_status(&Status::failed_precondition("x"), "/r");
        assert_eq!(e.code(), ErrorCode::CalculationFailed);
        assert_eq!(e.errors()[0].code, "no_convergence");
        let e = map_status(&Status::internal("x"), "");
        assert_eq!(e.code(), ErrorCode::Internal);
    }

    #[test]
    fn config_lookup() {
        assert!(EngineConfig::from_lookup(|_| None).unwrap().is_none());
        let c = EngineConfig::from_lookup(|n| match n {
            "ENGINE_URL" => Some("http://e:1".into()),
            "ENGINE_REQUEST_TIMEOUT_MS" => Some("1500".into()),
            _ => None,
        })
        .unwrap()
        .unwrap();
        assert_eq!(c.request_timeout, Duration::from_millis(1500));
    }
}
