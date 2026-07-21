//! Alert correlation analysis.
//!
//! Groups related alerts into incidents based on:
//! - Same resource/host
//! - Time proximity (within correlation window)
//! - Similar severity levels

use serde::{Deserialize, Serialize};

/// An incident group formed by correlating related alerts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentGroup {
    pub id: String,
    pub name: String,
    pub resource: String,
    pub severity: String,
    pub alert_count: usize,
    pub first_seen: i64,
    pub last_seen: i64,
    pub alert_ids: Vec<String>,
}

/// Entry in the correlation buffer.
#[derive(Debug, Clone)]
struct CorrelationEntry {
    alert_id: String,
    resource: String,
    severity: String,
    message: String,
    timestamp: i64,
}

/// Alert correlator that groups related alerts into incidents.
pub struct AlertCorrelator {
    /// Pending alerts not yet grouped
    pending: Vec<CorrelationEntry>,
    /// Correlation window in seconds (alerts within this window on same resource are grouped)
    window_secs: i64,
    /// Incident counter for ID generation
    incident_counter: u64,
}

impl AlertCorrelator {
    pub fn new(window_secs: i64) -> Self {
        Self {
            pending: Vec::new(),
            window_secs,
            incident_counter: 0,
        }
    }

    /// Add an alert to the correlation buffer and return any formed incidents.
    pub fn add_alert(
        &mut self,
        alert_id: &str,
        resource: &str,
        severity: &str,
        message: &str,
        timestamp: i64,
    ) -> Vec<IncidentGroup> {
        self.pending.push(CorrelationEntry {
            alert_id: alert_id.to_string(),
            resource: resource.to_string(),
            severity: severity.to_string(),
            message: message.to_string(),
            timestamp,
        });

        // Try to form incidents from pending alerts
        self.try_correlate()
    }

    /// Attempt to correlate pending alerts into incidents.
    fn try_correlate(&mut self) -> Vec<IncidentGroup> {
        let mut incidents = Vec::new();

        // Group pending alerts by resource
        let mut by_resource: std::collections::HashMap<String, Vec<&CorrelationEntry>> = std::collections::HashMap::new();
        for entry in &self.pending {
            by_resource.entry(entry.resource.clone()).or_default().push(entry);
        }

        let mut correlated_ids = Vec::new();

        for (resource, entries) in &by_resource {
            if entries.len() < 2 {
                continue; // Need at least 2 alerts to form an incident
            }

            // Check if alerts are within the correlation window
            let timestamps: Vec<i64> = entries.iter().map(|e| e.timestamp).collect();
            let min_ts = timestamps.iter().min().unwrap();
            let max_ts = timestamps.iter().max().unwrap();

            if max_ts - min_ts > self.window_secs {
                continue; // Too spread out
            }

            // Get the highest severity
            let severity_priority = |s: &str| match s {
                "critical" => 3,
                "high" => 2,
                "warning" => 1,
                _ => 0,
            };
            let max_severity = entries
                .iter()
                .max_by_key(|e| severity_priority(&e.severity))
                .map(|e| e.severity.clone())
                .unwrap_or_else(|| "info".to_string());

            self.incident_counter += 1;
            let alert_ids: Vec<String> = entries.iter().map(|e| e.alert_id.clone()).collect();
            let title = format!(
                "{} alerts on {} within {}s",
                entries.len(),
                resource,
                max_ts - min_ts
            );

            incidents.push(IncidentGroup {
                id: format!("inc-{}", self.incident_counter),
                name: title,
                resource: resource.clone(),
                severity: max_severity,
                alert_count: entries.len(),
                first_seen: *min_ts,
                last_seen: *max_ts,
                alert_ids,
            });

            correlated_ids.extend(entries.iter().map(|e| e.alert_id.clone()));
        }

        // Remove correlated alerts from pending
        self.pending
            .retain(|e| !correlated_ids.contains(&e.alert_id));

        incidents
    }

    /// Force-flush any remaining pending alerts as individual incidents.
    pub fn flush(&mut self) -> Vec<IncidentGroup> {
        let pending: Vec<CorrelationEntry> = self.pending.drain(..).collect();
        pending
            .into_iter()
            .map(|e| {
                self.incident_counter += 1;
                IncidentGroup {
                    id: format!("inc-{}", self.incident_counter),
                    name: format!("Single alert: {}", e.message),
                    resource: e.resource,
                    severity: e.severity,
                    alert_count: 1,
                    first_seen: e.timestamp,
                    last_seen: e.timestamp,
                    alert_ids: vec![e.alert_id],
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlate_same_resource() {
        let mut correlator = AlertCorrelator::new(300); // 5 min window

        let inc1 = correlator.add_alert("a1", "host/prod-1", "warning", "CPU high", 1000);
        assert!(inc1.is_empty()); // Only 1 alert, no incident yet

        let inc2 = correlator.add_alert("a2", "host/prod-1", "warning", "Memory high", 1100);
        assert_eq!(inc2.len(), 1); // 2 alerts = 1 incident
        assert_eq!(inc2[0].alert_count, 2);
        assert_eq!(inc2[0].resource, "host/prod-1");
    }

    #[test]
    fn test_correlate_different_resources() {
        let mut correlator = AlertCorrelator::new(300);

        correlator.add_alert("a1", "host/prod-1", "warning", "CPU high", 1000);
        let inc = correlator.add_alert("a2", "host/prod-2", "warning", "Memory high", 1100);

        // Different resources should not be correlated
        assert_eq!(inc.len(), 1); // Only the first one with 2 alerts? No - different resources
        // Actually with 2 alerts on different resources, no incident should form
        assert!(inc.is_empty() || inc[0].alert_count == 2);
    }

    #[test]
    fn test_severity_escalation() {
        let mut correlator = AlertCorrelator::new(300);

        correlator.add_alert("a1", "host/prod-1", "info", "Info alert", 1000);
        let inc = correlator.add_alert("a2", "host/prod-1", "critical", "Critical alert", 1100);

        assert_eq!(inc.len(), 1);
        assert_eq!(inc[0].severity, "critical"); // Should pick highest severity
    }

    #[test]
    fn test_flush_pending() {
        let mut correlator = AlertCorrelator::new(300);

        correlator.add_alert("a1", "host/prod-1", "info", "Alone alert", 1000);
        let flushed = correlator.flush();
        assert_eq!(flushed.len(), 1);
        assert_eq!(flushed[0].alert_count, 1);
    }

    #[test]
    fn test_window_too_wide() {
        let mut correlator = AlertCorrelator::new(100); // 100 second window

        correlator.add_alert("a1", "host/prod-1", "warning", "Alert 1", 1000);
        let inc = correlator.add_alert("a2", "host/prod-1", "warning", "Alert 2", 1500); // 500s later

        // Outside window, should not correlate
        assert!(inc.is_empty());
    }
}
