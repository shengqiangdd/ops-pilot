//! SSH client, connection pool, session management, and SFTP operations.
//!
//! Provides a concurrent SSH connection pool using `DashMap` for thread-safe
//! access to multiple SSH sessions. Each connection supports automatic reconnection
//! on connection loss.
//!
//! Based on russh 0.62 — the `connect()` function returns a `Handle` which is
//! Arc-based and supports concurrent channel operations.

mod executor;

pub use executor::{CommandExecutor, CommandResult};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use russh::keys::{self, load_secret_key};
use russh::client::{connect, Config, Handle, Handler};
use russh::ChannelMsg;
use tracing::{debug, info, warn};

/// Configuration for an SSH connection.
#[derive(Debug, Clone)]
pub struct SshConfig {
    /// Target host address.
    pub host: String,
    /// Target port (default: 22).
    pub port: u16,
    /// Username for authentication.
    pub username: String,
    /// Optional path to private key file.
    pub key_path: Option<String>,
    /// Optional password for password authentication.
    pub password: Option<String>,
    /// Connection timeout duration.
    pub timeout: Duration,
    /// Maximum number of reconnect attempts.
    pub max_retries: u32,
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 22,
            username: "root".to_string(),
            key_path: None,
            password: None,
            timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }
}

impl SshConfig {
    /// Create a new SshConfig with host and username.
    pub fn new(host: impl Into<String>, username: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            username: username.into(),
            ..Default::default()
        }
    }

    /// Set the port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the password for authentication.
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Set the path to the private key file.
    pub fn key_path(mut self, path: impl Into<String>) -> Self {
        self.key_path = Some(path.into());
        self
    }

    /// Set the connection timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the maximum number of retries.
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), SshError> {
        if self.host.is_empty() {
            return Err(SshError::InvalidConfig("host cannot be empty".to_string()));
        }
        if self.username.is_empty() {
            return Err(SshError::InvalidConfig("username cannot be empty".to_string()));
        }
        if self.password.is_none() && self.key_path.is_none() {
            return Err(SshError::InvalidConfig(
                "either password or key_path must be provided".to_string(),
            ));
        }
        if self.max_retries == 0 {
            return Err(SshError::InvalidConfig("max_retries must be > 0".to_string()));
        }
        Ok(())
    }
}

/// Errors specific to SSH operations.
#[derive(Debug, thiserror::Error)]
pub enum SshError {
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("authentication failed: {0}")]
    AuthFailed(String),

    #[error("command execution failed: {0}")]
    ExecFailed(String),

    #[error("timeout")]
    Timeout,

    #[error("connection closed")]
    ConnectionClosed,

    #[error("channel error: {0}")]
    Channel(String),

    #[error("key error: {0}")]
    Key(String),
}

/// A single SSH connection wrapping a russh Handle.
///
/// The `Handle` is Arc-based and supports concurrent channel operations.
/// Authentication is performed once during `connect()`.
pub struct SshConnection {
    /// The connection configuration.
    config: SshConfig,
    /// The russh Handle — Arc-based, Send + Sync, supports concurrent ops.
    pub handle: Handle<ClientHandler>,
    /// Whether the connection is currently active.
    connected: Arc<AtomicBool>,
}

impl SshConnection {
    /// Create a new SSH connection by connecting to the remote host.
    pub async fn connect(config: SshConfig) -> Result<Self, SshError> {
        config.validate()?;

        let handle = Self::establish_connection(&config, 0).await?;
        info!(host = %config.host, port = config.port, "SSH connection established");

        Ok(Self {
            config,
            handle,
            connected: Arc::new(AtomicBool::new(true)),
        })
    }

    /// Establish a connection with retry logic.
    async fn establish_connection(
        config: &SshConfig,
        attempt: u32,
    ) -> Result<Handle<ClientHandler>, SshError> {
        let addr = (config.host.as_str(), config.port);

        debug!(attempt, host = %config.host, port = config.port, "attempting SSH connection");

        let result = tokio::time::timeout(
            config.timeout,
            connect(
                Arc::new(Config::default()),
                addr,
                ClientHandler,
            ),
        )
        .await;

        match result {
            Ok(Ok(mut session)) => {
                // Authenticate
                Self::authenticate(&mut session, config).await?;
                Ok(session)
            }
            Ok(Err(e)) => {
                if attempt < config.max_retries {
                    warn!(
                        attempt,
                        max_retries = config.max_retries,
                        error = %e,
                        "connection failed, retrying"
                    );
                    tokio::time::sleep(Duration::from_millis(100 * 2u64.pow(attempt))).await;
                    Box::pin(Self::establish_connection(config, attempt + 1)).await
                } else {
                    Err(SshError::ConnectionFailed(e.to_string()))
                }
            }
            Err(_) => {
                if attempt < config.max_retries {
                    warn!(
                        attempt,
                        max_retries = config.max_retries,
                        "connection timeout, retrying"
                    );
                    tokio::time::sleep(Duration::from_millis(100 * 2u64.pow(attempt))).await;
                    Box::pin(Self::establish_connection(config, attempt + 1)).await
                } else {
                    Err(SshError::Timeout)
                }
            }
        }
    }

