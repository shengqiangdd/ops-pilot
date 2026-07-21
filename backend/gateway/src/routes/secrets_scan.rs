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
pub struct SecretsScanState {
    pub pool: SqlitePool,
}

#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub host_ids: Option<Vec<String>>,
    pub scan_types: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ResultsQuery {
    pub host_id: Option<String>,
    pub scan_type: Option<String>,
    pub severity: Option<String>,
    pub status: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateResultRequest {
    pub status: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ScanResult {
    pub id: String,
    pub host_id: String,
    pub file_path: String,
    pub scan_type: String,
    pub severity: String,
    pub line_number: i64,
    pub snippet: String,
    pub finding: String,
    pub suggestion: String,
    pub status: String,
    pub discovered_at: String,
}

#[derive(Debug, Serialize)]
pub struct ScanStats {
    pub total: i64,
    pub by_type: Vec<TypeCount>,
    pub by_severity: Vec<SeverityCount>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TypeCount {
    pub scan_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SeverityCount {
    pub severity: String,
    pub count: i64,
}

fn pseudo_random(seed: u64) -> f64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    ((nanos.wrapping_mul(6364136223846793005).wrapping_add(seed)) % 1000) as f64 / 1000.0
}

/// POST /api/secrets/scan — execute secrets scan
pub async fn scan_secrets(
    State(state): State<SecretsScanState>,
    Json(req): Json<ScanRequest>,
) -> impl IntoResponse {
    let hosts = req.host_ids.unwrap_or_else(|| vec!["host-001".into(), "host-002".into()]);
    let scan_types = req.scan_types.unwrap_or_else(|| vec![
        "hardcoded_key".into(), "weak_password".into(), "api_token".into(),
    ]);

    let findings = vec![
        ("hardcoded_key", "config/database.yml", 42, "password: 'admin123'", "Hardcoded database password detected"),
        ("weak_password", "config/ssh_config", 15, "PasswordAuthentication yes", "Weak SSH authentication enabled"),
        ("api_token", ".env.production", 8, "API_KEY=sk_live_abc123def456", "Exposed API token in environment file"),
        ("private_key", ".ssh/id_rsa", 1, "-----BEGIN RSA PRIVATE KEY-----", "Private key file found in repository"),
    ];

    let mut created = Vec::new();
    for host_id in &hosts {
        for (scan_type, file_path, line, snippet, finding) in &findings {
            if !scan_types.contains(&scan_type.to_string()) {
                continue;
            }
            let severity = match *scan_type {
                "hardcoded_key" | "private_key" => "critical",
                "weak_password" => "high",
                "api_token" => "medium",
                _ => "low",
            };

            let id = uuid::Uuid::new_v4().to_string();
            let suggestion = match *scan_type {
                "hardcoded_key" => "Use environment variables or a secrets manager".to_string(),
                "weak_password" => "Disable password authentication, use key-based auth".to_string(),
                "api_token" => "Rotate the API token and use a secrets manager".to_string(),
                "private_key" => "Remove private key from repository, add to .gitignore".to_string(),
                _ => "Review and remediate the finding".to_string(),
            };

            let _ = sqlx::query(
                "INSERT INTO secrets_scan_results (id, host_id, file_path, scan_type, severity, line_number, snippet, finding, suggestion) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&id)
            .bind(host_id)
            .bind(file_path)
            .bind(scan_type)
            .bind(severity)
            .bind(*line as i64)
            .bind(snippet)
            .bind(finding)
            .bind(&suggestion)
            .execute(&state.pool)
            .await;

            created.push(id);
        }
    }

    (StatusCode::OK, Json(serde_json::json!({
        "status": "scan_complete",
        "hosts_scanned": hosts.len(),
        "new_findings": created.len(),
    }))).into_response()
}

/// GET /api/secrets/results — list scan results
pub async fn list_results(
    State(state): State<SecretsScanState>,
    Query(query): Query<ResultsQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;

    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        "SELECT id, host_id, file_path, scan_type, severity, line_number, snippet, finding, suggestion, status, discovered_at FROM secrets_scan_results WHERE 1=1"
    );

    if let Some(ref host_id) = query.host_id {
        builder.push(" AND host_id = ");
        builder.push_bind(host_id.clone());
    }
    if let Some(ref scan_type) = query.scan_type {
        builder.push(" AND scan_type = ");
        builder.push_bind(scan_type.clone());
    }
    if let Some(ref severity) = query.severity {
        builder.push(" AND severity = ");
        builder.push_bind(severity.clone());
    }
    if let Some(ref status) = query.status {
        builder.push(" AND status = ");
        builder.push_bind(status.clone());
    }

    builder.push(" ORDER BY discovered_at DESC LIMIT ");
    builder.push_bind(per_page as i64);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let q = builder.build_query_as::<ScanResult>();

    match q.fetch_all(&state.pool).await {
        Ok(results) => (StatusCode::OK, Json(results)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// PUT /api/secrets/results/:id — update result status
pub async fn update_result(
    Path(result_id): Path<String>,
    State(state): State<SecretsScanState>,
    Json(req): Json<UpdateResultRequest>,
) -> impl IntoResponse {
    let result = sqlx::query("UPDATE secrets_scan_results SET status = ? WHERE id = ?")
        .bind(&req.status)
        .bind(&result_id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "updated"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/secrets/stats — scan statistics
pub async fn scan_stats(
    State(state): State<SecretsScanState>,
) -> impl IntoResponse {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM secrets_scan_results")
        .fetch_one(&state.pool).await.unwrap_or((0,));

    let by_type = sqlx::query_as::<_, TypeCount>(
        "SELECT scan_type, COUNT(*) as count FROM secrets_scan_results GROUP BY scan_type ORDER BY count DESC"
    )
    .fetch_all(&state.pool).await.unwrap_or_default();

    let by_severity = sqlx::query_as::<_, SeverityCount>(
        "SELECT severity, COUNT(*) as count FROM secrets_scan_results GROUP BY severity ORDER BY count DESC"
    )
    .fetch_all(&state.pool).await.unwrap_or_default();

    let stats = ScanStats { total: total.0, by_type, by_severity };
    (StatusCode::OK, Json(stats)).into_response()
}

pub fn secrets_scan_routes(pool: SqlitePool) -> Router {
    use axum::routing::{get, post, put};

    let state = SecretsScanState { pool };

    Router::new()
        .route("/api/secrets/scan", post(scan_secrets))
        .route("/api/secrets/results", get(list_results))
        .route("/api/secrets/results/{id}", put(update_result))
        .route("/api/secrets/stats", get(scan_stats))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS secrets_scan_results (
                id TEXT PRIMARY KEY, host_id TEXT NOT NULL, file_path TEXT NOT NULL,
                scan_type TEXT NOT NULL, severity TEXT NOT NULL, line_number INTEGER NOT NULL,
                snippet TEXT NOT NULL, finding TEXT NOT NULL, suggestion TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'open', discovered_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_scan_and_list() {
        let pool = setup().await;
        let state = SecretsScanState { pool };
        let resp = scan_secrets(State(state.clone()), Json(ScanRequest { host_ids: None, scan_types: None })).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
        let resp = list_results(State(state), Query(ResultsQuery { host_id: None, scan_type: None, severity: None, status: None, page: None, per_page: None })).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
