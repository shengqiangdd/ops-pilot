//! Backup & restore API integration tests.
//!
//! Tests the export/import endpoints with in-memory SQLite databases
//! on independent random-port servers.

use std::net::SocketAddr;

use sqlx::SqlitePool;

// ── Helpers ──────────────────────────────────────────────────────────────

async fn setup_db_with_data() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )"
    ).execute(&pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS notification_channels (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            channel_type TEXT NOT NULL,
            config TEXT NOT NULL DEFAULT '{}',
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )"
    ).execute(&pool).await.unwrap();

    // Seed data
    sqlx::query("INSERT INTO config (key, value) VALUES ('site_name', 'My OpsPilot')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO config (key, value) VALUES ('theme', 'dark')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO notification_channels (id, name, channel_type, config) VALUES (?, ?, ?, ?)"
    )
    .bind("ch-webhook").bind("Slack").bind("webhook")
    .bind(r#"{"url": "https://hooks.slack.com/xxx"}"#)
    .execute(&pool).await.unwrap();

    pool
}

async fn setup_empty_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )"
    ).execute(&pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS notification_channels (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            channel_type TEXT NOT NULL,
            config TEXT NOT NULL DEFAULT '{}',
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )"
    ).execute(&pool).await.unwrap();

    pool
}

async fn setup_backup_app_with_data() -> (SocketAddr, SqlitePool) {
    let pool = setup_db_with_data().await;
    let app = ops_pilot_gateway::routes::backup::backup_routes(pool.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (addr, pool)
}

async fn setup_backup_app_empty() -> (SocketAddr, SqlitePool) {
    let pool = setup_empty_db().await;
    let app = ops_pilot_gateway::routes::backup::backup_routes(pool.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (addr, pool)
}

// ── Tests ― Export ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_export_backup_returns_ok() {
    let (addr, _pool) = setup_backup_app_with_data().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{}/api/backup/export", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok", "Export should report status ok");
}

#[tokio::test]
async fn test_export_contains_version_and_created_at() {
    let (addr, _pool) = setup_backup_app_with_data().await;
    let client = reqwest::Client::new();

    let body: serde_json::Value = client
        .get(format!("http://{}/api/backup/export", addr))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let data = &body["data"];
    assert!(data["version"].is_string(), "Backup should contain a version");
    assert!(data["created_at"].is_string(), "Backup should contain created_at timestamp");
    assert!(!data["created_at"].as_str().unwrap().is_empty(), "Timestamp should not be empty");
}

#[tokio::test]
async fn test_export_config_contains_seeded_data() {
    let (addr, _pool) = setup_backup_app_with_data().await;
    let client = reqwest::Client::new();

    let body: serde_json::Value = client
        .get(format!("http://{}/api/backup/export", addr))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let config_rows = body["data"]["tables"]["config"].as_array().unwrap();
    assert_eq!(config_rows.len(), 2, "Should have 2 config entries");

    let keys: Vec<&str> = config_rows
        .iter()
        .filter_map(|r| r["key"].as_str())
        .collect();
    assert!(keys.contains(&"site_name"), "Should contain site_name key");
    assert!(keys.contains(&"theme"), "Should contain theme key");
}

#[tokio::test]
async fn test_export_contains_notification_channels() {
    let (addr, _pool) = setup_backup_app_with_data().await;
    let client = reqwest::Client::new();

    let body: serde_json::Value = client
        .get(format!("http://{}/api/backup/export", addr))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let channels = body["data"]["tables"]["notification_channels"].as_array().unwrap();
    assert_eq!(channels.len(), 1, "Should have 1 notification channel");
    assert_eq!(channels[0]["name"], "Slack");
}

#[tokio::test]
async fn test_export_empty_database() {
    let (addr, _pool) = setup_backup_app_empty().await;
    let client = reqwest::Client::new();

    let body: serde_json::Value = client
        .get(format!("http://{}/api/backup/export", addr))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(body["status"], "ok");

    let config_rows = body["data"]["tables"]["config"].as_array().unwrap();
    assert!(config_rows.is_empty(), "Empty database should have no config rows");
}

// ── Tests ― Import ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_import_valid_backup() {
    let (addr, pool) = setup_backup_app_empty().await;
    let client = reqwest::Client::new();

    // Build a valid backup payload
    let backup = serde_json::json!({
        "version": "2.0",
        "created_at": "2026-01-01T00:00:00Z",
        "tables": {
            "config": [
                {"key": "imported_key", "value": "imported_value"}
            ]
        }
    });

    let resp = client
        .post(format!("http://{}/api/backup/import", addr))
        .json(&backup)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok", "Import should report status ok");

    // Verify data was imported
    let row: (String,) = sqlx::query_as("SELECT value FROM config WHERE key = 'imported_key'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.0, "imported_value");
}

