//! Authorization: the one place that knows permissions, preset roles, scopes and hidden fields
//! (ADR-0009 rule 8; plan > Yetki matrisi; T2.13).
//!
//! - [`Permission`]: the fixed permission list (the database CHECK on `role_permissions`
//!   mirrors it; a test compares both).
//! - [`PRESETS`]: the preset roles seeded when a workspace is created ([`seed_roles`]).
//! - [`ScopedGrant`], [`Target`], [`ScopeFilter`]: scoped assignments. A record outside the
//!   caller's scope is `404`, a missing permission inside the scope is `403` (ADR-0008 3.3).
//! - [`RequestContext::redact`]: cost and price fields leave the response when the caller lacks
//!   `cost.view` (ADR-0009 rule 8.5).

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    context::RequestContext,
    error::{ApiError, ApiResult, ErrorCode, FieldIssue},
};

/// Every permission of the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Permission {
    /// View recipes.
    RecipeView,
    /// Edit recipe drafts.
    RecipeEdit,
    /// Approve recipe versions.
    RecipeApprove,
    /// Plan batches, assign vessels.
    BatchPlan,
    /// Record brewing data, close steps.
    BatchRecord,
    /// Correct a closed batch.
    BatchCorrectClosed,
    /// Quality measurements and tastings.
    QualityRecord,
    /// Release for packaging.
    ReleaseApprove,
    /// Stock movements and counts.
    StockMove,
    /// See costs and prices.
    CostView,
    /// Reports and exports.
    ReportsExport,
    /// Equipment and profile settings.
    SettingsEquipment,
    /// Users, roles and permissions.
    UsersManage,
    /// Workspace settings and deletion.
    WorkspaceManage,
    /// Subscription.
    BillingManage,
    /// Support access grants.
    SupportGrant,
    /// Device pairing and keys.
    DevicesManage,
}

use Permission::{
    BatchCorrectClosed, BatchPlan, BatchRecord, BillingManage, CostView, DevicesManage,
    QualityRecord, RecipeApprove, RecipeEdit, RecipeView, ReleaseApprove, ReportsExport,
    SettingsEquipment, StockMove, SupportGrant, UsersManage, WorkspaceManage,
};

impl Permission {
    /// Every permission, in the order of ADR-0009 rule 8.1.
    pub const ALL: [Self; 17] = [
        RecipeView,
        RecipeEdit,
        RecipeApprove,
        BatchPlan,
        BatchRecord,
        BatchCorrectClosed,
        QualityRecord,
        ReleaseApprove,
        StockMove,
        CostView,
        ReportsExport,
        SettingsEquipment,
        UsersManage,
        WorkspaceManage,
        BillingManage,
        SupportGrant,
        DevicesManage,
    ];

    /// The code stored in `role_permissions.permission` and sent to clients.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            RecipeView => "recipe.view",
            RecipeEdit => "recipe.edit",
            RecipeApprove => "recipe.approve",
            BatchPlan => "batch.plan",
            BatchRecord => "batch.record",
            BatchCorrectClosed => "batch.correct_closed",
            QualityRecord => "quality.record",
            ReleaseApprove => "release.approve",
            StockMove => "stock.move",
            CostView => "cost.view",
            ReportsExport => "reports.export",
            SettingsEquipment => "settings.equipment",
            UsersManage => "users.manage",
            WorkspaceManage => "workspace.manage",
            BillingManage => "billing.manage",
            SupportGrant => "support.grant",
            DevicesManage => "devices.manage",
        }
    }

    /// Parses a code; `None` for anything outside the list.
    #[must_use]
    pub fn parse(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.as_str() == code)
    }

    /// The settings group the permission belongs to (plan > Ayarlarda yönetim: recipe, batch,
    /// quality, stock, cost, settings).
    #[must_use]
    pub const fn group(self) -> &'static str {
        match self {
            RecipeView | RecipeEdit | RecipeApprove => "recipe",
            BatchPlan | BatchRecord | BatchCorrectClosed => "batch",
            QualityRecord | ReleaseApprove => "quality",
            StockMove => "stock",
            CostView => "cost",
            ReportsExport | SettingsEquipment | UsersManage | WorkspaceManage | BillingManage
            | SupportGrant | DevicesManage => "settings",
        }
    }

    /// Permissions that only make sense for the whole workspace; a role holding one of them
    /// cannot be assigned with a scope.
    #[must_use]
    pub const fn workspace_level(self) -> bool {
        matches!(
            self,
            UsersManage | WorkspaceManage | BillingManage | SupportGrant | DevicesManage
        )
    }
}

