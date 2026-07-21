use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::Sqlite;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct FinOpsState {
    pub pool: SqlitePool,
}

#[derive(Debug, Deserialize)]
pub struct CostsQuery {
    pub provider: Option<String>,
    pub service: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBudgetRequest {
    pub name: String,
    pub amount: f64,
    pub period: String,
    pub start_date: String,
    pub end_date: String,
    pub notify_threshold: Option<f64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CostRecord {
    pub id: String,
    pub source_id: String,
    pub service: String,
    pub region: String,
    pub resource_id: String,
    pub cost_amount: f64,
    pub currency: String,
    pub usage_quantity: f64,
    pub usage_unit: String,
    pub record_date: String,
    pub tags_json: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CostBudget {
    pub id: String,
    pub name: String,
    pub amount: f64,
    pub period: String,
    pub start_date: String,
    pub end_date: String,
    pub notify_threshold: f64,
    pub actual_spend: f64,
    pub forecast_spend: f64,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct CostOverview {
    pub total_spend_this_month: f64,
    pub total_spend_last_month: f64,
    pub month_over_month_change: f64,
    pub budget_total: f64,
    pub budget_actual: f64,
    pub forecast_spend: f64,
    pub currency: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CostByService {
    pub service: String,
    pub total: f64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CostByProvider {
    pub provider: String,
    pub total: f64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CostBudgetRow {
    pub id: String,
    pub name: String,
    pub amount: f64,
    pub period: String,
    pub start_date: String,
    pub end_date: String,
    pub notify_threshold: f64,
    pub actual_spend: f64,
    pub forecast_spend: f64,
    pub status: String,
}

fn pseudo_random(seed: u64) -> f64 {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64;
    ((nanos.wrapping_mul(6364136223846793005).wrapping_add(seed)) % 1000) as f64 / 1000.0
}

/// GET /api/finops/overview — cost overview
pub async fn cost_overview(
    State(state): State<FinOpsState>,
) -> impl IntoResponse {
    let total_this_month: (f64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(cost_amount), 0) FROM cost_records WHERE record_date >= date('now', 'start of month')"
    ).fetch_one(&state.pool).await.unwrap_or((0.0,));

    let total_last_month: (f64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(cost_amount), 0) FROM cost_records WHERE record_date >= date('now', 'start of month', '-1 month') AND record_date < date('now', 'start of month')"
    ).fetch_one(&state.pool).await.unwrap_or((0.0,));

    let budget_total: (f64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(amount), 0) FROM cost_budgets WHERE start_date <= date('now') AND end_date >= date('now')"
    ).fetch_one(&state.pool).await.unwrap_or((0.0,));

    let budget_actual: (f64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(actual_spend), 0) FROM cost_budgets WHERE start_date <= date('now') AND end_date >= date('now')"
    ).fetch_one(&state.pool).await.unwrap_or((0.0,));

    let mom_change = if total_last_month.0 > 0.0 {
        ((total_this_month.0 - total_last_month.0) / total_last_month.0) * 100.0
    } else {
        0.0
    };

    // Simple forecast: project current month's trend
    let day_of_month = chrono::Utc::now().day() as f64;
    let days_in_month = 30.0;
    let forecast = if day_of_month > 0.0 {
        (total_this_month.0 / day_of_month) * days_in_month
    } else {
        0.0
    };

    let overview = CostOverview {
        total_spend_this_month: total_this_month.0,
        total_spend_last_month: total_last_month.0,
        month_over_month_change: mom_change,
        budget_total: budget_total.0,
        budget_actual: budget_actual.0,
        forecast_spend: forecast,
        currency: "CNY".to_string(),
    };

    (StatusCode::OK, Json(overview)).into_response()
}

/// GET /api/finops/costs — list cost records
pub async fn list_costs(
    State(state): State<FinOpsState>,
    Query(query): Query<CostsQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;

    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        "SELECT id, source_id, service, region, resource_id, cost_amount, currency, usage_quantity, usage_unit, record_date, tags_json FROM cost_records WHERE 1=1"
    );

    if let Some(ref provider) = query.provider {
        builder.push(" AND source_id IN (SELECT id FROM cost_sources WHERE provider = ");
        builder.push_bind(provider.clone());
        builder.push(")");
    }
    if let Some(ref service) = query.service {
        builder.push(" AND service = ");
        builder.push_bind(service.clone());
    }
    if let Some(ref from) = query.from {
        builder.push(" AND record_date >= ");
        builder.push_bind(from.clone());
    }
    if let Some(ref to) = query.to {
        builder.push(" AND record_date <= ");
        builder.push_bind(to.clone());
    }

    builder.push(" ORDER BY record_date DESC LIMIT ");
    builder.push_bind(per_page as i64);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let q = builder.build_query_as::<CostRecord>();
    match q.fetch_all(&state.pool).await {
        Ok(costs) => (StatusCode::OK, Json(costs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/finops/costs/by-service — cost by service
pub async fn cost_by_service(
    State(state): State<FinOpsState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, CostByService>(
        "SELECT service, SUM(cost_amount) as total FROM cost_records GROUP BY service ORDER BY total DESC"
    ).fetch_all(&state.pool).await;
    match result {
        Ok(costs) => (StatusCode::OK, Json(costs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/finops/costs/by-provider — cost by provider
pub async fn cost_by_provider(
    State(state): State<FinOpsState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, CostByProvider>(
        "SELECT cs.provider, SUM(cr.cost_amount) as total FROM cost_records cr JOIN cost_sources cs ON cr.source_id = cs.id GROUP BY cs.provider ORDER BY total DESC"
    ).fetch_all(&state.pool).await;
    match result {
        Ok(costs) => (StatusCode::OK, Json(costs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/finops/budgets — list budgets
pub async fn list_budgets(
    State(state): State<FinOpsState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, CostBudgetRow>(
        "SELECT id, name, amount, period, start_date, end_date, notify_threshold, actual_spend, forecast_spend, status FROM cost_budgets ORDER BY created_at DESC"
    ).fetch_all(&state.pool).await;
    match result {
        Ok(budgets) => (StatusCode::OK, Json(budgets)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// POST /api/finops/budgets — create budget
pub async fn create_budget(
    State(state): State<FinOpsState>,
    Json(req): Json<CreateBudgetRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let result = sqlx::query(
        "INSERT INTO cost_budgets (id, name, amount, period, start_date, end_date, notify_threshold) VALUES (?, ?, ?, ?, ?, ?, ?)"
    ).bind(&id).bind(&req.name).bind(req.amount).bind(&req.period)
        .bind(&req.start_date).bind(&req.end_date).bind(req.notify_threshold.unwrap_or(0.8))
        .execute(&state.pool).await;
    match result {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"id": id, "status": "created"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// DELETE /api/finops/budgets/:id — delete budget
pub async fn delete_budget(
    Path(budget_id): Path<String>,
    State(state): State<FinOpsState>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM cost_budgets WHERE id = ?").bind(&budget_id).execute(&state.pool).await;
    match result { Ok(_) => (StatusCode::NO_CONTENT).into_response(), Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response() }
}

/// GET /api/finops/forecast — cost forecast
pub async fn cost_forecast(
    State(state): State<FinOpsState>,
) -> impl IntoResponse {
    // Simple forecast based on historical data
    let result = sqlx::query_as::<_, (String, f64)>(
        "SELECT record_date, SUM(cost_amount) FROM cost_records WHERE record_date >= date('now', '-3 months') GROUP BY record_date ORDER BY record_date"
    ).fetch_all(&state.pool).await.unwrap_or_default();

    let null_f64: Option<f64> = None;
    let data_points: Vec<serde_json::Value> = result.iter().map(|(date, amount)| {
        serde_json::json!({"date": date, "actual": amount, "predicted": null_f64})
    }).collect();

    // Add forecast points
    let today = chrono::Utc::now();
    for i in 1..=30 {
        let date = (today + chrono::Duration::days(i)).format("%Y-%m-%d").to_string();
        let predicted = data_points.last().map(|d| d["actual"].as_f64().unwrap_or(0.0) * (1.0 + 0.01 * i as f64)).unwrap_or(0.0);
        let mut points = data_points.clone();
        points.push(serde_json::json!({"date": date, "actual": null_f64, "predicted": predicted}));
    }

    (StatusCode::OK, Json(serde_json::json!({
        "historical": data_points,
        "forecast_days": 30,
    }))).into_response()
}

pub fn finops_routes(pool: SqlitePool) -> Router {
    use axum::routing::{delete, get};
    let state = FinOpsState { pool };
    Router::new()
        .route("/api/finops/overview", get(cost_overview))
        .route("/api/finops/costs", get(list_costs))
        .route("/api/finops/costs/by-service", get(cost_by_service))
        .route("/api/finops/costs/by-provider", get(cost_by_provider))
        .route("/api/finops/budgets", get(list_budgets).post(create_budget))
        .route("/api/finops/budgets/{id}", delete(delete_budget))
        .route("/api/finops/forecast", get(cost_forecast))
        .with_state(state)
}
