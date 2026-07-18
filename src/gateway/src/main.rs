use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use ops_pilot_core::auth::AuthService;
use ops_pilot_core::db::Database;
use ops_pilot_core::ssh::SshConnectionPool;
use ops_pilot_gateway::agent::{AgentConfig, AgentOrchestrator};
use ops_pilot_gateway::routes::agent::agent_routes;
use ops_pilot_gateway::routes::hosts::host_routes;
use ops_pilot_gateway::routes::modules::{module_routes, ModuleManager};
use ops_pilot_gateway::tools::registry::ToolRegistry;
use ops_pilot_sdk::context::{EventBus, ModuleContext};
use serde::Deserialize;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

// ── Auth routes ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(serde::Serialize)]
struct AuthResponse {
    token: String,
}

#[derive(serde::Serialize)]
struct UserResponse {
    id: String,
    username: String,
    email: String,
}

#[derive(Clone)]
struct AuthState {
    service: Arc<AuthService>,
}

async fn register_handler(
    State(state): State<AuthState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    let user = state
        .service
        .register(&req.username, &req.email, &req.password)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(Json(UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
    }))
}

async fn login_handler(
    State(state): State<AuthState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let token = state
        .service
        .login(&req.username, &req.password)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    Ok(Json(AuthResponse { token }))
}

fn auth_routes(service: Arc<AuthService>) -> Router {
    let state = AuthState { service };
    Router::new()
        .route("/api/auth/register", post(register_handler))
        .route("/api/auth/login", post(login_handler))
        .with_state(state)
}

// ── Health routes ───────────────────────────────────────────────────────────

async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Placeholder LLM client used when no provider is configured.
struct PlaceholderLlm;

#[async_trait::async_trait]
impl ops_pilot_gateway::llm::LlmClient for PlaceholderLlm {
    async fn complete(
        &self,
        _messages: &[ops_pilot_gateway::llm::Message],
    ) -> Result<String, ops_pilot_gateway::llm::LlmError> {
        Ok("LLM not configured. Set LLM_PROVIDER and LLM_BASE_URL.".into())
    }

    async fn complete_with_tools(
        &self,
        _messages: &[ops_pilot_gateway::llm::Message],
        _tools: &[serde_json::Value],
    ) -> Result<ops_pilot_gateway::llm::CompletionResponse, ops_pilot_gateway::llm::LlmError> {
        Ok(ops_pilot_gateway::llm::CompletionResponse::text(
            "LLM not configured. Set LLM_PROVIDER and LLM_BASE_URL.",
        ))
    }

    async fn complete_stream(
        &self,
        _messages: &[ops_pilot_gateway::llm::Message],
    ) -> Result<
        std::pin::Pin<
            Box<
                dyn futures_util::Stream<
                        Item = Result<String, ops_pilot_gateway::llm::LlmError>,
                    > + Send,
            >,
        >,
        ops_pilot_gateway::llm::LlmError,
    > {
        Ok(Box::pin(futures_util::stream::empty()))
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ops_pilot=info,tower_http=info".into()),
        )
        .init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:ops-pilot.db".into());
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "change-me-to-a-random-string".into());

    let db = Database::new(&database_url)
        .await
        .expect("failed to initialize database");
    let pool = db.pool.clone();

    // Ensure agent_sessions table exists
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS agent_sessions (
            id TEXT PRIMARY KEY NOT NULL,
            messages TEXT NOT NULL,
            config TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'open',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&pool)
    .await
    .expect("failed to create agent_sessions table");

    let auth_service = Arc::new(AuthService::new(pool.clone(), jwt_secret));
    let host_service = Arc::new(ops_pilot_core::host::HostService::new(pool.clone()));
    let ssh_pool = Arc::new(SshConnectionPool::new());

    let module_loader = ops_pilot_sdk::loader::ModuleLoader::new();
    let module_manager = Arc::new(RwLock::new(ModuleManager::new(module_loader)));
    let tool_registry = Arc::new(ToolRegistry::new(module_manager.clone()));

    let event_bus = EventBus::new(256);
    let ctx = Arc::new(ModuleContext::new(
        Arc::new(pool.clone()),
        event_bus,
        std::path::PathBuf::from("/app/data"),
        "gateway".into(),
    ));

    let llm_client: Arc<dyn ops_pilot_gateway::llm::LlmClient> =
        match ops_pilot_gateway::llm::ProviderConfig::from_env() {
            Ok(config) => {
                tracing::info!(
                    provider = %config.provider_name,
                    model = %config.model,
                    base_url = %config.base_url,
                    "Using LLM provider"
                );
                Arc::new(ops_pilot_gateway::llm::ProviderLlm::new(config))
            }
            Err(_) => {
                tracing::warn!("No LLM provider configured (LLM_BASE_URL/LLM_API_KEY not set), using placeholder");
                Arc::new(PlaceholderLlm)
            }
        };

    let orchestrator = Arc::new(AgentOrchestrator::new(
        tool_registry.clone(),
        Arc::clone(&llm_client),
        AgentConfig::default(),
        pool.clone(),
    ));
    let _ = orchestrator.load_sessions().await;

    // Serve frontend static files from /app/static (or relative ./static)
    let static_dir = std::env::var("STATIC_DIR")
        .unwrap_or_else(|_| "static".into());
    let static_service = tower_http::services::ServeDir::new(&static_dir)
        .append_index_html_on_directories(true);

    let app = Router::new()
        .route("/api/v1/health", get(health_handler))
        .merge(auth_routes(auth_service.clone()))
        .merge(host_routes(host_service))
        .merge(module_routes(module_manager, ctx.clone()))
        .merge(agent_routes(tool_registry, llm_client, ctx, pool))
        .merge(ops_pilot_gateway::terminal::terminal_routes(
            ssh_pool,
            auth_service,
        ))
        .fallback_service(static_service)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3001".into());
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind");

    tracing::info!("OpsPilot gateway listening on {}", addr);
    axum::serve(listener, app).await.expect("server error");
}
