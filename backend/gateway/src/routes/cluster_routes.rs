use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use sqlx::SqlitePool;

use crate::cluster_manager::{ClusterManager, RegisterClusterRequest, UpdateClusterRequest};

#[derive(Clone)]
pub struct ClusterState {
    pub pool: SqlitePool,
}

/// GET /api/clusters — 列出所有集群。
pub async fn list_clusters(
    State(state): State<ClusterState>,
) -> impl IntoResponse {
    let mgr = ClusterManager::new(state.pool.clone());
    match mgr.list().await {
        Ok(clusters) => (StatusCode::OK, Json(clusters)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// POST /api/clusters — 注册新集群。
pub async fn register_cluster(
    State(state): State<ClusterState>,
    Json(req): Json<RegisterClusterRequest>,
) -> impl IntoResponse {
    let mgr = ClusterManager::new(state.pool.clone());
    match mgr.register(&req).await {
        Ok(cluster) => (StatusCode::CREATED, Json(cluster)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// GET /api/clusters/:id — 获取单个集群。
pub async fn get_cluster(
    Path(id): Path<String>,
    State(state): State<ClusterState>,
) -> impl IntoResponse {
    let mgr = ClusterManager::new(state.pool.clone());
    match mgr.get(&id).await {
        Ok(cluster) => (StatusCode::OK, Json(cluster)).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// PUT /api/clusters/:id — 更新集群。
pub async fn update_cluster(
    Path(id): Path<String>,
    State(state): State<ClusterState>,
    Json(req): Json<UpdateClusterRequest>,
) -> impl IntoResponse {
    let mgr = ClusterManager::new(state.pool.clone());
    match mgr.update(&id, &req).await {
        Ok(cluster) => (StatusCode::OK, Json(cluster)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// DELETE /api/clusters/:id — 删除集群。
pub async fn delete_cluster(
    Path(id): Path<String>,
    State(state): State<ClusterState>,
) -> impl IntoResponse {
    let mgr = ClusterManager::new(state.pool.clone());
    match mgr.delete(&id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "cluster not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// GET /api/clusters/:id/status — 获取集群状态。
pub async fn cluster_status(
    Path(id): Path<String>,
    State(state): State<ClusterState>,
) -> impl IntoResponse {
    let mgr = ClusterManager::new(state.pool.clone());
    match mgr.status(&id).await {
        Ok(status) => (StatusCode::OK, Json(status)).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

pub fn cluster_routes(pool: SqlitePool) -> Router {
    let state = ClusterState { pool };

    Router::new()
        .route("/api/clusters", axum::routing::get(list_clusters).post(register_cluster))
        .route("/api/clusters/{id}", axum::routing::get(get_cluster).put(update_cluster).delete(delete_cluster))
        .route("/api/clusters/{id}/status", axum::routing::get(cluster_status))
        .with_state(state)
}
