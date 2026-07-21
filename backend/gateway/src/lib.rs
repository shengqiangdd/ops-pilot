//! # ops-pilot-gateway
//!
//! HTTP 网关层 —— 基于 Axum 的 REST API 服务器和 WebSocket 终端。
//!
//! 本 crate 负责：
//!
//! - **路由** ([`routes`]) — 主机管理、模块控制、Agent 对话、Vault 操作等 API 端点
//! - **认证** ([`middleware`]) — JWT 令牌验证中间件和请求提取器
//! - **Agent** ([`agent`]) — ReAct 循环编排、会话管理、工具调用分发
//! - **LLM** ([`llm`]) — LLM 提供商配置、HTTP 客户端、流式响应处理
//! - **终端** ([`terminal`]) — SSH 交互式终端的 WebSocket 代理
//! - **工具注册** ([`tools`]) — 模块工具聚合、索引缓存、OpenAI schema 格式化
//!
//! 网关本身不包含业务逻辑 —— 所有计算委托给 `ops-pilot-core` 和 `ops-pilot-mod-*` 模块。

pub mod agent;
pub mod alert_advisor;
pub mod alert_suppression;
pub mod anomaly_detector;
pub mod audit_log;
pub mod backup;
pub mod cluster_manager;
pub mod dashboard_store;
pub mod docs;
pub mod gitops;
pub mod llm;
pub mod metrics;
pub mod middleware;
pub mod notify;
pub mod oauth2;
pub mod otel_integration;
pub mod rca_engine;
pub mod report_generator;
pub mod retry_queue;
pub mod routes;
pub mod security_headers;
pub mod seed;
pub mod session_recorder;
pub mod terminal;
pub mod tools;
pub mod ws_events;

use std::sync::Arc;
use axum::Router;
use sqlx::SqlitePool;

/// Shared application state — used by route handlers and test helpers.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub alert_suppressor: crate::alert_suppression::AlertSuppressor,
}

/// Create a minimum test router with health, docs, CORS, and security headers.
///
/// This is the same pattern used by E2E integration tests so they exercise
/// real middleware (CORS, security headers, tracing) without starting the
/// full module-heavy server.
pub async fn create_router(_state: Arc<AppState>) -> Router {
    use axum::routing::get;
    use axum::response::Json;
    use axum::http::StatusCode;
    use tower_http::cors::CorsLayer;
    use tower_http::trace::TraceLayer;

    let app = Router::new()
        .route("/api/v1/health", get(|| async {
            Json(serde_json::json!({"status": "ok"}))
        }))
        // API docs routes
        .route("/api/docs/openapi.json", get(crate::docs::openapi_json))
        .route("/api/docs/swagger-ui", get(crate::docs::swagger_ui))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // Apply security headers (CSP, HSTS, X-Frame-Options, etc.)
    let app = crate::security_headers::security_header_layers()
        .into_iter()
        .fold(app, |router, layer| router.layer(layer));

    // Return JSON for 404s (instead of the default HTML fallback)
    app.fallback(|| async {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "not found"})))
    })
}
