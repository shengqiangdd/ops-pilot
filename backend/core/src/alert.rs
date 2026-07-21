//! Audit alert engine — rule-based anomaly detection over audit events.
//!
//! Subscribes to `EventBus::AuditLog` events and maintains sliding windows
//! for detection. Rules include night-batch operations, high failure rates,
//! and first-time host connections.
//!
//! ## Optimizations
//! - Sliding window TTL: entries older than max_window_minutes are pruned on each insert
//! - Window size cap: each user's deque is capped at MAX_WINDOW_SIZE entries
//! - Alert deduplication: identical alert messages within merge_window_secs are suppressed

use std::collections::{HashSet, VecDeque, HashMap};
use std::hash::{Hash, Hasher, DefaultHasher};

use dashmap::DashMap;
use ops_pilot_sdk::events::{global_event_bus, OpsEvent};
use serde::{Deserialize, Serialize};

use crate::audit::AuditEntry;

/// Maximum number of entries kept per user in a sliding window.
const MAX_WINDOW_SIZE: usize = 1000;

/// Severity of an alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// A detected alert event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub rule_name: String,
    pub user: String,
    pub message: String,
    pub severity: AlertSeverity,
    pub created_at: i64,
}

/// Alert detection rules.
#[derive(Debug, Clone)]
pub enum AlertRule {
    /// Night batch: same user performs >= threshold operations on distinct hosts
    /// within window_minutes during 00:00-06:00.
    NightBatch {
        threshold: usize,
        window_minutes: i64,
    },
    /// High failure: same user has >= threshold failed operations within window_minutes.
    HighFailure {
        threshold: usize,
        window_minutes: i64,
    },
    /// First connect: user connects to a host for the first time.
    FirstConnect,
}

impl AlertRule {
    pub fn name(&self) -> &'static str {
        match self {
            Self::NightBatch { .. } => "night_batch",
            Self::HighFailure { .. } => "high_failure",
            Self::FirstConnect => "first_connect",
        }
    }

    /// Maximum window minutes for this rule (used for TTL pruning).
    pub fn window_minutes(&self) -> i64 {
        match self {
            Self::NightBatch { window_minutes, .. } => *window_minutes,
            Self::HighFailure { window_minutes, .. } => *window_minutes,
            Self::FirstConnect => 0, // No sliding window needed
        }
    }
}

/// Sliding window entry for a user.
#[derive(Clone)]
struct WindowEntry {
    timestamp: i64,
    outcome: String,
    resource: String,
}

/// Alert deduplicator — suppresses identical alert messages within a time window.
pub struct AlertDeduplicator {
    /// message_fingerprint → (count, first_seen_timestamp)
    recent: HashMap<u64, (usize, i64)>,
    /// Merge window in seconds: same fingerprint within this window is suppressed
    merge_window_secs: i64,
}

impl AlertDeduplicator {
    pub fn new(merge_window_secs: i64) -> Self {
        Self {
            recent: HashMap::new(),
            merge_window_secs,
        }
    }

    /// Create a simple fingerprint from an alert message.
    /// Uses the raw message content hashed via SipHash.
    fn fingerprint(message: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        message.hash(&mut hasher);
        hasher.finish()
    }

    /// Returns true if this alert message is a duplicate within the merge window.
    pub fn is_duplicate(&mut self, message: &str, now: i64) -> bool {
        let fp = Self::fingerprint(message);
        let entry = self.recent.entry(fp).or_insert((0, now));
        entry.0 += 1;

        // If first occurrence or outside merge window, not a duplicate
        if entry.0 == 1 || (now - entry.1) > self.merge_window_secs {
            entry.1 = now;
            entry.0 = 1;
            return false;
        }
        // Second or more occurrence within merge window = duplicate
        true
    }

    /// Prune entries older than 2x the merge window.
    pub fn prune(&mut self, now: i64) {
        let cutoff = now - self.merge_window_secs * 2;
        self.recent.retain(|_, (_, ts)| *ts > cutoff);
    }
}

/// Alert engine with in-memory sliding windows.
pub struct AlertEngine {
    rules: Vec<AlertRule>,
    /// user_id → deque of (timestamp, action, outcome, resource)
    windows: DashMap<String, VecDeque<WindowEntry>>,
    /// user_id → set of known host resources (for FirstConnect)
    known_hosts: DashMap<String, HashSet<String>>,
    /// Alert deduplication fingerprints (message_hash → count)
    dedup_fingerprints: DashMap<u64, (usize, i64)>,
}

impl AlertEngine {
    /// Create a new alert engine with default rules.
    pub fn new() -> Self {
        Self {
            rules: vec![
                AlertRule::NightBatch {
                    threshold: 5,
                    window_minutes: 60,
                },
                AlertRule::HighFailure {
                    threshold: 10,
                    window_minutes: 5,
                },
                AlertRule::FirstConnect,
            ],
            windows: DashMap::new(),
            known_hosts: DashMap::new(),
            dedup_fingerprints: DashMap::new(),
        }
    }

