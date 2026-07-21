use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::gitops::GitOps;

#[derive(Clone)]
pub struct GitOpsState {
    pub pool: SqlitePool,
    pub gitops: GitOps,
}

#[derive(Deserialize)]
pub struct SyncRequest {
    pub message: Option<String>,
}

/// GET /api/gitops/status — 获取 git 仓库状态。
pub async fn gitops_status(
    State(state): State<GitOpsState>,
) -> impl IntoResponse {
    match state.gitops.status() {
        Ok(status) => (StatusCode::OK, Json(status)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// POST /api/gitops/sync — 同步配置（git add + commit + push）。
pub async fn gitops_sync(
    State(state): State<GitOpsState>,
    Json(req): Json<SyncRequest>,
) -> impl IntoResponse {
    let message = req
        .message
        .unwrap_or_else(|| "ops-pilot: config sync".into());

    match state.gitops.sync(&message) {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

pub fn gitops_routes(pool: SqlitePool) -> Router {
    let gitops = GitOps::new("/app/data");
    let state = GitOpsState { pool, gitops };

    Router::new()
        .route("/api/gitops/status", axum::routing::get(gitops_status))
        .route("/api/gitops/sync", axum::routing::post(gitops_sync))
        .with_state(state)
}
