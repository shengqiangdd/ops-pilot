use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::audit_log::{AuditLogger, SlowQueryTracker};

#[derive(Clone)]
pub struct AuditLogState {
    pub pool: SqlitePool,
}

#[derive(Deserialize)]
pub struct LogActionRequest {
    pub actor: String,
    pub action: String,
    pub resource: String,
    pub detail: Option<String>,
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// GET /api/audit/logs — 查询审计日志。
pub async fn list_audit_logs(
    State(state): State<AuditLogState>,
    axum::extract::Query(query): axum::extract::Query<ListQuery>,
) -> impl IntoResponse {
    let logger = AuditLogger::new(state.pool.clone());
    let limit = query.limit.unwrap_or(50).min(200);
    let offset = query.offset.unwrap_or(0);

    match logger.list_logs(limit, offset).await {
        Ok(logs) => (StatusCode::OK, Json(logs)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// POST /api/audit/logs — 记录一条审计日志。
pub async fn create_audit_log(
    State(state): State<AuditLogState>,
    Json(req): Json<LogActionRequest>,
) -> impl IntoResponse {
    let logger = AuditLogger::new(state.pool.clone());
    let detail = req.detail.unwrap_or_default();

    match logger.log_action(&req.actor, &req.action, &req.resource, &detail).await {
        Ok(()) => (StatusCode::CREATED, Json(serde_json::json!({"ok": true}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// GET /api/audit/slow-queries — 查询慢查询列表。
pub async fn list_slow_queries(
    State(state): State<AuditLogState>,
    axum::extract::Query(query): axum::extract::Query<ListQuery>,
) -> impl IntoResponse {
    let tracker = SlowQueryTracker::new(state.pool.clone());
    let limit = query.limit.unwrap_or(50).min(200);

    match tracker.list_slow_queries(limit).await {
        Ok(queries) => (StatusCode::OK, Json(queries)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

pub fn audit_log_routes(pool: SqlitePool) -> Router {
    let state = AuditLogState { pool };

    Router::new()
        .route(
            "/api/audit/logs",
            axum::routing::get(list_audit_logs).post(create_audit_log),
        )
        .route("/api/audit/slow-queries", axum::routing::get(list_slow_queries))
        .with_state(state)
}
