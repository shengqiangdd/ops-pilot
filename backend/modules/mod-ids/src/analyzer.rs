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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_bruteforce_detection() {
        let analyzer = LogAnalyzer::new();
        let result = analyzer.analyze_line(
            "Failed password for root from 192.168.1.100 port 22 ssh2",
            "ssh",
        );
        assert!(result.is_threat);
        assert_eq!(result.threat_type.as_deref(), Some("ssh_bruteforce"));
        assert_eq!(result.source_ip.as_deref(), Some("192.168.1.100"));
        assert_eq!(result.severity, "high");
    }

    #[test]
    fn test_sudo_failure_detection() {
        let analyzer = LogAnalyzer::new();
        let result = analyzer.analyze_line(
            "sudo: pam_unix(sudo:auth): authentication failure; logname=admin uid=1000",
            "syslog",
        );
        assert!(result.is_threat);
        assert_eq!(result.threat_type.as_deref(), Some("sudo_abuse"));
        assert_eq!(result.severity, "high");
    }

    #[test]
    fn test_web_attack_403() {
        let analyzer = LogAnalyzer::new();
        let result = analyzer.analyze_line(
            "GET /admin HTTP/1.1\" 403 1234",
            "web",
        );
        assert!(result.is_threat);
        assert_eq!(result.threat_type.as_deref(), Some("web_attack"));
    }

    #[test]
    fn test_normal_log_no_false_positive() {
        let analyzer = LogAnalyzer::new();
        let result = analyzer.analyze_line(
            "Service nginx started successfully on port 80",
            "syslog",
        );
        assert!(!result.is_threat);
        assert_eq!(result.threat_type, None);
        assert_eq!(result.severity, "info");
    }

    #[test]
    fn test_extract_ip_found() {
        let ip = extract_ip("Connection from 10.0.0.5 port 443");
        assert_eq!(ip.as_deref(), Some("10.0.0.5"));
    }

    #[test]
    fn test_extract_ip_none() {
        let ip = extract_ip("No IP address in this line");
        assert_eq!(ip, None);
    }

    #[test]
    fn test_check_ssh_bruteforce_batch() {
        let analyzer = LogAnalyzer::new();
        let lines = vec![
            "Failed password for admin from 1.2.3.4 port 22".to_string(),
            "Accepted publickey for user1 from 5.6.7.8 port 22".to_string(),
            "Invalid user hacker from 9.9.9.9 port 22".to_string(),
        ];
        let results = analyzer.check_ssh_bruteforce(&lines);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_threat));
    }

    #[test]
    fn test_invalid_user_ssh() {
        let analyzer = LogAnalyzer::new();
        let result = analyzer.analyze_line(
            "Failed password for invalid user test from 192.168.0.1 port 22",
            "ssh",
        );
        assert!(result.is_threat);
        assert_eq!(result.threat_type.as_deref(), Some("ssh_bruteforce"));
    }
}
