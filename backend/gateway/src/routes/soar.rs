use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::Sqlite;
use sqlx::SqlitePool;

/// Shared application state for SOAR routes.
#[derive(Clone)]
pub struct SoarState {
    pub pool: SqlitePool,
}

// ── Request/Response Types ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreatePlaybookRequest {
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: String,
    pub trigger_conditions_json: Option<String>,
    pub steps_json: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePlaybookRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub steps_json: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Playbook {
    pub id: String,
    pub name: String,
    pub description: String,
    pub trigger_type: String,
    pub trigger_conditions_json: String,
    pub steps_json: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Execution {
    pub id: String,
    pub playbook_id: String,
    pub trigger_source: String,
    pub trigger_id: String,
    pub status: String,
    pub result_json: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionDetail {
    pub execution: Execution,
    pub playbook_name: String,
    pub steps: Vec<ExecutionStep>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_type: String,
    pub status: String,
    pub message: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ExecutionsQuery {
    pub playbook_id: Option<String>,
    pub status: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

// ── Handlers ───────────────────────────────────────────────────────────

/// GET /api/soar/playbooks — list playbooks
pub async fn list_playbooks(
    State(state): State<SoarState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Playbook>(
        "SELECT id, name, description, trigger_type, trigger_conditions_json, steps_json, enabled, created_at, updated_at FROM soar_playbooks ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(playbooks) => (StatusCode::OK, Json(playbooks)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/soar/playbooks — create playbook
pub async fn create_playbook(
    State(state): State<SoarState>,
    Json(req): Json<CreatePlaybookRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();

    let result = sqlx::query(
        "INSERT INTO soar_playbooks (id, name, description, trigger_type, trigger_conditions_json, steps_json) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(req.description.as_deref().unwrap_or(""))
    .bind(&req.trigger_type)
    .bind(req.trigger_conditions_json.as_deref().unwrap_or("{}"))
    .bind(&req.steps_json)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let playbook = sqlx::query_as::<_, Playbook>(
                "SELECT id, name, description, trigger_type, trigger_conditions_json, steps_json, enabled, created_at, updated_at FROM soar_playbooks WHERE id = ?"
            )
            .bind(&id)
            .fetch_one(&state.pool)
            .await;
            match playbook {
                Ok(p) => (StatusCode::CREATED, Json(p)).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/soar/playbooks/:id — get playbook detail
pub async fn get_playbook(
    Path(playbook_id): Path<String>,
    State(state): State<SoarState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Playbook>(
        "SELECT id, name, description, trigger_type, trigger_conditions_json, steps_json, enabled, created_at, updated_at FROM soar_playbooks WHERE id = ?"
    )
    .bind(&playbook_id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(p)) => (StatusCode::OK, Json(p)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "playbook not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// PUT /api/soar/playbooks/:id — update playbook
pub async fn update_playbook(
    Path(playbook_id): Path<String>,
    State(state): State<SoarState>,
    Json(req): Json<UpdatePlaybookRequest>,
) -> impl IntoResponse {
    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new("UPDATE soar_playbooks SET ");

    let mut first = true;
    if let Some(name) = &req.name {
        builder.push("name = ");
        builder.push_bind(name.clone());
        first = false;
    }
    if let Some(description) = &req.description {
        if !first { builder.push(", "); }
        builder.push("description = ");
        builder.push_bind(description.clone());
        first = false;
    }
    if let Some(steps_json) = &req.steps_json {
        if !first { builder.push(", "); }
        builder.push("steps_json = ");
        builder.push_bind(steps_json.clone());
        first = false;
    }
    if let Some(enabled) = req.enabled {
        if !first { builder.push(", "); }
        builder.push("enabled = ");
        builder.push_bind(enabled);
        first = false;
    }

    if first {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "no fields to update"}))).into_response();
    }

    builder.push(", updated_at = datetime('now')");
    builder.push(" WHERE id = ");
    builder.push_bind(playbook_id.clone());

    match builder.build().execute(&state.pool).await {
        Ok(_) => {
            let playbook = sqlx::query_as::<_, Playbook>(
                "SELECT id, name, description, trigger_type, trigger_conditions_json, steps_json, enabled, created_at, updated_at FROM soar_playbooks WHERE id = ?"
            )
            .bind(&playbook_id)
            .fetch_one(&state.pool)
            .await;
            match playbook {
                Ok(p) => (StatusCode::OK, Json(p)).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// DELETE /api/soar/playbooks/:id — delete playbook
pub async fn delete_playbook(
    Path(playbook_id): Path<String>,
    State(state): State<SoarState>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM soar_playbooks WHERE id = ?")
        .bind(&playbook_id).execute(&state.pool).await;

    match result {
        Ok(_) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// POST /api/soar/playbooks/:id/execute — execute playbook
pub async fn execute_playbook(
    Path(playbook_id): Path<String>,
    State(state): State<SoarState>,
) -> impl IntoResponse {
    // Check if playbook exists
    let playbook = sqlx::query_as::<_, Playbook>(
        "SELECT id, name, description, trigger_type, trigger_conditions_json, steps_json, enabled, created_at, updated_at FROM soar_playbooks WHERE id = ?"
    )
    .bind(&playbook_id)
    .fetch_optional(&state.pool)
    .await;

    match playbook {
        Ok(Some(_pb)) => {
            let exec_id = uuid::Uuid::new_v4().to_string();

            // Parse steps to simulate execution
            let steps: Vec<serde_json::Value> = serde_json::from_str("[]").unwrap_or_default();

            let mut executed_steps = Vec::new();
            for step in &steps {
                let step_type = step["type"].as_str().unwrap_or("unknown");
                executed_steps.push(ExecutionStep {
                    step_type: step_type.to_string(),
                    status: "completed".to_string(),
                    message: format!("Step '{}' completed successfully", step_type),
                    duration_ms: Some(100),
                });
            }

            let result_json = serde_json::json!({
                "steps": executed_steps,
                "total_steps": executed_steps.len(),
                "completed_steps": executed_steps.len(),
            });

            let _ = sqlx::query(
                "INSERT INTO soar_executions (id, playbook_id, trigger_source, trigger_id, status, result_json) VALUES (?, ?, 'manual', ?, 'completed', ?)"
            )
            .bind(&exec_id)
            .bind(&playbook_id)
            .bind(&playbook_id)
            .bind(result_json.to_string())
            .execute(&state.pool)
            .await;

            let execution = sqlx::query_as::<_, Execution>(
                "SELECT id, playbook_id, trigger_source, trigger_id, status, result_json, started_at, finished_at FROM soar_executions WHERE id = ?"
            )
            .bind(&exec_id)
            .fetch_one(&state.pool)
            .await;

            match execution {
                Ok(e) => (StatusCode::CREATED, Json(e)).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "playbook not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/soar/executions — list executions
pub async fn list_executions(
    State(state): State<SoarState>,
    Query(query): Query<ExecutionsQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;

    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        "SELECT id, playbook_id, trigger_source, trigger_id, status, result_json, started_at, finished_at FROM soar_executions WHERE 1=1"
    );

    if let Some(ref playbook_id) = query.playbook_id {
        builder.push(" AND playbook_id = ");
        builder.push_bind(playbook_id.clone());
    }
    if let Some(ref status) = query.status {
        builder.push(" AND status = ");
        builder.push_bind(status.clone());
    }

    builder.push(" ORDER BY started_at DESC LIMIT ");
    builder.push_bind(per_page as i64);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let q = builder.build_query_as::<Execution>();

    match q.fetch_all(&state.pool).await {
        Ok(executions) => (StatusCode::OK, Json(executions)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/soar/executions/:id — get execution detail
pub async fn get_execution(
    Path(exec_id): Path<String>,
    State(state): State<SoarState>,
) -> impl IntoResponse {
    let execution = sqlx::query_as::<_, Execution>(
        "SELECT id, playbook_id, trigger_source, trigger_id, status, result_json, started_at, finished_at FROM soar_executions WHERE id = ?"
    )
    .bind(&exec_id)
    .fetch_optional(&state.pool)
    .await;

    match execution {
        Ok(Some(exec)) => {
            // Get playbook name
            let playbook_name = sqlx::query_scalar::<_, String>("SELECT name FROM soar_playbooks WHERE id = ?")
                .bind(&exec.playbook_id)
                .fetch_optional(&state.pool)
                .await
                .unwrap_or(None)
                .unwrap_or_else(|| "Unknown".to_string());

            let steps: Vec<ExecutionStep> = serde_json::from_str(&exec.result_json)
                .map(|v: serde_json::Value| {
                    v["steps"].as_array()
                        .map(|arr| arr.iter().filter_map(|s| serde_json::from_value(s.clone()).ok()).collect())
                        .unwrap_or_default()
                })
                .unwrap_or_default();

            let detail = ExecutionDetail { execution: exec, playbook_name, steps };
            (StatusCode::OK, Json(detail)).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "execution not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// Build the SOAR routes sub-router.
pub fn soar_routes(pool: SqlitePool) -> Router {
    use axum::routing::{delete, get, post, put};

    let state = SoarState { pool };

    Router::new()
        .route("/api/soar/playbooks", get(list_playbooks).post(create_playbook))
        .route("/api/soar/playbooks/{id}", get(get_playbook).put(update_playbook).delete(delete_playbook))
        .route("/api/soar/playbooks/{id}/execute", post(execute_playbook))
        .route("/api/soar/executions", get(list_executions))
        .route("/api/soar/executions/{id}", get(get_execution))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS soar_playbooks (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT NOT NULL DEFAULT '',
                trigger_type TEXT NOT NULL, trigger_conditions_json TEXT NOT NULL DEFAULT '{}',
                steps_json TEXT NOT NULL DEFAULT '[]', enabled BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS soar_executions (
                id TEXT PRIMARY KEY, playbook_id TEXT NOT NULL,
                trigger_source TEXT NOT NULL, trigger_id TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending', result_json TEXT NOT NULL DEFAULT '{}',
                started_at TEXT, finished_at TEXT
            )"
        ).execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_create_and_list_playbooks() {
        let pool = setup().await;
        let state = SoarState { pool };

        let req = CreatePlaybookRequest {
            name: "Isolate Host".into(),
            description: Some("Isolate compromised host".into()),
            trigger_type: "incident".into(),
            trigger_conditions_json: None,
            steps_json: r#"[{"type":"isolate_host"},{"type":"notify"}]"#.into(),
        };

        let resp = create_playbook(State(state.clone()), Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = list_playbooks(State(state)).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
