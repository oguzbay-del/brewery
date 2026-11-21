//! Alert rules: the condition type and the evaluation on measurement events.

use chrono::{DateTime, Utc};
use events::consumer::{BoxFuture, Event, Handler, HandlerError};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    Result, create,
    create::{NewNotification, create as create_notification},
    enter_workspace,
};

/// Topic of the event that triggers the evaluation.
pub const MEASUREMENT_TOPIC: &str = "batch.measurement_added";
/// `condition.schema_version` this code writes.
pub const CONDITION_VERSION: i64 = 1;
/// Measurement kinds a rule can watch (the `measurements.kind` values).
pub const KINDS: &[&str] = &[
    "sg",
    "temp_c",
    "ph",
    "pressure_bar",
    "volume_l",
    "abv_pct",
    "battery_v",
    "signal_dbm",
];

/// Comparison of a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Op {
    /// Value above the threshold.
    Gt,
    /// Value at or above the threshold.
    Gte,
    /// Value below the threshold.
    Lt,
    /// Value at or below the threshold.
    Lte,
}

/// The condition of an alert rule (`alert_rules.condition`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    /// Always [`CONDITION_VERSION`].
    pub schema_version: i64,
    /// Measurement kind.
    pub measurement: String,
    /// Comparison.
    pub op: Op,
    /// Threshold, in the SI unit of the measurement.
    pub value: f64,
    /// A rule fires at most once in this time for one recipient. Default 60.
    #[serde(default = "default_cooldown")]
    pub cooldown_min: i64,
}

const fn default_cooldown() -> i64 {
    60
}

impl Condition {
    /// Parses and checks a stored or submitted condition.
    ///
    /// # Errors
    /// The name of the invalid field, as a JSON pointer.
    pub fn parse(v: &Value) -> std::result::Result<Self, &'static str> {
        let c: Self = serde_json::from_value(v.clone()).map_err(|_| "/condition")?;
        if c.schema_version != CONDITION_VERSION {
            return Err("/condition/schema_version");
        }
        if !KINDS.contains(&c.measurement.as_str()) {
            return Err("/condition/measurement");
        }
        if !c.value.is_finite() {
            return Err("/condition/value");
        }
        if !(1..=10_080).contains(&c.cooldown_min) {
            return Err("/condition/cooldown_min");
        }
        Ok(c)
    }

    /// Whether `measured` fires the rule.
    #[must_use]
    pub fn fires(&self, measured: f64) -> bool {
        match self.op {
            Op::Gt => measured > self.value,
            Op::Gte => measured >= self.value,
            Op::Lt => measured < self.value,
            Op::Lte => measured <= self.value,
        }
    }

    /// Number of the cooldown window that holds `now`. Notifications of one rule in one window
    /// share a dedupe key.
    #[must_use]
    pub fn window(&self, now: DateTime<Utc>) -> i64 {
        now.timestamp().div_euclid(self.cooldown_min * 60)
    }
}

type RuleRow = (Uuid, String, Value, i16, Option<String>, Option<Uuid>);

