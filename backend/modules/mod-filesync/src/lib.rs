//! mod-filesync: File distribution module for pushing, pulling, and managing
//! remote files via SSH.
//!
//! Transfers are performed by base64-encoding file content and piping it through
//! SSH exec commands. Large files are chunked to stay within command-line limits.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use base64::Engine;
use chrono::Utc;
use ops_pilot_core::ssh::SshConnectionPool;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
use tracing::info;

/// Maximum bytes per base64 chunk (3 MiB raw → ~4 MiB encoded).
const CHUNK_SIZE: usize = 3 * 1024 * 1024;

/// File distribution module — transfers files over SSH using base64 encoding.
pub struct FileSyncModule {
    pool: Arc<SshConnectionPool>,
}

impl FileSyncModule {
    pub fn new(pool: Arc<SshConnectionPool>) -> Self {
        Self { pool }
    }

    /// Push a local file to a remote host.
    /// Reads the file in chunks, base64-encodes each chunk, and pipes it
    /// to `base64 -d` on the remote side.
    async fn push_file(
        &self,
        host_id: &str,
        source_path: &str,
        dest_path: &str,
    ) -> anyhow::Result<String> {
        let conn = self.pool.get(host_id).await.map_err(|e| {
            anyhow::anyhow!("no active SSH connection for host '{}': {}", host_id, e)
        })?;

        let data = tokio::fs::read(source_path).await?;
        let total_chunks = data.len().div_ceil(CHUNK_SIZE);

        // Create destination directory on remote host
        if let Some(parent) = Path::new(dest_path).parent() {
            let parent_str = parent.to_string_lossy();
            conn.exec(&format!("mkdir -p {}", parent_str)).await?;
        }

        for (i, chunk) in data.chunks(CHUNK_SIZE).enumerate() {
            let encoded = base64::engine::general_purpose::STANDARD.encode(chunk);
            let cmd = if i == 0 {
                format!("echo '{}' | base64 -d > {}", encoded, dest_path)
            } else {
                format!("echo '{}' | base64 -d >> {}", encoded, dest_path)
            };
            conn.exec(&cmd).await?;
        }

        let msg = format!(
            "pushed {} bytes ({} chunks) to {}@{}:{}",
            data.len(),
            total_chunks,
            conn.config().username,
            conn.config().host,
            dest_path
        );
        info!(source_path, dest_path, host_id, "{}", msg);
        Ok(msg)
    }

