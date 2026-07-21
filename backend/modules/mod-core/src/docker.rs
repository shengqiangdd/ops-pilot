//! Docker sub-module: container management, stats, lifecycle operations.

use async_trait::async_trait;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};

pub struct DockerModule;

impl Default for DockerModule {
    fn default() -> Self {
        Self::new()
    }
}

impl DockerModule {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl OpsModule for DockerModule {
    fn name(&self) -> &str {
        "mod-core::docker"
    }
    fn version(&self) -> &str {
        "0.1.0"
    }
    fn description(&self) -> &str {
        "Docker container management and monitoring"
    }
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "docker_list_containers".into(),
                description: "List all Docker containers".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "all": { "type": "boolean", "description": "Include stopped containers", "default": false }
                    }
                }),
            },
            ToolDefinition {
                name: "docker_start".into(),
                description: "Start a Docker container".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "container_id": { "type": "string" }
                    },
                    "required": ["container_id"]
                }),
            },
            ToolDefinition {
                name: "docker_stop".into(),
                description: "Stop a Docker container".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "container_id": { "type": "string" }
                    },
                    "required": ["container_id"]
                }),
            },
            ToolDefinition {
                name: "docker_restart".into(),
                description: "Restart a Docker container".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "container_id": { "type": "string" }
                    },
                    "required": ["container_id"]
                }),
            },
            ToolDefinition {
                name: "docker_stats".into(),
                description: "Get stats for a Docker container".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "container_id": { "type": "string" }
                    },
                    "required": ["container_id"]
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
        tracing::info!(tool, "docker module execute (stub)");
        match tool {
            "docker_list_containers" => Ok(serde_json::json!({"containers": []})),
            "docker_start" => {
                Ok(serde_json::json!({"status": "started", "id": params["container_id"]}))
            }
            "docker_stop" => {
                Ok(serde_json::json!({"status": "stopped", "id": params["container_id"]}))
            }
            "docker_restart" => {
                Ok(serde_json::json!({"status": "restarted", "id": params["container_id"]}))
            }
            "docker_stats" => {
                Ok(serde_json::json!({"id": params["container_id"], "cpu": 0.0, "memory": 0.0}))
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
            PathBuf::from("/tmp/test-docker"),
            "test-docker".into(),
        )
    }

    #[tokio::test]
    async fn test_module_metadata() {
        let m = DockerModule::new();
        assert_eq!(m.name(), "mod-core::docker");
        assert_eq!(m.version(), "0.1.0");
        assert!(m.description().contains("Docker"));
    }

    #[tokio::test]
    async fn test_tools_registered() {
        let m = DockerModule::new();
        let tools = m.tools();
        assert_eq!(tools.len(), 5);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"docker_list_containers"));
        assert!(names.contains(&"docker_start"));
        assert!(names.contains(&"docker_stop"));
        assert!(names.contains(&"docker_restart"));
        assert!(names.contains(&"docker_stats"));
    }

    #[tokio::test]
    async fn test_health_check() {
        let m = DockerModule::new();
        let ctx = make_ctx().await;
        let status = m.health_check(&ctx).await;
        assert!(matches!(status, HealthStatus::Healthy));
    }

    #[tokio::test]
    async fn test_unknown_tool() {
        let m = DockerModule::new();
        let ctx = make_ctx().await;
        let result = m.execute(&ctx, "nonexistent", serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_docker_list_containers_stub() {
        let m = DockerModule::new();
        let ctx = make_ctx().await;
        let result = m
            .execute(&ctx, "docker_list_containers", serde_json::json!({}))
            .await
            .unwrap();
        assert!(result["containers"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_docker_start_stub() {
        let m = DockerModule::new();
        let ctx = make_ctx().await;
        let result = m
            .execute(
                &ctx,
                "docker_start",
                serde_json::json!({ "container_id": "abc123" }),
            )
            .await
            .unwrap();
        assert_eq!(result["status"], "started");
        assert_eq!(result["id"], "abc123");
    }
}
