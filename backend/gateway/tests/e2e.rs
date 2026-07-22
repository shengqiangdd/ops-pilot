//! E2E tests for OpsPilot — full HTTP integration test suite.
//!
//! These tests start the gateway server in-process and exercise
//! real HTTP endpoints against an in-memory SQLite database.

use std::sync::Arc;
use std::net::SocketAddr;

use ops_pilot_gateway::AppState;
use sqlx::SqlitePool;

async fn setup_app() -> SocketAddr {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    // Run migrations
    sqlx::query("CREATE TABLE IF NOT EXISTS users (id TEXT PRIMARY KEY, username TEXT NOT NULL, password_hash TEXT NOT NULL, role TEXT NOT NULL DEFAULT 'user', created_at TEXT NOT NULL)")
        .execute(&pool).await.unwrap();
    
    let app_state = Arc::new(AppState {
        pool: pool.clone(),
        alert_suppressor: ops_pilot_gateway::alert_suppression::AlertSuppressor::new(30, 5),
    });
    let app = ops_pilot_gateway::create_router(app_state).await;
    
    // Bind to random port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    
    addr
}

#[tokio::test]
async fn test_health_endpoint() {
    let addr = setup_app().await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{}/api/v1/health", addr))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_404_returns_json() {
    let addr = setup_app().await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{}/api/v1/nonexistent", addr))
        .send()
        .await
        .unwrap();
    // Should be 404 with JSON body, not HTML
    assert_eq!(resp.status(), 404);
    let content_type = resp.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(content_type.contains("json"));
}

#[tokio::test]
async fn test_cors_headers() {
    let addr = setup_app().await;
    let client = reqwest::Client::new();
    let resp = client
        .request(reqwest::Method::OPTIONS, format!("http://{}/api/v1/health", addr))
        .header("Origin", "http://example.com")
        .header("Access-Control-Request-Method", "GET")
        .send()
        .await
        .unwrap();
    let headers = resp.headers();
    assert!(headers.contains_key("access-control-allow-origin"));
}

#[tokio::test]
async fn test_security_headers_present() {
    let addr = setup_app().await;
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let resp = client
        .get(format!("http://{}/api/v1/health", addr))
        .send()
        .await
        .unwrap();
    
    let headers = resp.headers();
    // HSTS
    assert!(headers.contains_key("strict-transport-security"));
    // CSP
    assert!(headers.contains_key("content-security-policy"));
    // X-Frame-Options
    assert!(headers.contains_key("x-frame-options"));
    // X-Content-Type-Options
    assert!(headers.contains_key("x-content-type-options"));
    // X-XSS-Protection (newly added)
    assert!(headers.contains_key("x-xss-protection"));
}

#[tokio::test]
async fn test_security_header_values() {
    let addr = setup_app().await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://{}/api/v1/health", addr))
        .send()
        .await
        .unwrap();
    
    let headers = resp.headers();
    
    assert_eq!(
        headers.get("x-content-type-options").and_then(|v| v.to_str().ok()),
        Some("nosniff")
    );
    assert_eq!(
        headers.get("x-frame-options").and_then(|v| v.to_str().ok()),
        Some("DENY")
    );
    assert_eq!(
        headers.get("x-xss-protection").and_then(|v| v.to_str().ok()),
        Some("1; mode=block")
    );
    assert_eq!(
        headers.get("referrer-policy").and_then(|v| v.to_str().ok()),
        Some("same-origin")
    );
}
