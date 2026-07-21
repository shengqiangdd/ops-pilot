use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::Sqlite;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct ApmState {
    pub pool: SqlitePool,
}

#[derive(Debug, Deserialize)]
pub struct TracesQuery {
    pub status: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorsQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateErrorRequest {
    pub status: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ApmService {
    pub id: String,
    pub name: String,
    #[sqlx(rename = "type")]
    pub service_type: String,
    pub host_id: String,
    pub endpoint: String,
    pub language: String,
    pub health: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ApmTrace {
    pub id: String,
    pub service_id: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation: String,
    pub duration_ms: f64,
    pub start_time: String,
    pub status: String,
    pub http_method: Option<String>,
    pub http_path: Option<String>,
    pub http_status: Option<i64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ApmError {
    pub id: String,
    pub service_id: String,
    pub error_type: String,
    pub error_message: String,
    pub stack_trace: String,
    pub count: i64,
    pub first_seen: String,
    pub last_seen: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ApmDashboard {
    pub total_services: i64,
    pub healthy_services: i64,
    pub total_requests: i64,
    pub error_rate: f64,
    pub avg_latency: f64,
    pub p99_latency: f64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateErrorStatusRequest {
    pub status: String,
}

/// GET /api/apm/services — list APM services
pub async fn list_services(
    State(state): State<ApmState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, ApmService>(
        "SELECT id, name, type, host_id, endpoint, language, health FROM apm_services ORDER BY name"
    ).fetch_all(&state.pool).await;
    match result {
        Ok(services) => (StatusCode::OK, Json(services)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/apm/services/:id — get service detail
pub async fn get_service(
    Path(service_id): Path<String>,
    State(state): State<ApmState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, ApmService>(
        "SELECT id, name, type, host_id, endpoint, language, health FROM apm_services WHERE id = ?"
    ).bind(&service_id).fetch_optional(&state.pool).await;
    match result {
        Ok(Some(s)) => (StatusCode::OK, Json(s)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "service not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/apm/services/:id/traces — list traces for service
pub async fn list_traces(
    Path(service_id): Path<String>,
    State(state): State<ApmState>,
    Query(query): Query<TracesQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;

    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        "SELECT id, service_id, trace_id, span_id, parent_span_id, operation, duration_ms, start_time, status, http_method, http_path, http_status, error_message FROM apm_traces WHERE service_id = "
    );
    builder.push_bind(service_id);
    if let Some(ref status) = query.status {
        builder.push(" AND status = ");
        builder.push_bind(status.clone());
    }
    builder.push(" ORDER BY start_time DESC LIMIT ");
    builder.push_bind(per_page as i64);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let q = builder.build_query_as::<ApmTrace>();
    match q.fetch_all(&state.pool).await {
        Ok(traces) => (StatusCode::OK, Json(traces)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/apm/traces/:id — get trace detail
pub async fn get_trace(
    Path(trace_id): Path<String>,
    State(state): State<ApmState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, ApmTrace>(
        "SELECT id, service_id, trace_id, span_id, parent_span_id, operation, duration_ms, start_time, status, http_method, http_path, http_status, error_message FROM apm_traces WHERE trace_id = ? OR id = ?"
    ).bind(&trace_id).bind(&trace_id).fetch_all(&state.pool).await;
    match result {
        Ok(traces) => (StatusCode::OK, Json(traces)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/apm/traces/recent-errors — recent errors
pub async fn recent_errors(
    State(state): State<ApmState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, ApmError>(
        "SELECT id, service_id, error_type, error_message, stack_trace, count, first_seen, last_seen, status FROM apm_errors ORDER BY last_seen DESC LIMIT 50"
    ).fetch_all(&state.pool).await;
    match result {
        Ok(errors) => (StatusCode::OK, Json(errors)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/apm/services/:id/errors — service errors
pub async fn service_errors(
    Path(service_id): Path<String>,
    State(state): State<ApmState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, ApmError>(
        "SELECT id, service_id, error_type, error_message, stack_trace, count, first_seen, last_seen, status FROM apm_errors WHERE service_id = ? ORDER BY last_seen DESC"
    ).bind(&service_id).fetch_all(&state.pool).await;
    match result {
        Ok(errors) => (StatusCode::OK, Json(errors)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// PUT /api/apm/errors/:id — update error status
pub async fn update_error(
    Path(error_id): Path<String>,
    State(state): State<ApmState>,
    Json(req): Json<UpdateErrorStatusRequest>,
) -> impl IntoResponse {
    let result = sqlx::query("UPDATE apm_errors SET status = ? WHERE id = ?")
        .bind(&req.status).bind(&error_id).execute(&state.pool).await;
    match result {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "updated"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/apm/dashboard — APM dashboard overview
pub async fn apm_dashboard(
    State(state): State<ApmState>,
) -> impl IntoResponse {
    let total_services: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM apm_services").fetch_one(&state.pool).await.unwrap_or((0,));
    let healthy: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM apm_services WHERE health = 'healthy'").fetch_one(&state.pool).await.unwrap_or((0,));
    let total_requests: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM apm_traces").fetch_one(&state.pool).await.unwrap_or((0,));
    let error_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM apm_traces WHERE status = 'error'").fetch_one(&state.pool).await.unwrap_or((0,));
    let avg_latency: (f64,) = sqlx::query_as("SELECT COALESCE(AVG(duration_ms), 0) FROM apm_traces").fetch_one(&state.pool).await.unwrap_or((0.0,));
    let p99_latency: (f64,) = sqlx::query_as("SELECT COALESCE(MAX(duration_ms), 0) FROM apm_traces").fetch_one(&state.pool).await.unwrap_or((0.0,));

    let error_rate = if total_requests.0 > 0 { (error_count.0 as f64 / total_requests.0 as f64) * 100.0 } else { 0.0 };

    let dashboard = ApmDashboard {
        total_services: total_services.0,
        healthy_services: healthy.0,
        total_requests: total_requests.0,
        error_rate,
        avg_latency: avg_latency.0,
        p99_latency: p99_latency.0,
    };

    (StatusCode::OK, Json(dashboard)).into_response()
}

/// PUT /api/apm/errors/:id — update error
pub async fn update_error_status(
    Path(error_id): Path<String>,
    State(state): State<ApmState>,
    Json(req): Json<UpdateErrorStatusRequest>,
) -> impl IntoResponse {
    let result = sqlx::query("UPDATE apm_errors SET status = ? WHERE id = ?")
        .bind(&req.status).bind(&error_id).execute(&state.pool).await;
    match result {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "updated"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

pub fn apm_routes(pool: SqlitePool) -> Router {
    use axum::routing::{get, put};
    let state = ApmState { pool };
    Router::new()
        .route("/api/apm/services", get(list_services))
        .route("/api/apm/services/{id}", get(get_service))
        .route("/api/apm/services/{id}/traces", get(list_traces))
        .route("/api/apm/services/{id}/errors", get(service_errors))
        .route("/api/apm/traces/{id}", get(get_trace))
        .route("/api/apm/traces/recent-errors", get(recent_errors))
        .route("/api/apm/dashboard", get(apm_dashboard))
        .route("/api/apm/errors/{id}", put(update_error_status))
        .with_state(state)
}
