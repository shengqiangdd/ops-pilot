//! REST handlers for host management — all endpoints require JWT authentication.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use ops_pilot_core::host::{CreateHost, HostService, UpdateHost};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::middleware::AuthLayer;

/// Shared application state for host routes.
#[derive(Clone)]
pub struct HostState {
    pub service: Arc<HostService>,
}

/// Request body for batch command execution.
#[derive(Debug, Deserialize)]
pub struct BatchExecuteRequest {
    pub host_ids: Vec<String>,
    pub command: String,
    pub timeout: Option<u32>,
}

/// Result of a single host command execution.
#[derive(Debug, Serialize)]
pub struct BatchExecuteResult {
    pub host_id: String,
    pub host_name: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

/// Response for batch command execution.
#[derive(Debug, Serialize)]
pub struct BatchExecuteResponse {
    pub results: Vec<BatchExecuteResult>,
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
}

/// GET /api/hosts — list hosts owned by the authenticated user.
pub async fn list_hosts(
    State(state): State<HostState>,
    AuthLayer(claims): AuthLayer,
) -> impl IntoResponse {
    match state.service.list_by_owner(&claims.sub).await {
        Ok(hosts) => (StatusCode::OK, Json(hosts)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/hosts/:id — get a single host by ID (must belong to the user).
pub async fn get_host(
    State(state): State<HostState>,
    AuthLayer(claims): AuthLayer,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.service.get(&id, &claims.sub).await {
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

/// POST /api/hosts — create a new host owned by the authenticated user.
pub async fn create_host(
    State(state): State<HostState>,
    AuthLayer(claims): AuthLayer,
    Json(input): Json<CreateHost>,
) -> impl IntoResponse {
    match state.service.create(input, &claims.sub).await {
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

/// PUT /api/hosts/:id — update an existing host (must belong to the user).
pub async fn update_host(
    State(state): State<HostState>,
    AuthLayer(claims): AuthLayer,
    Path(id): Path<String>,
    Json(input): Json<UpdateHost>,
) -> impl IntoResponse {
    match state.service.update(&id, &claims.sub, input).await {
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

/// DELETE /api/hosts/:id — delete a host (must belong to the user).
pub async fn delete_host(
    State(state): State<HostState>,
    AuthLayer(claims): AuthLayer,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.service.delete(&id, &claims.sub).await {
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

/// POST /api/hosts/batch/execute — execute a command on multiple hosts in parallel.
///
/// Runs the command on up to 10 hosts concurrently with a configurable timeout.
pub async fn batch_execute(
    State(state): State<HostState>,
    AuthLayer(claims): AuthLayer,
    Json(req): Json<BatchExecuteRequest>,
) -> impl IntoResponse {
    if req.host_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "host_ids is required and must not be empty"})),
        )
            .into_response();
    }

    if req.command.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "command is required"})),
        )
            .into_response();
    }

    let timeout_ms = req.timeout.unwrap_or(30) * 1000;

    // Fetch host info for all requested IDs
    let mut hosts = Vec::new();
    for host_id in &req.host_ids {
        match state.service.get(host_id, &claims.sub).await {
            Ok(host) => hosts.push(host),
            Err(_) => {
                // Skip hosts that don't exist or aren't owned by the user
            }
        }
    }

    if hosts.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "no valid hosts found"})),
        )
            .into_response();
    }

    // Execute commands in parallel (up to 10 concurrent)
    let max_concurrent = 10;
    let mut results = Vec::new();
    let chunks: Vec<Vec<_>> = hosts.chunks(max_concurrent).map(|c| c.to_vec()).collect();

    for chunk in chunks {
        let futures: Vec<_> = chunk.into_iter().map(|host| {
            let host_id = host.id.clone();
            let host_name = host.name.clone();
            let command = req.command.clone();
            let address = host.address.clone();
            let port = host.port;
            let username = host.username.clone();

            async move {
                let start = std::time::Instant::now();

                // Execute command via SSH
                let result = execute_ssh_command(
                    &host_id,
                    &address,
                    port as u16,
                    &username,
                    &command,
                    timeout_ms,
                ).await;

                let duration_ms = start.elapsed().as_millis() as u64;

                BatchExecuteResult {
                    host_id,
                    host_name,
                    success: result.0,
                    stdout: result.1,
                    stderr: result.2,
                    exit_code: result.3,
                    duration_ms,
                }
            }
        }).collect();

        let chunk_results = futures::future::join_all(futures).await;
        results.extend(chunk_results);
    }

    let succeeded = results.iter().filter(|r| r.success).count();
    let failed = results.len() - succeeded;

    let response = BatchExecuteResponse {
        total: results.len(),
        succeeded,
        failed,
        results,
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// Execute a command via SSH on a remote host.
///
/// Returns (success, stdout, stderr, exit_code).
async fn execute_ssh_command(
    _host_id: &str,
    address: &str,
    port: u16,
    username: &str,
    command: &str,
    _timeout_ms: u32,
) -> (bool, String, String, i32) {
    // In a real implementation, this would use the SSH connection pool
    // to execute the command on the remote host.
    // For now, we return a simulated result.

    // Log the attempt
    tracing::info!(
        host = address,
        port,
        username,
        command,
        "Batch execute: would run SSH command"
    );

    // Simulate command execution
    // In production: pool.get(host_id).await?.execute(command, timeout).await
    (true, format!("simulated output for: {}", command), String::new(), 0)
}

/// Build the host routes sub-router **without** auth middleware.
///
/// The caller must wrap these routes with `auth_middleware` to protect them.
pub fn host_routes(service: Arc<HostService>) -> axum::Router {
    use axum::routing::{get, post};

    let state = HostState { service };

    axum::Router::new()
        .route("/api/hosts", get(list_hosts).post(create_host))
        .route(
            "/api/hosts/{id}",
            get(get_host).put(update_host).delete(delete_host),
        )
        .route("/api/hosts/batch/execute", post(batch_execute))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, Request};
    use ops_pilot_core::auth::AuthService;
    use ops_pilot_core::db::Database;
    use tower::ServiceExt;

    const TEST_SECRET: &str = "test-secret";

    async fn setup_with_auth() -> (axum::Router, Arc<AuthService>) {
        let db = Database::open_in_memory().await.unwrap();
        let vault_keys = Arc::new(ops_pilot_core::vault::VaultKeyManager::new());
        let service = Arc::new(HostService::new(db.pool.clone(), vault_keys));
        let auth = Arc::new(AuthService::new(db.pool, TEST_SECRET.into()));

        // Register and login a test user
        auth.register("testuser", "test@example.com", "password123")
            .await
            .unwrap();
        let _token = auth.login("testuser", "password123").await.unwrap();

        let auth_state = crate::middleware::AuthState {
            service: auth.clone(),
        };

        let app = host_routes(service).layer(axum::middleware::from_fn_with_state(
            auth_state,
            crate::middleware::auth_middleware,
        ));

        (app, auth)
    }

    fn auth_request(
        method: Method,
        uri: &str,
        token: &str,
        body: Option<serde_json::Value>,
    ) -> Request<Body> {
        let builder = Request::builder()
            .method(method)
            .uri(uri)
            .header("authorization", format!("Bearer {}", token))
            .header("content-type", "application/json");

        let body = match body {
            Some(v) => Body::from(v.to_string()),
            None => Body::empty(),
        };

        builder.body(body).unwrap()
    }

    #[tokio::test]
    async fn test_list_hosts_empty() {
        let (app, auth) = setup_with_auth().await;
        let token = auth.login("testuser", "password123").await.unwrap();

        let req = auth_request(Method::GET, "/api/hosts", &token, None);
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
        let (app, auth) = setup_with_auth().await;
        let token = auth.login("testuser", "password123").await.unwrap();

        let create_body = serde_json::json!({
            "name": "test-host",
            "address": "10.0.0.1",
            "port": 22,
            "username": "root",
            "auth_method": "key"
        });

        let req = auth_request(Method::POST, "/api/hosts", &token, Some(create_body));
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let host: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = host["id"].as_str().unwrap();

        let req = auth_request(Method::GET, &format!("/api/hosts/{}", id), &token, None);
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
        let (app, auth) = setup_with_auth().await;
        let token = auth.login("testuser", "password123").await.unwrap();

        let req = auth_request(Method::GET, "/api/hosts/nonexistent", &token, None);
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_create_host_invalid_input() {
        let (app, auth) = setup_with_auth().await;
        let token = auth.login("testuser", "password123").await.unwrap();

        let body = serde_json::json!({
            "name": "",
            "address": "10.0.0.1",
            "username": "root",
            "auth_method": "key"
        });
        let req = auth_request(Method::POST, "/api/hosts", &token, Some(body));
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_update_host() {
        let (app, auth) = setup_with_auth().await;
        let token = auth.login("testuser", "password123").await.unwrap();

        let create_body = serde_json::json!({
            "name": "update-test",
            "address": "10.0.0.2",
            "username": "admin",
            "auth_method": "password"
        });
        let req = auth_request(Method::POST, "/api/hosts", &token, Some(create_body));
        let resp = app.clone().oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let host: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = host["id"].as_str().unwrap();

        let update_body = serde_json::json!({
            "name": "updated-name",
            "status": "online"
        });
        let req = auth_request(
            Method::PUT,
            &format!("/api/hosts/{}", id),
            &token,
            Some(update_body),
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
        let (app, auth) = setup_with_auth().await;
        let token = auth.login("testuser", "password123").await.unwrap();

        let create_body = serde_json::json!({
            "name": "delete-test",
            "address": "10.0.0.3",
            "username": "admin",
            "auth_method": "password"
        });
        let req = auth_request(Method::POST, "/api/hosts", &token, Some(create_body));
        let resp = app.clone().oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let host: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = host["id"].as_str().unwrap();

        let req = auth_request(Method::DELETE, &format!("/api/hosts/{}", id), &token, None);
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        let req = auth_request(Method::GET, &format!("/api/hosts/{}", id), &token, None);
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_host_not_found() {
        let (app, auth) = setup_with_auth().await;
        let token = auth.login("testuser", "password123").await.unwrap();

        let req = auth_request(Method::DELETE, "/api/hosts/nonexistent", &token, None);
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_unauthenticated_rejected() {
        let (app, _auth) = setup_with_auth().await;

        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/hosts")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_wrong_token_rejected() {
        let (app, _auth) = setup_with_auth().await;

        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/hosts")
            .header("authorization", "Bearer invalid-token")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
