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
struct MonitorState {
    manager: Arc<RwLock<ModuleManager>>,
    ctx: Arc<ModuleContext>,
}

async fn get_metrics(State(state): State<MonitorState>, Path(host_id): Path<String>) -> impl IntoResponse {
    let mgr = state.manager.read().await;
    match mgr.get_module_ref("mod-monitor") {
        Some(module) => {
            let params = serde_json::json!({ "host_id": host_id });
            match module.execute(&state.ctx, "monitor_get_timeseries", params).await {
                Ok(v) => (StatusCode::OK, Json(v)).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
            }
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "mod-monitor not loaded"}))).into_response(),
    }
}

async fn collect_metrics(State(state): State<MonitorState>, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    let mgr = state.manager.read().await;
    match mgr.get_module_ref("mod-monitor") {
        Some(module) => match module.execute(&state.ctx, "monitor_collect", body).await {
            Ok(v) => (StatusCode::OK, Json(v)).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
        },
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "mod-monitor not loaded"}))).into_response(),
    }
}

pub fn monitor_routes(manager: Arc<RwLock<ModuleManager>>, ctx: Arc<ModuleContext>) -> Router {
    let state = MonitorState { manager, ctx };
    Router::new()
        .route("/api/monitor/metrics/{host_id}", axum::routing::get(get_metrics))
        .route("/api/monitor/collect", axum::routing::post(collect_metrics))
        .with_state(state)
}
