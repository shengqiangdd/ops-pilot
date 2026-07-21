use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::Sqlite;
use sqlx::SqlitePool;

/// Shared application state for vulnerabilities routes.
#[derive(Clone)]
pub struct VulnerabilitiesState {
    pub pool: SqlitePool,
}

// ── Request/Response Types ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct VulnerabilitiesQuery {
    pub severity: Option<String>,
    pub status: Option<String>,
    pub host_id: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVulnerabilityRequest {
    pub cve_id: String,
    pub title: String,
    pub description: Option<String>,
    pub severity: String,
    pub cvss_score: Option<f64>,
    pub affected_host: Option<String>,
    pub affected_service: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVulnerabilityRequest {
    pub status: Option<String>,
    pub assigned_to: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Vulnerability {
    pub id: String,
    pub cve_id: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub cvss_score: f64,
    pub affected_host: String,
    pub affected_service: String,
    pub status: String,
    pub discovered_at: String,
    pub assigned_to: String,
    pub fixed_at: Option<String>,
    pub notes: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct VulnerabilityStats {
    pub total: i64,
    pub critical: i64,
    pub high: i64,
    pub medium: i64,
    pub low: i64,
    pub open: i64,
    pub in_progress: i64,
    pub fixed: i64,
}

// ── Handlers ───────────────────────────────────────────────────────────

/// GET /api/vulnerabilities — list vulnerabilities
pub async fn list_vulnerabilities(
    State(state): State<VulnerabilitiesState>,
    Query(query): Query<VulnerabilitiesQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;

    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        "SELECT id, cve_id, title, description, severity, cvss_score, affected_host, affected_service, status, discovered_at, assigned_to, fixed_at, notes, created_at FROM vulnerabilities WHERE 1=1"
    );

    if let Some(ref severity) = query.severity {
        builder.push(" AND severity = ");
        builder.push_bind(severity.clone());
    }
    if let Some(ref status) = query.status {
        builder.push(" AND status = ");
        builder.push_bind(status.clone());
    }
    if let Some(ref host_id) = query.host_id {
        builder.push(" AND affected_host = ");
        builder.push_bind(host_id.clone());
    }

    builder.push(" ORDER BY discovered_at DESC LIMIT ");
    builder.push_bind(per_page as i64);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let q = builder.build_query_as::<Vulnerability>();

    match q.fetch_all(&state.pool).await {
        Ok(vulns) => (StatusCode::OK, Json(vulns)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/vulnerabilities — create vulnerability
pub async fn create_vulnerability(
    State(state): State<VulnerabilitiesState>,
    Json(req): Json<CreateVulnerabilityRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();

    let result = sqlx::query(
        "INSERT INTO vulnerabilities (id, cve_id, title, description, severity, cvss_score, affected_host, affected_service, notes) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.cve_id)
    .bind(&req.title)
    .bind(req.description.as_deref().unwrap_or(""))
    .bind(&req.severity)
    .bind(req.cvss_score.unwrap_or(0.0))
    .bind(req.affected_host.as_deref().unwrap_or(""))
    .bind(req.affected_service.as_deref().unwrap_or(""))
    .bind(req.notes.as_deref().unwrap_or(""))
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let vuln = sqlx::query_as::<_, Vulnerability>(
                "SELECT id, cve_id, title, description, severity, cvss_score, affected_host, affected_service, status, discovered_at, assigned_to, fixed_at, notes, created_at FROM vulnerabilities WHERE id = ?"
            )
            .bind(&id)
            .fetch_one(&state.pool)
            .await;
            match vuln {
                Ok(v) => (StatusCode::CREATED, Json(v)).into_response(),
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

/// GET /api/vulnerabilities/:id — get vulnerability detail
pub async fn get_vulnerability(
    Path(vuln_id): Path<String>,
    State(state): State<VulnerabilitiesState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Vulnerability>(
        "SELECT id, cve_id, title, description, severity, cvss_score, affected_host, affected_service, status, discovered_at, assigned_to, fixed_at, notes, created_at FROM vulnerabilities WHERE id = ?"
    )
    .bind(&vuln_id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(v)) => (StatusCode::OK, Json(v)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "vulnerability not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// PUT /api/vulnerabilities/:id — update vulnerability
pub async fn update_vulnerability(
    Path(vuln_id): Path<String>,
    State(state): State<VulnerabilitiesState>,
    Json(req): Json<UpdateVulnerabilityRequest>,
) -> impl IntoResponse {
    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new("UPDATE vulnerabilities SET ");

    let mut first = true;
    if let Some(status) = &req.status {
        builder.push("status = ");
        builder.push_bind(status.clone());
        first = false;
        if status == "fixed" {
            builder.push(", fixed_at = datetime('now')");
        }
    }
    if let Some(assigned_to) = &req.assigned_to {
        if !first { builder.push(", "); }
        builder.push("assigned_to = ");
        builder.push_bind(assigned_to.clone());
        first = false;
    }
    if let Some(notes) = &req.notes {
        if !first { builder.push(", "); }
        builder.push("notes = ");
        builder.push_bind(notes.clone());
        first = false;
    }

    if first {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "no fields to update"})),
        )
            .into_response();
    }

    builder.push(" WHERE id = ");
    builder.push_bind(vuln_id.clone());

    match builder.build().execute(&state.pool).await {
        Ok(_) => {
            let vuln = sqlx::query_as::<_, Vulnerability>(
                "SELECT id, cve_id, title, description, severity, cvss_score, affected_host, affected_service, status, discovered_at, assigned_to, fixed_at, notes, created_at FROM vulnerabilities WHERE id = ?"
            )
            .bind(&vuln_id)
            .fetch_one(&state.pool)
            .await;
            match vuln {
                Ok(v) => (StatusCode::OK, Json(v)).into_response(),
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

/// DELETE /api/vulnerabilities/:id — delete vulnerability
pub async fn delete_vulnerability(
    Path(vuln_id): Path<String>,
    State(state): State<VulnerabilitiesState>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM vulnerabilities WHERE id = ?")
        .bind(&vuln_id)
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

/// POST /api/vulnerabilities/:id/verify — verify vulnerability fix
pub async fn verify_vulnerability(
    Path(vuln_id): Path<String>,
    State(state): State<VulnerabilitiesState>,
) -> impl IntoResponse {
    let result = sqlx::query(
        "UPDATE vulnerabilities SET status = 'verified', fixed_at = datetime('now') WHERE id = ?"
    )
    .bind(&vuln_id)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "verified"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/vulnerabilities/stats — vulnerability statistics
pub async fn vuln_stats(
    State(state): State<VulnerabilitiesState>,
) -> impl IntoResponse {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vulnerabilities")
        .fetch_one(&state.pool).await.unwrap_or((0,));
    let critical: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vulnerabilities WHERE severity = 'critical'")
        .fetch_one(&state.pool).await.unwrap_or((0,));
    let high: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vulnerabilities WHERE severity = 'high'")
        .fetch_one(&state.pool).await.unwrap_or((0,));
    let medium: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vulnerabilities WHERE severity = 'medium'")
        .fetch_one(&state.pool).await.unwrap_or((0,));
    let low: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vulnerabilities WHERE severity = 'low'")
        .fetch_one(&state.pool).await.unwrap_or((0,));
    let open: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vulnerabilities WHERE status = 'open'")
        .fetch_one(&state.pool).await.unwrap_or((0,));
    let in_progress: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vulnerabilities WHERE status = 'in_progress'")
        .fetch_one(&state.pool).await.unwrap_or((0,));
    let fixed: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vulnerabilities WHERE status IN ('fixed', 'verified')")
        .fetch_one(&state.pool).await.unwrap_or((0,));

    let stats = VulnerabilityStats {
        total: total.0,
        critical: critical.0,
        high: high.0,
        medium: medium.0,
        low: low.0,
        open: open.0,
        in_progress: in_progress.0,
        fixed: fixed.0,
    };

    (StatusCode::OK, Json(stats)).into_response()
}

/// POST /api/vulnerabilities/scan — simulate vulnerability scan
pub async fn scan_vulnerabilities(
    State(state): State<VulnerabilitiesState>,
) -> impl IntoResponse {
    // Simulate finding some vulnerabilities
    let findings = vec![
        ("CVE-2026-1234", "Remote Code Execution in OpenSSL", "critical", 9.8),
        ("CVE-2026-5678", "SQL Injection in Web Application", "high", 8.5),
        ("CVE-2026-9012", "Cross-Site Scripting in Login Form", "medium", 6.1),
        ("CVE-2026-3456", "Information Disclosure in API", "low", 3.7),
    ];

    let mut created = Vec::new();
    for (cve_id, title, severity, cvss) in findings {
        let id = uuid::Uuid::new_v4().to_string();
        let _ = sqlx::query(
            "INSERT INTO vulnerabilities (id, cve_id, title, severity, cvss_score, affected_host, affected_service) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(cve_id)
        .bind(title)
        .bind(severity)
        .bind(cvss)
        .bind("scanned-host")
        .bind("web-application")
        .execute(&state.pool)
        .await;
        created.push(id);
    }

    (StatusCode::OK, Json(serde_json::json!({
        "status": "scan_complete",
        "new_vulnerabilities": created.len(),
        "vulnerability_ids": created,
    }))).into_response()
}

/// Build the vulnerabilities routes sub-router.
pub fn vulnerabilities_routes(pool: SqlitePool) -> Router {
    use axum::routing::{delete, get, post, put};

    let state = VulnerabilitiesState { pool };

    Router::new()
        .route("/api/vulnerabilities", get(list_vulnerabilities).post(create_vulnerability))
        .route("/api/vulnerabilities/stats", get(vuln_stats))
        .route("/api/vulnerabilities/scan", post(scan_vulnerabilities))
        .route("/api/vulnerabilities/{id}", get(get_vulnerability).put(update_vulnerability).delete(delete_vulnerability))
        .route("/api/vulnerabilities/{id}/verify", post(verify_vulnerability))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS vulnerabilities (
                id TEXT PRIMARY KEY, cve_id TEXT NOT NULL, title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '', severity TEXT NOT NULL,
                cvss_score REAL NOT NULL DEFAULT 0.0, affected_host TEXT NOT NULL DEFAULT '',
                affected_service TEXT NOT NULL DEFAULT '', status TEXT NOT NULL DEFAULT 'open',
                discovered_at TEXT NOT NULL DEFAULT (datetime('now')),
                assigned_to TEXT NOT NULL DEFAULT '', fixed_at TEXT,
                notes TEXT NOT NULL DEFAULT '', created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_create_and_list_vulnerabilities() {
        let pool = setup().await;
        let state = VulnerabilitiesState { pool };

        let req = CreateVulnerabilityRequest {
            cve_id: "CVE-2026-0001".into(),
            title: "Test Vulnerability".into(),
            description: None,
            severity: "high".into(),
            cvss_score: Some(8.0),
            affected_host: None,
            affected_service: None,
            notes: None,
        };

        let resp = create_vulnerability(State(state.clone()), Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = list_vulnerabilities(State(state), Query(VulnerabilitiesQuery {
            severity: None, status: None, host_id: None, page: None, per_page: None,
        })).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
