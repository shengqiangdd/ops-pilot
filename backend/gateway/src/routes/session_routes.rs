use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::session_recorder::{SessionRecorder, RecordRequest};

#[derive(Clone)]
pub struct SessionState {
    pub pool: SqlitePool,
}

#[derive(Deserialize)]
pub struct ListSessionQuery {
    pub host: Option<String>,
    pub user: Option<String>,
}

/// GET /api/sessions — 列出 session 列表。
pub async fn list_sessions(
    State(state): State<SessionState>,
    axum::extract::Query(query): axum::extract::Query<ListSessionQuery>,
) -> impl IntoResponse {
    let recorder = SessionRecorder::new(state.pool.clone());
    match recorder.list_sessions(query.host.as_deref(), query.user.as_deref()).await {
        Ok(sessions) => (StatusCode::OK, Json(sessions)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// GET /api/sessions/:id/replay — 回放 session。
pub async fn replay_session(
    Path(session_id): Path<String>,
    State(state): State<SessionState>,
) -> impl IntoResponse {
    let recorder = SessionRecorder::new(state.pool.clone());
    match recorder.replay(&session_id).await {
        Ok(records) => (StatusCode::OK, Json(records)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// POST /api/sessions/record — 记录一条操作。
pub async fn record_session(
    State(state): State<SessionState>,
    Json(req): Json<RecordRequest>,
) -> impl IntoResponse {
    let recorder = SessionRecorder::new(state.pool.clone());
    match recorder.record(&req).await {
        Ok(record) => (StatusCode::CREATED, Json(record)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

pub fn session_routes(pool: SqlitePool) -> Router {
    let state = SessionState { pool };

    Router::new()
        .route("/api/sessions", axum::routing::get(list_sessions))
        .route("/api/sessions/{id}/replay", axum::routing::get(replay_session))
        .route("/api/sessions/record", axum::routing::post(record_session))
        .with_state(state)
}
