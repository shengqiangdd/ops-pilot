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
struct EscalationState {
    manager: Arc<RwLock<ModuleManager>>,
    ctx: Arc<ModuleContext>,
}

async fn trigger(State(state): State<EscalationState>, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    let mgr = state.manager.read().await;
    match mgr.get_module_ref("mod-alert-escalation") {
        Some(module) => match module.execute(&state.ctx, "escalation_trigger", body).await {
            Ok(v) => (StatusCode::OK, Json(v)).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
        },
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "mod-alert-escalation not loaded"}))).into_response(),
    }
}

async fn define_policy(State(state): State<EscalationState>, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    let mgr = state.manager.read().await;
    match mgr.get_module_ref("mod-alert-escalation") {
        Some(module) => match module.execute(&state.ctx, "escalation_define_policy", body).await {
            Ok(v) => (StatusCode::OK, Json(v)).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
        },
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "mod-alert-escalation not loaded"}))).into_response(),
    }
}

pub fn escalation_routes(manager: Arc<RwLock<ModuleManager>>, ctx: Arc<ModuleContext>) -> Router {
    let state = EscalationState { manager, ctx };
    Router::new()
        .route("/api/escalation/policies", axum::routing::post(define_policy))
        .route("/api/escalation/trigger", axum::routing::post(trigger))
        .with_state(state)
}
