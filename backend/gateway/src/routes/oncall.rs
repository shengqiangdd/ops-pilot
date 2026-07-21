use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::Sqlite;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct OnCallState {
    pub pool: SqlitePool,
}

#[derive(Debug, Deserialize)]
pub struct CreateScheduleRequest {
    pub name: String,
    pub description: Option<String>,
    pub timezone: Option<String>,
    pub rotation_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateShiftRequest {
    pub user_id: String,
    pub start_time: String,
    pub end_time: String,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOverrideRequest {
    pub user_id: String,
    pub date: String,
    pub reason: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct OnCallSchedule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub timezone: String,
    pub rotation_type: String,
    pub starts_at: String,
    pub ends_at: String,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct OnCallShift {
    pub id: String,
    pub schedule_id: String,
    pub user_id: String,
    pub start_time: String,
    pub end_time: String,
    pub role: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct OnCallOverride {
    pub id: String,
    pub schedule_id: String,
    pub user_id: String,
    pub date: String,
    pub reason: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct OnCallEscalation {
    pub id: String,
    pub alert_id: String,
    pub shift_id: String,
    pub notified_at: String,
    pub acknowledged_at: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct CurrentOnCall {
    pub schedule_name: String,
    pub primary_user: String,
    pub secondary_user: Option<String>,
    pub start_time: String,
    pub end_time: String,
}

/// GET /api/oncall/schedules — list schedules
pub async fn list_schedules(
    State(state): State<OnCallState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, OnCallSchedule>(
        "SELECT id, name, description, timezone, rotation_type, starts_at, ends_at, enabled, created_at FROM oncall_schedules ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await;
    match result {
        Ok(schedules) => (StatusCode::OK, Json(schedules)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// POST /api/oncall/schedules — create schedule
pub async fn create_schedule(
    State(state): State<OnCallState>,
    Json(req): Json<CreateScheduleRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let result = sqlx::query(
        "INSERT INTO oncall_schedules (id, name, description, timezone, rotation_type) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(req.description.as_deref().unwrap_or(""))
    .bind(req.timezone.as_deref().unwrap_or("UTC"))
    .bind(req.rotation_type.as_deref().unwrap_or("weekly"))
    .execute(&state.pool)
    .await;
    match result {
        Ok(_) => {
            let schedule = sqlx::query_as::<_, OnCallSchedule>(
                "SELECT id, name, description, timezone, rotation_type, starts_at, ends_at, enabled, created_at FROM oncall_schedules WHERE id = ?"
            ).bind(&id).fetch_one(&state.pool).await;
            match schedule { Ok(s) => (StatusCode::CREATED, Json(s)).into_response(), Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response() }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/oncall/schedules/:id — get schedule
pub async fn get_schedule(
    Path(schedule_id): Path<String>,
    State(state): State<OnCallState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, OnCallSchedule>(
        "SELECT id, name, description, timezone, rotation_type, starts_at, ends_at, enabled, created_at FROM oncall_schedules WHERE id = ?"
    ).bind(&schedule_id).fetch_optional(&state.pool).await;
    match result {
        Ok(Some(s)) => (StatusCode::OK, Json(s)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "schedule not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// PUT /api/oncall/schedules/:id — update schedule
pub async fn update_schedule(
    Path(schedule_id): Path<String>,
    State(state): State<OnCallState>,
    Json(req): Json<CreateScheduleRequest>,
) -> impl IntoResponse {
    let result = sqlx::query("UPDATE oncall_schedules SET name = ?, description = ?, timezone = ?, rotation_type = ? WHERE id = ?")
        .bind(&req.name)
        .bind(req.description.as_deref().unwrap_or(""))
        .bind(req.timezone.as_deref().unwrap_or("UTC"))
        .bind(req.rotation_type.as_deref().unwrap_or("weekly"))
        .bind(&schedule_id)
        .execute(&state.pool)
        .await;
    match result {
        Ok(_) => {
            let schedule = sqlx::query_as::<_, OnCallSchedule>(
                "SELECT id, name, description, timezone, rotation_type, starts_at, ends_at, enabled, created_at FROM oncall_schedules WHERE id = ?"
            ).bind(&schedule_id).fetch_one(&state.pool).await;
            match schedule { Ok(s) => (StatusCode::OK, Json(s)).into_response(), Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response() }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// DELETE /api/oncall/schedules/:id
pub async fn delete_schedule(
    Path(schedule_id): Path<String>,
    State(state): State<OnCallState>,
) -> impl IntoResponse {
    let _ = sqlx::query("DELETE FROM oncall_shifts WHERE schedule_id = ?").bind(&schedule_id).execute(&state.pool).await;
    let result = sqlx::query("DELETE FROM oncall_schedules WHERE id = ?").bind(&schedule_id).execute(&state.pool).await;
    match result { Ok(_) => (StatusCode::NO_CONTENT).into_response(), Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response() }
}

/// POST /api/oncall/schedules/:id/shifts — create shift
pub async fn create_shift(
    Path(schedule_id): Path<String>,
    State(state): State<OnCallState>,
    Json(req): Json<CreateShiftRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let result = sqlx::query(
        "INSERT INTO oncall_shifts (id, schedule_id, user_id, start_time, end_time, role) VALUES (?, ?, ?, ?, ?, ?)"
    ).bind(&id).bind(&schedule_id).bind(&req.user_id).bind(&req.start_time).bind(&req.end_time).bind(req.role.as_deref().unwrap_or("primary")).execute(&state.pool).await;
    match result {
        Ok(_) => {
            let shift = sqlx::query_as::<_, OnCallShift>(
                "SELECT id, schedule_id, user_id, start_time, end_time, role FROM oncall_shifts WHERE id = ?"
            ).bind(&id).fetch_one(&state.pool).await;
            match shift { Ok(s) => (StatusCode::CREATED, Json(s)).into_response(), Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response() }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/oncall/shifts — list shifts
pub async fn list_shifts(
    State(state): State<OnCallState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, OnCallShift>(
        "SELECT id, schedule_id, user_id, start_time, end_time, role FROM oncall_shifts ORDER BY start_time DESC"
    ).fetch_all(&state.pool).await;
    match result {
        Ok(shifts) => (StatusCode::OK, Json(shifts)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/oncall/current — current on-call
pub async fn current_oncall(
    State(state): State<OnCallState>,
) -> impl IntoResponse {
    let now = chrono::Utc::now().to_rfc3339();
    let result = sqlx::query_as::<_, OnCallShift>(
        "SELECT id, schedule_id, user_id, start_time, end_time, role FROM oncall_shifts WHERE start_time <= ? AND end_time >= ? ORDER BY start_time DESC LIMIT 1"
    ).bind(&now).bind(&now).fetch_optional(&state.pool).await;
    match result {
        Ok(Some(shift)) => {
            let schedule = sqlx::query_as::<_, OnCallSchedule>(
                "SELECT id, name, description, timezone, rotation_type, starts_at, ends_at, enabled, created_at FROM oncall_schedules WHERE id = ?"
            ).bind(&shift.schedule_id).fetch_optional(&state.pool).await.unwrap_or(None);
            let schedule_name = schedule.map(|s| s.name).unwrap_or_else(|| "Unknown".to_string());
            let current = CurrentOnCall {
                schedule_name,
                primary_user: shift.user_id.clone(),
                secondary_user: None,
                start_time: shift.start_time,
                end_time: shift.end_time,
            };
            (StatusCode::OK, Json(current)).into_response()
        }
        Ok(None) => (StatusCode::OK, Json(serde_json::json!({"message": "No on-call shift currently active"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// POST /api/oncall/overrides — create override
pub async fn create_override(
    State(state): State<OnCallState>,
    Json(req): Json<CreateOverrideRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let result = sqlx::query("INSERT INTO oncall_overrides (id, schedule_id, user_id, date, reason) VALUES (?, ?, ?, ?, ?)")
        .bind(&id).bind("default").bind(&req.user_id).bind(&req.date).bind(&req.reason).execute(&state.pool).await;
    match result {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"id": id, "status": "created"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/oncall/escalations — list escalations
pub async fn list_escalations(
    State(state): State<OnCallState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, OnCallEscalation>(
        "SELECT id, alert_id, shift_id, notified_at, acknowledged_at, status FROM oncall_escalations ORDER BY notified_at DESC"
    ).fetch_all(&state.pool).await;
    match result {
        Ok(escalations) => (StatusCode::OK, Json(escalations)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

pub fn oncall_routes(pool: SqlitePool) -> Router {
    use axum::routing::{delete, get, post, put};
    let state = OnCallState { pool };
    Router::new()
        .route("/api/oncall/schedules", get(list_schedules).post(create_schedule))
        .route("/api/oncall/schedules/{id}", get(get_schedule).put(update_schedule).delete(delete_schedule))
        .route("/api/oncall/schedules/{id}/shifts", post(create_shift))
        .route("/api/oncall/shifts", get(list_shifts))
        .route("/api/oncall/current", get(current_oncall))
        .route("/api/oncall/overrides", post(create_override))
        .route("/api/oncall/escalations", get(list_escalations))
        .with_state(state)
}
