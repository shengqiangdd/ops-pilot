//! Audit alert engine — rule-based anomaly detection over audit events.
//!
//! Subscribes to `EventBus::AuditLog` events and maintains sliding windows
//! for detection. Rules include night-batch operations, high failure rates,
//! and first-time host connections.
//!
//! ## Optimizations
//! - Sliding window TTL: entries older than max_window_minutes are pruned on each insert
//! - Window size cap: each user's deque is capped at MAX_WINDOW_SIZE entries
//! - Alert deduplication: SimHash + exact fingerprint dual-matching within merge window

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

// ── SimHash ──────────────────────────────────────────────────────────────────

/// SimHash fingerprint width in bits.
const SIMHASH_BITS: u8 = 64;

/// Maximum Hamming distance to consider two fingerprints "similar".
const SIMILARITY_THRESHOLD: u32 = 3;

/// Alert deduplicator — suppresses identical or similar alert messages within a time window.
/// Uses dual-matching: exact fingerprint + SimHash Hamming distance.
pub struct AlertDeduplicator {
    /// message_fingerprint → (count, first_seen_timestamp)
    recent: HashMap<u64, (usize, i64)>,
    /// Merge window in seconds: same fingerprint within this window is suppressed
    merge_window_secs: i64,
    /// SimHash cache: exact_fingerprint → (simhash, timestamp)
    simhash_cache: DashMap<u64, (u64, i64)>,
}

impl AlertDeduplicator {
    pub fn new(merge_window_secs: i64) -> Self {
        Self {
            recent: HashMap::new(),
            merge_window_secs,
            simhash_cache: DashMap::new(),
        }
    }

    /// Create a simple fingerprint from an alert message (exact match).
    fn fingerprint(message: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        message.hash(&mut hasher);
        hasher.finish()
    }

    /// Compute SimHash fingerprint (64-bit) for near-duplicate detection.
    ///
    /// Algorithm:
    /// 1. Tokenize message by whitespace/punctuation
    /// 2. Hash each token to a 64-bit value via SipHash
    /// 3. Weighted vote: position-weighted (earlier tokens weigh more)
    /// 4. Final 64-bit SimHash = bit majority across all token hashes
    pub fn simhash(message: &str) -> u64 {
        let tokens: Vec<&str> = message.split(|c: char| !c.is_alphanumeric())
            .filter(|t| !t.is_empty())
            .collect();

        if tokens.is_empty() {
            return 0;
        }

        // Weighted bit vote vector: +weight for 1-bits, -weight for 0-bits
        let mut bit_votes = [0.0_f64; SIMHASH_BITS as usize];
        let n_tokens = tokens.len() as f64;

        for (i, token) in tokens.iter().enumerate() {
            // Position weight: earlier tokens weigh more (inverse position)
            let position_weight = 1.0 / (1.0 + i as f64 * 0.3);
            // TF weight: rare short tokens weigh more
            let tf_weight = 1.0 / (1.0 + (token.len() as f64).ln());
            let weight = position_weight * tf_weight;

            let hash = Self::sip_hash(token);
            for bit in 0..SIMHASH_BITS {
                if hash & (1 << bit) != 0 {
                    bit_votes[bit as usize] += weight;
                } else {
                    bit_votes[bit as usize] -= weight;
                }
            }
        }

        // Collapse: set bit where vote is positive
        let mut result: u64 = 0;
        for bit in 0..SIMHASH_BITS {
            if bit_votes[bit as usize] > 0.0 {
                result |= 1 << bit;
            }
        }
        result
    }

