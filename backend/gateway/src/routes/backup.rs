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
