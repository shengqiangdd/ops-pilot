//! OpenAI-compatible LLM provider implementation.
//!
//! Supports OpenAI, DeepSeek, OpenRouter, and any other provider
//! that implements the OpenAI chat completions API format.
//!
//! Configuration via environment variables:
//! - `LLM_PROVIDER` — Provider name (for display only)
//! - `LLM_BASE_URL` — API base URL (e.g., `https://api.openai.com/v1`)
//! - `LLM_API_KEY` — API key
//! - `LLM_MODEL` — Model name (e.g., `gpt-4o`, `deepseek-chat`)

use async_trait::async_trait;
use futures_util::Stream;
use reqwest::Client;
use serde::Deserialize;
use std::pin::Pin;
use std::time::Duration;
use tracing::{debug, warn};

use super::{CompletionResponse, LlmClient, LlmError, Message, Role};

/// Configuration for the OpenAI-compatible provider.
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub provider_name: String,
}

impl ProviderConfig {
    /// Load configuration from environment variables.
    pub fn from_env() -> Result<Self, LlmError> {
        let base_url = std::env::var("LLM_BASE_URL")
            .map_err(|_| LlmError::Other("LLM_BASE_URL not set".into()))?;
        let api_key = std::env::var("LLM_API_KEY")
            .map_err(|_| LlmError::Other("LLM_API_KEY not set".into()))?;
        let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o".into());
        let provider_name =
            std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "openai".into());

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            model,
            provider_name,
        })
    }
}

/// OpenAI-compatible LLM client.
pub struct ProviderLlm {
    config: ProviderConfig,
    client: Client,
}

impl ProviderLlm {
    pub fn new(config: ProviderConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("failed to build HTTP client");
        Self { config, client }
    }

    fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.config.base_url)
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.config.api_key)
    }

    /// Build the OpenAI-format request body.
    fn build_request_body(
        &self,
        messages: &[Message],
        tools: Option<&[serde_json::Value]>,
        stream: bool,
    ) -> serde_json::Value {
        let messages_json: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                let mut obj = serde_json::json!({
                    "role": match m.role {
                        Role::System => "system",
                        Role::User => "user",
                        Role::Assistant => "assistant",
                    },
                    "content": m.content,
                });
                if let Some(ref tool_calls) = m.tool_calls {
                    obj["tool_calls"] = serde_json::to_value(tool_calls).unwrap_or_default();
                }
                obj
            })
            .collect();

        let mut body = serde_json::json!({
            "model": self.config.model,
            "messages": messages_json,
            "stream": stream,
        });

        if let Some(tools) = tools {
            if !tools.is_empty() {
                body["tools"] = serde_json::to_value(tools).unwrap_or_default();
            }
        }

        body
    }

    /// Send a request and return the raw response (or error).
    async fn send_request(
        &self,
        body: &serde_json::Value,
    ) -> Result<reqwest::Response, LlmError> {
        let resp = self
            .client
            .post(self.chat_url())
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| LlmError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            warn!(
                provider = %self.config.provider_name,
                status,
                body = %text,
                "LLM API error"
            );
            return Err(LlmError::Api {
                status,
                message: text,
            });
        }

        Ok(resp)
    }
}

// ── Response types ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<super::ToolCall>>,
}

#[derive(Debug, Deserialize)]
struct StreamChunkResponse {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: Option<StreamDelta>,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    content: Option<String>,
}

// ── LlmClient implementation ─────────────────────────────────────────────

#[async_trait]
impl LlmClient for ProviderLlm {
    async fn complete(&self, messages: &[Message]) -> Result<String, LlmError> {
        let body = self.build_request_body(messages, None, false);
        debug!(
            provider = %self.config.provider_name,
            model = %self.config.model,
            messages = messages.len(),
            "LLM complete request"
        );

        let resp = self.send_request(&body).await?;
        let completion: ChatCompletionResponse = resp.json().await
            .map_err(|e| LlmError::Http(e.to_string()))?;

        let content = completion
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(content)
    }

