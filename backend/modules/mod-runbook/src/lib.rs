//! mod-runbook: AI-powered Runbook automation.
//!
//! Accepts natural language descriptions of operational procedures,
//! converts them into multi-step runbooks, executes with approval gates,
//! and records results for audit and replay.

pub mod executor;
pub mod plans;

use std::sync::Arc;

use async_trait::async_trait;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use tracing::info;

pub struct ModRunbook {
    db: SqlitePool,
    store: Arc<plans::RunbookStore>,
}

impl ModRunbook {
    pub async fn new(db: SqlitePool) -> Self {
        let store = Arc::new(plans::RunbookStore::new(db.clone()).await);
        Self { db, store }
    }
}

#[async_trait]
impl OpsModule for ModRunbook {
    fn name(&self) -> &str {
        "mod-runbook"
    }

    fn description(&self) -> &str {
        "AI Runbook automation — natural language to multi-step ops procedures"
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
                name: "runbook_create".into(),
                description: "Create a runbook from a natural language description".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "description": {"type": "string", "description": "Natural language description of the procedure"}
                    },
                    "required": ["name", "description"]
                }),
            },
            ToolDefinition {
                name: "runbook_execute".into(),
                description: "Execute a runbook by name".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "target_host_id": {"type": "string"}
                    },
                    "required": ["name"]
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
            "runbook_create" => {
                let name = params["name"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing name"))?
                    .to_string();
                let description = params["description"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing description"))?
                    .to_string();

                let steps = plans::parse_description_to_steps(&description);
                let runbook = plans::Runbook {
                    name: name.clone(),
                    description,
                    steps,
                };

                self.store.save_runbook(&runbook).await?;
                info!(name = %runbook.name, steps = runbook.steps.len(), "Runbook created");
                Ok(serde_json::to_value(&runbook)?)
            }
            "runbook_execute" => {
                let name = params["name"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing name"))?;
                let host_id = params["target_host_id"].as_str().unwrap_or("localhost");

                let runbook = self
                    .store
                    .get_runbook(name)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("runbook '{}' not found", name))?;

                let mut exec = executor::RunbookExecutor::new();
                let result = exec.execute_runbook(&runbook, host_id).await?;

                self.store.save_execution(name, &result).await?;
                info!(name, host_id, steps = result.steps.len(), "Runbook executed");
                Ok(serde_json::to_value(&result)?)
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

    #[tokio::test]
    async fn test_module_metadata() {
        let db = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let m = ModRunbook::new(db).await;
        assert_eq!(m.name(), "mod-runbook");
        assert!(m.description().contains("Runbook"));
    }
}
