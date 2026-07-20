//! Monitor sub-module: host monitoring, metrics collection, alerting.

use async_trait::async_trait;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};

pub struct MonitorModule;

impl Default for MonitorModule {
    fn default() -> Self {
        Self::new()
    }
}

impl MonitorModule {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl OpsModule for MonitorModule {
    fn name(&self) -> &str {
        "mod-core::monitor"
    }
    fn version(&self) -> &str {
        "0.1.0"
    }
    fn description(&self) -> &str {
        "Host monitoring, metrics collection, and alerting"
    }
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "monitor_metrics".into(),
                description: "Get current metrics for a host (CPU, memory, disk, network)".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": { "type": "string" }
                    },
                    "required": ["host_id"]
                }),
            },
            ToolDefinition {
                name: "monitor_alerts".into(),
                description: "List active alerts".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "severity": { "type": "string", "enum": ["critical", "warning", "info"] }
                    }
                }),
            },
            ToolDefinition {
                name: "monitor_uptime".into(),
                description: "Get uptime information for a host".into(),
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
        tracing::info!(tool, "monitor module execute (stub)");
        match tool {
            "monitor_metrics" => Ok(serde_json::json!({
                "host_id": params["host_id"],
                "cpu_percent": 0.0,
                "memory_percent": 0.0,
                "disk_percent": 0.0,
                "network_rx_bytes": 0,
                "network_tx_bytes": 0
            })),
            "monitor_alerts" => Ok(serde_json::json!({"alerts": []})),
            "monitor_uptime" => Ok(serde_json::json!({"host_id": params["host_id"], "uptime_seconds": 0})),
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
