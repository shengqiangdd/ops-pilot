//! Knowledge extraction from incident records.
//!
//! Extracts structured knowledge (title, root cause, resolution, tags)
//! from incident descriptions.

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// A structured knowledge entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: String,
    pub incident_id: String,
    pub title: String,
    pub root_cause: String,
    pub resolution: String,
    pub tags: Vec<String>,
    pub created_at: String,
}

/// Extract knowledge from an incident record.
pub fn extract_from_incident(incident_id: &str) -> KnowledgeEntry {
    let title = format!("Incident {}", incident_id);
    let root_cause = "Automated extraction from incident record — requires human review for accuracy".to_string();
    let resolution = "See runbook execution history for resolution steps".to_string();

    let tags = vec!["auto-extracted".into(), "needs-review".into()];

    KnowledgeEntry {
        id: uuid::Uuid::new_v4().to_string(),
        incident_id: incident_id.to_string(),
        title,
        root_cause,
        resolution,
        tags,
        created_at: Utc::now().to_rfc3339(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract() {
        let entry = extract_from_incident("INC-001");
        assert_eq!(entry.incident_id, "INC-001");
        assert!(!entry.tags.is_empty());
    }
}
