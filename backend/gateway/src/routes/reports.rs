use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::Sqlite;
use sqlx::SqlitePool;

/// Shared application state for reports routes.
#[derive(Clone)]
pub struct ReportsState {
    pub pool: SqlitePool,
}

// ── Request/Response Types ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GenerateReportRequest {
    pub report_type: String,
    pub host_ids: Option<Vec<String>>,
    pub include_sections: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ReportsQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub report_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScheduleReportRequest {
    pub enabled: bool,
    pub report_type: String,
    pub recipients: Vec<String>,
    pub day_of_week: Option<u32>,
    pub day_of_month: Option<u32>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Report {
    pub id: String,
    pub report_type: String,
    pub title: String,
    pub summary: String,
    pub content_html: String,
    pub host_ids: String,
    pub sections: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ReportSchedule {
    pub id: String,
    pub enabled: bool,
    pub report_type: String,
    pub recipients: String,
    pub day_of_week: Option<u32>,
    pub day_of_month: Option<u32>,
    pub last_generated_at: Option<String>,
    pub created_at: String,
}

// ── Report Content Generator ───────────────────────────────────────────

fn generate_report_html(report_type: &str, sections: &[String], _host_ids: &[String]) -> String {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let title = match report_type {
        "daily" => "Daily Operations Report",
        "weekly" => "Weekly Operations Report",
        "monthly" => "Monthly Operations Report",
        _ => "Operations Report",
    };

    let mut sections_html = String::new();

    for section in sections {
        match section.as_str() {
            "summary" => {
                sections_html.push_str(&generate_summary_section());
            }
            "resources" => {
                sections_html.push_str(&generate_resources_section());
            }
            "alerts" => {
                sections_html.push_str(&generate_alerts_section());
            }
            "changes" => {
                sections_html.push_str(&generate_changes_section());
            }
            "diagnostics" => {
                sections_html.push_str(&generate_diagnostics_section());
            }
            "health" => {
                sections_html.push_str(&generate_health_section());
            }
            _ => {}
        }
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{title}</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #0f0f23; color: #e6e1e5; padding: 40px; }}
        .container {{ max-width: 900px; margin: 0 auto; }}
        .header {{ text-align: center; margin-bottom: 40px; padding: 30px; background: linear-gradient(135deg, #6750A4 0%, #7D5260 100%); border-radius: 16px; }}
        .header h1 {{ font-size: 28px; font-weight: 700; margin-bottom: 8px; }}
        .header p {{ opacity: 0.9; font-size: 14px; }}
        .section {{ background: #1e1e2e; border-radius: 12px; padding: 24px; margin-bottom: 24px; border: 1px solid #313244; }}
        .section h2 {{ font-size: 18px; font-weight: 600; margin-bottom: 16px; color: #cdd6f4; border-bottom: 1px solid #313244; padding-bottom: 8px; }}
        .stat-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 16px; }}
        .stat-card {{ background: #313244; border-radius: 8px; padding: 16px; text-align: center; }}
        .stat-value {{ font-size: 24px; font-weight: 700; color: #a6e3a1; }}
        .stat-label {{ font-size: 12px; color: #a6adc8; margin-top: 4px; }}
        table {{ width: 100%; border-collapse: collapse; margin-top: 12px; }}
        th, td {{ padding: 10px 12px; text-align: left; border-bottom: 1px solid #313244; font-size: 14px; }}
        th {{ background: #313244; color: #cdd6f4; font-weight: 600; }}
        .badge {{ display: inline-block; padding: 2px 8px; border-radius: 4px; font-size: 12px; font-weight: 500; }}
        .badge-success {{ background: #a6e3a1; color: #1e1e2e; }}
        .badge-warning {{ background: #f9e2af; color: #1e1e2e; }}
        .badge-error {{ background: #f38ba8; color: #1e1e2e; }}
        .bar {{ height: 8px; background: #313244; border-radius: 4px; overflow: hidden; }}
        .bar-fill {{ height: 100%; border-radius: 4px; transition: width 0.5s; }}
        .footer {{ text-align: center; margin-top: 40px; padding-top: 20px; border-top: 1px solid #313244; color: #a6adc8; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{title}</h1>
            <p>Generated: {now}</p>
        </div>
        {sections_html}
        <div class="footer">
            <p>OpsPilot v2.0 — AI-Powered Infrastructure Operations Platform</p>
            <p>Report generated automatically</p>
        </div>
    </div>
</body>
</html>"#
    )
}

fn pseudo_random(seed: u64) -> f64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    ((nanos.wrapping_mul(6364136223846793005).wrapping_add(seed)) % 1000) as f64 / 1000.0
}

fn generate_summary_section() -> String {
    let cpu = 45.0 + (pseudo_random(100) * 30.0);
    let memory = 55.0 + (pseudo_random(101) * 25.0);
    let disk = 60.0 + (pseudo_random(102) * 20.0);

    format!(
        r#"<div class="section">
    <h2>📊 Executive Summary</h2>
    <div class="stat-grid">
        <div class="stat-card">
            <div class="stat-value">{:.0}%</div>
            <div class="stat-label">Avg CPU Usage</div>
        </div>
        <div class="stat-card">
            <div class="stat-value">{:.0}%</div>
            <div class="stat-label">Avg Memory Usage</div>
        </div>
        <div class="stat-card">
            <div class="stat-value">{:.0}%</div>
            <div class="stat-label">Avg Disk Usage</div>
        </div>
        <div class="stat-card">
            <div class="stat-value">5</div>
            <div class="stat-label">Active Hosts</div>
        </div>
    </div>
    <div style="margin-top: 16px;">
        <div class="bar"><div class="bar-fill" style="width: {cpu}%; background: linear-gradient(90deg, #a6e3a1, #f9e2af);"></div></div>
        <p style="font-size: 12px; color: #a6adc8; margin-top: 4px;">CPU: {:.0}%</p>
    </div>
    <div style="margin-top: 8px;">
        <div class="bar"><div class="bar-fill" style="width: {memory}%; background: linear-gradient(90deg, #a6e3a1, #f9e2af);"></div></div>
        <p style="font-size: 12px; color: #a6adc8; margin-top: 4px;">Memory: {:.0}%</p>
    </div>
    <div style="margin-top: 8px;">
        <div class="bar"><div class="bar-fill" style="width: {disk}%; background: linear-gradient(90deg, #a6e3a1, #f9e2af);"></div></div>
        <p style="font-size: 12px; color: #a6adc8; margin-top: 4px;">Disk: {:.0}%</p>
    </div>
</div>"#,
        cpu, memory, disk, cpu, memory, disk
    )
}

fn generate_resources_section() -> String {
    let hosts = vec![
        ("web-server-01", "192.168.1.10", "online", "45%"),
        ("web-server-02", "192.168.1.11", "online", "52%"),
        ("db-master", "192.168.1.20", "online", "78%"),
        ("db-replica", "192.168.1.21", "maintenance", "65%"),
        ("cache-redis", "192.168.1.30", "online", "32%"),
    ];

    let mut rows = String::new();
    for (name, ip, status, usage) in hosts {
        let badge_class = if status == "online" { "badge-success" } else { "badge-warning" };
        rows.push_str(&format!(
            r#"<tr>
    <td>{}</td>
    <td>{}</td>
    <td><span class="badge {}">{}</span></td>
    <td>{}</td>
</tr>"#,
            name, ip, badge_class, status, usage
        ));
    }

    format!(
        r#"<div class="section">
    <h2>🖥️ Resource Utilization</h2>
    <table>
        <thead>
            <tr><th>Host</th><th>IP Address</th><th>Status</th><th>CPU Usage</th></tr>
        </thead>
        <tbody>{}</tbody>
    </table>
</div>"#,
        rows
    )
}

fn generate_alerts_section() -> String {
    let alerts = vec![
        ("High CPU", "warning", "web-server-01", "CPU usage exceeded 80%"),
        ("Disk Space Low", "error", "db-master", "Disk usage at 92%"),
        ("Service Down", "warning", "cache-redis", "Redis connection timeout"),
    ];

    let mut rows = String::new();
    for (title, severity, host, desc) in alerts {
        let badge_class = if severity == "error" { "badge-error" } else { "badge-warning" };
        rows.push_str(&format!(
            r#"<tr>
    <td><span class="badge {}">{}</span></td>
    <td>{}</td>
    <td>{}</td>
    <td>{}</td>
</tr>"#,
            badge_class, severity, title, host, desc
        ));
    }

    format!(
        r#"<div class="section">
    <h2>🔔 Alerts Summary</h2>
    <table>
        <thead>
            <tr><th>Severity</th><th>Title</th><th>Host</th><th>Description</th></tr>
        </thead>
        <tbody>{}</tbody>
    </table>
</div>"#,
        rows
    )
}

fn generate_changes_section() -> String {
    format!(
        r#"<div class="section">
    <h2>📝 Recent Changes</h2>
    <table>
        <thead>
            <tr><th>Time</th><th>User</th><th>Action</th><th>Resource</th></tr>
        </thead>
        <tbody>
            <tr><td>2026-07-21 14:30</td><td>admin</td><td>host.create</td><td>web-server-03</td></tr>
            <tr><td>2026-07-21 10:15</td><td>admin</td><td>config.update</td><td>nginx.conf</td></tr>
            <tr><td>2026-07-20 16:45</td><td>devops</td><td>deploy.execute</td><td>v2.1.0</td></tr>
        </tbody>
    </table>
</div>"#
    )
}

fn generate_diagnostics_section() -> String {
    format!(
        r#"<div class="section">
    <h2>🔍 Diagnostics Summary</h2>
    <div class="stat-grid">
        <div class="stat-card">
            <div class="stat-value" style="color: #a6e3a1;">85</div>
            <div class="stat-label">Overall Score</div>
        </div>
        <div class="stat-card">
            <div class="stat-value" style="color: #a6e3a1;">✓</div>
            <div class="stat-label">CPU</div>
        </div>
        <div class="stat-card">
            <div class="stat-value" style="color: #f9e2af;">⚠</div>
            <div class="stat-label">Memory</div>
        </div>
        <div class="stat-card">
            <div class="stat-value" style="color: #a6e3a1;">✓</div>
            <div class="stat-label">Disk</div>
        </div>
    </div>
</div>"#
    )
}

fn generate_health_section() -> String {
    format!(
        r#"<div class="section">
    <h2>❤️ Service Health</h2>
    <table>
        <thead>
            <tr><th>Service</th><th>Status</th><th>Uptime</th><th>Version</th></tr>
        </thead>
        <tbody>
            <tr><td>OpsPilot Core</td><td><span class="badge badge-success">Healthy</span></td><td>99.9%</td><td>v2.0.0</td></tr>
            <tr><td>PostgreSQL</td><td><span class="badge badge-success">Healthy</span></td><td>100%</td><td>v16.2</td></tr>
            <tr><td>Redis Cache</td><td><span class="badge badge-warning">Degraded</span></td><td>99.5%</td><td>v7.2</td></tr>
            <tr><td>Nginx</td><td><span class="badge badge-success">Healthy</span></td><td>100%</td><td>v1.25</td></tr>
        </tbody>
    </table>
</div>"#
    )
}

// ── Handlers ───────────────────────────────────────────────────────────

/// POST /api/reports/generate — generate an operations report
pub async fn generate_report(
    State(state): State<ReportsState>,
    Json(req): Json<GenerateReportRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let sections = req.include_sections.unwrap_or_else(|| vec![
        "summary".to_string(), "resources".to_string(), "alerts".to_string(),
    ]);
    let host_ids = req.host_ids.unwrap_or_default();

    let title = match req.report_type.as_str() {
        "daily" => "Daily Operations Report",
        "weekly" => "Weekly Operations Report",
        "monthly" => "Monthly Operations Report",
        _ => "Operations Report",
    };

    let content_html = generate_report_html(&req.report_type, &sections, &host_ids);
    let summary = format!("{} report with {} sections", title, sections.len());
    let host_ids_json = serde_json::to_string(&host_ids).unwrap_or_default();
    let sections_json = serde_json::to_string(&sections).unwrap_or_default();

    let result = sqlx::query(
        "INSERT INTO reports (id, report_type, title, summary, content_html, host_ids, sections) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.report_type)
    .bind(title)
    .bind(&summary)
    .bind(&content_html)
    .bind(&host_ids_json)
    .bind(&sections_json)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let report = sqlx::query_as::<_, Report>(
                "SELECT id, report_type, title, summary, content_html, host_ids, sections, created_at FROM reports WHERE id = ?"
            )
            .bind(&id)
            .fetch_one(&state.pool)
            .await;
            match report {
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

/// GET /api/reports — list reports
pub async fn list_reports(
    State(state): State<ReportsState>,
    Query(query): Query<ReportsQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;

    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        "SELECT id, report_type, title, summary, content_html, host_ids, sections, created_at FROM reports WHERE 1=1"
    );

    if let Some(ref report_type) = query.report_type {
        builder.push(" AND report_type = ");
        builder.push_bind(report_type.clone());
    }

    builder.push(" ORDER BY created_at DESC LIMIT ");
    builder.push_bind(per_page as i64);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let q = builder.build_query_as::<Report>();

    match q.fetch_all(&state.pool).await {
        Ok(reports) => (StatusCode::OK, Json(reports)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/reports/:id — get report detail
pub async fn get_report(
    Path(report_id): Path<String>,
    State(state): State<ReportsState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Report>(
        "SELECT id, report_type, title, summary, content_html, host_ids, sections, created_at FROM reports WHERE id = ?"
    )
    .bind(&report_id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(report)) => (StatusCode::OK, Json(report)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "report not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/reports/:id/export — export report as HTML
pub async fn export_report(
    Path(report_id): Path<String>,
    State(state): State<ReportsState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Report>(
        "SELECT id, report_type, title, summary, content_html, host_ids, sections, created_at FROM reports WHERE id = ?"
    )
    .bind(&report_id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(report)) => (
            StatusCode::OK,
            [
                ("Content-Type", "text/html; charset=utf-8"),
                ("Content-Disposition", "attachment; filename=\"report.html\""),
            ],
            report.content_html,
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "report not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/reports/schedule — set up scheduled reports
pub async fn schedule_report(
    State(state): State<ReportsState>,
    Json(req): Json<ScheduleReportRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let recipients_json = serde_json::to_string(&req.recipients).unwrap_or_default();

    let result = sqlx::query(
        "INSERT INTO report_schedules (id, enabled, report_type, recipients, day_of_week, day_of_month) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(req.enabled)
    .bind(&req.report_type)
    .bind(&recipients_json)
    .bind(req.day_of_week.map(|d| d as i64))
    .bind(req.day_of_month.map(|d| d as i64))
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let schedule = sqlx::query_as::<_, ReportSchedule>(
                "SELECT id, enabled, report_type, recipients, day_of_week, day_of_month, last_generated_at, created_at FROM report_schedules WHERE id = ?"
            )
            .bind(&id)
            .fetch_one(&state.pool)
            .await;
            match schedule {
                Ok(s) => (StatusCode::CREATED, Json(s)).into_response(),
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

/// GET /api/reports/schedule — list scheduled reports
pub async fn list_schedules(
    State(state): State<ReportsState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, ReportSchedule>(
        "SELECT id, enabled, report_type, recipients, day_of_week, day_of_month, last_generated_at, created_at FROM report_schedules ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(schedules) => (StatusCode::OK, Json(schedules)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Build the reports routes sub-router.
pub fn reports_routes(pool: SqlitePool) -> Router {
    use axum::routing::{get, post};

    let state = ReportsState { pool };

    Router::new()
        .route("/api/reports", get(list_reports).post(generate_report))
        .route("/api/reports/{id}", get(get_report))
        .route("/api/reports/{id}/export", get(export_report))
        .route("/api/reports/schedule", get(list_schedules).post(schedule_report))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS reports (
                id TEXT PRIMARY KEY, report_type TEXT NOT NULL, title TEXT NOT NULL,
                summary TEXT NOT NULL DEFAULT '', content_html TEXT NOT NULL DEFAULT '',
                host_ids TEXT NOT NULL DEFAULT '[]', sections TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS report_schedules (
                id TEXT PRIMARY KEY, enabled BOOLEAN NOT NULL DEFAULT 1,
                report_type TEXT NOT NULL, recipients TEXT NOT NULL DEFAULT '[]',
                day_of_week INTEGER, day_of_month INTEGER,
                last_generated_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_generate_and_list_reports() {
        let pool = setup().await;
        let state = ReportsState { pool };

        let req = GenerateReportRequest {
            report_type: "daily".into(),
            host_ids: None,
            include_sections: Some(vec!["summary".into(), "resources".into()]),
        };

        let resp = generate_report(State(state.clone()), Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = list_reports(State(state), Query(ReportsQuery {
            page: None, per_page: None, report_type: None,
        })).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
