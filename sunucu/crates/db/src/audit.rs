//! Append-only audit log with a SHA-256 hash chain per workspace (ADR-0007 rule 5).
//!
//! Each row stores the hash of the previous row (`prev_hash`) and its own
//! `hash = SHA-256(prev_hash || canonical row)`. The canonical row is the RFC 8785 (JCS)
//! serialisation of a fixed JSON object built by [`canonical_row`]. The first row of a
//! workspace chains to [`GENESIS_HASH`]. Appends take the workspace's `audit_heads` row with
//! `FOR UPDATE`, so concurrent writers of one workspace queue up and never fork the chain;
//! other workspaces are not blocked.
//!
//! Hashing and verification live here, in Rust; the database only keeps shape, order and
//! the no-UPDATE/DELETE rule (migration 0006).

use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{PgConnection, Row, types::Uuid};

use crate::{ActorKind, Error, Result, set_workspace_id};

/// `prev_hash` of the first row of every chain.
pub const GENESIS_HASH: [u8; 32] = [0; 32];

/// Rows read per round trip while verifying.
const VERIFY_BATCH: i64 = 1000;

/// Audit event kinds (the `action` column). Which events are written is fixed by ADR-0007
/// rule 5.1 and ADR-0009; endpoints call [`append_audit`] with one of these.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditEvent {
    /// A role was created.
    RoleCreate,
    /// A role was renamed or its settings changed.
    RoleUpdate,
    /// A role was deleted.
    RoleDelete,
    /// The permission set of a role changed.
    RolePermissionsSet,
    /// A role was assigned to a member.
    RoleAssign,
    /// A role assignment was removed.
    RoleUnassign,
    /// A member joined, was disabled or re-enabled.
    MemberStatusChange,
    /// A member was removed from the workspace.
    MemberRemove,
    /// An invitation was created.
    InvitationCreate,
    /// An invitation was revoked.
    InvitationRevoke,
    /// A closed batch was corrected (reason required).
    BatchClosedCorrection,
    /// A recipe version was approved.
    RecipeApprove,
    /// A recipe was archived.
    RecipeArchive,
    /// A record was soft-deleted.
    RecordDelete,
    /// A soft-deleted record was restored.
    RecordRestore,
    /// A subscription was corrected by hand (reason required).
    SubscriptionManualAdjust,
    /// An entitlement was corrected by hand (reason required).
    EntitlementManualAdjust,
    /// A support grant was given, revoked or used.
    SupportGrantChange,
    /// A support staff action (one row per `support_actions` row; reason required).
    SupportAction,
    /// A sign-in method was added.
    LoginMethodAdd,
    /// A sign-in method was removed.
    LoginMethodRemove,
    /// A passkey was added or removed.
    PasskeyChange,
    /// A session was closed.
    SessionRevoke,
    /// A device key was renewed.
    DeviceKeyRenew,
    /// Data export was requested.
    DataExport,
    /// Account deletion was requested or carried out.
    AccountDelete,
}

impl AuditEvent {
    /// Every event kind.
    pub const ALL: [Self; 26] = [
        Self::RoleCreate,
        Self::RoleUpdate,
        Self::RoleDelete,
        Self::RolePermissionsSet,
        Self::RoleAssign,
        Self::RoleUnassign,
        Self::MemberStatusChange,
        Self::MemberRemove,
        Self::InvitationCreate,
        Self::InvitationRevoke,
        Self::BatchClosedCorrection,
        Self::RecipeApprove,
        Self::RecipeArchive,
        Self::RecordDelete,
        Self::RecordRestore,
        Self::SubscriptionManualAdjust,
        Self::EntitlementManualAdjust,
        Self::SupportGrantChange,
        Self::SupportAction,
        Self::LoginMethodAdd,
        Self::LoginMethodRemove,
        Self::PasskeyChange,
        Self::SessionRevoke,
        Self::DeviceKeyRenew,
        Self::DataExport,
        Self::AccountDelete,
    ];

