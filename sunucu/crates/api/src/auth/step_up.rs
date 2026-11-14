//! Second step for sensitive operations (ADR-0009 rule 5.3 to 5.5).
//!
//! A successful passkey check writes `step_up_at` on the session (see [`super::passkey`]); it
//! stays valid for [`STEP_UP_TTL`]. [`AuthServices::require_step_up`] is what a protected
//! handler calls. It is kept apart from the permission list on purpose: the permission matrix
//! says who may do something, this says how freshly the caller proved to be that person.
//!
//! Rules:
//! - A member with `users.manage` must own a passkey; without one every protected operation is
//!   `403 step_up_required` with the field code `passkey_enrollment_required`.
//! - A user with a passkey needs a `step_up_at` not older than 15 minutes (`passkey_required`).
//! - A home user without a passkey may instead have signed in within the last 10 minutes
//!   (`reauthentication_required` otherwise).

use chrono::{DateTime, Duration, Utc};

use super::{AuthServices, session::SessionRecord};
use crate::{
    context::RequestContext,
    error::{ApiError, ApiResult, ErrorCode, FieldIssue},
};

/// How long a passkey check counts as a second step (ADR-0009 rule 5.3).
pub const STEP_UP_TTL: Duration = Duration::minutes(15);
/// "Signed in within the last 10 minutes" for users without a passkey (rule 5.5).
pub const RECENT_SIGN_IN: Duration = Duration::minutes(10);
/// Permission whose holders must own a passkey (rule 5.4).
pub const PASSKEY_REQUIRED_PERMISSION: &str = "users.manage";

/// Operations that ask for the second step (ADR-0009 rule 5.3). Handlers name theirs so the
/// log shows what was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtectedAction {
    /// User, role and permission changes.
    UserChange,
    /// Sending an invitation.
    InviteSend,
    /// Subscription changes.
    Subscription,
    /// Deleting the workspace.
    WorkspaceDelete,
    /// Deleting the account.
    AccountDelete,
    /// Opening shared device mode.
    SharedDeviceOpen,
    /// Renewing a shared device key.
    DeviceKeyRotate,
    /// Granting support access.
    SupportGrant,
    /// Removing a sign-in method.
    IdentityRemove,
    /// Adding or removing a passkey.
    PasskeyChange,
    /// Full export.
    FullExport,
}

impl ProtectedAction {
    /// Stable name for logs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UserChange => "user_change",
            Self::InviteSend => "invite_send",
            Self::Subscription => "subscription",
            Self::WorkspaceDelete => "workspace_delete",
            Self::AccountDelete => "account_delete",
            Self::SharedDeviceOpen => "shared_device_open",
            Self::DeviceKeyRotate => "device_key_rotate",
            Self::SupportGrant => "support_grant",
            Self::IdentityRemove => "identity_remove",
            Self::PasskeyChange => "passkey_change",
            Self::FullExport => "full_export",
        }
    }
}

/// Why the second step is missing; the field code of the `403`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepUpGap {
    /// A passkey must be registered first.
    PasskeyEnrollmentRequired,
    /// A fresh passkey check is needed.
    PasskeyRequired,
    /// A fresh sign-in is needed (no passkey).
    ReauthenticationRequired,
}

impl StepUpGap {
    /// The code in `errors[0].code`.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::PasskeyEnrollmentRequired => "passkey_enrollment_required",
            Self::PasskeyRequired => "passkey_required",
            Self::ReauthenticationRequired => "reauthentication_required",
        }
    }
}

/// The pure rule: `None` when the session may do a protected operation at `now`.
#[must_use]
pub fn step_up_gap(
    record: &SessionRecord,
    has_passkey: bool,
    needs_passkey: bool,
    now: DateTime<Utc>,
) -> Option<StepUpGap> {
    let stepped = record.step_up_at.is_some_and(|t| now - t < STEP_UP_TTL);
    if needs_passkey && !has_passkey {
        return Some(StepUpGap::PasskeyEnrollmentRequired);
    }
    if has_passkey {
        return (!stepped).then_some(StepUpGap::PasskeyRequired);
    }
    let signed_in_recently = now - record.created_at < RECENT_SIGN_IN;
    (!(stepped || signed_in_recently)).then_some(StepUpGap::ReauthenticationRequired)
}

