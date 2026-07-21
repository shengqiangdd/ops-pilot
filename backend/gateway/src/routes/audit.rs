use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::Sqlite;
use sqlx::SqlitePool;

/// Shared application state for audit routes.
#[derive(Clone)]
pub struct AuditState {
    pub pool: SqlitePool,
}

#[derive(Deserialize)]
pub struct AuditQuery {
    pub user: Option<String>,
    pub action: Option<String>,
    pub resource: Option<String>,
    pub outcome: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Deserialize)]
pub struct ExportQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub user: Option<String>,
    pub action: Option<String>,
}

/// GET /api/audit/logs — query audit logs with optional filters and pagination.
pub async fn list_audit_logs(
    State(state): State<AuditState>,
    Query(query): Query<AuditQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;
    let limit = per_page as i64;

    // Count total matching rows
    let mut count_builder: sqlx::QueryBuilder<Sqlite> =
        sqlx::QueryBuilder::new("SELECT COUNT(*) as \"count\" FROM audit_log WHERE 1=1");

    apply_filters(&mut count_builder, &query);

    let total: i64 = match count_builder
        .build_query_as::<CountRow>()
        .fetch_one(&state.pool)
        .await
    {
        Ok(row) => row.count,
        Err(_) => 0,
    };

    // Fetch paginated data
    let mut data_builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        r#"SELECT id, "user", action, resource, outcome, created_at FROM audit_log WHERE 1=1"#,
    );

    apply_filters(&mut data_builder, &query);

    data_builder.push(" ORDER BY created_at DESC, rowid DESC LIMIT ");
    data_builder.push_bind(limit);
    data_builder.push(" OFFSET ");
    data_builder.push_bind(offset);

    let q = data_builder.build_query_as::<AuditLogRow>();

    match q.fetch_all(&state.pool).await {
        Ok(rows) => {
            let entries: Vec<serde_json::Value> = rows.into_iter().map(|r| r.into_json()).collect();
            let resp = PaginatedResponse {
                data: entries,
                total,
                page,
                per_page,
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/audit/export — export audit logs as CSV.
pub async fn export_audit_logs(
    State(state): State<AuditState>,
    Query(query): Query<ExportQuery>,
) -> impl IntoResponse {
    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        r#"SELECT id, "user", action, resource, outcome, created_at FROM audit_log WHERE 1=1"#,
    );

    if let Some(ref user) = query.user {
        builder.push(" AND \"user\" = ");
        builder.push_bind(user);
    }
    if let Some(ref action) = query.action {
        builder.push(" AND action = ");
        builder.push_bind(action);
    }
    if let Some(ref from) = query.from {
        builder.push(" AND created_at >= ");
        builder.push_bind(from);
    }
    if let Some(ref to) = query.to {
        builder.push(" AND created_at <= ");
        builder.push_bind(to);
    }

    builder.push(" ORDER BY created_at DESC, rowid DESC");

    let q = builder.build_query_as::<AuditLogRow>();

    match q.fetch_all(&state.pool).await {
        Ok(rows) => {
            let mut csv = String::from("id,user,action,resource,outcome,created_at\n");
            for row in rows {
                csv.push_str(&format!(
                    "{},{},{},{},{},{}\n",
                    escape_csv(&row.id),
                    escape_csv(&row.user),
                    escape_csv(&row.action),
                    escape_csv(&row.resource),
                    escape_csv(&row.outcome),
                    escape_csv(&row.created_at),
                ));
            }
            (
                StatusCode::OK,
                [
                    ("Content-Type", "text/csv; charset=utf-8"),
                    (
                        "Content-Disposition",
                        "attachment; filename=\"audit_log.csv\"",
                    ),
                ],
                csv,
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/audit/stats — audit statistics: counts by action and by outcome.
pub async fn audit_stats(State(state): State<AuditState>) -> impl IntoResponse {
    let action_result = sqlx::query_as::<_, ActionCount>(
        "SELECT action, COUNT(*) as count FROM audit_log GROUP BY action ORDER BY count DESC",
    )
    .fetch_all(&state.pool)
    .await;

    let outcome_result = sqlx::query_as::<_, OutcomeCount>(
        "SELECT outcome, COUNT(*) as count FROM audit_log GROUP BY outcome ORDER BY count DESC",
    )
    .fetch_all(&state.pool)
    .await;

    match (action_result, outcome_result) {
        (Ok(actions), Ok(outcomes)) => {
            let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
                .fetch_one(&state.pool)
                .await
                .unwrap_or(0);

            let stats = serde_json::json!({
                "total": total,
                "by_action": actions.into_iter().map(|r| serde_json::json!({
                    "action": r.action,
                    "count": r.count,
                })).collect::<Vec<_>>(),
                "by_outcome": outcomes.into_iter().map(|r| serde_json::json!({
                    "outcome": r.outcome,
                    "count": r.count,
                })).collect::<Vec<_>>(),
            });
            (StatusCode::OK, Json(stats)).into_response()
        }
        (Err(e), _) | (_, Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Build the audit routes sub-router.
pub fn audit_routes(pool: SqlitePool) -> axum::Router {
    use axum::routing::get;

    let state = AuditState { pool };

    axum::Router::new()
        .route("/api/audit/logs", get(list_audit_logs))
        .route("/api/audit/stats", get(audit_stats))
        .route("/api/audit/export", get(export_audit_logs))
        .with_state(state)
}

fn apply_filters(builder: &mut sqlx::QueryBuilder<Sqlite>, query: &AuditQuery) {
    if let Some(ref user) = query.user {
        builder.push(" AND \"user\" = ");
        builder.push_bind(user.clone());
    }
    if let Some(ref action) = query.action {
        builder.push(" AND action = ");
        builder.push_bind(action.clone());
    }
    if let Some(ref resource) = query.resource {
        builder.push(" AND resource = ");
        builder.push_bind(resource.clone());
    }
    if let Some(ref outcome) = query.outcome {
        builder.push(" AND outcome = ");
        builder.push_bind(outcome.clone());
    }
    if let Some(ref from) = query.from {
        builder.push(" AND created_at >= ");
        builder.push_bind(from.clone());
    }
    if let Some(ref to) = query.to {
        builder.push(" AND created_at <= ");
        builder.push_bind(to.clone());
    }
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[derive(sqlx::FromRow)]
struct AuditLogRow {
    id: String,
    #[sqlx(rename = "user")]
    user: String,
    action: String,
    resource: String,
    outcome: String,
    created_at: String,
}

impl AuditLogRow {
    fn into_json(self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "user": self.user,
            "action": self.action,
            "resource": self.resource,
            "outcome": self.outcome,
            "created_at": self.created_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct ActionCount {
    action: String,
    count: i64,
}

#[derive(sqlx::FromRow)]
struct OutcomeCount {
    outcome: String,
    count: i64,
}

#[derive(sqlx::FromRow)]
struct CountRow {
    count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ops_pilot_core::audit::AuditTrail;
    use ops_pilot_core::db::Database;
    use ops_pilot_sdk::context::EventBus;

    async fn setup() -> (SqlitePool, AuditTrail) {
        let db = Database::open_in_memory().await.unwrap();
        let bus = EventBus::new(64);
        let trail = AuditTrail::new(&db, bus);
        (db.pool.clone(), trail)
    }

    #[tokio::test]
    async fn list_logs_empty() {
        let (pool, _trail) = setup().await;
        let state = AuditState { pool };

        let resp = list_audit_logs(State(state), Query(AuditQuery {
            user: None, action: None, resource: None, outcome: None,
            from: None, to: None, page: None, per_page: None,
        })).await.into_response();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let resp: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(resp["total"], 0);
        assert_eq!(resp["page"], 1);
        assert_eq!(resp["per_page"], 20);
        assert!(resp["data"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn list_logs_returns_paginated() {
        let (pool, trail) = setup().await;
        trail.log("alice", "connect", "host/prod", "success").await.unwrap();
        trail.log("bob", "delete", "key/secret", "denied").await.unwrap();

        let state = AuditState { pool: pool.clone() };
        let resp = list_audit_logs(State(state), Query(AuditQuery {
            user: None, action: None, resource: None, outcome: None,
            from: None, to: None, page: None, per_page: None,
        })).await.into_response();

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let resp: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(resp["total"], 2);
        assert_eq!(resp["data"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn list_logs_filter_by_user() {
        let (pool, trail) = setup().await;
        trail.log("alice", "connect", "host/prod", "success").await.unwrap();
        trail.log("bob", "delete", "key/secret", "denied").await.unwrap();

        let state = AuditState { pool: pool.clone() };
        let resp = list_audit_logs(State(state), Query(AuditQuery {
            user: Some("alice".into()), action: None, resource: None, outcome: None,
            from: None, to: None, page: None, per_page: None,
        })).await.into_response();

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let resp: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(resp["total"], 1);
        assert_eq!(resp["data"][0]["user"], "alice");
    }

    #[tokio::test]
    async fn list_logs_pagination() {
        let (pool, trail) = setup().await;
        for i in 0..5 {
            trail.log(&format!("user{}", i), "action", "resource", "ok").await.unwrap();
        }

        let state = AuditState { pool: pool.clone() };
        let resp = list_audit_logs(State(state), Query(AuditQuery {
            user: None, action: None, resource: None, outcome: None,
            from: None, to: None, page: Some(2), per_page: Some(2),
        })).await.into_response();

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let resp: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(resp["total"], 5);
        assert_eq!(resp["page"], 2);
        assert_eq!(resp["per_page"], 2);
        assert_eq!(resp["data"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn list_logs_date_filter() {
        let (pool, trail) = setup().await;
        trail.log("alice", "connect", "host/prod", "success").await.unwrap();
        trail.log("bob", "delete", "key/secret", "denied").await.unwrap();

        let state = AuditState { pool: pool.clone() };
        let resp = list_audit_logs(State(state), Query(AuditQuery {
            user: None, action: None, resource: None, outcome: None,
            from: Some("2000-01-01".into()), to: Some("2099-12-31".into()),
            page: None, per_page: None,
        })).await.into_response();

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let resp: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(resp["total"], 2);

        let resp2 = list_audit_logs(State(AuditState { pool: pool.clone() }), Query(AuditQuery {
            user: None, action: None, resource: None, outcome: None,
            from: Some("2099-01-01".into()), to: None,
            page: None, per_page: None,
        })).await.into_response();
        let body2 = axum::body::to_bytes(resp2.into_body(), usize::MAX).await.unwrap();
        let resp2: serde_json::Value = serde_json::from_slice(&body2).unwrap();
        assert_eq!(resp2["total"], 0);
    }

    #[tokio::test]
    async fn stats_returns_totals() {
        let (pool, trail) = setup().await;
        trail.log("alice", "connect", "host/prod", "success").await.unwrap();
        trail.log("bob", "delete", "key/secret", "denied").await.unwrap();
        trail.log("alice", "connect", "host/staging", "success").await.unwrap();

        let state = AuditState { pool };
        let resp = audit_stats(State(state)).await.into_response();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let stats: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(stats["total"], 3);
        assert!(stats["by_action"].is_array());
        assert!(stats["by_outcome"].is_array());
    }

    #[tokio::test]
    async fn export_csv() {
        let (pool, trail) = setup().await;
        trail.log("alice", "connect", "host/prod", "success").await.unwrap();
        trail.log("bob", "delete", "key/secret", "denied").await.unwrap();

        let state = AuditState { pool };
        let resp = export_audit_logs(State(state), Query(ExportQuery {
            from: None, to: None, user: None, action: None,
        })).await.into_response();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let csv = String::from_utf8(body.to_vec()).unwrap();
        assert!(csv.starts_with("id,user,action,resource,outcome,created_at\n"));
        assert!(csv.contains("alice"));
        assert!(csv.contains("bob"));
    }

    #[tokio::test]
    async fn export_csv_with_filter() {
        let (pool, trail) = setup().await;
        trail.log("alice", "connect", "host/prod", "success").await.unwrap();
        trail.log("bob", "delete", "key/secret", "denied").await.unwrap();

        let state = AuditState { pool };
        let resp = export_audit_logs(State(state), Query(ExportQuery {
            from: None, to: None, user: Some("alice".into()), action: None,
        })).await.into_response();

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let csv = String::from_utf8(body.to_vec()).unwrap();
        assert!(csv.contains("alice"));
        assert!(!csv.contains("bob"));
    }

    #[test]
    fn escape_csv_basic() {
        assert_eq!(escape_csv("hello"), "hello");
        assert_eq!(escape_csv("a,b"), "\"a,b\"");
        assert_eq!(escape_csv("a\"b"), "\"a\"\"b\"");
    }
}
