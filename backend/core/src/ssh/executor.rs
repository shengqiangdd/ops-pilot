//! SSH command orchestration layer.
//!
//! Provides `CommandExecutor` for executing commands across multiple hosts
//! with parallel execution support and structured result capture.

use std::sync::Arc;
use std::time::Instant;

use russh::ChannelMsg;
use tokio::task::JoinSet;
use tracing::{debug, warn};

use super::{SshConnectionPool, SshError};

/// Result of a remote command execution.
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Standard output captured from the command.
    pub stdout: String,
    /// Standard error captured from the command.
    pub stderr: String,
    /// Process exit code, if the command finished.
    pub exit_code: Option<i32>,
    /// Wall-clock duration of the execution in milliseconds.
    pub duration_ms: u64,
}

impl CommandResult {
    /// Returns true if the command exited with code 0.
    pub fn success(&self) -> bool {
        self.exit_code == Some(0)
    }
}

/// Orchestrates SSH command execution across one or many hosts.
pub struct CommandExecutor {
    pool: Arc<SshConnectionPool>,
}

impl CommandExecutor {
    /// Create a new executor wrapping the given connection pool.
    pub fn new(pool: Arc<SshConnectionPool>) -> Self {
        Self { pool }
    }

    /// Execute a command on a single host identified by `host_id`.
    ///
    /// Looks up the connection in the pool, opens a channel, runs the command,
    /// and captures stdout, stderr, and exit code into a `CommandResult`.
    pub async fn exec_on_host(
        &self,
        host_id: &str,
        command: &str,
    ) -> Result<CommandResult, SshError> {
        let conn = self.pool.get(host_id).await?;
        let start = Instant::now();

        let handle = conn.handle.read().await;
        let channel = handle
            .channel_open_session()
            .await
            .map_err(|e| SshError::Channel(format!("failed to open channel: {}", e)))?;

        channel
            .exec(true, command.as_bytes())
            .await
            .map_err(|e| SshError::Channel(format!("exec failed: {}", e)))?;

        let mut stdout = String::new();
        let mut stderr = String::new();
        let mut exit_code: Option<i32> = None;
        let mut channel = channel;

        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { data } => {
                    stdout.push_str(&String::from_utf8_lossy(&data));
                }
                ChannelMsg::ExtendedData { data, .. } => {
                    stderr.push_str(&String::from_utf8_lossy(&data));
                }
                ChannelMsg::ExitStatus { exit_status } => {
                    exit_code = Some(exit_status as i32);
                }
                ChannelMsg::Eof => break,
                _ => continue,
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        debug!(
            host_id,
            command_len = command.len(),
            stdout_len = stdout.len(),
            stderr_len = stderr.len(),
            ?exit_code,
            duration_ms,
            "command completed"
        );

        Ok(CommandResult {
            stdout,
            stderr,
            exit_code,
            duration_ms,
        })
    }

    /// Execute a command on multiple hosts in parallel.
    ///
    /// Returns a vec of `(host_id, result)` pairs. Hosts that fail to execute
    /// the command will have their error logged and be omitted from the result
    /// vec — callers should compare the returned length against the input to
    /// detect partial failures.
    pub async fn exec_on_all(
        &self,
        hosts: &[String],
        command: &str,
    ) -> Vec<(String, Result<CommandResult, SshError>)> {
        let pool = self.pool.clone();
        let command = command.to_string();

        let mut join_set = JoinSet::new();

        for host_id in hosts {
            let pool = pool.clone();
            let host_id = host_id.clone();
            let command = command.clone();

            join_set.spawn(async move {
                let result = Self::exec_on_host_inner(pool, &host_id, &command).await;
                (host_id, result)
            });
        }

        let mut results = Vec::with_capacity(hosts.len());
        while let Some(join_result) = join_set.join_next().await {
            match join_result {
                Ok((host_id, result)) => {
                    results.push((host_id, result));
                }
                Err(e) => {
                    warn!(error = %e, "task join error during parallel exec");
                }
            }
        }

        results
    }

    /// Internal helper that calls `exec_on_host` via a temporary executor.
    /// Avoids borrowing `self` across spawned tasks in `exec_on_all`.
    async fn exec_on_host_inner(
        pool: Arc<SshConnectionPool>,
        host_id: &str,
        command: &str,
    ) -> Result<CommandResult, SshError> {
        let executor = CommandExecutor { pool };
        executor.exec_on_host(host_id, command).await
    }

    /// Get a reference to the underlying connection pool.
    pub fn pool(&self) -> &SshConnectionPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_result_success() {
        let result = CommandResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: 100,
        };
        assert!(result.success());
    }

    #[test]
    fn test_command_result_failure() {
        let result = CommandResult {
            stdout: String::new(),
            stderr: "error".to_string(),
            exit_code: Some(1),
            duration_ms: 50,
        };
        assert!(!result.success());
    }

    #[test]
    fn test_command_result_no_exit_code() {
        let result = CommandResult {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            duration_ms: 0,
        };
        assert!(!result.success());
    }

    #[test]
    fn test_command_result_clone() {
        let result = CommandResult {
            stdout: "out".to_string(),
            stderr: "err".to_string(),
            exit_code: Some(0),
            duration_ms: 42,
        };
        let cloned = result.clone();
        assert_eq!(result.stdout, cloned.stdout);
        assert_eq!(result.stderr, cloned.stderr);
        assert_eq!(result.exit_code, cloned.exit_code);
        assert_eq!(result.duration_ms, cloned.duration_ms);
    }

    #[test]
    fn test_executor_creation() {
        let pool = Arc::new(SshConnectionPool::new());
        let executor = CommandExecutor::new(pool.clone());
        assert_eq!(executor.pool().connection_count(), 0);
    }

    #[tokio::test]
    async fn test_exec_on_host_no_connection() {
        let pool = Arc::new(SshConnectionPool::new());
        let executor = CommandExecutor::new(pool);
        let result = executor.exec_on_host("missing", "whoami").await;
        assert!(matches!(result, Err(SshError::ConnectionClosed)));
    }

    #[tokio::test]
    async fn test_exec_on_all_empty_hosts() {
        let pool = Arc::new(SshConnectionPool::new());
        let executor = CommandExecutor::new(pool);
        let results = executor.exec_on_all(&[], "uptime").await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_exec_on_all_missing_hosts() {
        let pool = Arc::new(SshConnectionPool::new());
        let executor = CommandExecutor::new(pool);
        let hosts = vec!["host1".to_string(), "host2".to_string()];
        let results = executor.exec_on_all(&hosts, "uptime").await;
        assert_eq!(results.len(), 2);
        for (_host_id, result) in &results {
            assert!(matches!(result, Err(SshError::ConnectionClosed)));
        }
    }
}
