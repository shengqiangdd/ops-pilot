# Module SDK Reference

This document specifies the complete Module SDK for OpsPilot, including trait definitions, lifecycle management, configuration, and example implementations.

---

## Table of Contents

- [Overview](#overview)
- [OpsModule Trait](#opsmodule-trait)
- [ToolDefinition](#tooldefinition)
- [ModuleContext](#modulecontext)
- [Event System](#event-system)
- [Lifecycle Hooks](#lifecycle-hooks)
- [Configuration Schema](#configuration-schema)
- [Module Manifest](#module-manifest)
- [Example Module](#example-module)
- [Packaging & Distribution](#packaging--distribution)

---

## Overview

Every OpsPilot module is a Rust crate that implements the `OpsModule` trait from the `ops-pilot-sdk` crate. Modules are loaded at runtime by the core engine and communicate through a well-defined interface.

**Module Responsibilities:**

- Register tools that can be invoked via the API, CLI, or AI Gateway.
- React to events from the event bus.
- Expose health status for monitoring.
- Declare configuration requirements.

**Module Constraints:**

- Must be `Send + Sync + 'static` (required for async multi-threaded execution).
- Must not block the tokio runtime (use `tokio::spawn` for CPU-bound work).
- Must handle panics gracefully (the runtime catches them, but it's unclean).

---

## OpsModule Trait

```rust
use async_trait::async_trait;
use serde_json::Value;

use crate::{ModuleContext, ModuleAction, OpsEvent, ToolDefinition, HealthStatus};

/// The core trait that all OpsPilot modules must implement.
///
/// Each module is a self-contained unit of functionality that registers
/// tools, handles events, and reports health status.
///
/// # Example
///
/// ```rust
/// struct MyModule {
///     config: MyConfig,
/// }
///
/// #[async_trait]
/// impl OpsModule for MyModule {
///     fn name(&self) -> &str {
///         "my-module"
///     }
///
///     fn version(&self) -> &str {
///         "0.1.0"
///     }
///
///     // ... remaining trait methods
/// }
/// ```
#[async_trait]
pub trait OpsModule: Send + Sync + 'static {
    /// Returns the unique module identifier (e.g., "mod-rca", "mod-finops").
    /// Must be lowercase with hyphens, matching the crate name.
    fn name(&self) -> &str;

    /// Returns the semantic version of this module.
    /// Must follow semver: MAJOR.MINOR.PATCH
    fn version(&self) -> &str;

    /// Returns a human-readable description of what this module does.
    /// Shown in the Web UI module browser.
    fn description(&self) -> &str;

    /// Returns the names of other modules this module depends on.
    /// The core engine ensures dependencies are loaded first.
    /// Return an empty vec if there are no dependencies.
    fn dependencies(&self) -> Vec<&str>;

    /// Returns the list of tools this module provides.
    /// Tools are registered with the AI Gateway and API layer.
    /// Each tool can be invoked by users, the CLI, or LLMs.
    fn tools(&self) -> Vec<ToolDefinition>;

    /// Executes a tool invocation.
    ///
    /// # Arguments
    /// * `ctx` — The module context providing access to core services.
    /// * `tool` — The tool name (must match one returned by `tools()`).
    /// * `params` — JSON parameters for the tool invocation.
    ///
    /// # Returns
    /// The tool result as a JSON value, or an error.
    async fn execute(
        &self,
        ctx: &ModuleContext,
        tool: &str,
        params: Value,
    ) -> Result<Value, ModuleError>;

    /// Called when an event is published on the event bus.
    /// Return `Some(ModuleAction)` to request the core engine to perform an action,
    /// or `None` to ignore the event.
    ///
    /// Events are delivered to all modules; filter by event type in your implementation.
    async fn on_event(
        &self,
        ctx: &ModuleContext,
        event: &OpsEvent,
    ) -> Option<ModuleAction>;

    /// Performs a health check and returns the current status.
    /// Called periodically by the monitoring engine (default: every 60 seconds).
    async fn health_check(&self, ctx: &ModuleContext) -> HealthStatus;
}
```

---

## ToolDefinition

Describes a single tool that a module provides. Used for API documentation, AI Gateway function calling, and CLI help text.

```rust
use serde::{Deserialize, Serialize};

/// A tool that can be invoked by users or LLMs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Unique tool name within the module (e.g., "analyze", "scan").
    /// Format: "module_name.tool_name" when registered with the gateway.
    pub name: String,

    /// Human-readable description of what the tool does.
    /// Used by LLMs to decide when to call this tool.
    pub description: String,

    /// JSON Schema describing the expected parameters.
    /// LLMs use this to generate correct input.
    /// Example:
    /// ```json
    /// {
    ///   "type": "object",
    ///   "properties": {
    ///     "host": {
    ///       "type": "string",
    ///       "description": "Target host identifier"
    ///     },
    ///     "command": {
    ///       "type": "string",
    ///       "description": "Shell command to execute"
    ///     }
    ///   },
    ///   "required": ["host", "command"]
    /// }
    /// ```
    pub parameters: serde_json::Value,

    /// Permissions required to invoke this tool.
    /// The user must have ALL listed permissions.
    /// Example: vec!["host.read", "ssh.exec"]
    pub required_permissions: Vec<String>,

    /// Risk level of this tool. Determines UI warnings and confirmation prompts.
    pub risk_level: RiskLevel,

    /// Whether this tool requires an active SSH connection to a host.
    /// If true, the API layer validates connection state before invocation.
    pub requires_connection: bool,

    /// Optional: example invocations for documentation.
    pub examples: Vec<ToolExample>,
}

/// Risk level determines UI behavior and confirmation requirements.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    /// Read-only operations. No confirmation needed.
    /// Example: host.list, cost.analyze
    Low,

    /// State-changing operations. Show confirmation in UI.
    /// Example: ssh.exec (non-destructive), docker.restart
    Medium,

    /// Destructive or irreversible operations. Require explicit confirmation.
    /// Example: ssh.exec (rm, drop), host.delete, security.rotate
    High,

    /// Catastrophic operations. Require admin approval.
    /// Example: factory.reset, database.purge
    Critical,
}

/// Example invocation for documentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExample {
    /// Short description of the example scenario.
    pub description: String,

    /// Example input parameters.
    pub input: serde_json::Value,

    /// Example output.
    pub output: serde_json::Value,
}
```

---

## ModuleContext

Provided to every module method. Grants controlled access to core services without tight coupling.

```rust
use std::sync::Arc;
use sqlx::SqlitePool;

/// The module context provides access to core services.
/// Modules must use this instead of accessing services directly.
pub struct ModuleContext {
    /// Database access (read-only by default; writes go through audit).
    pub db: Arc<SqlitePool>,

    /// Event bus for publishing and subscribing to events.
    pub event_bus: EventBus,

    /// SSH connection pool for executing remote commands.
    pub ssh_pool: Arc<SshConnectionPool>,

    /// Docker client for container operations.
    pub docker: Arc<DockerClient>,

    /// AI Gateway for LLM invocations.
    pub ai: Arc<AiGateway>,

    /// Secrets vault for accessing encrypted credentials.
    pub vault: Arc<SecretsVault>,

    /// The module's own configuration (deserialized from TOML).
    pub config: serde_json::Value,

    /// Current module identifier.
    pub module_id: String,

    /// Logger scoped to this module.
    pub logger: tracing::Span,
}

impl ModuleContext {
    /// Retrieve a secret from the vault.
    /// Access is logged in the audit trail.
    pub async fn get_secret(&self, key: &str) -> Result<String, ModuleError> {
        self.vault.get(key).await.map_err(|e| ModuleError::Vault(e))
    }

    /// Publish an event to the event bus.
    pub async fn emit(&self, event: OpsEvent) {
        self.event_bus.publish(event).await;
    }

    /// Execute an SSH command on a host.
    pub async fn ssh_exec(
        &self,
        host_id: &str,
        command: &str,
    ) -> Result<SshResult, ModuleError> {
        let session = self.ssh_pool
            .get(host_id)
            .await
            .map_err(|e| ModuleError::Ssh(e))?;

        session.exec(command)
            .await
            .map_err(|e| ModuleError::Ssh(e))
    }

    /// Query the database.
    pub async fn query<T: for<'r> sqlx::FromRow<'r, sqlx::SqliteRow>>(
        &self,
        sql: &str,
    ) -> Result<Vec<T>, ModuleError> {
        sqlx::query_as::<_, T>(sql)
            .fetch_all(self.db.as_ref())
            .await
            .map_err(|e| ModuleError::Database(e))
    }

    /// Invoke an LLM with a prompt.
    pub async fn ai_complete(
        &self,
        prompt: &str,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<String, ModuleError> {
        self.ai.complete(prompt, tools).await.map_err(|e| ModuleError::Ai(e))
    }

    /// Log an action to the audit trail.
    pub async fn audit_log(
        &self,
        action: &str,
        target: &str,
        details: serde_json::Value,
    ) {
        let record = AuditRecord {
            module_id: self.module_id.clone(),
            action: action.to_string(),
            target: target.to_string(),
            details,
            timestamp: chrono::Utc::now(),
        };
        self.event_bus.publish(OpsEvent::AuditRecordCreated(record)).await;
    }
}
```

---

## Event System

Events are the primary communication mechanism between modules and the core engine.

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// All events that can flow through the event bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum OpsEvent {
    // ── Host Events ──────────────────────────────────────────
    /// A host was added to the system.
    HostAdded {
        host_id: String,
        name: String,
        ip: String,
    },

    /// A host was removed.
    HostRemoved {
        host_id: String,
    },

    /// Host health status changed.
    HostHealthChanged {
        host_id: String,
        status: HealthStatus,
        previous: HealthStatus,
    },

    /// SSH connection established or lost.
    SshConnectionChanged {
        host_id: String,
        connected: bool,
        reason: Option<String>,
    },

    // ── Alert Events ─────────────────────────────────────────
    /// A monitoring alert was triggered.
    AlertTriggered {
        alert_id: String,
        host_id: String,
        metric: String,
        severity: AlertSeverity,
        message: String,
        value: f64,
        threshold: f64,
    },

    /// An alert was resolved.
    AlertResolved {
        alert_id: String,
        host_id: String,
        metric: String,
        resolution: String,
    },

    // ── Cost Events ──────────────────────────────────────────
    /// A cost anomaly was detected.
    CostAnomalyDetected {
        anomaly_id: String,
        provider: String,
        service: String,
        region: String,
        current_cost: f64,
        expected_cost: f64,
        deviation_pct: f64,
    },

    // ── Security Events ──────────────────────────────────────
    /// A security scan completed.
    SecurityScanCompleted {
        scan_id: String,
        host_id: Option<String>,
        vulnerabilities_found: u32,
        critical: u32,
        high: u32,
    },

    /// A secret was accessed.
    SecretAccessed {
        key: String,
        module_id: String,
        action: String,
    },

    // ── Module Events ────────────────────────────────────────
    /// A module was loaded.
    ModuleLoaded {
        module_id: String,
        version: String,
    },

    /// A module encountered an error.
    ModuleError {
        module_id: String,
        error: String,
        tool: Option<String>,
    },

    /// A module's health status changed.
    ModuleHealthChanged {
        module_id: String,
        status: HealthStatus,
    },

    // ── Audit Events ─────────────────────────────────────────
    /// An audit record was created.
    AuditRecordCreated(AuditRecord),

    // ── System Events ────────────────────────────────────────
    /// System startup.
    SystemStarted {
        version: String,
        modules_loaded: Vec<String>,
    },

    /// System shutdown initiated.
    SystemShutdown {
        reason: String,
    },
}

/// Severity level for alerts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// An audit trail record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub module_id: String,
    pub action: String,
    pub target: String,
    pub details: serde_json::Value,
    pub risk_level: String,
    pub ai_generated: bool,
}

/// Action that a module can request from the core engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleAction {
    /// Execute an SSH command on a host.
    SshExec {
        host_id: String,
        command: String,
    },

    /// Send a notification (email, Slack, webhook).
    Notify {
        channel: String,
        message: String,
        severity: AlertSeverity,
    },

    /// Create a new alert.
    CreateAlert {
        host_id: String,
        metric: String,
        message: String,
        severity: AlertSeverity,
    },

    /// Schedule a future action.
    Schedule {
        cron: String,
        action: Box<ModuleAction>,
    },

    /// Invoke another module's tool.
    InvokeTool {
        module: String,
        tool: String,
        params: serde_json::Value,
    },
}
```

---

## Lifecycle Hooks

Modules go through a defined lifecycle managed by the core engine.

```rust
use async_trait::async_trait;

/// Extended lifecycle hooks for modules that need them.
/// These are optional; default implementations are no-ops.
#[async_trait]
pub trait ModuleLifecycle: OpsModule {
    /// Called once when the module is first loaded.
    /// Use this for one-time setup (e.g., run migrations, create tables).
    ///
    /// If this returns an error, the module fails to load and is disabled.
    async fn init(&self, ctx: &ModuleContext) -> Result<(), ModuleError> {
        Ok(())
    }

    /// Called when the module is started (after init).
    /// Start background tasks, subscribe to events, begin periodic work.
    async fn start(&self, ctx: &ModuleContext) -> Result<(), ModuleError> {
        Ok(())
    }

    /// Called when the module is being stopped.
    /// Gracefully shut down background tasks, flush buffers.
    /// You have up to 30 seconds before force-kill.
    async fn stop(&self, ctx: &ModuleContext) -> Result<(), ModuleError> {
        Ok(())
    }

    /// Called when the module's configuration is updated.
    /// Reload configuration without restarting the module.
    async fn reload_config(
        &self,
        ctx: &ModuleContext,
        new_config: serde_json::Value,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    /// Called when a dependent module fails.
    /// Decide whether to degrade gracefully or stop.
    async fn on_dependency_failed(
        &self,
        ctx: &ModuleContext,
        dependency: &str,
        error: &ModuleError,
    ) -> Result<(), ModuleError> {
        Ok(())
    }
}
```

**Lifecycle State Machine:**

```
                    ┌──────────┐
                    │  Loaded  │
                    └────┬─────┘
                         │ init()
                         ▼
                    ┌──────────┐
                    │  Ready   │
                    └────┬─────┘
                         │ start()
                         ▼
                    ┌──────────┐
         ┌─────────│ Running  │─────────┐
         │         └────┬─────┘         │
         │              │ stop()        │
         │              ▼               │
         │         ┌──────────┐         │
         │         │ Stopped  │         │
         │         └──────────┘         │
         │                              │
         │ reload_config()              │
         │ (stays Running)              │
         │                              │
         └── on_dependency_failed()     │
             (may trigger stop)         │
                                        │
              Error in init/start ──────┘
              → Failed state
```

---

## Configuration Schema

Each module declares its configuration schema in TOML format. The core engine validates configuration at startup and provides it to the module via `ModuleContext::config`.

```toml
# config.toml — Example module configuration

[modules.mod-rca]
enabled = true

[modules.mod-rca.config]
# Maximum number of log lines to analyze per incident
max_log_lines = 10000

# LLM provider for analysis (inherits global if omitted)
llm_provider = "ollama"
llm_model = "qwen2.5:32b"

# Time window for log correlation (in minutes)
correlation_window = 30

# Auto-fix confidence threshold (0.0 - 1.0)
# Actions below this confidence require manual approval
auto_fix_threshold = 0.8

# Severity levels that trigger automatic analysis
auto_analyze_severities = ["warning", "critical"]

[modules.mod-rca.config.alert_sources]
# Where to pull alerts from
prometheus = true
grafana = true
custom_webhooks = true

[modules.mod-finops]
enabled = true

[modules.mod-finops.config]
scan_interval_hours = 6
anomaly_deviation_sigma = 3.0

[modules.mod-finops.config.providers.aws]
enabled = true
# AWS credentials from secrets vault
credentials_key = "aws_access_key"
secret_key = "aws_secret_key"
regions = ["us-east-1", "us-west-2", "eu-west-1"]

[modules.mod-finops.config.providers.gcp]
enabled = false

[modules.mod-security]
enabled = false

[modules.mod-security.config]
scan_schedule = "0 2 * * *"  # Daily at 2 AM
compliance_frameworks = ["cis", "soc2"]
```

**Schema Validation:**

Modules can optionally define a JSON Schema for their configuration:

```rust
impl OpsModule for MyModule {
    fn config_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "max_log_lines": {
                    "type": "integer",
                    "default": 10000,
                    "minimum": 100,
                    "description": "Maximum log lines per incident"
                },
                "correlation_window": {
                    "type": "integer",
                    "default": 30,
                    "minimum": 1,
                    "description": "Correlation window in minutes"
                }
            },
            "required": []
        }))
    }
}
```

---

## Module Manifest

Every module crate must include a `module.toml` manifest in its root directory. The core engine reads this at load time.

```toml
# module.toml — Module manifest for mod-rca

[package]
name = "mod-rca"
version = "0.1.0"
description = "AI-powered Root Cause Analysis for infrastructure incidents"
authors = ["OpsPilot Contributors"]
license = "MIT"
repository = "https://github.com/OWNER/ops-pilot"

[module]
# Unique identifier (must match crate name)
id = "mod-rca"

# Minimum OpsPilot version required
min_ops_version = "0.1.0"

# Module category for the marketplace
category = "analysis"

# Tags for searchability
tags = ["rca", "incident", "ai", "logs", "analysis"]

# Icon (emoji or URL)
icon = "🔍"

# Color for the UI badge
color = "#f59e0b"

# Entry point crate
crate = "mod_rca"

# Dependencies on other modules
dependencies = ["mod-core"]

# Optional: capabilities this module provides
[module.capabilities]
ai_tools = true       # Registers tools with the AI Gateway
event_handlers = true  # Listens to events on the bus
scheduled_tasks = true # Has cron-based periodic tasks

# Module configuration schema (JSON Schema)
[module.config_schema]
type = "object"
properties.max_log_lines.type = "integer"
properties.max_log_lines.default = 10000

# Required permissions for this module to function
[module.permissions]
requires = ["host.read", "ssh.exec", "audit.write"]
```

---

## Example Module

A complete minimal "Hello World" module that demonstrates all SDK features.

### `src/modules/mod-hello/Cargo.toml`

```toml
[package]
name = "mod-hello"
version = "0.1.0"
edition = "2021"
license = "MIT"

[lib]
name = "mod_hello"
crate-type = ["cdylib"]

[dependencies]
ops-pilot-sdk = { path = "../../sdk" }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }
```

### `src/modules/mod-hello/src/lib.rs`

```rust
use async_trait::async_trait;
use ops_pilot_sdk::{
    HealthStatus, ModuleAction, ModuleContext, ModuleError, OpsEvent, OpsModule,
    RiskLevel, ToolDefinition, ToolExample,
};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Configuration for the hello module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloConfig {
    /// A greeting message.
    pub greeting: String,

    /// Whether to log every invocation.
    pub verbose: bool,
}

impl Default for HelloConfig {
    fn default() -> Self {
        Self {
            greeting: "Hello from OpsPilot!".to_string(),
            verbose: false,
        }
    }
}

/// The Hello World module — a minimal example for learning the SDK.
pub struct HelloModule {
    config: HelloConfig,
    invocation_count: std::sync::atomic::AtomicU64,
}

impl HelloModule {
    pub fn new() -> Self {
        Self {
            config: HelloConfig::default(),
            invocation_count: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

#[async_trait]
impl OpsModule for HelloModule {
    fn name(&self) -> &str {
        "mod-hello"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "A minimal example module for learning the OpsPilot SDK"
    }

    fn dependencies(&self) -> Vec<&str> {
        vec![] // No dependencies
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "greet".to_string(),
                description: "Returns a greeting message".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name to greet",
                            "default": "World"
                        }
                    },
                    "required": []
                }),
                required_permissions: vec!["dashboard.read".to_string()],
                risk_level: RiskLevel::Low,
                requires_connection: false,
                examples: vec![ToolExample {
                    description: "Greet a user".to_string(),
                    input: serde_json::json!({"name": "OpsPilot"}),
                    output: serde_json::json!({
                        "message": "Hello from OpsPilot, OpsPilot!",
                        "invocation_count": 1
                    }),
                }],
            },
            ToolDefinition {
                name: "count".to_string(),
                description: "Returns the total number of invocations".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
                required_permissions: vec!["dashboard.read".to_string()],
                risk_level: RiskLevel::Low,
                requires_connection: false,
                examples: vec![],
            },
        ]
    }

    async fn execute(
        &self,
        ctx: &ModuleContext,
        tool: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, ModuleError> {
        let count = self.invocation_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;

        if self.config.verbose {
            info!(tool = tool, count = count, "Tool invoked");
        }

        match tool {
            "greet" => {
                let name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("World");

                let message = format!("{}, {}!", self.config.greeting, name);

                ctx.audit_log("hello.greet", &name, serde_json::json!({"name": name})).await;

                Ok(serde_json::json!({
                    "message": message,
                    "invocation_count": count,
                }))
            }
            "count" => {
                Ok(serde_json::json!({
                    "invocation_count": count,
                }))
            }
            _ => Err(ModuleError::UnknownTool(tool.to_string())),
        }
    }

    async fn on_event(
        &self,
        _ctx: &ModuleContext,
        event: &OpsEvent,
    ) -> Option<ModuleAction> {
        // This module doesn't react to any events.
        // In a real module, you'd match on event types:
        //
        // match event {
        //     OpsEvent::AlertTriggered { severity: AlertSeverity::Critical, .. } => {
        //         Some(ModuleAction::Notify { ... })
        //     }
        //     _ => None,
        // }
        let _ = event;
        None
    }

    async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus {
        let count = self.invocation_count.load(std::sync::atomic::Ordering::Relaxed);
        HealthStatus::Healthy {
            message: format!("OK — {} invocations", count),
            details: Some(serde_json::json!({
                "invocation_count": count,
                "greeting": self.config.greeting,
            })),
        }
    }
}

// ── Module Registration ──────────────────────────────────────────

/// Entry point called by the module loader.
/// Returns the module instance.
pub fn create_module() -> Box<dyn OpsModule> {
    Box::new(HelloModule::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ops_pilot_sdk::test::MockModuleContext;

    #[tokio::test]
    async fn test_greet_tool() {
        let module = HelloModule::new();
        let ctx = MockModuleContext::new();
        let result = module
            .execute(&ctx, "greet", serde_json::json!({"name": "Test"}))
            .await
            .unwrap();

        assert_eq!(result["message"], "Hello from OpsPilot, Test!");
        assert_eq!(result["invocation_count"], 1);
    }

    #[tokio::test]
    async fn test_greet_default_name() {
        let module = HelloModule::new();
        let ctx = MockModuleContext::new();
        let result = module
            .execute(&ctx, "greet", serde_json::json!({}))
            .await
            .unwrap();

        assert_eq!(result["message"], "Hello from OpsPilot, World!");
    }

    #[tokio::test]
    async fn test_count_tool() {
        let module = HelloModule::new();
        let ctx = MockModuleContext::new();

        module.execute(&ctx, "greet", serde_json::json!({})).await.unwrap();
        module.execute(&ctx, "greet", serde_json::json!({})).await.unwrap();
        let result = module.execute(&ctx, "count", serde_json::json!({})).await.unwrap();

        assert_eq!(result["invocation_count"], 3);
    }

    #[tokio::test]
    async fn test_unknown_tool() {
        let module = HelloModule::new();
        let ctx = MockModuleContext::new();
        let result = module
            .execute(&ctx, "nonexistent", serde_json::json!({}))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_health_check() {
        let module = HelloModule::new();
        let ctx = MockModuleContext::new();
        let status = module.health_check(&ctx).await;

        match status {
            HealthStatus::Healthy { .. } => {}
            _ => panic!("Expected Healthy status"),
        }
    }

    #[tokio::test]
    async fn test_tool_metadata() {
        let module = HelloModule::new();
        let tools = module.tools();

        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].name, "greet");
        assert_eq!(tools[1].name, "count");
        assert_eq!(tools[0].risk_level, RiskLevel::Low);
    }
}
```

---

## Packaging & Distribution

### Building a Module

Modules are compiled as dynamic libraries (`.so`/`.dylib`/`.dll`):

```bash
# Build the module
cd src/modules/mod-hello
cargo build --release

# The output is: target/release/libmod_hello.so (Linux)
#                target/release/libmod_hello.dylib (macOS)
```

### Installing a Module

**Manual installation:**

```bash
# Copy the compiled library
cp target/release/libmod_hello.so /var/lib/ops-pilot/modules/

# Copy the manifest
cp module.toml /var/lib/ops-pilot/modules/mod-hello/

# Restart ops-pilot or use the API to reload modules
curl -X POST http://localhost:3000/api/v1/modules/mod-hello/enable \
  -H "Authorization: Bearer $TOKEN"
```

**CLI installation:**

```bash
ops-pilot module install mod-hello
ops-pilot module install https://github.com/user/mod-hello/releases/download/v0.1.0/mod-hello-0.1.0.tar.gz
```

### Module Marketplace (Phase 4)

```bash
# Search for modules
ops-pilot module search rca

# Install from marketplace
ops-pilot module install mod-rca@0.2.0

# Update all modules
ops-pilot module update --all
```

### Creating a Release

Each module crate should include a GitHub Actions workflow:

```yaml
# .github/workflows/module-release.yml
name: Module Release
on:
  push:
    tags:
      - 'mod-*@*'

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      - name: Package
        run: |
          MODULE_NAME=$(echo ${{ github.ref_name }} | cut -d@ -f1)
          MODULE_VERSION=$(echo ${{ github.ref_name }} | cut -d@ -f2)
          tar czf ${MODULE_NAME}-${MODULE_VERSION}.tar.gz \
            -C target/release lib${MODULE_NAME//-/_}.so \
            module.toml
      - name: Release
        uses: softprops/action-gh-release@v2
        with:
          files: "*.tar.gz"
```

### Versioning

Follow semver. Breaking changes to the `OpsModule` trait bump the major version. New tools or events are minor versions. Bug fixes are patch versions.

```toml
# In your module's Cargo.toml
[package]
version = "0.2.0"  # Added new tool, no breaking changes
```
