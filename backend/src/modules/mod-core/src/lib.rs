//! mod-core: Wraps core infrastructure (SSH, Docker, Host, Monitor)
//! as an OpsModule, making them manageable via the ModuleLoader.

pub mod docker;
pub mod host;
pub mod monitor;
pub mod ssh;

use async_trait::async_trait;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};

/// The core module aggregates all infrastructure sub-modules.
pub struct ModCore {
    sub_modules: Vec<Box<dyn OpsModule>>,
}

impl Default for ModCore {
    fn default() -> Self {
        Self::new()
    }
}

impl ModCore {
    pub fn new() -> Self {
        Self {
            sub_modules: vec![
                Box::new(ssh::SshModule::new()),
                Box::new(docker::DockerModule::new()),
                Box::new(host::HostModule::new()),
                Box::new(monitor::MonitorModule::new()),
            ],
        }
    }
}

#[async_trait]
impl OpsModule for ModCore {
    fn name(&self) -> &str {
        "mod-core"
    }
    fn version(&self) -> &str {
        "0.1.0"
    }
    fn description(&self) -> &str {
        "Core infrastructure module: SSH, Docker, host management, monitoring"
    }
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        self.sub_modules.iter().flat_map(|m| m.tools()).collect()
    }

    async fn execute(
        &self,
        ctx: &ModuleContext,
        tool: &str,
        params: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        for m in &self.sub_modules {
            if m.tools().iter().any(|t| t.name == tool) {
                return m.execute(ctx, tool, params).await;
            }
        }
        anyhow::bail!("tool '{}' not found in mod-core", tool)
    }

    async fn on_event(&self, ctx: &ModuleContext, event: &OpsEvent) -> Option<ModuleAction> {
        for m in &self.sub_modules {
            if let Some(action) = m.on_event(ctx, event).await {
                return Some(action);
            }
        }
        None
    }

    async fn health_check(&self, ctx: &ModuleContext) -> HealthStatus {
        for m in &self.sub_modules {
            match m.health_check(ctx).await {
                HealthStatus::Unhealthy { reason } => {
                    return HealthStatus::Unhealthy {
                        reason: format!("{}: {}", m.name(), reason),
                    }
                }
                HealthStatus::Degraded { reason } => {
                    return HealthStatus::Degraded {
                        reason: format!("{}: {}", m.name(), reason),
                    }
                }
                _ => {}
            }
        }
        HealthStatus::Healthy
    }
}
