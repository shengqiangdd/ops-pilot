//! mod-container-sec: Container security scanning for OpsPilot.
//!
//! Scans container images for vulnerabilities, checks runtime security
//! configurations, and monitors container events.

pub mod image_scanner;
pub mod runtime_checker;


use async_trait::async_trait;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
use sqlx::SqlitePool;

pub struct ModContainerSec {
    image_scanner: image_scanner::ImageScanner,
    runtime_checker: runtime_checker::RuntimeChecker,
}

impl ModContainerSec {
    pub async fn new(_db: SqlitePool) -> Self {
        Self {
            image_scanner: image_scanner::ImageScanner::new(),
            runtime_checker: runtime_checker::RuntimeChecker::new(),
        }
    }
}

#[async_trait]
impl OpsModule for ModContainerSec {
    fn name(&self) -> &str { "mod-container-sec" }
    fn version(&self) -> &str { "0.1.0" }
    fn description(&self) -> &str { "Container security — image scanning, runtime checks, audit" }
    fn dependencies(&self) -> Vec<&str> { vec!["mod-core"] }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "container_scan_image".into(),
                description: "Scan a container image for security issues".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "image_name": {"type": "string", "description": "Full image name (e.g. nginx:latest)"}
                    },
                    "required": ["image_name"]
                }),
            },
            ToolDefinition {
                name: "container_check_runtime".into(),
                description: "Check Docker/containerd runtime security configuration".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ]
    }

    async fn execute(&self, _ctx: &ModuleContext, tool: &str, params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        match tool {
            "container_scan_image" => {
                let image = params["image_name"].as_str().ok_or_else(|| anyhow::anyhow!("missing image_name"))?;
                let result = self.image_scanner.scan(image);
                Ok(serde_json::to_value(&result)?)
            }
            "container_check_runtime" => {
                let result = self.runtime_checker.check();
                Ok(serde_json::to_value(&result)?)
            }
            _ => anyhow::bail!("unknown tool: {}", tool),
        }
    }

    async fn on_event(&self, _ctx: &ModuleContext, _event: &OpsEvent) -> Option<ModuleAction> { None }
    async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus { HealthStatus::Healthy }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;
    use ops_pilot_sdk::context::EventBus;

    #[tokio::test]
    async fn test_module_metadata() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let module = ModContainerSec::new(pool).await;
        assert_eq!(module.name(), "mod-container-sec");
        assert_eq!(module.version(), "0.1.0");
        assert!(!module.description().is_empty());
        assert_eq!(module.dependencies(), vec!["mod-core"]);
    }

    #[tokio::test]
    async fn test_tools_registered() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let module = ModContainerSec::new(pool).await;
        let tools = module.tools();
        assert_eq!(tools.len(), 2);
        assert!(tools.iter().any(|t| t.name == "container_scan_image"));
        assert!(tools.iter().any(|t| t.name == "container_check_runtime"));
        for t in &tools {
            assert!(!t.description.is_empty());
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let ctx = ModuleContext::new(
            Arc::new(pool),
            EventBus::new(16),
            PathBuf::from("/tmp"),
            "mod-container-sec".into(),
        );
        let pool2 = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let module = ModContainerSec::new(pool2).await;
        let status = module.health_check(&ctx).await;
        assert!(matches!(status, HealthStatus::Healthy));
    }
}