    /// The value stored in `audit_log.action`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RoleCreate => "role.create",
            Self::RoleUpdate => "role.update",
            Self::RoleDelete => "role.delete",
            Self::RolePermissionsSet => "role.permissions_set",
            Self::RoleAssign => "role.assign",
            Self::RoleUnassign => "role.unassign",
            Self::MemberStatusChange => "member.status_change",
            Self::MemberRemove => "member.remove",
            Self::InvitationCreate => "invitation.create",
            Self::InvitationRevoke => "invitation.revoke",
            Self::BatchClosedCorrection => "batch.closed_correction",
            Self::RecipeApprove => "recipe.approve",
            Self::RecipeArchive => "recipe.archive",
            Self::RecordDelete => "record.delete",
            Self::RecordRestore => "record.restore",
            Self::SubscriptionManualAdjust => "subscription.manual_adjust",
            Self::EntitlementManualAdjust => "entitlement.manual_adjust",
            Self::SupportGrantChange => "support.grant_change",
            Self::SupportAction => "support.action",
            Self::LoginMethodAdd => "login_method.add",
            Self::LoginMethodRemove => "login_method.remove",
            Self::PasskeyChange => "passkey.change",
            Self::SessionRevoke => "session.revoke",
            Self::DeviceKeyRenew => "device_key.renew",
            Self::DataExport => "data.export",
            Self::AccountDelete => "account.delete",
        }
    }

    /// Whether the event must carry a non-empty reason (ADR-0007 rule 5.1: corrections of
    /// closed batches, manual corrections, support actions).
    #[must_use]
    pub const fn requires_reason(self) -> bool {
        matches!(
            self,
            Self::BatchClosedCorrection
                | Self::SubscriptionManualAdjust
                | Self::EntitlementManualAdjust
                | Self::SupportAction
        )
    }
}

/// An audit row to append. Personal data (names, e-mail, free text) must not go into the
/// values: use [`changed_field`] for such fields (ADR-0007 rule 5.5).
#[derive(Debug, Clone)]
pub struct NewAuditEntry {
    /// Who acts.
    pub actor_kind: ActorKind,
    /// The acting user or staff member, `None` for the system.
    pub actor_id: Option<Uuid>,
    /// What happened.
    pub event: AuditEvent,
    /// Kind of the affected record (a table-like name, at most 100 characters).
    pub record_kind: String,
    /// Identifier of the affected record.
    pub record_id: Option<Uuid>,
    /// Value before the change.
    pub previous_value: Option<Value>,
    /// Value after the change.
    pub new_value: Option<Value>,
    /// Justification, mandatory for some events.
    pub reason: Option<String>,
}

/// The chain position an append produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditRecord {
    /// Row identifier.
    pub id: Uuid,
    /// Sequence number within the workspace, starting at 1.
    pub seq: i64,
    /// Hash of this row.
    pub hash: [u8; 32],
}

/// Placeholder for a personal-data field: marks it as changed and nothing else. No digest of
/// the value is written: names and e-mail addresses have low entropy, so a digest could be
/// reversed with a dictionary (F2.1 finding 9).
#[must_use]
pub fn changed_field() -> Value {
    json!({ "changed": true })
}

/// Everything the hash covers, in the fixed shape used for the chain.
#[derive(Debug)]
pub struct CanonicalFields<'a> {
    /// Row identifier.
    pub id: Uuid,
    /// Workspace.
    pub workspace_id: Uuid,
    /// Sequence number.
    pub seq: i64,
    /// Creation time (microsecond precision, as stored).
    pub created_at: DateTime<Utc>,
    /// `actor_kind` column.
    pub actor_kind: &'a str,
    /// `actor_id` column.
    pub actor_id: Option<Uuid>,
    /// `action` column.
    pub action: &'a str,
    /// `record_kind` column.
    pub record_kind: &'a str,
    /// `record_id` column.
    pub record_id: Option<Uuid>,
    /// `previous_value` column.
    pub previous_value: Option<&'a Value>,
    /// `new_value` column.
    pub new_value: Option<&'a Value>,
    /// `reason` column.
    pub reason: Option<&'a str>,
}

