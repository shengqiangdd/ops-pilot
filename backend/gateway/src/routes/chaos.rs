use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::Sqlite;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct ChaosState {
    pub pool: SqlitePool,
}

#[derive(Debug, Deserialize)]
pub struct CreateExperimentRequest {
    pub name: String,
    pub description: Option<String>,
    pub target_host_ids_json: Option<String>,
    pub target_type: Option<String>,
    pub fault_type: String,
    pub duration_seconds: Option<u32>,
    pub params_json: Option<String>,
    pub created_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateExperimentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub fault_type: Option<String>,
    pub duration_seconds: Option<u32>,
    pub params_json: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ChaosExperiment {
    pub id: String,
    pub name: String,
    pub description: String,
    pub target_host_ids_json: String,
    pub target_type: String,
    pub fault_type: String,
    pub duration_seconds: u32,
    pub params_json: String,
    pub status: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ChaosExecution {
    pub id: String,
    pub experiment_id: String,
    pub step: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub result_json: String,
}

#[derive(Debug, Serialize)]
pub struct ChaosStats {
    pub total_experiments: i64,
    pub completed: i64,
    pub failed: i64,
    pub running: i64,
}

fn pseudo_random(seed: u64) -> f64 {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64;
    ((nanos.wrapping_mul(6364136223846793005).wrapping_add(seed)) % 1000) as f64 / 1000.0
}

/// GET /api/chaos/experiments — list experiments
pub async fn list_experiments(
    State(state): State<ChaosState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, ChaosExperiment>(
        "SELECT id, name, description, target_host_ids_json, target_type, fault_type, duration_seconds, params_json, status, started_at, finished_at, created_by, created_at FROM chaos_experiments ORDER BY created_at DESC"
    ).fetch_all(&state.pool).await;
    match result {
        Ok(experiments) => (StatusCode::OK, Json(experiments)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// POST /api/chaos/experiments — create experiment
pub async fn create_experiment(
    State(state): State<ChaosState>,
    Json(req): Json<CreateExperimentRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let result = sqlx::query(
        "INSERT INTO chaos_experiments (id, name, description, target_host_ids_json, target_type, fault_type, duration_seconds, params_json, created_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    ).bind(&id).bind(&req.name).bind(req.description.as_deref().unwrap_or(""))
        .bind(req.target_host_ids_json.as_deref().unwrap_or("[]"))
        .bind(req.target_type.as_deref().unwrap_or("host"))
        .bind(&req.fault_type).bind(req.duration_seconds.unwrap_or(60) as i64)
        .bind(req.params_json.as_deref().unwrap_or("{}"))
        .bind(req.created_by.as_deref().unwrap_or(""))
        .execute(&state.pool).await;
    match result {
        Ok(_) => {
            let exp = sqlx::query_as::<_, ChaosExperiment>(
                "SELECT id, name, description, target_host_ids_json, target_type, fault_type, duration_seconds, params_json, status, started_at, finished_at, created_by, created_at FROM chaos_experiments WHERE id = ?"
            ).bind(&id).fetch_one(&state.pool).await;
            match exp { Ok(e) => (StatusCode::CREATED, Json(e)).into_response(), Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response() }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/chaos/experiments/:id — get experiment
pub async fn get_experiment(
    Path(exp_id): Path<String>,
    State(state): State<ChaosState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, ChaosExperiment>(
        "SELECT id, name, description, target_host_ids_json, target_type, fault_type, duration_seconds, params_json, status, started_at, finished_at, created_by, created_at FROM chaos_experiments WHERE id = ?"
    ).bind(&exp_id).fetch_optional(&state.pool).await;
    match result {
        Ok(Some(e)) => (StatusCode::OK, Json(e)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "experiment not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// PUT /api/chaos/experiments/:id — update experiment
pub async fn update_experiment(
    Path(exp_id): Path<String>,
    State(state): State<ChaosState>,
    Json(req): Json<UpdateExperimentRequest>,
) -> impl IntoResponse {
    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new("UPDATE chaos_experiments SET ");
    let mut first = true;
    if let Some(name) = &req.name { builder.push("name = "); builder.push_bind(name.clone()); first = false; }
    if let Some(desc) = &req.description { if !first { builder.push(", "); } builder.push("description = "); builder.push_bind(desc.clone()); first = false; }
    if let Some(ft) = &req.fault_type { if !first { builder.push(", "); } builder.push("fault_type = "); builder.push_bind(ft.clone()); first = false; }
    if let Some(dur) = req.duration_seconds { if !first { builder.push(", "); } builder.push("duration_seconds = "); builder.push_bind(dur as i64); first = false; }
    if let Some(params) = &req.params_json { if !first { builder.push(", "); } builder.push("params_json = "); builder.push_bind(params.clone()); first = false; }
    if first { return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "no fields to update"}))).into_response(); }
    builder.push(" WHERE id = "); builder.push_bind(exp_id.clone());
    match builder.build().execute(&state.pool).await {
        Ok(_) => {
            let exp = sqlx::query_as::<_, ChaosExperiment>(
                "SELECT id, name, description, target_host_ids_json, target_type, fault_type, duration_seconds, params_json, status, started_at, finished_at, created_by, created_at FROM chaos_experiments WHERE id = ?"
            ).bind(&exp_id).fetch_one(&state.pool).await;
            match exp { Ok(e) => (StatusCode::OK, Json(e)).into_response(), Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response() }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// DELETE /api/chaos/experiments/:id
pub async fn delete_experiment(
    Path(exp_id): Path<String>,
    State(state): State<ChaosState>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM chaos_experiments WHERE id = ?").bind(&exp_id).execute(&state.pool).await;
    match result { Ok(_) => (StatusCode::NO_CONTENT).into_response(), Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response() }
}

/// POST /api/chaos/experiments/:id/run — run experiment
pub async fn run_experiment(
    Path(exp_id): Path<String>,
    State(state): State<ChaosState>,
) -> impl IntoResponse {
    let _ = sqlx::query("UPDATE chaos_experiments SET status = 'running', started_at = datetime('now') WHERE id = ?")
        .bind(&exp_id).execute(&state.pool).await;
    let exec_id = uuid::Uuid::new_v4().to_string();
    let _ = sqlx::query("INSERT INTO chaos_executions (id, experiment_id, step, status, result_json) VALUES (?, ?, 'start', 'completed', '{}')")
        .bind(&exec_id).bind(&exp_id).execute(&state.pool).await;
    // Simulate completion after delay (in real system, this would be async)
    let _ = sqlx::query("UPDATE chaos_experiments SET status = 'completed', finished_at = datetime('now') WHERE id = ? AND status = 'running'")
        .bind(&exp_id).execute(&state.pool).await;
    (StatusCode::OK, Json(serde_json::json!({"status": "completed", "execution_id": exec_id}))).into_response()
}

/// POST /api/chaos/experiments/:id/stop — stop experiment
pub async fn stop_experiment(
    Path(exp_id): Path<String>,
    State(state): State<ChaosState>,
) -> impl IntoResponse {
    let result = sqlx::query("UPDATE chaos_experiments SET status = 'stopped', finished_at = datetime('now') WHERE id = ?")
        .bind(&exp_id).execute(&state.pool).await;
    match result { Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "stopped"}))).into_response(), Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response() }
}

/// GET /api/chaos/executions — list executions
pub async fn list_executions(
    State(state): State<ChaosState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, ChaosExecution>(
        "SELECT id, experiment_id, step, status, started_at, finished_at, result_json FROM chaos_executions ORDER BY started_at DESC"
    ).fetch_all(&state.pool).await;
    match result {
        Ok(execs) => (StatusCode::OK, Json(execs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/chaos/stats
pub async fn chaos_stats(
    State(state): State<ChaosState>,
) -> impl IntoResponse {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM chaos_experiments").fetch_one(&state.pool).await.unwrap_or((0,));
    let completed: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM chaos_experiments WHERE status = 'completed'").fetch_one(&state.pool).await.unwrap_or((0,));
    let failed: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM chaos_experiments WHERE status = 'failed'").fetch_one(&state.pool).await.unwrap_or((0,));
    let running: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM chaos_experiments WHERE status = 'running'").fetch_one(&state.pool).await.unwrap_or((0,));
    (StatusCode::OK, Json(ChaosStats { total_experiments: total.0, completed: completed.0, failed: failed.0, running: running.0 })).into_response()
}

pub fn chaos_routes(pool: SqlitePool) -> Router {
    use axum::routing::{get, post};
    let state = ChaosState { pool };
    Router::new()
        .route("/api/chaos/experiments", get(list_experiments).post(create_experiment))
        .route("/api/chaos/experiments/{id}", get(get_experiment).put(update_experiment).delete(delete_experiment))
        .route("/api/chaos/experiments/{id}/run", post(run_experiment))
        .route("/api/chaos/experiments/{id}/stop", post(stop_experiment))
        .route("/api/chaos/executions", get(list_executions))
        .route("/api/chaos/stats", get(chaos_stats))
        .with_state(state)
}
