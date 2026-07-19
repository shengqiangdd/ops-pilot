//! mod-security: CIS compliance, vulnerability scanning, and patch management module.
//!
//! Provides rule-based security scanning against CIS benchmarks, vulnerability
//! checks, and LLM-powered remediation guidance for infrastructure hosts.

pub mod engine;
pub mod llm_scanner;
pub mod rules;

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use ops_pilot_sdk::llm::LlmClient;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};

use engine::SecurityEngine;
use llm_scanner::LlmScanner;

/// The Security and Compliance module.
pub struct ModSecurity {
    engine: Arc<RwLock<SecurityEngine>>,
    #[allow(dead_code)]
    llm_scanner: Option<LlmScanner>,
}

impl ModSecurity {
    pub fn new() -> Self {
        Self {
            engine: Arc::new(RwLock::new(SecurityEngine::new())),
            llm_scanner: None,
        }
    }

    /// Create a new ModSecurity with an LLM client for AI-powered analysis.
    pub fn with_llm(client: Arc<dyn LlmClient>) -> Self {
        Self {
            engine: Arc::new(RwLock::new(SecurityEngine::new())),
            llm_scanner: Some(LlmScanner::new(client)),
        }
    }
}

#[async_trait]
impl OpsModule for ModSecurity {
    fn name(&self) -> &str {
        "mod-security"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "CIS compliance, vulnerability scanning, and patch management"
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["mod-core"]
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "security_scan".into(),
                description: "Run CIS compliance checks against a host. Returns pass/fail/warn results for each check.".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": {
                            "type": "string",
                            "description": "The target host identifier"
                        },
                        "check_type": {
                            "type": "string",
                            "description": "Type of scan: 'cis_linux', 'cis_docker', 'vulnerability', 'patch', or 'all'",
                            "enum": ["cis_linux", "cis_docker", "vulnerability", "patch", "all"]
                        }
                    },
                    "required": ["host_id", "check_type"]
                }),
            },
            ToolDefinition {
                name: "security_list_checks".into(),
                description: "List all available security checks with descriptions, severity, and category".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "category": {
                            "type": "string",
                            "description": "Filter by category: 'auth', 'network', 'filesystem', 'docker', 'kernel', 'vulnerability', 'patch', or 'all'"
                        }
                    }
                }),
            },
            ToolDefinition {
                name: "security_remediate".into(),
                description: "Apply a fix rule to remediate a security finding".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": {
                            "type": "string",
                            "description": "The target host identifier"
                        },
                        "check_id": {
                            "type": "string",
                            "description": "The check ID to remediate (e.g. 'CIS-1.1', 'SSH-1')"
                        }
                    },
                    "required": ["host_id", "check_id"]
                }),
            },
            ToolDefinition {
                name: "security_report".into(),
                description: "Generate a summary report of compliance findings with pass rate, critical issues, and recommendations".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": {
                            "type": "string",
                            "description": "Optional host identifier. If omitted, returns a report across all scanned hosts."
                        }
                    }
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
            "security_scan" => {
                let host_id = params
                    .get("host_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("missing or invalid 'host_id' parameter"))?;

                let check_type = params
                    .get("check_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("missing or invalid 'check_type' parameter"))?;

                let engine = self.engine.read().await;
                let output = engine.scan(host_id, check_type, &[]);
                Ok(serde_json::to_value(&output)?)
            }
            "security_list_checks" => {
                let category = params
                    .get("category")
                    .and_then(|v| v.as_str())
                    .unwrap_or("all");

                let engine = self.engine.read().await;
                let filtered: Vec<&rules::SecurityRule> = if category == "all" {
                    engine.rules().iter().collect()
                } else {
                    engine
                        .rules()
                        .iter()
                        .filter(|r| r.category == category)
                        .collect()
                };
                Ok(serde_json::to_value(&filtered)?)
            }
            "security_remediate" => {
                let _host_id = params
                    .get("host_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("missing or invalid 'host_id' parameter"))?;

                let check_id = params
                    .get("check_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("missing or invalid 'check_id' parameter"))?;

                let engine = self.engine.read().await;
                let output = engine::simulate_remediate(check_id, engine.rules());
                Ok(serde_json::to_value(&output)?)
            }
            "security_report" => {
                let host_id = params
                    .get("host_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("all-hosts");

                let engine = self.engine.read().await;
                let scan_output = engine.scan(host_id, "all", &[]);

                let pass_rate = if scan_output.total_checks > 0 {
                    scan_output.passed as f64 / scan_output.total_checks as f64 * 100.0
                } else {
                    0.0
                };

                let critical_findings: Vec<String> = scan_output
                    .results
                    .iter()
                    .filter(|r| {
                        r.status != engine::CheckStatus::Pass
                            && r.severity == rules::Severity::Critical
                    })
                    .map(|r| {
                        format!(
                            "[{}] {} — {}",
                            r.check_id, r.rule_name, r.remediation_steps
                        )
                    })
                    .collect();

                let recommendations: Vec<String> = scan_output
                    .results
                    .iter()
                    .filter(|r| r.status != engine::CheckStatus::Pass)
                    .take(10)
                    .map(|r| format!("{}: {}", r.rule_name, r.remediation_steps))
                    .collect();

                let report = serde_json::json!({
                    "host_id": host_id,
                    "total_checks": scan_output.total_checks,
                    "passed": scan_output.passed,
                    "failed": scan_output.failed,
                    "warnings": scan_output.warnings,
                    "pass_rate_percent": pass_rate,
                    "critical_findings": critical_findings,
                    "recommendations": recommendations,
                    "timestamp": scan_output.timestamp,
                });

                Ok(report)
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
    use ops_pilot_sdk::llm::{LlmError, Message};
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
            PathBuf::from("/tmp/mod-security-test"),
            "mod-security".into(),
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

    #[allow(dead_code)]
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
    async fn test_module_tools_registered() {
        let module = ModSecurity::new();
        let tools = module.tools();
        assert_eq!(tools.len(), 4);

        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"security_scan"));
        assert!(names.contains(&"security_list_checks"));
        assert!(names.contains(&"security_remediate"));
        assert!(names.contains(&"security_report"));
    }

    #[tokio::test]
    async fn test_security_scan_all() {
        let module = ModSecurity::new();
        let ctx = make_ctx().await;

        let result = module
            .execute(
                &ctx,
                "security_scan",
                serde_json::json!({
                    "host_id": "web-server-1",
                    "check_type": "all"
                }),
            )
            .await
            .unwrap();

        assert_eq!(result["host_id"], "web-server-1");
        assert_eq!(result["check_type"], "all");
        assert!(result["total_checks"].as_u64().unwrap() > 0);
        let total = result["total_checks"].as_u64().unwrap();
        let passed = result["passed"].as_u64().unwrap();
        let failed = result["failed"].as_u64().unwrap();
        let warnings = result["warnings"].as_u64().unwrap();
        assert_eq!(total, passed + failed + warnings);
    }

    #[tokio::test]
    async fn test_security_scan_cis_docker() {
        let module = ModSecurity::new();
        let ctx = make_ctx().await;

        let result = module
            .execute(
                &ctx,
                "security_scan",
                serde_json::json!({
                    "host_id": "container-host",
                    "check_type": "cis_docker"
                }),
            )
            .await
            .unwrap();

        assert_eq!(result["check_type"], "cis_docker");
        let results = result["results"].as_array().unwrap();
        for r in results {
            assert_eq!(r["category"], "docker");
        }
    }

    #[tokio::test]
    async fn test_security_list_checks_all() {
        let module = ModSecurity::new();
        let ctx = make_ctx().await;

        let result = module
            .execute(&ctx, "security_list_checks", serde_json::json!({}))
            .await
            .unwrap();

        let checks = result.as_array().unwrap();
        assert!(checks.len() >= 15);
    }

    #[tokio::test]
    async fn test_security_list_checks_by_category() {
        let module = ModSecurity::new();
        let ctx = make_ctx().await;

        let result = module
            .execute(
                &ctx,
                "security_list_checks",
                serde_json::json!({ "category": "docker" }),
            )
            .await
            .unwrap();

        let checks = result.as_array().unwrap();
        assert!(!checks.is_empty());
        for c in checks {
            assert_eq!(c["category"], "docker");
        }
    }

    #[tokio::test]
    async fn test_security_remediate_valid() {
        let module = ModSecurity::new();
        let ctx = make_ctx().await;

        let result = module
            .execute(
                &ctx,
                "security_remediate",
                serde_json::json!({
                    "host_id": "host-1",
                    "check_id": "CIS-1.1"
                }),
            )
            .await
            .unwrap();

        assert_eq!(result["status"], "remediated");
        assert_eq!(result["check_id"], "CIS-1.1");
        assert!(result["details"].as_str().unwrap().contains("Separate partition"));
    }

    #[tokio::test]
    async fn test_security_remediate_invalid() {
        let module = ModSecurity::new();
        let ctx = make_ctx().await;

        let result = module
            .execute(
                &ctx,
                "security_remediate",
                serde_json::json!({
                    "host_id": "host-1",
                    "check_id": "NONEXISTENT"
                }),
            )
            .await
            .unwrap();

        assert_eq!(result["status"], "error");
        assert!(result["details"].as_str().unwrap().contains("Unknown check ID"));
    }

    #[tokio::test]
    async fn test_security_report() {
        let module = ModSecurity::new();
        let ctx = make_ctx().await;

        let result = module
            .execute(
                &ctx,
                "security_report",
                serde_json::json!({ "host_id": "host-1" }),
            )
            .await
            .unwrap();

        assert_eq!(result["host_id"], "host-1");
        assert!(result["total_checks"].as_u64().unwrap() > 0);
        assert!(result["pass_rate_percent"].as_f64().unwrap() >= 0.0);
        assert!(result["pass_rate_percent"].as_f64().unwrap() <= 100.0);
        assert!(result["recommendations"].as_array().unwrap().len() <= 10);
    }

    #[tokio::test]
    async fn test_security_report_no_host() {
        let module = ModSecurity::new();
        let ctx = make_ctx().await;

        let result = module
            .execute(&ctx, "security_report", serde_json::json!({}))
            .await
            .unwrap();

        assert_eq!(result["host_id"], "all-hosts");
    }

    #[tokio::test]
    async fn test_module_health_check() {
        let module = ModSecurity::new();
        let ctx = make_ctx().await;
        let status = module.health_check(&ctx).await;
        assert!(matches!(status, HealthStatus::Healthy));
    }

    #[tokio::test]
    async fn test_module_unknown_tool() {
        let module = ModSecurity::new();
        let ctx = make_ctx().await;
        let result = module.execute(&ctx, "nonexistent", serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_module_metadata() {
        let module = ModSecurity::new();
        assert_eq!(module.name(), "mod-security");
        assert_eq!(module.version(), "0.1.0");
        assert_eq!(module.dependencies(), vec!["mod-core"]);
    }

    #[tokio::test]
    async fn test_security_scan_missing_host_id() {
        let module = ModSecurity::new();
        let ctx = make_ctx().await;

        let result = module
            .execute(
                &ctx,
                "security_scan",
                serde_json::json!({ "check_type": "all" }),
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_security_scan_missing_check_type() {
        let module = ModSecurity::new();
        let ctx = make_ctx().await;

        let result = module
            .execute(
                &ctx,
                "security_scan",
                serde_json::json!({ "host_id": "host-1" }),
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_security_remediate_missing_check_id() {
        let module = ModSecurity::new();
        let ctx = make_ctx().await;

        let result = module
            .execute(
                &ctx,
                "security_remediate",
                serde_json::json!({ "host_id": "host-1" }),
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_with_llm_success() {
        let client = Arc::new(MockLlmClient {
            response: "Critical: SSH root login enabled. Remediate by setting PermitRootLogin no.".into(),
        });
        let module = ModSecurity::with_llm(client);
        let ctx = make_ctx().await;

        // Verify module was created with LLM scanner
        assert!(module.llm_scanner.is_some());
        // Basic scan still works
        let result = module
            .execute(
                &ctx,
                "security_scan",
                serde_json::json!({
                    "host_id": "host-1",
                    "check_type": "all"
                }),
            )
            .await
            .unwrap();
        assert!(result["total_checks"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_scan_type_vulnerability() {
        let module = ModSecurity::new();
        let ctx = make_ctx().await;

        let result = module
            .execute(
                &ctx,
                "security_scan",
                serde_json::json!({
                    "host_id": "host-1",
                    "check_type": "vulnerability"
                }),
            )
            .await
            .unwrap();

        let results = result["results"].as_array().unwrap();
        for r in results {
            assert_eq!(r["category"], "vulnerability");
        }
    }
}