/// The canonical (RFC 8785) bytes of a row. Absent values are JSON `null`; `created_at` is
/// RFC 3339 UTC with six fractional digits.
pub fn canonical_row(row: &CanonicalFields<'_>) -> Result<Vec<u8>> {
    let value = json!({
        "id": row.id,
        "workspace_id": row.workspace_id,
        "seq": row.seq,
        "created_at": row.created_at.to_rfc3339_opts(SecondsFormat::Micros, true),
        "actor_kind": row.actor_kind,
        "actor_id": row.actor_id,
        "action": row.action,
        "record_kind": row.record_kind,
        "record_id": row.record_id,
        "previous_value": row.previous_value,
        "new_value": row.new_value,
        "reason": row.reason,
    });
    serde_json_canonicalizer::to_vec(&value).map_err(|e| Error::AuditCanonical(e.to_string()))
}

/// `SHA-256(prev_hash || canonical)`.
#[must_use]
pub fn chain_hash(prev_hash: &[u8; 32], canonical: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(prev_hash);
    hasher.update(canonical);
    hasher.finalize().into()
}

fn to_hash(bytes: Vec<u8>) -> Result<[u8; 32]> {
    bytes
        .try_into()
        .map_err(|_| Error::AuditCanonical("hash is not 32 bytes".into()))
}

/// Appends an entry to the workspace's chain.
///
/// Must run inside the request transaction (`conn` is a transaction connection with the
/// workspace context set, see [`crate::begin`]): the head row stays locked until commit, so a
/// rolled-back request leaves neither a row nor a gap, and the audit row commits together
/// with the change it describes.
pub async fn append_audit(
    conn: &mut PgConnection,
    workspace_id: Uuid,
    entry: &NewAuditEntry,
) -> Result<AuditRecord> {
    if entry.record_kind.is_empty() || entry.record_kind.chars().count() > 100 {
        return Err(Error::InvalidAuditEntry(
            "record_kind must be 1-100 characters",
        ));
    }
    if entry.event.requires_reason() && entry.reason.as_deref().is_none_or(|r| r.trim().is_empty())
    {
        return Err(Error::InvalidAuditEntry("this event requires a reason"));
    }

    sqlx::query(
        "INSERT INTO audit_heads (workspace_id, last_seq, last_hash) VALUES ($1, 0, $2)
         ON CONFLICT (workspace_id) DO NOTHING",
    )
    .bind(workspace_id)
    .bind(GENESIS_HASH.as_slice())
    .execute(&mut *conn)
    .await?;
    let head = sqlx::query(
        "SELECT last_seq, last_hash FROM audit_heads WHERE workspace_id = $1 FOR UPDATE",
    )
    .bind(workspace_id)
    .fetch_one(&mut *conn)
    .await?;
    let last_seq: i64 = head.try_get("last_seq")?;
    let prev_hash = to_hash(head.try_get("last_hash")?)?;

    let id = Uuid::now_v7();
    let seq = last_seq + 1;
    // The database stores microseconds; truncate here so the hash survives the round trip.
    let created_at = DateTime::<Utc>::from_timestamp_micros(Utc::now().timestamp_micros())
        .ok_or_else(|| Error::AuditCanonical("timestamp out of range".into()))?;
    let canonical = canonical_row(&CanonicalFields {
        id,
        workspace_id,
        seq,
        created_at,
        actor_kind: entry.actor_kind.as_str(),
        actor_id: entry.actor_id,
        action: entry.event.as_str(),
        record_kind: &entry.record_kind,
        record_id: entry.record_id,
        previous_value: entry.previous_value.as_ref(),
        new_value: entry.new_value.as_ref(),
        reason: entry.reason.as_deref(),
    })?;
    let hash = chain_hash(&prev_hash, &canonical);

    sqlx::query(
        "INSERT INTO audit_log (id, workspace_id, seq, created_at, actor_kind, actor_id, action,
                                record_kind, record_id, previous_value, new_value, reason,
                                prev_hash, hash)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
    )
    .bind(id)
    .bind(workspace_id)
    .bind(seq)
    .bind(created_at)
    .bind(entry.actor_kind.as_str())
    .bind(entry.actor_id)
    .bind(entry.event.as_str())
    .bind(&entry.record_kind)
    .bind(entry.record_id)
    .bind(entry.previous_value.as_ref())
    .bind(entry.new_value.as_ref())
    .bind(entry.reason.as_deref())
    .bind(prev_hash.as_slice())
    .bind(hash.as_slice())
    .execute(&mut *conn)
    .await?;
    sqlx::query(
        "UPDATE audit_heads SET last_seq = $2, last_hash = $3, updated_at = now()
         WHERE workspace_id = $1",
    )
    .bind(workspace_id)
    .bind(seq)
    .bind(hash.as_slice())
    .execute(&mut *conn)
    .await?;
    Ok(AuditRecord { id, seq, hash })
}

