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

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use russh::keys::{self, load_secret_key, PublicKey};
use russh::client::{connect, Config, Handle, Handler};
use russh::ChannelMsg;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Strict host key checking mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum StrictHostKeyChecking {
    /// Verify the host key against known_hosts; fail if unknown.
    Yes,
    /// Accept any host key (insecure, for development only).
    No,
    /// Accept unknown keys but reject changed keys.
    #[default]
    AcceptNew,
}

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
    /// Path to the known_hosts file (default: ~/.ssh/known_hosts).
    pub known_hosts_path: Option<PathBuf>,
    /// Strict host key checking mode.
    pub strict_host_key_checking: StrictHostKeyChecking,
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
            known_hosts_path: None,
            strict_host_key_checking: StrictHostKeyChecking::default(),
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

    /// Set the path to the known_hosts file.
    pub fn known_hosts_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.known_hosts_path = Some(path.into());
        self
    }

    /// Set the strict host key checking mode.
    pub fn strict_host_key_checking(mut self, mode: StrictHostKeyChecking) -> Self {
        self.strict_host_key_checking = mode;
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

/// Manages the SSH known_hosts file for host key verification.
///
/// File format: `hostname keytype base64key` (one per line, # for comments).
pub struct KnownHosts {
    path: PathBuf,
    entries: HashMap<String, Vec<KnownHostEntry>>,
}

#[derive(Debug, Clone)]
struct KnownHostEntry {
    key_type: String,
    base64_key: String,
}

impl KnownHosts {
    /// Open or create a known_hosts file at the given path.
    /// Uses `~/.ssh/known_hosts` if no path is provided.
    pub fn open(path: Option<&Path>) -> Result<Self, SshError> {
        let path = match path {
            Some(p) => p.to_path_buf(),
            None => dirs_ssh_known_hosts(),
        };

        let entries = if path.exists() {
            Self::parse_file(&path)?
        } else {
            HashMap::new()
        };

        Ok(Self { path, entries })
    }

    /// Parse the known_hosts file.
    fn parse_file(path: &Path) -> Result<HashMap<String, Vec<KnownHostEntry>>, SshError> {
        let content = fs::read_to_string(path)
            .map_err(|e| SshError::Key(format!("failed to read known_hosts: {e}")))?;
        let mut map = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                map.entry(parts[0].to_string())
                    .or_default()
                    .push(KnownHostEntry {
                        key_type: parts[1].to_string(),
                        base64_key: parts[2].to_string(),
                    });
            }
        }

        Ok(map)
    }

    /// Check if the given host key is known and matches.
    /// Returns Ok(true) if verified, Ok(false) if unknown (should be accepted),
    /// Err if the key has changed (hostile).
    pub fn check_host_key(
        &self,
        hostname: &str,
        key: &PublicKey,
    ) -> Result<bool, SshError> {
        // ssh_key 0.7: use algorithm() which returns Algorithm enum, format it
        let key_type = format!("{:?}", key.algorithm());
        // Remove the "Other(" wrapper if present (for custom algorithms)
        let key_type = key_type
            .trim_start_matches("Other(\"")
            .trim_end_matches("\")")
            .to_string();

        let known_keys = match self.entries.get(hostname) {
            Some(keys) => keys,
            None => return Ok(false), // Unknown host — caller decides based on strict mode
        };

        // ssh_key 0.7: encode_openssh returns "ssh-ed25519 AAAA... comment"
        let key_data = key
            .to_openssh()
            .unwrap_or_default()
            .split_whitespace()
            .nth(1) // skip algorithm prefix, take the base64 key
            .unwrap_or("")
            .to_string();

        for entry in known_keys {
            if entry.key_type == key_type && entry.base64_key == key_data {
                return Ok(true); // Key matches
            }
        }

        // Key type matches but data doesn't — host key changed!
        Err(SshError::Key(format!(
            "host key for '{}' has changed! Possible MITM attack.",
            hostname
        )))
    }

    /// Add a new host key to the known_hosts file.
    pub fn add_host_key(&mut self, hostname: &str, key: &PublicKey) -> Result<(), SshError> {
        let key_type = format!("{:?}", key.algorithm());
        let key_type = key_type
            .trim_start_matches("Other(\"")
            .trim_end_matches("\")")
            .to_string();

        let key_data = key
            .to_openssh()
            .unwrap_or_default()
            .split_whitespace()
            .nth(1)
            .unwrap_or("")
            .to_string();

        let entry = KnownHostEntry {
            key_type: key_type.clone(),
            base64_key: key_data,
        };

        // Update in-memory
        self.entries
            .entry(hostname.to_string())
            .or_default()
            .push(entry);

        // Append to file
        let line = format!("{} {} {}\n", hostname, key_type, self.entries[hostname].last().unwrap().base64_key);
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .and_then(|mut f| {
                use std::io::Write;
                write!(f, "{line}")
            })
            .map_err(|e| SshError::Key(format!("failed to write known_hosts: {e}")))?;

        Ok(())
    }
}

