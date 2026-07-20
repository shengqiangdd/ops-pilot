//! Predefined RCA analysis rules with metric threshold conditions.

use serde::{Deserialize, Serialize};

/// Severity level for an RCA finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    Warning,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "critical"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

/// Comparison operator for a metric threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Comparison {
    /// metric > threshold
    GreaterThan,
    /// metric >= threshold
    GreaterEqual,
    /// metric < threshold
    LessThan,
    /// metric <= threshold
    LessEqual,
}

impl Comparison {
    pub fn evaluate(&self, value: f64, threshold: f64) -> bool {
        match self {
            Comparison::GreaterThan => value > threshold,
            Comparison::GreaterEqual => value >= threshold,
            Comparison::LessThan => value < threshold,
            Comparison::LessEqual => value <= threshold,
        }
    }
}

/// A single condition: check if `metric` meets a threshold comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub metric: String,
    pub comparison: Comparison,
    pub threshold: f64,
}

/// A root-cause analysis rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RcaRule {
    pub name: String,
    pub description: String,
    pub conditions: Vec<Condition>,
    pub severity: Severity,
    pub suggested_fix: String,
}

impl RcaRule {
    /// Evaluate all conditions against the given symptoms.
    /// Returns the fraction of conditions met (0.0–1.0).
    /// A missing metric counts as a failed condition.
    pub fn evaluate(&self, symptoms: &std::collections::HashMap<String, f64>) -> f64 {
        if self.conditions.is_empty() {
            return 0.0;
        }

        let matched = self.conditions.iter().filter(|cond| {
            symptoms
                .get(&cond.metric)
                .is_some_and(|&v| cond.comparison.evaluate(v, cond.threshold))
        }).count();

        matched as f64 / self.conditions.len() as f64
    }
}

/// Build the built-in rule set.
pub fn builtin_rules() -> Vec<RcaRule> {
    vec![
        RcaRule {
            name: "memory_leak".into(),
            description: "High CPU combined with high memory usage suggests a possible memory leak".into(),
            conditions: vec![
                Condition {
                    metric: "cpu_percent".into(),
                    comparison: Comparison::GreaterThan,
                    threshold: 80.0,
                },
                Condition {
                    metric: "memory_percent".into(),
                    comparison: Comparison::GreaterThan,
                    threshold: 85.0,
                },
            ],
            severity: Severity::Critical,
            suggested_fix: "Identify processes with growing RSS, check for memory leaks in application code, consider restarting affected services".into(),
        },
        RcaRule {
            name: "disk_full_risk".into(),
            description: "Disk usage above 90% indicates imminent disk-full condition".into(),
            conditions: vec![
                Condition {
                    metric: "disk_percent".into(),
                    comparison: Comparison::GreaterThan,
                    threshold: 90.0,
                },
            ],
            severity: Severity::Critical,
            suggested_fix: "Clean up old logs, temp files, and unused Docker images. Expand volume if on cloud.".into(),
        },
        RcaRule {
            name: "network_issue".into(),
            description: "Network errors combined with high latency points to network degradation".into(),
            conditions: vec![
                Condition {
                    metric: "network_errors".into(),
                    comparison: Comparison::GreaterThan,
                    threshold: 10.0,
                },
                Condition {
                    metric: "latency_ms".into(),
                    comparison: Comparison::GreaterThan,
                    threshold: 200.0,
                },
            ],
            severity: Severity::Warning,
            suggested_fix: "Check network interface errors, verify DNS resolution, inspect firewall rules and routing tables".into(),
        },
        RcaRule {
            name: "container_instability".into(),
            description: "Multiple container restarts indicate container instability".into(),
            conditions: vec![
                Condition {
                    metric: "container_restarts".into(),
                    comparison: Comparison::GreaterThan,
                    threshold: 3.0,
                },
            ],
            severity: Severity::Warning,
            suggested_fix: "Check container logs for OOM kills, review resource limits, verify image health and entrypoint".into(),
        },
        RcaRule {
            name: "security_concern".into(),
            description: "Elevated authentication failures may indicate brute-force or misconfiguration".into(),
            conditions: vec![
                Condition {
                    metric: "auth_failures".into(),
                    comparison: Comparison::GreaterThan,
                    threshold: 5.0,
                },
            ],
            severity: Severity::Critical,
            suggested_fix: "Review auth logs, block offending IPs, verify credential rotation policies, check for compromised accounts".into(),
        },
        RcaRule {
            name: "memory_pressure".into(),
            description: "High memory with active swapping indicates memory pressure".into(),
            conditions: vec![
                Condition {
                    metric: "memory_percent".into(),
                    comparison: Comparison::GreaterThan,
                    threshold: 90.0,
                },
                Condition {
                    metric: "swap_percent".into(),
                    comparison: Comparison::GreaterThan,
                    threshold: 50.0,
                },
            ],
            severity: Severity::Warning,
            suggested_fix: "Identify top memory consumers, add swap space or RAM, tune OOM killer priorities".into(),
        },
        RcaRule {
            name: "io_bottleneck".into(),
            description: "High load average with low CPU suggests I/O wait bottleneck".into(),
            conditions: vec![
                Condition {
                    metric: "load_average".into(),
                    comparison: Comparison::GreaterThan,
                    threshold: 4.0,
                },
                Condition {
                    metric: "cpu_percent".into(),
                    comparison: Comparison::LessThan,
                    threshold: 30.0,
                },
            ],
            severity: Severity::Warning,
            suggested_fix: "Check disk I/O with iostat, identify heavy I/O processes, consider SSD upgrade or I/O scheduling tuning".into(),
        },
    ]
}