/// A preset role seeded into new workspaces (the plan's permission matrix and the two Home
/// sharing roles of ADR-0009 rule 8.1).
#[derive(Debug, Clone, Copy)]
pub struct PresetRole {
    /// `roles.template_key`.
    pub key: &'static str,
    /// Display name.
    pub name: &'static str,
    /// Screen opened after sign-in.
    pub landing_screen: &'static str,
    /// Seeded in business workspaces.
    pub business: bool,
    /// Seeded in home workspaces.
    pub home: bool,
    /// The permissions.
    pub permissions: &'static [Permission],
}

/// Preset roles (plan > Yetki matrisi). The administrator holds everything.
pub const PRESETS: [PresetRole; 8] = [
    PresetRole {
        key: "admin",
        name: "Administrator",
        landing_screen: "production_board",
        business: true,
        home: true,
        permissions: &Permission::ALL,
    },
    PresetRole {
        key: "brewmaster",
        name: "Brewmaster",
        landing_screen: "production_board",
        business: true,
        home: false,
        permissions: &[
            RecipeView,
            RecipeEdit,
            RecipeApprove,
            BatchPlan,
            BatchRecord,
            BatchCorrectClosed,
            QualityRecord,
            StockMove,
            CostView,
            ReportsExport,
            SettingsEquipment,
        ],
    },
    PresetRole {
        key: "operator",
        name: "Operator",
        landing_screen: "today",
        business: true,
        home: false,
        permissions: &[RecipeView, BatchRecord, QualityRecord],
    },
    PresetRole {
        key: "quality",
        name: "Quality",
        landing_screen: "quality_queue",
        business: true,
        home: false,
        permissions: &[
            RecipeView,
            BatchCorrectClosed,
            QualityRecord,
            ReleaseApprove,
            ReportsExport,
        ],
    },
    PresetRole {
        key: "warehouse",
        name: "Warehouse",
        landing_screen: "stock",
        business: true,
        home: false,
        permissions: &[RecipeView, StockMove, CostView, ReportsExport],
    },
    PresetRole {
        key: "viewer",
        name: "Viewer",
        landing_screen: "recipes",
        business: true,
        home: false,
        permissions: &[RecipeView, ReportsExport],
    },
    PresetRole {
        key: "share_editor",
        name: "Editor",
        landing_screen: "recipes",
        business: false,
        home: true,
        permissions: &[RecipeView, RecipeEdit, BatchRecord],
    },
    PresetRole {
        key: "share_viewer",
        name: "Viewer",
        landing_screen: "recipes",
        business: false,
        home: true,
        permissions: &[RecipeView],
    },
];

/// Looks up a preset by its template key.
#[must_use]
pub fn preset(key: &str) -> Option<&'static PresetRole> {
    PRESETS.iter().find(|p| p.key == key)
}

/// Seeds the preset roles of `mode` (`home` or `business`) into a new workspace and returns
/// the id of the administrator role. Runs in the creating transaction.
///
/// # Errors
/// Database errors.
pub async fn seed_roles(
    conn: &mut PgConnection,
    workspace_id: Uuid,
    mode: &str,
    user_id: Uuid,
) -> ApiResult<Uuid> {
    let mut admin = Uuid::nil();
    for p in PRESETS
        .iter()
        .filter(|p| if mode == "home" { p.home } else { p.business })
    {
        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO roles (id, workspace_id, name, template_key, landing_screen, \
                                created_by, updated_by) \
             VALUES ($1, $2, $3, $4, $5, $6, $6)",
        )
        .bind(id)
        .bind(workspace_id)
        .bind(p.name)
        .bind(p.key)
        .bind(p.landing_screen)
        .bind(user_id)
        .execute(&mut *conn)
        .await?;
        sqlx::query(
            "INSERT INTO role_permissions (workspace_id, role_id, permission, created_by) \
             SELECT $1, $2, c, $3 FROM unnest($4::text[]) AS c",
        )
        .bind(workspace_id)
        .bind(id)
        .bind(user_id)
        .bind(
            p.permissions
                .iter()
                .map(|x| x.as_str().to_owned())
                .collect::<Vec<_>>(),
        )
        .execute(&mut *conn)
        .await?;
        if p.key == "admin" {
            admin = id;
        }
    }
    Ok(admin)
}

