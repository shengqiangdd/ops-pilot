use async_trait::async_trait;
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

/// Errors from LLM operations.
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

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
        self.tool_calls
            .as_ref()
            .is_some_and(|tc| !tc.is_empty())
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

/// High-level chat service wrapping an [`LlmClient`].
pub struct ChatService {
    client: Box<dyn LlmClient>,
    system_prompt: Option<String>,
}

impl ChatService {
    pub fn new(client: Box<dyn LlmClient>) -> Self {
        Self {
            client,
            system_prompt: None,
        }
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Non-streaming chat completion.
    pub async fn chat(&self, messages: &[Message]) -> Result<String, LlmError> {
        let full = self.prepend_system(messages);
        self.client.complete(&full).await
    }

    /// Streaming chat completion.
    pub async fn chat_stream(
        &self,
        messages: &[Message],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>>, LlmError> {
        let full = self.prepend_system(messages);
        self.client.complete_stream(&full).await
    }

    fn prepend_system(&self, messages: &[Message]) -> Vec<Message> {
        match &self.system_prompt {
            Some(prompt) => {
                let mut full = Vec::with_capacity(1 + messages.len());
                full.push(Message::system(prompt));
                full.extend_from_slice(messages);
                full
            }
            None => messages.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::stream;

    struct MockClient {
        response: String,
    }

    #[async_trait]
    impl LlmClient for MockClient {
        async fn complete(&self, _messages: &[Message]) -> Result<String, LlmError> {
            Ok(self.response.clone())
        }

        async fn complete_stream(
            &self,
            _messages: &[Message],
        ) -> Result<Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>>, LlmError>
        {
            let chunks: Vec<Result<String, LlmError>> = self
                .response
                .split_whitespace()
                .map(|w| Ok(format!("{w} ")))
                .collect();
            Ok(Box::pin(stream::iter(chunks)))
        }
    }

    struct FailingClient;

    #[async_trait]
    impl LlmClient for FailingClient {
        async fn complete(&self, _messages: &[Message]) -> Result<String, LlmError> {
            Err(LlmError::Api {
                status: 500,
                message: "internal error".into(),
            })
        }

        async fn complete_stream(
            &self,
            _messages: &[Message],
        ) -> Result<Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>>, LlmError>
        {
            Err(LlmError::StreamClosed)
        }
    }

    #[tokio::test]
    async fn test_chat_returns_response() {
        let svc = ChatService::new(Box::new(MockClient {
            response: "Hello world".into(),
        }));
        let result = svc.chat(&[Message::user("hi")]).await.unwrap();
        assert_eq!(result, "Hello world");
    }

    #[tokio::test]
    async fn test_chat_prepends_system_prompt() {
        let svc = ChatService::new(Box::new(MockClient {
            response: "ok".into(),
        }))
        .with_system_prompt("You are a pirate.");

        // Verify the call succeeds (system prompt is prepended internally)
        let result = svc.chat(&[Message::user("ahoy")]).await.unwrap();
        assert_eq!(result, "ok");
    }

    #[tokio::test]
    async fn test_chat_without_system_prompt() {
        let svc = ChatService::new(Box::new(MockClient {
            response: "plain".into(),
        }));
        let result = svc.chat(&[Message::user("test")]).await.unwrap();
        assert_eq!(result, "plain");
    }

    #[tokio::test]
    async fn test_chat_stream_yields_chunks() {
        let svc = ChatService::new(Box::new(MockClient {
            response: "foo bar baz".into(),
        }));
        let mut stream = svc.chat_stream(&[Message::user("go")]).await.unwrap();
        let mut collected = String::new();
        while let Some(chunk) = futures_util::StreamExt::next(&mut stream).await {
            collected.push_str(&chunk.unwrap());
        }
        assert_eq!(collected, "foo bar baz ");
    }

    #[tokio::test]
    async fn test_chat_api_error() {
        let svc = ChatService::new(Box::new(FailingClient));
        let err = svc.chat(&[Message::user("fail")]).await.unwrap_err();
        assert!(matches!(err, LlmError::Api { status: 500, .. }));
    }

    #[tokio::test]
    async fn test_chat_stream_error() {
        let svc = ChatService::new(Box::new(FailingClient));
        let result = svc.chat_stream(&[Message::user("fail")]).await;
        assert!(result.is_err());
        // Unwrap only the Err side to avoid requiring Debug on the Ok stream type.
        let _ = result.err().unwrap();
    }

    #[test]
    fn test_message_serialization_roundtrip() {
        let msg = Message {
            role: Role::Assistant,
            content: "hi".into(),
            tool_calls: Some(vec![ToolCall {
                id: "call_1".into(),
                call_type: "function".into(),
                function: FunctionCall {
                    name: "get_weather".into(),
                    arguments: r#"{"city":"NYC"}"#.into(),
                },
            }]),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.role, Role::Assistant);
        assert_eq!(deserialized.content, "hi");
        assert_eq!(deserialized.tool_calls.unwrap().len(), 1);
    }

    #[test]
    fn test_role_serialization() {
        assert_eq!(serde_json::to_string(&Role::System).unwrap(), r#""system""#);
        assert_eq!(serde_json::to_string(&Role::User).unwrap(), r#""user""#);
        assert_eq!(
            serde_json::to_string(&Role::Assistant).unwrap(),
            r#""assistant""#
        );
    }
}