/// The `403 step_up_required` answer.
#[must_use]
pub fn step_up_error(gap: StepUpGap) -> ApiError {
    ApiError::new(ErrorCode::StepUpRequired).with_errors(vec![FieldIssue::new("/", gap.code())])
}

impl AuthServices {
    /// Number of passkeys of `user_id`.
    pub(crate) async fn passkey_count(&self, user_id: uuid::Uuid) -> ApiResult<i64> {
        let mut tx = db::begin(&self.db, &db::DbContext::user(user_id, None)).await?;
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM passkeys")
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(n)
    }

    /// Requires the second step for `action` (rules in the module description).
    ///
    /// # Errors
    /// `403 step_up_required` (field code tells what is missing); `401` without a session.
    pub async fn require_step_up(
        &self,
        ctx: &RequestContext,
        action: ProtectedAction,
    ) -> ApiResult<()> {
        let session_id = ctx
            .identity
            .session_id
            .ok_or_else(|| ApiError::new(ErrorCode::Unauthenticated))?;
        let user_id = ctx.identity.user_id;
        let record = self
            .sessions
            .get(user_id, session_id)
            .await?
            .ok_or_else(|| ApiError::new(ErrorCode::Unauthenticated))?;
        let has_passkey = self.passkey_count(user_id).await? > 0;
        let needs_passkey = ctx.has_permission(PASSKEY_REQUIRED_PERMISSION);
        match step_up_gap(&record, has_passkey, needs_passkey, Utc::now()) {
            None => Ok(()),
            Some(gap) => {
                tracing::info!(user_id = %user_id, action = action.as_str(), gap = gap.code(), "second step required");
                Err(step_up_error(gap))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::session::ClientKind;
    use uuid::Uuid;

    fn record(created: DateTime<Utc>, step_up: Option<DateTime<Utc>>) -> SessionRecord {
        SessionRecord {
            id: Uuid::now_v7(),
            user_id: Uuid::now_v7(),
            client_kind: ClientKind::Ios,
            shared_device_id: None,
            remember: false,
            created_at: created,
            last_activity_at: created,
            expires_at: created + Duration::days(1),
            step_up_at: step_up,
            revoked_at: None,
            idle_timeout_secs: None,
        }
    }

    #[test]
    fn administrator_without_passkey_must_enroll() {
        let now = Utc::now();
        let r = record(now, Some(now));
        assert_eq!(
            step_up_gap(&r, false, true, now),
            Some(StepUpGap::PasskeyEnrollmentRequired)
        );
    }

    #[test]
    fn passkey_holder_needs_a_fresh_check() {
        let now = Utc::now();
        let old = record(now - Duration::hours(1), None);
        assert_eq!(
            step_up_gap(&old, true, false, now),
            Some(StepUpGap::PasskeyRequired)
        );
        // A fresh sign-in alone is not enough once a passkey exists.
        let fresh = record(now, None);
        assert_eq!(
            step_up_gap(&fresh, true, false, now),
            Some(StepUpGap::PasskeyRequired)
        );
        let ok = record(now - Duration::hours(1), Some(now - Duration::minutes(14)));
        assert_eq!(step_up_gap(&ok, true, true, now), None);
        let stale = record(now - Duration::hours(1), Some(now - Duration::minutes(16)));
        assert_eq!(
            step_up_gap(&stale, true, true, now),
            Some(StepUpGap::PasskeyRequired)
        );
    }

    #[test]
    fn home_user_without_passkey_may_sign_in_again() {
        let now = Utc::now();
        assert_eq!(
            step_up_gap(&record(now - Duration::minutes(9), None), false, false, now),
            None
        );
        assert_eq!(
            step_up_gap(
                &record(now - Duration::minutes(11), None),
                false,
                false,
                now
            ),
            Some(StepUpGap::ReauthenticationRequired)
        );
    }
}
