//! SSH sub-module: exposes SSH connection and command execution as tools.

use std::sync::Arc;

use async_trait::async_trait;
use ops_pilot_core::crypto::{self, MasterKey};
use ops_pilot_core::ssh::{SshConfig, SshConnectionPool};
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
use sqlx::FromRow;

/// Row type for querying host credentials from the database.
#[derive(Debug, FromRow)]
struct HostRow {
    id: String,
    address: String,
    port: i32,
    username: String,
    credentials_encrypted: Option<Vec<u8>>,
    credentials_iv: Option<Vec<u8>>,
}

pub struct SshModule {
    pool: Arc<SshConnectionPool>,
}

impl Default for SshModule {
    fn default() -> Self {
        Self::new()
    }
}

impl SshModule {
    pub fn new() -> Self {
        Self {
            pool: Arc::new(SshConnectionPool::new()),
        }
    }

    pub fn with_pool(pool: Arc<SshConnectionPool>) -> Self {
        Self { pool }
    }

    /// Query host from DB and build an SshConfig.
    async fn resolve_host_config(
        db: &sqlx::SqlitePool,
        host_id: &str,
    ) -> anyhow::Result<SshConfig> {
        let row: HostRow = sqlx::query_as(
            "SELECT id, address, port, username, credentials_encrypted, credentials_iv \
             FROM hosts WHERE id = ?",
        )
        .bind(host_id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("host not found: {}", host_id))?;

        let key = MasterKey::load();
        let (password, private_key) = match (row.credentials_encrypted, row.credentials_iv) {
            (Some(ciphertext), Some(iv)) => {
                let plaintext = crypto::decrypt(&ciphertext, &key, &iv)
                    .map_err(|e| anyhow::anyhow!("failed to decrypt credentials: {}", e))?;
                let map: serde_json::Value = serde_json::from_slice(&plaintext)
                    .map_err(|e| anyhow::anyhow!("failed to parse credentials: {}", e))?;
                let password = map
                    .get("password")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let private_key = map
                    .get("private_key")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                (password, private_key)
            }
            _ => (None, None),
        };

        let mut config = SshConfig::new(&row.address, &row.username).port(row.port as u16);

        if let Some(pw) = password {
            config = config.password(pw);
        }
        if let Some(pk) = private_key {
            let path = std::env::temp_dir().join(format!("ops_pilot_key_{}", row.id));
            std::fs::write(&path, &pk)
                .map_err(|e| anyhow::anyhow!("failed to write temp key: {}", e))?;
            config = config.key_path(path.to_string_lossy());
        }

        Ok(config)
    }
}

#[async_trait]
impl OpsModule for SshModule {
    fn name(&self) -> &str {
        "mod-core::ssh"
    }
    fn version(&self) -> &str {
        "0.1.0"
    }
    fn description(&self) -> &str {
        "SSH connection management and remote command execution"
    }
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "ssh_connect".into(),
                description: "Establish an SSH connection to a remote host".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": { "type": "string", "description": "UUID of the host" }
                    },
                    "required": ["host_id"]
                }),
            },
            ToolDefinition {
                name: "ssh_exec".into(),
                description: "Execute a command on a remote host via SSH".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": { "type": "string", "description": "UUID of the host" },
                        "command": { "type": "string", "description": "Command to execute" }
                    },
                    "required": ["host_id", "command"]
                }),
            },
            ToolDefinition {
                name: "ssh_disconnect".into(),
                description: "Disconnect an SSH session".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": { "type": "string", "description": "UUID of the host" }
                    },
                    "required": ["host_id"]
                }),
            },
            ToolDefinition {
                name: "batch_ssh_exec".into(),
                description: "Execute a command on multiple hosts in parallel".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_ids": { "type": "array", "items": { "type": "string" }, "description": "List of host UUIDs" },
                        "command": { "type": "string", "description": "Command to execute" },
                        "timeout_secs": { "type": "integer", "description": "Per-host timeout in seconds", "default": 30 }
                    },
                    "required": ["host_ids", "command"]
                }),
            },
        ]
    }

    async fn execute(
        &self,
        ctx: &ModuleContext,
        tool: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        tracing::info!(tool, "ssh module execute");
        match tool {
            "ssh_connect" => {
                let host_id = params["host_id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("host_id is required"))?;

                let config = Self::resolve_host_config(ctx.db(), host_id).await?;
                self.pool.connect(host_id, config).await?;

                Ok(serde_json::json!({
                    "status": "connected",
                    "host_id": host_id
                }))
            }
            "ssh_exec" => {
                let host_id = params["host_id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("host_id is required"))?;
                let command = params["command"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("command is required"))?;

                let conn = self.pool.get(host_id).await?;
                let output = conn.exec(command).await?;

                Ok(serde_json::json!({
                    "exit_code": 0,
                    "output": output
                }))
            }
            "ssh_disconnect" => {
                let host_id = params["host_id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("host_id is required"))?;

                self.pool.disconnect(host_id).await?;

                Ok(serde_json::json!({
                    "status": "disconnected",
                    "host_id": host_id
                }))
            }
            "batch_ssh_exec" => {
                let host_ids: Vec<String> =
                    serde_json::from_value(params["host_ids"].clone())?;
                let command = params["command"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing command"))?;
                let timeout_secs = params
                    .get("timeout_secs")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(30);
                let timeout = std::time::Duration::from_secs(timeout_secs);

                let mut handles = Vec::new();
                for host_id in &host_ids {
                    let pool = self.pool.clone();
                    let cmd = command.to_string();
                    let hid = host_id.clone();
                    handles.push(tokio::spawn(async move {
                        match tokio::time::timeout(
                            timeout,
                            async {
                                let conn = pool.get(&hid).await?;
                                conn.exec(&cmd).await
                            },
                        )
                        .await
                        {
                            Ok(Ok(output)) => {
                                (
                                    hid.clone(),
                                    serde_json::json!({"success": true, "output": output}),
                                )
                            }
                            Ok(Err(e)) => (
                                hid.clone(),
                                serde_json::json!({"success": false, "error": e.to_string()}),
                            ),
                            Err(_) => (
                                hid.clone(),
                                serde_json::json!({"success": false, "error": "timeout"}),
                            ),
                        }
                    }));
                }

                let mut results = serde_json::Map::new();
                for handle in handles {
                    let (host_id, result) = handle.await?;
                    results.insert(host_id, result);
                }
                Ok(serde_json::Value::Object(results))
            }
            _ => anyhow::bail!("unknown tool: {}", tool),
        }
    }

    async fn on_event(&self, _ctx: &ModuleContext, _event: &OpsEvent) -> Option<ModuleAction> {
        None
    }

    async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus {
        HealthStatus::Healthy
    }
}