    async fn complete_with_tools(
        &self,
        messages: &[Message],
        tools: &[serde_json::Value],
    ) -> Result<CompletionResponse, LlmError> {
        let body = self.build_request_body(messages, Some(tools), false);
        debug!(
            provider = %self.config.provider_name,
            model = %self.config.model,
            messages = messages.len(),
            tools = tools.len(),
            "LLM complete_with_tools request"
        );

        let resp = self.send_request(&body).await?;
        let completion: ChatCompletionResponse = resp.json().await
            .map_err(|e| LlmError::Http(e.to_string()))?;

        let choice = completion
            .choices
            .first()
            .ok_or_else(|| LlmError::Other("no choices in response".into()))?;

        let content = choice.message.content.clone().unwrap_or_default();
        let tool_calls = choice.message.tool_calls.clone();

        Ok(CompletionResponse { content, tool_calls })
    }

    async fn complete_stream(
        &self,
        messages: &[Message],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>>, LlmError>
    {
        let body = self.build_request_body(messages, None, true);
        debug!(
            provider = %self.config.provider_name,
            model = %self.config.model,
            messages = messages.len(),
            "LLM complete_stream request"
        );

        let resp = self.send_request(&body).await?;
        let byte_stream = resp.bytes_stream();

        let mapped = futures_util::stream::unfold(
            (byte_stream, Vec::new()),
            |(mut stream, mut buffer)| async move {
                use futures_util::StreamExt;
                loop {
                    // Process any complete lines already in the buffer
                    if let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                        let line: Vec<u8> = buffer.drain(..=newline_pos).collect();
                        let line_str = String::from_utf8_lossy(&line);
                        let trimmed = line_str.trim();

                        if trimmed.is_empty() {
                            continue;
                        }
                        if trimmed == "data: [DONE]" {
                            return None;
                        }
                        if let Some(data) = trimmed.strip_prefix("data: ") {
                            match serde_json::from_str::<StreamChunkResponse>(data) {
                                Ok(chunk) => {
                                    if let Some(choice) = chunk.choices.first() {
                                        if let Some(ref delta) = choice.delta {
                                            if let Some(ref content) = delta.content {
                                                if !content.is_empty() {
                                                    return Some((
                                                        Ok(content.clone()),
                                                        (stream, buffer),
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to parse SSE chunk: {}", e);
                                }
                            }
                        }
                        continue;
                    }

                    // Read more bytes from the stream
                    match stream.next().await {
                        Some(Ok(bytes)) => {
                            buffer.extend_from_slice(&bytes);
                        }
                        Some(Err(e)) => {
                            return Some((Err(LlmError::Http(e.to_string())), (stream, buffer)));
                        }
                        None => {
                            if !buffer.is_empty() {
                                let remaining = String::from_utf8_lossy(&buffer);
                                if !remaining.trim().is_empty() {
                                    warn!("Stream ended with unparsed data: {}", remaining);
                                }
                            }
                            return None;
                        }
                    }
                }
            },
        );

        Ok(Box::pin(mapped))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_config(base_url: &str) -> ProviderConfig {
        ProviderConfig {
            base_url: base_url.to_string(),
            api_key: "test-key".into(),
            model: "gpt-4o".into(),
            provider_name: "test".into(),
        }
    }

    #[test]
    fn test_provider_config_from_env() {
        std::env::set_var("LLM_BASE_URL", "https://api.example.com/v1");
        std::env::set_var("LLM_API_KEY", "sk-test123");
        std::env::set_var("LLM_MODEL", "deepseek-chat");
        std::env::set_var("LLM_PROVIDER", "deepseek");

        let config = ProviderConfig::from_env().unwrap();
        assert_eq!(config.base_url, "https://api.example.com/v1");
        assert_eq!(config.api_key, "sk-test123");
        assert_eq!(config.model, "deepseek-chat");
        assert_eq!(config.provider_name, "deepseek");

        std::env::remove_var("LLM_BASE_URL");
        std::env::remove_var("LLM_API_KEY");
        std::env::remove_var("LLM_MODEL");
        std::env::remove_var("LLM_PROVIDER");
    }

    #[test]
    fn test_provider_config_missing_url() {
        std::env::remove_var("LLM_BASE_URL");
        std::env::set_var("LLM_API_KEY", "sk-test");
        assert!(ProviderConfig::from_env().is_err());
        std::env::remove_var("LLM_API_KEY");
    }

    #[test]
    fn test_provider_config_missing_key() {
        std::env::set_var("LLM_BASE_URL", "https://api.example.com");
        std::env::remove_var("LLM_API_KEY");
        assert!(ProviderConfig::from_env().is_err());
        std::env::remove_var("LLM_BASE_URL");
    }

    #[test]
    fn test_provider_config_defaults() {
        std::env::set_var("LLM_BASE_URL", "https://api.example.com/v1");
        std::env::set_var("LLM_API_KEY", "sk-test");

        let config = ProviderConfig::from_env().unwrap();
        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.provider_name, "openai");

        std::env::remove_var("LLM_BASE_URL");
        std::env::remove_var("LLM_API_KEY");
    }

    #[test]
    fn test_base_url_trailing_slash_trimmed() {
        std::env::set_var("LLM_BASE_URL", "https://api.example.com/v1/");
        std::env::set_var("LLM_API_KEY", "key");
        let config = ProviderConfig::from_env().unwrap();
        assert_eq!(config.base_url, "https://api.example.com/v1");
        std::env::remove_var("LLM_BASE_URL");
        std::env::remove_var("LLM_API_KEY");
    }

    #[test]
    fn test_chat_url() {
        let client = ProviderLlm::new(test_config("http://localhost"));
        assert_eq!(
            client.chat_url(),
            "http://localhost/chat/completions"
        );
    }

    #[test]
    fn test_auth_header() {
        let client = ProviderLlm::new(ProviderConfig {
            base_url: "http://localhost".into(),
            api_key: "sk-secret".into(),
            model: "gpt-4o".into(),
            provider_name: "test".into(),
        });
        assert_eq!(client.auth_header(), "Bearer sk-secret");
    }

    #[test]
    fn test_build_request_body_basic() {
        let client = ProviderLlm::new(test_config("http://localhost"));
        let body = client.build_request_body(&[Message::user("hi")], None, false);

        assert_eq!(body["model"], "gpt-4o");
        assert_eq!(body["stream"], false);
        let msgs = body["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0]["role"], "user");
        assert_eq!(msgs[0]["content"], "hi");
    }

    #[test]
    fn test_build_request_body_with_tools() {
        let client = ProviderLlm::new(test_config("http://localhost"));
        let tools = vec![serde_json::json!({
            "type": "function",
            "function": {"name": "test"}
        })];
        let body = client.build_request_body(&[Message::user("hi")], Some(&tools), false);
        assert!(body.get("tools").is_some());
        assert_eq!(body["tools"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_build_request_body_empty_tools_ignored() {
        let client = ProviderLlm::new(test_config("http://localhost"));
        let body = client.build_request_body(&[Message::user("hi")], Some(&[]), false);
        assert!(body.get("tools").is_none());
    }

    #[test]
    fn test_build_request_body_stream() {
        let client = ProviderLlm::new(test_config("http://localhost"));
        let body = client.build_request_body(&[Message::user("hi")], None, true);
        assert_eq!(body["stream"], true);
    }

    #[test]
    fn test_build_request_body_system_message() {
        let client = ProviderLlm::new(test_config("http://localhost"));
        let body = client.build_request_body(
            &[
                Message::system("You are helpful."),
                Message::user("Hello"),
            ],
            None,
            false,
        );
        let msgs = body["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0]["role"], "system");
        assert_eq!(msgs[1]["role"], "user");
    }

    #[test]
    fn test_build_request_body_tool_calls_in_assistant() {
        let client = ProviderLlm::new(test_config("http://localhost"));
        let msg = Message {
            role: Role::Assistant,
            content: "Let me check".into(),
            tool_calls: Some(vec![super::super::ToolCall {
                id: "call_1".into(),
                call_type: "function".into(),
                function: super::super::FunctionCall {
                    name: "get_weather".into(),
                    arguments: r#"{"city":"NYC"}"#.into(),
                },
            }]),
        };
        let body = client.build_request_body(&[msg], None, false);
        let msgs = body["messages"].as_array().unwrap();
        assert!(msgs[0].get("tool_calls").is_some());
    }

    // ── Integration tests with wiremock ──────────────────────────────────

    #[tokio::test]
    async fn test_complete_text_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": "Hello, world!"
                    }
                }]
            })))
            .mount(&server)
            .await;

        let client = ProviderLlm::new(test_config(&server.uri()));
        let result = client
            .complete(&[Message::user("Say hello")])
            .await
            .unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[tokio::test]
    async fn test_complete_with_tools_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": null,
                        "tool_calls": [{
                            "id": "call_abc123",
                            "type": "function",
                            "function": {
                                "name": "get_weather",
                                "arguments": "{\"city\":\"NYC\"}"
                            }
                        }]
                    }
                }]
            })))
            .mount(&server)
            .await;

        let client = ProviderLlm::new(test_config(&server.uri()));
        let tools = vec![serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Get weather",
                "parameters": {"type": "object", "properties": {"city": {"type": "string"}}}
            }
        })];

        let result = client
            .complete_with_tools(&[Message::user("Weather in NYC?")], &tools)
            .await
            .unwrap();

        assert!(result.has_tool_calls());
        let tc = result.tool_calls.unwrap();
        assert_eq!(tc.len(), 1);
        assert_eq!(tc[0].function.name, "get_weather");
        assert_eq!(tc[0].function.arguments, r#"{"city":"NYC"}"#);
    }

    #[tokio::test]
    async fn test_complete_api_error_401() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(401)
                    .set_body_string(r#"{"error": {"message": "Invalid API key"}}"#),
            )
            .mount(&server)
            .await;

        let client = ProviderLlm::new(test_config(&server.uri()));
        let err = client
            .complete(&[Message::user("test")])
            .await
            .unwrap_err();
        assert!(matches!(err, LlmError::Api { status: 401, .. }));
    }

    #[tokio::test]
    async fn test_complete_api_error_500() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
            .mount(&server)
            .await;

        let client = ProviderLlm::new(test_config(&server.uri()));
        let err = client
            .complete(&[Message::user("test")])
            .await
            .unwrap_err();
        assert!(matches!(err, LlmError::Api { status: 500, .. }));
    }

    #[tokio::test]
    async fn test_complete_empty_choices() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": []
            })))
            .mount(&server)
            .await;

        let client = ProviderLlm::new(test_config(&server.uri()));
        let result = client
            .complete(&[Message::user("test")])
            .await
            .unwrap();
        assert_eq!(result, "");
    }

    #[tokio::test]
    async fn test_stream_sse_response() {
        let server = MockServer::start().await;
        let sse_body = "\
data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n\
data: {\"choices\":[{\"delta\":{\"content\":\" world\"}}]}\n\n\
data: [DONE]\n\n";

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(sse_body)
                    .insert_header("content-type", "text/event-stream"),
            )
            .mount(&server)
            .await;

        let client = ProviderLlm::new(test_config(&server.uri()));
        let mut stream = client
            .complete_stream(&[Message::user("say hi")])
            .await
            .unwrap();

        let mut result = String::new();
        while let Some(chunk) = futures_util::StreamExt::next(&mut stream).await {
            result.push_str(&chunk.unwrap());
        }
        assert_eq!(result, "Hello world");
    }

    #[tokio::test]
    async fn test_stream_single_chunk() {
        let server = MockServer::start().await;
        let sse_body = "data: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}\n\ndata: [DONE]\n\n";

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(sse_body)
                    .insert_header("content-type", "text/event-stream"),
            )
            .mount(&server)
            .await;

        let client = ProviderLlm::new(test_config(&server.uri()));
        let mut stream = client
            .complete_stream(&[Message::user("hi")])
            .await
            .unwrap();

        let mut result = String::new();
        while let Some(chunk) = futures_util::StreamExt::next(&mut stream).await {
            result.push_str(&chunk.unwrap());
        }
        assert_eq!(result, "Hi");
    }
}
