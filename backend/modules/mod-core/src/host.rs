//! Host sub-module: host CRUD, status tracking, health checks.

use async_trait::async_trait;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};

pub struct HostModule;

impl Default for HostModule {
    fn default() -> Self {
        Self::new()
    }
}

impl HostModule {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl OpsModule for HostModule {
    fn name(&self) -> &str {
        "mod-core::host"
    }
    fn version(&self) -> &str {
        "0.1.0"
    }
    fn description(&self) -> &str {
        "Host management: CRUD, status tracking, health checks"
    }
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "host_list".into(),
                description: "List all registered hosts".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            ToolDefinition {
                name: "host_get".into(),
                description: "Get details of a specific host".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": { "type": "string" }
                    },
                    "required": ["host_id"]
                }),
            },
            ToolDefinition {
                name: "host_create".into(),
                description: "Register a new host".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "hostname": { "type": "string" },
                        "port": { "type": "integer", "default": 22 },
                        "username": { "type": "string" }
                    },
                    "required": ["name", "hostname", "username"]
                }),
            },
            ToolDefinition {
                name: "host_delete".into(),
                description: "Remove a host".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": { "type": "string" }
                    },
                    "required": ["host_id"]
                }),
            },
            ToolDefinition {
                name: "host_health_check".into(),
                description: "Run a health check on a host".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": { "type": "string" }
                    },
                    "required": ["host_id"]
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
        tracing::info!(tool, "host module execute (stub)");
        match tool {
            "host_list" => Ok(serde_json::json!({"hosts": []})),
            "host_get" => {
                Ok(serde_json::json!({"host_id": params["host_id"], "status": "unknown"}))
            }
            "host_create" => Ok(serde_json::json!({"host_id": "new-uuid", "name": params["name"]})),
            "host_delete" => Ok(serde_json::json!({"deleted": true, "host_id": params["host_id"]})),
            "host_health_check" => {
                Ok(serde_json::json!({"host_id": params["host_id"], "status": "healthy"}))
            }
            _ => anyhow::bail!("unknown tool: {}", tool),
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
    use ops_pilot_sdk::context::{EventBus, ModuleContext};
    use std::path::PathBuf;
    use std::sync::Arc;

    async fn make_ctx() -> ModuleContext {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        ModuleContext::new(
            Arc::new(pool),
            EventBus::new(16),
            PathBuf::from("/tmp/test-host"),
            "test-host".into(),
        )
    }

    #[tokio::test]
    async fn test_module_metadata() {
        let m = HostModule::new();
        assert_eq!(m.name(), "mod-core::host");
        assert_eq!(m.version(), "0.1.0");
        assert!(m.description().contains("Host"));
    }

    #[tokio::test]
    async fn test_tools_registered() {
        let m = HostModule::new();
        let tools = m.tools();
        assert_eq!(tools.len(), 5);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"host_list"));
        assert!(names.contains(&"host_get"));
        assert!(names.contains(&"host_create"));
        assert!(names.contains(&"host_delete"));
        assert!(names.contains(&"host_health_check"));
    }

    #[tokio::test]
    async fn test_health_check() {
        let m = HostModule::new();
        let ctx = make_ctx().await;
        let status = m.health_check(&ctx).await;
        assert!(matches!(status, HealthStatus::Healthy));
    }

    #[tokio::test]
    async fn test_unknown_tool() {
        let m = HostModule::new();
        let ctx = make_ctx().await;
        let result = m.execute(&ctx, "nonexistent", serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_host_list_stub() {
        let m = HostModule::new();
        let ctx = make_ctx().await;
        let result = m
            .execute(&ctx, "host_list", serde_json::json!({}))
            .await
            .unwrap();
        assert!(result["hosts"].as_array().unwrap().is_empty());
    }
}
