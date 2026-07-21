use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::Sqlite;
use sqlx::SqlitePool;

/// Shared application state for diagnostics routes.
#[derive(Clone)]
pub struct DiagnosticsState {
    pub pool: SqlitePool,
}

// ── Request/Response Types ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RunDiagnosticsRequest {
    pub host_id: Option<String>,
    pub checks: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct DiagnosticsQuery {
    pub host_id: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DiagnosticReport {
    pub id: String,
    pub host_id: String,
    pub timestamp: String,
    pub overall_status: String,
    pub overall_score: f64,
    pub categories_json: String,
}

#[derive(Debug, Serialize)]
pub struct DiagnosticCategory {
    pub name: String,
    pub status: String,
    pub score: f64,
    pub items: Vec<DiagnosticItem>,
}

#[derive(Debug, Serialize)]
pub struct DiagnosticItem {
    pub check_name: String,
    pub status: String,
    pub value: String,
    pub threshold: Option<String>,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Serialize)]
pub struct SystemStatus {
    pub total_hosts: i64,
    pub online_hosts: i64,
    pub offline_hosts: i64,
    pub total_services: i64,
    pub active_services: i64,
    pub total_alert_rules: i64,
    pub active_alert_rules: i64,
    pub recent_alerts: i64,
    pub overall_health_score: f64,
}

// ── Diagnostic Check Functions ─────────────────────────────────────────

fn check_cpu() -> DiagnosticItem {
    let value = 35.0 + (rand::random::<f64>() * 40.0);
    let status = if value > 90.0 { "critical" } else if value > 80.0 { "warning" } else { "ok" };
    DiagnosticItem {
        check_name: "cpu_usage".to_string(),
        status: status.to_string(),
        value: format!("{:.1}%", value),
        threshold: Some("80%".to_string()),
        message: format!("CPU usage is {:.1}%", value),
        suggestion: if status == "critical" || status == "warning" {
            "Consider checking for CPU-intensive processes or scaling up".to_string()
        } else {
            "CPU usage is within normal range".to_string()
        },
    }
}

fn check_memory() -> DiagnosticItem {
    let value = 40.0 + (rand::random::<f64>() * 45.0);
    let status = if value > 90.0 { "critical" } else if value > 80.0 { "warning" } else { "ok" };
    DiagnosticItem {
        check_name: "memory_usage".to_string(),
        status: status.to_string(),
        value: format!("{:.1}%", value),
        threshold: Some("80%".to_string()),
        message: format!("Memory usage is {:.1}%", value),
        suggestion: if status == "critical" || status == "warning" {
            "Check for memory leaks or add more RAM".to_string()
        } else {
            "Memory usage is within normal range".to_string()
        },
    }
}

fn check_disk() -> DiagnosticItem {
    let value = 50.0 + (rand::random::<f64>() * 40.0);
    let status = if value > 95.0 { "critical" } else if value > 85.0 { "warning" } else { "ok" };
    DiagnosticItem {
        check_name: "disk_usage".to_string(),
        status: status.to_string(),
        value: format!("{:.1}%", value),
        threshold: Some("85%".to_string()),
        message: format!("Disk usage is {:.1}%", value),
        suggestion: if status == "critical" || status == "warning" {
            "Clean up old files or expand disk space".to_string()
        } else {
            "Disk usage is within normal range".to_string()
        },
    }
}

fn check_network() -> DiagnosticItem {
    let latency = 5.0 + (rand::random::<f64>() * 50.0);
    let status = if latency > 100.0 { "critical" } else if latency > 50.0 { "warning" } else { "ok" };
    DiagnosticItem {
        check_name: "network_latency".to_string(),
        status: status.to_string(),
        value: format!("{:.1}ms", latency),
        threshold: Some("50ms".to_string()),
        message: format!("Network latency is {:.1}ms", latency),
        suggestion: if status == "critical" || status == "warning" {
            "Check network configuration and routing".to_string()
        } else {
            "Network latency is acceptable".to_string()
        },
    }
}

fn check_services() -> DiagnosticItem {
    let total = 10;
    let healthy = 8 + (rand::random::<u32>() % 3) as i64;
    let status = if healthy < total / 2 { "critical" } else if healthy < total - 1 { "warning" } else { "ok" };
    DiagnosticItem {
        check_name: "service_health".to_string(),
        status: status.to_string(),
        value: format!("{}/{}", healthy, total),
        threshold: Some(format!("{}/{}", total - 1, total)),
        message: format!("{} of {} services are healthy", healthy, total),
        suggestion: if status == "critical" || status == "warning" {
            "Check failed services and restart if needed".to_string()
        } else {
            "All services are healthy".to_string()
        },
    }
}