/// What a scoped assignment is limited to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeKind {
    /// One recipe and its versions.
    Recipe,
    /// One batch with steps, measurements and packaging.
    Batch,
    /// Batches held in one vessel.
    Vessel,
    /// Batches held in one production area.
    Area,
}

impl ScopeKind {
    /// The `scope_kind` column value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Recipe => "recipe",
            Self::Batch => "batch",
            Self::Vessel => "vessel",
            Self::Area => "area",
        }
    }

    /// Parses the column value.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "recipe" => Some(Self::Recipe),
            "batch" => Some(Self::Batch),
            "vessel" => Some(Self::Vessel),
            "area" => Some(Self::Area),
            _ => None,
        }
    }
}

/// Permissions a member holds inside one scope (from all of their scoped assignments).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopedGrant {
    /// Scope kind.
    pub kind: ScopeKind,
    /// The recipe, batch, vessel or area.
    pub id: Uuid,
    /// Permission codes.
    pub permissions: BTreeSet<String>,
}

/// The record a request touches, described by the scopes it belongs to. A resource endpoint
/// fills in what it knows (a batch knows its id, vessel and area; a recipe version its recipe).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Target {
    /// Recipe.
    pub recipe: Option<Uuid>,
    /// Batch.
    pub batch: Option<Uuid>,
    /// Vessel the batch sits in.
    pub vessel: Option<Uuid>,
    /// Production area the batch sits in.
    pub area: Option<Uuid>,
}

impl Target {
    /// A recipe.
    #[must_use]
    pub fn recipe(id: Uuid) -> Self {
        Self {
            recipe: Some(id),
            ..Self::default()
        }
    }

    /// A batch with the vessel and area it is in.
    #[must_use]
    pub fn batch(id: Uuid, vessel: Option<Uuid>, area: Option<Uuid>) -> Self {
        Self {
            batch: Some(id),
            vessel,
            area,
            ..Self::default()
        }
    }

    fn within(&self, grant: &ScopedGrant) -> bool {
        let own = match grant.kind {
            ScopeKind::Recipe => self.recipe,
            ScopeKind::Batch => self.batch,
            ScopeKind::Vessel => self.vessel,
            ScopeKind::Area => self.area,
        };
        own == Some(grant.id)
    }
}

/// Which records a list endpoint may show for one permission: bind the id lists as
/// `= ANY($n)` next to the workspace condition (ADR-0009 rule 8.3).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScopeFilter {
    /// The caller holds the permission for the whole workspace.
    pub workspace_wide: bool,
    /// Recipes granted by scoped assignments.
    pub recipes: Vec<Uuid>,
    /// Batches granted directly.
    pub batches: Vec<Uuid>,
    /// Vessels whose batches are granted.
    pub vessels: Vec<Uuid>,
    /// Areas whose batches are granted.
    pub areas: Vec<Uuid>,
}

impl ScopeFilter {
    /// Nothing is visible.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.workspace_wide
            && self.recipes.is_empty()
            && self.batches.is_empty()
            && self.vessels.is_empty()
            && self.areas.is_empty()
    }
}

/// Whether a response key is a cost or price field: any `_`-separated part is `cost` or
/// `price` (`unit_cost`, `cost_currency`, `price`, `batch_cost_total`, ...).
#[must_use]
pub fn is_cost_field(key: &str) -> bool {
    key.split('_')
        .any(|part| matches!(part, "cost" | "costs" | "price" | "prices"))
}

fn strip_cost(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|k, _| !is_cost_field(k));
            map.values_mut().for_each(strip_cost);
        }
        Value::Array(items) => items.iter_mut().for_each(strip_cost),
        _ => {}
    }
}

fn has_cost(value: &Value, path: &str, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let here = format!("{path}/{k}");
                if is_cost_field(k) {
                    out.push(here.clone());
                }
                has_cost(v, &here, out);
            }
        }
        Value::Array(items) => {
            for (i, v) in items.iter().enumerate() {
                has_cost(v, &format!("{path}/{i}"), out);
            }
        }
        _ => {}
    }
}

