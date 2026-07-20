use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
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
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// GET /api/audit/logs — query audit logs with optional filters and pagination.
pub async fn list_audit_logs(
    State(state): State<AuditState>,
    Query(query): Query<AuditQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).min(500) as i64;
    let offset = query.offset.unwrap_or(0) as i64;

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
    if let Some(ref resource) = query.resource {
        builder.push(" AND resource = ");
        builder.push_bind(resource);
    }
    if let Some(ref outcome) = query.outcome {
        builder.push(" AND outcome = ");
        builder.push_bind(outcome);
    }

    builder.push(" ORDER BY created_at DESC, rowid DESC LIMIT ");
    builder.push_bind(limit);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let q = builder.build_query_as::<AuditLogRow>();

    match q.fetch_all(&state.pool).await {
        Ok(rows) => {
            let entries: Vec<serde_json::Value> = rows.into_iter().map(|r| r.into_json()).collect();
            (StatusCode::OK, Json(entries)).into_response()
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
        .with_state(state)
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
            limit: None, offset: None,
        })).await.into_response();

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn list_logs_returns_entries() {
        let (pool, trail) = setup().await;
        trail.log("alice", "connect", "host/prod", "success").await.unwrap();
        trail.log("bob", "delete", "key/secret", "denied").await.unwrap();

        let state = AuditState { pool: pool.clone() };
        let resp = list_audit_logs(State(state), Query(AuditQuery {
            user: None, action: None, resource: None, outcome: None,
            limit: None, offset: None,
        })).await.into_response();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let entries: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[tokio::test]
    async fn list_logs_filter_by_user() {
        let (pool, trail) = setup().await;
        trail.log("alice", "connect", "host/prod", "success").await.unwrap();
        trail.log("bob", "delete", "key/secret", "denied").await.unwrap();

        let state = AuditState { pool: pool.clone() };
        let resp = list_audit_logs(State(state), Query(AuditQuery {
            user: Some("alice".into()), action: None, resource: None, outcome: None,
            limit: None, offset: None,
        })).await.into_response();

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let entries: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["user"], "alice");
    }

    #[tokio::test]
    async fn list_logs_filter_by_action() {
        let (pool, trail) = setup().await;
        trail.log("alice", "connect", "host/prod", "success").await.unwrap();
        trail.log("alice", "delete", "key/secret", "denied").await.unwrap();

        let state = AuditState { pool: pool.clone() };
        let resp = list_audit_logs(State(state), Query(AuditQuery {
            user: None, action: Some("connect".into()), resource: None, outcome: None,
            limit: None, offset: None,
        })).await.into_response();

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let entries: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["action"], "connect");
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
            limit: Some(2), offset: Some(1),
        })).await.into_response();

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let entries: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
        assert_eq!(entries.len(), 2);
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
}