/// Where and how a chain is broken.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainBreak {
    /// Rows exist but `audit_heads` has no row for the workspace.
    MissingHead,
    /// A sequence number is missing or duplicated (a row was deleted or inserted).
    SeqGap {
        /// The sequence number the chain required.
        expected: i64,
        /// The sequence number found.
        found: i64,
    },
    /// `prev_hash` of a row differs from the hash of the row before it.
    PrevHashMismatch {
        /// Sequence number of the offending row.
        seq: i64,
    },
    /// The stored hash differs from the recomputed one (the row content changed).
    HashMismatch {
        /// Sequence number of the offending row.
        seq: i64,
    },
    /// The head does not point at the last row (rows were cut off or appended around the head).
    HeadMismatch {
        /// `last_seq` in `audit_heads`.
        head_seq: i64,
        /// Sequence number of the last row found (0 when none).
        last_row_seq: i64,
    },
}

impl std::fmt::Display for ChainBreak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingHead => write!(f, "audit_heads row is missing"),
            Self::SeqGap { expected, found } => {
                write!(f, "sequence gap: expected {expected}, found {found}")
            }
            Self::PrevHashMismatch { seq } => write!(f, "row {seq}: prev_hash does not match"),
            Self::HashMismatch { seq } => write!(f, "row {seq}: hash does not match content"),
            Self::HeadMismatch {
                head_seq,
                last_row_seq,
            } => write!(
                f,
                "head points at {head_seq} but the last row is {last_row_seq}"
            ),
        }
    }
}

/// Result of verifying one workspace's chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainReport {
    /// The workspace.
    pub workspace_id: Uuid,
    /// Rows checked before the verification stopped.
    pub rows: i64,
    /// The first break found, `None` when the chain is intact.
    pub broken: Option<ChainBreak>,
}

impl ChainReport {
    /// Whether the chain is intact.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        self.broken.is_none()
    }
}

