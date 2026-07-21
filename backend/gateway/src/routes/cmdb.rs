use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::Sqlite;
use sqlx::SqlitePool;

/// Shared application state for CMDB routes.
#[derive(Clone)]
pub struct CmdbState {
    pub pool: SqlitePool,
}

// ── Service ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateServiceRequest {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub owner: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateServiceRequest {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub owner: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub owner: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ServiceDetail {
    pub service: Service,
    pub hosts: Vec<ServiceHost>,
    pub dependencies: Vec<ServiceDependency>,
}

#[derive(Debug, Deserialize)]
pub struct ServiceQuery {
    pub search: Option<String>,
    pub status: Option<String>,
}

/// GET /api/cmdb/services — list services
pub async fn list_services(
    State(state): State<CmdbState>,
    axum::extract::Query(query): axum::extract::Query<ServiceQuery>,
) -> impl IntoResponse {
    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new(
        "SELECT id, name, version, description, owner, status, created_at, updated_at FROM services WHERE 1=1"
    );

    if let Some(ref search) = query.search {
        builder.push(" AND (name LIKE ");
        builder.push_bind(format!("%{}%", search));
        builder.push(" OR description LIKE ");
        builder.push_bind(format!("%{}%", search));
        builder.push(")");
    }
    if let Some(ref status) = query.status {
        builder.push(" AND status = ");
        builder.push_bind(status.clone());
    }

    builder.push(" ORDER BY name ASC");

    let q = builder.build_query_as::<Service>();
    let result = q.fetch_all(&state.pool).await;

    match result {
        Ok(services) => (StatusCode::OK, Json(services)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/cmdb/services — create a service
pub async fn create_service(
    State(state): State<CmdbState>,
    Json(req): Json<CreateServiceRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();

    let result = sqlx::query(
        "INSERT INTO services (id, name, version, description, owner, status) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(req.version.as_deref().unwrap_or(""))
    .bind(req.description.as_deref().unwrap_or(""))
    .bind(req.owner.as_deref().unwrap_or(""))
    .bind(req.status.as_deref().unwrap_or("active"))
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let service = sqlx::query_as::<_, Service>(
                "SELECT id, name, version, description, owner, status, created_at, updated_at FROM services WHERE id = ?"
            )
            .bind(&id)
            .fetch_one(&state.pool)
            .await;
            match service {
                Ok(s) => (StatusCode::CREATED, Json(s)).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/cmdb/services/:id — get service detail
pub async fn get_service(
    Path(service_id): Path<String>,
    State(state): State<CmdbState>,
) -> impl IntoResponse {
    let service = sqlx::query_as::<_, Service>(
        "SELECT id, name, version, description, owner, status, created_at, updated_at FROM services WHERE id = ?"
    )
    .bind(&service_id)
    .fetch_optional(&state.pool)
    .await;

    match service {
        Ok(Some(s)) => {
            let hosts = get_service_hosts(&state.pool, &service_id).await;
            let deps = get_service_dependencies(&state.pool, &service_id).await;
            let detail = ServiceDetail { service: s, hosts, dependencies: deps };
            (StatusCode::OK, Json(detail)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "service not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// PUT /api/cmdb/services/:id — update a service
pub async fn update_service(
    Path(service_id): Path<String>,
    State(state): State<CmdbState>,
    Json(req): Json<UpdateServiceRequest>,
) -> impl IntoResponse {
    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new("UPDATE services SET ");

    let mut first = true;

    if let Some(name) = &req.name {
        if !first { builder.push(", "); }
        builder.push("name = ");
        builder.push_bind(name.clone());
        first = false;
    }
    if let Some(version) = &req.version {
        if !first { builder.push(", "); }
        builder.push("version = ");
        builder.push_bind(version.clone());
        first = false;
    }
    if let Some(description) = &req.description {
        if !first { builder.push(", "); }
        builder.push("description = ");
        builder.push_bind(description.clone());
        first = false;
    }
    if let Some(owner) = &req.owner {
        if !first { builder.push(", "); }
        builder.push("owner = ");
        builder.push_bind(owner.clone());
        first = false;
    }
    if let Some(status) = &req.status {
        if !first { builder.push(", "); }
        builder.push("status = ");
        builder.push_bind(status.clone());
        first = false;
    }

    if first {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "no fields to update"})),
        )
            .into_response();
    }

    builder.push(", updated_at = datetime('now')");
    builder.push(" WHERE id = ");
    builder.push_bind(service_id.clone());

    let result = builder.build().execute(&state.pool).await;

    match result {
        Ok(_) => {
            let service = sqlx::query_as::<_, Service>(
                "SELECT id, name, version, description, owner, status, created_at, updated_at FROM services WHERE id = ?"
            )
            .bind(&service_id)
            .fetch_one(&state.pool)
            .await;
            match service {
                Ok(s) => (StatusCode::OK, Json(s)).into_response(),
                Err(e) => (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// DELETE /api/cmdb/services/:id — delete a service
pub async fn delete_service(
    Path(service_id): Path<String>,
    State(state): State<CmdbState>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM services WHERE id = ?")
        .bind(&service_id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// ── Service Hosts ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AddServiceHostRequest {
    pub host_id: String,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ServiceHost {
    pub id: String,
    pub service_id: String,
    pub host_id: String,
    pub role: String,
    pub created_at: String,
}

/// POST /api/cmdb/services/:id/hosts — add host to service
pub async fn add_service_host(
    Path(service_id): Path<String>,
    State(state): State<CmdbState>,
    Json(req): Json<AddServiceHostRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let role = req.role.as_deref().unwrap_or("app");

    let result = sqlx::query(
        "INSERT INTO service_hosts (id, service_id, host_id, role) VALUES (?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&service_id)
    .bind(&req.host_id)
    .bind(role)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"status": "ok", "id": id}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// DELETE /api/cmdb/services/:id/hosts/:hostId — remove host from service
pub async fn remove_service_host(
    Path((service_id, host_id)): Path<(String, String)>,
    State(state): State<CmdbState>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM service_hosts WHERE service_id = ? AND host_id = ?")
        .bind(&service_id)
        .bind(&host_id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn get_service_hosts(pool: &SqlitePool, service_id: &str) -> Vec<ServiceHost> {
    sqlx::query_as::<_, ServiceHost>(
        "SELECT id, service_id, host_id, role, created_at FROM service_hosts WHERE service_id = ?"
    )
    .bind(service_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

// ── Service Dependencies ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AddDependencyRequest {
    pub target_service_id: String,
    pub dependency_type: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ServiceDependency {
    pub id: String,
    pub source_service_id: String,
    pub target_service_id: String,
    pub dependency_type: String,
    pub description: String,
    pub created_at: String,
}

/// GET /api/cmdb/services/:id/dependencies — get service dependencies
pub async fn get_dependencies(
    Path(service_id): Path<String>,
    State(state): State<CmdbState>,
) -> impl IntoResponse {
    let deps = get_service_dependencies(&state.pool, &service_id).await;
    (StatusCode::OK, Json(deps)).into_response()
}

/// POST /api/cmdb/services/:id/dependencies — add dependency
pub async fn add_dependency(
    Path(service_id): Path<String>,
    State(state): State<CmdbState>,
    Json(req): Json<AddDependencyRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let dep_type = req.dependency_type.as_deref().unwrap_or("hard");

    let result = sqlx::query(
        "INSERT INTO service_dependencies (id, source_service_id, target_service_id, dependency_type, description) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&service_id)
    .bind(&req.target_service_id)
    .bind(dep_type)
    .bind(req.description.as_deref().unwrap_or(""))
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"status": "ok", "id": id}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn get_service_dependencies(pool: &SqlitePool, service_id: &str) -> Vec<ServiceDependency> {
    sqlx::query_as::<_, ServiceDependency>(
        "SELECT id, source_service_id, target_service_id, dependency_type, description, created_at FROM service_dependencies WHERE source_service_id = ? OR target_service_id = ?"
    )
    .bind(service_id)
    .bind(service_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

// ── Config Versions ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateConfigVersionRequest {
    pub service_id: String,
    pub config_json: String,
    pub changed_by: Option<String>,
    pub change_note: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ConfigVersion {
    pub id: String,
    pub service_id: String,
    pub config_json: String,
    pub version: i64,
    pub changed_by: String,
    pub change_note: String,
    pub created_at: String,
}

/// GET /api/cmdb/configs — list config versions
pub async fn list_configs(
    State(state): State<CmdbState>,
    axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let service_id = query.get("service_id");

    let result = if let Some(sid) = service_id {
        sqlx::query_as::<_, ConfigVersion>(
            "SELECT id, service_id, config_json, version, changed_by, change_note, created_at FROM config_versions WHERE service_id = ? ORDER BY version DESC"
        )
        .bind(sid)
        .fetch_all(&state.pool)
        .await
    } else {
        sqlx::query_as::<_, ConfigVersion>(
            "SELECT id, service_id, config_json, version, changed_by, change_note, created_at FROM config_versions ORDER BY created_at DESC LIMIT 100"
        )
        .fetch_all(&state.pool)
        .await
    };

    match result {
        Ok(configs) => (StatusCode::OK, Json(configs)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/cmdb/configs — create a new config version
pub async fn create_config_version(
    State(state): State<CmdbState>,
    Json(req): Json<CreateConfigVersionRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();

    // Get next version number
    let max_version: Option<(i64,)> = sqlx::query_as(
        "SELECT MAX(version) FROM config_versions WHERE service_id = ?"
    )
    .bind(&req.service_id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    let version = max_version.map(|m| m.0).unwrap_or(0) + 1;

    let result = sqlx::query(
        "INSERT INTO config_versions (id, service_id, config_json, version, changed_by, change_note) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.service_id)
    .bind(&req.config_json)
    .bind(version)
    .bind(req.changed_by.as_deref().unwrap_or(""))
    .bind(req.change_note.as_deref().unwrap_or(""))
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let config = sqlx::query_as::<_, ConfigVersion>(
                "SELECT id, service_id, config_json, version, changed_by, change_note, created_at FROM config_versions WHERE id = ?"
            )
            .bind(&id)
            .fetch_one(&state.pool)
            .await;
            match config {
                Ok(c) => (StatusCode::CREATED, Json(c)).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/cmdb/configs/:id — get config version detail
pub async fn get_config_version(
    Path(config_id): Path<String>,
    State(state): State<CmdbState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, ConfigVersion>(
        "SELECT id, service_id, config_json, version, changed_by, change_note, created_at FROM config_versions WHERE id = ?"
    )
    .bind(&config_id)
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(c)) => (StatusCode::OK, Json(c)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "config version not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Build the CMDB routes sub-router.
pub fn cmdb_routes(pool: SqlitePool) -> Router {
    use axum::routing::{delete, get, post};

    let state = CmdbState { pool };

    Router::new()
        .route("/api/cmdb/services", get(list_services).post(create_service))
        .route("/api/cmdb/services/{id}", get(get_service).put(update_service).delete(delete_service))
        .route("/api/cmdb/services/{id}/hosts", post(add_service_host))
        .route("/api/cmdb/services/{id}/hosts/{host_id}", delete(remove_service_host))
        .route("/api/cmdb/services/{id}/dependencies", get(get_dependencies).post(add_dependency))
        .route("/api/cmdb/configs", get(list_configs).post(create_config_version))
        .route("/api/cmdb/configs/{id}", get(get_config_version))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS services (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, version TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '', owner TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'active',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS service_hosts (
                id TEXT PRIMARY KEY, service_id TEXT NOT NULL, host_id TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'app', created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS service_dependencies (
                id TEXT PRIMARY KEY, source_service_id TEXT NOT NULL, target_service_id TEXT NOT NULL,
                dependency_type TEXT NOT NULL DEFAULT 'hard', description TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS config_versions (
                id TEXT PRIMARY KEY, service_id TEXT NOT NULL, config_json TEXT NOT NULL DEFAULT '{}',
                version INTEGER NOT NULL DEFAULT 1, changed_by TEXT NOT NULL DEFAULT '',
                change_note TEXT NOT NULL DEFAULT '', created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_create_and_list_services() {
        let pool = setup().await;
        let state = CmdbState { pool };

        let req = CreateServiceRequest {
            name: "web-api".into(),
            version: Some("1.0.0".into()),
            description: Some("Web API service".into()),
            owner: Some("team-backend".into()),
            status: None,
        };

        let resp = create_service(State(state.clone()), Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = list_services(State(state), axum::extract::Query(ServiceQuery { search: None, status: None })).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
