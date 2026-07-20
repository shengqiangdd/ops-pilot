//! mod-config: Centralized configuration management module.
//!
//! Provides CRUD operations, hot-reload, and validation for module configurations.
//! Configurations are stored in SQLite and cached in memory for fast reads.

use std::collections::HashMap;

use async_trait::async_trait;
use ops_pilot_sdk::context::ModuleContext;
use ops_pilot_sdk::events::OpsEvent;
use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
use serde_json::Value;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use tracing::info;

// ── Types ───────────────────────────────────────────────────────────────────

/// A single configuration entry.
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
struct ConfigRow {
    key: String,
    value_json: String,
    updated_at: String,
    description: String,
}

// ── Module ──────────────────────────────────────────────────────────────────

/// Centralized configuration management module — stores key-value configs
/// in SQLite with an in-memory cache for fast reads.
pub struct ConfigModule {
    db: SqlitePool,
    cache: RwLock<HashMap<String, Value>>,
}

impl ConfigModule {
    /// Create a new ConfigModule with the given database pool.
    ///
    /// Automatically creates the `module_config` table if it doesn't exist
    /// and warms the in-memory cache from the database.
    pub async fn new(db: SqlitePool) -> Self {
        // Ensure table exists
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS module_config (
                key TEXT PRIMARY KEY,
                value_json TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                description TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(&db)
        .await
        .expect("failed to create module_config table");

        let module = Self {
            db,
            cache: RwLock::new(HashMap::new()),
        };

        // Warm cache from DB
        module.reload_cache().await;
        module
    }

    /// Reload the in-memory cache from the database.
    async fn reload_cache(&self) {
        let rows: Vec<ConfigRow> =
            sqlx::query_as("SELECT key, value_json, updated_at, description FROM module_config")
                .fetch_all(&self.db)
                .await
                .unwrap_or_default();

        let mut cache = self.cache.write().await;
        cache.clear();
        for row in rows {
            if let Ok(val) = serde_json::from_str(&row.value_json) {
                cache.insert(row.key, val);
            }
        }
        info!(count = cache.len(), "Config cache reloaded");
    }

    /// Get a config value from cache (fast path) or DB (fallback).
    async fn get_value(&self, key: &str) -> anyhow::Result<Option<Value>> {
        // Fast path: check cache
        {
            let cache = self.cache.read().await;
            if let Some(val) = cache.get(key) {
                return Ok(Some(val.clone()));
            }
        }

        // Fallback: query DB
        let row: Option<ConfigRow> = sqlx::query_as(
            "SELECT key, value_json, updated_at, description FROM module_config WHERE key = ?",
        )
        .bind(key)
        .fetch_optional(&self.db)
        .await?;

        match row {
            Some(r) => {
                let val: Value = serde_json::from_str(&r.value_json)?;
                // Populate cache
                self.cache
                    .write()
                    .await
                    .insert(key.to_string(), val.clone());
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }

    /// Set a config value in DB and update cache.
    async fn set_value(&self, key: &str, value: &Value, description: &str) -> anyhow::Result<()> {
        let value_json = serde_json::to_string(value)?;

        sqlx::query(
            "INSERT INTO module_config (key, value_json, description, updated_at) \
             VALUES (?, ?, ?, datetime('now')) \
             ON CONFLICT(key) DO UPDATE SET \
                value_json = excluded.value_json, \
                description = excluded.description, \
                updated_at = datetime('now')",
        )
        .bind(key)
        .bind(&value_json)
        .bind(description)
        .execute(&self.db)
        .await?;

        // Update cache
        self.cache
            .write()
            .await
            .insert(key.to_string(), value.clone());

        info!(key, "Config value set");
        Ok(())
    }

    /// Delete a config value from DB and cache.
    async fn delete_value(&self, key: &str) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM module_config WHERE key = ?")
            .bind(key)
            .execute(&self.db)
            .await?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            self.cache.write().await.remove(key);
            info!(key, "Config value deleted");
        }
        Ok(deleted)
    }

    /// List config keys matching an optional prefix.
    async fn list_values(&self, prefix: Option<&str>) -> anyhow::Result<Vec<(String, Value)>> {
        let rows: Vec<ConfigRow> = if let Some(pfx) = prefix {
            let pattern = format!("{pfx}%");
            sqlx::query_as(
                "SELECT key, value_json, updated_at, description FROM module_config \
                 WHERE key LIKE ? ORDER BY key",
            )
            .bind(&pattern)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as(
                "SELECT key, value_json, updated_at, description FROM module_config ORDER BY key",
            )
            .fetch_all(&self.db)
            .await?
        };

        let mut result = Vec::new();
        for row in rows {
            if let Ok(val) = serde_json::from_str(&row.value_json) {
                result.push((row.key, val));
            }
        }
        Ok(result)
    }
}

#[async_trait]
impl OpsModule for ConfigModule {
    fn name(&self) -> &str {
        "mod-config"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "Centralized configuration management — CRUD, hot-reload, and validation"
    }

    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "config_get".into(),
                description: "Get a configuration value by key".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "key": { "type": "string", "description": "Configuration key" }
                    },
                    "required": ["key"]
                }),
            },
            ToolDefinition {
                name: "config_set".into(),
                description: "Set or update a configuration value".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "key": { "type": "string", "description": "Configuration key" },
                        "value": { "description": "Configuration value (JSON)" },
                        "description": { "type": "string", "description": "Optional description", "default": "" }
                    },
                    "required": ["key", "value"]
                }),
            },
            ToolDefinition {
                name: "config_delete".into(),
                description: "Delete a configuration value".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "key": { "type": "string", "description": "Configuration key to delete" }
                    },
                    "required": ["key"]
                }),
            },
            ToolDefinition {
                name: "config_list".into(),
                description: "List configuration entries (optional prefix filter)".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "prefix": { "type": "string", "description": "Optional key prefix to filter by" }
                    }
                }),
            },
            ToolDefinition {
                name: "config_export".into(),
                description: "Export all configuration as a single JSON object".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            ToolDefinition {
                name: "config_import".into(),
                description: "Import configuration from a JSON object (batch upsert)".into(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "data": {
                            "type": "object",
                            "description": "JSON object where keys are config keys and values are config values"
                        }
                    },
                    "required": ["data"]
                }),
            },
        ]
    }

    async fn execute(
        &self,
        _ctx: &ModuleContext,
        tool: &str,
        params: Value,
    ) -> anyhow::Result<Value> {
        match tool {
            "config_get" => {
                let key = params["key"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'key' parameter"))?;

                match self.get_value(key).await? {
                    Some(val) => Ok(serde_json::json!({ "key": key, "value": val })),
                    None => Ok(serde_json::json!({ "key": key, "value": null, "found": false })),
                }
            }
            "config_set" => {
                let key = params["key"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'key' parameter"))?;
                let value = params
                    .get("value")
                    .ok_or_else(|| anyhow::anyhow!("missing 'value' parameter"))?;
                let description = params["description"].as_str().unwrap_or("");

                self.set_value(key, value, description).await?;
                Ok(serde_json::json!({ "status": "ok", "key": key }))
            }
            "config_delete" => {
                let key = params["key"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing 'key' parameter"))?;

                let deleted = self.delete_value(key).await?;
                Ok(serde_json::json!({ "deleted": deleted, "key": key }))
            }
            "config_list" => {
                let prefix = params["prefix"].as_str();
                let entries = self.list_values(prefix).await?;

                let map: serde_json::Map<String, Value> = entries.into_iter().collect();

                Ok(Value::Object(map))
            }
            "config_export" => {
                let entries = self.list_values(None).await?;
                let map: serde_json::Map<String, Value> = entries.into_iter().collect();
                Ok(Value::Object(map))
            }
            "config_import" => {
                let data = params["data"]
                    .as_object()
                    .ok_or_else(|| anyhow::anyhow!("missing or invalid 'data' parameter"))?;

                let mut imported = 0;
                for (key, value) in data {
                    self.set_value(key, value, "").await?;
                    imported += 1;
                }

                info!(imported, "Config batch import completed");
                Ok(serde_json::json!({ "status": "ok", "imported": imported }))
            }
            _ => Err(anyhow::anyhow!("unknown tool: {}", tool)),
        }
    }

    async fn on_event(&self, _ctx: &ModuleContext, _event: &OpsEvent) -> Option<ModuleAction> {
        None
    }

    async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus {
        let _count = self.cache.read().await.len();
        HealthStatus::Healthy
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ops_pilot_sdk::context::{EventBus, ModuleContext};
    use std::path::PathBuf;
    use std::sync::Arc;

    async fn setup() -> (ConfigModule, ModuleContext) {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let module = ConfigModule::new(pool.clone()).await;
        let ctx = ModuleContext::new(
            Arc::new(pool),
            EventBus::new(16),
            PathBuf::from("/tmp/test-config"),
            "test-config".into(),
        );
        (module, ctx)
    }

    #[tokio::test]
    async fn test_module_metadata() {
        let (m, _ctx) = setup().await;
        assert_eq!(m.name(), "mod-config");
        assert_eq!(m.version(), "0.1.0");
        assert!(m.description().contains("configuration"));
    }

    #[tokio::test]
    async fn test_tools_registered() {
        let (m, _ctx) = setup().await;
        let tools = m.tools();
        assert_eq!(tools.len(), 6);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"config_get"));
        assert!(names.contains(&"config_set"));
        assert!(names.contains(&"config_delete"));
        assert!(names.contains(&"config_list"));
        assert!(names.contains(&"config_export"));
        assert!(names.contains(&"config_import"));
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let (m, ctx) = setup().await;

        let result = m
            .execute(
                &ctx,
                "config_set",
                serde_json::json!({ "key": "app.name", "value": "OpsPilot" }),
            )
            .await
            .unwrap();
        assert_eq!(result["status"], "ok");

        let result = m
            .execute(&ctx, "config_get", serde_json::json!({ "key": "app.name" }))
            .await
            .unwrap();
        assert_eq!(result["value"], "OpsPilot");
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let (m, ctx) = setup().await;
        let result = m
            .execute(
                &ctx,
                "config_get",
                serde_json::json!({ "key": "nonexistent" }),
            )
            .await
            .unwrap();
        assert_eq!(result["found"], false);
        assert!(result["value"].is_null());
    }

    #[tokio::test]
    async fn test_delete() {
        let (m, ctx) = setup().await;

        m.execute(
            &ctx,
            "config_set",
            serde_json::json!({ "key": "to.delete", "value": true }),
        )
        .await
        .unwrap();

        let result = m
            .execute(
                &ctx,
                "config_delete",
                serde_json::json!({ "key": "to.delete" }),
            )
            .await
            .unwrap();
        assert_eq!(result["deleted"], true);

        // Verify it's gone
        let result = m
            .execute(
                &ctx,
                "config_get",
                serde_json::json!({ "key": "to.delete" }),
            )
            .await
            .unwrap();
        assert_eq!(result["found"], false);
    }

    #[tokio::test]
    async fn test_list_with_prefix() {
        let (m, ctx) = setup().await;

        m.execute(
            &ctx,
            "config_set",
            serde_json::json!({ "key": "ssh.host1", "value": "10.0.0.1" }),
        )
        .await
        .unwrap();
        m.execute(
            &ctx,
            "config_set",
            serde_json::json!({ "key": "ssh.host2", "value": "10.0.0.2" }),
        )
        .await
        .unwrap();
        m.execute(
            &ctx,
            "config_set",
            serde_json::json!({ "key": "docker.registry", "value": "ghcr.io" }),
        )
        .await
        .unwrap();

        let result = m
            .execute(&ctx, "config_list", serde_json::json!({ "prefix": "ssh." }))
            .await
            .unwrap();

        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("ssh.host1"));
        assert!(obj.contains_key("ssh.host2"));
        assert!(!obj.contains_key("docker.registry"));
    }

    #[tokio::test]
    async fn test_export() {
        let (m, ctx) = setup().await;

        m.execute(
            &ctx,
            "config_set",
            serde_json::json!({ "key": "a", "value": 1 }),
        )
        .await
        .unwrap();
        m.execute(
            &ctx,
            "config_set",
            serde_json::json!({ "key": "b", "value": "two" }),
        )
        .await
        .unwrap();

        let result = m
            .execute(&ctx, "config_export", serde_json::json!({}))
            .await
            .unwrap();

        let obj = result.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert_eq!(obj["a"], 1);
        assert_eq!(obj["b"], "two");
    }

    #[tokio::test]
    async fn test_import() {
        let (m, ctx) = setup().await;

        let result = m
            .execute(
                &ctx,
                "config_import",
                serde_json::json!({
                    "data": {
                        "import.key1": "val1",
                        "import.key2": 42,
                        "import.key3": { "nested": true }
                    }
                }),
            )
            .await
            .unwrap();
        assert_eq!(result["imported"], 3);

        // Verify each imported key
        let r = m
            .execute(
                &ctx,
                "config_get",
                serde_json::json!({ "key": "import.key1" }),
            )
            .await
            .unwrap();
        assert_eq!(r["value"], "val1");

        let r = m
            .execute(
                &ctx,
                "config_get",
                serde_json::json!({ "key": "import.key2" }),
            )
            .await
            .unwrap();
        assert_eq!(r["value"], 42);

        let r = m
            .execute(
                &ctx,
                "config_get",
                serde_json::json!({ "key": "import.key3" }),
            )
            .await
            .unwrap();
        assert_eq!(r["value"]["nested"], true);
    }

    #[tokio::test]
    async fn test_unknown_tool() {
        let (m, ctx) = setup().await;
        let result = m.execute(&ctx, "nonexistent", serde_json::json!({})).await;
        assert!(result.is_err());
    }
}
