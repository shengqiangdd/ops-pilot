use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use sqlx::SqlitePool;

use crate::dashboard_store::{self, SaveLayoutRequest};

#[derive(Clone)]
pub struct DashboardState {
    pub pool: SqlitePool,
}

/// GET /api/dashboard/layouts — 列出所有仪表盘布局。
pub async fn list_layouts(
    State(state): State<DashboardState>,
) -> impl IntoResponse {
    match dashboard_store::list_layouts(&state.pool).await {
        Ok(layouts) => (StatusCode::OK, Json(layouts)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// GET /api/dashboard/layouts/:id — 获取单个仪表盘布局。
pub async fn get_layout(
    Path(id): Path<String>,
    State(state): State<DashboardState>,
) -> impl IntoResponse {
    match dashboard_store::load_layout(&state.pool, &id).await {
        Ok(layout) => (StatusCode::OK, Json(layout)).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// POST /api/dashboard/layouts — 创建仪表盘布局。
pub async fn create_layout(
    State(state): State<DashboardState>,
    Json(req): Json<SaveLayoutRequest>,
) -> impl IntoResponse {
    match dashboard_store::save_layout(&state.pool, None, &req).await {
        Ok(layout) => (StatusCode::CREATED, Json(layout)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// PUT /api/dashboard/layouts/:id — 更新仪表盘布局。
pub async fn update_layout(
    Path(id): Path<String>,
    State(state): State<DashboardState>,
    Json(req): Json<SaveLayoutRequest>,
) -> impl IntoResponse {
    match dashboard_store::save_layout(&state.pool, Some(&id), &req).await {
        Ok(layout) => (StatusCode::OK, Json(layout)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// DELETE /api/dashboard/layouts/:id — 删除仪表盘布局。
pub async fn delete_layout(
    Path(id): Path<String>,
    State(state): State<DashboardState>,
) -> impl IntoResponse {
    match dashboard_store::delete_layout(&state.pool, &id).await {
        Ok(true) => (StatusCode::NO_CONTENT).into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "layout not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

pub fn dashboard_routes(pool: SqlitePool) -> Router {
    let state = DashboardState { pool };

    Router::new()
        .route(
            "/api/dashboard/layouts",
            axum::routing::get(list_layouts).post(create_layout),
        )
        .route(
            "/api/dashboard/layouts/{id}",
            axum::routing::get(get_layout)
                .put(update_layout)
                .delete(delete_layout),
        )
        .with_state(state)
}
