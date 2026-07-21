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
