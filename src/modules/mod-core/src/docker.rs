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
            "docker_start" => Ok(serde_json::json!({"status": "started", "id": params["container_id"]})),
            "docker_stop" => Ok(serde_json::json!({"status": "stopped", "id": params["container_id"]})),
            "docker_restart" => Ok(serde_json::json!({"status": "restarted", "id": params["container_id"]})),
            "docker_stats" => Ok(serde_json::json!({"id": params["container_id"], "cpu": 0.0, "memory": 0.0})),
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
