//! OAuth2 login integration for Github, Gitlab, Google, and generic providers.
//!
//! Flow:
//! 1. GET /api/auth/oauth2/:provider — redirect to provider's authorization URL
//! 2. GET /api/auth/oauth2/:provider/callback — handle callback, exchange code for token,
//!    fetch user info, find or create user, return JWT.

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json,
};
use ops_pilot_core::auth::AuthService;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Clone)]
pub struct OAuth2State {
    pub service: Arc<AuthService>,
    /// In-memory state store (state_token → provider). In production use Redis/DB.
    pub states: Arc<tokio::sync::RwLock<HashMap<String, String>>>,
}

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
struct TokenResponse {
    pub token: String,
    pub role: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
struct GithubUser {
    login: String,
    email: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitlabUser {
    username: String,
    email: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleUser {
    email: String,
    name: Option<String>,
    #[allow(dead_code)]
    picture: Option<String>,
}

fn generate_state() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    format!("{:x}", ts)
}

/// Build the authorization URL for a given provider.
fn auth_url(provider: &str, state: &str) -> Option<String> {
    match provider {
        "github" => {
            let client_id = std::env::var("GITHUB_CLIENT_ID").ok()?;
            Some(format!(
                "https://github.com/login/oauth/authorize?client_id={}&scope=read:user+user:email&state={}",
                client_id, state
            ))
        }
        "gitlab" => {
            let client_id = std::env::var("GITLAB_CLIENT_ID").ok()?;
            let base = std::env::var("GITLAB_BASE_URL").unwrap_or_else(|_| "https://gitlab.com".into());
            Some(format!(
                "{}/oauth/authorize?client_id={}&redirect_uri=&response_type=code&scope=openid+profile+email&state={}",
                base, client_id, state
            ))
        }
        "google" => {
            let client_id = std::env::var("GOOGLE_CLIENT_ID").ok()?;
            Some(format!(
                "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri=&response_type=code&scope=openid+email+profile&state={}",
                client_id, state
            ))
        }
        _ => None,
    }
}

/// Exchange authorization code for access token, then fetch user info.
async fn exchange_and_fetch_user(
    provider: &str,
    code: &str,
) -> anyhow::Result<(String, String, String)> {
    let http = reqwest::Client::new();

    match provider {
        "github" => {
            let client_id = std::env::var("GITHUB_CLIENT_ID").map_err(|_| anyhow::anyhow!("GITHUB_CLIENT_ID not set"))?;
            let client_secret = std::env::var("GITHUB_CLIENT_SECRET").map_err(|_| anyhow::anyhow!("GITHUB_CLIENT_SECRET not set"))?;

            let token_resp: serde_json::Value = http.post("https://github.com/login/oauth/access_token")
                .header("Accept", "application/json")
                .json(&serde_json::json!({
                    "client_id": client_id,
                    "client_secret": client_secret,
                    "code": code,
                }))
                .send().await?.json().await?;

            let access_token = token_resp["access_token"].as_str()
                .ok_or_else(|| anyhow::anyhow!("no access_token: {:?}", token_resp))?;

            let user: GithubUser = http.get("https://api.github.com/user")
                .header("Authorization", format!("Bearer {}", access_token))
                .header("User-Agent", "OpsPilot")
                .send().await?.json().await?;

            let email = user.email.unwrap_or_else(|| format!("{}@github.local", user.login));
            let name = user.name.unwrap_or_else(|| user.login.clone());
            Ok((user.login, email, name))
        }
        "gitlab" => {
            let client_id = std::env::var("GITLAB_CLIENT_ID").map_err(|_| anyhow::anyhow!("GITLAB_CLIENT_ID not set"))?;
            let client_secret = std::env::var("GITLAB_CLIENT_SECRET").map_err(|_| anyhow::anyhow!("GITLAB_CLIENT_SECRET not set"))?;
            let base = std::env::var("GITLAB_BASE_URL").unwrap_or_else(|_| "https://gitlab.com".into());

            let token_resp: serde_json::Value = http.post(format!("{}/oauth/token", base))
                .header("Accept", "application/json")
                .json(&serde_json::json!({
                    "client_id": client_id,
                    "client_secret": client_secret,
                    "code": code,
                    "grant_type": "authorization_code",
                }))
                .send().await?.json().await?;

            let access_token = token_resp["access_token"].as_str()
                .ok_or_else(|| anyhow::anyhow!("no access_token"))?;

            let user: GitlabUser = http.get(format!("{}/api/v4/user", base))
                .header("Authorization", format!("Bearer {}", access_token))
                .send().await?.json().await?;

            let email = user.email.unwrap_or_else(|| format!("{}@gitlab.local", user.username));
            let name = user.name.unwrap_or_else(|| user.username.clone());
            Ok((user.username, email, name))
        }
        "google" => {
            let client_id = std::env::var("GOOGLE_CLIENT_ID").map_err(|_| anyhow::anyhow!("GOOGLE_CLIENT_ID not set"))?;
            let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").map_err(|_| anyhow::anyhow!("GOOGLE_CLIENT_SECRET not set"))?;

            let token_resp: serde_json::Value = http.post("https://oauth2.googleapis.com/token")
                .header("Accept", "application/json")
                .json(&serde_json::json!({
                    "client_id": client_id,
                    "client_secret": client_secret,
                    "code": code,
                    "grant_type": "authorization_code",
                }))
                .send().await?.json().await?;

            let access_token = token_resp["access_token"].as_str()
                .ok_or_else(|| anyhow::anyhow!("no access_token"))?;

            let user: GoogleUser = http.get("https://www.googleapis.com/oauth2/v2/userinfo")
                .header("Authorization", format!("Bearer {}", access_token))
                .send().await?.json().await?;

            let username = user.email.split('@').next().unwrap_or("user").to_string();
            let name = user.name.unwrap_or_else(|| username.clone());
            Ok((username, user.email, name))
        }
        _ => Err(anyhow::anyhow!("unsupported provider: {}", provider)),
    }
}

/// GET /api/auth/oauth2/:provider — redirect to provider authorization URL.
pub async fn oauth2_redirect(
    Path(provider): Path<String>,
    axum::extract::Extension(state): axum::extract::Extension<OAuth2State>,
) -> impl IntoResponse {
    let state_token = generate_state();
    state.states.write().await.insert(state_token.clone(), provider.clone());

    match auth_url(&provider, &state_token) {
        Some(url) => Redirect::temporary(&url).into_response(),
        None => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("unsupported provider: {}", provider)})),
        )
            .into_response(),
    }
}

