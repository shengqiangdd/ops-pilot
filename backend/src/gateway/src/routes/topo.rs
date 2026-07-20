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
struct TopoState {
    manager: Arc<RwLock<ModuleManager>>,
    ctx: Arc<ModuleContext>,
}

async fn get_graph(State(state): State<TopoState>) -> impl IntoResponse {
    let mgr = state.manager.read().await;
    match mgr.get_module_ref("mod-topo") {
        Some(module) => match module.execute(&state.ctx, "topo_get_graph", serde_json::json!({})).await {
            Ok(v) => (StatusCode::OK, Json(v)).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
        },
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "mod-topo not loaded"}))).into_response(),
    }
}

async fn discover(State(state): State<TopoState>, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    let mgr = state.manager.read().await;
    match mgr.get_module_ref("mod-topo") {
        Some(module) => match module.execute(&state.ctx, "topo_discover", body).await {
            Ok(v) => (StatusCode::OK, Json(v)).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
        },
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "mod-topo not loaded"}))).into_response(),
    }
}

pub fn topo_routes(manager: Arc<RwLock<ModuleManager>>, ctx: Arc<ModuleContext>) -> Router {
    let state = TopoState { manager, ctx };
    Router::new()
        .route("/api/topo/graph", axum::routing::get(get_graph))
        .route("/api/topo/discover", axum::routing::post(discover))
        .with_state(state)
}
