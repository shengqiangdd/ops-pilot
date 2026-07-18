//! mod-rca: Root Cause Analysis module for OpsPilot.
//!
//! Provides rule-based and AI-powered diagnostics that analyze system symptoms
//! and return ranked root causes with confidence scores and suggested fixes.

pub mod analyzer;
pub mod llm_analyzer;
pub mod rules;

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use ops_pilot_gateway::llm::LlmClient;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};

use analyzer::{RcaAnalyzer, RcaHistoryEntry};
use llm_analyzer::{AnalyzerError, LlmAnalyzer};

/// The Root Cause Analysis module.
pub struct ModRca {
    analyzer: Arc<RwLock<RcaAnalyzer>>,
    llm_analyzer: Option<LlmAnalyzer>,
}

impl ModRca {
    pub fn new() -> Self {
        Self {
            analyzer: Arc::new(RwLock::new(RcaAnalyzer::new())),
            llm_analyzer: None,
        }
    }

    /// Create a new ModRca with an LLM client for AI-powered analysis.
    pub fn with_llm(client: Arc<dyn LlmClient>) -> Self {
        Self {
            analyzer: Arc::new(RwLock::new(RcaAnalyzer::new())),
            llm_analyzer: Some(LlmAnalyzer::new(client)),
        }
    }
}

