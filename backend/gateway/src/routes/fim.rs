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
struct FimState {
    manager: Arc<RwLock<ModuleManager>>,
    ctx: Arc<ModuleContext>,
}

async fn create_baseline(State(state): State<FimState>, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    let mgr = state.manager.read().await;
    match mgr.get_module_ref("mod-fim") {
        Some(module) => match module.execute(&state.ctx, "fim_baseline", body).await {
            Ok(v) => (StatusCode::OK, Json(v)).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
        },
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "mod-fim not loaded"}))).into_response(),
    }
}

async fn scan(State(state): State<FimState>, Path(host_id): Path<String>) -> impl IntoResponse {
    let mgr = state.manager.read().await;
    match mgr.get_module_ref("mod-fim") {
        Some(module) => {
            let params = serde_json::json!({ "host_id": host_id });
            match module.execute(&state.ctx, "fim_scan", params).await {
                Ok(v) => (StatusCode::OK, Json(v)).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
            }
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "mod-fim not loaded"}))).into_response(),
    }
}

pub fn fim_routes(manager: Arc<RwLock<ModuleManager>>, ctx: Arc<ModuleContext>) -> Router {
    let state = FimState { manager, ctx };
    Router::new()
        .route("/api/fim/baseline", axum::routing::post(create_baseline))
        .route("/api/fim/scan/{host_id}", axum::routing::get(scan))
        .with_state(state)
}
