use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use serde_json::Value;
use tokio::sync::RwLock;

use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::traits::OpsModule;

use crate::routes::modules::ModuleManager;

/// Central tool registry that aggregates tool definitions from all enabled
/// modules and routes AI function calls back to the correct module.
pub struct ToolRegistry {
    manager: Arc<RwLock<ModuleManager>>,
}

impl ToolRegistry {
    pub fn new(manager: Arc<RwLock<ModuleManager>>) -> Self {
        Self { manager }
    }

    /// Returns OpenAI-compatible function schemas for all enabled modules' tools.
    ///
    /// Each entry has the shape:
    /// ```json
    /// { "type": "function", "function": { "name": "...", "description": "...", "parameters": {...} } }
    /// ```
    pub async fn get_tools_for_llm(&self) -> Vec<Value> {
        let modules = self.collect_enabled_modules().await;
        let mut tools = Vec::new();

        for (_module_name, module) in modules {
            for def in module.tools() {
                tools.push(serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": def.name,
                        "description": def.description,
                        "parameters": def.parameters,
                    }
                }));
            }
        }

        tools
    }

    /// Route a tool call to the correct module's `execute()` method.
    ///
    /// The `name` is the tool name (not the module name). The registry looks up
    /// which module owns that tool and delegates the call.
    pub async fn invoke_tool(&self, ctx: &ModuleContext, name: &str, params: Value) -> Result<Value> {
        let modules = self.collect_enabled_modules().await;

        for (_module_name, module) in &modules {
            for def in module.tools() {
                if def.name == name {
                    return module.execute(ctx, name, params).await;
                }
            }
        }

        Err(anyhow!("unknown tool: {name}"))
    }

    /// Convenience method returning the full tools array as a JSON `Value`.
    pub async fn tool_schemas_json(&self) -> Value {
        Value::Array(self.get_tools_for_llm().await)
    }

    /// Collect all enabled modules as `(module_name, Arc<dyn OpsModule>)`.
    async fn collect_enabled_modules(&self) -> Vec<(String, Arc<dyn OpsModule>)> {
        let manager = self.manager.read().await;
        let module_infos = manager.list_modules();
        let mut result = Vec::with_capacity(module_infos.len());

        for info in module_infos {
            if !info.enabled {
                continue;
            }
            if let Some(module) = manager.get_module_ref(&info.name) {
                result.push((info.name, module));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use ops_pilot_sdk::context::EventBus;
    use ops_pilot_sdk::events::OpsEvent;
    use ops_pilot_sdk::loader::ModuleLoader;
    use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, ToolDefinition};
    use serde_json::json;
    use sqlx::SqlitePool;

    // ── Test helpers ────────────────────────────────────────────────────

    async fn make_ctx(name: &str) -> ModuleContext {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        ModuleContext::new(
            Arc::new(pool),
            EventBus::new(16),
            std::path::PathBuf::from(format!("/tmp/{name}")),
            name.to_string(),
        )
    }

    struct ToolModule {
        module_name: String,
        tool_defs: Vec<ToolDefinition>,
    }

    impl ToolModule {
        fn new(module_name: &str, tool_defs: Vec<ToolDefinition>) -> Self {
            Self {
                module_name: module_name.to_string(),
                tool_defs,
            }
        }
    }

    #[async_trait]
    impl OpsModule for ToolModule {
        fn name(&self) -> &str {
            &self.module_name
        }
        fn version(&self) -> &str {
            "0.1.0"
        }
        fn description(&self) -> &str {
            "test tool module"
        }
        fn dependencies(&self) -> Vec<&str> {
            vec![]
        }
        fn tools(&self) -> Vec<ToolDefinition> {
            self.tool_defs.clone()
        }
        async fn execute(&self, _ctx: &ModuleContext, tool: &str, _params: Value) -> Result<Value> {
            Ok(json!({"executed": tool, "module": self.module_name}))
        }
        async fn on_event(&self, _ctx: &ModuleContext, _e: &OpsEvent) -> Option<ModuleAction> {
            None
        }
        async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus {
            HealthStatus::Healthy
        }
    }

    fn make_tool(name: &str, desc: &str) -> ToolDefinition {
        ToolDefinition {
            name: name.to_string(),
            description: desc.to_string(),
            parameters: json!({"type": "object", "properties": {}}),
        }
    }

    async fn build_registry(modules: Vec<(&str, Vec<ToolDefinition>, bool)>) -> ToolRegistry {
        let mut loader = ModuleLoader::new();
        for (name, tools, _enabled) in &modules {
            let ctx = make_ctx(name).await;
            loader
                .load_module(ctx, Box::new(ToolModule::new(name, tools.clone())))
                .await
                .unwrap();
        }

        let mut manager = ModuleManager::new(loader);
        for (name, _, enabled) in modules {
            if !enabled {
                manager.disable(name);
            }
        }

        ToolRegistry::new(Arc::new(RwLock::new(manager)))
    }

    // ── Tests ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_tools_aggregates_from_multiple_modules() {
        let registry = build_registry(vec![
            ("mod-a", vec![make_tool("tool_a1", "A1"), make_tool("tool_a2", "A2")], true),
            ("mod-b", vec![make_tool("tool_b1", "B1")], true),
        ])
        .await;

        let tools = registry.get_tools_for_llm().await;
        assert_eq!(tools.len(), 3);

        let names: Vec<&str> = tools
            .iter()
            .map(|t| t["function"]["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"tool_a1"));
        assert!(names.contains(&"tool_a2"));
        assert!(names.contains(&"tool_b1"));
    }

    #[tokio::test]
    async fn test_invoke_routes_to_correct_module() {
        let registry = build_registry(vec![
            (
                "mod-alpha",
                vec![make_tool("alpha_tool", "Alpha")],
                true,
            ),
            (
                "mod-beta",
                vec![make_tool("beta_tool", "Beta")],
                true,
            ),
        ])
        .await;

        let ctx = make_ctx("test").await;

        let result = registry.invoke_tool(&ctx, "alpha_tool", json!({})).await.unwrap();
        assert_eq!(result["module"], "mod-alpha");
        assert_eq!(result["executed"], "alpha_tool");

        let result = registry.invoke_tool(&ctx, "beta_tool", json!({})).await.unwrap();
        assert_eq!(result["module"], "mod-beta");
        assert_eq!(result["executed"], "beta_tool");
    }

    #[tokio::test]
    async fn test_invoke_unknown_tool_returns_error() {
        let registry = build_registry(vec![
            ("mod-x", vec![make_tool("real_tool", "Exists")], true),
        ])
        .await;

        let ctx = make_ctx("test").await;
        let err = registry
            .invoke_tool(&ctx, "nonexistent_tool", json!({}))
            .await
            .unwrap_err();

        assert!(err.to_string().contains("unknown tool: nonexistent_tool"));
    }

    #[tokio::test]
    async fn test_disabled_module_tools_not_included() {
        let registry = build_registry(vec![
            ("mod-on", vec![make_tool("on_tool", "Enabled")], true),
            ("mod-off", vec![make_tool("off_tool", "Disabled")], false),
        ])
        .await;

        let tools = registry.get_tools_for_llm().await;
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["function"]["name"], "on_tool");
    }

    #[tokio::test]
    async fn test_tool_schemas_json_returns_array() {
        let registry = build_registry(vec![
            ("mod-j", vec![make_tool("j_tool", "JSON")], true),
        ])
        .await;

        let schemas = registry.tool_schemas_json().await;
        assert!(schemas.is_array());
        assert_eq!(schemas.as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_openai_schema_format() {
        let registry = build_registry(vec![(
            "mod-fmt",
            vec![ToolDefinition {
                name: "get_weather".into(),
                description: "Get weather for a city".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "city": { "type": "string", "description": "City name" }
                    },
                    "required": ["city"]
                }),
            }],
            true,
        )])
        .await;

        let tools = registry.get_tools_for_llm().await;
        assert_eq!(tools.len(), 1);

        let tool = &tools[0];
        assert_eq!(tool["type"], "function");
        assert_eq!(tool["function"]["name"], "get_weather");
        assert_eq!(tool["function"]["description"], "Get weather for a city");
        assert_eq!(tool["function"]["parameters"]["type"], "object");
    }
}
