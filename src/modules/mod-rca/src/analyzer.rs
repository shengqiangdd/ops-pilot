//! Rule-based RCA engine: evaluates symptoms against rules and returns ranked root causes.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::rules::{builtin_rules, RcaRule, Severity};

/// A ranked root-cause result from analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RcaResult {
    pub rule_name: String,
    pub description: String,
    pub severity: Severity,
    /// Confidence score between 0.0 and 1.0 (fraction of conditions matched).
    pub confidence: f64,
    pub suggested_fix: String,
    pub matched_symptoms: Vec<String>,
}

/// A historical analysis entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RcaHistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub symptoms: HashMap<String, f64>,
    pub results: Vec<RcaResult>,
}

/// The RCA analyzer engine.
pub struct RcaAnalyzer {
    rules: Vec<RcaRule>,
    history: Vec<RcaHistoryEntry>,
}

impl RcaAnalyzer {
    /// Create a new analyzer with the built-in rule set.
    pub fn new() -> Self {
        Self {
            rules: builtin_rules(),
            history: Vec::new(),
        }
    }

    /// Return the list of loaded rules.
    pub fn rules(&self) -> &[RcaRule] {
        &self.rules
    }

    /// Return the analysis history.
    pub fn history(&self) -> &[RcaHistoryEntry] {
        &self.history
    }

    /// Analyze symptoms and return ranked root causes (highest confidence first).
    /// Results with confidence == 0.0 are excluded.
    pub fn analyze(&mut self, symptoms: HashMap<String, f64>) -> Vec<RcaResult> {
        let mut results: Vec<RcaResult> = Vec::new();

        for rule in &self.rules {
            let confidence = rule.evaluate(&symptoms);
            if confidence > 0.0 {
                let matched_symptoms: Vec<String> = rule
                    .conditions
                    .iter()
                    .filter_map(|c| {
                        symptoms.get(&c.metric).map(|_| c.metric.clone())
                    })
                    .collect();

                results.push(RcaResult {
                    rule_name: rule.name.clone(),
                    description: rule.description.clone(),
                    severity: rule.severity,
                    confidence,
                    suggested_fix: rule.suggested_fix.clone(),
                    matched_symptoms,
                });
            }
        }

        // Sort by confidence descending, then by severity (Critical first).
        results.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(severity_rank(&a.severity).cmp(&severity_rank(&b.severity)))
        });

        // Record in history.
        self.history.push(RcaHistoryEntry {
            timestamp: Utc::now(),
            symptoms,
            results: results.clone(),
        });

        results
    }
}

fn severity_rank(s: &Severity) -> u8 {
    match s {
        Severity::Critical => 0,
        Severity::Warning => 1,
        Severity::Info => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_detects_memory_leak_pattern() {
        let mut analyzer = RcaAnalyzer::new();
        let mut symptoms = HashMap::new();
        symptoms.insert("cpu_percent".into(), 92.0);
        symptoms.insert("memory_percent".into(), 88.0);

        let results = analyzer.analyze(symptoms);
        assert!(!results.is_empty());
        assert_eq!(results[0].rule_name, "memory_leak");
        assert!(results[0].confidence >= 1.0);
    }

    #[test]
    fn test_analyzer_detects_disk_full_risk() {
        let mut analyzer = RcaAnalyzer::new();
        let mut symptoms = HashMap::new();
        symptoms.insert("disk_percent".into(), 95.0);

        let results = analyzer.analyze(symptoms);
        assert!(!results.is_empty());
        assert_eq!(results[0].rule_name, "disk_full_risk");
        assert!(results[0].confidence >= 1.0);
    }

    #[test]
    fn test_analyzer_no_match_returns_empty() {
        let mut analyzer = RcaAnalyzer::new();
        let mut symptoms = HashMap::new();
        symptoms.insert("cpu_percent".into(), 50.0);
        symptoms.insert("memory_percent".into(), 40.0);
        symptoms.insert("disk_percent".into(), 30.0);

        let results = analyzer.analyze(symptoms);
        assert!(results.is_empty());
    }

    #[test]
    fn test_rca_rules_lists_all_rules() {
        let analyzer = RcaAnalyzer::new();
        let rules = analyzer.rules();
        assert_eq!(rules.len(), 7);
        let names: Vec<&str> = rules.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"memory_leak"));
        assert!(names.contains(&"disk_full_risk"));
        assert!(names.contains(&"network_issue"));
        assert!(names.contains(&"container_instability"));
        assert!(names.contains(&"security_concern"));
        assert!(names.contains(&"memory_pressure"));
        assert!(names.contains(&"io_bottleneck"));
    }

    #[test]
    fn test_rca_analyze_returns_ranked_results() {
        let mut analyzer = RcaAnalyzer::new();
        let mut symptoms = HashMap::new();
        // Trigger multiple rules: memory_leak (2 conditions) + disk_full_risk (1 condition)
        symptoms.insert("cpu_percent".into(), 90.0);
        symptoms.insert("memory_percent".into(), 92.0);
        symptoms.insert("disk_percent".into(), 95.0);

        let results = analyzer.analyze(symptoms);
        assert!(results.len() >= 3); // memory_leak + memory_pressure + disk_full_risk

        // memory_leak has 2/2 = 1.0 confidence, should be first
        assert_eq!(results[0].rule_name, "memory_leak");
        assert!((results[0].confidence - 1.0).abs() < f64::EPSILON);

        // Results should be sorted by confidence descending
        for i in 1..results.len() {
            assert!(results[i].confidence <= results[i - 1].confidence);
        }
    }

    #[test]
    fn test_analyzer_records_history() {
        let mut analyzer = RcaAnalyzer::new();
        let mut symptoms = HashMap::new();
        symptoms.insert("disk_percent".into(), 95.0);

        analyzer.analyze(symptoms);
        assert_eq!(analyzer.history().len(), 1);
        assert!(!analyzer.history()[0].results.is_empty());
    }
}
