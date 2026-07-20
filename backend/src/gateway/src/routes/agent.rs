//! REST handlers for the Agent Orchestrator (Task 2.10).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::agent::{AgentConfig, AgentOrchestrator};
use crate::llm::LlmClient;
use crate::tools::registry::ToolRegistry;

/// Shared state for agent routes.
#[derive(Clone)]
pub struct AgentState {
    pub orchestrator: std::sync::Arc<AgentOrchestrator>,
    pub ctx: std::sync::Arc<ops_pilot_sdk::context::ModuleContext>,
}

/// Request body for creating a new session.
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    /// Optional custom system prompt.
    pub system_prompt: Option<String>,
    /// Optional max tokens for context window.
    pub max_tokens: Option<usize>,
    /// Optional max turns per request.
    pub max_turns: Option<usize>,
}

/// Request body for sending a chat message.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
}

/// Response for session creation.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub session_id: String,
}

// ── Route handlers ──────────────────────────────────────────────────────────

/// POST /api/agent/session — create a new agent session.
async fn create_session(
    State(state): State<AgentState>,
    Json(req): Json<Option<CreateSessionRequest>>,
) -> impl IntoResponse {
    let _ = req; // Accept optional config but use defaults for now
    let session_id = state.orchestrator.create_session().await;
    (
        StatusCode::CREATED,
        Json(SessionResponse { session_id }),
    )
        .into_response()
}

/// POST /api/agent/chat — send a message and get the agent's response.
async fn chat(
    State(state): State<AgentState>,
    Path(session_id): Path<String>,
    Json(req): Json<ChatRequest>,
) -> impl IntoResponse {
    match state.orchestrator.chat(&session_id, &req.message, &state.ctx).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

/// DELETE /api/agent/session/:id — close an agent session.
async fn close_session(
    State(state): State<AgentState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    if state.orchestrator.close_session(&session_id).await {
        (StatusCode::OK, Json(serde_json::json!({"closed": true}))).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "session not found"})),
        )
            .into_response()
    }
}

