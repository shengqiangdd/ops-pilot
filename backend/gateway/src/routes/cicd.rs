use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Shared application state for CI/CD routes.
#[derive(Clone)]
pub struct CICDState {
    pub pool: SqlitePool,
}

// ── Pipeline Templates ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub description: Option<String>,
    pub stages_json: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PipelineTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub stages_json: String,
    pub created_at: String,
    pub updated_at: String,
}

/// GET /api/cicd/templates — list pipeline templates
pub async fn list_templates(
    State(state): State<CICDState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, PipelineTemplate>(
        "SELECT id, name, description, stages_json, created_at, updated_at FROM pipeline_templates ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(templates) => (StatusCode::OK, Json(templates)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/cicd/templates — create a pipeline template
pub async fn create_template(
    State(state): State<CICDState>,
    Json(req): Json<CreateTemplateRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let stages = req.stages_json.unwrap_or_else(|| "[]".to_string());

    let result = sqlx::query(
        "INSERT INTO pipeline_templates (id, name, description, stages_json) VALUES (?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(req.description.as_deref().unwrap_or(""))
    .bind(&stages)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let template = sqlx::query_as::<_, PipelineTemplate>(
                "SELECT id, name, description, stages_json, created_at, updated_at FROM pipeline_templates WHERE id = ?"
            )
            .bind(&id)
            .fetch_one(&state.pool)
            .await;
            match template {
                Ok(t) => (StatusCode::CREATED, Json(t)).into_response(),
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

/// GET /api/cicd/templates/:id — get template detail
pub async fn get_template(
    Path(template_id): Path<String>,
    State(state): State<CICDState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, PipelineTemplate>(
        "SELECT id, name, description, stages_json, created_at, updated_at FROM pipeline_templates WHERE id = ?"
    )
    .bind(&template_id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(t)) => (StatusCode::OK, Json(t)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "template not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// DELETE /api/cicd/templates/:id — delete template
pub async fn delete_template(
    Path(template_id): Path<String>,
    State(state): State<CICDState>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM pipeline_templates WHERE id = ?")
        .bind(&template_id)
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

// ── Pipeline Runs ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateRunRequest {
    pub template_id: String,
    pub name: Option<String>,
    pub branch: Option<String>,
    pub commit_sha: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PipelineRun {
    pub id: String,
    pub template_id: String,
    pub name: String,
    pub status: String,
    pub triggered_by: String,
    pub branch: String,
    pub commit_sha: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct PipelineRunDetail {
    pub run: PipelineRun,
    pub stages: Vec<PipelineStageRun>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PipelineStageRun {
    pub id: String,
    pub run_id: String,
    pub stage_name: String,
    pub status: String,
    pub log: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

/// POST /api/cicd/runs — trigger a pipeline run
pub async fn create_run(
    State(state): State<CICDState>,
    Json(req): Json<CreateRunRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let name = req.name.unwrap_or_else(|| format!("run-{}", &id[..8]));

    let result = sqlx::query(
        "INSERT INTO pipeline_runs (id, template_id, name, branch, commit_sha) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.template_id)
    .bind(&name)
    .bind(req.branch.as_deref().unwrap_or("main"))
    .bind(req.commit_sha.as_deref().unwrap_or(""))
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            // Create initial stage runs based on template
            let template = sqlx::query_as::<_, PipelineTemplate>(
                "SELECT id, name, description, stages_json, created_at, updated_at FROM pipeline_templates WHERE id = ?"
            )
            .bind(&req.template_id)
            .fetch_optional(&state.pool)
            .await;

            if let Ok(Some(t)) = template {
                if let Ok(stages) = serde_json::from_str::<Vec<String>>(&t.stages_json) {
                    for stage_name in stages {
                        let stage_id = uuid::Uuid::new_v4().to_string();
                        let _ = sqlx::query(
                            "INSERT INTO pipeline_stage_runs (id, run_id, stage_name, status) VALUES (?, ?, ?, 'pending')"
                        )
                        .bind(&stage_id)
                        .bind(&id)
                        .bind(&stage_name)
                        .execute(&state.pool)
                        .await;
                    }
                }
            }

            let run = sqlx::query_as::<_, PipelineRun>(
                "SELECT id, template_id, name, status, triggered_by, branch, commit_sha, started_at, finished_at, duration_ms, created_at FROM pipeline_runs WHERE id = ?"
            )
            .bind(&id)
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

/// GET /api/cicd/runs — list pipeline runs
pub async fn list_runs(
    State(state): State<CICDState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, PipelineRun>(
        "SELECT id, template_id, name, status, triggered_by, branch, commit_sha, started_at, finished_at, duration_ms, created_at FROM pipeline_runs ORDER BY created_at DESC LIMIT 100"
    )
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

/// GET /api/cicd/runs/:id — get run detail with stages
pub async fn get_run(
    Path(run_id): Path<String>,
    State(state): State<CICDState>,
) -> impl IntoResponse {
    let run = sqlx::query_as::<_, PipelineRun>(
        "SELECT id, template_id, name, status, triggered_by, branch, commit_sha, started_at, finished_at, duration_ms, created_at FROM pipeline_runs WHERE id = ?"
    )
    .bind(&run_id)
    .fetch_optional(&state.pool)
    .await;

    match run {
        Ok(Some(r)) => {
            let stages = sqlx::query_as::<_, PipelineStageRun>(
                "SELECT id, run_id, stage_name, status, log, started_at, finished_at FROM pipeline_stage_runs WHERE run_id = ? ORDER BY started_at ASC"
            )
            .bind(&run_id)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

            let detail = PipelineRunDetail { run: r, stages };
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

/// POST /api/cicd/runs/:id/cancel — cancel a pipeline run
pub async fn cancel_run(
    Path(run_id): Path<String>,
    State(state): State<CICDState>,
) -> impl IntoResponse {
    let result = sqlx::query(
        "UPDATE pipeline_runs SET status = 'cancelled', finished_at = datetime('now') WHERE id = ? AND status IN ('pending', 'running')"
    )
    .bind(&run_id)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "cancelled"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// ── Deployments ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateDeploymentRequest {
    pub name: String,
    pub service_id: Option<String>,
    pub environment: Option<String>,
    pub strategy: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Deployment {
    pub id: String,
    pub name: String,
    pub service_id: String,
    pub environment: String,
    pub strategy: String,
    pub status: String,
    pub version: String,
    pub config_json: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub created_at: String,
}

/// POST /api/cicd/deployments — create a deployment
pub async fn create_deployment(
    State(state): State<CICDState>,
    Json(req): Json<CreateDeploymentRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();

    let result = sqlx::query(
        "INSERT INTO deployments (id, name, service_id, environment, strategy, version) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(req.service_id.as_deref().unwrap_or(""))
    .bind(req.environment.as_deref().unwrap_or("dev"))
    .bind(req.strategy.as_deref().unwrap_or("rolling"))
    .bind(req.version.as_deref().unwrap_or("1.0.0"))
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let deployment = sqlx::query_as::<_, Deployment>(
                "SELECT id, name, service_id, environment, strategy, status, version, config_json, started_at, finished_at, created_at FROM deployments WHERE id = ?"
            )
            .bind(&id)
            .fetch_one(&state.pool)
            .await;
            match deployment {
                Ok(d) => (StatusCode::CREATED, Json(d)).into_response(),
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

/// GET /api/cicd/deployments — list deployments
pub async fn list_deployments(
    State(state): State<CICDState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Deployment>(
        "SELECT id, name, service_id, environment, strategy, status, version, config_json, started_at, finished_at, created_at FROM deployments ORDER BY created_at DESC LIMIT 100"
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(deployments) => (StatusCode::OK, Json(deployments)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// PUT /api/cicd/deployments/:id/rollback — rollback a deployment
pub async fn rollback_deployment(
    Path(deployment_id): Path<String>,
    State(state): State<CICDState>,
) -> impl IntoResponse {
    // Get the deployment to rollback
    let deployment = sqlx::query_as::<_, Deployment>(
        "SELECT id, name, service_id, environment, strategy, status, version, config_json, started_at, finished_at, created_at FROM deployments WHERE id = ?"
    )
    .bind(&deployment_id)
    .fetch_optional(&state.pool)
    .await;

    match deployment {
        Ok(Some(d)) => {
            // Create a new deployment with rollback status
            let new_id = uuid::Uuid::new_v4().to_string();
            let new_name = format!("{}-rollback", d.name);

            let result = sqlx::query(
                "INSERT INTO deployments (id, name, service_id, environment, strategy, status, version) VALUES (?, ?, ?, ?, ?, 'rollback', ?)"
            )
            .bind(&new_id)
            .bind(&new_name)
            .bind(&d.service_id)
            .bind(&d.environment)
            .bind(&d.strategy)
            .bind(&d.version)
            .execute(&state.pool)
            .await;

            match result {
                Ok(_) => {
                    let new_deployment = sqlx::query_as::<_, Deployment>(
                        "SELECT id, name, service_id, environment, strategy, status, version, config_json, started_at, finished_at, created_at FROM deployments WHERE id = ?"
                    )
                    .bind(&new_id)
                    .fetch_one(&state.pool)
                    .await;
                    match new_deployment {
                        Ok(nd) => (StatusCode::OK, Json(nd)).into_response(),
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
            Json(serde_json::json!({"error": "deployment not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Build the CI/CD routes sub-router.
pub fn cicd_routes(pool: SqlitePool) -> Router {
    use axum::routing::{get, post, put};

    let state = CICDState { pool };

    Router::new()
        .route("/api/cicd/templates", get(list_templates).post(create_template))
        .route("/api/cicd/templates/{id}", get(get_template).delete(delete_template))
        .route("/api/cicd/runs", get(list_runs).post(create_run))
        .route("/api/cicd/runs/{id}", get(get_run))
        .route("/api/cicd/runs/{id}/cancel", post(cancel_run))
        .route("/api/cicd/deployments", get(list_deployments).post(create_deployment))
        .route("/api/cicd/deployments/{id}/rollback", put(rollback_deployment))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS pipeline_templates (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT NOT NULL DEFAULT '',
                stages_json TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS pipeline_runs (
                id TEXT PRIMARY KEY, template_id TEXT NOT NULL, name TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending', triggered_by TEXT NOT NULL DEFAULT '',
                branch TEXT NOT NULL DEFAULT 'main', commit_sha TEXT NOT NULL DEFAULT '',
                started_at TEXT, finished_at TEXT, duration_ms INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS pipeline_stage_runs (
                id TEXT PRIMARY KEY, run_id TEXT NOT NULL, stage_name TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending', log TEXT NOT NULL DEFAULT '',
                started_at TEXT, finished_at TEXT
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS deployments (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, service_id TEXT NOT NULL DEFAULT '',
                environment TEXT NOT NULL DEFAULT 'dev', strategy TEXT NOT NULL DEFAULT 'rolling',
                status TEXT NOT NULL DEFAULT 'pending', version TEXT NOT NULL DEFAULT '',
                config_json TEXT NOT NULL DEFAULT '{}',
                started_at TEXT, finished_at TEXT, created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_create_and_list_templates() {
        let pool = setup().await;
        let state = CICDState { pool };

        let req = CreateTemplateRequest {
            name: "Build & Deploy".into(),
            description: Some("CI/CD pipeline".into()),
            stages_json: Some(r#"["build","test","deploy"]"#.into()),
        };

        let resp = create_template(State(state.clone()), Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = list_templates(State(state)).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