/// Recomputes and checks one workspace's chain. The connection must be able to read the
/// workspace's rows: the workspace context is set here, and the owner/superuser migration
/// role bypasses RLS anyway.
pub async fn verify_chain(conn: &mut PgConnection, workspace_id: Uuid) -> Result<ChainReport> {
    let mut tx = sqlx::Connection::begin(&mut *conn).await?;
    set_workspace_id(&mut tx, workspace_id).await?;

    let head = sqlx::query("SELECT last_seq, last_hash FROM audit_heads WHERE workspace_id = $1")
        .bind(workspace_id)
        .fetch_optional(&mut *tx)
        .await?;

    let mut expected_seq: i64 = 1;
    let mut prev_hash = GENESIS_HASH;
    let mut rows: i64 = 0;
    let mut broken = None;
    let mut after_seq: i64 = 0;
    'scan: loop {
        let batch = sqlx::query(
            "SELECT id, seq, created_at, actor_kind, actor_id, action, record_kind, record_id,
                    previous_value, new_value, reason, prev_hash, hash
             FROM audit_log WHERE workspace_id = $1 AND seq > $2 ORDER BY seq LIMIT $3",
        )
        .bind(workspace_id)
        .bind(after_seq)
        .bind(VERIFY_BATCH)
        .fetch_all(&mut *tx)
        .await?;
        if batch.is_empty() {
            break;
        }
        for row in &batch {
            rows += 1;
            let seq: i64 = row.try_get("seq")?;
            after_seq = seq;
            if seq != expected_seq {
                broken = Some(ChainBreak::SeqGap {
                    expected: expected_seq,
                    found: seq,
                });
                break 'scan;
            }
            let stored_prev = to_hash(row.try_get("prev_hash")?)?;
            if stored_prev != prev_hash {
                broken = Some(ChainBreak::PrevHashMismatch { seq });
                break 'scan;
            }
            let stored_hash = to_hash(row.try_get("hash")?)?;
            let previous_value: Option<Value> = row.try_get("previous_value")?;
            let new_value: Option<Value> = row.try_get("new_value")?;
            let actor_kind: String = row.try_get("actor_kind")?;
            let action: String = row.try_get("action")?;
            let record_kind: String = row.try_get("record_kind")?;
            let reason: Option<String> = row.try_get("reason")?;
            let canonical = canonical_row(&CanonicalFields {
                id: row.try_get("id")?,
                workspace_id,
                seq,
                created_at: row.try_get("created_at")?,
                actor_kind: &actor_kind,
                actor_id: row.try_get("actor_id")?,
                action: &action,
                record_kind: &record_kind,
                record_id: row.try_get("record_id")?,
                previous_value: previous_value.as_ref(),
                new_value: new_value.as_ref(),
                reason: reason.as_deref(),
            })?;
            if chain_hash(&prev_hash, &canonical) != stored_hash {
                broken = Some(ChainBreak::HashMismatch { seq });
                break 'scan;
            }
            prev_hash = stored_hash;
            expected_seq += 1;
        }
    }

    if broken.is_none() {
        let last_row_seq = expected_seq - 1;
        broken = match head {
            None if rows > 0 => Some(ChainBreak::MissingHead),
            None => None,
            Some(head) => {
                let head_seq: i64 = head.try_get("last_seq")?;
                let head_hash = to_hash(head.try_get("last_hash")?)?;
                (head_seq != last_row_seq || head_hash != prev_hash).then_some(
                    ChainBreak::HeadMismatch {
                        head_seq,
                        last_row_seq,
                    },
                )
            }
        };
    }
    tx.rollback().await?;
    Ok(ChainReport {
        workspace_id,
        rows,
        broken,
    })
}

/// Verifies every workspace that has audit rows or a head. The connection must bypass row
/// level security (superuser or the migration role), otherwise the workspaces cannot be
/// listed; this is refused ([`Error::VerifyNeedsBypass`]) rather than reported as an empty
/// success.
pub async fn verify_all_chains(conn: &mut PgConnection) -> Result<Vec<ChainReport>> {
    let bypass: bool = sqlx::query_scalar(
        "SELECT rolsuper OR rolbypassrls FROM pg_roles WHERE rolname = current_user",
    )
    .fetch_one(&mut *conn)
    .await?;
    if !bypass {
        return Err(Error::VerifyNeedsBypass);
    }
    let workspaces: Vec<Uuid> = sqlx::query_scalar(
        "SELECT workspace_id FROM audit_heads UNION SELECT workspace_id FROM audit_log
         ORDER BY 1",
    )
    .fetch_all(&mut *conn)
    .await?;
    let mut reports = Vec::with_capacity(workspaces.len());
    for ws in workspaces {
        reports.push(verify_chain(conn, ws).await?);
    }
    Ok(reports)
}