/// Build the agent routes sub-router.
pub fn agent_routes(
    tool_registry: std::sync::Arc<ToolRegistry>,
    llm_client: std::sync::Arc<dyn LlmClient>,
    ctx: std::sync::Arc<ops_pilot_sdk::context::ModuleContext>,
    pool: sqlx::SqlitePool,
) -> Router {
    let orchestrator = std::sync::Arc::new(AgentOrchestrator::new(
        tool_registry,
        llm_client,
        AgentConfig::default(),
        pool,
    ));
    let state = AgentState { orchestrator, ctx };

    Router::new()
        .route("/api/agent/session", post(create_session))
        .route("/api/agent/chat/{session_id}", post(chat))
        .route("/api/agent/session/{session_id}", delete(close_session))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentResponse;
    use crate::llm::{CompletionResponse, LlmError, Message};
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::Request;
    use ops_pilot_sdk::context::{EventBus, ModuleContext};
    use ops_pilot_sdk::events::OpsEvent;
    use ops_pilot_sdk::loader::ModuleLoader;
    use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
    use serde_json::json;
    use sqlx::SqlitePool;
    use std::path::PathBuf;
    use std::pin::Pin;
    use std::sync::Arc;
    use tower::ServiceExt;

    // ── Mock LLM ───────────────────────────────────────────────────────

    struct MockLlm {
        responses: std::sync::Mutex<Vec<CompletionResponse>>,
    }

    impl MockLlm {
        fn single(text: &str) -> Self {
            Self {
                responses: std::sync::Mutex::new(vec![CompletionResponse::text(text)]),
            }
        }

        fn sequence(responses: Vec<CompletionResponse>) -> Self {
            Self {
                responses: std::sync::Mutex::new(responses),
            }
        }
    }

    #[async_trait]
    impl LlmClient for MockLlm {
        async fn complete(&self, _messages: &[Message]) -> Result<String, LlmError> {
            let mut r = self.responses.lock().unwrap();
            Ok(if r.is_empty() {
                "ok".into()
            } else {
                r.remove(0).content
            })
        }

        async fn complete_with_tools(
            &self,
            _messages: &[Message],
            _tools: &[serde_json::Value],
        ) -> Result<CompletionResponse, LlmError> {
            let mut r = self.responses.lock().unwrap();
            Ok(if r.is_empty() {
                CompletionResponse::text("ok")
            } else {
                r.remove(0)
            })
        }

        async fn complete_stream(
            &self,
            _messages: &[Message],
        ) -> Result<Pin<Box<dyn futures_util::Stream<Item = Result<String, LlmError>> + Send>>, LlmError>
        {
            Ok(Box::pin(futures_util::stream::empty()))
        }
    }

    // ── Stub Module ─────────────────────────────────────────────────────

    struct StubMod;

    #[async_trait]
    impl OpsModule for StubMod {
        fn name(&self) -> &str { "stub" }
        fn version(&self) -> &str { "0.1.0" }
        fn description(&self) -> &str { "stub" }
        fn dependencies(&self) -> Vec<&str> { vec![] }
        fn tools(&self) -> Vec<ToolDefinition> { vec![] }
        async fn execute(&self, _ctx: &ModuleContext, _t: &str, _p: serde_json::Value) -> anyhow::Result<serde_json::Value> {
            Ok(json!({}))
        }
        async fn on_event(&self, _ctx: &ModuleContext, _e: &OpsEvent) -> Option<ModuleAction> { None }
        async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus { HealthStatus::Healthy }
    }

    async fn test_app(llm: std::sync::Arc<dyn LlmClient>) -> Router {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        // Create agent_sessions table for tests
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
        .unwrap();

        let ctx = Arc::new(ModuleContext::new(
            Arc::new(pool.clone()),
            EventBus::new(16),
            PathBuf::from("/tmp/agent-test"),
            "test".into(),
        ));

        let mut loader = ModuleLoader::new();
        loader
            .load_module((*ctx).clone(), Box::new(StubMod))
            .await
            .unwrap();
        let manager = Arc::new(tokio::sync::RwLock::new(
            crate::routes::modules::ModuleManager::new(loader),
        ));
        let registry = Arc::new(ToolRegistry::new(manager));

        agent_routes(registry, llm, ctx, pool)
    }

    #[tokio::test]
    async fn test_create_session() {
        let app = test_app(std::sync::Arc::new(MockLlm::single("hi"))).await;
        let req = Request::builder()
            .method("POST")
            .uri("/api/agent/session")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let session: SessionResponse = serde_json::from_slice(&body).unwrap();
        assert!(!session.session_id.is_empty());
    }

    #[tokio::test]
    async fn test_chat_and_close() {
        let llm: std::sync::Arc<dyn LlmClient> = std::sync::Arc::new(MockLlm::sequence(vec![
            CompletionResponse::text("Hello! How can I help?"),
            CompletionResponse::text("Goodbye!"),
        ]));
        let app = test_app(llm).await;

        // Create session
        let req = Request::builder()
            .method("POST")
            .uri("/api/agent/session")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let session: SessionResponse = serde_json::from_slice(&body).unwrap();

        // Send message
        let req = Request::builder()
            .method("POST")
            .uri(format!("/api/agent/chat/{}", session.session_id))
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&ChatRequest {
                message: "Hi there".into(),
            }).unwrap()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let chat_resp: AgentResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(chat_resp.content, "Hello! How can I help?");

        // Close session
        let req = Request::builder()
            .method("DELETE")
            .uri(format!("/api/agent/session/{}", session.session_id))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_chat_session_not_found() {
        let app = test_app(std::sync::Arc::new(MockLlm::single("ok"))).await;
        let req = Request::builder()
            .method("POST")
            .uri("/api/agent/chat/nonexistent")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&ChatRequest {
                message: "hi".into(),
            }).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_close_nonexistent_session() {
        let app = test_app(std::sync::Arc::new(MockLlm::single("ok"))).await;
        let req = Request::builder()
            .method("DELETE")
            .uri("/api/agent/session/nonexistent")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
