use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Shared application state for timeline routes.
#[derive(Clone)]
pub struct TimelineState {
    pub pool: SqlitePool,
}

#[derive(Debug, Deserialize)]
pub struct TimelineQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub types: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TimelineEvent {
    pub id: String,
    pub timestamp: String,
    #[sqlx(rename = "type")]
    pub event_type: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub source: String,
}

/// GET /api/timeline/events — aggregate events from multiple tables
pub async fn list_events(
    State(state): State<TimelineState>,
    Query(query): Query<TimelineQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).min(200) as i64;
    let types_filter = query.types.unwrap_or_default();
    let type_list: Vec<&str> = if types_filter.is_empty() {
        vec!["alert", "audit", "operation"]
    } else {
        types_filter.split(',').collect()
    };

    let mut all_events: Vec<TimelineEvent> = Vec::new();

    // Fetch audit logs
    if type_list.contains(&"audit") {
        let mut sql = String::from(
            r#"SELECT id, created_at as timestamp, 'audit' as type, 'info' as severity, action as title, resource as description, "user" as source FROM audit_log WHERE 1=1"#
        );
        if let Some(ref from) = query.from {
            sql.push_str(&format!(" AND created_at >= '{}'", from));
        }
        if let Some(ref to) = query.to {
            sql.push_str(&format!(" AND created_at <= '{}'", to));
        }
        sql.push_str(" ORDER BY created_at DESC");
        sql.push_str(&format!(" LIMIT {}", limit));

        if let Ok(rows) = sqlx::query_as::<_, TimelineEvent>(&sql)
            .fetch_all(&state.pool)
            .await
        {
            all_events.extend(rows);
        }
    }

    // Fetch alert history
    if type_list.contains(&"alert") {
        let mut sql = String::from(
            "SELECT h.id, h.triggered_at as timestamp, 'alert' as type, h.severity, COALESCE(r.name, 'Unknown') as title, h.message as description, 'system' as source FROM alert_history h LEFT JOIN alert_rules r ON h.rule_id = r.id WHERE 1=1"
        );
        if let Some(ref from) = query.from {
            sql.push_str(&format!(" AND h.triggered_at >= '{}'", from));
        }
        if let Some(ref to) = query.to {
            sql.push_str(&format!(" AND h.triggered_at <= '{}'", to));
        }
        sql.push_str(" ORDER BY h.triggered_at DESC");
        sql.push_str(&format!(" LIMIT {}", limit));

        if let Ok(rows) = sqlx::query_as::<_, TimelineEvent>(&sql)
            .fetch_all(&state.pool)
            .await
        {
            all_events.extend(rows);
        }
    }

    // Fetch host operations (status changes)
    if type_list.contains(&"operation") {
        let mut sql = String::from(
            r#"SELECT id, updated_at as timestamp, 'operation' as type, 'info' as severity, name as title, address as description, status as source FROM hosts WHERE 1=1"#
        );
        if let Some(ref from) = query.from {
            sql.push_str(&format!(" AND updated_at >= '{}'", from));
        }
        if let Some(ref to) = query.to {
            sql.push_str(&format!(" AND updated_at <= '{}'", to));
        }
        sql.push_str(" ORDER BY updated_at DESC");
        sql.push_str(&format!(" LIMIT {}", limit));

        if let Ok(rows) = sqlx::query_as::<_, TimelineEvent>(&sql)
            .fetch_all(&state.pool)
            .await
        {
            all_events.extend(rows);
        }
    }

    // Sort by timestamp descending
    all_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    all_events.truncate(limit as usize);

    (StatusCode::OK, Json(all_events)).into_response()
}

/// Build the timeline routes sub-router.
pub fn timeline_routes(pool: SqlitePool) -> Router {
    use axum::routing::get;

    let state = TimelineState { pool };

    Router::new()
        .route("/api/timeline/events", get(list_events))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                "user" TEXT NOT NULL,
                action TEXT NOT NULL,
                resource TEXT NOT NULL,
                outcome TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"#
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS alert_history (
                id TEXT PRIMARY KEY,
                rule_id TEXT NOT NULL,
                severity TEXT NOT NULL,
                message TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'firing',
                triggered_at TEXT NOT NULL DEFAULT (datetime('now')),
                acknowledged_at TEXT
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS alert_rules (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS hosts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                address TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'unknown',
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_list_events_empty() {
        let pool = setup().await;
        let state = TimelineState { pool };
        let resp = list_events(State(state), Query(TimelineQuery {
            from: None, to: None, types: None, limit: None,
        })).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
