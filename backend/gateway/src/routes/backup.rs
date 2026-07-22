//! Backup & restore routes — export/import all system config as JSON.

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
    Router,
    routing::{get, post},
};
use sqlx::SqlitePool;
use crate::backup;

/// Shared state for backup routes.
#[derive(Clone)]
pub struct BackupState {
    pub pool: SqlitePool,
}

pub fn backup_routes(pool: SqlitePool) -> Router {
    let state = BackupState { pool };
    Router::new()
        .route("/api/backup/export", get(export_backup_handler))
        .route("/api/backup/import", post(import_backup_handler))
        .with_state(state)
}

async fn export_backup_handler(
    State(state): State<BackupState>,
) -> impl IntoResponse {
    match backup::export_backup(&state.pool).await {
        Ok(data) => Json(serde_json::json!({
            "status": "ok",
            "data": data
        })),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}

async fn import_backup_handler(
    State(state): State<BackupState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let backup_data: backup::SystemBackup = match serde_json::from_value(body) {
        Ok(b) => b,
        Err(e) => return Json(serde_json::json!({
            "status": "error",
            "message": format!("Invalid backup format: {}", e)
        })),
    };

    match backup::import_backup(&state.pool, &backup_data).await {
        Ok(results) => Json(serde_json::json!({
            "status": "ok",
            "results": results
        })),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use tower::ServiceExt;

    async fn setup_pool_with_tables() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
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
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO config (key, value) VALUES ('test_key', 'test_value')"
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    #[tokio::test]
    async fn test_export_backup() {
        let pool = setup_pool_with_tables().await;
        let app = backup_routes(pool);

        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/backup/export")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();
        assert_eq!(body["status"], "ok");
        let data = &body["data"];
        assert!(data["version"].is_string());
        assert!(data["tables"]["config"].is_array());
    }

    #[tokio::test]
    async fn test_export_contains_config_data() {
        let pool = setup_pool_with_tables().await;
        let app = backup_routes(pool);

        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/backup/export")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();
        let config_rows = body["data"]["tables"]["config"].as_array().unwrap();
        assert!(!config_rows.is_empty());
        assert_eq!(config_rows[0]["key"], "test_key");
        assert_eq!(config_rows[0]["value"], "test_value");
    }

    #[tokio::test]
    async fn test_import_invalid_body() {
        let pool = setup_pool_with_tables().await;
        let app = backup_routes(pool);

        let req_body = serde_json::json!({"not": "a backup"});
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/backup/import")
            .header("content-type", "application/json")
            .body(Body::from(req_body.to_string()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();
        assert_eq!(body["status"], "error");
    }

    #[tokio::test]
    async fn test_export_created_at_present() {
        let pool = setup_pool_with_tables().await;
        let app = backup_routes(pool);

        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/backup/export")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();
        assert!(body["data"]["created_at"].is_string());
        assert!(!body["data"]["created_at"].as_str().unwrap().is_empty());
    }
}