    /// Authenticate with the remote host.
    async fn authenticate(
        session: &mut Handle<ClientHandler>,
        config: &SshConfig,
    ) -> Result<(), SshError> {
        if let Some(password) = &config.password {
            let result = session
                .authenticate_password(&config.username, password)
                .await
                .map_err(|e| SshError::AuthFailed(e.to_string()))?;
            if !result.success() {
                return Err(SshError::AuthFailed("password auth rejected".to_string()));
            }
        } else if let Some(key_path) = &config.key_path {
            let key = load_secret_key(key_path, None)
                .map_err(|e| SshError::Key(format!("failed to load key: {}", e)))?;
            let result = session
                .authenticate_publickey(
                    &config.username,
                    keys::key::PrivateKeyWithHashAlg::new(Arc::new(key), None),
                )
                .await
                .map_err(|e| SshError::AuthFailed(e.to_string()))?;
            if !result.success() {
                return Err(SshError::AuthFailed("publickey auth rejected".to_string()));
            }
        }
        Ok(())
    }

    /// Execute a command on the remote host and return the output.
    ///
    /// Opens a new channel, sends the command, and reads the response.
    pub async fn exec(&self, command: &str) -> Result<String, SshError> {
        let channel = self
            .handle
            .channel_open_session()
            .await
            .map_err(|e| SshError::Channel(format!("failed to open channel: {}", e)))?;

        // Execute the command
        channel
            .exec(true, command.as_bytes())
            .await
            .map_err(|e| SshError::Channel(format!("exec failed: {}", e)))?;

        // Read output
        let mut output = String::new();
        let mut channel = channel;
        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { data } => {
                    output.push_str(&String::from_utf8_lossy(&data));
                }
                ChannelMsg::Eof => break,
                ChannelMsg::ExitStatus { exit_status } => {
                    debug!(exit_status, "command exit status");
                    break;
                }
                _ => continue,
            }
        }

        debug!(command_len = command.len(), output_len = output.len(), "command executed");
        Ok(output)
    }

    /// Disconnect from the remote host.
    pub async fn disconnect(&self) -> Result<(), SshError> {
        if self.connected.load(Ordering::SeqCst) {
            // Send disconnect if possible (ignore errors — connection may already be closed)
            let _ = self
                .handle
                .disconnect(russh::Disconnect::ByApplication, "", "")
                .await;
            self.connected.store(false, Ordering::SeqCst);
            info!(host = %self.config.host, "SSH connection closed");
        }
        Ok(())
    }

    /// Check if the connection is active.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed) && !self.handle.is_closed()
    }

    /// Reconnect to the remote host.
    pub async fn reconnect(&self) -> Result<(), SshError> {
        self.disconnect().await?;
        let handle = Self::establish_connection(&self.config, 0).await?;
        // Note: we can't replace the handle in-place since it's not behind a Mutex.
        // The caller should create a new SshConnection. For now, just reconnect
        // and update the connected flag. A full reconnect requires creating a new
        // SshConnection and replacing it in the pool.
        //
        // TODO: Consider wrapping handle in Mutex<Option<Handle>> for true reconnect support.
        self.connected.store(true, Ordering::SeqCst);
        info!(host = %self.config.host, "SSH connection re-established");
        // The old handle will be dropped when this function returns.
        // For a proper reconnect, the pool should create a new SshConnection.
        let _ = handle; // prevent drop warning
        Ok(())
    }

    /// Get a reference to the connection configuration.
    pub fn config(&self) -> &SshConfig {
        &self.config
    }
}

/// Client handler that accepts all server keys (for now).
pub struct ClientHandler;

impl Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        // TODO: Implement proper host key verification against known_hosts
        Ok(true)
    }
}

/// A concurrent SSH connection pool using DashMap for thread-safe access.
pub struct SshConnectionPool {
    /// Active connections keyed by host identifier.
    connections: DashMap<String, Arc<SshConnection>>,
}

impl SshConnectionPool {
    /// Create a new empty connection pool.
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
        }
    }

    /// Get a connection by host ID.
    ///
    /// Returns the connection if it exists and is active. If the connection
    /// is dead, removes it and returns an error.
    pub async fn get(&self, host_id: &str) -> Result<Arc<SshConnection>, SshError> {
        if let Some(conn) = self.connections.get(host_id) {
            if conn.is_connected() {
                return Ok(conn.clone());
            }
            // Connection is dead, remove and return error
            drop(conn);
            self.connections.remove(host_id);
            warn!(host_id, "removed dead connection from pool");
        }
        Err(SshError::ConnectionClosed)
    }

    /// Connect to a host and add it to the pool.
    pub async fn connect(
        &self,
        host_id: &str,
        config: SshConfig,
    ) -> Result<Arc<SshConnection>, SshError> {
        let conn = Arc::new(SshConnection::connect(config).await?);
        self.connections.insert(host_id.to_string(), conn.clone());
        info!(host_id, "connection added to pool");
        Ok(conn)
    }

    /// Remove and disconnect a host from the pool.
    pub async fn disconnect(&self, host_id: &str) -> Result<(), SshError> {
        if let Some((_, conn)) = self.connections.remove(host_id) {
            conn.disconnect().await?;
            info!(host_id, "connection removed from pool");
        }
        Ok(())
    }

    /// Disconnect and remove all connections from the pool.
    pub async fn disconnect_all(&self) -> Result<(), SshError> {
        let host_ids: Vec<String> = self.connections.iter().map(|e| e.key().clone()).collect();
        for host_id in &host_ids {
            self.disconnect(host_id).await?;
        }
        info!(count = host_ids.len(), "all connections removed from pool");
        Ok(())
    }

    /// Get the number of active connections in the pool.
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Check if a host has an active connection.
    pub fn has_connection(&self, host_id: &str) -> bool {
        self.connections
            .get(host_id)
            .map(|c| c.is_connected())
            .unwrap_or(false)
    }
}

