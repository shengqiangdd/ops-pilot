//! REST handlers for module management (Task 2.7).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use ops_pilot_sdk::loader::ModuleLoader;
use ops_pilot_sdk::traits::HealthStatus;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Tracks which modules are enabled/disabled.
pub struct ModuleManager {
    loader: ModuleLoader,
    enabled: HashMap<String, bool>,
}

impl ModuleManager {
    pub fn new(loader: ModuleLoader) -> Self {
        Self {
            loader,
            enabled: HashMap::new(),
        }
    }

    /// List all loaded modules with their metadata and enabled status.
    pub fn list_modules(&self) -> Vec<ModuleInfo> {
        self.loader
            .list_modules()
            .into_iter()
            .map(|(name, version)| {
                let enabled = self.enabled.get(name).copied().unwrap_or(true);
                let module = self.loader.get_module(name).unwrap();
                ModuleInfo {
                    name: name.to_string(),
                    version: version.to_string(),
                    description: module.description().to_string(),
                    enabled,
                }
            })
            .collect()
    }

    /// Get details for a specific module.
    pub fn get_module(&self, name: &str) -> Option<ModuleInfo> {
        let module = self.loader.get_module(name)?;
        let enabled = self.enabled.get(name).copied().unwrap_or(true);
        Some(ModuleInfo {
            name: name.to_string(),
            version: module.version().to_string(),
            description: module.description().to_string(),
            enabled,
        })
    }

    /// Enable a module. Returns false if the module is not loaded.
    pub fn enable(&mut self, name: &str) -> bool {
        if self.loader.get_module(name).is_none() {
            return false;
        }
        self.enabled.insert(name.to_string(), true);
        true
    }

    /// Disable a module. Returns false if the module is not loaded.
    pub fn disable(&mut self, name: &str) -> bool {
        if self.loader.get_module(name).is_none() {
            return false;
        }
        self.enabled.insert(name.to_string(), false);
        true
    }

    /// Check if a module is enabled.
    pub fn is_enabled(&self, name: &str) -> bool {
        self.enabled.get(name).copied().unwrap_or(true)
    }

    /// Run health check on a specific module.
    pub async fn health_check(
        &self,
        name: &str,
        ctx: &ops_pilot_sdk::context::ModuleContext,
    ) -> Option<HealthStatus> {
        let module = self.loader.get_module(name)?;
        Some(module.health_check(ctx).await)
    }

    /// Run health checks on all loaded modules.
    pub async fn health_check_all(
        &self,
        ctx: &ops_pilot_sdk::context::ModuleContext,
    ) -> Vec<ModuleHealth> {
        let results = self.loader.health_check_all(ctx).await;
        results
            .into_iter()
            .map(|(name, status)| {
                let enabled = self.enabled.get(&name).copied().unwrap_or(true);
                ModuleHealth {
                    name,
                    status,
                    enabled,
                }
            })
            .collect()
    }

    /// Get the underlying module reference by name.
    pub fn get_module_ref(&self, name: &str) -> Option<Arc<dyn ops_pilot_sdk::traits::OpsModule>> {
        self.loader.get_module(name)
    }

    /// Access the inner loader (for registration).
    pub fn loader_mut(&mut self) -> &mut ModuleLoader {
        &mut self.loader
    }
}

/// Module info returned by list/detail endpoints.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct ModuleInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
}

/// Health status for a single module.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct ModuleHealth {
    pub name: String,
    pub status: HealthStatus,
    pub enabled: bool,
}

/// Shared state for module routes.
#[derive(Clone)]
pub struct ModuleState {
    pub manager: Arc<RwLock<ModuleManager>>,
    pub ctx: Arc<ops_pilot_sdk::context::ModuleContext>,
}

// ── Route handlers ──────────────────────────────────────────────────────────

/// GET /api/modules — list all loaded modules.
async fn list_modules(State(state): State<ModuleState>) -> impl IntoResponse {
    let manager = state.manager.read().await;
    let modules = manager.list_modules();
    (StatusCode::OK, Json(modules)).into_response()
}

/// GET /api/modules/:name — get module details.
async fn get_module(
    State(state): State<ModuleState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let manager = state.manager.read().await;
    match manager.get_module(&name) {
        Some(info) => (StatusCode::OK, Json(info)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("module '{}' not found", name)})),
        )
            .into_response(),
    }
}

/// POST /api/modules/:name/enable — enable a module.
async fn enable_module(
    State(state): State<ModuleState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let mut manager = state.manager.write().await;
    if manager.enable(&name) {
        (StatusCode::OK, Json(serde_json::json!({"enabled": true}))).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("module '{}' not found", name)})),
        )
            .into_response()
    }
}

/// POST /api/modules/:name/disable — disable a module.
async fn disable_module(
    State(state): State<ModuleState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let mut manager = state.manager.write().await;
    if manager.disable(&name) {
        (StatusCode::OK, Json(serde_json::json!({"enabled": false}))).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("module '{}' not found", name)})),
        )
            .into_response()
    }
}

/// GET /api/modules/:name/health — health check a specific module.
async fn module_health(
    State(state): State<ModuleState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let manager = state.manager.read().await;
    match manager.health_check(&name, &state.ctx).await {
        Some(status) => (StatusCode::OK, Json(status)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("module '{}' not found", name)})),
        )
            .into_response(),
    }
}

