//! Container image security scanner.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageScanResult {
    pub image_name: String,
    pub issues: Vec<ImageIssue>,
    pub score: f64,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageIssue {
    pub check: String,
    pub severity: String,
    pub description: String,
    pub recommendation: String,
}

pub struct ImageScanner;

impl ImageScanner {
    pub fn new() -> Self { Self }

    pub fn scan(&self, image_name: &str) -> ImageScanResult {
        let mut issues = Vec::new();

        // Check for latest tag
        if image_name.ends_with(":latest") || !image_name.contains(':') {
            issues.push(ImageIssue {
                check: "latest_tag".into(),
                severity: "medium".into(),
                description: "Image uses 'latest' tag or no tag specified".into(),
                recommendation: "Use specific version tags for reproducibility".into(),
            });
        }

        // Check for common security issues based on image name patterns
        let lower = image_name.to_lowercase();
        if lower.contains("root") || lower.contains("admin") {
            issues.push(ImageIssue {
                check: "root_user".into(),
                severity: "high".into(),
                description: "Image may run as root user".into(),
                recommendation: "Use non-root user in Dockerfile (USER directive)".into(),
            });
        }

        // Check for known vulnerable base images
        if lower.contains("ubuntu:18") || lower.contains("centos:6") || lower.contains("alpine:3.6") {
            issues.push(ImageIssue {
                check: "outdated_base".into(),
                severity: "critical".into(),
                description: "Image uses outdated base with known vulnerabilities".into(),
                recommendation: "Update to a supported base image version".into(),
            });
        }

        // Score calculation
        let score = if issues.is_empty() {
            100.0
        } else {
            let critical_count = issues.iter().filter(|i| i.severity == "critical").count();
            let high_count = issues.iter().filter(|i| i.severity == "high").count();
            (100.0 - (critical_count as f64 * 30.0) - (high_count as f64 * 15.0)).max(0.0)
        };

        let risk_level = if score < 50.0 { "critical" } else if score < 75.0 { "warning" } else { "ok" }.to_string();

        ImageScanResult { image_name: image_name.to_string(), issues, score, risk_level }
    }
}
