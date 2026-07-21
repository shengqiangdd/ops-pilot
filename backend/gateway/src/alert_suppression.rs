//! Alert suppression and aggregation.
//!
//! Suppresses duplicate alerts within a configurable time window and
//! aggregates similar alerts into a single notification with count.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{Utc, Duration};

/// Key used to group similar alerts.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct SuppressionKey {
    host: String,
    alert_type: String,
    severity: String,
}

/// Tracked alert state.
#[derive(Debug, Clone)]
struct AlertState {
    first_seen: chrono::DateTime<Utc>,
    last_seen: chrono::DateTime<Utc>,
    count: u32,
    suppressed: bool,
    #[allow(dead_code)]
    message: String,
}

/// Alert suppression engine.
#[derive(Clone)]
pub struct AlertSuppressor {
    window_minutes: i64,
    max_suppress_count: u32,
    state: Arc<RwLock<HashMap<SuppressionKey, AlertState>>>,
}

impl AlertSuppressor {
    pub fn new(window_minutes: i64, max_suppress_count: u32) -> Self {
        Self {
            window_minutes,
            max_suppress_count,
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if an alert should be suppressed.
    /// Returns `true` if the alert is a duplicate (will be aggregated).
    /// Returns `false` if the alert should be dispatched normally.
    pub async fn should_suppress(
        &self,
        host: &str,
        alert_type: &str,
        severity: &str,
        message: &str,
    ) -> bool {
        let key = SuppressionKey {
            host: host.to_string(),
            alert_type: alert_type.to_string(),
            severity: severity.to_string(),
        };

        let now = Utc::now();

        // Check existing state outside the lock to avoid borrow conflicts.
        // We use a separate flag/map entry decision approach.
        let should_aggregate = {
            let state = self.state.read().await;
            if let Some(existing) = state.get(&key) {
                if now - existing.first_seen < Duration::minutes(self.window_minutes) {
                    Some(existing.clone())
                } else {
                    None
                }
            } else {
                None
            }
        };

        let mut state = self.state.write().await;

        if let Some(mut existing) = should_aggregate {
            // Within window — suppress and aggregate
            existing.last_seen = now;
            existing.count += 1;
            // After max_suppress_count, allow through as a summary
            if existing.count >= self.max_suppress_count {
                // Reset counter and let this one through as aggregation summary
                state.insert(key, AlertState {
                    first_seen: now,
                    last_seen: now,
                    count: 1,
                    suppressed: true,
                    message: format!("[AGGREGATED] Alert '{}' fired {} times on {} since {} — last: {}",
                        alert_type, existing.count, host, existing.first_seen.format("%H:%M:%S"), message),
                });
                return false; // let the summary through
            }
            state.insert(key, existing);
            return true; // suppress
        }

        // New alert or outside window
        state.insert(key, AlertState {
            first_seen: now,
            last_seen: now,
            count: 1,
            suppressed: false,
            message: message.to_string(),
        });
        false
    }

    /// Get current suppression stats.
    pub async fn stats(&self) -> HashMap<String, serde_json::Value> {
        let state = self.state.read().await;
        let mut result = HashMap::new();
        let mut total_suppressed = 0u32;
        let mut active_groups = 0u32;

        for val in state.values() {
            if val.suppressed || val.count > 1 {
                total_suppressed += val.count;
                active_groups += 1;
            }
        }

        result.insert("total_suppressed".to_string(), serde_json::Value::Number(total_suppressed.into()));
        result.insert("active_groups".to_string(), serde_json::Value::Number(active_groups.into()));
        result
    }
}
