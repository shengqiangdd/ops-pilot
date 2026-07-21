use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::Sqlite;
use sqlx::SqlitePool;

/// Shared application state for SLOs routes.
#[derive(Clone)]
pub struct SlosState {
    pub pool: SqlitePool,
}

// ── Request/Response Types ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SlosQuery {
    pub status: Option<String>,
    pub service_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSloRequest {
    pub name: String,
    pub description: Option<String>,
    pub service_id: Option<String>,
    pub sli_type: String,
    pub target_percentage: f64,
    pub window_days: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSloRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub target_percentage: Option<f64>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SLO {
    pub id: String,
    pub name: String,
    pub description: String,
    pub service_id: String,
    pub sli_type: String,
    pub target_percentage: f64,
    pub window_days: u32,
    pub current_sli: f64,
    pub error_budget_remaining: f64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SliEvent {
    pub id: String,
    pub slo_id: String,
    pub timestamp: String,
    pub sli_value: f64,
    pub budget_consumed: f64,
    pub breached: bool,
}

#[derive(Debug, Serialize)]
pub struct BurnRateAlert {
    pub slo_id: String,
    pub slo_name: String,
    pub burn_rate: f64,
    pub error_budget_remaining: f64,
    pub estimated_breach_hours: f64,
    pub severity: String,
    pub suggestion: String,
}

// ── Handlers ───────────────────────────────────────────────────────────

/// GET /api/slos — list SLOs
pub async fn list_slos(
    State(state): State<SlosState>,
    Query(query): Query<SlosQuery>,
) -> impl IntoResponse {
    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        "SELECT id, name, description, service_id, sli_type, target_percentage, window_days, current_sli, error_budget_remaining, status, created_at FROM slos WHERE 1=1"
    );

    if let Some(ref status) = query.status {
        builder.push(" AND status = ");
        builder.push_bind(status.clone());
    }
    if let Some(ref service_id) = query.service_id {
        builder.push(" AND service_id = ");
        builder.push_bind(service_id.clone());
    }

    builder.push(" ORDER BY created_at DESC");

    let q = builder.build_query_as::<SLO>();

    match q.fetch_all(&state.pool).await {
        Ok(slos) => (StatusCode::OK, Json(slos)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/slos — create SLO
pub async fn create_slo(
    State(state): State<SlosState>,
    Json(req): Json<CreateSloRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let window = req.window_days.unwrap_or(30);

    // Simulate initial SLI value
    let current_sli = 99.5 + (pseudo_random(id.len() as u64) * 0.5);
    let error_budget = 100.0 - req.target_percentage;
    let error_budget_remaining = error_budget * (current_sli - req.target_percentage + error_budget) / error_budget;

    let result = sqlx::query(
        "INSERT INTO slos (id, name, description, service_id, sli_type, target_percentage, window_days, current_sli, error_budget_remaining) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(req.description.as_deref().unwrap_or(""))
    .bind(req.service_id.as_deref().unwrap_or(""))
    .bind(&req.sli_type)
    .bind(req.target_percentage)
    .bind(window as i64)
    .bind(current_sli)
    .bind(error_budget_remaining.max(0.0))
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let slo = sqlx::query_as::<_, SLO>(
                "SELECT id, name, description, service_id, sli_type, target_percentage, window_days, current_sli, error_budget_remaining, status, created_at FROM slos WHERE id = ?"
            )
            .bind(&id)
            .fetch_one(&state.pool)
            .await;
            match slo {
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

/// GET /api/slos/:id — get SLO detail
pub async fn get_slo(
    Path(slo_id): Path<String>,
    State(state): State<SlosState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, SLO>(
        "SELECT id, name, description, service_id, sli_type, target_percentage, window_days, current_sli, error_budget_remaining, status, created_at FROM slos WHERE id = ?"
    )
    .bind(&slo_id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(slo)) => {
            let events = sqlx::query_as::<_, SliEvent>(
                "SELECT id, slo_id, timestamp, sli_value, budget_consumed, breached FROM sli_events WHERE slo_id = ? ORDER BY timestamp DESC LIMIT 100"
            )
            .bind(&slo_id)
            .fetch_all(&state.pool)
            .await
            .unwrap_or_default();

            let detail = serde_json::json!({
                "slo": slo,
                "events": events,
            });
            (StatusCode::OK, Json(detail)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "SLO not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// PUT /api/slos/:id — update SLO
pub async fn update_slo(
    Path(slo_id): Path<String>,
    State(state): State<SlosState>,
    Json(req): Json<UpdateSloRequest>,
) -> impl IntoResponse {
    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new("UPDATE slos SET ");

    let mut first = true;
    if let Some(name) = &req.name {
        builder.push("name = ");
        builder.push_bind(name.clone());
        first = false;
    }
    if let Some(description) = &req.description {
        if !first { builder.push(", "); }
        builder.push("description = ");
        builder.push_bind(description.clone());
        first = false;
    }
    if let Some(target) = req.target_percentage {
        if !first { builder.push(", "); }
        builder.push("target_percentage = ");
        builder.push_bind(target);
        first = false;
    }
    if let Some(status) = &req.status {
        if !first { builder.push(", "); }
        builder.push("status = ");
        builder.push_bind(status.clone());
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
    builder.push_bind(slo_id.clone());

    match builder.build().execute(&state.pool).await {
        Ok(_) => {
            let slo = sqlx::query_as::<_, SLO>(
                "SELECT id, name, description, service_id, sli_type, target_percentage, window_days, current_sli, error_budget_remaining, status, created_at FROM slos WHERE id = ?"
            )
            .bind(&slo_id)
            .fetch_one(&state.pool)
            .await;
            match slo {
                Ok(s) => (StatusCode::OK, Json(s)).into_response(),
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

/// DELETE /api/slos/:id — delete SLO
pub async fn delete_slo(
    Path(slo_id): Path<String>,
    State(state): State<SlosState>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM slos WHERE id = ?")
        .bind(&slo_id)
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

/// POST /api/slos/:id/evaluate — evaluate SLI
pub async fn evaluate_slo(
    Path(slo_id): Path<String>,
    State(state): State<SlosState>,
) -> impl IntoResponse {
    let slo = sqlx::query_as::<_, SLO>(
        "SELECT id, name, description, service_id, sli_type, target_percentage, window_days, current_sli, error_budget_remaining, status, created_at FROM slos WHERE id = ?"
    )
    .bind(&slo_id)
    .fetch_optional(&state.pool)
    .await;

    match slo {
        Ok(Some(mut s)) => {
            // Simulate SLI evaluation
            let new_sli = 99.0 + pseudo_random(s.id.len() as u64) * 1.0;
            let error_budget = 100.0 - s.target_percentage;
            let error_budget_remaining = (error_budget - (s.target_percentage - new_sli)).max(0.0);
            let status = if new_sli >= s.target_percentage {
                "compliant"
            } else if error_budget_remaining > error_budget * 0.5 {
                "at_risk"
            } else {
                "breached"
            };

            // Update SLO
            let _ = sqlx::query("UPDATE slos SET current_sli = ?, error_budget_remaining = ?, status = ? WHERE id = ?")
                .bind(new_sli)
                .bind(error_budget_remaining)
                .bind(status)
                .bind(&slo_id)
                .execute(&state.pool)
                .await;

            // Record event
            let event_id = uuid::Uuid::new_v4().to_string();
            let breached = new_sli < s.target_percentage;
            let budget_consumed = (s.target_percentage - new_sli).max(0.0);
            let _ = sqlx::query(
                "INSERT INTO sli_events (id, slo_id, sli_value, budget_consumed, breached) VALUES (?, ?, ?, ?, ?)"
            )
            .bind(&event_id)
            .bind(&slo_id)
            .bind(new_sli)
            .bind(budget_consumed)
            .bind(breached)
            .execute(&state.pool)
            .await;

            s.current_sli = new_sli;
            s.error_budget_remaining = error_budget_remaining;
            s.status = status.to_string();

            (StatusCode::OK, Json(s)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "SLO not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/slos/burn-rate — burn rate alerts
pub async fn burn_rate_alerts(
    State(state): State<SlosState>,
) -> impl IntoResponse {
    let slos = sqlx::query_as::<_, SLO>(
        "SELECT id, name, description, service_id, sli_type, target_percentage, window_days, current_sli, error_budget_remaining, status, created_at FROM slos WHERE status != 'compliant'"
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let mut alerts = Vec::new();
    for slo in slos {
        let error_budget = 100.0 - slo.target_percentage;
        let burn_rate = if error_budget > 0.0 {
            (error_budget - slo.error_budget_remaining) / error_budget * 100.0
        } else {
            100.0
        };

        let estimated_breach_hours = if burn_rate > 0.0 && slo.error_budget_remaining > 0.0 {
            slo.error_budget_remaining / (burn_rate / 24.0)
        } else {
            0.0
        };

        let severity = if burn_rate > 80.0 {
            "critical"
        } else if burn_rate > 50.0 {
            "warning"
        } else {
            "low"
        };

        let suggestion = match severity {
            "critical" => "Immediate action required! Error budget nearly exhausted.".to_string(),
            "warning" => "Consider reviewing recent incidents and optimizing.".to_string(),
            _ => "Monitor closely and investigate anomalies.".to_string(),
        };

        alerts.push(BurnRateAlert {
            slo_id: slo.id,
            slo_name: slo.name,
            burn_rate,
            error_budget_remaining: slo.error_budget_remaining,
            estimated_breach_hours,
            severity: severity.to_string(),
            suggestion,
        });
    }

    alerts.sort_by(|a, b| {
        b.burn_rate.partial_cmp(&a.burn_rate).unwrap_or(std::cmp::Ordering::Equal)
    });

    (StatusCode::OK, Json(alerts)).into_response()
}

/// Build the SLOs routes sub-router.
pub fn slos_routes(pool: SqlitePool) -> Router {
    use axum::routing::{get, post};

    let state = SlosState { pool };

    Router::new()
        .route("/api/slos", get(list_slos).post(create_slo))
        .route("/api/slos/burn-rate", get(burn_rate_alerts))
        .route("/api/slos/{id}", get(get_slo).put(update_slo).delete(delete_slo))
        .route("/api/slos/{id}/evaluate", post(evaluate_slo))
        .with_state(state)
}

fn pseudo_random(seed: u64) -> f64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    ((nanos.wrapping_mul(6364136223846793005).wrapping_add(seed)) % 1000) as f64 / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS slos (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT NOT NULL DEFAULT '',
                service_id TEXT NOT NULL DEFAULT '', sli_type TEXT NOT NULL,
                target_percentage REAL NOT NULL DEFAULT 99.9, window_days INTEGER NOT NULL DEFAULT 30,
                current_sli REAL NOT NULL DEFAULT 100.0,
                error_budget_remaining REAL NOT NULL DEFAULT 1.0,
                status TEXT NOT NULL DEFAULT 'compliant',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sli_events (
                id TEXT PRIMARY KEY, slo_id TEXT NOT NULL,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                sli_value REAL NOT NULL, budget_consumed REAL NOT NULL,
                breached BOOLEAN NOT NULL DEFAULT 0
            )"
        ).execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_create_and_list_slos() {
        let pool = setup().await;
        let state = SlosState { pool };

        let req = CreateSloRequest {
            name: "API Availability".into(),
            description: Some("99.9% availability SLA".into()),
            service_id: Some("svc-001".into()),
            sli_type: "availability".into(),
            target_percentage: 99.9,
            window_days: Some(30),
        };

        let resp = create_slo(State(state.clone()), Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = list_slos(State(state), Query(SlosQuery {
            status: None, service_id: None,
        })).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