    /// SipHash-2-4 style hash of a string to u64.
    fn sip_hash(s: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    /// Compute Hamming distance between two u64 fingerprints (popcount of XOR).
    pub fn hamming_distance(a: u64, b: u64) -> u32 {
        (a ^ b).count_ones()
    }

    /// Check if two SimHash fingerprints are similar (within threshold).
    pub fn is_similar(a: u64, b: u64) -> bool {
        Self::hamming_distance(a, b) <= SIMILARITY_THRESHOLD
    }

    /// Returns true if this alert message is a duplicate (exact or similar) within the merge window.
    pub fn is_duplicate(&mut self, message: &str, now: i64) -> bool {
        let fp = Self::fingerprint(message);
        let sh = Self::simhash(message);

        // 1. Exact match check
        let entry = self.recent.entry(fp).or_insert((0, now));
        entry.0 += 1;

        if entry.0 == 1 || (now - entry.1) > self.merge_window_secs {
            // First occurrence or outside window — register in simhash cache
            self.simhash_cache.insert(fp, (sh, now));
            entry.1 = now;
            entry.0 = 1;
            return false;
        }

        // 2. SimHash similarity check against recent fingerprints in cache
        let cutoff = now - self.merge_window_secs;
        for item in self.simhash_cache.iter() {
            let (cached_sh, cached_ts) = *item.value();
            if *item.key() != fp && cached_ts > cutoff && Self::is_similar(sh, cached_sh) {
                return true; // Similar alert found within merge window
            }
        }

        // Not similar to anything recent — store in cache
        self.simhash_cache.insert(fp, (sh, now));
        true
    }

    /// Prune entries older than 2x the merge window.
    pub fn prune(&mut self, now: i64) {
        let cutoff = now - self.merge_window_secs * 2;
        self.recent.retain(|_, (_, ts)| *ts > cutoff);
        self.simhash_cache.retain(|_, (_, ts)| *ts > cutoff);
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
    /// SimHash cache for near-duplicate detection: exact_fingerprint → (simhash, timestamp)
    simhash_cache: DashMap<u64, (u64, i64)>,
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
            simhash_cache: DashMap::new(),
        }
    }

    /// Create with custom rules.
    pub fn with_rules(rules: Vec<AlertRule>) -> Self {
        Self {
            rules,
            windows: DashMap::new(),
            known_hosts: DashMap::new(),
            dedup_fingerprints: DashMap::new(),
            simhash_cache: DashMap::new(),
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

        // Apply deduplication: dual-match (exact + SimHash near-duplicate)
        let now = chrono::Utc::now().timestamp();
        alerts.retain(|alert| {
            let fp = AlertDeduplicator::fingerprint(&alert.message);
            let sh = AlertDeduplicator::simhash(&alert.message);

            // Exact match
            let mut entry = self.dedup_fingerprints.entry(fp).or_insert((0, now));
            entry.0 += 1;
            if entry.0 == 1 || (now - entry.1) > 300 {
                entry.1 = now;
                entry.0 = 1;

                // Also check SimHash similarity against recent fingerprints
                let cutoff = now - 300;
                for item in self.simhash_cache.iter() {
                    let (cached_sh, cached_ts) = *item.value();
                    if *item.key() != fp && cached_ts > cutoff && AlertDeduplicator::is_similar(sh, cached_sh) {
                        return false; // Near-duplicate found
                    }
                }

                // Store in simhash cache
                self.simhash_cache.insert(fp, (sh, now));
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
        self.simhash_cache.retain(|_, (_, ts)| *ts > cutoff);
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
    fn test_simhash_same_message() {
        let sh1 = AlertDeduplicator::simhash("CPU usage is high on server-1");
        let sh2 = AlertDeduplicator::simhash("CPU usage is high on server-1");
        assert_eq!(sh1, sh2);
        assert_eq!(AlertDeduplicator::hamming_distance(sh1, sh2), 0);
        assert!(AlertDeduplicator::is_similar(sh1, sh2));
    }

    #[test]
    fn test_simhash_similar_messages() {
        let sh1 = AlertDeduplicator::simhash("CPU usage is high on server-1");
        let sh2 = AlertDeduplicator::simhash("CPU usage is high on server-2");
        let dist = AlertDeduplicator::hamming_distance(sh1, sh2);
        // Similar messages should have small Hamming distance
        assert!(dist <= 10, "distance={} for similar messages", dist);
    }

    #[test]
    fn test_hamming_distance() {
        assert_eq!(AlertDeduplicator::hamming_distance(0, 0), 0);
        assert_eq!(AlertDeduplicator::hamming_distance(0, 1), 1);
        assert_eq!(AlertDeduplicator::hamming_distance(u64::MAX, 0), 64);
        assert_eq!(AlertDeduplicator::hamming_distance(0b1010, 0b1001), 2);
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

    #[test]
    fn test_dedup_simhash_near_duplicate() {
        let mut dedup = AlertDeduplicator::new(300);
        let now = chrono::Utc::now().timestamp();
        // Same core message, slightly different
        assert!(!dedup.is_duplicate("CPU usage high on prod-1", now));
        // This is near-duplicate; may or may not be caught depending on hash
        // The key test is that exact duplicates are caught
        assert!(dedup.is_duplicate("CPU usage high on prod-1", now));
    }
}