    /// Create with custom rules.
    pub fn with_rules(rules: Vec<AlertRule>) -> Self {
        Self {
            rules,
            windows: DashMap::new(),
            known_hosts: DashMap::new(),
            dedup_fingerprints: DashMap::new(),
        }
    }

    /// Add an entry to the sliding window with TTL and size cap.
    fn add_to_window(&self, user: &str, entry: WindowEntry) {
        // Determine the maximum age we need to keep
        let max_window = self.rules.iter()
            .map(|r| r.window_minutes())
            .max()
            .unwrap_or(60);
        let cutoff = entry.timestamp - max_window * 60;

        self.windows.entry(user.to_string()).and_modify(|deque| {
            // Prune expired entries
            while let Some(front) = deque.front() {
                if front.timestamp < cutoff {
                    deque.pop_front();
                } else {
                    break;
                }
            }
            // Enforce size cap
            while deque.len() >= MAX_WINDOW_SIZE {
                deque.pop_front();
            }
            deque.push_back(entry.clone());
        }).or_insert_with(|| {
            let mut d = VecDeque::new();
            d.push_back(entry.clone());
            d
        });
    }

    /// Process an audit entry and return any triggered alerts.
    pub fn handle_event(&self, entry: &AuditEntry) -> Vec<AlertEvent> {
        let ts = parse_timestamp(&entry.created_at);
        let mut alerts = Vec::new();

        // Add to sliding window (with TTL and size cap)
        self.add_to_window(&entry.user, WindowEntry {
            timestamp: ts,
            outcome: entry.outcome.clone(),
            resource: entry.resource.clone(),
        });

        for rule in &self.rules {
            if let Some(alert) = self.evaluate_rule(rule, entry, ts) {
                alerts.push(alert);
            }
        }

        // Apply deduplication: filter out alerts with identical message fingerprints
        let now = chrono::Utc::now().timestamp();
        alerts.retain(|alert| {
            let fp = AlertDeduplicator::fingerprint(&alert.message);
            let mut entry = self.dedup_fingerprints.entry(fp).or_insert((0, now));
            entry.0 += 1;
            // First occurrence or outside merge window
            if entry.0 == 1 || (now - entry.1) > 300 {
                entry.1 = now;
                entry.0 = 1;
                true
            } else {
                false // Duplicate
            }
        });

        // Publish events for triggered alerts via the global event bus
        for alert in &alerts {
            let _ = global_event_bus().publish(OpsEvent::AlertTriggered {
                severity: format!("{:?}", alert.severity).to_lowercase(),
                message: alert.message.clone(),
            });
        }

        alerts
    }

    fn evaluate_rule(&self, rule: &AlertRule, entry: &AuditEntry, now: i64) -> Option<AlertEvent> {
        let user_id = &entry.user;
        match rule {
            AlertRule::NightBatch {
                threshold,
                window_minutes,
            } => {
                // Check if current time is in night window (0-6 hours)
                let hour = (now % 86400) / 3600;
                if hour >= 6 {
                    return None;
                }

                let window = self.windows.get(user_id)?;
                let cutoff = now - window_minutes * 60;
                let distinct_hosts: HashSet<&str> = window
                    .iter()
                    .filter(|e| e.timestamp >= cutoff)
                    .filter(|e| e.resource.starts_with("host:"))
                    .map(|e| e.resource.as_str())
                    .collect();

                if distinct_hosts.len() >= *threshold {
                    Some(AlertEvent {
                        rule_name: rule.name().into(),
                        user: user_id.into(),
                        message: format!(
                            "Night batch: {} distinct hosts accessed in {} minutes (threshold: {})",
                            distinct_hosts.len(),
                            window_minutes,
                            threshold,
                        ),
                        severity: AlertSeverity::Warning,
                        created_at: now,
                    })
                } else {
                    None
                }
            }
            AlertRule::HighFailure {
                threshold,
                window_minutes,
            } => {
                let window = self.windows.get(user_id)?;
                let cutoff = now - window_minutes * 60;
                let failures: usize = window
                    .iter()
                    .filter(|e| e.timestamp >= cutoff)
                    .filter(|e| e.outcome != "success")
                    .count();

                if failures >= *threshold {
                    Some(AlertEvent {
                        rule_name: rule.name().into(),
                        user: user_id.into(),
                        message: format!(
                            "High failure rate: {} failed operations in {} minutes (threshold: {})",
                            failures, window_minutes, threshold,
                        ),
                        severity: AlertSeverity::Critical,
                        created_at: now,
                    })
                } else {
                    None
                }
            }
            AlertRule::FirstConnect => {
                if !entry.action.contains("connect") {
                    return None;
                }
                if !entry.resource.starts_with("host:") {
                    return None;
                }

                let mut hosts = self.known_hosts.entry(user_id.to_string()).or_default();
                if hosts.contains(&entry.resource) {
                    None
                } else {
                    hosts.insert(entry.resource.clone());
                    Some(AlertEvent {
                        rule_name: rule.name().into(),
                        user: user_id.into(),
                        message: format!("First connection to {}", entry.resource,),
                        severity: AlertSeverity::Info,
                        created_at: now,
                    })
                }
            }
        }
    }

