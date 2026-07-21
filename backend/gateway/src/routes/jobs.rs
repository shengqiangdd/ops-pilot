use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Shared application state for jobs routes.
#[derive(Clone)]
pub struct JobsState {
    pub pool: SqlitePool,
}

// ── Job Definitions ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub name: String,
    pub description: Option<String>,
    pub steps_json: String,
    pub retry_policy: Option<String>,
    pub timeout_seconds: Option<u32>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps_json: String,
    pub retry_policy: String,
    pub timeout_seconds: u32,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct JobStep {
    pub id: String,
    pub job_id: String,
    pub step_name: String,
    pub step_type: String,
    pub config_json: String,
    pub step_order: i32,
}

// ── Job Runs ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct JobRun {
    pub id: String,
    pub job_id: String,
    pub status: String,
    pub triggered_by: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub duration_ms: Option<i64>,
    pub error_message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct JobStepRun {
    pub id: String,
    pub run_id: String,
    pub step_name: String,
    pub status: String,
    pub output: String,
    pub error: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct JobRunDetail {
    pub run: JobRun,
    pub steps: Vec<JobStepRun>,
}

// ── Handlers ───────────────────────────────────────────────────────────

/// GET /api/jobs — list all jobs
pub async fn list_jobs(
    State(state): State<JobsState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Job>(
        "SELECT id, name, description, steps_json, retry_policy, timeout_seconds, enabled, created_at, updated_at FROM jobs ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(jobs) => (StatusCode::OK, Json(jobs)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/jobs — create a job
pub async fn create_job(
    State(state): State<JobsState>,
    Json(req): Json<CreateJobRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();

    let result = sqlx::query(
        "INSERT INTO jobs (id, name, description, steps_json, retry_policy, timeout_seconds) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(req.description.as_deref().unwrap_or(""))
    .bind(&req.steps_json)
    .bind(req.retry_policy.as_deref().unwrap_or("none"))
    .bind(req.timeout_seconds.unwrap_or(3600) as i64)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let job = sqlx::query_as::<_, Job>(
                "SELECT id, name, description, steps_json, retry_policy, timeout_seconds, enabled, created_at, updated_at FROM jobs WHERE id = ?"
            )
            .bind(&id)
            .fetch_one(&state.pool)
            .await;
            match job {
                Ok(j) => (StatusCode::CREATED, Json(j)).into_response(),
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

/// GET /api/jobs/:id — get job detail
pub async fn get_job(
    Path(job_id): Path<String>,
    State(state): State<JobsState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Job>(
        "SELECT id, name, description, steps_json, retry_policy, timeout_seconds, enabled, created_at, updated_at FROM jobs WHERE id = ?"
    )
    .bind(&job_id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(job)) => (StatusCode::OK, Json(job)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "job not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// DELETE /api/jobs/:id — delete a job
pub async fn delete_job(
    Path(job_id): Path<String>,
    State(state): State<JobsState>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM jobs WHERE id = ?")
        .bind(&job_id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/jobs/:id/execute — execute a job
pub async fn execute_job(
    Path(job_id): Path<String>,
    State(state): State<JobsState>,
) -> impl IntoResponse {
    // Check if job exists
    let job = sqlx::query_as::<_, Job>(
        "SELECT id, name, description, steps_json, retry_policy, timeout_seconds, enabled, created_at, updated_at FROM jobs WHERE id = ?"
    )
    .bind(&job_id)
    .fetch_optional(&state.pool)
    .await;

    match job {
        Ok(Some(_)) => {
            // Create a new run
            let run_id = uuid::Uuid::new_v4().to_string();

            let result = sqlx::query(
                "INSERT INTO job_runs (id, job_id, status) VALUES (?, ?, 'running')"
            )
            .bind(&run_id)
            .bind(&job_id)
            .execute(&state.pool)
            .await;

            match result {
                Ok(_) => {
                    let run = sqlx::query_as::<_, JobRun>(
                        "SELECT id, job_id, status, triggered_by, started_at, finished_at, duration_ms, error_message, created_at FROM job_runs WHERE id = ?"
                    )
                    .bind(&run_id)
                    .fetch_one(&state.pool)
                    .await;
                    match run {
                        Ok(r) => (StatusCode::CREATED, Json(r)).into_response(),
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
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "job not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/jobs/:id/runs — list job runs
pub async fn list_job_runs(
    Path(job_id): Path<String>,
    State(state): State<JobsState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, JobRun>(
        "SELECT id, job_id, status, triggered_by, started_at, finished_at, duration_ms, error_message, created_at FROM job_runs WHERE job_id = ? ORDER BY created_at DESC LIMIT 50"
    )
    .bind(&job_id)
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(runs) => (StatusCode::OK, Json(runs)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/jobs/runs/:run_id — get run detail with steps
pub async fn get_run_detail(
    Path(run_id): Path<String>,
    State(state): State<JobsState>,
) -> impl IntoResponse {
    let run = sqlx::query_as::<_, JobRun>(
        "SELECT id, job_id, status, triggered_by, started_at, finished_at, duration_ms, error_message, created_at FROM job_runs WHERE id = ?"
    )
    .bind(&run_id)
    .fetch_optional(&state.pool)
    .await;

    match run {
        Ok(Some(r)) => {
            let steps = sqlx::query_as::<_, JobStepRun>(
                "SELECT id, run_id, step_name, status, output, error, started_at, finished_at FROM job_step_runs WHERE run_id = ? ORDER BY started_at ASC"
            )
            .bind(&run_id)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

            let detail = JobRunDetail { run: r, steps };
            (StatusCode::OK, Json(detail)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "run not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Build the jobs routes sub-router.
pub fn jobs_routes(pool: SqlitePool) -> Router {
    use axum::routing::{get, post};

    let state = JobsState { pool };

    Router::new()
        .route("/api/jobs", get(list_jobs).post(create_job))
        .route("/api/jobs/{id}", get(get_job).delete(delete_job))
        .route("/api/jobs/{id}/execute", post(execute_job))
        .route("/api/jobs/{id}/runs", get(list_job_runs))
        .route("/api/jobs/runs/{run_id}", get(get_run_detail))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT NOT NULL DEFAULT '',
                steps_json TEXT NOT NULL DEFAULT '[]', retry_policy TEXT NOT NULL DEFAULT 'none',
                timeout_seconds INTEGER NOT NULL DEFAULT 3600, enabled BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS job_runs (
                id TEXT PRIMARY KEY, job_id TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'pending',
                triggered_by TEXT NOT NULL DEFAULT '', started_at TEXT, finished_at TEXT,
                duration_ms INTEGER, error_message TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS job_step_runs (
                id TEXT PRIMARY KEY, run_id TEXT NOT NULL, step_name TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending', output TEXT NOT NULL DEFAULT '',
                error TEXT, started_at TEXT, finished_at TEXT
            )"
        ).execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_create_and_list_jobs() {
        let pool = setup().await;
        let state = JobsState { pool };

        let req = CreateJobRequest {
            name: "Deploy App".into(),
            description: Some("Deploy application".into()),
            steps_json: r#"[{"type":"ssh_command","command":"deploy.sh"}]"#.into(),
            retry_policy: Some("on_failure".into()),
            timeout_seconds: Some(300),
        };

        let resp = create_job(State(state.clone()), Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = list_jobs(State(state)).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
