use axum::{
    extract::Query,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    #[serde(rename = "type")]
    type_: String,
    label: String,
    description: String,
    path: String,
}

/// Build the global search routes sub-router.
pub fn search_routes(pool: SqlitePool) -> Router {
    use axum::routing::get;

    let state = SearchState { pool };

    Router::new()
        .route("/api/search", get(search_handler))
        .with_state(state)
}

#[derive(Clone)]
struct SearchState {
    pool: SqlitePool,
}

async fn search_handler(
    Query(params): Query<SearchQuery>,
    axum::extract::State(state): axum::extract::State<SearchState>,
) -> Json<Value> {
    let q = params.q.trim();
    if q.is_empty() {
        return Json(serde_json::json!({"results": []}));
    }

    let pattern = format!("%{}%", q);
    let mut results: Vec<SearchResult> = Vec::new();

    // Search hosts
    if let Ok(hosts) = sqlx::query_as::<_, (String, String)>(
        "SELECT name, ip FROM hosts WHERE name LIKE ? OR ip LIKE ? LIMIT 5"
    )
    .bind(&pattern).bind(&pattern)
    .fetch_all(&state.pool).await
    {
        for (name, ip) in hosts {
            results.push(SearchResult {
                type_: "host".to_string(),
                label: name.clone(),
                description: ip,
                path: format!("/hosts?q={}", name),
            });
        }
    }

    // Search alerts
    if let Ok(alerts) = sqlx::query_as::<_, (String, String)>(
        "SELECT title, severity FROM alert_history WHERE title LIKE ? LIMIT 5"
    )
    .bind(&pattern)
    .fetch_all(&state.pool).await
    {
        for (title, severity) in alerts {
            results.push(SearchResult {
                type_: "alert".to_string(),
                label: title,
                description: format!("Severity: {}", severity),
                path: "/alert-history".to_string(),
            });
        }
    }

    // Search knowledge
    if let Ok(knowledge) = sqlx::query_as::<_, (String, String)>(
        "SELECT title, root_cause FROM knowledge_entries WHERE title LIKE ? OR root_cause LIKE ? LIMIT 5"
    )
    .bind(&pattern).bind(&pattern)
    .fetch_all(&state.pool).await
    {
        for (title, cause) in knowledge {
            results.push(SearchResult {
                type_: "knowledge".to_string(),
                label: title,
                description: cause,
                path: "/knowledge".to_string(),
            });
        }
    }

    // Search runbooks
    if let Ok(runbooks) = sqlx::query_as::<_, (String, String)>(
        "SELECT id, name FROM runbook WHERE name LIKE ? LIMIT 5"
    )
    .bind(&pattern)
    .fetch_all(&state.pool).await
    {
        for (id, name) in runbooks {
            results.push(SearchResult {
                type_: "runbook".to_string(),
                label: name,
                description: format!("Runbook #{}", id),
                path: "/runbook".to_string(),
            });
        }
    }

    Json(serde_json::json!({"results": results}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use tower::ServiceExt;

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS hosts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                ip TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO hosts (id, name, ip) VALUES ('h1', 'web-server', '10.0.0.1')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO hosts (id, name, ip) VALUES ('h2', 'db-server', '10.0.0.2')",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    #[tokio::test]
    async fn test_search_empty_query() {
        let pool = setup_pool().await;
        let app = search_routes(pool);

        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/search?q=")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();
        assert!(body["results"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_search_hosts_by_name() {
        let pool = setup_pool().await;
        let app = search_routes(pool);

        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/search?q=web")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();
        let results = body["results"].as_array().unwrap();
        assert!(!results.is_empty());
        let types: Vec<&str> = results
            .iter()
            .filter_map(|r| r["type"].as_str())
            .collect();
        assert!(types.contains(&"host"));
    }

    #[tokio::test]
    async fn test_search_hosts_by_ip() {
        let pool = setup_pool().await;
        let app = search_routes(pool);

        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/search?q=10.0.0.2")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();
        let results = body["results"].as_array().unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r["label"] == "db-server"));
    }

    #[tokio::test]
    async fn test_search_no_matches() {
        let pool = setup_pool().await;
        let app = search_routes(pool);

        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/search?q=zzzznonexistent")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .map(|b| serde_json::from_slice(&b).unwrap())
            .unwrap();
        assert!(body["results"].as_array().unwrap().is_empty());
    }
}
