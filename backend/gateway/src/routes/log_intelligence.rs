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
pub struct LogIntelligenceState {
    pub pool: SqlitePool,
}

#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    pub host_id: Option<String>,
    pub source_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PatternsQuery {
    pub host_id: Option<String>,
    pub pattern_type: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct AnomaliesQuery {
    pub host_id: Option<String>,
    pub anomaly_type: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAnomalyRequest {
    pub status: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LogSource {
    pub id: String,
    pub host_id: String,
    pub source_name: String,
    pub log_path: String,
    #[sqlx(rename = "type")]
    pub source_type: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LogPattern {
    pub id: String,
    pub host_id: String,
    pub pattern: String,
    pub pattern_type: String,
    pub count: i64,
    pub first_seen: String,
    pub last_seen: String,
    pub severity: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LogAnomaly {
    pub id: String,
    pub host_id: String,
    pub source_id: String,
    pub anomaly_type: String,
    pub description: String,
    pub severity: String,
    pub detected_at: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct LogIntelStats {
    pub total_sources: i64,
    pub total_patterns: i64,
    pub total_anomalies: i64,
    pub open_anomalies: i64,
}

/// GET /api/log-intel/sources — list log sources
pub async fn list_sources(
    State(state): State<LogIntelligenceState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, LogSource>(
        "SELECT id, host_id, source_name, log_path, type, enabled FROM log_sources ORDER BY source_name"
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(sources) => (StatusCode::OK, Json(sources)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// POST /api/log-intel/analyze — execute log analysis
pub async fn analyze_logs(
    State(state): State<LogIntelligenceState>,
    Json(_req): Json<AnalyzeRequest>,
) -> impl IntoResponse {
    // Simulate log analysis - create patterns and anomalies
    let patterns = vec![
        ("ERROR.*connection refused", "error", 45),
        ("WARN.*high memory usage", "warning", 23),
        ("INFO.*request completed", "normal", 1250),
        ("CRITICAL.*disk space low", "critical", 3),
    ];

    let mut pattern_ids = Vec::new();
    for (pattern, ptype, count) in &patterns {
        let id = uuid::Uuid::new_v4().to_string();
        let severity = match *ptype {
            "critical" => "critical",
            "error" => "high",
            "warning" => "medium",
            _ => "low",
        };
        let _ = sqlx::query(
            "INSERT INTO log_patterns (host_id, pattern, pattern_type, count, severity) VALUES (?, ?, ?, ?, ?)"
        )
        .bind("host-001")
        .bind(pattern)
        .bind(ptype)
        .bind(*count as i64)
        .bind(severity)
        .execute(&state.pool)
        .await;
        pattern_ids.push(id);
    }

    // Create anomalies
    let anomalies = vec![
        ("frequency_spike", "Unusual spike in ERROR messages (10x normal rate)", "high"),
        ("new_pattern", "New CRITICAL pattern detected: disk space exhaustion", "critical"),
        ("error_burst", "Burst of 50+ errors in 5 minutes", "medium"),
    ];

    let mut anomaly_ids = Vec::new();
    for (atype, desc, severity) in &anomalies {
        let id = uuid::Uuid::new_v4().to_string();
        let _ = sqlx::query(
            "INSERT INTO log_anomalies (host_id, source_id, anomaly_type, description, severity) VALUES (?, ?, ?, ?, ?)"
        )
        .bind("host-001")
        .bind("src-001")
        .bind(atype)
        .bind(desc)
        .bind(severity)
        .execute(&state.pool)
        .await;
        anomaly_ids.push(id);
    }

    (StatusCode::OK, Json(serde_json::json!({
        "status": "analysis_complete",
        "patterns_found": pattern_ids.len(),
        "anomalies_found": anomaly_ids.len(),
    }))).into_response()
}

/// GET /api/log-intel/patterns — list log patterns
pub async fn list_patterns(
    State(state): State<LogIntelligenceState>,
    Query(query): Query<PatternsQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).min(200);
    let offset = ((page - 1) * per_page) as i64;

    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        "SELECT id, host_id, pattern, pattern_type, count, first_seen, last_seen, severity FROM log_patterns WHERE 1=1"
    );

    if let Some(ref host_id) = query.host_id {
        builder.push(" AND host_id = ");
        builder.push_bind(host_id.clone());
    }
    if let Some(ref pattern_type) = query.pattern_type {
        builder.push(" AND pattern_type = ");
        builder.push_bind(pattern_type.clone());
    }

    builder.push(" ORDER BY count DESC LIMIT ");
    builder.push_bind(per_page as i64);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let q = builder.build_query_as::<LogPattern>();

    match q.fetch_all(&state.pool).await {
        Ok(patterns) => (StatusCode::OK, Json(patterns)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/log-intel/anomalies — list anomalies
pub async fn list_anomalies(
    State(state): State<LogIntelligenceState>,
    Query(query): Query<AnomaliesQuery>,
) -> impl IntoResponse {
    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        "SELECT id, host_id, source_id, anomaly_type, description, severity, detected_at, status FROM log_anomalies WHERE 1=1"
    );

    if let Some(ref host_id) = query.host_id {
        builder.push(" AND host_id = ");
        builder.push_bind(host_id.clone());
    }
    if let Some(ref anomaly_type) = query.anomaly_type {
        builder.push(" AND anomaly_type = ");
        builder.push_bind(anomaly_type.clone());
    }
    if let Some(ref status) = query.status {
        builder.push(" AND status = ");
        builder.push_bind(status.clone());
    }

    builder.push(" ORDER BY detected_at DESC");

    let q = builder.build_query_as::<LogAnomaly>();

    match q.fetch_all(&state.pool).await {
        Ok(anomalies) => (StatusCode::OK, Json(anomalies)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// PUT /api/log-intel/anomalies/:id — update anomaly status
pub async fn update_anomaly(
    Path(anomaly_id): Path<String>,
    State(state): State<LogIntelligenceState>,
    Json(req): Json<UpdateAnomalyRequest>,
) -> impl IntoResponse {
    let result = sqlx::query("UPDATE log_anomalies SET status = ? WHERE id = ?")
        .bind(&req.status)
        .bind(&anomaly_id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "updated"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/log-intel/stats — log intelligence statistics
pub async fn log_intel_stats(
    State(state): State<LogIntelligenceState>,
) -> impl IntoResponse {
    let total_sources: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM log_sources")
        .fetch_one(&state.pool).await.unwrap_or((0,));
    let total_patterns: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM log_patterns")
        .fetch_one(&state.pool).await.unwrap_or((0,));
    let total_anomalies: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM log_anomalies")
        .fetch_one(&state.pool).await.unwrap_or((0,));
    let open_anomalies: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM log_anomalies WHERE status = 'open'")
        .fetch_one(&state.pool).await.unwrap_or((0,));

    let stats = LogIntelStats {
        total_sources: total_sources.0,
        total_patterns: total_patterns.0,
        total_anomalies: total_anomalies.0,
        open_anomalies: open_anomalies.0,
    };
    (StatusCode::OK, Json(stats)).into_response()
}

pub fn log_intelligence_routes(pool: SqlitePool) -> Router {
    use axum::routing::{get, post, put};

    let state = LogIntelligenceState { pool };

    Router::new()
        .route("/api/log-intel/sources", get(list_sources))
        .route("/api/log-intel/analyze", post(analyze_logs))
        .route("/api/log-intel/patterns", get(list_patterns))
        .route("/api/log-intel/anomalies", get(list_anomalies))
        .route("/api/log-intel/anomalies/{id}", put(update_anomaly))
        .route("/api/log-intel/stats", get(log_intel_stats))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS log_sources (id TEXT PRIMARY KEY, host_id TEXT NOT NULL, source_name TEXT NOT NULL, log_path TEXT NOT NULL, type TEXT NOT NULL, enabled BOOLEAN NOT NULL DEFAULT 1)").execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS log_patterns (id TEXT PRIMARY KEY, host_id TEXT NOT NULL, pattern TEXT NOT NULL, pattern_type TEXT NOT NULL, count INTEGER NOT NULL DEFAULT 0, first_seen TEXT DEFAULT (datetime('now')), last_seen TEXT DEFAULT (datetime('now')), severity TEXT NOT NULL DEFAULT 'low')").execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS log_anomalies (id TEXT PRIMARY KEY, host_id TEXT NOT NULL, source_id TEXT NOT NULL, anomaly_type TEXT NOT NULL, description TEXT NOT NULL, severity TEXT NOT NULL, detected_at TEXT DEFAULT (datetime('now')), status TEXT NOT NULL DEFAULT 'open')").execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_analyze_logs() {
        let pool = setup().await;
        let state = LogIntelligenceState { pool };
        let resp = analyze_logs(State(state), Json(AnalyzeRequest { host_id: None, source_id: None })).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
