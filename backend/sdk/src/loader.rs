use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{bail, Result};

use crate::context::ModuleContext;
use crate::traits::{HealthStatus, OpsModule};

/// Errors specific to module loading.
#[derive(Debug, thiserror::Error)]
pub enum LoaderError {
    #[error("module \"{0}\" is already loaded")]
    AlreadyLoaded(String),
    #[error("missing dependency \"{dep}\" required by module \"{module}\"")]
    MissingDependency { module: String, dep: String },
}

/// Manages the lifecycle of loaded modules.
///
/// Modules are stored behind `Arc` so they can be shared safely across
/// async tasks.
pub struct ModuleLoader {
    modules: HashMap<String, Arc<dyn OpsModule>>,
}

impl ModuleLoader {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    /// Register a module after verifying its dependencies are already loaded.
    ///
    /// # Errors
    /// * [`LoaderError::AlreadyLoaded`] if a module with the same name exists.
    /// * [`LoaderError::MissingDependency`] if any declared dependency is absent.
    pub async fn load_module(
        &mut self,
        _ctx: ModuleContext,
        module: Box<dyn OpsModule>,
    ) -> Result<()> {
        let name = module.name().to_string();

        if self.modules.contains_key(&name) {
            bail!(LoaderError::AlreadyLoaded(name));
        }

        for dep in module.dependencies() {
            if !self.modules.contains_key(dep) {
                bail!(LoaderError::MissingDependency {
                    module: name,
                    dep: dep.to_string(),
                });
            }
        }

        self.modules.insert(name, Arc::from(module));
        Ok(())
    }

    /// Look up a loaded module by name.
    pub fn get_module(&self, name: &str) -> Option<Arc<dyn OpsModule>> {
        self.modules.get(name).cloned()
    }

    /// List all loaded modules as `(name, version)` pairs.
    pub fn list_modules(&self) -> Vec<(&str, &str)> {
        self.modules
            .iter()
            .map(|(name, m)| (name.as_str(), m.version()))
            .collect()
    }

