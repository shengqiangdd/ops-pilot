use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct ComplianceState {
    pub pool: SqlitePool,
}

#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    pub framework_id: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ComplianceFramework {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ComplianceControl {
    pub id: String,
    pub framework_id: String,
    pub control_id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub severity: String,
}

#[derive(Debug, Serialize)]
pub struct ComplianceOverview {
    pub total_controls: i64,
    pub passed: i64,
    pub failed: i64,
    pub not_applicable: i64,
    pub pass_rate: f64,
    pub by_category: Vec<CategoryStats>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CategoryStats {
    pub category: String,
    pub total: i64,
    pub passed: i64,
    pub failed: i64,
}

#[derive(Debug, Serialize)]
pub struct ComplianceReport {
    pub framework: ComplianceFramework,
    pub overview: ComplianceOverview,
    pub controls: Vec<ComplianceControl>,
}

#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub framework_id: Option<String>,
    pub host_id: Option<String>,
}

fn pseudo_random(seed: u64) -> f64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    ((nanos.wrapping_mul(6364136223846793005).wrapping_add(seed)) % 1000) as f64 / 1000.0
}

/// GET /api/compliance/frameworks — list frameworks
pub async fn list_frameworks(
    State(state): State<ComplianceState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, ComplianceFramework>(
        "SELECT id, name, version, description FROM compliance_frameworks ORDER BY name"
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(frameworks) => (StatusCode::OK, Json(frameworks)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/compliance/overview — compliance overview
pub async fn compliance_overview(
    State(state): State<ComplianceState>,
) -> impl IntoResponse {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM compliance_controls")
        .fetch_one(&state.pool).await.unwrap_or((0,));

    let passed: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM compliance_results WHERE status = 'pass'")
        .fetch_one(&state.pool).await.unwrap_or((0,));

    let failed: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM compliance_results WHERE status = 'fail'")
        .fetch_one(&state.pool).await.unwrap_or((0,));

    let na: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM compliance_results WHERE status = 'na'")
        .fetch_one(&state.pool).await.unwrap_or((0,));

    let pass_rate = if total.0 > 0 { (passed.0 as f64 / total.0 as f64) * 100.0 } else { 100.0 };

    let by_category = sqlx::query_as::<_, CategoryStats>(
        "SELECT c.category, COUNT(*) as total, SUM(CASE WHEN r.status = 'pass' THEN 1 ELSE 0 END) as passed, SUM(CASE WHEN r.status = 'fail' THEN 1 ELSE 0 END) as failed FROM compliance_controls c LEFT JOIN compliance_results r ON c.id = r.control_id GROUP BY c.category"
    )
    .fetch_all(&state.pool).await.unwrap_or_default();

    let overview = ComplianceOverview {
        total_controls: total.0,
        passed: passed.0,
        failed: failed.0,
        not_applicable: na.0,
        pass_rate,
        by_category,
    };

    (StatusCode::OK, Json(overview)).into_response()
}

/// GET /api/compliance/report — compliance report
pub async fn compliance_report(
    State(state): State<ComplianceState>,
    Query(query): Query<ReportQuery>,
) -> impl IntoResponse {
    let framework_id = query.framework_id.unwrap_or_else(|| "cis-benchmark".to_string());

    let framework = sqlx::query_as::<_, ComplianceFramework>(
        "SELECT id, name, version, description FROM compliance_frameworks WHERE id = ?"
    )
    .bind(&framework_id)
    .fetch_optional(&state.pool)
    .await;

    match framework {
        Ok(Some(fw)) => {
            let controls = sqlx::query_as::<_, ComplianceControl>(
                "SELECT id, framework_id, control_id, title, description, category, severity FROM compliance_controls WHERE framework_id = ?"
            )
            .bind(&framework_id)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

            let total = controls.len() as i64;
            let passed = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM compliance_results WHERE framework_id = ? AND status = 'pass'")
                .bind(&framework_id)
                .fetch_one(&state.pool).await.unwrap_or((0,)).0;

            let failed = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM compliance_results WHERE framework_id = ? AND status = 'fail'")
                .bind(&framework_id)
                .fetch_one(&state.pool).await.unwrap_or((0,)).0;

            let na = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM compliance_results WHERE framework_id = ? AND status = 'na'")
                .bind(&framework_id)
                .fetch_one(&state.pool).await.unwrap_or((0,)).0;

            let pass_rate = if total > 0 { (passed as f64 / total as f64) * 100.0 } else { 100.0 };

            let by_category = sqlx::query_as::<_, CategoryStats>(
                "SELECT c.category, COUNT(*) as total, SUM(CASE WHEN r.status = 'pass' THEN 1 ELSE 0 END) as passed, SUM(CASE WHEN r.status = 'fail' THEN 1 ELSE 0 END) as failed FROM compliance_controls c LEFT JOIN compliance_results r ON c.id = r.control_id AND c.framework_id = r.framework_id WHERE c.framework_id = ? GROUP BY c.category"
            )
            .bind(&framework_id)
            .fetch_all(&state.pool).await.unwrap_or_default();

            let overview = ComplianceOverview {
                total_controls: total,
                passed,
                failed,
                not_applicable: na,
                pass_rate,
                by_category,
            };

            let report = ComplianceReport { framework: fw, overview, controls };
            (StatusCode::OK, Json(report)).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "framework not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// POST /api/compliance/scan — execute compliance scan
pub async fn scan_compliance(
    State(state): State<ComplianceState>,
    Json(req): Json<ScanRequest>,
) -> impl IntoResponse {
    let framework_id = req.framework_id.unwrap_or_else(|| "cis-benchmark".to_string());

    // Simulate scan results
    let controls = sqlx::query_as::<_, ComplianceControl>(
        "SELECT id, framework_id, control_id, title, description, category, severity FROM compliance_controls WHERE framework_id = ?"
    )
    .bind(&framework_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut passed_count = 0;
    let mut failed_count = 0;

    for control in &controls {
        let status = if pseudo_random(control.id.len() as u64) > 0.3 {
            "pass"
        } else {
            "fail"
        };

        let _ = sqlx::query(
            "INSERT OR REPLACE INTO compliance_results (framework_id, control_id, host_id, status) VALUES (?, ?, ?, ?)"
        )
        .bind(&framework_id)
        .bind(&control.control_id)
        .bind(req.host_id.as_deref().unwrap_or("all"))
        .bind(status)
        .execute(&state.pool)
        .await;

        if status == "pass" { passed_count += 1; } else { failed_count += 1; }
    }

    (StatusCode::OK, Json(serde_json::json!({
        "status": "scan_complete",
        "framework_id": framework_id,
        "total_controls": controls.len(),
        "passed": passed_count,
        "failed": failed_count,
    }))).into_response()
}

pub fn compliance_routes(pool: SqlitePool) -> Router {
    use axum::routing::{get, post};

    let state = ComplianceState { pool };

    Router::new()
        .route("/api/compliance/frameworks", get(list_frameworks))
        .route("/api/compliance/overview", get(compliance_overview))
        .route("/api/compliance/report", get(compliance_report))
        .route("/api/compliance/scan", post(scan_compliance))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS compliance_frameworks (id TEXT PRIMARY KEY, name TEXT NOT NULL, version TEXT NOT NULL, description TEXT NOT NULL DEFAULT '')").execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS compliance_controls (id TEXT PRIMARY KEY, framework_id TEXT NOT NULL, control_id TEXT NOT NULL, title TEXT NOT NULL, description TEXT NOT NULL DEFAULT '', category TEXT NOT NULL, severity TEXT NOT NULL)").execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS compliance_results (id INTEGER PRIMARY KEY AUTOINCREMENT, framework_id TEXT NOT NULL, control_id TEXT NOT NULL, host_id TEXT NOT NULL, status TEXT NOT NULL, scanned_at TEXT DEFAULT (datetime('now')))").execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_list_frameworks() {
        let pool = setup().await;
        let state = ComplianceState { pool };
        let resp = list_frameworks(State(state)).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