#[tokio::test]
async fn test_import_invalid_format_returns_error() {
    let (addr, _pool) = setup_backup_app_empty().await;
    let client = reqwest::Client::new();

    // Send invalid JSON structure (not a SystemBackup)
    let resp = client
        .post(format!("http://{}/api/backup/import", addr))
        .json(&serde_json::json!({
            "not": "a backup",
            "missing": ["version", "tables"]
        }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "error", "Invalid format should report status error");
    assert!(
        body["message"].as_str().unwrap_or("").len() > 0,
        "Should include an error message"
    );
}

#[tokio::test]
async fn test_import_clear_and_replace() {
    let (addr, pool) = setup_backup_app_with_data().await;
    let client = reqwest::Client::new();

    // Export original data
    let export_body: serde_json::Value = client
        .get(format!("http://{}/api/backup/export", addr))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let original_config_len = export_body["data"]["tables"]["config"]
        .as_array()
        .unwrap()
        .len();
    assert_eq!(original_config_len, 2);

    // Import new data (should clear and replace)
    let backup = serde_json::json!({
        "version": "2.0",
        "created_at": "2026-06-01T00:00:00Z",
        "tables": {
            "config": [
                {"key": "new_key", "value": "new_value"}
            ],
            "notification_channels": []
        }
    });

    let resp = client
        .post(format!("http://{}/api/backup/import", addr))
        .json(&backup)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    // Verify old data is gone, new data present
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT key, value FROM config")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1, "Old config rows should be cleared");
    assert_eq!(rows[0].0, "new_key");
    assert_eq!(rows[0].1, "new_value");
}

#[tokio::test]
async fn test_import_round_trip() {
    let (addr1, _pool1) = setup_backup_app_with_data().await;
    let client = reqwest::Client::new();

    // Export from app1
    let export_body: serde_json::Value = client
        .get(format!("http://{}/api/backup/export", addr1))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let export_data = &export_body["data"];

    // Import into app2 (empty)
    let (addr2, pool2) = setup_backup_app_empty().await;

    let resp = client
        .post(format!("http://{}/api/backup/import", addr2))
        .json(&serde_json::json!({
            "version": export_data["version"],
            "created_at": export_data["created_at"],
            "tables": export_data["tables"],
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    // Verify round-trip: app2 should have the same data
    let config_rows: Vec<(String, String)> = sqlx::query_as("SELECT key, value FROM config")
        .fetch_all(&pool2)
        .await
        .unwrap();
    assert_eq!(config_rows.len(), 2, "Round-trip should restore 2 config entries");

    let channels: Vec<(String,)> = sqlx::query_as("SELECT name FROM notification_channels")
        .fetch_all(&pool2)
        .await
        .unwrap();
    assert_eq!(channels.len(), 1, "Round-trip should restore 1 channel");
    assert_eq!(channels[0].0, "Slack");
}

#[tokio::test]
async fn test_app_isolation() {
    let (addr1, _pool1) = setup_backup_app_with_data().await;
    let (addr2, _pool2) = setup_backup_app_empty().await;
    let client = reqwest::Client::new();

    // App1 has data
    let body1: serde_json::Value = client
        .get(format!("http://{}/api/backup/export", addr1))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(!body1["data"]["tables"]["config"].as_array().unwrap().is_empty());

    // App2 has no data
    let body2: serde_json::Value = client
        .get(format!("http://{}/api/backup/export", addr2))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(body2["data"]["tables"]["config"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_import_large_backup() {
    let (addr, pool) = setup_backup_app_empty().await;
    let client = reqwest::Client::new();

    // Build a backup with 100 config entries
    let mut config_entries = Vec::new();
    for i in 0..100 {
        config_entries.push(serde_json::json!({
            "key": format!("key_{}", i),
            "value": format!("value_{}", i),
        }));
    }

    let backup = serde_json::json!({
        "version": "2.0",
        "created_at": "2026-07-22T00:00:00Z",
        "tables": {
            "config": config_entries,
            "notification_channels": []
        }
    });

    let resp = client
        .post(format!("http://{}/api/backup/import", addr))
        .json(&backup)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    // Verify all 100 entries were imported
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM config")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 100, "All 100 config entries should be imported");
}
