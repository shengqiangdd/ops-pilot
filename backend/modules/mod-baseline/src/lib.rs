//! mod-baseline: Security baseline auditing module.
//!
//! Periodically checks managed hosts against a defined security baseline
//! (hardened OS config, SSH settings, kernel parameters, file permissions).
//! Produces drift reports and auto-remediation suggestions.

pub mod checks;
pub mod prediction;
pub mod reports;

use std::sync::Arc;

use async_trait::async_trait;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use tracing::info;

pub struct ModBaseline {
    #[allow(dead_code)]
    db: SqlitePool,
    checker: Arc<RwLock<checks::BaselineChecker>>,
    store: Arc<reports::ReportStore>,
}

impl ModBaseline {
    pub async fn new(db: SqlitePool, ssh_pool: Arc<ops_pilot_core::ssh::SshConnectionPool>) -> Self {
        let store = Arc::new(reports::ReportStore::new(db.clone()).await);
        Self {
            db,
            checker: Arc::new(RwLock::new(checks::BaselineChecker::new(ssh_pool))),
            store,
        }
    }
}

#[async_trait]
impl OpsModule for ModBaseline {
    fn name(&self) -> &str {
        "mod-baseline"
    }

    fn description(&self) -> &str {
        "Security baseline auditing — daily compliance checks and drift detection"
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
                name: "baseline_run_check".into(),
                description: "Run baseline checks on a host".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": {"type": "string"},
                        "check_name": {"type": "string", "description": "Specific check or 'all'"}
                    },
                    "required": ["host_id"]
                }),
            },
            ToolDefinition {
                name: "baseline_report".into(),
                description: "Get the latest baseline report for a host".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": {"type": "string"}
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
            "baseline_run_check" => {
                let host_id = params["host_id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing host_id"))?;
                let check_name = params["check_name"].as_str().unwrap_or("all");

                let mut checker = self.checker.write().await;
                let results = checker.run_checks(host_id, check_name).await?;

                let score = reports::compute_score(&results);
                self.store.save_report(host_id, &results, score).await?;

                info!(host_id, checks = results.len(), score, "Baseline check complete");
                Ok(serde_json::json!({
                    "host_id": host_id,
                    "results": results,
                    "score": score
                }))
            }
            "baseline_report" => {
                let host_id = params["host_id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing host_id"))?;

                match self.store.get_latest_report(host_id).await? {
                    Some(report) => Ok(serde_json::to_value(&report)?),
                    None => Ok(serde_json::json!({
                        "host_id": host_id,
                        "status": "no_report"
                    })),
                }
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
        let m = ModBaseline::new(db, Arc::new(ops_pilot_core::ssh::SshConnectionPool::new())).await;
        assert_eq!(m.name(), "mod-baseline");
        assert!(m.description().contains("baseline"));
    }
}