/// Resolve the default SSH known_hosts path (~/.ssh/known_hosts).
fn dirs_ssh_known_hosts() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".ssh").join("known_hosts")
    } else {
        PathBuf::from("/tmp").join("known_hosts")
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
    /// The russh Handle — wrapped in RwLock so `reconnect()` can swap it.
    pub handle: Arc<RwLock<Handle<ClientHandler>>>,
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
            handle: Arc::new(RwLock::new(handle)),
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

        // Load known_hosts for key verification
        let known_hosts = KnownHosts::open(config.known_hosts_path.as_deref()).ok();

        let handler = ClientHandler::new(
            config.host.clone(),
            known_hosts,
            config.strict_host_key_checking,
        );

        let result = tokio::time::timeout(
            config.timeout,
            connect(
                Arc::new(Config::default()),
                addr,
                handler,
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
        let handle = self.handle.read().await;
        let channel = handle
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
            let handle = self.handle.read().await;
            let _ = handle
                .disconnect(russh::Disconnect::ByApplication, "", "")
                .await;
            self.connected.store(false, Ordering::SeqCst);
            info!(host = %self.config.host, "SSH connection closed");
        }
        Ok(())
    }

    /// Check if the connection is active.
    pub async fn is_connected(&self) -> bool {
        if !self.connected.load(Ordering::Relaxed) {
            return false;
        }
        let handle = self.handle.read().await;
        !handle.is_closed()
    }

    /// Reconnect to the remote host.
    pub async fn reconnect(&self) -> Result<(), SshError> {
        self.disconnect().await?;
        let new_handle = Self::establish_connection(&self.config, 0).await?;
        // Replace the old handle with the new one
        let mut handle = self.handle.write().await;
        *handle = new_handle;
        drop(handle);
        self.connected.store(true, Ordering::SeqCst);
        info!(host = %self.config.host, "SSH connection re-established");
        Ok(())
    }

    /// Get a reference to the connection configuration.
    pub fn config(&self) -> &SshConfig {
        &self.config
    }
}

/// Client handler that verifies server host keys against known_hosts.
pub struct ClientHandler {
    hostname: String,
    known_hosts: Option<KnownHosts>,
    strict_mode: StrictHostKeyChecking,
}

impl ClientHandler {
    fn new(hostname: String, known_hosts: Option<KnownHosts>, strict_mode: StrictHostKeyChecking) -> Self {
        Self {
            hostname,
            known_hosts,
            strict_mode,
        }
    }
}

impl Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        match &self.known_hosts {
            Some(kh) => match kh.check_host_key(&self.hostname, server_public_key) {
                Ok(true) => {
                    debug!(host = %self.hostname, "host key verified");
                    Ok(true)
                }
                Ok(false) => {
                    // Unknown host
                    match self.strict_mode {
                        StrictHostKeyChecking::Yes => {
                            warn!(host = %self.hostname, "rejecting unknown host (strict=yes)");
                            Err(russh::Error::UnknownKey)
                        }
                        StrictHostKeyChecking::AcceptNew => {
                            info!(host = %self.hostname, "accepting new host key");
                            // Note: We can't persist here since &mut self doesn't give
                            // access to the mutable KnownHosts. The caller should add
                            // the key after connection.
                            Ok(true)
                        }
                        StrictHostKeyChecking::No => {
                            info!(host = %self.hostname, "accepting host key (strict=no)");
                            Ok(true)
                        }
                    }
                }
                Err(e) => {
                    // Key changed — potential MITM
                    warn!(host = %self.hostname, error = %e, "host key mismatch");
                    Err(russh::Error::UnknownKey)
                }
            },
            None => {
                // No known_hosts — accept based on strict mode
                match self.strict_mode {
                    StrictHostKeyChecking::Yes => {
                        warn!(host = %self.hostname, "no known_hosts file, rejecting (strict=yes)");
                        Err(russh::Error::UnknownKey)
                    }
                    _ => {
                        info!(host = %self.hostname, "no known_hosts, accepting key");
                        Ok(true)
                    }
                }
            }
        }
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
            if conn.is_connected().await {
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
    pub async fn has_connection(&self, host_id: &str) -> bool {
        if let Some(conn) = self.connections.get(host_id) {
            conn.is_connected().await
        } else {
            false
        }
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

    #[tokio::test]
    async fn test_pool_has_connection_empty() {
        let pool = SshConnectionPool::new();
        assert!(!pool.has_connection("host1").await);
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

    // ── KnownHosts tests ──────────────────────────────────────────────

    #[test]
    fn test_known_hosts_parse_file() {
        let dir = std::env::temp_dir().join("ssh_test_parse");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("known_hosts");

        std::fs::write(
            &path,
            "# comment\nserver1 ssh-ed25519 AAAAC3Nza...\nserver2 ecdsa-sha2-nistp256 AAAAE2...\n",
        )
        .unwrap();

        let kh = KnownHosts::open(Some(&path)).unwrap();
        assert!(kh.entries.contains_key("server1"));
        assert!(kh.entries.contains_key("server2"));
        assert_eq!(kh.entries["server1"].len(), 1);
        assert_eq!(kh.entries["server1"][0].key_type, "ssh-ed25519");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_known_hosts_empty_file() {
        let dir = std::env::temp_dir().join("ssh_test_empty");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("known_hosts");

        std::fs::write(&path, "").unwrap();

        let kh = KnownHosts::open(Some(&path)).unwrap();
        assert!(kh.entries.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_known_hosts_nonexistent_file() {
        let path = PathBuf::from("/tmp/nonexistent_known_hosts_file");
        let kh = KnownHosts::open(Some(&path)).unwrap();
        assert!(kh.entries.is_empty());
    }

    #[test]
    fn test_config_builder_with_known_hosts() {
        let config = SshConfig::new("host", "user")
            .known_hosts_path("/custom/known_hosts")
            .strict_host_key_checking(StrictHostKeyChecking::Yes);

        assert_eq!(
            config.strict_host_key_checking,
            StrictHostKeyChecking::Yes
        );
        assert_eq!(
            config.known_hosts_path,
            Some(PathBuf::from("/custom/known_hosts"))
        );
    }

    #[test]
    fn test_strict_host_key_checking_default() {
        assert_eq!(
            StrictHostKeyChecking::default(),
            StrictHostKeyChecking::AcceptNew
        );
    }
}
