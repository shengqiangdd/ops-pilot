use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use ops_pilot_core::auth::{AuthService, Role};
use serde::Deserialize;
use std::sync::Arc;

/// Shared application state for user routes.
#[derive(Clone)]
pub struct UserState {
    pub service: Arc<AuthService>,
}

#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
    email: String,
    password: String,
    role: Option<String>,
}

#[derive(Deserialize)]
struct UpdateRoleRequest {
    role: String,
}

/// GET /api/users — list all users (admin only).
pub async fn list_users(
    State(state): State<UserState>,
) -> impl IntoResponse {
    match state.service.list_users().await {
        Ok(users) => (StatusCode::OK, Json(users)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/users/me — get current user info.
pub async fn get_current_user(
    axum::extract::Extension(claims): axum::extract::Extension<ops_pilot_core::auth::UserIdClaims>,
    State(state): State<UserState>,
) -> impl IntoResponse {
    match state.service.get_user(&claims.sub).await {
        Ok(Some(user)) => (StatusCode::OK, Json(user)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "user not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/users — create a new user (admin only).
pub async fn create_user(
    State(state): State<UserState>,
    Json(req): Json<CreateUserRequest>,
) -> impl IntoResponse {
    let role = req.role.unwrap_or_else(|| "operator".to_string());

    // Validate role
    let role_enum = Role::from_str(&role);
    if role_enum.as_str() != role.to_lowercase() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "invalid role"})),
        )
            .into_response();
    }

    match state
        .service
        .create_user(&req.username, &req.email, &req.password, &role)
        .await
    {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(e) => {
            let status = match &e {
                ops_pilot_core::auth::AuthError::UserExists => StatusCode::CONFLICT,
                ops_pilot_core::auth::AuthError::PasswordTooShort => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(serde_json::json!({"error": e.to_string()}))).into_response()
        }
    }
}

/// PUT /api/users/:id/role — update a user's role (admin only).
pub async fn update_user_role(
    Path(user_id): Path<String>,
    State(state): State<UserState>,
    Json(req): Json<UpdateRoleRequest>,
) -> impl IntoResponse {
    // Validate role
    let role_enum = Role::from_str(&req.role);
    if role_enum.as_str() != req.role.to_lowercase() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "invalid role"})),
        )
            .into_response();
    }

    match state.service.update_user_role(&user_id, &req.role).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// DELETE /api/users/:id — delete a user (admin only).
pub async fn delete_user(
    Path(user_id): Path<String>,
    State(state): State<UserState>,
) -> impl IntoResponse {
    match state.service.delete_user(&user_id).await {
        Ok(()) => (StatusCode::NO_CONTENT).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// Build the user routes sub-router.
pub fn user_routes(service: Arc<AuthService>) -> axum::Router {
    use axum::routing::{delete, get, put};

    let state = UserState { service };

    axum::Router::new()
        .route("/api/users", get(list_users).post(create_user))
        .route("/api/users/me", get(get_current_user))
        .route("/api/users/{id}/role", put(update_user_role))
        .route("/api/users/{id}", delete(delete_user))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> Arc<AuthService> {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY NOT NULL,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'operator',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        Arc::new(AuthService::new(pool, "test-secret".into()))
    }

    #[tokio::test]
    async fn test_list_users_empty() {
        let svc = setup().await;
        let state = UserState { service: svc };

        let resp = list_users(State(state)).await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_user() {
        let svc = setup().await;
        let state = UserState { service: svc };

        let req = CreateUserRequest {
            username: "alice".into(),
            email: "alice@test.com".into(),
            password: "password123".into(),
            role: Some("admin".into()),
        };

        let resp = create_user(State(state), Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_create_user_invalid_role() {
        let svc = setup().await;
        let state = UserState { service: svc };

        let req = CreateUserRequest {
            username: "bob".into(),
            email: "bob@test.com".into(),
            password: "password123".into(),
            role: Some("superadmin".into()),
        };

        let resp = create_user(State(state), Json(req)).await.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_role_permissions() {
        assert!(Role::Admin.has_permission("users:read"));
        assert!(Role::Admin.has_permission("anything"));
        assert!(Role::Operator.has_permission("hosts:read"));
        assert!(!Role::Operator.has_permission("users:write"));
        assert!(Role::Viewer.has_permission("hosts:read"));
        assert!(!Role::Viewer.has_permission("hosts:write"));
    }
}
