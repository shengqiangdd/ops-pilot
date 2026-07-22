//! Health-check endpoint integration tests.
//!
//! Validates that `/api/v1/health` returns the expected status, version,
//! and security headers.

use std::sync::Arc;
use std::net::SocketAddr;

use ops_pilot_gateway::AppState;
use sqlx::SqlitePool;

// ── Helper ───────────────────────────────────────────────────────────────

async fn setup_health_app() -> SocketAddr {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let app_state = Arc::new(AppState {
        pool: pool.clone(),
        alert_suppressor: ops_pilot_gateway::alert_suppression::AlertSuppressor::new(30, 5),
    });
    let app = ops_pilot_gateway::create_router(app_state).await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    addr
}

// ── Tests ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_health_returns_200() {
    let addr = setup_health_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{}/api/v1/health", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_health_returns_json() {
    let addr = setup_health_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{}/api/v1/health", addr))
        .send()
        .await
        .unwrap();

    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.contains("json"),
        "Expected JSON content-type, got: {content_type}"
    );
}

#[tokio::test]
async fn test_health_body_contains_status_and_version() {
    let addr = setup_health_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{}/api/v1/health", addr))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok", "Health status should be 'ok'");
    assert!(
        body.get("version").and_then(|v| v.as_str()).is_some(),
        "Health response should include version field"
    );
}

#[tokio::test]
async fn test_health_security_headers_present() {
    let addr = setup_health_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{}/api/v1/health", addr))
        .send()
        .await
        .unwrap();

    let headers = resp.headers();

    // Core security headers
    assert!(
        headers.contains_key("x-content-type-options"),
        "Missing X-Content-Type-Options"
    );
    assert!(
        headers.contains_key("x-frame-options"),
        "Missing X-Frame-Options"
    );
    assert!(
        headers.contains_key("content-security-policy"),
        "Missing Content-Security-Policy"
    );
    assert!(
        headers.contains_key("strict-transport-security"),
        "Missing Strict-Transport-Security"
    );
    assert!(
        headers.contains_key("referrer-policy"),
        "Missing Referrer-Policy"
    );

    // Pre-existing optional ones
    assert!(
        headers.contains_key("permissions-policy"),
        "Missing Permissions-Policy"
    );
}

#[tokio::test]
async fn test_health_security_header_values_are_correct() {
    let addr = setup_health_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{}/api/v1/health", addr))
        .send()
        .await
        .unwrap();

    let headers = resp.headers();

    // Check specific header values
    let csp = headers
        .get("content-security-policy")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        csp.contains("default-src 'self'"),
        "CSP should include default-src 'self', got: {csp}"
    );
    assert!(
        csp.contains("frame-ancestors 'none'"),
        "CSP should include frame-ancestors 'none'"
    );

    let hsts = headers
        .get("strict-transport-security")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        hsts.contains("max-age=31536000"),
        "HSTS should contain max-age=31536000, got: {hsts}"
    );

    let referrer = headers
        .get("referrer-policy")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(referrer, "same-origin");

    let x_frame = headers
        .get("x-frame-options")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(x_frame, "DENY");

    let x_content = headers
        .get("x-content-type-options")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(x_content, "nosniff");
}

#[tokio::test]
async fn test_health_is_get_only() {
    let addr = setup_health_app().await;
    let client = reqwest::Client::new();

    // POST should be 405 Method Not Allowed
    let resp = client
        .post(format!("http://{}/api/v1/health", addr))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        405,
        "POST to health should return 405"
    );
}
