//! Health check routes.

use axum::{response::IntoResponse, Json, Router};

/// Build the health routes sub-router.
pub fn health_routes() -> Router {
    use axum::routing::get;
    Router::new().route("/api/v1/health", get(health_handler))
}

/// GET /api/v1/health — health check endpoint.
pub async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_returns_ok() {
        let app = health_routes();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/health")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();
        assert_eq!(body["status"], "ok");
        assert!(body["version"].is_string());
    }

    #[tokio::test]
    async fn test_health_content_type() {
        let app = health_routes();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/health")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let content_type = resp.headers().get("content-type").unwrap().to_str().unwrap();
        assert!(content_type.contains("json"));
    }

    #[tokio::test]
    async fn test_health_version_present() {
        let app = health_routes();
        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/health")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();
        let version = body["version"].as_str().unwrap();
        assert!(!version.is_empty(), "version must not be empty");
    }
}