impl RequestContext {
    /// Whether the caller holds `permission` for the whole workspace.
    #[must_use]
    pub fn can(&self, permission: Permission) -> bool {
        self.permissions.contains(permission.as_str())
    }

    /// Requires `permission` for the whole workspace.
    ///
    /// # Errors
    /// `403 forbidden`.
    pub fn require_permission(&self, permission: Permission) -> ApiResult<()> {
        self.require(permission.as_str())
    }

    /// Requires `permission` for the record described by `target`: held for the whole
    /// workspace or through a scoped assignment that covers the record.
    ///
    /// # Errors
    /// `404 not_found` when the record is outside everything the caller may see (no
    /// `recipe.view` for the workspace and no scoped assignment covering it); otherwise
    /// `403 forbidden`.
    pub fn require_for(&self, permission: Permission, target: &Target) -> ApiResult<()> {
        if self.can(permission)
            || self
                .scoped
                .iter()
                .any(|g| target.within(g) && g.permissions.contains(permission.as_str()))
        {
            return Ok(());
        }
        let visible = self.can(RecipeView) || self.scoped.iter().any(|g| target.within(g));
        Err(ApiError::new(if visible {
            ErrorCode::Forbidden
        } else {
            ErrorCode::NotFound
        }))
    }

    /// The records a list may show for `permission`.
    #[must_use]
    pub fn scope_filter(&self, permission: Permission) -> ScopeFilter {
        let mut f = ScopeFilter {
            workspace_wide: self.can(permission),
            ..ScopeFilter::default()
        };
        for g in self
            .scoped
            .iter()
            .filter(|g| g.permissions.contains(permission.as_str()))
        {
            match g.kind {
                ScopeKind::Recipe => f.recipes.push(g.id),
                ScopeKind::Batch => f.batches.push(g.id),
                ScopeKind::Vessel => f.vessels.push(g.id),
                ScopeKind::Area => f.areas.push(g.id),
            }
        }
        f
    }

    /// Removes cost and price fields from a response body unless the caller holds
    /// `cost.view`. The fields are absent, never `null` (ADR-0008 rule 2.5).
    pub fn redact(&self, body: &mut Value) {
        if !self.can(CostView) {
            strip_cost(body);
        }
    }

    /// Serializes `body` and applies [`Self::redact`]: the one serialization path for
    /// responses that may carry cost fields.
    ///
    /// # Errors
    /// `500` when serialization fails.
    pub fn redacted_json<T: Serialize>(&self, body: &T) -> ApiResult<Value> {
        let mut value = serde_json::to_value(body).map_err(|e| {
            tracing::error!(error = %e, "response serialization failed");
            ApiError::new(ErrorCode::Internal)
        })?;
        self.redact(&mut value);
        Ok(value)
    }

