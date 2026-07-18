//! REST handlers for vault passphrase management — all endpoints require JWT auth.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use ops_pilot_core::auth::AuthService;
use ops_pilot_core::vault::VaultKeyManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::middleware::AuthLayer;

/// Shared application state for vault routes.
#[derive(Clone)]
pub struct VaultState {
    pub auth: Arc<AuthService>,
    pub vault_keys: Arc<VaultKeyManager>,
}

/// Request body for setting vault passphrase.
#[derive(Deserialize)]
pub struct SetPassphraseRequest {
    pub login_password: String,
    pub passphrase: String,
    pub passphrase_confirm: String,
}

/// Request body for unlocking the vault.
#[derive(Deserialize)]
pub struct UnlockRequest {
    pub login_password: String,
    pub passphrase: String,
}

/// Response for vault status.
#[derive(Serialize, Deserialize)]
pub struct VaultStatusResponse {
    pub unlocked: bool,
    pub has_passphrase: bool,
}

/// POST /api/vault/set-passphrase — set or update vault passphrase.
pub async fn set_passphrase(
    State(state): State<VaultState>,
    AuthLayer(claims): AuthLayer,
    Json(req): Json<SetPassphraseRequest>,
) -> impl IntoResponse {
    if req.passphrase != req.passphrase_confirm {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "passphrases do not match"})),
        )
            .into_response();
    }

    match state
        .auth
        .set_vault_passphrase(&claims.sub, &req.login_password, &req.passphrase)
        .await
    {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response(),
        Err(e) => {
            let status = match &e {
                ops_pilot_core::auth::AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
                ops_pilot_core::auth::AuthError::PassphraseTooShort => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(serde_json::json!({"error": e.to_string()}))).into_response()
        }
    }
}

/// POST /api/vault/unlock — unlock vault with passphrase, cache vault key.
pub async fn unlock(
    State(state): State<VaultState>,
    AuthLayer(claims): AuthLayer,
    Json(req): Json<UnlockRequest>,
) -> impl IntoResponse {
    match state
        .auth
        .unlock_vault(&claims.sub, &req.login_password, &req.passphrase)
        .await
    {
        Ok(vault_key) => {
            state.vault_keys.set(&claims.sub, vault_key);
            (
                StatusCode::OK,
                Json(serde_json::json!({"status": "unlocked"})),
            )
                .into_response()
        }
        Err(e) => {
            let status = match &e {
                ops_pilot_core::auth::AuthError::VaultNotSetup => StatusCode::NOT_FOUND,
                ops_pilot_core::auth::AuthError::VaultPassphraseMismatch => StatusCode::UNAUTHORIZED,
                ops_pilot_core::auth::AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(serde_json::json!({"error": e.to_string()}))).into_response()
        }
    }
}

/// POST /api/vault/lock — lock vault, remove cached key.
pub async fn lock(
    State(state): State<VaultState>,
    AuthLayer(claims): AuthLayer,
) -> impl IntoResponse {
    state.vault_keys.remove(&claims.sub);
    (StatusCode::OK, Json(serde_json::json!({"status": "locked"}))).into_response()
}

/// GET /api/vault/status — check vault lock status and passphrase existence.
pub async fn status(
    State(state): State<VaultState>,
    AuthLayer(claims): AuthLayer,
) -> impl IntoResponse {
    let unlocked = state.vault_keys.is_unlocked(&claims.sub);
    let has_passphrase = state.auth.has_vault(&claims.sub).await.unwrap_or(false);

    (
        StatusCode::OK,
        Json(VaultStatusResponse {
            unlocked,
            has_passphrase,
        }),
    )
        .into_response()
}

