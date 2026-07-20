use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use ops_pilot_sdk::context::ModuleContext;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::modules::ModuleManager;

#[derive(Clone)]
struct RunbookState {
    manager: Arc<RwLock<ModuleManager>>,
    ctx: Arc<ModuleContext>,
}

async fn create(State(state): State<RunbookState>, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    let mgr = state.manager.read().await;
    match mgr.get_module_ref("mod-runbook") {
        Some(module) => match module.execute(&state.ctx, "runbook_create", body).await {
            Ok(v) => (StatusCode::OK, Json(v)).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
        },
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "mod-runbook not loaded"}))).into_response(),
    }
}

async fn execute(State(state): State<RunbookState>, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    let mgr = state.manager.read().await;
    match mgr.get_module_ref("mod-runbook") {
        Some(module) => match module.execute(&state.ctx, "runbook_execute", body).await {
            Ok(v) => (StatusCode::OK, Json(v)).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
        },
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "mod-runbook not loaded"}))).into_response(),
    }
}

pub fn runbook_routes(manager: Arc<RwLock<ModuleManager>>, ctx: Arc<ModuleContext>) -> Router {
    let state = RunbookState { manager, ctx };
    Router::new()
        .route("/api/runbook/create", axum::routing::post(create))
        .route("/api/runbook/execute", axum::routing::post(execute))
        .with_state(state)
}