    /// Refuses a request body that writes cost or price fields without `cost.view`.
    ///
    /// # Errors
    /// `403 forbidden` naming the fields.
    pub fn reject_cost_writes(&self, body: &Value) -> ApiResult<()> {
        if self.can(CostView) {
            return Ok(());
        }
        let mut found = Vec::new();
        has_cost(body, "", &mut found);
        if found.is_empty() {
            Ok(())
        } else {
            Err(ApiError::new(ErrorCode::Forbidden).with_errors(
                found
                    .into_iter()
                    .map(|f| FieldIssue::new(f, "permission_required"))
                    .collect(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::context::Identity;

    fn ctx(perms: &[Permission], scoped: Vec<ScopedGrant>) -> RequestContext {
        RequestContext {
            identity: Identity {
                user_id: Uuid::nil(),
                session_id: None,
                actor_kind: db::ActorKind::User,
            },
            workspace_id: Some(Uuid::nil()),
            permissions: perms.iter().map(|p| p.as_str().to_owned()).collect(),
            scoped,
            idempotency_key: None,
        }
    }

    #[test]
    fn permission_codes_round_trip_and_match_the_database_check() {
        let unique: BTreeSet<_> = Permission::ALL.iter().map(|p| p.as_str()).collect();
        assert_eq!(unique.len(), 17);
        for p in Permission::ALL {
            assert_eq!(Permission::parse(p.as_str()), Some(p));
        }
        assert_eq!(Permission::parse("recipe.delete"), None);
    }

    #[test]
    fn preset_roles_follow_the_plan_matrix() {
        assert_eq!(preset("admin").unwrap().permissions.len(), 17);
        // The matrix rows 1-13 for the six business roles (plan > Yetki matrisi).
        let matrix: [(Permission, &str); 13] = [
            (RecipeView, "abokdw"),
            (RecipeEdit, "ab"),
            (RecipeApprove, "ab"),
            (BatchPlan, "ab"),
            (BatchRecord, "abo"),
            (BatchCorrectClosed, "abk"),
            (QualityRecord, "aboK"),
            (ReleaseApprove, "ak"),
            (StockMove, "abd"),
            (CostView, "abd"),
            (ReportsExport, "abkdw"),
            (SettingsEquipment, "ab"),
            (UsersManage, "a"),
        ];
        // a admin, b brewmaster, o operator, k quality, d depo (warehouse), w viewer; the
        // quality-measurement row lists "K" for quality because `k` is taken by the loop key.
        let roles = [
            ('a', "admin"),
            ('b', "brewmaster"),
            ('o', "operator"),
            ('k', "quality"),
            ('d', "warehouse"),
            ('w', "viewer"),
        ];
        for (p, who) in matrix {
            for (letter, key) in roles {
                let expected = who.contains(letter) || (who.contains('K') && letter == 'k');
                assert_eq!(
                    preset(key).unwrap().permissions.contains(&p),
                    expected,
                    "{key} x {}",
                    p.as_str()
                );
            }
        }
        for key in [
            "workspace.manage",
            "billing.manage",
            "support.grant",
            "devices.manage",
        ] {
            let p = Permission::parse(key).unwrap();
            for (_, role) in &roles[1..] {
                assert!(!preset(role).unwrap().permissions.contains(&p), "{role}");
            }
        }
        assert_eq!(
            preset("share_editor").unwrap().permissions,
            [RecipeView, RecipeEdit, BatchRecord]
        );
        assert_eq!(preset("share_viewer").unwrap().permissions, [RecipeView]);
    }

    #[test]
    fn scope_decides_between_403_and_404() {
        let (a, b) = (Uuid::from_u128(1), Uuid::from_u128(2));
        let grant = ScopedGrant {
            kind: ScopeKind::Recipe,
            id: a,
            permissions: [RecipeView, RecipeEdit]
                .iter()
                .map(|p| p.as_str().to_owned())
                .collect(),
        };
        let c = ctx(&[], vec![grant]);
        assert!(c.require_for(RecipeEdit, &Target::recipe(a)).is_ok());
        assert_eq!(
            c.require_for(RecipeEdit, &Target::recipe(b))
                .unwrap_err()
                .code(),
            ErrorCode::NotFound
        );
        assert_eq!(
            c.require_for(RecipeApprove, &Target::recipe(a))
                .unwrap_err()
                .code(),
            ErrorCode::Forbidden
        );
        let wide = ctx(&[RecipeView], vec![]);
        assert_eq!(
            wide.require_for(RecipeEdit, &Target::recipe(b))
                .unwrap_err()
                .code(),
            ErrorCode::Forbidden
        );
        assert_eq!(c.scope_filter(RecipeEdit).recipes, vec![a]);
        assert!(c.scope_filter(BatchRecord).is_empty());
    }

    #[test]
    fn cost_fields_leave_the_body_without_cost_view() {
        let body = json!({"name": "IPA", "unit_cost": 1.2, "cost_currency": "USD",
            "items": [{"price": 3, "kg": 2, "price_currency": "USD"}], "costume": 1});
        let mut hidden = body.clone();
        ctx(&[RecipeView], vec![]).redact(&mut hidden);
        assert_eq!(
            hidden,
            json!({"name": "IPA", "items": [{"kg": 2}], "costume": 1})
        );
        let mut shown = body.clone();
        ctx(&[CostView], vec![]).redact(&mut shown);
        assert_eq!(shown, body);
        assert!(
            ctx(&[], vec![])
                .reject_cost_writes(&json!({"a": {"price": 1}}))
                .is_err()
        );
        assert!(
            ctx(&[CostView], vec![])
                .reject_cost_writes(&json!({"price": 1}))
                .is_ok()
        );
    }
}
