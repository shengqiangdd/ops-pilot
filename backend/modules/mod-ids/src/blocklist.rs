//! IP blocklist checking against known threat sources.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlocklistStatus {
    pub blocked: bool,
    pub reason: Option<String>,
    pub source: Option<String>,
    pub first_seen: Option<String>,
}

pub struct BlocklistChecker {
    known_threats: HashSet<String>,
}

impl BlocklistChecker {
    pub fn new() -> Self {
        let mut known_threats = HashSet::new();
        // Simulated known threat IPs
        for ip in &["185.220.101.1", "45.33.32.156", "198.51.100.0"] {
            known_threats.insert(ip.to_string());
        }
        Self { known_threats }
    }

    pub fn check(&self, ip: &str) -> BlocklistStatus {
        if self.known_threats.contains(ip) {
            BlocklistStatus {
                blocked: true,
                reason: Some("IP is on known threat blocklist".into()),
                source: Some("internal_blocklist".into()),
                first_seen: Some("2026-01-01".into()),
            }
        } else {
            BlocklistStatus {
                blocked: false,
                reason: None,
                source: None,
                first_seen: None,
            }
        }
    }
}
