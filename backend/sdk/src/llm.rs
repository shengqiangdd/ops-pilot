//! Core LLM types shared between gateway and modules.
//!
//! This module defines the [`LlmClient`] trait, error types, and message
//! structures so that modules can depend on the SDK instead of the gateway,
//! breaking the circular dependency cycle.

use async_trait::async_trait;
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

/// Errors from LLM operations.
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },

    #[error("stream ended unexpectedly")]
    StreamClosed,

    #[error("{0}")]
    Other(String),
}

/// Message roles in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// A single chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            tool_calls: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            tool_calls: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            tool_calls: None,
        }
    }
}

/// A tool/function call requested by the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

/// Details of a function call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// A structured completion response that may include tool calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// The assistant's text content (may be empty if only tool calls).
    pub content: String,
    /// Tool calls requested by the model, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl CompletionResponse {
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            tool_calls: None,
        }
    }

    pub fn has_tool_calls(&self) -> bool {
        self.tool_calls.as_ref().is_some_and(|tc| !tc.is_empty())
    }
}

/// Trait for OpenAI-compatible LLM backends.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Send a non-streaming chat completion request (text only).
    async fn complete(&self, messages: &[Message]) -> Result<String, LlmError>;

    /// Send a non-streaming completion with tool support.
    ///
    /// Default implementation calls `complete()` and returns text-only.
    /// Override this to return structured tool calls from the API.
    async fn complete_with_tools(
        &self,
        messages: &[Message],
        _tools: &[serde_json::Value],
    ) -> Result<CompletionResponse, LlmError> {
        let text = self.complete(messages).await?;
        Ok(CompletionResponse::text(text))
    }

    /// Send a streaming chat completion request.
    async fn complete_stream(
        &self,
        messages: &[Message],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>>, LlmError>;
}
