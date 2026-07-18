//! REST handlers for host management.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use ops_pilot_core::host::{CreateHost, HostService, UpdateHost};
use std::sync::Arc;

/// Shared application state for host routes.
#[derive(Clone)]
pub struct HostState {
    pub service: Arc<HostService>,
}

/// GET /api/hosts — list all hosts.
pub async fn list_hosts(State(state): State<HostState>) -> impl IntoResponse {
    match state.service.list().await {
        Ok(hosts) => (StatusCode::OK, Json(hosts)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/hosts/:id — get a single host by ID.
pub async fn get_host(
    State(state): State<HostState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.service.get(&id).await {
        Ok(host) => (StatusCode::OK, Json(host)).into_response(),
        Err(e) => {
            let status = match &e {
                ops_pilot_core::host::HostError::NotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(serde_json::json!({"error": e.to_string()}))).into_response()
        }
    }
}

/// POST /api/hosts — create a new host.
pub async fn create_host(
    State(state): State<HostState>,
    Json(input): Json<CreateHost>,
) -> impl IntoResponse {
    match state.service.create(input).await {
        Ok(host) => (StatusCode::CREATED, Json(host)).into_response(),
        Err(e) => {
            let status = match &e {
                ops_pilot_core::host::HostError::InvalidInput(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(serde_json::json!({"error": e.to_string()}))).into_response()
        }
    }
}

/// PUT /api/hosts/:id — update an existing host.
pub async fn update_host(
    State(state): State<HostState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateHost>,
) -> impl IntoResponse {
    match state.service.update(&id, input).await {
        Ok(host) => (StatusCode::OK, Json(host)).into_response(),
        Err(e) => {
            let status = match &e {
                ops_pilot_core::host::HostError::NotFound(_) => StatusCode::NOT_FOUND,
                ops_pilot_core::host::HostError::InvalidInput(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(serde_json::json!({"error": e.to_string()}))).into_response()
        }
    }
}

/// DELETE /api/hosts/:id — delete a host.
pub async fn delete_host(
    State(state): State<HostState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.service.delete(&id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            let status = match &e {
                ops_pilot_core::host::HostError::NotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(serde_json::json!({"error": e.to_string()}))).into_response()
        }
    }
}

/// Build the host routes sub-router.
pub fn host_routes(service: Arc<HostService>) -> axum::Router {
    use axum::routing::get;

    let state = HostState { service };

    axum::Router::new()
        .route("/api/hosts", get(list_hosts).post(create_host))
        .route(
            "/api/hosts/{id}",
            get(get_host).put(update_host).delete(delete_host),
        )
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, Method};
    use ops_pilot_core::db::Database;
    use tower::ServiceExt;

    async fn test_app() -> axum::Router {
        let db = Database::open_in_memory().await.unwrap();
        let service = Arc::new(HostService::new(db.pool));
        host_routes(service)
    }

    fn json_request(method: Method, uri: &str, body: serde_json::Value) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    #[tokio::test]
    async fn test_list_hosts_empty() {
        let app = test_app().await;
        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/hosts")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let hosts: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
        assert!(hosts.is_empty());
    }

    #[tokio::test]
    async fn test_create_and_get_host() {
        let app = test_app().await;

        let create_body = serde_json::json!({
            "name": "test-host",
            "address": "10.0.0.1",
            "port": 22,
            "username": "root",
            "auth_method": "key"
        });

        // POST /api/hosts
        let req = json_request(Method::POST, "/api/hosts", create_body);
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let host: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = host["id"].as_str().unwrap();

        // GET /api/hosts/:id
        let req = Request::builder()
            .method(Method::GET)
            .uri(format!("/api/hosts/{}", id))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(fetched["name"], "test-host");
        assert_eq!(fetched["address"], "10.0.0.1");
    }

    #[tokio::test]
    async fn test_get_host_not_found() {
        let app = test_app().await;
        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/hosts/nonexistent")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_create_host_invalid_input() {
        let app = test_app().await;
        let body = serde_json::json!({
            "name": "",
            "address": "10.0.0.1",
            "username": "root",
            "auth_method": "key"
        });
        let req = json_request(Method::POST, "/api/hosts", body);
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_update_host() {
        let app = test_app().await;

        // Create first
        let create_body = serde_json::json!({
            "name": "update-test",
            "address": "10.0.0.2",
            "username": "admin",
            "auth_method": "password"
        });
        let req = json_request(Method::POST, "/api/hosts", create_body);
        let resp = app.clone().oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let host: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = host["id"].as_str().unwrap();

        // Update
        let update_body = serde_json::json!({
            "name": "updated-name",
            "status": "online"
        });
        let req = json_request(
            Method::PUT,
            &format!("/api/hosts/{}", id),
            update_body,
        );
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(updated["name"], "updated-name");
        assert_eq!(updated["status"], "online");
    }

    #[tokio::test]
    async fn test_delete_host() {
        let app = test_app().await;

        // Create
        let create_body = serde_json::json!({
            "name": "delete-test",
            "address": "10.0.0.3",
            "username": "admin",
            "auth_method": "password"
        });
        let req = json_request(Method::POST, "/api/hosts", create_body);
        let resp = app.clone().oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let host: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = host["id"].as_str().unwrap();

        // Delete
        let req = Request::builder()
            .method(Method::DELETE)
            .uri(format!("/api/hosts/{}", id))
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        // Verify deleted
        let req = Request::builder()
            .method(Method::GET)
            .uri(format!("/api/hosts/{}", id))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_host_not_found() {
        let app = test_app().await;
        let req = Request::builder()
            .method(Method::DELETE)
            .uri("/api/hosts/nonexistent")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
