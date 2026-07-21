//! mod-alert-escalation: Alert escalation and notification routing.
//!
//! Defines escalation policies (e.g. P1 → immediate SMS, P3 → email summary),
//! manages on-call schedules, and dispatches alerts to the appropriate channels
//! (webhook, email, ChatOps) with escalating frequency.

pub mod policy;
pub mod schedule;

use std::sync::Arc;

use async_trait::async_trait;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use tracing::info;

pub struct ModAlertEscalation {
    db: SqlitePool,
    policies: Arc<RwLock<Vec<policy::EscalationPolicy>>>,
    schedules: Arc<RwLock<Vec<schedule::OnCallSchedule>>>,
}

impl ModAlertEscalation {
    pub async fn new(db: SqlitePool) -> Self {
        let store = Self {
            db,
            policies: Arc::new(RwLock::new(policy::default_policies())),
            schedules: Arc::new(RwLock::new(Vec::new())),
        };
        store
    }
}

#[async_trait]
impl OpsModule for ModAlertEscalation {
    fn name(&self) -> &str {
        "mod-alert-escalation"
    }

    fn description(&self) -> &str {
        "Alert escalation policies — severity-based, time-based, multi-channel dispatch"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "escalation_define_policy".into(),
                description: "Define or update an escalation policy".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "severity": {"type": "string", "enum": ["P1","P2","P3","P4"]},
                        "escalation_delay_minutes": {"type": "integer"},
                        "channels": {"type": "array", "items": {"type": "string"}}
                    },
                    "required": ["name", "severity"]
                }),
            },
            ToolDefinition {
                name: "escalation_trigger".into(),
                description: "Trigger an escalation for an alert".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "alert_id": {"type": "string"},
                        "severity": {"type": "string"},
                        "message": {"type": "string"}
                    },
                    "required": ["alert_id", "severity"]
                }),
            },
        ]
    }

    async fn execute(
        &self,
        _ctx: &ModuleContext,
        tool: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        match tool {
            "escalation_define_policy" => {
                let name = params["name"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing name"))?
                    .to_string();
                let severity = params["severity"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing severity"))?
                    .to_string();
                let delay = params["escalation_delay_minutes"]
                    .as_i64()
                    .unwrap_or(15) as u32;
                let channels: Vec<String> = params["channels"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_else(|| vec!["webhook".into()]);

                let p = policy::EscalationPolicy {
                    name: name.clone(),
                    severity: severity.clone(),
                    delay_minutes: delay,
                    channels,
                };

                let mut policies = self.policies.write().await;
                policies.retain(|existing| existing.severity != severity);
                policies.push(p);

                info!(name, severity, "Escalation policy defined");
                Ok(serde_json::json!({ "status": "ok", "name": name, "severity": severity }))
            }
            "escalation_trigger" => {
                let alert_id = params["alert_id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing alert_id"))?
                    .to_string();
                let severity = params["severity"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing severity"))?
                    .to_string();
                let message = params["message"]
                    .as_str()
                    .unwrap_or("alert triggered")
                    .to_string();

                let policies = self.policies.read().await;
                let matched = policies.iter().find(|p| p.severity == severity);

                let result = match matched {
                    Some(p) => {
                        info!(
                            alert_id,
                            severity,
                            channels = ?p.channels,
                            delay = p.delay_minutes,
                            "Escalation triggered"
                        );
                        serde_json::json!({
                            "status": "escalated",
                            "alert_id": alert_id,
                            "policy": p.name,
                            "channels": p.channels,
                            "delay_minutes": p.delay_minutes
                        })
                    }
                    None => {
                        info!(alert_id, severity, "No policy matched, using default");
                        serde_json::json!({
                            "status": "no_policy",
                            "alert_id": alert_id,
                            "severity": severity
                        })
                    }
                };
                Ok(result)
            }
            _ => Err(anyhow::anyhow!("unknown tool: {}", tool)),
        }
    }

    async fn on_event(&self, _ctx: &ModuleContext, _event: &OpsEvent) -> Option<ModuleAction> {
        None
    }

    async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus {
        HealthStatus::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_module_metadata() {
        let db = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let m = ModAlertEscalation::new(db).await;
        assert_eq!(m.name(), "mod-alert-escalation");
    }

    #[tokio::test]
    async fn test_define_and_trigger() {
        let db = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let m = ModAlertEscalation::new(db).await;
        let ctx = ops_pilot_sdk::context::ModuleContext::new(
            Arc::new(SqlitePool::connect("sqlite::memory:").await.unwrap()),
            ops_pilot_sdk::context::EventBus::new(16),
            std::path::PathBuf::from("/tmp/test"),
            "test".into(),
        );

        let result = m.execute(&ctx, "escalation_define_policy", serde_json::json!({
            "name": "Critical PagerDuty",
            "severity": "P1",
            "escalation_delay_minutes": 5,
            "channels": ["pagerduty", "sms"]
        })).await.unwrap();
        assert_eq!(result["status"], "ok");

        let result = m.execute(&ctx, "escalation_trigger", serde_json::json!({
            "alert_id": "alert-001",
            "severity": "P1",
            "message": "Service down"
        })).await.unwrap();
        assert_eq!(result["status"], "escalated");
    }
}
