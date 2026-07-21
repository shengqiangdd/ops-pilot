//! Log analyzer for security threat detection.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub is_threat: bool,
    pub threat_type: Option<String>,
    pub severity: String,
    pub description: String,
    pub source_ip: Option<String>,
    pub confidence: f64,
}

pub struct LogAnalyzer;

impl LogAnalyzer {
    pub fn new() -> Self { Self }

    pub fn analyze_line(&self, line: &str, _source: &str) -> AnalysisResult {
        let lower = line.to_lowercase();

        // SSH brute force detection
        if lower.contains("failed password") || lower.contains("invalid user") {
            return AnalysisResult {
                is_threat: true,
                threat_type: Some("ssh_bruteforce".into()),
                severity: "high".into(),
                description: "SSH brute force attempt detected".into(),
                source_ip: extract_ip(line),
                confidence: 0.9,
            };
        }

        // Sudo authentication failure
        if lower.contains("sudo") && lower.contains("authentication failure") {
            return AnalysisResult {
                is_threat: true,
                threat_type: Some("sudo_abuse".into()),
                severity: "high".into(),
                description: "Sudo authentication failure detected".into(),
                source_ip: extract_ip(line),
                confidence: 0.85,
            };
        }

        // Web attack patterns
        if lower.contains("403") || lower.contains("sql injection") || lower.contains("xss") {
            return AnalysisResult {
                is_threat: true,
                threat_type: Some("web_attack".into()),
                severity: "medium".into(),
                description: "Potential web attack detected".into(),
                source_ip: extract_ip(line),
                confidence: 0.7,
            };
        }

        AnalysisResult {
            is_threat: false,
            threat_type: None,
            severity: "info".into(),
            description: "No threat detected".into(),
            source_ip: None,
            confidence: 0.1,
        }
    }

    pub fn check_ssh_bruteforce(&self, lines: &[String]) -> Vec<AnalysisResult> {
        lines.iter()
            .filter(|l| l.to_lowercase().contains("failed password") || l.to_lowercase().contains("invalid user"))
            .map(|l| self.analyze_line(l, "ssh"))
            .collect()
    }
}

fn extract_ip(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    for part in &parts {
        if part.parse::<std::net::Ipv4Addr>().is_ok() {
            return Some(part.to_string());
        }
    }
    None
}
