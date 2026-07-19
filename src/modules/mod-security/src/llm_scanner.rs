//! LLM-powered security analysis: enriches scan results with prioritized remediation advice.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use ops_pilot_gateway::llm::{LlmClient, LlmError, Message};

use crate::engine::{CheckStatus, ScanResult};
use crate::rules::Severity;

/// Combined result from scan + LLM analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSecurityReport {
    pub host_id: String,
    pub scan_summary: ScanSummary,
    pub llm_analysis: String,
    pub llm_success: bool,
}

/// Summary statistics fed into the LLM prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    pub total_checks: usize,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
    pub critical_findings: Vec<String>,
    pub high_findings: Vec<String>,
}

impl From<&[ScanResult]> for ScanSummary {
    fn from(results: &[ScanResult]) -> Self {
        let passed = results.iter().filter(|r| r.status == CheckStatus::Pass).count();
        let failed = results.iter().filter(|r| r.status == CheckStatus::Fail).count();
        let warnings = results.iter().filter(|r| r.status == CheckStatus::Warn).count();
        let critical_findings: Vec<String> = results
            .iter()
            .filter(|r| r.status != CheckStatus::Pass && r.severity == Severity::Critical)
            .map(|r| format!("[{}] {}", r.check_id, r.rule_name))
            .collect();
        let high_findings: Vec<String> = results
            .iter()
            .filter(|r| r.status != CheckStatus::Pass && r.severity == Severity::High)
            .map(|r| format!("[{}] {}", r.check_id, r.rule_name))
            .collect();

        Self {
            total_checks: results.len(),
            passed,
            failed,
            warnings,
            critical_findings,
            high_findings,
        }
    }
}

/// Errors from the LLM security analyzer.
#[derive(Debug, thiserror::Error)]
pub enum ScannerError {
    #[error("LLM error: {0}")]
    Llm(#[from] LlmError),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// LLM-powered security analyzer.
pub struct LlmScanner {
    client: Arc<dyn LlmClient>,
}

impl LlmScanner {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    /// Generate an LLM-powered security report from scan results.
    pub async fn analyze(
        &self,
        host_id: &str,
        scan_results: &[ScanResult],
    ) -> Result<LlmSecurityReport, ScannerError> {
        let summary = ScanSummary::from(scan_results);
        let messages = self.build_prompt(host_id, scan_results, &summary);

        match self.client.complete(&messages).await {
            Ok(response) => Ok(LlmSecurityReport {
                host_id: host_id.to_string(),
                scan_summary: summary,
                llm_analysis: response,
                llm_success: true,
            }),
            Err(e) => {
                tracing::warn!("LLM security analysis failed: {}", e);
                Ok(LlmSecurityReport {
                    host_id: host_id.to_string(),
                    scan_summary: summary,
                    llm_analysis: format!("LLM unavailable: {e}. Using rule-based analysis only."),
                    llm_success: false,
                })
            }
        }
    }

    fn build_prompt(
        &self,
        host_id: &str,
        scan_results: &[ScanResult],
        summary: &ScanSummary,
    ) -> Vec<Message> {
        let findings_str = if scan_results.is_empty() {
            "  (no findings)".to_string()
        } else {
            scan_results
                .iter()
                .filter(|r| r.status != CheckStatus::Pass)
                .map(|r| {
                    format!(
                        "  - [{}][{}] {} — actual: {}, expected: {}\n    Remediation: {}",
                        r.severity, r.check_id, r.rule_name, r.actual_value, r.expected_value, r.remediation_steps
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        let system_prompt = "You are an expert Linux/Docker security engineer and CIS benchmark auditor. \
            You analyze compliance scan results and produce actionable security reports. \
            Be specific and technical. Structure your response with: \
            1. Executive Summary \
            2. Critical Findings (immediate action required) \
            3. Prioritized Remediation Plan \
            4. Risk Assessment \
            5. Compliance Score and Recommendations";

        let user_prompt = format!(
            "Analyze this security scan for host {host_id}:\n\n\
             ## Scan Summary\n\
             - Total checks: {}\n\
             - Passed: {}\n\
             - Failed: {}\n\
             - Warnings: {}\n\
             - Critical findings: {}\n\
             - High findings: {}\n\n\
             ## Failed/Warning Findings\n{findings_str}\n\n\
             Provide a prioritized security report with specific remediation steps.",
            summary.total_checks,
            summary.passed,
            summary.failed,
            summary.warnings,
            summary.critical_findings.len(),
            summary.high_findings.len(),
        );

        vec![
            Message::system(system_prompt),
            Message::user(user_prompt),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::engine::ScanResult;
    use crate::rules::Severity;
    use futures_util::stream;
    use ops_pilot_gateway::llm::{LlmError, Message};
    use std::pin::Pin;

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

    fn sample_results() -> Vec<ScanResult> {
        vec![
            ScanResult {
                check_id: "CIS-1.1".into(),
                rule_name: "Separate partition for /tmp".into(),
                status: CheckStatus::Pass,
                actual_value: "/tmp on separate partition".into(),
                expected_value: "/tmp on separate partition".into(),
                severity: Severity::Medium,
                category: "filesystem".into(),
                remediation_steps: "Create separate partition".into(),
            },
            ScanResult {
                check_id: "SSH-1".into(),
                rule_name: "SSH: PermitRootLogin no".into(),
                status: CheckStatus::Fail,
                actual_value: "yes".into(),
                expected_value: "no".into(),
                severity: Severity::Critical,
                category: "auth".into(),
                remediation_steps: "Set PermitRootLogin no in sshd_config".into(),
            },
        ]
    }

    #[tokio::test]
    async fn test_llm_scanner_success() {
        let client = Arc::new(MockLlmClient {
            response: "Critical: SSH root login is enabled. Remediate immediately.".into(),
        });
        let scanner = LlmScanner::new(client);
        let result = scanner.analyze("host-1", &sample_results()).await.unwrap();

        assert!(result.llm_success);
        assert!(result.llm_analysis.contains("SSH root login"));
        assert_eq!(result.host_id, "host-1");
        assert_eq!(result.scan_summary.total_checks, 2);
        assert_eq!(result.scan_summary.passed, 1);
        assert_eq!(result.scan_summary.failed, 1);
    }

    #[tokio::test]
    async fn test_llm_scanner_fallback() {
        let client = Arc::new(FailingLlmClient);
        let scanner = LlmScanner::new(client);
        let result = scanner.analyze("host-2", &sample_results()).await.unwrap();

        assert!(!result.llm_success);
        assert!(result.llm_analysis.contains("LLM unavailable"));
    }

    #[tokio::test]
    async fn test_scan_summary_from_results() {
        let summary = ScanSummary::from(sample_results().as_slice());
        assert_eq!(summary.total_checks, 2);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.warnings, 0);
        assert_eq!(summary.critical_findings.len(), 1);
        assert!(summary.critical_findings[0].contains("SSH-1"));
    }

    #[tokio::test]
    async fn test_scan_summary_empty() {
        let summary = ScanSummary::from(Vec::<ScanResult>::new().as_slice());
        assert_eq!(summary.total_checks, 0);
        assert!(summary.critical_findings.is_empty());
    }
}
