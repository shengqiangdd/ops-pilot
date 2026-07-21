//! Roles management routes — CRUD for custom roles and user role assignment.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::Sqlite;
use sqlx::SqlitePool;

/// Shared application state for roles routes.
#[derive(Clone)]
pub struct RolesState {
    pub pool: SqlitePool,
}

#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub permissions: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub permissions: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRoleRequest {
    pub role: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub permissions: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub created_at: String,
}

// ── Role Handlers ──────────────────────────────────────────────────────

/// GET /api/roles — list all roles
pub async fn list_roles(
    State(state): State<RolesState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, Role>(
        "SELECT id, name, permissions, created_at FROM roles ORDER BY name"
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(roles) => (StatusCode::OK, Json(roles)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/roles — create a new role
pub async fn create_role(
    State(state): State<RolesState>,
    Json(req): Json<CreateRoleRequest>,
) -> impl IntoResponse {
    let id = uuid::Uuid::new_v4().to_string();

    let result = sqlx::query(
        "INSERT INTO roles (id, name, permissions) VALUES (?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.permissions)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => {
            let role = sqlx::query_as::<_, Role>(
                "SELECT id, name, permissions, created_at FROM roles WHERE id = ?"
            )
            .bind(&id)
            .fetch_one(&state.pool)
            .await;
            match role {
                Ok(r) => (StatusCode::CREATED, Json(r)).into_response(),
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

/// PUT /api/roles/:id — update a role
pub async fn update_role(
    Path(role_id): Path<String>,
    State(state): State<RolesState>,
    Json(req): Json<UpdateRoleRequest>,
) -> impl IntoResponse {
    let mut builder: sqlx::QueryBuilder<Sqlite> = sqlx::QueryBuilder::new("UPDATE roles SET ");

    let mut first = true;
    if let Some(name) = &req.name {
        builder.push("name = ");
        builder.push_bind(name.clone());
        first = false;
    }
    if let Some(permissions) = &req.permissions {
        if !first { builder.push(", "); }
        builder.push("permissions = ");
        builder.push_bind(permissions.clone());
        first = false;
    }

    if first {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "no fields to update"})),
        )
            .into_response();
    }

    builder.push(" WHERE id = ");
    builder.push_bind(role_id.clone());

    match builder.build().execute(&state.pool).await {
        Ok(_) => {
            let role = sqlx::query_as::<_, Role>(
                "SELECT id, name, permissions, created_at FROM roles WHERE id = ?"
            )
            .bind(&role_id)
            .fetch_one(&state.pool)
            .await;
            match role {
                Ok(r) => (StatusCode::OK, Json(r)).into_response(),
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

/// DELETE /api/roles/:id — delete a role
pub async fn delete_role(
    Path(role_id): Path<String>,
    State(state): State<RolesState>,
) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM roles WHERE id = ?")
        .bind(&role_id)
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

// ── User Role Handlers ─────────────────────────────────────────────────

/// PUT /api/users/:id/role — update user role
pub async fn update_user_role(
    Path(user_id): Path<String>,
    State(state): State<RolesState>,
    Json(req): Json<UpdateUserRoleRequest>,
) -> impl IntoResponse {
    // Validate role
    let role_enum = match req.role.as_str() {
        "admin" | "operator" | "viewer" => req.role.clone(),
        _ => return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "invalid role"})),
        )
            .into_response(),
    };

    let result = sqlx::query("UPDATE users SET role = ? WHERE id = ?")
        .bind(&role_enum)
        .bind(&user_id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "ok", "role": role_enum}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/users — list all users with role info
pub async fn list_users(
    State(state): State<RolesState>,
) -> impl IntoResponse {
    let result = sqlx::query_as::<_, UserInfo>(
        "SELECT id, username, email, role, created_at FROM users ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(users) => (StatusCode::OK, Json(users)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Build the roles routes sub-router.
pub fn roles_routes(pool: SqlitePool) -> Router {
    use axum::routing::{get, put};

    let state = RolesState { pool };

    Router::new()
        .route("/api/roles", get(list_roles).post(create_role))
        .route("/api/roles/{id}", put(update_role).delete(delete_role))
        .route("/api/users/role/{id}", put(update_user_role))
        .route("/api/users", get(list_users))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS roles (
                id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE,
                permissions TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY, username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE, password_hash TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'operator',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )"
        ).execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_create_and_list_roles() {
        let pool = setup().await;
        let state = RolesState { pool };

        let req = CreateRoleRequest {
            name: "custom_role".into(),
            permissions: r#"["hosts:read","hosts:write"]"#.into(),
        };

        let resp = create_role(State(state.clone()), Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let resp = list_roles(State(state)).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
