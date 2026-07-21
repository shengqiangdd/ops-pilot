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
pub struct ChangeAnalysisState {
    pub pool: SqlitePool,
}

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub change_type: Option<String>,
    pub status: Option<String>,
    pub host_id: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEventRequest {
    pub host_id: String,
    pub change_type: String,
    pub source: String,
    pub description: String,
    pub content_diff: Option<String>,
    pub proposed_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewRequest {
    pub status: String,
    pub reviewed_by: Option<String>,
    pub review_notes: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ChangeEvent {
    pub id: String,
    pub host_id: String,
    pub change_type: String,
    pub source: String,
    pub description: String,
    pub content_diff: String,
    pub risk_score: f64,
    pub risk_factors_json: String,
    pub status: String,
    pub proposed_by: String,
    pub reviewed_by: String,
    pub created_at: String,
    pub reviewed_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChangeStats {
    pub total: i64,
    pub pending: i64,
    pub approved: i64,
    pub rejected: i64,
    pub high_risk: i64,
}

fn pseudo_random(seed: u64) -> f64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    ((nanos.wrapping_mul(6364136223846793005).wrapping_add(seed)) % 1000) as f64 / 1000.0
}

/// Analyze change risk based on content and metadata
fn analyze_risk(change_type: &str, description: &str, content_diff: &str) -> (f64, Vec<String>) {
    let mut risk_score = 0.3;
    let mut factors = Vec::new();

    // High-risk keywords
    let high_risk_keywords = ["database", "password", "firewall", "certificate", "ssl", "migration", "drop table", "delete from"];
    for keyword in &high_risk_keywords {
        if description.to_lowercase().contains(keyword) || content_diff.to_lowercase().contains(keyword) {
            risk_score += 0.15;
            factors.push(format!("Contains high-risk keyword: {}", keyword));
        }
    }

    // Change type risk
    match change_type {
        "config" => risk_score += 0.1,
        "package" => risk_score += 0.15,
        "script" => risk_score += 0.2,
        "deployment" => risk_score += 0.25,
        "permission" => risk_score += 0.3,
        _ => {}
    }

    // Content diff size
    let diff_lines = content_diff.lines().count();
    if diff_lines > 100 {
        risk_score += 0.1;
        factors.push(format!("Large change: {} lines", diff_lines));
    }

    risk_score = risk_score.min(1.0);
    (risk_score, factors)
}

/// GET /api/change-analysis/events — list change events
pub async fn list_events(
    State(state): State<ChangeAnalysisState>,
    Query(query): Query<EventsQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;

    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        "SELECT id, host_id, change_type, source, description, content_diff, risk_score, risk_factors_json, status, proposed_by, reviewed_by, created_at, reviewed_at FROM change_events WHERE 1=1"
    );

    if let Some(ref change_type) = query.change_type {
        builder.push(" AND change_type = ");
        builder.push_bind(change_type.clone());
    }
    if let Some(ref status) = query.status {
        builder.push(" AND status = ");
        builder.push_bind(status.clone());
    }
    if let Some(ref host_id) = query.host_id {
        builder.push(" AND host_id = ");
        builder.push_bind(host_id.clone());
    }

    builder.push(" ORDER BY created_at DESC LIMIT ");
    builder.push_bind(per_page as i64);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let q = builder.build_query_as::<ChangeEvent>();

    match q.fetch_all(&state.pool).await {
        Ok(events) => (StatusCode::OK, Json(events)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// POST /api/change-analysis/events — create change event
pub async fn create_event(
    State(state): State<ChangeAnalysisState>,
    Json(req): Json<CreateEventRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let content_diff = req.content_diff.unwrap_or_default();
    let (risk_score, risk_factors) = analyze_risk(&req.change_type, &req.description, &content_diff);
    let risk_factors_json = serde_json::to_string(&risk_factors).unwrap_or_default();

    let result = sqlx::query(
        "INSERT INTO change_events (id, host_id, change_type, source, description, content_diff, risk_score, risk_factors_json, proposed_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.host_id)
    .bind(&req.change_type)
    .bind(&req.source)
    .bind(&req.description)
    .bind(&content_diff)
    .bind(risk_score)
    .bind(&risk_factors_json)
    .bind(req.proposed_by.as_deref().unwrap_or(""))
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let event = sqlx::query_as::<_, ChangeEvent>(
                "SELECT id, host_id, change_type, source, description, content_diff, risk_score, risk_factors_json, status, proposed_by, reviewed_by, created_at, reviewed_at FROM change_events WHERE id = ?"
            )
            .bind(&id)
            .fetch_one(&state.pool)
            .await;
            match event {
                Ok(e) => (StatusCode::CREATED, Json(e)).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/change-analysis/events/:id — get event detail
pub async fn get_event(
    Path(event_id): Path<String>,
    State(state): State<ChangeAnalysisState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, ChangeEvent>(
        "SELECT id, host_id, change_type, source, description, content_diff, risk_score, risk_factors_json, status, proposed_by, reviewed_by, created_at, reviewed_at FROM change_events WHERE id = ?"
    )
    .bind(&event_id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(e)) => (StatusCode::OK, Json(e)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "event not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// PUT /api/change-analysis/events/:id — review event
pub async fn review_event(
    Path(event_id): Path<String>,
    State(state): State<ChangeAnalysisState>,
    Json(req): Json<ReviewRequest>,
) -> impl IntoResponse {
    let result = sqlx::query(
        "UPDATE change_events SET status = ?, reviewed_by = ?, reviewed_at = datetime('now') WHERE id = ?"
    )
    .bind(&req.status)
    .bind(req.reviewed_by.as_deref().unwrap_or(""))
    .bind(&event_id)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let event = sqlx::query_as::<_, ChangeEvent>(
                "SELECT id, host_id, change_type, source, description, content_diff, risk_score, risk_factors_json, status, proposed_by, reviewed_by, created_at, reviewed_at FROM change_events WHERE id = ?"
            )
            .bind(&event_id)
            .fetch_one(&state.pool)
            .await;
            match event {
                Ok(e) => (StatusCode::OK, Json(e)).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// POST /api/change-analysis/analyze — analyze change risk
pub async fn analyze_change(
    State(_state): State<ChangeAnalysisState>,
    Json(req): Json<CreateEventRequest>,
) -> impl IntoResponse {
    let content_diff = req.content_diff.unwrap_or_default();
    let (risk_score, risk_factors) = analyze_risk(&req.change_type, &req.description, &content_diff);

    let risk_level = if risk_score > 0.7 { "high" } else if risk_score > 0.4 { "medium" } else { "low" };

    (StatusCode::OK, Json(serde_json::json!({
        "risk_score": risk_score,
        "risk_level": risk_level,
        "risk_factors": risk_factors,
        "recommendation": if risk_score > 0.7 {
            "This change has high risk. Recommend thorough review and staged deployment."
        } else if risk_score > 0.4 {
            "Moderate risk. Review changes carefully before deployment."
        } else {
            "Low risk. Standard review process recommended."
        }
    }))).into_response()
}

/// GET /api/change-analysis/stats — change statistics
pub async fn change_stats(
    State(state): State<ChangeAnalysisState>,
) -> impl IntoResponse {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM change_events")
        .fetch_one(&state.pool).await.unwrap_or((0,));
    let pending: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM change_events WHERE status = 'pending_review'")
        .fetch_one(&state.pool).await.unwrap_or((0,));
    let approved: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM change_events WHERE status = 'approved'")
        .fetch_one(&state.pool).await.unwrap_or((0,));
    let rejected: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM change_events WHERE status = 'rejected'")
        .fetch_one(&state.pool).await.unwrap_or((0,));
    let high_risk: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM change_events WHERE risk_score > 0.7")
        .fetch_one(&state.pool).await.unwrap_or((0,));

    let stats = ChangeStats {
        total: total.0, pending: pending.0, approved: approved.0,
        rejected: rejected.0, high_risk: high_risk.0,
    };
    (StatusCode::OK, Json(stats)).into_response()
}

/// GET /api/change-analysis/related-incidents/:id — related incidents
pub async fn related_incidents(
    Path(event_id): Path<String>,
    State(_state): State<ChangeAnalysisState>,
) -> impl IntoResponse {
    // Simulate related incidents
    let incidents = vec![
        serde_json::json!({"id": "inc-001", "name": "Service restart after config change", "severity": "medium", "status": "resolved"}),
        serde_json::json!({"id": "inc-002", "name": "Database connection timeout", "severity": "high", "status": "open"}),
    ];

    (StatusCode::OK, Json(serde_json::json!({
        "event_id": event_id,
        "related_incidents": incidents,
    }))).into_response()
}

pub fn change_analysis_routes(pool: SqlitePool) -> Router {
    use axum::routing::{get, post, put};

    let state = ChangeAnalysisState { pool };

    Router::new()
        .route("/api/change-analysis/events", get(list_events).post(create_event))
        .route("/api/change-analysis/events/{id}", get(get_event).put(review_event))
        .route("/api/change-analysis/analyze", post(analyze_change))
        .route("/api/change-analysis/stats", get(change_stats))
        .route("/api/change-analysis/related-incidents/{id}", get(related_incidents))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS change_events (id TEXT PRIMARY KEY, host_id TEXT NOT NULL, change_type TEXT NOT NULL, source TEXT NOT NULL, description TEXT NOT NULL, content_diff TEXT NOT NULL DEFAULT '', risk_score REAL NOT NULL DEFAULT 0.0, risk_factors_json TEXT NOT NULL DEFAULT '[]', status TEXT NOT NULL DEFAULT 'pending_review', proposed_by TEXT NOT NULL DEFAULT '', reviewed_by TEXT NOT NULL DEFAULT '', created_at TEXT NOT NULL DEFAULT (datetime('now')), reviewed_at TEXT)").execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_analyze_change() {
        let pool = setup().await;
        let state = ChangeAnalysisState { pool };
        let req = CreateEventRequest {
            host_id: "host-001".into(), change_type: "config".into(), source: "manual".into(),
            description: "Update database password".into(), content_diff: None, proposed_by: None,
        };
        let resp = analyze_change(State(state), Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
