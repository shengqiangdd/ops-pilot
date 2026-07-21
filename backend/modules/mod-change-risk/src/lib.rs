//! mod-change-risk: Change Risk Assessment engine.
//!
//! Evaluates infrastructure changes for risk before execution, considering
//! historical failure rates, impact scope, change type, and timing.

pub mod engine;

use std::sync::Arc;

use async_trait::async_trait;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
use sqlx::SqlitePool;
use tracing::info;

pub struct ModChangeRisk {
    engine: engine::ChangeRiskEngine,
}

impl ModChangeRisk {
    pub async fn new(db: SqlitePool) -> Self {
        Self {
            engine: engine::ChangeRiskEngine::new(db),
        }
    }
}

#[async_trait]
impl OpsModule for ModChangeRisk {
    fn name(&self) -> &str { "mod-change-risk" }
    fn version(&self) -> &str { "0.1.0" }
    fn description(&self) -> &str { "Change risk assessment — evaluates changes before execution" }
    fn dependencies(&self) -> Vec<&str> { vec!["mod-core"] }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "change_risk_assess".into(),
                description: "Assess risk score for a proposed change".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "resource": {"type": "string", "description": "Affected resource (e.g. host/prod-1)"},
                        "change_type": {"type": "string", "enum": ["config_change","deployment","restart","read_only"]},
                        "description": {"type": "string"},
                        "affected_services": {"type": "array", "items": {"type": "string"}}
                    },
                    "required": ["resource", "change_type", "description"]
                }),
            },
            ToolDefinition {
                name: "change_risk_check_conflicts".into(),
                description: "Check for conflicting concurrent changes".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "changes": {"type": "array", "items": {"type": "object", "properties": {"resource": {"type": "string"}, "change_type": {"type": "string"}, "description": {"type": "string"}}}}
                    },
                    "required": ["changes"]
                }),
            },
            ToolDefinition {
                name: "change_risk_auto_approve".into(),
                description: "Auto-approve or reject a change based on risk assessment".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "score": {"type": "number"},
                        "level": {"type": "string"}
                    },
                    "required": ["score", "level"]
                }),
            },
        ]
    }

    async fn execute(&self, _ctx: &ModuleContext, tool: &str, params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        match tool {
            "change_risk_assess" => {
                let resource = params["resource"].as_str().ok_or_else(|| anyhow::anyhow!("missing resource"))?.to_string();
                let change_type = params["change_type"].as_str().unwrap_or("config_change").to_string();
                let description = params["description"].as_str().unwrap_or("").to_string();
                let affected: Vec<String> = params["affected_services"].as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();

                let assessment = self.engine.assess(&resource, &change_type, &description, &affected).await;
                Ok(serde_json::to_value(&assessment)?)
            }
            "change_risk_check_conflicts" => {
                let changes: Vec<engine::ChangeRequest> = params["changes"].as_array()
                    .map(|a| a.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect())
                    .unwrap_or_default();
                let conflicts = self.engine.check_conflicts(&changes).await;
                Ok(serde_json::to_value(&conflicts)?)
            }
            "change_risk_auto_approve" => {
                let score = params["score"].as_f64().unwrap_or(0.5);
                let level = params["level"].as_str().unwrap_or("medium").to_string();
                let decision = self.engine.auto_approve(score, &level);
                Ok(serde_json::to_value(&decision)?)
            }
            _ => anyhow::bail!("unknown tool: {}", tool),
        }
    }

    async fn on_event(&self, _ctx: &ModuleContext, _event: &OpsEvent) -> Option<ModuleAction> { None }
    async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus { HealthStatus::Healthy }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ops_pilot_sdk::context::{EventBus, ModuleContext};
    use std::path::PathBuf;

    async fn make_ctx() -> ModuleContext {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        ModuleContext::new(Arc::new(pool), EventBus::new(64), PathBuf::from("/tmp/test"), "test".into())
    }

    #[tokio::test]
    async fn test_assess_change() {
        let module = ModChangeRisk::new(sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap()).await;
        let ctx = make_ctx().await;
        let result = module.execute(&ctx, "change_risk_assess", serde_json::json!({
            "resource": "host/prod-1", "change_type": "deployment", "description": "Deploy v2.0"
        })).await.unwrap();
        assert!(result["score"].as_f64().is_some());
    }

    #[tokio::test]
    async fn test_auto_approve_low_risk() {
        let module = ModChangeRisk::new(sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap()).await;
        let ctx = make_ctx().await;
        let result = module.execute(&ctx, "change_risk_auto_approve", serde_json::json!({
            "score": 0.2, "level": "low"
        })).await.unwrap();
        assert!(result["approved"].as_bool().unwrap());
    }
}
