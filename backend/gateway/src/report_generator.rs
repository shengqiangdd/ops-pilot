//! Automated operational report generation.
//!
//! Generates daily/weekly reports with system health, alert statistics,
//! resource trends, and key metrics.

use anyhow::Result;
use chrono::{Utc, Duration};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;
use std::collections::HashMap;

/// A generated report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpsReport {
    pub id: String,
    pub title: String,
    pub report_type: String,    // "daily", "weekly", "monthly"
    pub period_start: String,
    pub period_end: String,
    pub generated_at: String,
    pub summary: ReportSummary,
    pub sections: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReportSummary {
    pub total_alerts: u64,
    pub critical_alerts: u64,
    pub resolved_incidents: u64,
    pub active_incidents: u64,
    pub total_hosts: u64,
    pub healthy_hosts: u64,
    pub total_vulnerabilities: u64,
    pub sla_achievement: f64,
}

/// Generate a daily ops report.
pub async fn generate_daily_report(pool: &SqlitePool) -> Result<OpsReport> {
    let now = Utc::now();
    let yesterday = now - Duration::hours(24);

    // Alerts in last 24h
    let total_alerts: u64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM alert_history WHERE created_at >= ?"
    )
    .bind(yesterday.to_rfc3339())
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let critical_alerts: u64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM alert_history WHERE severity = 'critical' AND created_at >= ?"
    )
    .bind(yesterday.to_rfc3339())
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    // Incidents
    let resolved_incidents: u64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM incidents WHERE status = 'resolved' AND updated_at >= ?"
    )
    .bind(yesterday.to_rfc3339())
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let active_incidents: u64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM incidents WHERE status != 'resolved'"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    // Hosts
    let total_hosts: u64 = sqlx::query_scalar("SELECT COUNT(*) FROM hosts")
        .fetch_one(pool).await.unwrap_or(0);

    let healthy_hosts: u64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM hosts WHERE status = 'online'"
    )
    .fetch_one(pool).await.unwrap_or(0);

    // Vulnerabilities
    let total_vulns: u64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM vulnerabilities WHERE status = 'open'"
    )
    .fetch_one(pool).await.unwrap_or(0);

    // SLO achievement
    let sla: f64 = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(AVG(sli * 100.0), 100.0) FROM slo_definitions"
    )
    .fetch_one(pool).await.unwrap_or(100.0);

    let summary = ReportSummary {
        total_alerts,
        critical_alerts,
        resolved_incidents,
        active_incidents,
        total_hosts,
        healthy_hosts,
        total_vulnerabilities: total_vulns,
        sla_achievement: sla,
    };

    // Build sections
    let mut sections = HashMap::new();

    // Top alerts by host (if table exists)
    if let Ok(top_alerts) = sqlx::query_as::<_, (String, u64)>(
        "SELECT host, COUNT(*) as cnt FROM alert_history WHERE created_at >= ?
         GROUP BY host ORDER BY cnt DESC LIMIT 5"
    )
    .bind(yesterday.to_rfc3339())
    .fetch_all(pool)
    .await
    {
        let top: Vec<Value> = top_alerts.iter().map(|(h, c)| {
            serde_json::json!({"host": h, "count": c})
        }).collect();
        sections.insert("top_alert_hosts".to_string(), Value::Array(top));
    }

    // Alert timeline (hourly buckets)
    if let Ok(hourly) = sqlx::query_as::<_, (String, u64)>(
        "SELECT substr(created_at, 12, 2) as hour, COUNT(*) as cnt
         FROM alert_history WHERE created_at >= ?
         GROUP BY hour ORDER BY hour"
    )
    .bind(yesterday.to_rfc3339())
    .fetch_all(pool)
    .await
    {
        let timeline: Vec<Value> = hourly.iter().map(|(h, c)| {
            serde_json::json!({"hour": h, "count": c})
        }).collect();
        sections.insert("alert_timeline".to_string(), Value::Array(timeline));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let report = OpsReport {
        id: id.clone(),
        title: format!("OpsPilot 运维日报 - {}", now.format("%Y-%m-%d")),
        report_type: "daily".to_string(),
        period_start: yesterday.to_rfc3339(),
        period_end: now.to_rfc3339(),
        generated_at: now.to_rfc3339(),
        summary,
        sections,
    };

    // Persist
    sqlx::query(
        r#"INSERT INTO reports (id, title, report_type, period_start, period_end, generated_at, summary_json, sections_json)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#
    )
    .bind(&report.id)
    .bind(&report.title)
    .bind(&report.report_type)
    .bind(&report.period_start)
    .bind(&report.period_end)
    .bind(&report.generated_at)
    .bind(serde_json::to_string(&report.summary)?)
    .bind(serde_json::to_string(&report.sections)?)
    .execute(pool)
    .await?;

    Ok(report)
}

/// List all generated reports.
pub async fn list_reports(pool: &SqlitePool) -> Result<Vec<OpsReport>> {
    // For simplicity, reconstruct from DB
    #[derive(sqlx::FromRow)]
    struct ReportRow {
        id: String,
        title: String,
        report_type: String,
        period_start: String,
        period_end: String,
        generated_at: String,
        summary_json: String,
        sections_json: String,
    }

    let rows = sqlx::query_as::<_, ReportRow>(
        "SELECT id, title, report_type, period_start, period_end, generated_at, summary_json, sections_json
         FROM reports WHERE summary_json IS NOT NULL AND sections_json IS NOT NULL
         ORDER BY generated_at DESC LIMIT 50"
    )
    .fetch_all(pool)
    .await?;

    let mut reports = Vec::new();
    for row in rows {
        let summary: ReportSummary = serde_json::from_str(&row.summary_json).unwrap_or_default();
        let sections: HashMap<String, Value> = serde_json::from_str(&row.sections_json).unwrap_or_default();
        reports.push(OpsReport {
            id: row.id,
            title: row.title,
            report_type: row.report_type,
            period_start: row.period_start,
            period_end: row.period_end,
            generated_at: row.generated_at,
            summary,
            sections,
        });
    }
    Ok(reports)
}
