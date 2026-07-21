use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use sqlx::SqlitePool;

use crate::rca_engine::RcaEngine;

#[derive(Clone)]
pub struct RcaState {
    pub pool: SqlitePool,
}

/// GET /api/rca/correlate/:alert_id — 关联分析。
pub async fn correlate_alert(
    Path(alert_id): Path<String>,
    State(state): State<RcaState>,
) -> impl IntoResponse {
    let engine = RcaEngine::new(state.pool.clone());
    match engine.correlate(&alert_id).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// GET /api/rca/causal-chain/:incident_id — 因果链。
pub async fn causal_chain(
    Path(incident_id): Path<String>,
    State(state): State<RcaState>,
) -> impl IntoResponse {
    let engine = RcaEngine::new(state.pool.clone());
    match engine.causal_chain(&incident_id).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

pub fn rca_routes(pool: SqlitePool) -> Router {
    let state = RcaState { pool };

    Router::new()
        .route("/api/rca/correlate/{alert_id}", axum::routing::get(correlate_alert))
        .route("/api/rca/causal-chain/{incident_id}", axum::routing::get(causal_chain))
        .with_state(state)
}
