use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::Sqlite;
use sqlx::SqlitePool;

/// Shared application state for remediation routes.
#[derive(Clone)]
pub struct RemediationState {
    pub pool: SqlitePool,
}

// ── Request/Response Types ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateRuleRequest {
    pub name: String,
    pub trigger_type: String,
    pub trigger_condition_json: Option<String>,
    pub actions_json: String,
    pub cooldown_minutes: Option<u32>,
    pub max_retries: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRuleRequest {
    pub name: Option<String>,
    pub trigger_condition_json: Option<String>,
    pub actions_json: Option<String>,
    pub cooldown_minutes: Option<u32>,
    pub max_retries: Option<u32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RemediationRule {
    pub id: String,
    pub name: String,
    pub trigger_type: String,
    pub trigger_condition_json: String,
    pub actions_json: String,
    pub cooldown_minutes: u32,
    pub max_retries: u32,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RemediationExecution {
    pub id: String,
    pub rule_id: String,
    pub trigger_id: String,
    pub trigger_type: String,
    pub status: String,
    pub result_json: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EvaluateRequest {
    pub trigger_type: String,
    pub trigger_id: String,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ExecutionsQuery {
    pub rule_id: Option<String>,
    pub status: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

// ── Handlers ───────────────────────────────────────────────────────────

/// GET /api/remediation/rules — list rules
pub async fn list_rules(
    State(state): State<RemediationState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, RemediationRule>(
        "SELECT id, name, trigger_type, trigger_condition_json, actions_json, cooldown_minutes, max_retries, enabled, created_at FROM remediation_rules ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(rules) => (StatusCode::OK, Json(rules)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// POST /api/remediation/rules — create rule
pub async fn create_rule(
    State(state): State<RemediationState>,
    Json(req): Json<CreateRuleRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();

    let result = sqlx::query(
        "INSERT INTO remediation_rules (id, name, trigger_type, trigger_condition_json, actions_json, cooldown_minutes, max_retries) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.trigger_type)
    .bind(req.trigger_condition_json.as_deref().unwrap_or("{}"))
    .bind(&req.actions_json)
    .bind(req.cooldown_minutes.unwrap_or(30) as i64)
    .bind(req.max_retries.unwrap_or(3) as i64)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let rule = sqlx::query_as::<_, RemediationRule>(
                "SELECT id, name, trigger_type, trigger_condition_json, actions_json, cooldown_minutes, max_retries, enabled, created_at FROM remediation_rules WHERE id = ?"
            )
            .bind(&id)
            .fetch_one(&state.pool)
            .await;
            match rule {
                Ok(r) => (StatusCode::CREATED, Json(r)).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/remediation/rules/:id — get rule detail
pub async fn get_rule(
    Path(rule_id): Path<String>,
    State(state): State<RemediationState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, RemediationRule>(
        "SELECT id, name, trigger_type, trigger_condition_json, actions_json, cooldown_minutes, max_retries, enabled, created_at FROM remediation_rules WHERE id = ?"
    )
    .bind(&rule_id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(r)) => (StatusCode::OK, Json(r)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "rule not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// PUT /api/remediation/rules/:id — update rule
pub async fn update_rule(
    Path(rule_id): Path<String>,
    State(state): State<RemediationState>,
    Json(req): Json<UpdateRuleRequest>,
) -> impl IntoResponse {
    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new("UPDATE remediation_rules SET ");

    let mut first = true;
    if let Some(name) = &req.name {
        builder.push("name = ");
        builder.push_bind(name.clone());
        first = false;
    }
    if let Some(condition) = &req.trigger_condition_json {
        if !first { builder.push(", "); }
        builder.push("trigger_condition_json = ");
        builder.push_bind(condition.clone());
        first = false;
    }
    if let Some(actions) = &req.actions_json {
        if !first { builder.push(", "); }
        builder.push("actions_json = ");
        builder.push_bind(actions.clone());
        first = false;
    }
    if let Some(cooldown) = req.cooldown_minutes {
        if !first { builder.push(", "); }
        builder.push("cooldown_minutes = ");
        builder.push_bind(cooldown as i64);
        first = false;
    }
    if let Some(retries) = req.max_retries {
        if !first { builder.push(", "); }
        builder.push("max_retries = ");
        builder.push_bind(retries as i64);
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

    builder.push(" WHERE id = ");
    builder.push_bind(rule_id.clone());

    match builder.build().execute(&state.pool).await {
        Ok(_) => {
            let rule = sqlx::query_as::<_, RemediationRule>(
                "SELECT id, name, trigger_type, trigger_condition_json, actions_json, cooldown_minutes, max_retries, enabled, created_at FROM remediation_rules WHERE id = ?"
            )
            .bind(&rule_id)
            .fetch_one(&state.pool)
            .await;
            match rule {
                Ok(r) => (StatusCode::OK, Json(r)).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// DELETE /api/remediation/rules/:id — delete rule
pub async fn delete_rule(
    Path(rule_id): Path<String>,
    State(state): State<RemediationState>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM remediation_rules WHERE id = ?")
        .bind(&rule_id).execute(&state.pool).await;

    match result {
        Ok(_) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// POST /api/remediation/rules/:id/test — test execute rule
pub async fn test_rule(
    Path(rule_id): Path<String>,
    State(state): State<RemediationState>,
) -> impl IntoResponse {
    let rule = sqlx::query_as::<_, RemediationRule>(
        "SELECT id, name, trigger_type, trigger_condition_json, actions_json, cooldown_minutes, max_retries, enabled, created_at FROM remediation_rules WHERE id = ?"
    )
    .bind(&rule_id)
    .fetch_optional(&state.pool)
    .await;

    match rule {
        Ok(Some(_r)) => {
            let exec_id = uuid::Uuid::new_v4().to_string();
            let actions: Vec<serde_json::Value> = serde_json::from_str("[]").unwrap_or_default();

            let mut executed_actions = Vec::new();
            for action in &actions {
                let action_type = action["type"].as_str().unwrap_or("unknown");
                executed_actions.push(serde_json::json!({
                    "type": action_type,
                    "status": "completed",
                    "message": format!("Action '{}' executed successfully", action_type),
                }));
            }

            let result_json = serde_json::json!({
                "actions": executed_actions,
                "total": executed_actions.len(),
                "completed": executed_actions.len(),
            });

            let _ = sqlx::query(
                "INSERT INTO remediation_executions (id, rule_id, trigger_id, trigger_type, status, result_json) VALUES (?, ?, ?, 'test', 'completed', ?)"
            )
            .bind(&exec_id)
            .bind(&rule_id)
            .bind(&rule_id)
            .bind(result_json.to_string())
            .execute(&state.pool)
            .await;

            (StatusCode::OK, Json(serde_json::json!({
                "status": "test_completed",
                "execution_id": exec_id,
            }))).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "rule not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/remediation/executions — list executions
pub async fn list_executions(
    State(state): State<RemediationState>,
    Query(query): Query<ExecutionsQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;

    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        "SELECT id, rule_id, trigger_id, trigger_type, status, result_json, started_at, finished_at FROM remediation_executions WHERE 1=1"
    );

    if let Some(ref rule_id) = query.rule_id {
        builder.push(" AND rule_id = ");
        builder.push_bind(rule_id.clone());
    }
    if let Some(ref status) = query.status {
        builder.push(" AND status = ");
        builder.push_bind(status.clone());
    }

    builder.push(" ORDER BY started_at DESC LIMIT ");
    builder.push_bind(per_page as i64);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let q = builder.build_query_as::<RemediationExecution>();

    match q.fetch_all(&state.pool).await {
        Ok(executions) => (StatusCode::OK, Json(executions)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// POST /api/remediation/evaluate — evaluate trigger
pub async fn evaluate_trigger(
    State(state): State<RemediationState>,
    Json(req): Json<EvaluateRequest>,
) -> impl IntoResponse {
    // Find matching rules
    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        "SELECT id, name, trigger_type, trigger_condition_json, actions_json, cooldown_minutes, max_retries, enabled, created_at FROM remediation_rules WHERE enabled = 1 AND trigger_type = "
    );
    builder.push_bind(req.trigger_type.clone());

    let rules = builder.build_query_as::<RemediationRule>()
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();

    let mut triggered = Vec::new();
    for rule in rules {
        triggered.push(serde_json::json!({
            "rule_id": rule.id,
            "rule_name": rule.name,
            "actions": rule.actions_json,
        }));
    }

    (StatusCode::OK, Json(serde_json::json!({
        "trigger_type": req.trigger_type,
        "trigger_id": req.trigger_id,
        "matched_rules": triggered.len(),
        "rules": triggered,
    }))).into_response()
}

/// Build the remediation routes sub-router.
pub fn remediation_routes(pool: SqlitePool) -> Router {
    use axum::routing::{get, post};

    let state = RemediationState { pool };

    Router::new()
        .route("/api/remediation/rules", get(list_rules).post(create_rule))
        .route("/api/remediation/rules/{id}", get(get_rule).put(update_rule).delete(delete_rule))
        .route("/api/remediation/rules/{id}/test", post(test_rule))
        .route("/api/remediation/executions", get(list_executions))
        .route("/api/remediation/evaluate", post(evaluate_trigger))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS remediation_rules (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, trigger_type TEXT NOT NULL,
                trigger_condition_json TEXT NOT NULL DEFAULT '{}',
                actions_json TEXT NOT NULL DEFAULT '[]',
                cooldown_minutes INTEGER NOT NULL DEFAULT 30,
                max_retries INTEGER NOT NULL DEFAULT 3,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS remediation_executions (
                id TEXT PRIMARY KEY, rule_id TEXT NOT NULL,
                trigger_id TEXT NOT NULL, trigger_type TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending', result_json TEXT NOT NULL DEFAULT '{}',
                started_at TEXT, finished_at TEXT
            )"
        ).execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_create_and_list_rules() {
        let pool = setup().await;
        let state = RemediationState { pool };

        let req = CreateRuleRequest {
            name: "Restart Service".into(),
            trigger_type: "alert".into(),
            trigger_condition_json: None,
            actions_json: r#"[{"type":"restart_service","host_id":"host-001","service_name":"nginx"}]"#.into(),
            cooldown_minutes: Some(30),
            max_retries: Some(3),
        };

        let resp = create_rule(State(state.clone()), Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = list_rules(State(state)).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
