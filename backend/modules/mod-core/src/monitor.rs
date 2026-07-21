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
        ctx: &ModuleContext,
        tool: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        tracing::info!(tool, "monitor module execute (stub)");
        match tool {
            "monitor_metrics" => {
                let host_id = params["host_id"].as_str().unwrap_or("unknown").to_string();
                let result = serde_json::json!({
                    "host_id": &host_id,
                    "cpu_percent": 0.0,
                    "memory_percent": 0.0,
                    "disk_percent": 0.0,
                    "network_rx_bytes": 0,
                    "network_tx_bytes": 0
                });

                // Publish metric update event via the global event bus
                if let Ok(uuid) = host_id.parse() {
                    let _ = ops_pilot_sdk::global_event_bus().publish(OpsEvent::MetricUpdated {
                        host_id: uuid,
                        metric: "cpu_percent".into(),
                        value: 0.0,
                    });
                }

                // Also emit via context for backward compatibility
                ctx.emit(OpsEvent::MetricUpdated {
                    host_id: host_id.parse().unwrap_or_default(),
                    metric: "cpu_percent".into(),
                    value: 0.0,
                })
                .await;

                Ok(result)
            }
            "monitor_alerts" => Ok(serde_json::json!({"alerts": []})),
            "monitor_uptime" => {
                Ok(serde_json::json!({"host_id": params["host_id"], "uptime_seconds": 0}))
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
            PathBuf::from("/tmp/test-monitor"),
            "test-monitor".into(),
        )
    }

    #[tokio::test]
    async fn test_module_metadata() {
        let m = MonitorModule::new();
        assert_eq!(m.name(), "mod-core::monitor");
        assert_eq!(m.version(), "0.1.0");
        assert!(m.description().contains("monitoring"));
    }

    #[tokio::test]
    async fn test_tools_registered() {
        let m = MonitorModule::new();
        let tools = m.tools();
        assert_eq!(tools.len(), 3);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"monitor_metrics"));
        assert!(names.contains(&"monitor_alerts"));
        assert!(names.contains(&"monitor_uptime"));
    }

    #[tokio::test]
    async fn test_health_check() {
        let m = MonitorModule::new();
        let ctx = make_ctx().await;
        let status = m.health_check(&ctx).await;
        assert!(matches!(status, HealthStatus::Healthy));
    }

    #[tokio::test]
    async fn test_unknown_tool() {
        let m = MonitorModule::new();
        let ctx = make_ctx().await;
        let result = m.execute(&ctx, "nonexistent", serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_monitor_metrics_stub() {
        let m = MonitorModule::new();
        let ctx = make_ctx().await;
        let result = m
            .execute(
                &ctx,
                "monitor_metrics",
                serde_json::json!({ "host_id": "test-host" }),
            )
            .await
            .unwrap();
        assert_eq!(result["host_id"], "test-host");
        assert_eq!(result["cpu_percent"], 0.0);
    }

    #[tokio::test]
    async fn test_monitor_alerts_stub() {
        let m = MonitorModule::new();
        let ctx = make_ctx().await;
        let result = m
            .execute(&ctx, "monitor_alerts", serde_json::json!({}))
            .await
            .unwrap();
        assert!(result["alerts"].as_array().unwrap().is_empty());
    }
}
