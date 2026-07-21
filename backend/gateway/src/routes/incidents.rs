use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::Sqlite;
use sqlx::SqlitePool;

/// Shared application state for incidents routes.
#[derive(Clone)]
pub struct IncidentsState {
    pub pool: SqlitePool,
}

// ── Request/Response Types ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct IncidentsQuery {
    pub status: Option<String>,
    pub host_id: Option<String>,
    pub severity: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIncidentRequest {
    pub status: Option<String>,
    pub assigned_to: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignIncidentRequest {
    pub assigned_to: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Incident {
    pub id: String,
    pub name: String,
    pub status: String,
    pub severity: String,
    pub host_id: String,
    pub first_seen: String,
    pub last_seen: String,
    pub alert_count: i64,
    pub summary: String,
    pub assigned_to: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct IncidentAlert {
    pub id: String,
    pub incident_id: String,
    pub alert_id: String,
    pub alert_message: String,
    pub alert_severity: String,
    pub triggered_at: String,
}

#[derive(Debug, Serialize)]
pub struct IncidentDetail {
    pub incident: Incident,
    pub alerts: Vec<IncidentAlert>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct IncidentStats {
    pub total: i64,
    pub open: i64,
    pub acknowledged: i64,
    pub resolved: i64,
}

// ── Handlers ───────────────────────────────────────────────────────────

/// GET /api/incidents — list incidents
pub async fn list_incidents(
    State(state): State<IncidentsState>,
    Query(query): Query<IncidentsQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;

    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        "SELECT id, name, status, severity, host_id, first_seen, last_seen, alert_count, summary, assigned_to, created_at FROM incident_groups WHERE 1=1"
    );

    if let Some(ref status) = query.status {
        builder.push(" AND status = ");
        builder.push_bind(status.clone());
    }
    if let Some(ref host_id) = query.host_id {
        builder.push(" AND host_id = ");
        builder.push_bind(host_id.clone());
    }
    if let Some(ref severity) = query.severity {
        builder.push(" AND severity = ");
        builder.push_bind(severity.clone());
    }

    builder.push(" ORDER BY last_seen DESC LIMIT ");
    builder.push_bind(per_page as i64);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let q = builder.build_query_as::<Incident>();

    match q.fetch_all(&state.pool).await {
        Ok(incidents) => (StatusCode::OK, Json(incidents)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/incidents/:id — get incident detail
pub async fn get_incident(
    Path(incident_id): Path<String>,
    State(state): State<IncidentsState>,
) -> impl IntoResponse {
    let incident = sqlx::query_as::<_, Incident>(
        "SELECT id, name, status, severity, host_id, first_seen, last_seen, alert_count, summary, assigned_to, created_at FROM incident_groups WHERE id = ?"
    )
    .bind(&incident_id)
    .fetch_optional(&state.pool)
    .await;

    match incident {
        Ok(Some(inc)) => {
            let alerts = sqlx::query_as::<_, IncidentAlert>(
                "SELECT id, incident_id, alert_id, alert_message, alert_severity, triggered_at FROM incident_alerts WHERE incident_id = ? ORDER BY triggered_at DESC"
            )
            .bind(&incident_id)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

            let detail = IncidentDetail { incident: inc, alerts };
            (StatusCode::OK, Json(detail)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "incident not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// PUT /api/incidents/:id — update incident
pub async fn update_incident(
    Path(incident_id): Path<String>,
    State(state): State<IncidentsState>,
    Json(req): Json<UpdateIncidentRequest>,
) -> impl IntoResponse {
    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new("UPDATE incident_groups SET ");

    let mut first = true;
    if let Some(status) = &req.status {
        builder.push("status = ");
        builder.push_bind(status.clone());
        first = false;
    }
    if let Some(assigned_to) = &req.assigned_to {
        if !first { builder.push(", "); }
        builder.push("assigned_to = ");
        builder.push_bind(assigned_to.clone());
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
    builder.push_bind(incident_id.clone());

    match builder.build().execute(&state.pool).await {
        Ok(_) => {
            let incident = sqlx::query_as::<_, Incident>(
                "SELECT id, name, status, severity, host_id, first_seen, last_seen, alert_count, summary, assigned_to, created_at FROM incident_groups WHERE id = ?"
            )
            .bind(&incident_id)
            .fetch_one(&state.pool)
            .await;
            match incident {
                Ok(i) => (StatusCode::OK, Json(i)).into_response(),
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

/// POST /api/incidents/:id/assign — assign incident
pub async fn assign_incident(
    Path(incident_id): Path<String>,
    State(state): State<IncidentsState>,
    Json(req): Json<AssignIncidentRequest>,
) -> impl IntoResponse {
    let result = sqlx::query("UPDATE incident_groups SET assigned_to = ? WHERE id = ?")
        .bind(&req.assigned_to)
        .bind(&incident_id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "assigned"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/incidents/stats — incident statistics
pub async fn incident_stats(
    State(state): State<IncidentsState>,
) -> impl IntoResponse {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM incident_groups")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let open: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM incident_groups WHERE status = 'open'")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let acknowledged: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM incident_groups WHERE status = 'acknowledged'")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let resolved: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM incident_groups WHERE status = 'resolved'")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let stats = IncidentStats {
        total: total.0,
        open: open.0,
        acknowledged: acknowledged.0,
        resolved: resolved.0,
    };

    (StatusCode::OK, Json(stats)).into_response()
}

/// Build the incidents routes sub-router.
pub fn incidents_routes(pool: SqlitePool) -> Router {
    use axum::routing::{get, post};

    let state = IncidentsState { pool };

    Router::new()
        .route("/api/incidents", get(list_incidents))
        .route("/api/incidents/stats", get(incident_stats))
        .route("/api/incidents/{id}", get(get_incident).put(update_incident))
        .route("/api/incidents/{id}/assign", post(assign_incident))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS incident_groups (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'open',
                severity TEXT NOT NULL, host_id TEXT NOT NULL DEFAULT '',
                first_seen TEXT NOT NULL DEFAULT (datetime('now')),
                last_seen TEXT NOT NULL DEFAULT (datetime('now')),
                alert_count INTEGER NOT NULL DEFAULT 0,
                summary TEXT NOT NULL DEFAULT '',
                assigned_to TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS incident_alerts (
                id TEXT PRIMARY KEY, incident_id TEXT NOT NULL,
                alert_id TEXT NOT NULL, alert_message TEXT NOT NULL,
                alert_severity TEXT NOT NULL, triggered_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_list_incidents_empty() {
        let pool = setup().await;
        let state = IncidentsState { pool };
        let resp = list_incidents(State(state), Query(IncidentsQuery {
            status: None, host_id: None, severity: None, page: None, per_page: None,
        })).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
