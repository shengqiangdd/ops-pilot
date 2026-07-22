//! Authentication routes — register, login, and token verification.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use ops_pilot_core::auth::AuthService;
use serde::Deserialize;
use std::sync::Arc;

/// Shared auth state for routes.
#[derive(Clone)]
pub struct AuthState {
    pub service: Arc<AuthService>,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub role: String,
}

#[derive(serde::Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
}

/// POST /api/auth/register — create a new user account.
pub async fn register_handler(
    State(state): State<AuthState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    let user = state
        .service
        .register(&req.username, &req.email, &req.password)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let role = "operator".to_string();
    Ok(Json(UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        role,
    }))
}

/// POST /api/auth/login — authenticate and receive a JWT token.
pub async fn login_handler(
    State(state): State<AuthState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let token = state
        .service
        .login(&req.username, &req.password)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let claims = state
        .service
        .verify_token(&token)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    Ok(Json(AuthResponse {
        token,
        role: claims.role,
    }))
}

/// Build the auth routes sub-router.
pub fn auth_routes(service: Arc<AuthService>) -> Router {
    use axum::routing::post;
    let state = AuthState { service };
    Router::new()
        .route("/api/auth/register", post(register_handler))
        .route("/api/auth/login", post(login_handler))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use ops_pilot_core::auth::AuthService;
    use tower::ServiceExt;

    const TEST_SECRET: &str = "test-secret-for-auth-tests";

    /// Setup — returns (Router, shared_pool) so both direct-service and HTTP calls
    /// access the same in-memory database via the same single-connection pool.
    async fn setup() -> (Router, sqlx::SqlitePool) {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY NOT NULL,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                vault_key_encrypted TEXT,
                vault_password_hash TEXT,
                role TEXT NOT NULL DEFAULT 'operator',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        let svc = Arc::new(AuthService::new(pool.clone(), TEST_SECRET.into()));
        let app = auth_routes(svc);
        (app, pool)
    }

    #[tokio::test]
    async fn test_register_user() {
        let (app, _pool) = setup().await;

        let req_body = serde_json::json!({
            "username": "testuser",
            "email": "test@example.com",
            "password": "secure-password-123"
        });

        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/register")
            .header("content-type", "application/json")
            .body(Body::from(req_body.to_string()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();
        assert_eq!(body["username"], "testuser");
        assert_eq!(body["email"], "test@example.com");
        assert!(body["id"].is_string());
    }

    #[tokio::test]
    async fn test_register_duplicate_user() {
        let (app, _pool) = setup().await;

        let req_body = serde_json::json!({
            "username": "dupe",
            "email": "dupe@example.com",
            "password": "password123"
        });

        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/register")
            .header("content-type", "application/json")
            .body(Body::from(req_body.to_string()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Register same username again
        let req_body = serde_json::json!({
            "username": "dupe",
            "email": "other@example.com",
            "password": "password123"
        });
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/register")
            .header("content-type", "application/json")
            .body(Body::from(req_body.to_string()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_login_success() {
        let (app, pool) = setup().await;
        let svc = Arc::new(AuthService::new(pool, TEST_SECRET.into()));

        // Register via direct service call (same DB connection)
        svc.register("logintest", "login@example.com", "mypassword")
            .await
            .unwrap();

        // Login via HTTP
        let req_body = serde_json::json!({
            "username": "logintest",
            "password": "mypassword"
        });
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(req_body.to_string()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();
        assert!(body["token"].is_string());
        assert!(body["role"].is_string());
        assert!(!body["token"].as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_login_invalid_credentials() {
        let (app, _pool) = setup().await;

        let req_body = serde_json::json!({
            "username": "nonexistent",
            "password": "wrongpassword"
        });
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(req_body.to_string()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_login_wrong_password() {
        let (app, pool) = setup().await;
        let svc = Arc::new(AuthService::new(pool, TEST_SECRET.into()));

        // Register via direct service call
        svc.register("wpuser", "wp@example.com", "correctpassword")
            .await
            .unwrap();

        // Login with wrong password via HTTP
        let req_body = serde_json::json!({
            "username": "wpuser",
            "password": "wrongpassword"
        });
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(req_body.to_string()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_verify_token() {
        let (app, pool) = setup().await;
        let svc = Arc::new(AuthService::new(pool, TEST_SECRET.into()));

        // Register via direct service call
        svc.register("vtoken", "vt@example.com", "password123")
            .await
            .unwrap();

        // Login via HTTP
        let req_body = serde_json::json!({
            "username": "vtoken",
            "password": "password123"
        });
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(req_body.to_string()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();
        let token = body["token"].as_str().unwrap();

        // Verify token via service — sub is the user's UUID, not the username
        let claims = svc.verify_token(token).unwrap();
        assert!(uuid::Uuid::parse_str(&claims.sub).is_ok(), "sub should be a valid UUID");
        assert_eq!(claims.role, "operator");
    }

    #[tokio::test]
    async fn test_register_empty_password_rejected() {
        let (app, _pool) = setup().await;

        let req_body = serde_json::json!({
            "username": "nopass",
            "email": "nopass@example.com",
            "password": ""
        });
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/register")
            .header("content-type", "application/json")
            .body(Body::from(req_body.to_string()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert!(resp.status().is_client_error());
    }
}
