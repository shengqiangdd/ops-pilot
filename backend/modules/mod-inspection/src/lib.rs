//! mod-inspection: Intelligent inspection engine for OpsPilot.
//!
//! Runs automated health checks, security audits, and generates reports.

pub mod engine;

use std::sync::Arc;

use async_trait::async_trait;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
use sqlx::SqlitePool;
use tracing::info;

pub struct ModInspection {
    engine: engine::InspectionEngine,
}

impl ModInspection {
    pub async fn new(db: SqlitePool) -> Self {
        Self {
            engine: engine::InspectionEngine::new(db).await,
        }
    }
}

#[async_trait]
impl OpsModule for ModInspection {
    fn name(&self) -> &str { "mod-inspection" }
    fn version(&self) -> &str { "0.1.0" }
    fn description(&self) -> &str { "Intelligent inspection — automated health, security, and compliance checks" }
    fn dependencies(&self) -> Vec<&str> { vec!["mod-core"] }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "inspection_create".into(),
                description: "Create an inspection task with categories to check".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "categories": {"type": "array", "items": {"type": "string"}, "description": "Categories: health, process, certificate, security, log, network"}
                    },
                    "required": ["name"]
                }),
            },
            ToolDefinition {
                name: "inspection_run".into(),
                description: "Execute an inspection task and return results".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "task_id": {"type": "string"}
                    },
                    "required": ["task_id"]
                }),
            },
            ToolDefinition {
                name: "inspection_results".into(),
                description: "List past inspection results".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": {"type": "integer", "default": 10}
                    }
                }),
            },
            ToolDefinition {
                name: "inspection_report".into(),
                description: "Generate a markdown report for an inspection".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "result_id": {"type": "string"}
                    },
                    "required": ["result_id"]
                }),
            },
        ]
    }

    async fn execute(&self, _ctx: &ModuleContext, tool: &str, params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        match tool {
            "inspection_create" => {
                let name = params["name"].as_str().ok_or_else(|| anyhow::anyhow!("missing name"))?.to_string();
                let categories: Vec<String> = params["categories"].as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_else(|| vec!["health".into(), "security".into()]);
                let task = self.engine.create_task(&name, &categories).await?;
                Ok(serde_json::to_value(&task)?)
            }
            "inspection_run" => {
                let task_id = params["task_id"].as_str().ok_or_else(|| anyhow::anyhow!("missing task_id"))?.to_string();
                let result = self.engine.run_inspection(&task_id).await?;
                Ok(serde_json::to_value(&result)?)
            }
            "inspection_results" => {
                let limit = params["limit"].as_u64().unwrap_or(10) as u32;
                let results = self.engine.list_results(limit).await?;
                Ok(serde_json::to_value(&results)?)
            }
            "inspection_report" => {
                let result_id = params["result_id"].as_str().ok_or_else(|| anyhow::anyhow!("missing result_id"))?.to_string();
                let report = self.engine.generate_report(&result_id).await?;
                Ok(serde_json::json!({"report": report}))
            }
            _ => anyhow::bail!("unknown tool: {}", tool),
        }
    }

    async fn on_event(&self, _ctx: &ModuleContext, _event: &OpsEvent) -> Option<ModuleAction> { None }
    async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus { HealthStatus::Healthy }
}
