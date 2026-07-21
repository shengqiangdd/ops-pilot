//! mod-fim: File Integrity Monitoring module.
//!
//! Tracks cryptographic hashes of critical system files on managed hosts,
//! detects unauthorized modifications, and generates alerts on drift.
//! Supports scheduled baseline scans and real-time change detection.

pub mod scanner;
pub mod storage;

use std::sync::Arc;

use async_trait::async_trait;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use tracing::info;

pub struct ModFim {
    db: SqlitePool,
    scanner: Arc<RwLock<scanner::FimScanner>>,
    store: Arc<storage::FimStore>,
}

impl ModFim {
    pub async fn new(db: SqlitePool, ssh_pool: Arc<ops_pilot_core::ssh::SshConnectionPool>) -> Self {
        let store = Arc::new(storage::FimStore::new(db.clone()).await);
        Self {
            db,
            scanner: Arc::new(RwLock::new(scanner::FimScanner::new(ssh_pool))),
            store,
        }
    }
}

#[async_trait]
impl OpsModule for ModFim {
    fn name(&self) -> &str {
        "mod-fim"
    }

    fn description(&self) -> &str {
        "File Integrity Monitoring — detect unauthorized file changes"
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
                name: "fim_baseline".into(),
                description: "Create or update a file integrity baseline for a host".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": {"type": "string"},
                        "paths": {"type": "array", "items": {"type": "string"}}
                    },
                    "required": ["host_id"]
                }),
            },
            ToolDefinition {
                name: "fim_scan".into(),
                description: "Run an integrity scan against the baseline".into(),
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
            "fim_baseline" => {
                let host_id = params["host_id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing host_id"))?;
                let paths: Vec<String> = params["paths"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_else(|| {
                        vec![
                            "/etc/passwd".into(),
                            "/etc/shadow".into(),
                            "/etc/ssh/sshd_config".into(),
                            "/etc/sudoers".into(),
                            "/etc/hosts".into(),
                        ]
                    });

                let scanner = self.scanner.write().await;
                let hashes = scanner.compute_hashes(host_id, &paths).await?;

                for (path, hash) in &hashes {
                    self.store.upsert_baseline(host_id, path, hash).await?;
                }

                info!(host_id, files = hashes.len(), "FIM baseline created");
                Ok(serde_json::json!({
                    "status": "ok",
                    "host_id": host_id,
                    "files_baselined": hashes.len()
                }))
            }
            "fim_scan" => {
                let host_id = params["host_id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing host_id"))?;

                let baseline = self.store.get_baseline(host_id).await?;
                if baseline.is_empty() {
                    return Ok(serde_json::json!({
                        "status": "no_baseline",
                        "host_id": host_id
                    }));
                }

                let paths: Vec<String> = baseline.iter().map(|(p, _)| p.clone()).collect();
                let scanner = self.scanner.write().await;
                let current = scanner.compute_hashes(host_id, &paths).await?;

                let mut changes = Vec::new();
                for (path, baseline_hash) in &baseline {
                    match current.get(path) {
                        Some(current_hash) if current_hash != baseline_hash => {
                            changes.push(serde_json::json!({
                                "path": path,
                                "status": "modified",
                                "old_hash": baseline_hash,
                                "new_hash": current_hash
                            }));
                        }
                        None => {
                            changes.push(serde_json::json!({
                                "path": path,
                                "status": "deleted"
                            }));
                        }
                        _ => {}
                    }
                }

                // Check for new files that weren't in baseline
                for (path, hash) in &current {
                    if !baseline.iter().any(|(p, _)| p == path) {
                        changes.push(serde_json::json!({
                            "path": path,
                            "status": "added",
                            "hash": hash
                        }));
                    }
                }

                info!(host_id, changes = changes.len(), "FIM scan complete");
                Ok(serde_json::json!({
                    "host_id": host_id,
                    "changes": changes,
                    "total_files": current.len()
                }))
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
        let m = ModFim::new(db, Arc::new(ops_pilot_core::ssh::SshConnectionPool::new())).await;
        assert_eq!(m.name(), "mod-fim");
        assert!(m.description().contains("Integrity"));
    }
}