/// Evaluates the alert rules for the latest measurements of a batch. Returns the number of
/// notifications created.
///
/// # Errors
/// Database errors.
pub async fn evaluate_batch(
    conn: &mut PgConnection,
    workspace_id: Uuid,
    batch_id: Uuid,
    now: DateTime<Utc>,
) -> Result<usize> {
    let rules: Vec<RuleRow> = sqlx::query_as(
        "SELECT id, name, condition, priority, scope_kind, scope_id FROM alert_rules
         WHERE enabled AND deleted_at IS NULL AND (muted_until IS NULL OR muted_until <= $1)
           AND (scope_kind IS NULL OR scope_kind = 'device'
                OR (scope_kind = 'batch' AND scope_id = $2))",
    )
    .bind(now)
    .bind(batch_id)
    .fetch_all(&mut *conn)
    .await?;
    if rules.is_empty() {
        return Ok(0);
    }
    let members = create::recipients(conn, workspace_id).await?;
    let mut created = 0;
    for (rule_id, name, raw, priority, scope_kind, scope_id) in rules {
        let Ok(rule_cond) = Condition::parse(&raw) else {
            tracing::warn!(%rule_id, "alert rule has an invalid condition; skipped");
            continue;
        };
        let latest: Option<(f64, Option<Uuid>)> = sqlx::query_as(
            "SELECT value, device_id FROM measurements
             WHERE batch_id = $1 AND kind = $2 ORDER BY measured_at DESC, created_at DESC LIMIT 1",
        )
        .bind(batch_id)
        .bind(&rule_cond.measurement)
        .fetch_optional(&mut *conn)
        .await?;
        let Some((value, device_id)) = latest else {
            continue;
        };
        if scope_kind.as_deref() == Some("device") && device_id != scope_id {
            continue;
        }
        if !rule_cond.fires(value) {
            continue;
        }
        let window = rule_cond.window(now);
        for member in &members {
            let n = NewNotification {
                workspace_id,
                recipient_member_id: *member,
                priority,
                kind: "alert.threshold".to_owned(),
                text_key: "notification.alert.threshold".to_owned(),
                params: json!({
                    "rule_name": name,
                    "measurement": rule_cond.measurement,
                    "op": rule_cond.op,
                    "threshold": rule_cond.value,
                    "value": value,
                }),
                batch_id: Some(batch_id),
                batch_step_id: None,
                dedupe_key: Some(format!("alert:{rule_id}:{window}:{member}")),
            };
            if create_notification(conn, &n).await?.is_some() {
                created += 1;
            }
        }
    }
    Ok(created)
}

/// Consumer of `batch.measurement_added`.
#[derive(Debug, Clone, Copy, Default)]
pub struct AlertHandler;

impl Handler for AlertHandler {
    fn handle<'a>(
        &'a self,
        event: &'a Event,
        tx: &'a mut PgConnection,
    ) -> BoxFuture<'a, std::result::Result<(), HandlerError>> {
        Box::pin(async move {
            if event.topic != MEASUREMENT_TOPIC {
                return Ok(());
            }
            let (Some(ws), Some(batch)) = (
                event.workspace_id,
                event
                    .body
                    .pointer("/entity/id")
                    .and_then(Value::as_str)
                    .and_then(|s| Uuid::parse_str(s).ok()),
            ) else {
                return Err(HandlerError::Permanent("event has no batch".to_owned()));
            };
            enter_workspace(tx, ws).await?;
            evaluate_batch(tx, ws, batch, Utc::now()).await?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cond(op: Op, value: f64) -> Condition {
        Condition {
            schema_version: 1,
            measurement: "temp_c".to_owned(),
            op,
            value,
            cooldown_min: 60,
        }
    }

    #[test]
    fn comparisons() {
        assert!(cond(Op::Gt, 22.0).fires(22.1));
        assert!(!cond(Op::Gt, 22.0).fires(22.0));
        assert!(cond(Op::Gte, 22.0).fires(22.0));
        assert!(cond(Op::Lt, 10.0).fires(9.9));
        assert!(cond(Op::Lte, 10.0).fires(10.0));
    }

    #[test]
    fn parse_checks_fields() {
        let ok = json!({"schema_version": 1, "measurement": "sg", "op": "lt", "value": 1.01});
        assert_eq!(Condition::parse(&ok).unwrap().cooldown_min, 60);
        let bad_kind = json!({"schema_version": 1, "measurement": "x", "op": "lt", "value": 1});
        assert_eq!(Condition::parse(&bad_kind), Err("/condition/measurement"));
        let bad_cooldown = json!({"schema_version": 1, "measurement": "sg", "op": "lt",
            "value": 1, "cooldown_min": 0});
        assert_eq!(
            Condition::parse(&bad_cooldown),
            Err("/condition/cooldown_min")
        );
        assert_eq!(Condition::parse(&json!({})), Err("/condition"));
    }

    #[test]
    fn windows_group_by_cooldown() {
        let c = cond(Op::Gt, 1.0);
        let t = DateTime::<Utc>::from_timestamp(7200, 0).unwrap();
        let later = DateTime::<Utc>::from_timestamp(7200 + 3599, 0).unwrap();
        let next = DateTime::<Utc>::from_timestamp(7200 + 3600, 0).unwrap();
        assert_eq!(c.window(t), c.window(later));
        assert_ne!(c.window(t), c.window(next));
    }
}
