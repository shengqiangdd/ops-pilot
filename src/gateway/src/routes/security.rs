//! REST handlers for security scanning (delegates to mod-security).

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use ops_pilot_sdk::context::ModuleContext;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::Deserialize;
use crate::routes::modules::ModuleManager;

#[derive(Clone)]
pub struct SecurityState {
    pub manager: Arc<RwLock<ModuleManager>>,
    pub ctx: Arc<ModuleContext>,
}

#[derive(Deserialize)]
pub struct ScanRequest {
    pub host_id: String,
    pub check_type: String,
}

async fn security_scan_handler(
    State(state): State<SecurityState>,
    Json(req): Json<ScanRequest>,
) -> impl IntoResponse {
    let manager = state.manager.read().await;
    match manager.get_module_ref("mod-security") {
        Some(module) => {
            match module.execute(&state.ctx, "security_scan", serde_json::json!({
                "host_id": req.host_id,
                "check_type": req.check_type,
            })).await {
                Ok(value) => (StatusCode::OK, Json(value)).into_response(),
                Err(e) => {
                    tracing::error!(error = %e, "security_scan execution failed");
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response()
                }
            }
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "mod-security not loaded"}))).into_response(),
    }
}

async fn security_list_checks_handler(
    State(state): State<SecurityState>,
) -> impl IntoResponse {
    let manager = state.manager.read().await;
    match manager.get_module_ref("mod-security") {
        Some(module) => {
            match module.execute(&state.ctx, "security_list_checks", serde_json::json!({})).await {
                Ok(value) => (StatusCode::OK, Json(value)).into_response(),
                Err(e) => {
                    tracing::error!(error = %e, "security_list_checks failed");
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response()
                }
            }
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "mod-security not loaded"}))).into_response(),
    }
}

pub fn security_routes(manager: Arc<RwLock<ModuleManager>>, ctx: Arc<ModuleContext>) -> Router {
    let state = SecurityState { manager, ctx };

    Router::new()
        .route("/api/security/scan", post(security_scan_handler))
        .route("/api/security/checks", get(security_list_checks_handler))
        .with_state(state)
}
