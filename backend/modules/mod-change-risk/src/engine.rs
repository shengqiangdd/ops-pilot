//! Change risk assessment engine with heuristic scoring.

use chrono::Timelike;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRequest {
    pub resource: String,
    pub change_type: String,
    pub description: String,
    pub affected_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub score: f64,
    pub level: String,
    pub factors: Vec<RiskFactor>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub category: String,
    pub impact: String,
    pub probability: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub change_ids: Vec<String>,
    pub resource: String,
    pub conflict_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalDecision {
    pub approved: bool,
    pub reason: String,
    pub required_approvers: Vec<String>,
}

pub struct ChangeRiskEngine {
    db: SqlitePool,
}

impl ChangeRiskEngine {
    pub fn new(db: SqlitePool) -> Self { Self { db } }

    pub async fn assess(&self, resource: &str, change_type: &str, description: &str, affected_services: &[String]) -> RiskAssessment {
        let mut score = 0.3;
        let mut factors = Vec::new();

        // Factor 1: Change type risk
        let type_factor = match change_type {
            "config_change" => { factors.push(RiskFactor { category: "change_type".into(), impact: "high".into(), probability: "medium".into(), description: "Configuration change".into() }); 0.3 }
            "deployment" => { factors.push(RiskFactor { category: "change_type".into(), impact: "high".into(), probability: "medium".into(), description: "Deployment change".into() }); 0.25 }
            "restart" => { factors.push(RiskFactor { category: "change_type".into(), impact: "medium".into(), probability: "low".into(), description: "Service restart".into() }); 0.15 }
            _ => { factors.push(RiskFactor { category: "change_type".into(), impact: "low".into(), probability: "low".into(), description: "Read-only change".into() }); 0.05 }
        };
        score += type_factor;

        // Factor 2: Impact scope
        let impact_score = (affected_services.len() as f64 * 0.05).min(0.25);
        if affected_services.len() > 2 {
            factors.push(RiskFactor { category: "impact_scope".into(), impact: "high".into(), probability: "high".into(), description: format!("Affects {} services", affected_services.len()) });
        }
        score += impact_score;

        // Factor 3: Time window
        let hour = chrono::Utc::now().hour();
        if hour >= 2 && hour <= 5 {
            score -= 0.1;
            factors.push(RiskFactor { category: "timing".into(), impact: "low".into(), probability: "low".into(), description: "Off-hours deployment (lower risk)".into() });
        } else if hour >= 9 && hour <= 17 {
            score += 0.1;
            factors.push(RiskFactor { category: "timing".into(), impact: "medium".into(), probability: "medium".into(), description: "Business hours deployment".into() });
        }

        // Factor 4: Description keywords
        let desc_lower = description.to_lowercase();
        if desc_lower.contains("emergency") || desc_lower.contains("urgent") {
            score += 0.15;
            factors.push(RiskFactor { category: "urgency".into(), impact: "high".into(), probability: "medium".into(), description: "Urgent/emergency change".into() });
        }

        score = score.clamp(0.0, 1.0);
        let level = if score >= 0.7 { "critical" } else if score >= 0.4 { "high" } else if score >= 0.2 { "medium" } else { "low" }.to_string();

        let recommendation = if score >= 0.7 {
            "Requires senior approval. Consider staged rollout and rollback plan.".into()
        } else if score >= 0.4 {
            "Review carefully. Ensure rollback plan exists.".into()
        } else {
            "Standard review process is sufficient.".into()
        };

        RiskAssessment { score, level, factors, recommendation }
    }

    pub async fn check_conflicts(&self, changes: &[ChangeRequest]) -> Vec<Conflict> {
        let mut conflicts = Vec::new();
        let mut by_resource: std::collections::HashMap<String, Vec<&ChangeRequest>> = std::collections::HashMap::new();
        for c in changes {
            by_resource.entry(c.resource.clone()).or_default().push(c);
        }
        for (resource, group) in &by_resource {
            if group.len() > 1 {
                conflicts.push(Conflict {
                    change_ids: group.iter().map(|c| c.resource.clone()).collect(),
                    resource: resource.clone(),
                    conflict_type: "concurrent_changes".into(),
                    description: format!("{} concurrent changes on {}", group.len(), resource),
                });
            }
        }
        conflicts
    }

    pub fn auto_approve(&self, score: f64, level: &str) -> ApprovalDecision {
        match level {
            "low" => ApprovalDecision { approved: true, reason: "Low risk change auto-approved".into(), required_approvers: vec![] },
            "medium" => ApprovalDecision { approved: score < 0.3, reason: if score < 0.3 { "Medium risk within acceptable threshold".into() } else { "Requires manual review".into() }, required_approvers: vec!["team-lead".into()] },
            _ => ApprovalDecision { approved: false, reason: "High/critical risk requires manual approval".into(), required_approvers: vec!["senior-engineer".into(), "team-lead".into()] },
        }
    }
}
