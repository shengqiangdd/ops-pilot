use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use ops_pilot_sdk::context::ModuleContext;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::modules::ModuleManager;

#[derive(Clone)]
struct BaselineState {
    manager: Arc<RwLock<ModuleManager>>,
    ctx: Arc<ModuleContext>,
}

async fn run_check(State(state): State<BaselineState>, Path(host_id): Path<String>, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    let mgr = state.manager.read().await;
    match mgr.get_module_ref("mod-baseline") {
        Some(module) => {
            let params = serde_json::json!({
                "host_id": host_id,
                "check_name": body.get("check_name").and_then(|v| v.as_str()).unwrap_or("all")
            });
            match module.execute(&state.ctx, "baseline_run_check", params).await {
                Ok(v) => (StatusCode::OK, Json(v)).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
            }
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "mod-baseline not loaded"}))).into_response(),
    }
}

async fn get_report(State(state): State<BaselineState>, Path(host_id): Path<String>) -> impl IntoResponse {
    let mgr = state.manager.read().await;
    match mgr.get_module_ref("mod-baseline") {
        Some(module) => {
            let params = serde_json::json!({ "host_id": host_id });
            match module.execute(&state.ctx, "baseline_report", params).await {
                Ok(v) => (StatusCode::OK, Json(v)).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
            }
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "mod-baseline not loaded"}))).into_response(),
    }
}

pub fn baseline_routes(manager: Arc<RwLock<ModuleManager>>, ctx: Arc<ModuleContext>) -> Router {
    let state = BaselineState { manager, ctx };
    Router::new()
        .route("/api/baseline/check/{host_id}", axum::routing::post(run_check))
        .route("/api/baseline/report/{host_id}", axum::routing::get(get_report))
        .with_state(state)
}
