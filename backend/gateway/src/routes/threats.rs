use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::Sqlite;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct ThreatsState {
    pub pool: SqlitePool,
}

#[derive(Debug, Serialize)]
pub struct ThreatOverview {
    pub total_indicators: i64,
    pub affected_assets: i64,
    pub critical_count: i64,
    pub high_count: i64,
    pub medium_count: i64,
    pub low_count: i64,
    pub today_new: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ThreatIndicator {
    pub id: String,
    pub feed_id: String,
    pub indicator_type: String,
    pub indicator_value: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub first_seen: String,
    pub last_seen: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AffectedAsset {
    pub id: String,
    pub host_id: String,
    pub host_name: String,
    pub indicator_type: String,
    pub indicator_value: String,
    pub severity: String,
    pub threat_title: String,
    pub risk_level: String,
    pub suggestion: String,
}

#[derive(Debug, Deserialize)]
pub struct IndicatorsQuery {
    pub indicator_type: Option<String>,
    pub severity: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

fn pseudo_random(seed: u64) -> f64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    ((nanos.wrapping_mul(6364136223846793005).wrapping_add(seed)) % 1000) as f64 / 1000.0
}

/// GET /api/threats/overview — threat overview
pub async fn threat_overview(
    State(state): State<ThreatsState>,
) -> impl IntoResponse {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM threat_indicators")
        .fetch_one(&state.pool).await.unwrap_or((0,));

    let affected: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT host_id) FROM asset_exposure")
        .fetch_one(&state.pool).await.unwrap_or((0,));

    let critical: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM threat_indicators WHERE severity = 'critical'")
        .fetch_one(&state.pool).await.unwrap_or((0,));

    let high: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM threat_indicators WHERE severity = 'high'")
        .fetch_one(&state.pool).await.unwrap_or((0,));

    let medium: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM threat_indicators WHERE severity = 'medium'")
        .fetch_one(&state.pool).await.unwrap_or((0,));

    let low: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM threat_indicators WHERE severity = 'low'")
        .fetch_one(&state.pool).await.unwrap_or((0,));

    let today_new: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM threat_indicators WHERE date(first_seen) = date('now')")
        .fetch_one(&state.pool).await.unwrap_or((0,));

    let overview = ThreatOverview {
        total_indicators: total.0,
        affected_assets: affected.0,
        critical_count: critical.0,
        high_count: high.0,
        medium_count: medium.0,
        low_count: low.0,
        today_new: today_new.0,
    };

    (StatusCode::OK, Json(overview)).into_response()
}

/// GET /api/threats/indicators — list threat indicators
pub async fn list_indicators(
    State(state): State<ThreatsState>,
    Query(query): Query<IndicatorsQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;

    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        "SELECT id, feed_id, indicator_type, indicator_value, severity, title, description, first_seen, last_seen FROM threat_indicators WHERE 1=1"
    );

    if let Some(ref indicator_type) = query.indicator_type {
        builder.push(" AND indicator_type = ");
        builder.push_bind(indicator_type.clone());
    }
    if let Some(ref severity) = query.severity {
        builder.push(" AND severity = ");
        builder.push_bind(severity.clone());
    }

    builder.push(" ORDER BY first_seen DESC LIMIT ");
    builder.push_bind(per_page as i64);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let q = builder.build_query_as::<ThreatIndicator>();

    match q.fetch_all(&state.pool).await {
        Ok(indicators) => (StatusCode::OK, Json(indicators)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/threats/affected-assets — list affected assets
pub async fn affected_assets(
    State(state): State<ThreatsState>,
) -> impl IntoResponse {
    // Simulate affected assets data
    let assets = vec![
        ("host-001", "web-server-01", "ip", "192.168.1.10", "high", "C2 Server IP detected", "high", "Block IP at firewall"),
        ("host-002", "db-master", "domain", "malware-c2.evil.com", "critical", "Malicious domain contacted", "critical", "Isolate host and investigate"),
        ("host-003", "cache-redis", "hash", "d41d8cd98f00b204", "medium", "Known malware hash", "medium", "Run full AV scan"),
    ];

    let result: Vec<serde_json::Value> = assets.iter().map(|(id, name, itype, value, severity, title, risk, suggestion)| {
        serde_json::json!({
            "id": id,
            "host_id": id,
            "host_name": name,
            "indicator_type": itype,
            "indicator_value": value,
            "severity": severity,
            "threat_title": title,
            "risk_level": risk,
            "suggestion": suggestion,
        })
    }).collect();

    (StatusCode::OK, Json(result)).into_response()
}

pub fn threats_routes(pool: SqlitePool) -> Router {
    use axum::routing::get;

    let state = ThreatsState { pool };

    Router::new()
        .route("/api/threats/overview", get(threat_overview))
        .route("/api/threats/indicators", get(list_indicators))
        .route("/api/threats/affected-assets", get(affected_assets))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS threat_indicators (id TEXT PRIMARY KEY, feed_id TEXT NOT NULL, indicator_type TEXT NOT NULL, indicator_value TEXT NOT NULL, severity TEXT NOT NULL, title TEXT NOT NULL, description TEXT NOT NULL DEFAULT '', first_seen TEXT NOT NULL DEFAULT (datetime('now')), last_seen TEXT NOT NULL DEFAULT (datetime('now')))").execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS asset_exposure (id TEXT PRIMARY KEY, host_id TEXT NOT NULL, indicator_id TEXT NOT NULL)").execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS threat_feeds (id TEXT PRIMARY KEY, name TEXT NOT NULL, source_url TEXT NOT NULL, source_type TEXT NOT NULL, enabled BOOLEAN NOT NULL DEFAULT 1, last_updated TEXT)").execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_threat_overview() {
        let pool = setup().await;
        let state = ThreatsState { pool };
        let resp = threat_overview(State(state)).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
