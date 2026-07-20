//! mod-monitor: Cross-host performance monitoring module.
//!
//! Collects CPU, memory, disk, and network metrics from managed hosts
//! via SSH. Stores time-series data in SQLite and exposes JSON endpoints
//! for the frontend dashboard (ECharts).

pub mod collector;
pub mod models;
pub mod storage;

use std::sync::Arc;

use async_trait::async_trait;
use ops_pilot_core::ssh::SshConnectionPool;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use tracing::info;

pub struct ModMonitor {
    collector: Arc<RwLock<collector::MetricsCollector>>,
    store: Arc<storage::MetricStore>,
}

impl ModMonitor {
    pub async fn new(db: SqlitePool, ssh_pool: Arc<SshConnectionPool>) -> Self {
        let store = Arc::new(storage::MetricStore::new(db).await);
        Self {
            collector: Arc::new(RwLock::new(collector::MetricsCollector::new(ssh_pool))),
            store,
        }
    }
}

#[async_trait]
impl OpsModule for ModMonitor {
    fn name(&self) -> &str {
        "mod-monitor"
    }

    fn description(&self) -> &str {
        "Cross-host performance monitoring with time-series metrics and dashboards"
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
                name: "monitor_collect".into(),
                description: "Collect latest metrics from a host".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": {"type": "string", "description": "Target host ID"}
                    },
                    "required": ["host_id"]
                }),
            },
            ToolDefinition {
                name: "monitor_get_timeseries".into(),
                description: "Get time-series metrics for a host over a time range".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": {"type": "string"},
                        "metric": {"type": "string", "enum": ["cpu", "memory", "disk", "network"]},
                        "since": {"type": "string", "description": "ISO8601 start time"}
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
        match tool {
            "monitor_collect" => {
                let host_id = params["host_id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing host_id"))?;

                let mut collector = self.collector.write().await;
                let metrics = collector.collect_host(host_id).await?;
                let points = collector.metrics_to_points(&metrics);
                for point in &points {
                    self.store.insert_point(point).await?;
                }

                Ok(serde_json::to_value(&metrics)?)
            }
            "monitor_get_timeseries" => {
                let host_id = params["host_id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing host_id"))?;
                let metric_filter = params["metric"].as_str();
                let since = params["since"].as_str();

                let points = self.store.query_points(host_id, metric_filter, since).await?;
                Ok(serde_json::to_value(&points)?)
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
    use ops_pilot_core::ssh::SshConnectionPool;

    #[tokio::test]
    async fn test_module_metadata() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let m = ModMonitor::new(pool, Arc::new(SshConnectionPool::new())).await;
        assert_eq!(m.name(), "mod-monitor");
        assert!(m.description().contains("monitoring"));
    }

    #[tokio::test]
    async fn test_tools_registered() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let m = ModMonitor::new(pool, Arc::new(SshConnectionPool::new())).await;
        let tools = m.tools();
        assert_eq!(tools.len(), 2);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"monitor_collect"));
        assert!(names.contains(&"monitor_get_timeseries"));
    }
}
