//! Search endpoint integration tests.
//!
//! Tests the `/api/search` endpoint by building a minimal Axum router
//! with the real search handler against an in-memory SQLite database.

use std::net::SocketAddr;
use sqlx::SqlitePool;

// ── Helpers ──────────────────────────────────────────────────────────────

/// Seed the in-memory database with sample data for all searchable entities.
async fn seed_search_data(pool: &SqlitePool) {
    // Create all tables the search route queries
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS hosts (
            id TEXT PRIMARY KEY, name TEXT, ip TEXT, port INTEGER,
            username TEXT, auth_method TEXT, status TEXT, created_at TEXT
        )"
    ).execute(pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS alert_history (
            id TEXT PRIMARY KEY, title TEXT, severity TEXT, message TEXT,
            status TEXT, triggered_at TEXT, rule_name TEXT
        )"
    ).execute(pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS knowledge_entries (
            id TEXT PRIMARY KEY, title TEXT, root_cause TEXT, solution TEXT, created_at TEXT
        )"
    ).execute(pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS runbook (
            id TEXT PRIMARY KEY, name TEXT, description TEXT, steps TEXT, created_at TEXT
        )"
    ).execute(pool).await.unwrap();

    // Seed hosts
    let hosts = vec![
        ("h1", "web-server-01", "192.168.1.10", "online", "key"),
        ("h2", "db-server-01", "192.168.1.20", "online", "key"),
        ("h3", "cache-node-01", "192.168.1.30", "offline", "password"),
        ("h4", "monitoring-host", "10.0.0.50", "online", "key"),
    ];
    for (id, name, ip, status, auth) in &hosts {
        sqlx::query(
            "INSERT INTO hosts (id, name, ip, port, username, auth_method, status) VALUES (?, ?, ?, 22, 'root', ?, ?)"
        )
        .bind(id).bind(name).bind(ip).bind(auth).bind(status)
        .execute(pool).await.unwrap();
    }

    // Seed alerts
    let alerts = vec![
        ("a1", "High CPU on web-server-01", "critical", "CPU at 95%", "firing"),
        ("a2", "Disk space low on db-server-01", "warning", "Disk at 85%", "firing"),
        ("a3", "Memory leak detected", "critical", "Memory usage at 90%", "acknowledged"),
    ];
    for (id, title, severity, message, status) in &alerts {
        sqlx::query(
            "INSERT INTO alert_history (id, title, severity, message, status, triggered_at, rule_name) VALUES (?, ?, ?, ?, ?, datetime('now'), ?)"
        )
        .bind(id).bind(title).bind(severity).bind(message).bind(status).bind(title)
        .execute(pool).await.unwrap();
    }

    // Seed knowledge
    let knowledge = vec![
        ("k1", "Postgres connection pool exhaustion", "Too many concurrent connections"),
        ("k2", "Nginx 502 errors", "Backend service unavailable"),
        ("k3", "Redis OOM", "maxmemory limit reached"),
    ];
    for (id, title, cause) in &knowledge {
        sqlx::query(
            "INSERT INTO knowledge_entries (id, title, root_cause, solution) VALUES (?, ?, ?, 'Restart service')"
        )
        .bind(id).bind(title).bind(cause)
        .execute(pool).await.unwrap();
    }

    // Seed runbooks
    let runbooks = vec![
        ("r1", "Restart Web Server", "Steps to restart gracefully"),
        ("r2", "Database Failover", "Manual failover steps"),
    ];
    for (id, name, desc) in &runbooks {
        sqlx::query(
            "INSERT INTO runbook (id, name, description) VALUES (?, ?, ?)"
        )
        .bind(id).bind(name).bind(desc)
        .execute(pool).await.unwrap();
    }
}

/// Create an empty database (tables exist, no data).
async fn setup_empty_tables(pool: &SqlitePool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS hosts (
            id TEXT PRIMARY KEY, name TEXT, ip TEXT, port INTEGER,
            username TEXT, auth_method TEXT, status TEXT, created_at TEXT
        )"
    ).execute(pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS alert_history (
            id TEXT PRIMARY KEY, title TEXT, severity TEXT, message TEXT,
            status TEXT, triggered_at TEXT, rule_name TEXT
        )"
    ).execute(pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS knowledge_entries (
            id TEXT PRIMARY KEY, title TEXT, root_cause TEXT, solution TEXT, created_at TEXT
        )"
    ).execute(pool).await.unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS runbook (
            id TEXT PRIMARY KEY, name TEXT, description TEXT, steps TEXT, created_at TEXT
        )"
    ).execute(pool).await.unwrap();
}