#[async_trait]
impl OpsModule for ModRca {
    fn name(&self) -> &str {
        "mod-rca"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "Root Cause Analysis: rule-based diagnostics from system symptoms"
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["mod-core"]
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "rca_analyze".into(),
                description: "Analyze symptoms and return likely root causes with confidence scores".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "symptoms": {
                            "type": "object",
                            "description": "Map of metric_name to numeric value (e.g. {\"cpu_percent\": 92.0, \"memory_percent\": 88.0})",
                            "additionalProperties": { "type": "number" }
                        }
                    },
                    "required": ["symptoms"]
                }),
            },
            ToolDefinition {
                name: "rca_rules".into(),
                description: "List all available analysis rules with their conditions and severity".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            ToolDefinition {
                name: "rca_history".into(),
                description: "Get past analysis results".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of history entries to return (default 10)"
                        }
                    }
                }),
            },
            ToolDefinition {
                name: "rca_analyze_with_llm".into(),
                description: "Analyze symptoms with rule-based engine then get deeper LLM-powered root cause analysis with fix suggestions".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "symptoms": {
                            "type": "object",
                            "description": "Map of metric_name to numeric value (e.g. {\"cpu_percent\": 92.0, \"memory_percent\": 88.0})",
                            "additionalProperties": { "type": "number" }
                        },
                        "description": {
                            "type": "string",
                            "description": "Natural language description of the issue being observed"
                        }
                    },
                    "required": ["symptoms", "description"]
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
            "rca_analyze" => {
                let symptoms_map = params
                    .get("symptoms")
                    .and_then(|v| v.as_object())
                    .ok_or_else(|| anyhow::anyhow!("missing or invalid 'symptoms' parameter"))?;

                let mut symptoms = HashMap::with_capacity(symptoms_map.len());
                for (k, v) in symptoms_map {
                    let val = v
                        .as_f64()
                        .ok_or_else(|| anyhow::anyhow!("symptom '{}' must be a number", k))?;
                    symptoms.insert(k.clone(), val);
                }

                let mut analyzer = self.analyzer.write().await;
                let results = analyzer.analyze(symptoms);
                Ok(serde_json::to_value(&results)?)
            }
            "rca_rules" => {
                let analyzer = self.analyzer.read().await;
                let rules = analyzer.rules();
                Ok(serde_json::to_value(rules)?)
            }
            "rca_history" => {
                let limit = params
                    .get("limit")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as usize;

                let analyzer = self.analyzer.read().await;
                let history = analyzer.history();
                let start = history.len().saturating_sub(limit);
                let entries: Vec<&RcaHistoryEntry> = history[start..].iter().collect();
                Ok(serde_json::to_value(&entries)?)
            }
            "rca_analyze_with_llm" => {
                let symptoms_map = params
                    .get("symptoms")
                    .and_then(|v| v.as_object())
                    .ok_or_else(|| anyhow::anyhow!("missing or invalid 'symptoms' parameter"))?;

                let description = params
                    .get("description")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("missing or invalid 'description' parameter"))?;

                let mut symptoms = HashMap::with_capacity(symptoms_map.len());
                for (k, v) in symptoms_map {
                    let val = v
                        .as_f64()
                        .ok_or_else(|| anyhow::anyhow!("symptom '{}' must be a number", k))?;
                    symptoms.insert(k.clone(), val);
                }

                // Step 1: Run rule-based analysis.
                let rule_results = {
                    let mut analyzer = self.analyzer.write().await;
                    analyzer.analyze(symptoms.clone())
                };

                // Step 2: Run LLM analysis if available.
                match &self.llm_analyzer {
                    Some(llm) => {
                        let result = llm
                            .analyze(&symptoms, &rule_results, description)
                            .await
                            .map_err(|e| match e {
                                AnalyzerError::Llm(le) => anyhow::anyhow!("LLM error: {}", le),
                                AnalyzerError::Serialization(se) => {
                                    anyhow::anyhow!("serialization error: {}", se)
                                }
                            })?;
                        Ok(serde_json::to_value(&result)?)
                    }
                    None => {
                        // No LLM configured — return rule-based results with a note.
                        Ok(serde_json::json!({
                            "rule_results": rule_results,
                            "issue_description": description,
                            "llm_analysis": "LLM not configured. Only rule-based analysis available.",
                            "llm_success": false,
                        }))
                    }
                }
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
    use ops_pilot_gateway::llm::{LlmError, Message};
    use ops_pilot_sdk::context::EventBus;
    use sqlx::SqlitePool;
    use std::path::PathBuf;
    use std::pin::Pin;
    use std::sync::Arc;

    async fn make_ctx() -> ModuleContext {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        ModuleContext::new(
            Arc::new(pool),
            EventBus::new(64),
            PathBuf::from("/tmp/mod-rca-test"),
            "mod-rca".into(),
        )
    }

    struct MockLlmClient {
        response: String,
    }

    #[async_trait]
    impl LlmClient for MockLlmClient {
        async fn complete(&self, _messages: &[Message]) -> Result<String, LlmError> {
            Ok(self.response.clone())
        }

        async fn complete_stream(
            &self,
            _messages: &[Message],
        ) -> Result<Pin<Box<dyn futures_util::Stream<Item = Result<String, LlmError>> + Send>>, LlmError>
        {
            use futures_util::stream;
            let chunks: Vec<Result<String, LlmError>> = self
                .response
                .split_whitespace()
                .map(|w| Ok(format!("{w} ")))
                .collect();
            Ok(Box::pin(stream::iter(chunks)))
        }
    }

    struct FailingLlmClient;

    #[async_trait]
    impl LlmClient for FailingLlmClient {
        async fn complete(&self, _messages: &[Message]) -> Result<String, LlmError> {
            Err(LlmError::Api {
                status: 503,
                message: "unavailable".into(),
            })
        }

        async fn complete_stream(
            &self,
            _messages: &[Message],
        ) -> Result<Pin<Box<dyn futures_util::Stream<Item = Result<String, LlmError>> + Send>>, LlmError>
        {
            Err(LlmError::StreamClosed)
        }
    }

    #[tokio::test]
    async fn test_rca_module_tools_registered() {
        let module = ModRca::new();
        let tools = module.tools();
        assert_eq!(tools.len(), 4);

        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"rca_analyze"));
        assert!(names.contains(&"rca_rules"));
        assert!(names.contains(&"rca_history"));
        assert!(names.contains(&"rca_analyze_with_llm"));
    }

    #[tokio::test]
    async fn test_rca_module_execute_rules() {
        let module = ModRca::new();
        let ctx = make_ctx().await;

        let result = module
            .execute(&ctx, "rca_rules", serde_json::json!({}))
            .await
            .unwrap();

        let rules = result.as_array().unwrap();
        assert_eq!(rules.len(), 7);
    }

    #[tokio::test]
    async fn test_rca_module_execute_analyze() {
        let module = ModRca::new();
        let ctx = make_ctx().await;

        let result = module
            .execute(
                &ctx,
                "rca_analyze",
                serde_json::json!({
                    "symptoms": {
                        "cpu_percent": 92.0,
                        "memory_percent": 88.0
                    }
                }),
            )
            .await
            .unwrap();

        let results = result.as_array().unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0]["rule_name"], "memory_leak");
    }

    #[tokio::test]
    async fn test_rca_module_execute_history() {
        let module = ModRca::new();
        let ctx = make_ctx().await;

        module
            .execute(
                &ctx,
                "rca_analyze",
                serde_json::json!({
                    "symptoms": { "disk_percent": 95.0 }
                }),
            )
            .await
            .unwrap();

        let result = module
            .execute(&ctx, "rca_history", serde_json::json!({}))
            .await
            .unwrap();

        let history = result.as_array().unwrap();
        assert_eq!(history.len(), 1);
    }

    #[tokio::test]
    async fn test_rca_module_health_check() {
        let module = ModRca::new();
        let ctx = make_ctx().await;
        let status = module.health_check(&ctx).await;
        assert!(matches!(status, HealthStatus::Healthy));
    }

    #[tokio::test]
    async fn test_rca_module_unknown_tool() {
        let module = ModRca::new();
        let ctx = make_ctx().await;
        let result = module.execute(&ctx, "nonexistent", serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rca_analyze_with_llm_success() {
        let client = Arc::new(MockLlmClient {
            response: "Root cause: memory leak in worker process. Fix: restart and profile.".into(),
        });
        let module = ModRca::with_llm(client);
        let ctx = make_ctx().await;

        let result = module
            .execute(
                &ctx,
                "rca_analyze_with_llm",
                serde_json::json!({
                    "symptoms": {
                        "cpu_percent": 92.0,
                        "memory_percent": 88.0
                    },
                    "description": "Server is running very slowly"
                }),
            )
            .await
            .unwrap();

        assert!(result["llm_success"].as_bool().unwrap());
        assert!(result["llm_analysis"].as_str().unwrap().contains("memory leak"));
        assert_eq!(result["issue_description"].as_str().unwrap(), "Server is running very slowly");
        let rules = result["rule_results"].as_array().unwrap();
        assert!(!rules.is_empty());
        assert_eq!(rules[0]["rule_name"], "memory_leak");
    }

    #[tokio::test]
    async fn test_rca_analyze_with_llm_no_llm_configured() {
        let module = ModRca::new(); // No LLM
        let ctx = make_ctx().await;

        let result = module
            .execute(
                &ctx,
                "rca_analyze_with_llm",
                serde_json::json!({
                    "symptoms": { "disk_percent": 95.0 },
                    "description": "Disk space alarm"
                }),
            )
            .await
            .unwrap();

        assert!(!result["llm_success"].as_bool().unwrap());
        assert!(result["llm_analysis"].as_str().unwrap().contains("not configured"));
    }

    #[tokio::test]
    async fn test_rca_analyze_with_llm_fallback_on_failure() {
        let client = Arc::new(FailingLlmClient);
        let module = ModRca::with_llm(client);
        let ctx = make_ctx().await;

        let result = module
            .execute(
                &ctx,
                "rca_analyze_with_llm",
                serde_json::json!({
                    "symptoms": { "cpu_percent": 92.0, "memory_percent": 88.0 },
                    "description": "High resource usage"
                }),
            )
            .await
            .unwrap();

        // LLM failed, but rule-based results should still be present
        assert!(!result["llm_success"].as_bool().unwrap());
        let rules = result["rule_results"].as_array().unwrap();
        assert!(!rules.is_empty());
        assert_eq!(rules[0]["rule_name"], "memory_leak");
    }

    #[tokio::test]
    async fn test_rca_analyze_with_llm_missing_symptoms() {
        let module = ModRca::new();
        let ctx = make_ctx().await;

        let result = module
            .execute(
                &ctx,
                "rca_analyze_with_llm",
                serde_json::json!({
                    "description": "Something is wrong"
                }),
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rca_analyze_with_llm_missing_description() {
        let module = ModRca::new();
        let ctx = make_ctx().await;

        let result = module
            .execute(
                &ctx,
                "rca_analyze_with_llm",
                serde_json::json!({
                    "symptoms": { "cpu_percent": 90.0 }
                }),
            )
            .await;

        assert!(result.is_err());
    }
}
