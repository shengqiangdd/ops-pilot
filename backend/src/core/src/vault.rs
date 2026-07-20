//! In-memory vault key cache with TTL auto-expiry.
//!
//! Keys are loaded by `AuthService::unlock_vault()` and auto-expire after
//! 15 minutes of inactivity. Explicit lock / logout removes immediately.

use dashmap::DashMap;
use std::time::{Duration, Instant};

const DEFAULT_TTL: Duration = Duration::from_secs(15 * 60); // 15 minutes

struct VaultEntry {
    key: [u8; 32],
    last_access: Instant,
}

/// Holds decrypted vault keys in memory keyed by user_id, with TTL expiry.
pub struct VaultKeyManager {
    entries: DashMap<String, VaultEntry>,
    ttl: Duration,
}

impl VaultKeyManager {
    pub fn new() -> Self {
        Self {
            entries: DashMap::new(),
            ttl: DEFAULT_TTL,
        }
    }

    /// Create with a custom TTL (for testing).
    #[cfg(test)]
    fn with_ttl(ttl: Duration) -> Self {
        Self {
            entries: DashMap::new(),
            ttl,
        }
    }

    /// Store a vault key for a user (called after successful unlock).
    pub fn set(&self, user_id: &str, key: [u8; 32]) {
        self.entries.insert(
            user_id.to_string(),
            VaultEntry {
                key,
                last_access: Instant::now(),
            },
        );
    }

    /// Get the vault key for a user. Returns None if not unlocked or expired.
    pub fn get(&self, user_id: &str) -> Option<[u8; 32]> {
        // First check if entry exists and is not expired
        let expired = self
            .entries
            .get(user_id)
            .map(|entry| entry.last_access.elapsed() > self.ttl)
            .unwrap_or(true);

        if expired {
            self.entries.remove(user_id);
            return None;
        }

        // Refresh last_access via entry_mut
        self.entries.get_mut(user_id).map(|mut entry| {
            entry.last_access = Instant::now();
            entry.key
        })
    }

    /// Remove vault key (on logout / token expiry / explicit lock).
    pub fn remove(&self, user_id: &str) {
        self.entries.remove(user_id);
    }

    /// Check if a user's vault is currently unlocked (and not expired).
    pub fn is_unlocked(&self, user_id: &str) -> bool {
        self.get(user_id).is_some()
    }
}

impl Default for VaultKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get() {
        let mgr = VaultKeyManager::new();
        let key = [1u8; 32];
        mgr.set("user-1", key);
        assert_eq!(mgr.get("user-1"), Some(key));
    }

    #[test]
    fn test_get_missing() {
        let mgr = VaultKeyManager::new();
        assert!(mgr.get("nonexistent").is_none());
    }

    #[test]
    fn test_remove() {
        let mgr = VaultKeyManager::new();
        mgr.set("user-1", [2u8; 32]);
        mgr.remove("user-1");
        assert!(mgr.get("user-1").is_none());
    }

    #[test]
    fn test_is_unlocked() {
        let mgr = VaultKeyManager::new();
        assert!(!mgr.is_unlocked("user-1"));
        mgr.set("user-1", [3u8; 32]);
        assert!(mgr.is_unlocked("user-1"));
        mgr.remove("user-1");
        assert!(!mgr.is_unlocked("user-1"));
    }

    #[test]
    fn test_overwrite_key() {
        let mgr = VaultKeyManager::new();
        mgr.set("user-1", [1u8; 32]);
        mgr.set("user-1", [2u8; 32]);
        assert_eq!(mgr.get("user-1"), Some([2u8; 32]));
    }

    #[test]
    fn test_multiple_users() {
        let mgr = VaultKeyManager::new();
        mgr.set("user-1", [1u8; 32]);
        mgr.set("user-2", [2u8; 32]);
        assert_eq!(mgr.get("user-1"), Some([1u8; 32]));
        assert_eq!(mgr.get("user-2"), Some([2u8; 32]));
        mgr.remove("user-1");
        assert!(mgr.get("user-1").is_none());
        assert_eq!(mgr.get("user-2"), Some([2u8; 32]));
    }

    #[test]
    fn test_ttl_expiry() {
        let mgr = VaultKeyManager::with_ttl(Duration::from_millis(50));
        mgr.set("user-1", [4u8; 32]);
        assert!(mgr.is_unlocked("user-1"));

        // Wait for TTL to expire
        std::thread::sleep(Duration::from_millis(60));
        assert!(!mgr.is_unlocked("user-1"));
        assert!(mgr.get("user-1").is_none());
    }
}