async fn setup_search_app() -> (SocketAddr, SqlitePool) {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    seed_search_data(&pool).await;

    let app = ops_pilot_gateway::routes::search::search_routes(pool.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (addr, pool)
}

async fn setup_empty_search_app() -> (SocketAddr, SqlitePool) {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    setup_empty_tables(&pool).await;

    let app = ops_pilot_gateway::routes::search::search_routes(pool.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (addr, pool)
}

// ── Tests ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_search_returns_results() {
    let (addr, _pool) = setup_search_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{}/api/search?q=web", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200, "Search should return 200");

    let body: serde_json::Value = resp.json().await.unwrap();
    let results = body["results"].as_array().unwrap();
    assert!(!results.is_empty(), "Search 'web' should find at least one result");

    // Should contain host results
    let host_results: Vec<&serde_json::Value> = results
        .iter()
        .filter(|r| r["type"] == "host")
        .collect();
    assert!(!host_results.is_empty(), "Should find host results for 'web'");
}

#[tokio::test]
async fn test_search_finds_alerts() {
    let (addr, _pool) = setup_search_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{}/api/search?q=CPU", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let results = body["results"].as_array().unwrap();
    assert!(!results.is_empty(), "Search 'CPU' should find alerts");

    let alert_results: Vec<&serde_json::Value> = results
        .iter()
        .filter(|r| r["type"] == "alert")
        .collect();
    assert!(!alert_results.is_empty(), "Should find alert results for 'CPU'");
}

#[tokio::test]
async fn test_search_finds_knowledge() {
    let (addr, _pool) = setup_search_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{}/api/search?q=Postgres", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let results = body["results"].as_array().unwrap();
    assert!(!results.is_empty(), "Search 'Postgres' should find knowledge entries");

    let knowledge_results: Vec<&serde_json::Value> = results
        .iter()
        .filter(|r| r["type"] == "knowledge")
        .collect();
    assert!(!knowledge_results.is_empty());
}

#[tokio::test]
async fn test_search_finds_runbooks() {
    let (addr, _pool) = setup_search_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{}/api/search?q=Database", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let results = body["results"].as_array().unwrap();
    assert!(!results.is_empty(), "Search 'Database' should find runbooks");
}

#[tokio::test]
async fn test_search_empty_query_returns_no_results() {
    let (addr, _pool) = setup_search_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{}/api/search?q=", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let results = body["results"].as_array().unwrap();
    assert!(results.is_empty(), "Empty query should return no results");
}

#[tokio::test]
async fn test_search_no_match_returns_empty() {
    let (addr, _pool) = setup_search_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{}/api/search?q=xyznonexistent12345", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let results = body["results"].as_array().unwrap();
    assert!(results.is_empty(), "Non-matching query should return no results");
}

#[tokio::test]
async fn test_search_no_data_returns_empty() {
    let (addr, _pool) = setup_empty_search_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{}/api/search?q=anything", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let results = body["results"].as_array().unwrap();
    assert!(results.is_empty(), "Empty database should return empty results");
}

#[tokio::test]
async fn test_search_pagination_implicit() {
    let (addr, _pool) = setup_search_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{}/api/search?q=01", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let results = body["results"].as_array().unwrap();

    // Each category uses LIMIT 5, 4 categories => at most 20 results
    assert!(
        results.len() <= 20,
        "Results should be bounded, got {}",
        results.len()
    );
}

#[tokio::test]
async fn test_search_result_has_expected_fields() {
    let (addr, _pool) = setup_search_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("http://{}/api/search?q=web-server-01", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let results = body["results"].as_array().unwrap();
    assert!(!results.is_empty(), "Should find the web server host");

    let result = &results[0];
    assert!(result.get("type").is_some(), "Result should have 'type' field");
    assert!(result.get("label").is_some(), "Result should have 'label' field");
    assert!(result.get("description").is_some(), "Result should have 'description' field");
    assert!(result.get("path").is_some(), "Result should have 'path' field");
}

#[tokio::test]
async fn test_search_finds_multiple_types() {
    let (addr, _pool) = setup_search_app().await;
    let client = reqwest::Client::new();

    // 'server' should match hosts and possibly other entities
    let resp = client
        .get(format!("http://{}/api/search?q=server", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let results = body["results"].as_array().unwrap();

    // Count unique types
    let mut types: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for r in results {
        if let Some(t) = r["type"].as_str() {
            types.insert(t);
        }
    }
    assert!(
        types.contains(&"host"),
        "Should find host results for 'server'"
    );
}

#[tokio::test]
async fn test_search_isolation() {
    // Verify each search app instance has its own data
    let (addr1, _pool1) = setup_search_app().await;
    let (addr2, _pool2) = setup_empty_search_app().await;

    let client = reqwest::Client::new();

    // Seeded db should have results
    let resp1 = client
        .get(format!("http://{}/api/search?q=web", addr1))
        .send()
        .await
        .unwrap();
    let body1: serde_json::Value = resp1.json().await.unwrap();
    assert!(!body1["results"].as_array().unwrap().is_empty());

    // Empty db should have no results
    let resp2 = client
        .get(format!("http://{}/api/search?q=web", addr2))
        .send()
        .await
        .unwrap();
    let body2: serde_json::Value = resp2.json().await.unwrap();
    assert!(body2["results"].as_array().unwrap().is_empty());
}
