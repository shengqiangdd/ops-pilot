//! LLM-powered RCA analyzer: enriches rule-based findings with deeper analysis.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use ops_pilot_sdk::llm::{LlmClient, LlmError, Message};

use crate::analyzer::RcaResult;

/// Combined result from rule-based + LLM analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmAnalysisResult {
    /// Rule-based findings that triggered before the LLM call.
    pub rule_results: Vec<RcaResult>,
    /// The natural language issue description provided by the user.
    pub issue_description: String,
    /// LLM's root cause analysis and fix suggestions.
    pub llm_analysis: String,
    /// Whether the LLM call succeeded.
    pub llm_success: bool,
}

/// Errors from the LLM analyzer.
#[derive(Debug, thiserror::Error)]
pub enum AnalyzerError {
    #[error("LLM error: {0}")]
    Llm(#[from] LlmError),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// LLM-powered RCA analyzer.
///
/// Takes a trait object for the LLM client so it can be tested with mocks.
pub struct LlmAnalyzer {
    client: Arc<dyn LlmClient>,
}

impl LlmAnalyzer {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    /// Run LLM analysis given symptoms, rule-based results, and an issue description.
    pub async fn analyze(
        &self,
        symptoms: &HashMap<String, f64>,
        rule_results: &[RcaResult],
        issue_description: &str,
    ) -> Result<LlmAnalysisResult, AnalyzerError> {
        let messages = self.build_prompt(symptoms, rule_results, issue_description);

        match self.client.complete(&messages).await {
            Ok(response) => Ok(LlmAnalysisResult {
                rule_results: rule_results.to_vec(),
                issue_description: issue_description.to_string(),
                llm_analysis: response,
                llm_success: true,
            }),
            Err(e) => {
                tracing::warn!(
                    "LLM analysis failed, falling back to rule-based only: {}",
                    e
                );
                Ok(LlmAnalysisResult {
                    rule_results: rule_results.to_vec(),
                    issue_description: issue_description.to_string(),
                    llm_analysis: format!("LLM unavailable: {e}. Using rule-based analysis only."),
                    llm_success: false,
                })
            }
        }
    }

    /// Build the prompt messages for the LLM.
    fn build_prompt(
        &self,
        symptoms: &HashMap<String, f64>,
        rule_results: &[RcaResult],
        issue_description: &str,
    ) -> Vec<Message> {
        let symptoms_str = symptoms
            .iter()
            .map(|(k, v)| format!("  - {}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        let rules_str = if rule_results.is_empty() {
            "  (no rules matched)".to_string()
        } else {
            rule_results
                .iter()
                .map(|r| {
                    format!(
                        "  - [{}] {} (confidence: {:.0}%): {} → Fix: {}",
                        r.severity,
                        r.rule_name,
                        r.confidence * 100.0,
                        r.description,
                        r.suggested_fix
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        let system_prompt = "You are an expert infrastructure operations and SRE assistant. \
            You analyze system symptoms and rule-based diagnostic results to identify root causes \
            and provide actionable fix suggestions. Be specific and technical. \
            Structure your response with: \
            1. Root Cause Analysis \
            2. Contributing Factors \
            3. Recommended Actions (prioritized) \
            4. Prevention Measures";

        let user_prompt = format!(
            "Analyze this infrastructure issue:\n\n\
             ## Issue Description\n{issue_description}\n\n\
             ## System Symptoms\n{symptoms_str}\n\n\
             ## Rule-Based Analysis Results\n{rules_str}\n\n\
             Based on the above, provide a detailed root cause analysis with specific fix suggestions."
        );

        vec![Message::system(system_prompt), Message::user(user_prompt)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Severity;
    use async_trait::async_trait;
    use futures_util::stream;
    use ops_pilot_sdk::llm::{LlmError, Message};
    use std::pin::Pin;

    /// A mock LLM client that returns a canned response.
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
        ) -> Result<
            Pin<Box<dyn futures_util::Stream<Item = Result<String, LlmError>> + Send>>,
            LlmError,
        > {
            let chunks: Vec<Result<String, LlmError>> = self
                .response
                .split_whitespace()
                .map(|w| Ok(format!("{w} ")))
                .collect();
            Ok(Box::pin(stream::iter(chunks)))
        }
    }

    /// A mock LLM client that always fails.
    struct FailingLlmClient;

    #[async_trait]
    impl LlmClient for FailingLlmClient {
        async fn complete(&self, _messages: &[Message]) -> Result<String, LlmError> {
            Err(LlmError::Api {
                status: 503,
                message: "service unavailable".into(),
            })
        }

        async fn complete_stream(
            &self,
            _messages: &[Message],
        ) -> Result<
            Pin<Box<dyn futures_util::Stream<Item = Result<String, LlmError>> + Send>>,
            LlmError,
        > {
            Err(LlmError::StreamClosed)
        }
    }

    fn sample_symptoms() -> HashMap<String, f64> {
        let mut m = HashMap::new();
        m.insert("cpu_percent".into(), 92.0);
        m.insert("memory_percent".into(), 88.0);
        m
    }

    fn sample_rule_results() -> Vec<RcaResult> {
        vec![RcaResult {
            rule_name: "memory_leak".into(),
            description: "High CPU combined with high memory usage".into(),
            severity: Severity::Critical,
            confidence: 1.0,
            suggested_fix: "Identify processes with growing RSS".into(),
            matched_symptoms: vec!["cpu_percent".into(), "memory_percent".into()],
        }]
    }

    #[tokio::test]
    async fn test_llm_analyzer_success() {
        let client = Arc::new(MockLlmClient {
            response: "Root cause: memory leak in worker process. Fix: restart workers.".into(),
        });
        let analyzer = LlmAnalyzer::new(client);

        let result = analyzer
            .analyze(&sample_symptoms(), &sample_rule_results(), "Server is slow")
            .await
            .unwrap();

        assert!(result.llm_success);
        assert!(result.llm_analysis.contains("memory leak"));
        assert_eq!(result.rule_results.len(), 1);
        assert_eq!(result.rule_results[0].rule_name, "memory_leak");
        assert_eq!(result.issue_description, "Server is slow");
    }

    #[tokio::test]
    async fn test_llm_analyzer_fallback_on_error() {
        let client = Arc::new(FailingLlmClient);
        let analyzer = LlmAnalyzer::new(client);

        let result = analyzer
            .analyze(&sample_symptoms(), &sample_rule_results(), "Disk full")
            .await
            .unwrap();

        assert!(!result.llm_success);
        assert!(result.llm_analysis.contains("LLM unavailable"));
        assert!(result.llm_analysis.contains("rule-based analysis only"));
        assert_eq!(result.rule_results.len(), 1);
    }

    #[tokio::test]
    async fn test_llm_analyzer_empty_rules() {
        let client = Arc::new(MockLlmClient {
            response: "No obvious issue from metrics.".into(),
        });
        let analyzer = LlmAnalyzer::new(client);

        let result = analyzer
            .analyze(&sample_symptoms(), &[], "Everything looks normal")
            .await
            .unwrap();

        assert!(result.llm_success);
        assert!(result.rule_results.is_empty());
    }

    #[tokio::test]
    async fn test_llm_analyzer_builds_correct_prompt() {
        let client = Arc::new(MockLlmClient {
            response: "ok".into(),
        });
        let analyzer = LlmAnalyzer::new(client);

        let mut symptoms = HashMap::new();
        symptoms.insert("disk_percent".into(), 95.0);

        let result = analyzer
            .analyze(&symptoms, &[], "Disk usage alarm triggered")
            .await
            .unwrap();

        assert!(result.llm_success);
        assert_eq!(result.issue_description, "Disk usage alarm triggered");
    }

    #[tokio::test]
    async fn test_analyzer_error_type_display() {
        let err = AnalyzerError::Llm(LlmError::Api {
            status: 500,
            message: "test".into(),
        });
        assert!(err.to_string().contains("LLM error"));
    }
}