/// GET /api/auth/oauth2/:provider/callback — handle OAuth2 callback.
pub async fn oauth2_callback(
    Path(provider): Path<String>,
    axum::extract::Extension(state): axum::extract::Extension<OAuth2State>,
    Query(params): Query<CallbackParams>,
) -> impl IntoResponse {
    // Validate state
    if let Some(error) = &params.error {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": format!("OAuth2 error: {}", error)})),
        )
            .into_response();
    }

    let state_token = params.state.as_deref().unwrap_or("");
    let stored = state.states.write().await.remove(state_token);
    if stored.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "invalid or expired state"})),
        )
            .into_response();
    }

    let code = match &params.code {
        Some(c) => c,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "missing authorization code"})),
            )
                .into_response();
        }
    };

    // Exchange code for user info
    match exchange_and_fetch_user(&provider, code).await {
        Ok((username, email, _name)) => {
            // Find or create user in database
            let service = &state.service;
            let user = match service.find_or_create_oauth_user(&username, &email, "operator").await {
                Ok(u) => u,
                Err(e) => {
                    warn!(error = %e, "Failed to find/create OAuth user");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "failed to process user"})),
                    )
                        .into_response();
                }
            };

            // Generate JWT
            match service.generate_token(&user.username, &user.role) {
                Ok(token) => {
                    info!(provider, username = %user.username, "OAuth2 login successful");
                    (StatusCode::OK, Json(TokenResponse { token, role: user.role, username: user.username })).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response(),
            }
        }
        Err(e) => {
            warn!(error = %e, provider, "OAuth2 exchange failed");
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": format!("OAuth2 authentication failed: {}", e)})),
            )
                .into_response()
        }
    }
}

/// GET /api/auth/oauth2/providers — list available OAuth2 providers.
pub async fn list_providers() -> impl IntoResponse {
    let mut providers = Vec::new();
    if std::env::var("GITHUB_CLIENT_ID").is_ok() {
        providers.push(serde_json::json!({"id": "github", "name": "GitHub", "icon": "github"}));
    }
    if std::env::var("GITLAB_CLIENT_ID").is_ok() {
        providers.push(serde_json::json!({"id": "gitlab", "name": "GitLab", "icon": "gitlab"}));
    }
    if std::env::var("GOOGLE_CLIENT_ID").is_ok() {
        providers.push(serde_json::json!({"id": "google", "name": "Google", "icon": "google"}));
    }
    (StatusCode::OK, Json(providers)).into_response()
}