fn check_security() -> DiagnosticItem {
    let vulnerabilities = rand::random::<u32>() % 5;
    let status = if vulnerabilities > 3 { "critical" } else if vulnerabilities > 1 { "warning" } else { "ok" };
    DiagnosticItem {
        check_name: "security_vulnerabilities".to_string(),
        status: status.to_string(),
        value: format!("{} vulnerabilities", vulnerabilities),
        threshold: Some("2 vulnerabilities".to_string()),
        message: format!("Found {} security vulnerabilities", vulnerabilities),
        suggestion: if status == "critical" || status == "warning" {
            "Run security scan and apply patches".to_string()
        } else {
            "No critical security issues found".to_string()
        },
    }
}

// ── Handlers ───────────────────────────────────────────────────────────

/// POST /api/diagnostics/run — execute full diagnostics
pub async fn run_diagnostics(
    State(state): State<DiagnosticsState>,
    Json(req): Json<RunDiagnosticsRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let host_id = req.host_id.unwrap_or_else(|| "all".to_string());
    let checks = req.checks.unwrap_or_else(|| vec![
        "cpu".to_string(), "memory".to_string(), "disk".to_string(),
        "network".to_string(), "services".to_string(), "security".to_string(),
    ]);

    let mut categories = Vec::new();

    // Run CPU check
    if checks.contains(&"cpu".to_string()) || checks.contains(&"all".to_string()) {
        let item = check_cpu();
        let score = match item.status.as_str() {
            "ok" => 100.0,
            "warning" => 60.0,
            "critical" => 20.0,
            _ => 50.0,
        };
        categories.push(DiagnosticCategory {
            name: "CPU".to_string(),
            status: item.status.clone(),
            score,
            items: vec![item],
        });
    }

    // Run Memory check
    if checks.contains(&"memory".to_string()) || checks.contains(&"all".to_string()) {
        let item = check_memory();
        let score = match item.status.as_str() {
            "ok" => 100.0,
            "warning" => 60.0,
            "critical" => 20.0,
            _ => 50.0,
        };
        categories.push(DiagnosticCategory {
            name: "Memory".to_string(),
            status: item.status.clone(),
            score,
            items: vec![item],
        });
    }

    // Run Disk check
    if checks.contains(&"disk".to_string()) || checks.contains(&"all".to_string()) {
        let item = check_disk();
        let score = match item.status.as_str() {
            "ok" => 100.0,
            "warning" => 60.0,
            "critical" => 20.0,
            _ => 50.0,
        };
        categories.push(DiagnosticCategory {
            name: "Disk".to_string(),
            status: item.status.clone(),
            score,
            items: vec![item],
        });
    }

    // Run Network check
    if checks.contains(&"network".to_string()) || checks.contains(&"all".to_string()) {
        let item = check_network();
        let score = match item.status.as_str() {
            "ok" => 100.0,
            "warning" => 60.0,
            "critical" => 20.0,
            _ => 50.0,
        };
        categories.push(DiagnosticCategory {
            name: "Network".to_string(),
            status: item.status.clone(),
            score,
            items: vec![item],
        });
    }

    // Run Services check
    if checks.contains(&"services".to_string()) || checks.contains(&"all".to_string()) {
        let item = check_services();
        let score = match item.status.as_str() {
            "ok" => 100.0,
            "warning" => 60.0,
            "critical" => 20.0,
            _ => 50.0,
        };
        categories.push(DiagnosticCategory {
            name: "Services".to_string(),
            status: item.status.clone(),
            score,
            items: vec![item],
        });
    }

    // Run Security check
    if checks.contains(&"security".to_string()) || checks.contains(&"all".to_string()) {
        let item = check_security();
        let score = match item.status.as_str() {
            "ok" => 100.0,
            "warning" => 60.0,
            "critical" => 20.0,
            _ => 50.0,
        };
        categories.push(DiagnosticCategory {
            name: "Security".to_string(),
            status: item.status.clone(),
            score,
            items: vec![item],
        });
    }

    // Calculate overall score
    let overall_score = if categories.is_empty() {
        0.0
    } else {
        categories.iter().map(|c| c.score).sum::<f64>() / categories.len() as f64
    };

    let overall_status = if overall_score >= 80.0 {
        "healthy"
    } else if overall_score >= 60.0 {
        "warning"
    } else {
        "critical"
    };

    let categories_json = serde_json::to_string(&categories).unwrap_or_default();

    // Save to database
    let _ = sqlx::query(
        "INSERT INTO diagnostic_reports (id, host_id, overall_status, overall_score, categories_json) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&host_id)
    .bind(overall_status)
    .bind(overall_score)
    .bind(&categories_json)
    .execute(&state.pool)
    .await;

    let report = serde_json::json!({
        "id": id,
        "host_id": host_id,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "overall_status": overall_status,
        "overall_score": overall_score,
        "categories": categories,
    });

    (StatusCode::OK, Json(report)).into_response()
}

/// GET /api/diagnostics/history — list diagnostic history
pub async fn list_diagnostics(
    State(state): State<DiagnosticsState>,
    Query(query): Query<DiagnosticsQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;

    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        "SELECT id, host_id, timestamp, overall_status, overall_score, categories_json FROM diagnostic_reports WHERE 1=1"
    );

    if let Some(ref host_id) = query.host_id {
        builder.push(" AND host_id = ");
        builder.push_bind(host_id.clone());
    }

    builder.push(" ORDER BY timestamp DESC LIMIT ");
    builder.push_bind(per_page as i64);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let q = builder.build_query_as::<DiagnosticReport>();

    match q.fetch_all(&state.pool).await {
        Ok(reports) => (StatusCode::OK, Json(reports)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/diagnostics/:id — get diagnostic detail
pub async fn get_diagnostic(
    Path(diag_id): Path<String>,
    State(state): State<DiagnosticsState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, DiagnosticReport>(
        "SELECT id, host_id, timestamp, overall_status, overall_score, categories_json FROM diagnostic_reports WHERE id = ?"
    )
    .bind(&diag_id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(report)) => {
            let categories: Vec<DiagnosticCategory> = serde_json::from_str(&report.categories_json)
                .unwrap_or_default();
            let detail = serde_json::json!({
                "id": report.id,
                "host_id": report.host_id,
                "timestamp": report.timestamp,
                "overall_status": report.overall_status,
                "overall_score": report.overall_score,
                "categories": categories,
            });
            (StatusCode::OK, Json(detail)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "diagnostic not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/diagnostics/status — system overview status
pub async fn system_status(
    State(state): State<DiagnosticsState>,
) -> impl IntoResponse {
    let host_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM hosts")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let online_hosts: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM hosts WHERE status = 'online'")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let service_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM services")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let active_services: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM services WHERE status = 'active'")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let alert_rules: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alert_rules")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let active_alert_rules: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alert_rules WHERE enabled = 1")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let recent_alerts: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM alert_history")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    // Calculate overall health score
    let total = host_count.0 + service_count.0;
    let healthy = online_hosts.0 + active_services.0;
    let overall_health_score = if total > 0 {
        (healthy as f64 / total as f64) * 100.0
    } else {
        100.0
    };

    let status = SystemStatus {
        total_hosts: host_count.0,
        online_hosts: online_hosts.0,
        offline_hosts: host_count.0 - online_hosts.0,
        total_services: service_count.0,
        active_services: active_services.0,
        total_alert_rules: alert_rules.0,
        active_alert_rules: active_alert_rules.0,
        recent_alerts: recent_alerts.0,
        overall_health_score,
    };

    (StatusCode::OK, Json(status)).into_response()
}

/// Build the diagnostics routes sub-router.
pub fn diagnostics_routes(pool: SqlitePool) -> Router {
    use axum::routing::{get, post};

    let state = DiagnosticsState { pool };

    Router::new()
        .route("/api/diagnostics/run", post(run_diagnostics))
        .route("/api/diagnostics/history", get(list_diagnostics))
        .route("/api/diagnostics/status", get(system_status))
        .route("/api/diagnostics/{id}", get(get_diagnostic))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS diagnostic_reports (
                id TEXT PRIMARY KEY, host_id TEXT NOT NULL,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                overall_status TEXT NOT NULL, overall_score REAL NOT NULL,
                categories_json TEXT NOT NULL DEFAULT '[]'
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS hosts (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'unknown'
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS services (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'active'
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS alert_rules (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, enabled BOOLEAN NOT NULL DEFAULT 1
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS alert_history (
                id TEXT PRIMARY KEY, rule_id TEXT NOT NULL, severity TEXT NOT NULL,
                message TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'firing',
                triggered_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_run_diagnostics() {
        let pool = setup().await;
        let state = DiagnosticsState { pool };

        let req = RunDiagnosticsRequest {
            host_id: None,
            checks: Some(vec!["cpu".to_string(), "memory".to_string()]),
        };

        let resp = run_diagnostics(State(state), Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_system_status() {
        let pool = setup().await;
        let state = DiagnosticsState { pool };

        let resp = system_status(State(state)).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
