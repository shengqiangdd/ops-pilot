use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::otel_integration::{Oteler, IngestSpanRequest, TraceQuery};

#[derive(Clone)]
pub struct OtelerState {
    pub pool: SqlitePool,
}

#[derive(Deserialize)]
pub struct TraceQueryParams {
    pub service: Option<String>,
    pub operation: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub limit: Option<i64>,
}

/// POST /api/otel/ingest — 接收 span。
pub async fn ingest_span(
    State(state): State<OtelerState>,
    Json(req): Json<IngestSpanRequest>,
) -> impl IntoResponse {
    let oteler = Oteler::new(state.pool.clone());
    match oteler.ingest(&req).await {
        Ok(span) => (StatusCode::CREATED, Json(span)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// GET /api/otel/traces — 查询 trace 列表。
pub async fn query_traces(
    State(state): State<OtelerState>,
    axum::extract::Query(params): axum::extract::Query<TraceQueryParams>,
) -> impl IntoResponse {
    let oteler = Oteler::new(state.pool.clone());
    let q = TraceQuery {
        service: params.service,
        operation: params.operation,
        start_time: params.start_time,
        end_time: params.end_time,
        limit: params.limit,
    };
    match oteler.query_traces(&q).await {
        Ok(traces) => (StatusCode::OK, Json(traces)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// GET /api/otel/traces/:trace_id — 获取 trace 树。
pub async fn trace_tree(
    Path(trace_id): Path<String>,
    State(state): State<OtelerState>,
) -> impl IntoResponse {
    let oteler = Oteler::new(state.pool.clone());
    match oteler.query_trace_tree(&trace_id).await {
        Ok(Some(tree)) => (StatusCode::OK, Json(tree)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "trace not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

/// GET /api/otel/services — 列出已知 service。
pub async fn list_services(
    State(state): State<OtelerState>,
) -> impl IntoResponse {
    let oteler = Oteler::new(state.pool.clone());
    match oteler.list_services().await {
        Ok(services) => (StatusCode::OK, Json(services)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
            .into_response(),
    }
}

pub fn otel_routes(pool: SqlitePool) -> Router {
    let state = OtelerState { pool };

    Router::new()
        .route("/api/otel/ingest", axum::routing::post(ingest_span))
        .route("/api/otel/traces", axum::routing::get(query_traces))
        .route("/api/otel/traces/{trace_id}", axum::routing::get(trace_tree))
        .route("/api/otel/services", axum::routing::get(list_services))
        .with_state(state)
}
