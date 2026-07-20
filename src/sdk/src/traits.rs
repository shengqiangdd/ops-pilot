use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::context::ModuleContext;
use crate::events::OpsEvent;

/// Health status of a module.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}

/// Definition of a tool exposed by a module.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

/// Action that a module can request in response to an event.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModuleAction {
    pub tool: String,
    pub params: Value,
}

/// The core trait that every OpsPilot module must implement.
#[async_trait]
pub trait OpsModule: Send + Sync + 'static {
    /// Unique module name (e.g. "rca", "finops").
    fn name(&self) -> &str;

    /// Semver version string.
    fn version(&self) -> &str;

    /// Human-readable description.
    fn description(&self) -> &str;

    /// Module names this module depends on.
    fn dependencies(&self) -> Vec<&str>;

    /// Tools this module exposes to the AI gateway.
    fn tools(&self) -> Vec<ToolDefinition>;

    /// Execute a tool with the given parameters.
    async fn execute(&self, ctx: &ModuleContext, tool: &str, params: Value) -> anyhow::Result<Value>;

    /// Handle an event and optionally return an action.
    async fn on_event(&self, ctx: &ModuleContext, event: &OpsEvent) -> Option<ModuleAction>;

    /// Health check callback.
    async fn health_check(&self, ctx: &ModuleContext) -> HealthStatus;
}

/// Errors returned by [`ModuleManifest::validate`].
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("module name must be lowercase with hyphens (e.g. \"mod-rca\"), got: \"{0}\"")]
    InvalidName(String),
    #[error("module name must not be empty")]
    EmptyName,
    #[error("module version must not be empty")]
    EmptyVersion,
}

/// Metadata declared by a module crate (typically read from `module.toml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub min_core_version: String,
}

impl ModuleManifest {
    /// Validate the manifest fields.
    ///
    /// Currently checks that `name` is non-empty and matches the
    /// lowercase-with-hyphens convention (e.g. `"mod-rca"`).
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.name.is_empty() {
            return Err(ManifestError::EmptyName);
        }
        if self.version.is_empty() {
            return Err(ManifestError::EmptyVersion);
        }
        let valid = self
            .name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '-');
        if !valid {
            return Err(ManifestError::InvalidName(self.name.clone()));
        }
        Ok(())
    }
}