    /// Prune old entries from sliding windows (call periodically).
    pub fn prune_windows(&self, max_age_minutes: i64) {
        let cutoff = chrono::Utc::now().timestamp() - max_age_minutes * 60;
        for mut entry in self.windows.iter_mut() {
            entry.retain(|e| e.timestamp > cutoff);
        }
    }

    /// Prune deduplication cache.
    pub fn prune_dedup(&self) {
        let now = chrono::Utc::now().timestamp();
        let cutoff = now - 600; // 10 minutes
        self.dedup_fingerprints.retain(|_, (_, ts)| *ts > cutoff);
    }
}

impl Default for AlertEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a datetime string to Unix timestamp. Falls back to current time.
fn parse_timestamp(s: &str) -> i64 {
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.and_utc().timestamp())
        .unwrap_or_else(|_| chrono::Utc::now().timestamp())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(user: &str, action: &str, resource: &str, outcome: &str) -> AuditEntry {
        AuditEntry {
            id: uuid::Uuid::new_v4().to_string(),
            user: user.into(),
            action: action.into(),
            resource: resource.into(),
            outcome: outcome.into(),
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }

    #[test]
    fn test_first_connect_triggers() {
        let engine = AlertEngine::new();
        let e = make_entry("user1", "ssh.connect", "host:server-1", "success");
        let alerts = engine.handle_event(&e);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].rule_name, "first_connect");
        assert_eq!(alerts[0].severity, AlertSeverity::Info);
    }

    #[test]
    fn test_first_connect_no_repeat() {
        let engine = AlertEngine::new();
        let e1 = make_entry("user1", "ssh.connect", "host:server-1", "success");
        engine.handle_event(&e1);
        let e2 = make_entry("user1", "ssh.connect", "host:server-1", "success");
        let alerts = engine.handle_event(&e2);
        // Second connect to same host should not trigger
        assert!(!alerts.iter().any(|a| a.rule_name == "first_connect"));
    }

    #[test]
    fn test_first_connect_different_host() {
        let engine = AlertEngine::new();
        let e1 = make_entry("user1", "ssh.connect", "host:server-1", "success");
        engine.handle_event(&e1);
        let e2 = make_entry("user1", "ssh.connect", "host:server-2", "success");
        let alerts = engine.handle_event(&e2);
        assert!(alerts.iter().any(|a| a.rule_name == "first_connect"));
    }

    #[test]
    fn test_high_failure_triggers() {
        let engine = AlertEngine::with_rules(vec![AlertRule::HighFailure {
            threshold: 3,
            window_minutes: 5,
        }]);

        for i in 0..3 {
            let e = make_entry("user1", &format!("action_{i}"), "host:server-1", "failure");
            let alerts = engine.handle_event(&e);
            if i < 2 {
                assert!(alerts.is_empty(), "Should not trigger at attempt {}", i + 1);
            } else {
                assert!(
                    alerts.iter().any(|a| a.rule_name == "high_failure"),
                    "Should trigger at attempt {}",
                    i + 1
                );
            }
        }
    }

    #[test]
    fn test_high_failure_successes_not_counted() {
        let engine = AlertEngine::with_rules(vec![AlertRule::HighFailure {
            threshold: 3,
            window_minutes: 5,
        }]);

        for _ in 0..5 {
            let e = make_entry("user1", "action", "host:server-1", "success");
            let alerts = engine.handle_event(&e);
            assert!(alerts.is_empty());
        }
    }

    #[test]
    fn test_custom_rules_empty() {
        let engine = AlertEngine::with_rules(vec![]);
        let e = make_entry("user1", "ssh.connect", "host:s1", "success");
        let alerts = engine.handle_event(&e);
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_different_users_independent() {
        let engine = AlertEngine::with_rules(vec![AlertRule::HighFailure {
            threshold: 2,
            window_minutes: 5,
        }]);

        let e1 = make_entry("user1", "a1", "r1", "failure");
        engine.handle_event(&e1);
        let e2 = make_entry("user2", "a2", "r2", "failure");
        let alerts = engine.handle_event(&e2);
        // user2 only has 1 failure, should not trigger
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_prune_windows() {
        let engine = AlertEngine::new();
        let e = make_entry("user1", "action", "host:s1", "success");
        engine.handle_event(&e);
        assert!(engine.windows.get("user1").unwrap().len() == 1);

        engine.prune_windows(0); // prune everything
        assert!(engine.windows.get("user1").unwrap().is_empty());
    }
}
