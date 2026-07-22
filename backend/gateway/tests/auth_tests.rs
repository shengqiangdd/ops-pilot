//! Authentication integration tests.
//!
//! Tests the full login/logout/token-refresh/invalid-token lifecycle
//! by building a minimal Axum router with auth endpoints connected to
//! an in-memory SQLite database.

use std::sync::Arc;
use std::net::SocketAddr;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use ops_pilot_core::auth::AuthService;
use serde::Deserialize;
use sqlx::SqlitePool;

// ── Auth route handlers (mirroring main.rs) ─────────────────────────────

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Clone)]
struct AuthState {
    service: Arc<AuthService>,
}

async fn register_handler(
    State(state): State<AuthState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user = state
        .service
        .register(&req.username, &req.email, &req.password)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(Json(serde_json::json!({
        "id": user.id,
        "username": user.username,
        "email": user.email,
        "role": user.role,
    })))
}

async fn login_handler(
    State(state): State<AuthState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let token = state
        .service
        .login(&req.username, &req.password)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let claims = state
        .service
        .verify_token(&token)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "token": token,
        "role": claims.role,
    })))
}

async fn me_handler(
    State(state): State<AuthState>,
    req: axum::http::Request<axum::body::Body>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Extract Bearer token from Authorization header
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid Authorization format".to_string()))?;

    let claims = state
        .service
        .verify_token(token)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    Ok(Json(serde_json::json!({
        "sub": claims.sub,
        "role": claims.role,
        "exp": claims.exp,
    })))
}

// ── Test helpers ─────────────────────────────────────────────────────────

async fn auth_routes(service: Arc<AuthService>) -> Router {
    let state = AuthState { service };
    Router::new()
        .route("/api/auth/register", post(register_handler))
        .route("/api/auth/login", post(login_handler))
        .route("/api/auth/me", get(me_handler))
        .with_state(state)
}

async fn setup_auth_app() -> (SocketAddr, SqlitePool, Arc<AuthService>) {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
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

    let auth_service = Arc::new(AuthService::new(pool.clone(), "test-secret".into()));
    let app = auth_routes(auth_service.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (addr, pool, auth_service)
}

// ── Tests ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_register_and_login_flow() {
    let (addr, _pool, _svc) = setup_auth_app().await;
    let client = reqwest::Client::new();

    // Register
    let reg_resp = client
        .post(format!("http://{}/api/auth/register", addr))
        .json(&serde_json::json!({
            "username": "flowuser",
            "email": "flow@example.com",
            "password": "secure12345",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(reg_resp.status(), 200, "Registration should succeed");

    let reg_body: serde_json::Value = reg_resp.json().await.unwrap();
    assert_eq!(reg_body["username"], "flowuser");
    assert_eq!(reg_body["email"], "flow@example.com");

    // Login
    let login_resp = client
        .post(format!("http://{}/api/auth/login", addr))
        .json(&serde_json::json!({
            "username": "flowuser",
            "password": "secure12345",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(login_resp.status(), 200, "Login should succeed");

    let login_body: serde_json::Value = login_resp.json().await.unwrap();
    let token = login_body["token"].as_str().unwrap();
    assert!(
        token.len() > 20,
        "Token should be a reasonable JWT length"
    );
    assert_eq!(login_body["role"], "operator");
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let (addr, _pool, _svc) = setup_auth_app().await;
    let client = reqwest::Client::new();

    // Wrong password
    let resp = client
        .post(format!("http://{}/api/auth/login", addr))
        .json(&serde_json::json!({
            "username": "flowuser",
            "password": "wrongpass",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401, "Wrong password should return 401");

    // Non-existent user
    let resp = client
        .post(format!("http://{}/api/auth/login", addr))
        .json(&serde_json::json!({
            "username": "nobody",
            "password": "anything",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401, "Non-existent user should return 401");
}

#[tokio::test]
async fn test_protected_endpoint_with_valid_token() {
    let (addr, _pool, svc) = setup_auth_app().await;
    let client = reqwest::Client::new();

    // Create user and get token
    svc.register("prot_user", "prot@example.com", "password123")
        .await
        .unwrap();
    let token = svc.login("prot_user", "password123").await.unwrap();

    // Access /me with valid token
    let resp = client
        .get(format!("http://{}/api/auth/me", addr))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "Valid token should grant access");

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.get("sub").is_some(), "Response should contain 'sub'");
    assert_eq!(body["role"], "operator");
}

#[tokio::test]
async fn test_protected_endpoint_rejects_no_token() {
    let (addr, _pool, _svc) = setup_auth_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{}/api/auth/me", addr))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401, "No token should return 401");
}

#[tokio::test]
async fn test_protected_endpoint_rejects_invalid_token() {
    let (addr, _pool, _svc) = setup_auth_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{}/api/auth/me", addr))
        .header("Authorization", "Bearer this.is.not.a.valid.jwt")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401, "Invalid token should return 401");
}

#[tokio::test]
async fn test_register_duplicate_username() {
    let (addr, _pool, svc) = setup_auth_app().await;
    let client = reqwest::Client::new();

    // Register first user via service
    svc.register("dupuser", "dup@example.com", "password123")
        .await
        .unwrap();

    // Try registering same username
    let resp = client
        .post(format!("http://{}/api/auth/register", addr))
        .json(&serde_json::json!({
            "username": "dupuser",
            "email": "other@example.com",
            "password": "password123",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400, "Duplicate username should return 400");
}

#[tokio::test]
async fn test_token_refresh_concept() {
    // Token "refresh" is done by re-login (issuing a new token)
    let (_addr, _pool, svc) = setup_auth_app().await;

    svc.register("refresh_user", "refresh@example.com", "password123")
        .await
        .unwrap();

    let token1 = svc.login("refresh_user", "password123").await.unwrap();
    let token2 = svc.login("refresh_user", "password123").await.unwrap();

    // Both tokens should be valid (different JWTs due to different iat)
    assert!(svc.verify_token(&token1).is_ok());
    assert!(svc.verify_token(&token2).is_ok());

    // Tokens should be different strings (different iat)
    assert_ne!(
        token1, token2,
        "Each login should produce a unique token"
    );
}

#[tokio::test]
async fn test_wrong_secret_rejects_token() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
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

    let signer = Arc::new(AuthService::new(pool.clone(), "signer-secret".into()));
    let verifier = Arc::new(AuthService::new(pool.clone(), "verifier-secret".into()));

    signer
        .register("cross_user", "cross@example.com", "password123")
        .await
        .unwrap();

    let token = signer.login("cross_user", "password123").await.unwrap();

    // Verifying with a different secret should fail
    assert!(
        verifier.verify_token(&token).is_err(),
        "Token signed with one secret should fail verification with another"
    );
}
