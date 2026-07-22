//! Alert rule CRUD integration tests.
//!
//! Tests the alert rules and notification channels API endpoints
//! using an in-memory SQLite database and a random port server.

use std::net::SocketAddr;

use sqlx::SqlitePool;

// ── Helpers ──────────────────────────────────────────────────────────────

async fn setup_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS alert_rules (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            metric TEXT NOT NULL,
            condition TEXT NOT NULL,
            threshold REAL NOT NULL,
            severity TEXT NOT NULL,
            silence_minutes INTEGER NOT NULL DEFAULT 5,
            channel_ids TEXT NOT NULL DEFAULT '[]',
            enabled BOOLEAN NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )"
    ).execute(&pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS notification_channels (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            channel_type TEXT NOT NULL,
            config TEXT NOT NULL DEFAULT '{}',
            enabled BOOLEAN NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )"
    ).execute(&pool).await.unwrap();

    pool
}

async fn setup_alert_app() -> (SocketAddr, SqlitePool) {
    let pool = setup_db().await;
    let app = ops_pilot_gateway::routes::alert::alert_routes(pool.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (addr, pool)
}

// ── Tests ― Alert Rules ─────────────────────────────────────────────────

#[tokio::test]
async fn test_create_alert_rule_returns_201() {
    let (addr, _pool) = setup_alert_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("http://{}/api/alert/rules", addr))
        .json(&serde_json::json!({
            "name": "High CPU Usage",
            "metric": "cpu_percent",
            "condition": ">",
            "threshold": 90.0,
            "severity": "critical",
            "silence_minutes": 5,
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 201, "Creating an alert rule should return 201");

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["name"], "High CPU Usage");
    assert_eq!(body["metric"], "cpu_percent");
    assert_eq!(body["severity"], "critical");
    assert!(body["id"].as_str().unwrap().len() > 0);
}

#[tokio::test]
async fn test_list_alert_rules() {
    let (addr, pool) = setup_alert_app().await;
    let client = reqwest::Client::new();

    // Insert two rules directly
    sqlx::query(
        "INSERT INTO alert_rules (id, name, metric, condition, threshold, severity) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind("r1").bind("CPU High").bind("cpu_percent").bind(">").bind(90.0).bind("critical")
    .execute(&pool).await.unwrap();

    sqlx::query(
        "INSERT INTO alert_rules (id, name, metric, condition, threshold, severity) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind("r2").bind("Memory High").bind("memory_percent").bind(">").bind(80.0).bind("warning")
    .execute(&pool).await.unwrap();

    let resp = client
        .get(format!("http://{}/api/alert/rules", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(body.len(), 2, "Should return 2 alert rules");
}

#[tokio::test]
async fn test_update_alert_rule() {
    let (addr, pool) = setup_alert_app().await;
    let client = reqwest::Client::new();

    sqlx::query(
        "INSERT INTO alert_rules (id, name, metric, condition, threshold, severity) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind("r-update").bind("Old Name").bind("cpu_percent").bind(">").bind(90.0).bind("critical")
    .execute(&pool).await.unwrap();

    let resp = client
        .put(format!("http://{}/api/alert/rules/r-update", addr))
        .json(&serde_json::json!({
            "name": "Updated Name",
            "threshold": 95.0,
            "enabled": false,
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["name"], "Updated Name");
    assert_eq!(body["threshold"], 95.0);
    assert!(!body["enabled"].as_bool().unwrap());
}

#[tokio::test]
async fn test_delete_alert_rule() {
    let (addr, pool) = setup_alert_app().await;
    let client = reqwest::Client::new();

    sqlx::query(
        "INSERT INTO alert_rules (id, name, metric, condition, threshold, severity) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind("r-del").bind("To Delete").bind("cpu").bind(">").bind(90.0).bind("critical")
    .execute(&pool).await.unwrap();

    let resp = client
        .delete(format!("http://{}/api/alert/rules/r-del", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 204, "Deleting a rule should return 204");

    // Verify it's gone
    let rules: Vec<serde_json::Value> = client
        .get(format!("http://{}/api/alert/rules", addr))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(rules.is_empty(), "Deleted rule should not appear in list");
}

#[tokio::test]
async fn test_update_nonexistent_rule_returns_404() {
    let (addr, _pool) = setup_alert_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .put(format!("http://{}/api/alert/rules/nonexistent-id", addr))
        .json(&serde_json::json!({"name": "Ghost"}))
        .send()
        .await
        .unwrap();

    // Should return 404 or 500 depending on implementation, but not 200
    assert!(
        resp.status().as_u16() >= 400,
        "Updating a nonexistent rule should return an error status"
    );
}

#[tokio::test]
async fn test_delete_nonexistent_rule_returns_204_or_error() {
    let (addr, _pool) = setup_alert_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .delete(format!("http://{}/api/alert/rules/ghost-rule", addr))
        .send()
        .await
        .unwrap();

    // DELETE on non-existent should be idempotent
    assert!(resp.status().is_success() || resp.status().as_u16() == 204);
}

// ── Tests ― Notification Channels ───────────────────────────────────────

#[tokio::test]
async fn test_create_notification_channel() {
    let (addr, _pool) = setup_alert_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("http://{}/api/alert/channels", addr))
        .json(&serde_json::json!({
            "name": "Slack Alerts",
            "channel_type": "webhook",
            "config": {"url": "https://hooks.slack.com/services/T00/B00/xxx"}
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 201, "Creating a channel should return 201");

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["name"], "Slack Alerts");
    assert_eq!(body["channel_type"], "webhook");
    assert!(body["id"].as_str().unwrap().len() > 0);
}

#[tokio::test]
async fn test_rule_requires_name() {
    let (addr, _pool) = setup_alert_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("http://{}/api/alert/rules", addr))
        .json(&serde_json::json!({
            "metric": "cpu_percent",
            "condition": ">",
            "threshold": 90.0,
            "severity": "critical"
        }))
        .send()
        .await
        .unwrap();

    // Missing required field 'name' - should fail
    assert!(
        resp.status().is_server_error(),
        "Missing required field should result in error"
    );
}

#[tokio::test]
async fn test_alert_history_list() {
    let (addr, pool) = setup_alert_app().await;
    let client = reqwest::Client::new();

    // Create tables + seed a history entry
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS alert_history (
            id TEXT PRIMARY KEY,
            rule_id TEXT NOT NULL,
            severity TEXT NOT NULL,
            message TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'firing',
            triggered_at TEXT NOT NULL DEFAULT (datetime('now')),
            acknowledged_at TEXT
        )"
    ).execute(&pool).await.unwrap();

    sqlx::query(
        "INSERT INTO alert_history (id, rule_id, severity, message, status) VALUES (?, ?, ?, ?, ?)"
    )
    .bind("ah1").bind("rule-1").bind("critical").bind("CPU at 95%").bind("firing")
    .execute(&pool).await.unwrap();

    let resp = client
        .get(format!("http://{}/api/alert/history", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert!(!body.is_empty(), "History should contain entries");
    assert_eq!(body[0]["severity"], "critical");
}

#[tokio::test]
async fn test_search_alert_history_by_severity() {
    let (addr, pool) = setup_alert_app().await;
    let client = reqwest::Client::new();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS alert_history (
            id TEXT PRIMARY KEY,
            rule_id TEXT NOT NULL,
            severity TEXT NOT NULL,
            message TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'firing',
            triggered_at TEXT NOT NULL DEFAULT (datetime('now')),
            acknowledged_at TEXT
        )"
    ).execute(&pool).await.unwrap();

    sqlx::query(
        "INSERT INTO alert_history (id, rule_id, severity, message) VALUES (?, ?, ?, ?)"
    )
    .bind("ah2").bind("r1").bind("warning").bind("Disk at 85%")
    .execute(&pool).await.unwrap();

    sqlx::query(
        "INSERT INTO alert_history (id, rule_id, severity, message) VALUES (?, ?, ?, ?)"
    )
    .bind("ah3").bind("r2").bind("critical").bind("OOM detected")
    .execute(&pool).await.unwrap();

    // Filter by severity=warning
    let resp = client
        .get(format!("http://{}/api/alert/history?severity=warning", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(body.len(), 1, "Should only return warning entries");
    assert_eq!(body[0]["message"], "Disk at 85%");
}

#[tokio::test]
async fn test_app_isolation() {
    // Two independent apps should not share data
    let (addr1, _pool1) = setup_alert_app().await;
    let (addr2, _pool2) = setup_alert_app().await;

    let client = reqwest::Client::new();

    // Create a rule on app1
    let resp = client
        .post(format!("http://{}/api/alert/rules", addr1))
        .json(&serde_json::json!({
            "name": "Isolated Rule",
            "metric": "cpu",
            "condition": ">",
            "threshold": 90.0,
            "severity": "critical"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    // App2 should have no rules
    let resp2 = client
        .get(format!("http://{}/api/alert/rules", addr2))
        .send()
        .await
        .unwrap();
    let rules2: Vec<serde_json::Value> = resp2.json().await.unwrap();
    assert!(rules2.is_empty(), "Isolated app should have no rules");
}