/// GET /api/health — aggregate health of all modules.
async fn health_all(State(state): State<ModuleState>) -> impl IntoResponse {
    let manager = state.manager.read().await;
    let results = manager.health_check_all(&state.ctx).await;
    (StatusCode::OK, Json(results)).into_response()
}

/// Build the module routes sub-router.
pub fn module_routes(manager: Arc<RwLock<ModuleManager>>, ctx: Arc<ops_pilot_sdk::context::ModuleContext>) -> Router {
    let state = ModuleState { manager, ctx };

    Router::new()
        .route("/api/modules", get(list_modules))
        .route("/api/modules/{name}", get(get_module))
        .route("/api/modules/{name}/enable", post(enable_module))
        .route("/api/modules/{name}/disable", post(disable_module))
        .route("/api/modules/{name}/health", get(module_health))
        .route("/api/health", get(health_all))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use ops_pilot_sdk::context::{EventBus, ModuleContext};
    use ops_pilot_sdk::events::OpsEvent;
    use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
    use serde_json::Value;
    use sqlx::SqlitePool;
    use tower::ServiceExt;

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
    }

    impl StubModule {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
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
            vec![]
        }
        fn tools(&self) -> Vec<ToolDefinition> {
            vec![]
        }
        async fn execute(&self, _ctx: &ModuleContext, _tool: &str, _p: Value) -> anyhow::Result<Value> {
            Ok(Value::Null)
        }
        async fn on_event(&self, _ctx: &ModuleContext, _e: &OpsEvent) -> Option<ModuleAction> {
            None
        }
        async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus {
            HealthStatus::Healthy
        }
    }

    async fn make_manager(names: &[&str]) -> (Arc<RwLock<ModuleManager>>, Arc<ModuleContext>) {
        let mut loader = ModuleLoader::new();
        let ctx = make_ctx("test").await;
        for name in names {
            loader
                .load_module(make_ctx(name).await, Box::new(StubModule::new(name)))
                .await
                .unwrap();
        }
        let manager = Arc::new(RwLock::new(ModuleManager::new(loader)));
        (manager, Arc::new(ctx))
    }

    async fn test_app(names: &[&str]) -> axum::Router {
        let (manager, ctx) = make_manager(names).await;
        module_routes(manager, ctx)
    }

    #[tokio::test]
    async fn test_list_modules_returns_all() {
        let app = test_app(&["alpha", "beta"]).await;
        let req = axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri("/api/modules")
            .body(axum::body::Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let modules: Vec<ModuleInfo> = serde_json::from_slice(&body).unwrap();
        assert_eq!(modules.len(), 2);

        let mut names: Vec<&str> = modules.iter().map(|m| m.name.as_str()).collect();
        names.sort();
        assert_eq!(names, vec!["alpha", "beta"]);

        // All should be enabled by default
        for m in &modules {
            assert!(m.enabled);
            assert_eq!(m.version, "0.1.0");
            assert_eq!(m.description, "test stub");
        }
    }

    #[tokio::test]
    async fn test_enable_disable_toggle() {
        let (manager, _ctx) = make_manager(&["mymod"]).await;

        // Initially enabled
        {
            let mgr = manager.read().await;
            assert!(mgr.is_enabled("mymod"));
        }

        // Disable
        {
            let mut mgr = manager.write().await;
            assert!(mgr.disable("mymod"));
            assert!(!mgr.is_enabled("mymod"));
        }

        // Re-enable
        {
            let mut mgr = manager.write().await;
            assert!(mgr.enable("mymod"));
            assert!(mgr.is_enabled("mymod"));
        }

        // Non-existent module returns false
        {
            let mut mgr = manager.write().await;
            assert!(!mgr.disable("nonexistent"));
            assert!(!mgr.enable("nonexistent"));
        }
    }

    #[tokio::test]
    async fn test_health_check_specific_module() {
        let app = test_app(&["healthy-mod"]).await;
        let req = axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri("/api/modules/healthy-mod/health")
            .body(axum::body::Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let status: HealthStatus = serde_json::from_slice(&body).unwrap();
        assert!(matches!(status, HealthStatus::Healthy));
    }

    #[tokio::test]
    async fn test_health_check_specific_not_found() {
        let app = test_app(&["exists"]).await;
        let req = axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri("/api/modules/ghost/health")
            .body(axum::body::Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_health_check_all_returns_statuses() {
        let app = test_app(&["mod-a", "mod-b", "mod-c"]).await;
        let req = axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri("/api/health")
            .body(axum::body::Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let results: Vec<ModuleHealth> = serde_json::from_slice(&body).unwrap();
        assert_eq!(results.len(), 3);

        for mh in &results {
            assert!(mh.enabled);
            assert!(matches!(mh.status, HealthStatus::Healthy));
        }
    }

    #[tokio::test]
    async fn test_enable_disable_via_api() {
        let app = test_app(&["api-mod"]).await;

        // Disable via API
        let req = axum::http::Request::builder()
            .method(axum::http::Method::POST)
            .uri("/api/modules/api-mod/disable")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // List should show disabled
        let req = axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri("/api/modules")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let modules: Vec<ModuleInfo> = serde_json::from_slice(&body).unwrap();
        assert!(!modules[0].enabled);

        // Re-enable
        let req = axum::http::Request::builder()
            .method(axum::http::Method::POST)
            .uri("/api/modules/api-mod/enable")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_module_not_found() {
        let app = test_app(&["real"]).await;
        let req = axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri("/api/modules/fake")
            .body(axum::body::Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
