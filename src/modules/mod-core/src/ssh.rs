//! SSH sub-module: exposes SSH connection and command execution as tools.

use async_trait::async_trait;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};

pub struct SshModule;

impl Default for SshModule {
    fn default() -> Self {
        Self::new()
    }
}

impl SshModule {
    pub fn new() -> Self {
        Self
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
        ]
    }

    async fn execute(
        &self,
        _ctx: &ModuleContext,
        tool: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        // TODO: Wire up to ops_pilot_core::ssh::SshConnectionPool
        tracing::info!(tool, "ssh module execute (stub)");
        match tool {
            "ssh_connect" => Ok(serde_json::json!({"status": "connected", "host_id": params["host_id"]})),
            "ssh_exec" => Ok(serde_json::json!({"exit_code": 0, "output": "stub output"})),
            "ssh_disconnect" => Ok(serde_json::json!({"status": "disconnected"})),
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
