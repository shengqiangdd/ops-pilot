//! Agent orchestration: ReAct loop, tool calling, session management.
//!
//! Implements a Reason-Act loop where the LLM thinks, optionally calls tools,
//! and feeds results back until it produces a final text response.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

use crate::llm::{LlmClient, Message, Role, ToolCall};
use crate::tools::registry::ToolRegistry;

// ── Constants ───────────────────────────────────────────────────────────────

/// Default maximum number of think-act turns before forcing a response.
const DEFAULT_MAX_TURNS: usize = 10;

/// Approximate characters per token (English).
const CHARS_PER_TOKEN: usize = 4;

// ── Types ───────────────────────────────────────────────────────────────────

/// Configuration for an agent session.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// System prompt prepended to every LLM call.
    pub system_prompt: String,
    /// Maximum tokens the context window can hold.
    pub max_tokens: usize,
    /// Maximum think-act turns per user message.
    pub max_turns: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            system_prompt: "You are OpsPilot, an AI infrastructure operations assistant. \
                You have access to tools for managing servers, containers, and monitoring. \
                Use tools when needed to help the user. Think step by step."
                .into(),
            max_tokens: 8_000,
            max_turns: DEFAULT_MAX_TURNS,
        }
    }
}

/// A single turn in the ReAct loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTurn {
    pub turn: usize,
    pub action: String,
    pub result: String,
}

/// Result of processing a user message through the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub session_id: String,
    pub content: String,
    pub turns: Vec<AgentTurn>,
    pub truncated: bool,
}

/// The agent session holding conversation state.
pub struct AgentSession {
    pub session_id: String,
    messages: Vec<Message>,
    config: AgentConfig,
    tool_registry: Arc<ToolRegistry>,
    llm_client: Arc<dyn LlmClient>,
    turns: Vec<AgentTurn>,
}

impl AgentSession {
    pub fn new(
        tool_registry: Arc<ToolRegistry>,
        llm_client: Arc<dyn LlmClient>,
        config: AgentConfig,
    ) -> Self {
        let session_id = Uuid::new_v4().to_string();
        let mut messages = Vec::new();
        messages.push(Message::system(&config.system_prompt));

        Self {
            session_id,
            messages,
            config,
            tool_registry,
            llm_client,
            turns: Vec::new(),
        }
    }

    /// Process a user message through the ReAct loop.
    pub async fn chat(
        &mut self,
        user_message: &str,
        ctx: &ops_pilot_sdk::context::ModuleContext,
    ) -> Result<AgentResponse> {
        self.messages.push(Message::user(user_message));
        self.truncate_if_needed();

        let tools = self.tool_registry.get_tools_for_llm().await;
        let mut turn_count = 0;
        let truncated = false;

        loop {
            if turn_count >= self.config.max_turns {
                warn!(
                    session = %self.session_id,
                    turns = turn_count,
                    "Max turns reached, forcing final response"
                );
                // Ask LLM for a final text response without tools
                let response = self.llm_client.complete(&self.messages).await?;
                self.messages.push(Message::assistant(&response));
                return Ok(AgentResponse {
                    session_id: self.session_id.clone(),
                    content: response,
                    turns: self.turns.clone(),
                    truncated,
                });
            }

            let completion = self
                .llm_client
                .complete_with_tools(&self.messages, &tools)
                .await?;

            if completion.has_tool_calls() {
                // Record the assistant message with tool calls
                let assistant_msg = Message {
                    role: Role::Assistant,
                    content: completion.content.clone(),
                    tool_calls: completion.tool_calls.clone(),
                };
                self.messages.push(assistant_msg);

                // Execute each tool call
                let tool_calls = completion.tool_calls.unwrap();
                for tc in &tool_calls {
                    let tool_name = &tc.function.name;
                    let args: serde_json::Value =
                        serde_json::from_str(&tc.function.arguments).unwrap_or_default();

                    info!(
                        session = %self.session_id,
                        tool = %tool_name,
                        turn = turn_count + 1,
                        "Executing tool call"
                    );

                    let result = match self.tool_registry.invoke_tool(ctx, tool_name, args).await {
                        Ok(v) => v.to_string(),
                        Err(e) => {
                            warn!(tool = %tool_name, error = %e, "Tool execution failed");
                            format!("Error: {e}")
                        }
                    };

                    self.turns.push(AgentTurn {
                        turn: turn_count + 1,
                        action: format!("call {tool_name}"),
                        result: result.clone(),
                    });

                    // Add tool result message
                    self.messages.push(Message {
                        role: Role::Assistant,
                        content: String::new(),
                        tool_calls: Some(vec![ToolCall {
                            id: tc.id.clone(),
                            call_type: "function".into(),
                            function: crate::llm::FunctionCall {
                                name: tool_name.clone(),
                                arguments: tc.function.arguments.clone(),
                            },
                        }]),
                    });
                    self.messages.push(Message {
                        role: Role::User,
                        content: result,
                        tool_calls: None,
                    });
                }

                turn_count += 1;
                self.truncate_if_needed();
            } else {
                // No tool calls — final text response
                self.messages.push(Message::assistant(&completion.content));
                return Ok(AgentResponse {
                    session_id: self.session_id.clone(),
                    content: completion.content,
                    turns: self.turns.clone(),
                    truncated,
                });
            }
        }
    }

