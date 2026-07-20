//! Axum middleware and extractor for JWT authentication.
//!
//! The `auth_middleware` validates the `Authorization: Bearer <token>` header,
//! verifies it with `AuthService`, and inserts the `UserIdClaims` into the
//! request extensions so downstream handlers can access the authenticated user.

pub mod rate_limit;

use axum::{
    extract::{FromRequestParts, Request, State},
    http::header::AUTHORIZATION,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use ops_pilot_core::auth::{AuthService, UserIdClaims};
use std::sync::Arc;

/// Shared auth state passed to the middleware via axum State.
#[derive(Clone)]
pub struct AuthState {
    pub service: Arc<AuthService>,
}

/// Axum middleware that validates JWT tokens on protected routes.
///
/// Reads `AuthState` from axum State, extracts the `Authorization: Bearer <token>`
/// header, verifies the token, and inserts the `UserIdClaims` into request
/// extensions. Returns 401 on missing or invalid tokens.
pub async fn auth_middleware(
    State(state): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = state
        .service
        .verify_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Axum extractor that retrieves the authenticated user's claims from request
/// extensions. Must be used on routes protected by `auth_middleware`.
///
/// # Example
///
/// ```ignore
/// async fn protected_handler(
///     AuthLayer(claims): AuthLayer,
/// ) -> impl IntoResponse {
///     format!("Hello user {}", claims.sub)
/// }
/// ```
#[derive(Debug)]
pub struct AuthLayer(pub UserIdClaims);

impl<S> FromRequestParts<S> for AuthLayer
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let claims = parts
            .extensions
            .get::<UserIdClaims>()
            .cloned()
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "missing authentication claims".to_string(),
            ))?;
        Ok(AuthLayer(claims))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::Router;
    use axum::routing::get;
    use tower::ServiceExt;

    async fn setup_svc() -> Arc<AuthService> {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY NOT NULL,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                vault_key_encrypted TEXT,
                vault_password_hash TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&pool)
        .await
        .unwrap();
        Arc::new(AuthService::new(pool, "test-secret".into()))
    }

    fn test_app(state: AuthState) -> Router {
        Router::new()
            .route("/protected", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_middleware_rejects_no_header() {
        let svc = setup_svc().await;
        let app = test_app(AuthState { service: svc });

        let req = Request::builder()
            .uri("/protected")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_middleware_rejects_invalid_token() {
        let svc = setup_svc().await;
        let app = test_app(AuthState { service: svc });

        let req = Request::builder()
            .uri("/protected")
            .header(AUTHORIZATION, "Bearer invalid-token")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_middleware_accepts_valid_token() {
        let svc = setup_svc().await;
        svc.register("testuser", "test@example.com", "password123")
            .await
            .unwrap();
        let token = svc.login("testuser", "password123").await.unwrap();

        let app = test_app(AuthState { service: svc });

        let req = Request::builder()
            .uri("/protected")
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_middleware_rejects_non_bearer() {
        let svc = setup_svc().await;
        let app = test_app(AuthState { service: svc });

        let req = Request::builder()
            .uri("/protected")
            .header(AUTHORIZATION, "Basic dXNlcjpwYXNz")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_layer_extractor_missing() {
        let (parts, _) = Request::builder().body(Body::empty()).unwrap().into_parts();
        let mut parts = parts;
        let result = AuthLayer::from_request_parts(&mut parts, &()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().0, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_layer_extractor_present() {
        let claims = UserIdClaims {
            sub: "user-1".into(),
            iat: 1000000,
            exp: 10086400,
        };
        let (parts, _) = Request::builder().body(Body::empty()).unwrap().into_parts();
        let mut parts = parts;
        parts.extensions.insert(claims.clone());
        let result = AuthLayer::from_request_parts(&mut parts, &()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.sub, "user-1");
    }
}
