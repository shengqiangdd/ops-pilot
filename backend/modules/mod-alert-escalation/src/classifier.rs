//! Alert classification engine.
//!
//! Combines rule-based heuristics with simple ML-like features (frequency,
//! time patterns, history) to assign severity labels and suggest actions.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Classification result for an alert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classification {
    pub severity: String,
    pub confidence: f64,
    pub suggested_action: String,
    pub tags: Vec<String>,
}

/// Alert classifier that uses rules + frequency analysis.
pub struct AlertClassifier {
    /// Recent alert timestamps per resource (for frequency analysis)
    resource_frequency: HashMap<String, Vec<i64>>,
}

impl Default for AlertClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertClassifier {
    pub fn new() -> Self {
        Self {
            resource_frequency: HashMap::new(),
        }
    }

    /// Classify an alert based on its message, resource, and context.
    pub fn classify(&mut self, message: &str, resource: &str, timestamp: i64) -> Classification {
        let mut tags = Vec::new();
        let mut severity_score: f64 = 0.0; // 0.0 = info, 0.5 = warning, 1.0 = critical

        // Rule 1: Keyword-based severity
        let msg_lower = message.to_lowercase();
        if msg_lower.contains("critical") || msg_lower.contains("fatal") || msg_lower.contains("panic") {
            severity_score += 0.5;
            tags.push("critical_keyword".into());
        }
        if msg_lower.contains("error") || msg_lower.contains("failure") || msg_lower.contains("failed") {
            severity_score += 0.3;
            tags.push("error_keyword".into());
        }
        if msg_lower.contains("warning") || msg_lower.contains("degraded") {
            severity_score += 0.2;
            tags.push("warning_keyword".into());
        }

        // Rule 2: Resource frequency (burst detection)
        let timestamps = self.resource_frequency
            .entry(resource.to_string())
            .or_default();
        timestamps.push(timestamp);
        // Keep only last 60 entries
        let window_start = timestamp - 3600; // 1 hour
        timestamps.retain(|&t| t > window_start);
        let frequency = timestamps.len();
        if frequency > 10 {
            severity_score += 0.2;
            tags.push("high_frequency".into());
        }
        if frequency > 20 {
            severity_score += 0.2;
            tags.push("burst_detected".into());
        }

        // Rule 3: Time-based (night alerts are more suspicious)
        let hour = (timestamp % 86400) / 3600;
        if (0..6).contains(&hour) {
            severity_score += 0.1;
            tags.push("night_time".into());
        }

        // Determine severity
        let (severity, suggested_action) = if severity_score >= 0.7 {
            ("critical".to_string(), "Immediate investigation required. Consider escalating to on-call engineer.".to_string())
        } else if severity_score >= 0.4 {
            ("warning".to_string(), "Monitor closely. Review within the next hour.".to_string())
        } else {
            ("info".to_string(), "No immediate action required. Log for reference.".to_string())
        };

        let confidence = (severity_score.min(1.0) * 100.0).round() / 100.0;

        Classification {
            severity,
            confidence,
            suggested_action,
            tags,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_critical() {
        let mut classifier = AlertClassifier::new();
        // Pre-seed 12 alerts on the same resource to trigger high_frequency (+0.2)
        let ts = 1730000000; // Non-night timestamp (hour ~= 17)
        for i in 0..12 {
            classifier.classify("something happened", "host/prod-1", ts + i);
        }
        // Now classify critical alert: keyword(+0.5) + high_frequency(+0.2) = 0.7 → critical
        let result = classifier.classify("CRITICAL: Service down", "host/prod-1", ts + 12);
        assert_eq!(result.severity, "critical");
        assert!(result.confidence >= 0.7);
    }

    #[test]
    fn test_classify_info() {
        let mut classifier = AlertClassifier::new();
        let result = classifier.classify("Routine check passed", "host/prod-1", 1000);
        assert_eq!(result.severity, "info");
    }

    #[test]
    fn test_frequency_detection() {
        let mut classifier = AlertClassifier::new();
        let now = 1000000;
        // Simulate 15 alerts in quick succession
        for i in 0..15 {
            classifier.classify("test alert", "host/prod-1", now + i);
        }
        let result = classifier.classify("test alert", "host/prod-1", now + 15);
        assert!(result.tags.contains(&"high_frequency".to_string()));
    }

    #[test]
    fn test_night_time_tag() {
        let mut classifier = AlertClassifier::new();
        // 2 AM in seconds from midnight
        let timestamp = 2 * 3600;
        let result = classifier.classify("test", "host/prod-1", timestamp);
        assert!(result.tags.contains(&"night_time".to_string()));
    }
}