    /// Estimate total token count of conversation history.
    fn estimate_tokens(&self) -> usize {
        self.messages
            .iter()
            .map(|m| m.content.len() / CHARS_PER_TOKEN + 4) // +4 for role/metadata overhead
            .sum()
    }

    /// Truncate oldest messages (after system prompt) when approaching token limit.
    ///
    /// Preserves tool_call/tool_result atomicity: if the oldest message is an
    /// Assistant with tool_calls, we also remove the following User tool_result
    /// to avoid orphaned results that would confuse the LLM.
    fn truncate_if_needed(&mut self) {
        let estimated = self.estimate_tokens();
        if estimated <= self.config.max_tokens {
            return;
        }

        let before = self.messages.len();
        // Always keep the system prompt (index 0).
        while self.estimate_tokens() > self.config.max_tokens && self.messages.len() > 2 {
            // messages[1] is the oldest non-system message
            if self.messages[1].role == Role::Assistant && self.messages[1].tool_calls.is_some() {
                // This is a tool_call Assistant — remove it AND the next tool_result User
                self.messages.remove(1); // Assistant with tool_calls
                if self.messages.len() > 1 && self.messages[1].role == Role::User {
                    self.messages.remove(1); // User tool_result
                }
            } else {
                // Normal message — safe to remove
                self.messages.remove(1);
            }
        }

        let removed = before - self.messages.len();
        if removed > 0 {
            info!(
                session = %self.session_id,
                removed,
                remaining = self.messages.len(),
                "Truncated conversation history"
            );
        }
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn config(&self) -> &AgentConfig {
        &self.config
    }
}

// ── Agent Orchestrator ──────────────────────────────────────────────────────

/// Manages multiple agent sessions.
pub struct AgentOrchestrator {
    sessions: RwLock<HashMap<String, Arc<RwLock<AgentSession>>>>,
    tool_registry: Arc<ToolRegistry>,
    llm_client: Arc<dyn LlmClient>,
    default_config: AgentConfig,
}

impl AgentOrchestrator {
    pub fn new(
        tool_registry: Arc<ToolRegistry>,
        llm_client: Arc<dyn LlmClient>,
        default_config: AgentConfig,
    ) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            tool_registry,
            llm_client,
            default_config,
        }
    }

    /// Create a new session and return its ID.
    pub async fn create_session(&self) -> String {
        let session = AgentSession::new(
            Arc::clone(&self.tool_registry),
            Arc::clone(&self.llm_client),
            self.default_config.clone(),
        );
        let id = session.session_id.clone();
        let mut sessions = self.sessions.write().await;
        sessions.insert(id.clone(), Arc::new(RwLock::new(session)));
        info!(session = %id, "Created new agent session");
        id
    }

    /// Send a message to a session and get the agent's response.
    pub async fn chat(
        &self,
        session_id: &str,
        message: &str,
        ctx: &ops_pilot_sdk::context::ModuleContext,
    ) -> Result<AgentResponse> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| anyhow!("session '{}' not found", session_id))?;
        let session = Arc::clone(session);
        drop(sessions);

        let mut session = session.write().await;
        session.chat(message, ctx).await
    }

    /// Close and remove a session.
    pub async fn close_session(&self, session_id: &str) -> bool {
        let mut sessions = self.sessions.write().await;
        let removed = sessions.remove(session_id).is_some();
        if removed {
            info!(session = %session_id, "Closed agent session");
        }
        removed
    }

    /// Get the number of active sessions.
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use ops_pilot_sdk::context::{EventBus, ModuleContext};
    use ops_pilot_sdk::events::OpsEvent;
    use ops_pilot_sdk::loader::ModuleLoader;
    use ops_pilot_sdk::traits::{HealthStatus, ModuleAction, OpsModule, ToolDefinition};
    use serde_json::json;
    use sqlx::SqlitePool;
    use std::path::PathBuf;
    use std::pin::Pin;

    use crate::llm::{CompletionResponse, LlmError};

    // ── Mock LLM Client ────────────────────────────────────────────────

    /// Mock that returns pre-configured responses in sequence.
    struct MockLlmClient {
        responses: std::sync::Mutex<Vec<CompletionResponse>>,
    }

    impl MockLlmClient {
        fn single(text: &str) -> Self {
            Self {
                responses: std::sync::Mutex::new(vec![CompletionResponse::text(text)]),
            }
        }

        fn with_tool_call(tool_name: &str, args: &str, final_response: &str) -> Self {
            Self {
                responses: std::sync::Mutex::new(vec![
                    CompletionResponse {
                        content: String::new(),
                        tool_calls: Some(vec![ToolCall {
                            id: "call_1".into(),
                            call_type: "function".into(),
                            function: crate::llm::FunctionCall {
                                name: tool_name.into(),
                                arguments: args.into(),
                            },
                        }]),
                    },
                    CompletionResponse::text(final_response),
                ]),
            }
        }

        fn sequence(responses: Vec<CompletionResponse>) -> Self {
            Self {
                responses: std::sync::Mutex::new(responses),
            }
        }
    }

    #[async_trait]
    impl LlmClient for MockLlmClient {
        async fn complete(&self, _messages: &[Message]) -> Result<String, LlmError> {
            let mut responses = self.responses.lock().unwrap();
            if responses.is_empty() {
                return Ok("default response".into());
            }
            Ok(responses.remove(0).content)
        }

        async fn complete_with_tools(
            &self,
            _messages: &[Message],
            _tools: &[serde_json::Value],
        ) -> Result<CompletionResponse, LlmError> {
            let mut responses = self.responses.lock().unwrap();
            if responses.is_empty() {
                return Ok(CompletionResponse::text("default response"));
            }
            Ok(responses.remove(0))
        }

        async fn complete_stream(
            &self,
            _messages: &[Message],
        ) -> Result<Pin<Box<dyn futures_util::Stream<Item = Result<String, LlmError>> + Send>>, LlmError>
        {
            Ok(Box::pin(futures_util::stream::empty()))
        }
    }

    // ── Test Helpers ────────────────────────────────────────────────────

    async fn make_ctx(name: &str) -> ModuleContext {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        ModuleContext::new(
            Arc::new(pool),
            EventBus::new(16),
            PathBuf::from(format!("/tmp/{name}")),
            name.to_string(),
        )
    }

    struct StubModule {
        name: String,
    }

    #[async_trait]
    impl OpsModule for StubModule {
        fn name(&self) -> &str {
            &self.name
        }
        fn version(&self) -> &str {
            "0.1.0"
        }
        fn description(&self) -> &str {
            "stub"
        }
        fn dependencies(&self) -> Vec<&str> {
            vec![]
        }
        fn tools(&self) -> Vec<ToolDefinition> {
            vec![ToolDefinition {
                name: "get_server_status".into(),
                description: "Get server status".into(),
                parameters: json!({"type": "object", "properties": {}}),
            }]
        }
        async fn execute(&self, _ctx: &ModuleContext, _tool: &str, _p: serde_json::Value) -> Result<serde_json::Value> {
            Ok(json!({"status": "healthy", "cpu": 45.2}))
        }
        async fn on_event(&self, _ctx: &ModuleContext, _e: &OpsEvent) -> Option<ModuleAction> {
            None
        }
        async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus {
            HealthStatus::Healthy
        }
    }

    async fn build_orchestrator(llm: Arc<dyn LlmClient>) -> AgentOrchestrator {
        let mut loader = ModuleLoader::new();
        let ctx = make_ctx("stub").await;
        loader
            .load_module(ctx, Box::new(StubModule { name: "stub".into() }))
            .await
            .unwrap();
        let manager = std::sync::Arc::new(tokio::sync::RwLock::new(
            crate::routes::modules::ModuleManager::new(loader),
        ));
        let registry = Arc::new(ToolRegistry::new(manager));
        AgentOrchestrator::new(registry, llm, AgentConfig::default())
    }

    // ── Tests ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_simple_text_response() {
        let llm = Arc::new(MockLlmClient::single("The server is running fine."));
        let orchestrator = build_orchestrator(llm).await;

        let session_id = orchestrator.create_session().await;
        let ctx = make_ctx("test").await;

        let resp = orchestrator.chat(&session_id, "How is the server?", &ctx).await.unwrap();
        assert_eq!(resp.content, "The server is running fine.");
        assert!(resp.turns.is_empty());
    }

    #[tokio::test]
    async fn test_tool_call_and_response() {
        let llm = Arc::new(MockLlmClient::with_tool_call(
            "get_server_status",
            "{}",
            "The server status is healthy with 45.2% CPU usage.",
        ));
        let orchestrator = build_orchestrator(llm).await;

        let session_id = orchestrator.create_session().await;
        let ctx = make_ctx("test").await;

        let resp = orchestrator
            .chat(&session_id, "Check server status", &ctx)
            .await
            .unwrap();

        assert_eq!(
            resp.content,
            "The server status is healthy with 45.2% CPU usage."
        );
        assert_eq!(resp.turns.len(), 1);
        assert!(resp.turns[0].action.contains("get_server_status"));
        assert!(resp.turns[0].result.contains("healthy"));
    }

    #[tokio::test]
    async fn test_max_turns_limit() {
        // Create a mock that always wants to call tools (never gives final response)
        let llm = Arc::new(MockLlmClient::sequence(vec![
            CompletionResponse {
                content: String::new(),
                tool_calls: Some(vec![ToolCall {
                    id: "c1".into(),
                    call_type: "function".into(),
                    function: crate::llm::FunctionCall {
                        name: "get_server_status".into(),
                        arguments: "{}".into(),
                    },
                }]),
            },
            CompletionResponse {
                content: String::new(),
                tool_calls: Some(vec![ToolCall {
                    id: "c2".into(),
                    call_type: "function".into(),
                    function: crate::llm::FunctionCall {
                        name: "get_server_status".into(),
                        arguments: "{}".into(),
                    },
                }]),
            },
            CompletionResponse::text("I checked multiple times. All good."),
        ]));

        let config = AgentConfig {
            max_turns: 2,
            ..Default::default()
        };

        let mut loader = ModuleLoader::new();
        let ctx = make_ctx("stub").await;
        loader
            .load_module(ctx, Box::new(StubModule { name: "stub".into() }))
            .await
            .unwrap();
        let manager = Arc::new(tokio::sync::RwLock::new(
            crate::routes::modules::ModuleManager::new(loader),
        ));
        let registry = Arc::new(ToolRegistry::new(manager));
        let orchestrator = AgentOrchestrator::new(registry, llm, config);

        let session_id = orchestrator.create_session().await;
        let ctx = make_ctx("test").await;

        let resp = orchestrator
            .chat(&session_id, "Check everything", &ctx)
            .await
            .unwrap();

        // Should stop at max_turns and return whatever the LLM says
        assert!(!resp.turns.is_empty());
    }

    #[tokio::test]
    async fn test_session_not_found() {
        let llm = Arc::new(MockLlmClient::single("hi"));
        let orchestrator = build_orchestrator(llm).await;
        let ctx = make_ctx("test").await;

        let result = orchestrator.chat("nonexistent", "hello", &ctx).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_close_session() {
        let llm = Arc::new(MockLlmClient::single("ok"));
        let orchestrator = build_orchestrator(llm).await;

        let id = orchestrator.create_session().await;
        assert_eq!(orchestrator.session_count().await, 1);

        assert!(orchestrator.close_session(&id).await);
        assert_eq!(orchestrator.session_count().await, 0);

        // Closing again returns false
        assert!(!orchestrator.close_session(&id).await);
    }

    #[tokio::test]
    async fn test_token_estimation() {
        let llm = Arc::new(MockLlmClient::single("ok"));
        let orchestrator = build_orchestrator(llm).await;
        let session_id = orchestrator.create_session().await;
        let ctx = make_ctx("test").await;

        let sessions = orchestrator.sessions.read().await;
        let session_arc = sessions.get(&session_id).unwrap().clone();
        drop(sessions);

        let mut session = session_arc.write().await;
        session.chat("Hello, how are you?", &ctx).await.unwrap();

        let tokens = session.estimate_tokens();
        assert!(tokens > 0, "Should estimate some tokens");
    }

    #[tokio::test]
    async fn test_truncation() {
        let llm = Arc::new(MockLlmClient::sequence(vec![
            CompletionResponse::text("Response 1"),
            CompletionResponse::text("Response 2"),
            CompletionResponse::text("Response 3"),
            CompletionResponse::text("Final"),
        ]));

        let config = AgentConfig {
            max_tokens: 50, // Very small to trigger truncation
            ..Default::default()
        };

        let mut loader = ModuleLoader::new();
        let ctx = make_ctx("stub").await;
        loader
            .load_module(ctx, Box::new(StubModule { name: "stub".into() }))
            .await
            .unwrap();
        let manager = Arc::new(tokio::sync::RwLock::new(
            crate::routes::modules::ModuleManager::new(loader),
        ));
        let registry = Arc::new(ToolRegistry::new(manager));
        let orchestrator = AgentOrchestrator::new(registry, llm, config);

        let session_id = orchestrator.create_session().await;
        let ctx = make_ctx("test").await;

        // Send multiple messages to build up history
        orchestrator.chat(&session_id, "First message with some content", &ctx).await.unwrap();
        orchestrator.chat(&session_id, "Second message with more content", &ctx).await.unwrap();

        let sessions = orchestrator.sessions.read().await;
        let session_arc = sessions.get(&session_id).unwrap().clone();
        drop(sessions);

        let session = session_arc.read().await;
        // System prompt + latest user + latest assistant should be kept
        assert!(session.messages().len() <= 4, "Messages should be truncated");
    }

    #[tokio::test]
    async fn test_completion_response_types() {
        let text_only = CompletionResponse::text("hello");
        assert!(!text_only.has_tool_calls());
        assert_eq!(text_only.content, "hello");

        let with_calls = CompletionResponse {
            content: "thinking...".into(),
            tool_calls: Some(vec![ToolCall {
                id: "c1".into(),
                call_type: "function".into(),
                function: crate::llm::FunctionCall {
                    name: "test".into(),
                    arguments: "{}".into(),
                },
            }]),
        };
        assert!(with_calls.has_tool_calls());
    }

    #[tokio::test]
    async fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert!(config.system_prompt.contains("OpsPilot"));
        assert_eq!(config.max_tokens, 8_000);
        assert_eq!(config.max_turns, 10);
    }

    #[tokio::test]
    async fn test_multi_turn_conversation() {
        let llm = Arc::new(MockLlmClient::sequence(vec![
            CompletionResponse::text("I'll help you with that."),
            CompletionResponse::text("Is there anything else you need?"),
        ]));
        let orchestrator = build_orchestrator(llm).await;

        let session_id = orchestrator.create_session().await;
        let ctx = make_ctx("test").await;

        let resp1 = orchestrator.chat(&session_id, "Hello", &ctx).await.unwrap();
        assert_eq!(resp1.content, "I'll help you with that.");

        let resp2 = orchestrator.chat(&session_id, "Thanks!", &ctx).await.unwrap();
        assert_eq!(resp2.content, "Is there anything else you need?");

        // Session should have accumulated messages
        let sessions = orchestrator.sessions.read().await;
        let session = sessions.get(&session_id).unwrap().read().await;
        // system + user1 + assistant1 + user2 + assistant2
        assert_eq!(session.messages().len(), 5);
    }

    #[tokio::test]
    async fn test_tool_error_handling() {
        // Tool that doesn't exist should return error message, not crash
        let llm = Arc::new(MockLlmClient::sequence(vec![
            CompletionResponse {
                content: String::new(),
                tool_calls: Some(vec![ToolCall {
                    id: "c1".into(),
                    call_type: "function".into(),
                    function: crate::llm::FunctionCall {
                        name: "nonexistent_tool".into(),
                        arguments: "{}".into(),
                    },
                }]),
            },
            CompletionResponse::text("The tool isn't available right now."),
        ]));
        let orchestrator = build_orchestrator(llm).await;

        let session_id = orchestrator.create_session().await;
        let ctx = make_ctx("test").await;

        let resp = orchestrator
            .chat(&session_id, "Do something impossible", &ctx)
            .await
            .unwrap();

        assert_eq!(resp.content, "The tool isn't available right now.");
        assert_eq!(resp.turns.len(), 1);
        assert!(resp.turns[0].result.contains("Error"));
    }
}
