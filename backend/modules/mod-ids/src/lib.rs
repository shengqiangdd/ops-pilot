//! mod-ids: Intrusion Detection System for OpsPilot.
//!
//! Analyzes logs, detects anomalies, checks IPs against blocklists,
//! and provides geolocation information.

pub mod analyzer;
pub mod anomaly;
pub mod geo;
pub mod blocklist;


use async_trait::async_trait;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
use sqlx::SqlitePool;

pub struct ModIds {
    analyzer: analyzer::LogAnalyzer,
    anomaly_detector: anomaly::AnomalyDetector,
    geo: geo::GeoLookup,
    blocklist: blocklist::BlocklistChecker,
}

impl ModIds {
    pub async fn new(_db: SqlitePool) -> Self {
        Self {
            analyzer: analyzer::LogAnalyzer::new(),
            anomaly_detector: anomaly::AnomalyDetector::new(),
            geo: geo::GeoLookup::new(),
            blocklist: blocklist::BlocklistChecker::new(),
        }
    }
}

#[async_trait]
impl OpsModule for ModIds {
    fn name(&self) -> &str { "mod-ids" }
    fn version(&self) -> &str { "0.1.0" }
    fn description(&self) -> &str { "Intrusion Detection — log analysis, anomaly detection, geolocation, blocklist" }
    fn dependencies(&self) -> Vec<&str> { vec!["mod-core"] }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "ids_analyze_log".into(),
                description: "Analyze a log entry for security threats".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "log_line": {"type": "string", "description": "Log line to analyze"},
                        "source": {"type": "string", "description": "Log source type (ssh/syslog/web)"}
                    },
                    "required": ["log_line"]
                }),
            },
            ToolDefinition {
                name: "ids_check_anomaly".into(),
                description: "Check metrics for anomalies using Z-score detection".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "values": {"type": "array", "items": {"type": "number"}, "description": "Historical metric values"},
                        "current": {"type": "number", "description": "Current metric value to check"}
                    },
                    "required": ["values", "current"]
                }),
            },
            ToolDefinition {
                name: "ids_geolocate".into(),
                description: "Geolocate an IP address".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "ip": {"type": "string", "description": "IP address to geolocate"}
                    },
                    "required": ["ip"]
                }),
            },
            ToolDefinition {
                name: "ids_blocklist_check".into(),
                description: "Check if an IP is on known threat blocklists".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "ip": {"type": "string", "description": "IP address to check"}
                    },
                    "required": ["ip"]
                }),
            },
        ]
    }

    async fn execute(&self, _ctx: &ModuleContext, tool: &str, params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        match tool {
            "ids_analyze_log" => {
                let line = params["log_line"].as_str().ok_or_else(|| anyhow::anyhow!("missing log_line"))?;
                let source = params["source"].as_str().unwrap_or("syslog");
                let result = self.analyzer.analyze_line(line, source);
                Ok(serde_json::to_value(&result)?)
            }
            "ids_check_anomaly" => {
                let values: Vec<f64> = params["values"].as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_f64()).collect())
                    .unwrap_or_default();
                let current = params["current"].as_f64().unwrap_or(0.0);
                let baseline = self.anomaly_detector.detect_baseline(&values);
                let score = self.anomaly_detector.check_deviation(current, &baseline);
                Ok(serde_json::to_value(&score)?)
            }
            "ids_geolocate" => {
                let ip = params["ip"].as_str().ok_or_else(|| anyhow::anyhow!("missing ip"))?;
                let result = self.geo.lookup(ip);
                Ok(serde_json::to_value(&result)?)
            }
            "ids_blocklist_check" => {
                let ip = params["ip"].as_str().ok_or_else(|| anyhow::anyhow!("missing ip"))?;
                let result = self.blocklist.check(ip);
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
        let module = ModIds::new(pool).await;
        assert_eq!(module.name(), "mod-ids");
        assert_eq!(module.version(), "0.1.0");
        assert!(!module.description().is_empty());
        assert_eq!(module.dependencies(), vec!["mod-core"]);
    }

    #[tokio::test]
    async fn test_tools_registered() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let module = ModIds::new(pool).await;
        let tools = module.tools();
        assert_eq!(tools.len(), 4);
        assert!(tools.iter().any(|t| t.name == "ids_analyze_log"));
        assert!(tools.iter().any(|t| t.name == "ids_check_anomaly"));
        assert!(tools.iter().any(|t| t.name == "ids_geolocate"));
        assert!(tools.iter().any(|t| t.name == "ids_blocklist_check"));
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
            "mod-ids".into(),
        );
        let pool2 = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let module = ModIds::new(pool2).await;
        let status = module.health_check(&ctx).await;
        assert!(matches!(status, HealthStatus::Healthy));
    }
}
