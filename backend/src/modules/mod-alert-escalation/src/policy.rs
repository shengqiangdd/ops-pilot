//! Escalation policy definitions and engine.
//!
//! Defines how alerts escalate through severity levels, with configurable
//! delays, notification channels, and auto-escalation rules.

use serde::{Deserialize, Serialize};

/// An escalation policy for a specific severity level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationPolicy {
    /// Human-readable name (e.g. "Critical PagerDuty").
    pub name: String,
    /// Severity level this policy applies to (P1–P4).
    pub severity: String,
    /// Minutes to wait before escalating if unacknowledged.
    pub delay_minutes: u32,
    /// Notification channels (webhook, sms, email, pagerduty, chatops).
    pub channels: Vec<String>,
}

/// Default escalation policies for P1–P4.
pub fn default_policies() -> Vec<EscalationPolicy> {
    vec![
        EscalationPolicy {
            name: "P1 - Immediate".into(),
            severity: "P1".into(),
            delay_minutes: 0,
            channels: vec!["sms".into(), "pagerduty".into(), "webhook".into()],
        },
        EscalationPolicy {
            name: "P2 - Urgent".into(),
            severity: "P2".into(),
            delay_minutes: 5,
            channels: vec!["pagerduty".into(), "webhook".into()],
        },
        EscalationPolicy {
            name: "P3 - Normal".into(),
            severity: "P3".into(),
            delay_minutes: 30,
            channels: vec!["email".into(), "webhook".into()],
        },
        EscalationPolicy {
            name: "P4 - Low".into(),
            severity: "P4".into(),
            delay_minutes: 60,
            channels: vec!["email".into()],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policies() {
        let policies = default_policies();
        assert_eq!(policies.len(), 4);
        assert_eq!(policies[0].severity, "P1");
        assert_eq!(policies[0].delay_minutes, 0);
        assert!(policies[0].channels.contains(&"sms".to_string()));
    }
}