    /// Run health checks on every loaded module.
    pub async fn health_check_all(&self, ctx: &ModuleContext) -> Vec<(String, HealthStatus)> {
        let mut results = Vec::with_capacity(self.modules.len());
        for (name, module) in &self.modules {
            let status = module.health_check(ctx).await;
            results.push((name.clone(), status));
        }
        results
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::EventBus;
    use crate::events::OpsEvent;
    use crate::traits::{ModuleAction, ModuleManifest, ToolDefinition};

    use async_trait::async_trait;
    use serde_json::Value;
    use sqlx::SqlitePool;

    // ── helpers ──────────────────────────────────────────────────────────

    async fn make_ctx(name: &str) -> ModuleContext {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        ModuleContext::new(
            Arc::new(pool),
            EventBus::new(16),
            std::path::PathBuf::from(format!("/tmp/{name}")),
            name.to_string(),
        )
    }

    struct StubModule {
        name: String,
        deps: Vec<String>,
    }

    impl StubModule {
        fn new(name: &str, deps: Vec<&str>) -> Self {
            Self {
                name: name.to_string(),
                deps: deps.into_iter().map(String::from).collect(),
            }
        }
    }

    #[async_trait]
    impl OpsModule for StubModule {
        fn name(&self) -> &str {
            &self.name
        }
        fn version(&self) -> &str {
            "0.1.0"
        }
        fn description(&self) -> &str {
            "test stub"
        }
        fn dependencies(&self) -> Vec<&str> {
            self.deps.iter().map(|s| s.as_str()).collect()
        }
        fn tools(&self) -> Vec<ToolDefinition> {
            vec![]
        }
        async fn execute(
            &self,
            _ctx: &ModuleContext,
            _tool: &str,
            _p: Value,
        ) -> anyhow::Result<Value> {
            Ok(Value::Null)
        }
        async fn on_event(&self, _ctx: &ModuleContext, _e: &OpsEvent) -> Option<ModuleAction> {
            None
        }
        async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus {
            HealthStatus::Healthy
        }
    }

    // ── ModuleContext tests ──────────────────────────────────────────────

    #[tokio::test]
    async fn context_creation() {
        let ctx = make_ctx("test-mod").await;
        assert_eq!(ctx.module_name(), "test-mod");
        assert_eq!(ctx.config_dir().to_string_lossy(), "/tmp/test-mod");
    }

    #[tokio::test]
    async fn context_emit_does_not_panic() {
        let ctx = make_ctx("emitter").await;
        // No subscribers — emit should silently swallow the error.
        ctx.emit(OpsEvent::Custom {
            source: "test".into(),
            kind: "ping".into(),
            payload: serde_json::json!({}),
        })
        .await;
    }

    // ── ModuleManifest tests ────────────────────────────────────────────

    #[test]
    fn manifest_valid_name() {
        let m = ModuleManifest {
            name: "mod-rca".into(),
            version: "0.1.0".into(),
            description: "test".into(),
            authors: vec![],
            dependencies: vec![],
            min_core_version: String::new(),
        };
        assert!(m.validate().is_ok());
    }

    #[test]
    fn manifest_invalid_name_uppercase() {
        let m = ModuleManifest {
            name: "ModRCA".into(),
            version: "0.1.0".into(),
            description: "test".into(),
            authors: vec![],
            dependencies: vec![],
            min_core_version: String::new(),
        };
        assert!(m.validate().is_err());
    }

    #[test]
    fn manifest_invalid_name_underscore() {
        let m = ModuleManifest {
            name: "mod_rca".into(),
            version: "0.1.0".into(),
            description: "test".into(),
            authors: vec![],
            dependencies: vec![],
            min_core_version: String::new(),
        };
        assert!(m.validate().is_err());
    }

    #[test]
    fn manifest_empty_name() {
        let m = ModuleManifest {
            name: String::new(),
            version: "0.1.0".into(),
            description: "test".into(),
            authors: vec![],
            dependencies: vec![],
            min_core_version: String::new(),
        };
        assert!(m.validate().is_err());
    }

    #[test]
    fn manifest_empty_version() {
        let m = ModuleManifest {
            name: "mod-ok".into(),
            version: String::new(),
            description: "test".into(),
            authors: vec![],
            dependencies: vec![],
            min_core_version: String::new(),
        };
        assert!(m.validate().is_err());
    }

    // ── ModuleLoader tests ──────────────────────────────────────────────

    #[tokio::test]
    async fn loader_register_and_get() {
        let mut loader = ModuleLoader::new();
        let ctx = make_ctx("alpha").await;
        let module = Box::new(StubModule::new("alpha", vec![]));

        loader.load_module(ctx, module).await.unwrap();

        let got = loader.get_module("alpha").expect("module should exist");
        assert_eq!(got.name(), "alpha");
        assert_eq!(got.version(), "0.1.0");
    }

    #[tokio::test]
    async fn loader_list_modules() {
        let mut loader = ModuleLoader::new();
        loader
            .load_module(make_ctx("a").await, Box::new(StubModule::new("a", vec![])))
            .await
            .unwrap();
        loader
            .load_module(make_ctx("b").await, Box::new(StubModule::new("b", vec![])))
            .await
            .unwrap();

        let mut list = loader.list_modules();
        list.sort_by_key(|(n, _)| n.to_string());
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], ("a", "0.1.0"));
        assert_eq!(list[1], ("b", "0.1.0"));
    }

    #[tokio::test]
    async fn loader_duplicate_rejected() {
        let mut loader = ModuleLoader::new();
        loader
            .load_module(make_ctx("x").await, Box::new(StubModule::new("x", vec![])))
            .await
            .unwrap();

        let err = loader
            .load_module(make_ctx("x").await, Box::new(StubModule::new("x", vec![])))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("already loaded"));
    }

    #[tokio::test]
    async fn loader_missing_dependency_fails() {
        let mut loader = ModuleLoader::new();
        let err = loader
            .load_module(
                make_ctx("child").await,
                Box::new(StubModule::new("child", vec!["nonexistent"])),
            )
            .await
            .unwrap_err();
        assert!(err.to_string().contains("missing dependency"));
    }

    #[tokio::test]
    async fn loader_dependency_resolution() {
        let mut loader = ModuleLoader::new();
        // Load parent first, then child that depends on it.
        loader
            .load_module(
                make_ctx("parent").await,
                Box::new(StubModule::new("parent", vec![])),
            )
            .await
            .unwrap();
        loader
            .load_module(
                make_ctx("child").await,
                Box::new(StubModule::new("child", vec!["parent"])),
            )
            .await
            .unwrap();
        assert!(loader.get_module("child").is_some());
    }

    #[tokio::test]
    async fn loader_health_check_all() {
        let mut loader = ModuleLoader::new();
        loader
            .load_module(
                make_ctx("h1").await,
                Box::new(StubModule::new("h1", vec![])),
            )
            .await
            .unwrap();
        loader
            .load_module(
                make_ctx("h2").await,
                Box::new(StubModule::new("h2", vec![])),
            )
            .await
            .unwrap();

        let ctx = make_ctx("health-checker").await;
        let results = loader.health_check_all(&ctx).await;
        assert_eq!(results.len(), 2);
        for (_name, status) in &results {
            assert!(matches!(status, HealthStatus::Healthy));
        }
    }
}
