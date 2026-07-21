use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::anomaly_detector::{AnomalyDetector, DetectRequest};

#[derive(Clone)]
pub struct AnomalyState {
    pub pool: SqlitePool,
}

#[derive(Deserialize)]
pub struct TrendQuery {
    pub days: Option<i64>,
}

/// POST /api/anomaly/detect — 检测异常点。
pub async fn detect_anomaly(
    State(state): State<AnomalyState>,
    Json(req): Json<DetectRequest>,
) -> impl IntoResponse {
    let detector = AnomalyDetector::new(state.pool.clone());
    match detector.detect(&req).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// GET /api/anomaly/alert-trends — 分析历史告警趋势。
pub async fn alert_trends(
    State(state): State<AnomalyState>,
    axum::extract::Query(query): axum::extract::Query<TrendQuery>,
) -> impl IntoResponse {
    let detector = AnomalyDetector::new(state.pool.clone());
    let days = query.days.unwrap_or(7);
    match detector.alert_trends(days).await {
        Ok(trends) => (StatusCode::OK, Json(trends)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

pub fn anomaly_routes(pool: SqlitePool) -> Router {
    let state = AnomalyState { pool };

    Router::new()
        .route("/api/anomaly/detect", axum::routing::post(detect_anomaly))
        .route("/api/anomaly/alert-trends", axum::routing::get(alert_trends))
        .with_state(state)
}
