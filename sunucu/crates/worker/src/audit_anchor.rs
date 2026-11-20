//! External anchor of the audit chains (T2.39, F2.1 decision 12).
//!
//! An attacker with database access can delete rows and the chain head together; the chain
//! check alone cannot see that. So each day the worker writes the head of every chain and the
//! workspace deletion records to a retention-locked area of the object store (prefix
//! [`DIGEST_PREFIX`]). Each week it checks the chains against these digests.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use db::maintenance::{AuditHead, Tombstone};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, types::Uuid};
use storage::ObjectStore;

use crate::{Error, Result};

/// Key prefix of the digest objects. Nothing else writes under it.
pub const DIGEST_PREFIX: &str = "audit-digests/";
/// How long a digest object stays locked (about 13 months, as the longest retention of the
/// decisions it protects). To be verified with the product owner.
pub const DIGEST_RETENTION_DAYS: i64 = 400;
/// Days (back from today) that the weekly check reads. One more than seven covers a missed run.
pub const VERIFY_WINDOW_DAYS: i64 = 8;
/// Largest digest object the worker reads.
const MAX_DIGEST_BYTES: u64 = 64 * 1024 * 1024;
/// Version of the digest format.
const FORMAT_VERSION: u32 = 1;

/// One chain head in a digest object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadEntry {
    /// The workspace.
    pub workspace_id: Uuid,
    /// Sequence number of the last row.
    pub last_seq: i64,
    /// Hash of the last row, in hex.
    pub last_hash: String,
}

/// One deletion record in a digest object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeletionEntry {
    /// The deleted workspace.
    pub former_workspace_id: Uuid,
    /// When it was purged.
    pub deleted_at: DateTime<Utc>,
    /// Sequence number of the last audit row at purge time.
    pub audit_last_seq: i64,
    /// Hash of the last audit row at purge time, in hex.
    pub audit_last_hash: String,
}

/// The content of one digest object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Digest {
    /// Format version.
    pub format_version: u32,
    /// When the digest was taken.
    pub created_at: DateTime<Utc>,
    /// Chain heads.
    pub heads: Vec<HeadEntry>,
    /// Workspace deletion records.
    pub deletions: Vec<DeletionEntry>,
}

/// Object key of the digest of a day.
#[must_use]
pub fn digest_key(day: NaiveDate) -> String {
    format!("{DIGEST_PREFIX}{day}.json")
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::with_capacity(64), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

impl Digest {
    fn new(now: DateTime<Utc>, heads: &[AuditHead], tombstones: &[Tombstone]) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            created_at: now,
            heads: heads
                .iter()
                .map(|h| HeadEntry {
                    workspace_id: h.workspace_id,
                    last_seq: h.last_seq,
                    last_hash: hex(&h.last_hash),
                })
                .collect(),
            deletions: tombstones
                .iter()
                .map(|t| DeletionEntry {
                    former_workspace_id: t.former_workspace_id,
                    deleted_at: t.deleted_at,
                    audit_last_seq: t.audit_last_seq,
                    audit_last_hash: hex(&t.audit_last_hash),
                })
                .collect(),
        }
    }
}

/// What the daily job did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DigestOutcome {
    /// A new digest object was written.
    Written {
        /// Its key.
        key: String,
        /// Number of chain heads in it.
        heads: usize,
    },
    /// The digest of this day already exists; the job changed nothing.
    Exists {
        /// Its key.
        key: String,
    },
}

/// Writes the digest of today (UTC), locked until [`DIGEST_RETENTION_DAYS`] pass. A digest
/// that exists is not written again: a locked object cannot be replaced.
pub async fn write_daily_digest(
    pool: &PgPool,
    store: &dyn ObjectStore,
    now: DateTime<Utc>,
) -> Result<DigestOutcome> {
    let key = digest_key(now.date_naive());
    if store.head(&key).await?.is_some() {
        return Ok(DigestOutcome::Exists { key });
    }
    let heads = db::maintenance::list_audit_heads(pool).await?;
    let tombstones = db::maintenance::list_tombstones(pool).await?;
    let digest = Digest::new(now, &heads, &tombstones);
    let body = serde_json::to_vec(&digest).map_err(|e| Error::Format(e.to_string()))?;
    store
        .put_locked(
            &key,
            "application/json",
            body,
            now + Duration::days(DIGEST_RETENTION_DAYS),
        )
        .await?;
    Ok(DigestOutcome::Written {
        key,
        heads: heads.len(),
    })
}

