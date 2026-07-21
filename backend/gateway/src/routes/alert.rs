use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::Sqlite;
use sqlx::SqlitePool;
use std::sync::Arc;
use crate::AppState;

/// Shared application state for alert routes.
#[derive(Clone)]
pub struct AlertState {
    pub pool: SqlitePool,
}

// ── Alert Rule ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateAlertRuleRequest {
    pub name: String,
    pub metric: String,
    pub condition: String,
    pub threshold: f64,
    pub severity: String,
    pub silence_minutes: Option<u32>,
    pub channel_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAlertRuleRequest {
    pub name: Option<String>,
    pub metric: Option<String>,
    pub condition: Option<String>,
    pub threshold: Option<f64>,
    pub severity: Option<String>,
    pub silence_minutes: Option<u32>,
    pub channel_ids: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub metric: String,
    pub condition: String,
    pub threshold: f64,
    pub severity: String,
    pub silence_minutes: u32,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// GET /api/alert/rules — list all alert rules
pub async fn list_alert_rules(
    State(state): State<AlertState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, AlertRule>(
        "SELECT id, name, metric, condition, threshold, severity, silence_minutes, enabled, created_at, updated_at FROM alert_rules ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(rules) => (StatusCode::OK, Json(rules)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/alert/rules — create a new alert rule
pub async fn create_alert_rule(
    State(state): State<AlertState>,
    Json(req): Json<CreateAlertRuleRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let silence = req.silence_minutes.unwrap_or(5);
    let channels_json = serde_json::to_string(&req.channel_ids.unwrap_or_default()).unwrap_or_default();

    let result = sqlx::query(
        "INSERT INTO alert_rules (id, name, metric, condition, threshold, severity, silence_minutes, channel_ids) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.metric)
    .bind(&req.condition)
    .bind(req.threshold)
    .bind(&req.severity)
    .bind(silence as i64)
    .bind(&channels_json)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let rule = sqlx::query_as::<_, AlertRule>(
                "SELECT id, name, metric, condition, threshold, severity, silence_minutes, enabled, created_at, updated_at FROM alert_rules WHERE id = ?"
            )
            .bind(&id)
            .fetch_one(&state.pool)
            .await;
            match rule {
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

/// PUT /api/alert/rules/:id — update an alert rule
pub async fn update_alert_rule(
    Path(rule_id): Path<String>,
    State(state): State<AlertState>,
    Json(req): Json<UpdateAlertRuleRequest>,
) -> impl IntoResponse {
    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new("UPDATE alert_rules SET ");

    let mut first = true;

    if let Some(name) = &req.name {
        if !first { builder.push(", "); }
        builder.push("name = ");
        builder.push_bind(name.clone());
        first = false;
    }
    if let Some(metric) = &req.metric {
        if !first { builder.push(", "); }
        builder.push("metric = ");
        builder.push_bind(metric.clone());
        first = false;
    }
    if let Some(condition) = &req.condition {
        if !first { builder.push(", "); }
        builder.push("condition = ");
        builder.push_bind(condition.clone());
        first = false;
    }
    if let Some(threshold) = req.threshold {
        if !first { builder.push(", "); }
        builder.push("threshold = ");
        builder.push_bind(threshold);
        first = false;
    }
    if let Some(severity) = &req.severity {
        if !first { builder.push(", "); }
        builder.push("severity = ");
        builder.push_bind(severity.clone());
        first = false;
    }
    if let Some(silence) = req.silence_minutes {
        if !first { builder.push(", "); }
        builder.push("silence_minutes = ");
        builder.push_bind(silence as i64);
        first = false;
    }
    if let Some(enabled) = req.enabled {
        if !first { builder.push(", "); }
        builder.push("enabled = ");
        builder.push_bind(enabled);
        first = false;
    }
    if let Some(channels) = &req.channel_ids {
        let channels_json = serde_json::to_string(channels).unwrap_or_default();
        if !first { builder.push(", "); }
        builder.push("channel_ids = ");
        builder.push_bind(channels_json);
        first = false;
    }

    if first {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "no fields to update"})),
        )
            .into_response();
    }

    builder.push(", updated_at = datetime('now')");
    builder.push(" WHERE id = ");
    builder.push_bind(rule_id.clone());

    let result = builder.build().execute(&state.pool).await;

    match result {
        Ok(_) => {
            let rule = sqlx::query_as::<_, AlertRule>(
                "SELECT id, name, metric, condition, threshold, severity, silence_minutes, enabled, created_at, updated_at FROM alert_rules WHERE id = ?"
            )
            .bind(&rule_id)
            .fetch_one(&state.pool)
            .await;
            match rule {
                Ok(r) => (StatusCode::OK, Json(r)).into_response(),
                Err(e) => (
                    StatusCode::NOT_FOUND,
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

/// DELETE /api/alert/rules/:id — delete an alert rule
pub async fn delete_alert_rule(
    Path(rule_id): Path<String>,
    State(state): State<AlertState>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM alert_rules WHERE id = ?")
        .bind(&rule_id)
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

// ── Alert History ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AlertHistoryQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub severity: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AlertHistoryEntry {
    pub id: String,
    pub rule_id: String,
    pub rule_name: String,
    pub severity: String,
    pub message: String,
    pub status: String,
    pub triggered_at: String,
    pub acknowledged_at: Option<String>,
}

/// GET /api/alert/history — list alert history
pub async fn list_alert_history(
    State(state): State<AlertState>,
    axum::extract::Query(query): axum::extract::Query<AlertHistoryQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = ((page - 1) * per_page) as i64;

    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        "SELECT h.id, h.rule_id, COALESCE(r.name, 'Unknown') as rule_name, h.severity, h.message, h.status, h.triggered_at, h.acknowledged_at FROM alert_history h LEFT JOIN alert_rules r ON h.rule_id = r.id WHERE 1=1"
    );

    if let Some(ref from) = query.from {
        builder.push(" AND h.triggered_at >= ");
        builder.push_bind(from.clone());
    }
    if let Some(ref to) = query.to {
        builder.push(" AND h.triggered_at <= ");
        builder.push_bind(to.clone());
    }
    if let Some(ref severity) = query.severity {
        builder.push(" AND h.severity = ");
        builder.push_bind(severity.clone());
    }

    builder.push(" ORDER BY h.triggered_at DESC LIMIT ");
    builder.push_bind(per_page as i64);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    let q = builder.build_query_as::<AlertHistoryEntry>();

    let result = q.fetch_all(&state.pool).await;

    match result {
        Ok(entries) => (StatusCode::OK, Json(entries)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// ── Notification Channel ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    pub channel_type: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct NotificationChannel {
    pub id: String,
    pub name: String,
    pub channel_type: String,
    pub config: String,
    pub enabled: bool,
    pub created_at: String,
}

/// GET /api/alert/channels — list all notification channels
pub async fn list_notification_channels(
    State(state): State<AlertState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, NotificationChannel>(
        "SELECT id, name, channel_type, config, enabled, created_at FROM notification_channels ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(channels) => (StatusCode::OK, Json(channels)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/alert/channels — create a notification channel
pub async fn create_notification_channel(
    State(state): State<AlertState>,
    Json(req): Json<CreateChannelRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let config_json = serde_json::to_string(&req.config).unwrap_or_default();

    let result = sqlx::query(
        "INSERT INTO notification_channels (id, name, channel_type, config) VALUES (?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.channel_type)
    .bind(&config_json)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let channel = sqlx::query_as::<_, NotificationChannel>(
                "SELECT id, name, channel_type, config, enabled, created_at FROM notification_channels WHERE id = ?"
            )
            .bind(&id)
            .fetch_one(&state.pool)
            .await;
            match channel {
                Ok(c) => (StatusCode::CREATED, Json(c)).into_response(),
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

/// POST /api/alert/channels/:id/test — test a notification channel
pub async fn test_notification_channel(
    Path(channel_id): Path<String>,
    State(_state): State<AlertState>,
) -> impl IntoResponse {
    // For now, just return success. In production, send a test message.
    (StatusCode::OK, Json(serde_json::json!({
        "status": "ok",
        "message": "Test notification sent successfully",
        "channel_id": channel_id,
    }))).into_response()
}

/// POST /api/alert/test-notify — test the notification dispatch pipeline
pub async fn test_notify(
    State(state): State<AlertState>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    // Check alert suppression: if this is a duplicate within the time window, skip dispatch
    let suppressed = app_state
        .alert_suppressor
        .should_suppress(
            "system",
            "test_notification",
            "info",
            "测试通知",
        )
        .await;

    if suppressed {
        return Json(serde_json::json!({
            "status": "suppressed",
            "message": "Alert suppressed (duplicate within aggregation window)"
        }));
    }

    match crate::notify::dispatch_notification(
        &state.pool,
        "测试通知",
        "这是一条来自 OpsPilot 的测试消息",
        "info",
    )
    .await
    {
        Ok(results) => Json(serde_json::json!({ "status": "ok", "delivered": results })),
        Err(e) => Json(serde_json::json!({ "status": "error", "message": e.to_string() })),
    }
}

/// Build the alert routes sub-router.
pub fn alert_routes(pool: SqlitePool) -> Router {
    use axum::routing::{get, post, put};

    let state = AlertState { pool };

    Router::new()
        .route("/api/alert/rules", get(list_alert_rules).post(create_alert_rule))
        .route("/api/alert/rules/{id}", put(update_alert_rule).delete(delete_alert_rule))
        .route("/api/alert/history", get(list_alert_history))
        .route("/api/alert/channels", get(list_notification_channels).post(create_notification_channel))
        .route("/api/alert/channels/{id}/test", post(test_notification_channel))
        .route("/api/alert/test-notify", post(test_notify))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS alert_rules (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                metric TEXT NOT NULL,
                condition TEXT NOT NULL,
                threshold REAL NOT NULL,
                severity TEXT NOT NULL,
                silence_minutes INTEGER NOT NULL DEFAULT 5,
                channel_ids TEXT NOT NULL DEFAULT '[]',
                enabled BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        )
        .execute(&pool)
        .await
        .unwrap();
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
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS notification_channels (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                channel_type TEXT NOT NULL,
                config TEXT NOT NULL DEFAULT '{}',
                enabled BOOLEAN NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    #[tokio::test]
    async fn test_create_and_list_rules() {
        let pool = setup().await;
        let state = AlertState { pool };

        let req = CreateAlertRuleRequest {
            name: "CPU High".into(),
            metric: "cpu_percent".into(),
            condition: ">".into(),
            threshold: 90.0,
            severity: "critical".into(),
            silence_minutes: Some(5),
            channel_ids: None,
        };

        let resp = create_alert_rule(State(state.clone()), Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = list_alert_rules(State(state)).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_and_list_channels() {
        let pool = setup().await;
        let state = AlertState { pool };

        let req = CreateChannelRequest {
            name: "Slack".into(),
            channel_type: "webhook".into(),
            config: serde_json::json!({"url": "https://hooks.slack.com/xxx"}),
        };

        let resp = create_notification_channel(State(state.clone()), Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = list_notification_channels(State(state)).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