    /// Pull a remote file to a local path.
    /// Executes `base64 <remote_path>` on the remote host and decodes the output.
    async fn pull_file(
        &self,
        host_id: &str,
        remote_path: &str,
        local_path: &str,
    ) -> anyhow::Result<String> {
        let conn = self.pool.get(host_id).await.map_err(|e| {
            anyhow::anyhow!("no active SSH connection for host '{}': {}", host_id, e)
        })?;

        let output = conn.exec(&format!("base64 {}", remote_path)).await?;
        let trimmed = output.trim();
        let data = base64::engine::general_purpose::STANDARD.decode(trimmed)?;

        // Ensure local parent directory exists
        if let Some(parent) = Path::new(local_path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(local_path, &data).await?;

        let msg = format!(
            "pulled {} bytes from {}@{}:{} to {}",
            data.len(),
            conn.config().username,
            conn.config().host,
            remote_path,
            local_path
        );
        info!(remote_path, local_path, host_id, "{}", msg);
        Ok(msg)
    }

    /// Get file info (size, permissions, mtime) from a remote host.
    async fn stat_file(&self, host_id: &str, path: &str) -> anyhow::Result<serde_json::Value> {
        let conn = self.pool.get(host_id).await.map_err(|e| {
            anyhow::anyhow!("no active SSH connection for host '{}': {}", host_id, e)
        })?;

        let output = conn
            .exec(&format!(
                "stat -c 'size=%s mode=%a mtime=%Y' {} 2>/dev/null || ls -la {}",
                path, path
            ))
            .await?;

        let mut info = serde_json::json!({
            "host_id": host_id,
            "path": path,
            "raw": output.trim(),
        });

        // Try to parse stat output: "size=1234 mode=644 mtime=1700000000"
        for part in output.split_whitespace() {
            if let Some(val) = part.strip_prefix("size=") {
                if let Ok(s) = val.parse::<u64>() {
                    info["size"] = serde_json::json!(s);
                }
            } else if let Some(val) = part.strip_prefix("mode=") {
                info["mode"] = serde_json::json!(val);
            } else if let Some(val) = part.strip_prefix("mtime=") {
                if let Ok(t) = val.parse::<i64>() {
                    info["mtime"] = serde_json::json!(t);
                }
            }
        }

        Ok(info)
    }

    /// Backup a remote file to a backup directory.
    async fn backup_file(
        &self,
        host_id: &str,
        path: &str,
        backup_dir: &str,
    ) -> anyhow::Result<String> {
        let conn = self.pool.get(host_id).await.map_err(|e| {
            anyhow::anyhow!("no active SSH connection for host '{}': {}", host_id, e)
        })?;

        let filename = Path::new(path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "backup".to_string());

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let dest = format!("{}/{}.{}", backup_dir, filename, timestamp);

        // Ensure backup dir exists and copy
        conn.exec(&format!("mkdir -p {}", backup_dir)).await?;
        let output = conn.exec(&format!("cp {} {}", path, dest)).await?;

        let msg = format!(
            "backed up {} to {}@{}:{}",
            path,
            conn.config().username,
            conn.config().host,
            dest
        );
        info!(path, dest, host_id, "{}", msg);
        let _ = output;
        Ok(msg)
    }
}

#[async_trait]
impl OpsModule for FileSyncModule {
    fn name(&self) -> &str {
        "mod-filesync"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "File distribution module — push, pull, stat, and backup files via SSH"
    }

    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "file_push".into(),
                description: "Push a local file to a remote host via SSH".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": { "type": "string", "description": "UUID of the target host" },
                        "source_path": { "type": "string", "description": "Local file path to read" },
                        "dest_path": { "type": "string", "description": "Remote destination path" },
                        "timeout_secs": { "type": "integer", "description": "Timeout in seconds (default 30)", "default": 30 }
                    },
                    "required": ["host_id", "source_path", "dest_path"]
                }),
            },
            ToolDefinition {
                name: "file_pull".into(),
                description: "Pull a file from a remote host to local disk via SSH".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": { "type": "string", "description": "UUID of the target host" },
                        "remote_path": { "type": "string", "description": "Remote file path to read" },
                        "local_path": { "type": "string", "description": "Local destination path" },
                        "timeout_secs": { "type": "integer", "description": "Timeout in seconds (default 30)", "default": 30 }
                    },
                    "required": ["host_id", "remote_path", "local_path"]
                }),
            },
            ToolDefinition {
                name: "file_stat".into(),
                description: "Get file information (size, permissions, mtime) on a remote host".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": { "type": "string", "description": "UUID of the target host" },
                        "path": { "type": "string", "description": "Remote file path" }
                    },
                    "required": ["host_id", "path"]
                }),
            },
            ToolDefinition {
                name: "file_backup".into(),
                description: "Backup a remote file by copying it to a backup directory".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host_id": { "type": "string", "description": "UUID of the target host" },
                        "path": { "type": "string", "description": "Remote file path to backup" },
                        "backup_dir": { "type": "string", "description": "Backup directory (default /tmp/backup/)", "default": "/tmp/backup/" }
                    },
                    "required": ["host_id", "path"]
                }),
            },
        ]
    }

    async fn execute(
        &self,
        _ctx: &ModuleContext,
        tool: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        match tool {
            "file_push" => {
                let host_id = params["host_id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'host_id' parameter"))?;
                let source_path = params["source_path"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'source_path' parameter"))?;
                let dest_path = params["dest_path"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'dest_path' parameter"))?;

                let msg = self.push_file(host_id, source_path, dest_path).await?;
                Ok(serde_json::json!({ "status": "ok", "message": msg }))
            }
            "file_pull" => {
                let host_id = params["host_id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'host_id' parameter"))?;
                let remote_path = params["remote_path"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'remote_path' parameter"))?;
                let local_path = params["local_path"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'local_path' parameter"))?;

                let msg = self.pull_file(host_id, remote_path, local_path).await?;
                Ok(serde_json::json!({ "status": "ok", "message": msg }))
            }
            "file_stat" => {
                let host_id = params["host_id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'host_id' parameter"))?;
                let path = params["path"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'path' parameter"))?;

                let info = self.stat_file(host_id, path).await?;
                Ok(info)
            }
            "file_backup" => {
                let host_id = params["host_id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'host_id' parameter"))?;
                let path = params["path"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'path' parameter"))?;
                let backup_dir = params["backup_dir"].as_str().unwrap_or("/tmp/backup/");

                let msg = self.backup_file(host_id, path, backup_dir).await?;
                Ok(serde_json::json!({ "status": "ok", "message": msg }))
            }
            _ => Err(anyhow::anyhow!("unknown tool: {}", tool)),
        }
    }

    async fn on_event(&self, _ctx: &ModuleContext, _event: &OpsEvent) -> Option<ModuleAction> {
        None
    }

    async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus {
        HealthStatus::Healthy
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ops_pilot_sdk::context::{EventBus, ModuleContext};
    use std::path::PathBuf;

    async fn setup() -> (FileSyncModule, ModuleContext) {
        let pool = Arc::new(SshConnectionPool::new());
        let module = FileSyncModule::new(pool.clone());
        let ctx = ModuleContext::new(
            Arc::new(
                sqlx::SqlitePool::connect("sqlite::memory:")
                    .await
                    .unwrap(),
            ),
            EventBus::new(16),
            PathBuf::from("/tmp/test-filesync"),
            "test-filesync".into(),
        );
        (module, ctx)
    }

    #[tokio::test]
    async fn test_module_metadata() {
        let (m, _ctx) = setup().await;
        assert_eq!(m.name(), "mod-filesync");
        assert_eq!(m.version(), "0.1.0");
        assert!(m.description().contains("file"));
    }

    #[tokio::test]
    async fn test_tools_registered() {
        let (m, _ctx) = setup().await;
        let tools = m.tools();
        assert_eq!(tools.len(), 4);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"file_push"));
        assert!(names.contains(&"file_pull"));
        assert!(names.contains(&"file_stat"));
        assert!(names.contains(&"file_backup"));
    }

    #[tokio::test]
    async fn test_push_file_no_connection() {
        let (m, ctx) = setup().await;
        let result = m
            .execute(
                &ctx,
                "file_push",
                serde_json::json!({
                    "host_id": "nonexistent",
                    "source_path": "/tmp/test.txt",
                    "dest_path": "/remote/test.txt"
                }),
            )
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no active SSH"));
    }

    #[tokio::test]
    async fn test_pull_file_no_connection() {
        let (m, ctx) = setup().await;
        let result = m
            .execute(
                &ctx,
                "file_pull",
                serde_json::json!({
                    "host_id": "nonexistent",
                    "remote_path": "/remote/test.txt",
                    "local_path": "/tmp/test.txt"
                }),
            )
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no active SSH"));
    }

    #[tokio::test]
    async fn test_stat_file_no_connection() {
        let (m, ctx) = setup().await;
        let result = m
            .execute(
                &ctx,
                "file_stat",
                serde_json::json!({
                    "host_id": "nonexistent",
                    "path": "/etc/hostname"
                }),
            )
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no active SSH"));
    }

    #[tokio::test]
    async fn test_backup_file_no_connection() {
        let (m, ctx) = setup().await;
        let result = m
            .execute(
                &ctx,
                "file_backup",
                serde_json::json!({
                    "host_id": "nonexistent",
                    "path": "/etc/hostname"
                }),
            )
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no active SSH"));
    }

    #[tokio::test]
    async fn test_push_missing_params() {
        let (m, ctx) = setup().await;
        let result = m
            .execute(
                &ctx,
                "file_push",
                serde_json::json!({ "host_id": "h1" }),
            )
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing"));
    }

    #[tokio::test]
    async fn test_pull_missing_params() {
        let (m, ctx) = setup().await;
        let result = m
            .execute(
                &ctx,
                "file_pull",
                serde_json::json!({ "host_id": "h1" }),
            )
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing"));
    }

    #[tokio::test]
    async fn test_stat_missing_params() {
        let (m, ctx) = setup().await;
        let result = m
            .execute(&ctx, "file_stat", serde_json::json!({ "host_id": "h1" }))
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing"));
    }

    #[tokio::test]
    async fn test_backup_missing_params() {
        let (m, ctx) = setup().await;
        let result = m
            .execute(
                &ctx,
                "file_backup",
                serde_json::json!({ "host_id": "h1" }),
            )
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing"));
    }

    #[tokio::test]
    async fn test_unknown_tool() {
        let (m, ctx) = setup().await;
        let result = m.execute(&ctx, "nonexistent", serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_chunk_size_constant() {
        // Ensure CHUNK_SIZE is reasonable for base64 transfer
        assert!(CHUNK_SIZE >= 1024);
        assert!(CHUNK_SIZE <= 10 * 1024 * 1024);
    }
}
