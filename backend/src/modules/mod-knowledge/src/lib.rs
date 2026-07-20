//! mod-knowledge: Operations knowledge base.
//!
//! Automatically extracts structured knowledge from resolved incidents,
//! root cause analyses, and runbook executions. Supports full-text search
//! and AI-assisted retrieval for faster troubleshooting.

pub mod extraction;
pub mod search;

use std::sync::Arc;

use async_trait::async_trait;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use tracing::info;

pub struct ModKnowledge {
    db: SqlitePool,
    store: Arc<search::KnowledgeStore>,
}

impl ModKnowledge {
    pub async fn new(db: SqlitePool) -> Self {
        let store = Arc::new(search::KnowledgeStore::new(db.clone()).await);
        Self { db, store }
    }
}

#[async_trait]
impl OpsModule for ModKnowledge {
    fn name(&self) -> &str {
        "mod-knowledge"
    }

    fn description(&self) -> &str {
        "Ops knowledge base — auto-extract insights from incidents, searchable"
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
                name: "knowledge_search".into(),
                description: "Search the ops knowledge base".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "Search query"}
                    },
                    "required": ["query"]
                }),
            },
            ToolDefinition {
                name: "knowledge_extract".into(),
                description: "Extract knowledge from an incident record".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "incident_id": {"type": "string"}
                    },
                    "required": ["incident_id"]
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
            "knowledge_search" => {
                let query = params["query"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing query"))?;

                let results = self.store.search(query).await?;
                Ok(serde_json::json!({
                    "query": query,
                    "results": results
                }))
            }
            "knowledge_extract" => {
                let incident_id = params["incident_id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing incident_id"))?;

                let entry = extraction::extract_from_incident(incident_id);
                self.store.insert_entry(&entry).await?;

                info!(incident_id, title = %entry.title, "Knowledge extracted");
                Ok(serde_json::to_value(&entry)?)
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
        let m = ModKnowledge::new(db).await;
        assert_eq!(m.name(), "mod-knowledge");
        assert!(m.description().contains("knowledge"));
    }
}
