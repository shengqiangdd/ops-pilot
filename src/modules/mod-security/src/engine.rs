//! Security scanning engine: CIS benchmarks, vulnerability rules, and compliance checks.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::rules::{builtin_rules, SecurityRule, Severity};

/// The outcome of a single compliance check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckStatus {
    Pass,
    Fail,
    Warn,
}

impl std::fmt::Display for CheckStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckStatus::Pass => write!(f, "pass"),
            CheckStatus::Fail => write!(f, "fail"),
            CheckStatus::Warn => write!(f, "warn"),
        }
    }
}

/// Result of evaluating a single security rule against a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub check_id: String,
    pub rule_name: String,
    pub status: CheckStatus,
    pub actual_value: String,
    pub expected_value: String,
    pub severity: Severity,
    pub category: String,
    pub remediation_steps: String,
}

/// Aggregated scan output for a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanOutput {
    pub host_id: String,
    pub check_type: String,
    pub timestamp: DateTime<Utc>,
    pub total_checks: usize,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
    pub results: Vec<ScanResult>,
}

/// The security scanning engine.
pub struct SecurityEngine {
    rules: Vec<SecurityRule>,
}

impl SecurityEngine {
    pub fn new() -> Self {
        Self {
            rules: builtin_rules(),
        }
    }

    /// Return all loaded rules.
    pub fn rules(&self) -> &[SecurityRule] {
        &self.rules
    }

    /// Run a scan for the given check type and optional check IDs.
    ///
    /// If `check_ids` is empty, all rules matching `check_type` are evaluated.
    /// In a production deployment this would SSH into the host and inspect real state;
    /// here we simulate deterministic results for testing.
    pub fn scan(&self, host_id: &str, check_type: &str, check_ids: &[String]) -> ScanOutput {
        let applicable: Vec<&SecurityRule> = if check_ids.is_empty() {
            self.rules
                .iter()
                .filter(|r| r.matches_check_type(check_type))
                .collect()
        } else {
            self.rules
                .iter()
                .filter(|r| check_ids.contains(&r.id))
                .collect()
        };

        let mut results: Vec<ScanResult> = applicable
            .iter()
            .map(|rule| {
                let (status, actual, _expected) = rule.simulate_result();
                ScanResult {
                    check_id: rule.id.clone(),
                    rule_name: rule.name.clone(),
                    status,
                    actual_value: actual,
                    expected_value: rule.expected_value.clone(),
                    severity: rule.severity,
                    category: rule.category.clone(),
                    remediation_steps: rule.remediation_steps.clone(),
                }
            })
            .collect();

        // Sort: Fail first, then Warn, then Pass; within same status by severity.
        results.sort_by(|a, b| {
            status_rank(&a.status)
                .cmp(&status_rank(&b.status))
                .then(severity_rank(&a.severity).cmp(&severity_rank(&b.severity)))
        });

        let passed = results.iter().filter(|r| r.status == CheckStatus::Pass).count();
        let failed = results.iter().filter(|r| r.status == CheckStatus::Fail).count();
        let warnings = results.iter().filter(|r| r.status == CheckStatus::Warn).count();

        ScanOutput {
            host_id: host_id.to_string(),
            check_type: check_type.to_string(),
            timestamp: Utc::now(),
            total_checks: results.len(),
            passed,
            failed,
            warnings,
            results,
        }
    }
}

/// Simulate remediation for a given check.
pub fn simulate_remediate(check_id: &str, rules: &[SecurityRule]) -> RemediationOutput {
    match rules.iter().find(|r| r.id == check_id) {
        Some(rule) => RemediationOutput {
            check_id: check_id.to_string(),
            status: "remediated".to_string(),
            details: format!(
                "Applied fix for '{}': {}",
                rule.name, rule.remediation_steps
            ),
        },
        None => RemediationOutput {
            check_id: check_id.to_string(),
            status: "error".to_string(),
            details: format!("Unknown check ID: {check_id}"),
        },
    }
}

/// Output from a remediation attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationOutput {
    pub check_id: String,
    pub status: String,
    pub details: String,
}

fn status_rank(s: &CheckStatus) -> u8 {
    match s {
        CheckStatus::Fail => 0,
        CheckStatus::Warn => 1,
        CheckStatus::Pass => 2,
    }
}

fn severity_rank(s: &Severity) -> u8 {
    match s {
        Severity::Critical => 0,
        Severity::High => 1,
        Severity::Medium => 2,
        Severity::Low => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_loads_rules() {
        let engine = SecurityEngine::new();
        assert!(engine.rules().len() >= 15);
    }

    #[test]
    fn test_scan_returns_results() {
        let engine = SecurityEngine::new();
        let output = engine.scan("host-1", "all", &[]);
        assert_eq!(output.host_id, "host-1");
        assert_eq!(output.check_type, "all");
        assert!(!output.results.is_empty());
        assert_eq!(
            output.total_checks,
            output.passed + output.failed + output.warnings
        );
    }

    #[test]
    fn test_scan_filters_by_check_type() {
        let engine = SecurityEngine::new();
        let output = engine.scan("host-1", "vulnerability", &[]);
        for r in &output.results {
            assert_eq!(r.category, "vulnerability");
        }
    }

    #[test]
    fn test_scan_selective_check_ids() {
        let engine = SecurityEngine::new();
        let ids = vec!["CIS-1.1".to_string(), "CIS-2.1".to_string()];
        let output = engine.scan("host-1", "all", &ids);
        assert_eq!(output.results.len(), 2);
    }

    #[test]
    fn test_scan_results_sorted() {
        let engine = SecurityEngine::new();
        let output = engine.scan("host-1", "all", &[]);
        for window in output.results.windows(2) {
            let a = status_rank(&window[0].status).cmp(&status_rank(&window[1].status));
            let b = severity_rank(&window[0].severity).cmp(&severity_rank(&window[1].severity));
            assert!(a.then(b) != std::cmp::Ordering::Greater);
        }
    }

    #[test]
    fn test_remediate_valid_check() {
        let engine = SecurityEngine::new();
        let out = simulate_remediate("CIS-1.1", engine.rules());
        assert_eq!(out.status, "remediated");
    }

    #[test]
    fn test_remediate_invalid_check() {
        let engine = SecurityEngine::new();
        let out = simulate_remediate("NONEXISTENT", engine.rules());
        assert_eq!(out.status, "error");
    }
}