/// Result of the weekly check.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VerifyReport {
    /// Chains whose internal hashes were recomputed.
    pub chains_verified: usize,
    /// Digest objects that were read and compared.
    pub digests_checked: usize,
    /// Days in the window without a digest object.
    pub digests_missing: usize,
    /// What is wrong. Empty means the check passed.
    pub findings: Vec<String>,
}

impl VerifyReport {
    /// Whether the check found nothing wrong.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.findings.is_empty()
    }
}

fn parse_hash(text: &str) -> Option<[u8; 32]> {
    if text.len() != 64 || !text.is_ascii() {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[2 * i..2 * i + 2], 16).ok()?;
    }
    Some(out)
}

/// Verifies every chain, then compares the digests of the last [`VERIFY_WINDOW_DAYS`] days
/// with the database.
///
/// - Each chain must hash correctly from its first row to its head.
/// - Each digested head row must still exist with the same hash, or the workspace must have a
///   deletion record that agrees.
/// - Each digested deletion record must still exist unchanged.
///
/// A storage or database failure is an error. A mismatch is a finding in the report.
pub async fn verify_against_digests(
    pool: &PgPool,
    store: &dyn ObjectStore,
    now: DateTime<Utc>,
) -> Result<VerifyReport> {
    let mut report = VerifyReport::default();

    for workspace_id in &db::maintenance::list_workspace_ids(pool).await? {
        let mut conn = pool.acquire().await.map_err(db::Error::from)?;
        let chain = db::verify_chain(&mut conn, *workspace_id).await?;
        report.chains_verified += 1;
        if let Some(broken) = chain.broken {
            report.findings.push(format!(
                "chain of workspace {workspace_id} is broken: {broken}"
            ));
        }
    }

    let mut tombstones: Option<Vec<Tombstone>> = None;
    for back in 0..=VERIFY_WINDOW_DAYS {
        let day = (now - Duration::days(back)).date_naive();
        let key = digest_key(day);
        let Some(bytes) = store.get(&key, MAX_DIGEST_BYTES).await? else {
            report.digests_missing += 1;
            continue;
        };
        let digest: Digest = match serde_json::from_slice(&bytes) {
            Ok(d) => d,
            Err(e) => {
                report
                    .findings
                    .push(format!("digest {key} cannot be read: {e}"));
                continue;
            }
        };
        report.digests_checked += 1;
        if tombstones.is_none() {
            tombstones = Some(db::maintenance::list_tombstones(pool).await?);
        }
        let current = tombstones.as_deref().unwrap_or_default();
        check_heads(pool, &digest, &key, current, &mut report).await?;
        check_deletions(&digest, &key, current, &mut report);
    }
    Ok(report)
}

async fn check_heads(
    pool: &PgPool,
    digest: &Digest,
    key: &str,
    tombstones: &[Tombstone],
    report: &mut VerifyReport,
) -> Result<()> {
    for head in &digest.heads {
        let Some(expected) = parse_hash(&head.last_hash) else {
            report.findings.push(format!(
                "digest {key}: bad hash for workspace {}",
                head.workspace_id
            ));
            continue;
        };
        if head.last_seq == 0 {
            continue; // No row yet: nothing to compare.
        }
        match db::maintenance::audit_hash_at(pool, head.workspace_id, head.last_seq).await? {
            Some(found) if found == expected => {}
            Some(_) => report.findings.push(format!(
                "digest {key}: row {} of workspace {} changed",
                head.last_seq, head.workspace_id
            )),
            None => {
                let record = tombstones
                    .iter()
                    .find(|t| t.former_workspace_id == head.workspace_id);
                let agrees = record.is_some_and(|t| {
                    t.audit_last_seq > head.last_seq
                        || (t.audit_last_seq == head.last_seq && t.audit_last_hash == expected)
                });
                if !agrees {
                    report.findings.push(format!(
                        "digest {key}: row {} of workspace {} is missing and no deletion record explains it",
                        head.last_seq, head.workspace_id
                    ));
                }
            }
        }
    }
    Ok(())
}

fn check_deletions(
    digest: &Digest,
    key: &str,
    tombstones: &[Tombstone],
    report: &mut VerifyReport,
) {
    for entry in &digest.deletions {
        let same = tombstones.iter().any(|t| {
            t.former_workspace_id == entry.former_workspace_id
                && t.audit_last_seq == entry.audit_last_seq
                && hex(&t.audit_last_hash) == entry.audit_last_hash
        });
        if !same {
            report.findings.push(format!(
                "digest {key}: deletion record of workspace {} is missing or changed",
                entry.former_workspace_id
            ));
        }
    }
}
