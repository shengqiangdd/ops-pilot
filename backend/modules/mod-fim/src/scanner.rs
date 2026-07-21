//! File integrity scanner — SSH-based file hashing and comparison.
//!
//! Uses SHA-256 checksums to verify file integrity. Computes hashes
//! via `sha256sum` on remote hosts.

use std::collections::HashMap;
use std::sync::Arc;

use ops_pilot_core::ssh::SshConnectionPool;
use tracing::warn;

pub struct FimScanner {
    executor: ops_pilot_core::ssh::CommandExecutor,
}

impl FimScanner {
    pub fn new(ssh_pool: Arc<SshConnectionPool>) -> Self {
        Self {
            executor: ops_pilot_core::ssh::CommandExecutor::new(ssh_pool),
        }
    }

    /// Compute SHA-256 hashes for a list of files on a remote host.
    pub async fn compute_hashes(
        &self,
        host_id: &str,
        paths: &[String],
    ) -> anyhow::Result<HashMap<String, String>> {
        if paths.is_empty() {
            return Ok(HashMap::new());
        }

        let file_list: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        let cmd = format!(
            "sha256sum {} 2>/dev/null",
            file_list.join(" ")
        );

        let result = match self.executor.exec_on_host(host_id, &cmd).await {
            Ok(r) => r,
            Err(e) => {
                warn!(host_id, error = %e, "SSH exec failed for FIM scan");
                return Ok(HashMap::new());
            }
        };

        let mut hashes = HashMap::new();
        for line in result.stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            // sha256sum output: "hash  filename"
            let parts: Vec<&str> = line.splitn(2, "  ").collect();
            if parts.len() == 2 {
                hashes.insert(parts[1].to_string(), parts[0].to_string());
            }
        }

        Ok(hashes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_creation() {
        let scanner = FimScanner::new(Arc::new(SshConnectionPool::new()));
        assert!(std::mem::size_of_val(&scanner) > 0);
    }
}
