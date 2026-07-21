use std::sync::Arc;
use axum::{Extension, Json, Router, routing::{get, post}, response::IntoResponse};
use crate::AppState;
use crate::report_generator;

pub fn report_routes() -> Router {
    Router::new()
        .route("/api/reports/generate", post(generate_report))
        .route("/api/reports/list", get(list_reports))
        .route("/api/reports/download/{id}", get(download_report))
}

async fn generate_report(
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    match report_generator::generate_daily_report(&state.pool).await {
        Ok(report) => Json(serde_json::json!({
            "status": "ok",
            "report": report
        })),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}

async fn list_reports(
    Extension(state): Extension<Arc<AppState>>,
) -> impl IntoResponse {
    match report_generator::list_reports(&state.pool).await {
        Ok(reports) => Json(serde_json::json!({
            "status": "ok",
            "reports": reports
        })),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}

async fn download_report(
    Extension(state): Extension<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    // Return the report as JSON download
    // In production, generate PDF here
    let reports = report_generator::list_reports(&state.pool).await.unwrap_or_default();
    if let Some(report) = reports.into_iter().find(|r| r.id == id) {
        Json(serde_json::json!({
            "status": "ok",
            "report": report
        }))
    } else {
        Json(serde_json::json!({
            "status": "error",
            "message": "Report not found"
        }))
    }
}