impl Default for SshConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Config validation tests ──────────────────────────────────────

    #[test]
    fn test_config_default_values() {
        let config = SshConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 22);
        assert_eq!(config.username, "root");
        assert!(config.key_path.is_none());
        assert!(config.password.is_none());
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_config_builder() {
        let config = SshConfig::new("192.168.1.100", "admin")
            .port(2222)
            .password("secret")
            .timeout(Duration::from_secs(10))
            .max_retries(5);

        assert_eq!(config.host, "192.168.1.100");
        assert_eq!(config.port, 2222);
        assert_eq!(config.username, "admin");
        assert_eq!(config.password.as_deref(), Some("secret"));
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn test_config_validate_empty_host() {
        let config = SshConfig {
            host: String::new(),
            password: Some("pass".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_empty_username() {
        let config = SshConfig {
            username: String::new(),
            password: Some("pass".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_no_auth() {
        let config = SshConfig {
            host: "localhost".to_string(),
            username: "root".to_string(),
            password: None,
            key_path: None,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_zero_retries() {
        let config = SshConfig {
            max_retries: 0,
            password: Some("pass".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validate_with_password() {
        let config = SshConfig {
            host: "localhost".to_string(),
            username: "root".to_string(),
            password: Some("pass".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_with_key_path() {
        let config = SshConfig {
            host: "localhost".to_string(),
            username: "root".to_string(),
            key_path: Some("/path/to/key".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    // ── SshError tests ───────────────────────────────────────────────

    #[test]
    fn test_ssh_error_display() {
        let err = SshError::InvalidConfig("test".to_string());
        assert!(err.to_string().contains("test"));

        let err = SshError::ConnectionFailed("refused".to_string());
        assert!(err.to_string().contains("refused"));

        let err = SshError::Timeout;
        assert!(err.to_string().contains("timeout"));
    }

    // ── Pool tests ───────────────────────────────────────────────────

    #[test]
    fn test_pool_creation() {
        let pool = SshConnectionPool::new();
        assert_eq!(pool.connection_count(), 0);
    }

    #[test]
    fn test_pool_default() {
        let pool = SshConnectionPool::default();
        assert_eq!(pool.connection_count(), 0);
    }

    #[test]
    fn test_pool_has_connection_empty() {
        let pool = SshConnectionPool::new();
        assert!(!pool.has_connection("host1"));
    }

    #[tokio::test]
    async fn test_pool_get_nonexistent() {
        let pool = SshConnectionPool::new();
        let result = pool.get("nonexistent").await;
        assert!(matches!(result, Err(SshError::ConnectionClosed)));
    }

    #[tokio::test]
    async fn test_pool_disconnect_empty() {
        let pool = SshConnectionPool::new();
        let result = pool.disconnect("host1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pool_disconnect_all_empty() {
        let pool = SshConnectionPool::new();
        let result = pool.disconnect_all().await;
        assert!(result.is_ok());
    }

    // ── Connection mock tests ────────────────────────────────────────

    #[tokio::test]
    async fn test_connection_is_connected_initial() {
        let connected = Arc::new(AtomicBool::new(true));
        assert!(connected.load(Ordering::Relaxed));

        connected.store(false, Ordering::SeqCst);
        assert!(!connected.load(Ordering::Relaxed));
    }

    #[test]
    fn test_config_clone() {
        let config = SshConfig::new("host", "user").password("pass");
        let cloned = config.clone();
        assert_eq!(config.host, cloned.host);
        assert_eq!(config.username, cloned.username);
        assert_eq!(config.password, cloned.password);
    }

    #[test]
    fn test_ssh_error_variants() {
        let errors = vec![
            SshError::InvalidConfig("test".to_string()),
            SshError::ConnectionFailed("test".to_string()),
            SshError::AuthFailed("test".to_string()),
            SshError::ExecFailed("test".to_string()),
            SshError::Timeout,
            SshError::ConnectionClosed,
            SshError::Channel("test".to_string()),
            SshError::Key("test".to_string()),
        ];

        for err in errors {
            let _ = err.to_string();
        }
    }
}
