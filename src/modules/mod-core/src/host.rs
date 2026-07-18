//! Host sub-module: host CRUD, status tracking, health checks.

use async_trait::async_trait;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};

pub struct HostModule;

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
            "host_get" => Ok(serde_json::json!({"host_id": params["host_id"], "status": "unknown"})),
            "host_create" => Ok(serde_json::json!({"host_id": "new-uuid", "name": params["name"]})),
            "host_delete" => Ok(serde_json::json!({"deleted": true, "host_id": params["host_id"]})),
            "host_health_check" => Ok(serde_json::json!({"host_id": params["host_id"], "status": "healthy"})),
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