/// Build the vault routes sub-router **without** auth middleware.
pub fn vault_routes(auth: Arc<AuthService>, vault_keys: Arc<VaultKeyManager>) -> axum::Router {
    use axum::routing::{get, post};

    let state = VaultState { auth, vault_keys };

    axum::Router::new()
        .route("/api/vault/set-passphrase", post(set_passphrase))
        .route("/api/vault/unlock", post(unlock))
        .route("/api/vault/lock", post(lock))
        .route("/api/vault/status", get(status))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, Method};
    use axum::routing::{get, post};
    use ops_pilot_core::auth::AuthService;
    use ops_pilot_core::db::Database;
    use tower::ServiceExt;

    const TEST_SECRET: &str = "test-secret";

    async fn setup_with_auth() -> (axum::Router, Arc<AuthService>, Arc<VaultKeyManager>) {
        let db = Database::open_in_memory().await.unwrap();
        let vault_keys = Arc::new(VaultKeyManager::new());
        let auth = Arc::new(AuthService::new(db.pool, TEST_SECRET.into()));

        auth.register("testuser", "test@example.com", "password123")
            .await
            .unwrap();

        let auth_state = crate::middleware::AuthState {
            service: auth.clone(),
        };

        let vault_state = VaultState {
            auth: auth.clone(),
            vault_keys: vault_keys.clone(),
        };

        let app = axum::Router::new()
            .route("/api/vault/set-passphrase", post(set_passphrase))
            .route("/api/vault/unlock", post(unlock))
            .route("/api/vault/lock", post(lock))
            .route("/api/vault/status", get(status))
            .with_state(vault_state)
            .layer(axum::middleware::from_fn_with_state(
                auth_state,
                crate::middleware::auth_middleware,
            ));

        (app, auth, vault_keys)
    }

    fn auth_request(
        method: Method,
        uri: &str,
        token: &str,
        body: Option<serde_json::Value>,
    ) -> Request<Body> {
        let mut builder = Request::builder()
            .method(method)
            .uri(uri)
            .header("authorization", format!("Bearer {}", token))
            .header("content-type", "application/json");

        let body = match body {
            Some(v) => Body::from(v.to_string()),
            None => Body::empty(),
        };

        builder.body(body).unwrap()
    }

    #[tokio::test]
    async fn test_set_passphrase_success() {
        let (app, auth, _) = setup_with_auth().await;
        let token = auth.login("testuser", "password123").await.unwrap();

        let body = serde_json::json!({
            "login_password": "password123",
            "passphrase": "my-vault-pass",
            "passphrase_confirm": "my-vault-pass"
        });
        let req = auth_request(Method::POST, "/api/vault/set-passphrase", &token, Some(body));
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_set_passphrase_mismatch() {
        let (app, auth, _) = setup_with_auth().await;
        let token = auth.login("testuser", "password123").await.unwrap();

        let body = serde_json::json!({
            "login_password": "password123",
            "passphrase": "my-vault-pass",
            "passphrase_confirm": "different-pass"
        });
        let req = auth_request(Method::POST, "/api/vault/set-passphrase", &token, Some(body));
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_set_passphrase_wrong_login() {
        let (app, auth, _) = setup_with_auth().await;
        let token = auth.login("testuser", "password123").await.unwrap();

        let body = serde_json::json!({
            "login_password": "wrongpassword",
            "passphrase": "my-vault-pass",
            "passphrase_confirm": "my-vault-pass"
        });
        let req = auth_request(Method::POST, "/api/vault/set-passphrase", &token, Some(body));
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_unlock_vault_success() {
        let (app, auth, vault_keys) = setup_with_auth().await;
        let token = auth.login("testuser", "password123").await.unwrap();

        // Set passphrase first
        let body = serde_json::json!({
            "login_password": "password123",
            "passphrase": "my-vault-pass",
            "passphrase_confirm": "my-vault-pass"
        });
        let req = auth_request(Method::POST, "/api/vault/set-passphrase", &token, Some(body));
        app.clone().oneshot(req).await.unwrap();

        // Unlock
        let body = serde_json::json!({
            "login_password": "password123",
            "passphrase": "my-vault-pass"
        });
        let req = auth_request(Method::POST, "/api/vault/unlock", &token, Some(body));
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify key is cached
        let user_id = auth.verify_token(&token).unwrap().sub;
        assert!(vault_keys.is_unlocked(&user_id));
    }

    #[tokio::test]
    async fn test_unlock_vault_not_setup() {
        let (app, auth, _) = setup_with_auth().await;
        let token = auth.login("testuser", "password123").await.unwrap();

        let body = serde_json::json!({
            "login_password": "password123",
            "passphrase": "anything"
        });
        let req = auth_request(Method::POST, "/api/vault/unlock", &token, Some(body));
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_lock_vault() {
        let (app, auth, vault_keys) = setup_with_auth().await;
        let token = auth.login("testuser", "password123").await.unwrap();
        let user_id = auth.verify_token(&token).unwrap().sub;

        // Set and unlock first
        let body = serde_json::json!({
            "login_password": "password123",
            "passphrase": "my-vault-pass",
            "passphrase_confirm": "my-vault-pass"
        });
        let req = auth_request(Method::POST, "/api/vault/set-passphrase", &token, Some(body));
        app.clone().oneshot(req).await.unwrap();

        let body = serde_json::json!({
            "login_password": "password123",
            "passphrase": "my-vault-pass"
        });
        let req = auth_request(Method::POST, "/api/vault/unlock", &token, Some(body));
        app.clone().oneshot(req).await.unwrap();
        assert!(vault_keys.is_unlocked(&user_id));

        // Lock
        let req = auth_request(Method::POST, "/api/vault/lock", &token, None);
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(!vault_keys.is_unlocked(&user_id));
    }

    #[tokio::test]
    async fn test_vault_status() {
        let (app, auth, _) = setup_with_auth().await;
        let token = auth.login("testuser", "password123").await.unwrap();

        let req = auth_request(Method::GET, "/api/vault/status", &token, None);
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let status: VaultStatusResponse = serde_json::from_slice(&body).unwrap();
        assert!(!status.unlocked);
        assert!(!status.has_passphrase);
    }

    #[tokio::test]
    async fn test_vault_status_after_setup() {
        let (app, auth, _) = setup_with_auth().await;
        let token = auth.login("testuser", "password123").await.unwrap();

        // Set passphrase
        let body = serde_json::json!({
            "login_password": "password123",
            "passphrase": "my-vault-pass",
            "passphrase_confirm": "my-vault-pass"
        });
        let req = auth_request(Method::POST, "/api/vault/set-passphrase", &token, Some(body));
        app.clone().oneshot(req).await.unwrap();

        let req = auth_request(Method::GET, "/api/vault/status", &token, None);
        let resp = app.oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let status: VaultStatusResponse = serde_json::from_slice(&body).unwrap();
        assert!(!status.unlocked);
        assert!(status.has_passphrase);
    }

    #[tokio::test]
    async fn test_unauthenticated_rejected() {
        let (app, _, _) = setup_with_auth().await;

        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/vault/status")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
