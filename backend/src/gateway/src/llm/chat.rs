//! LLM chat types re-exported from SDK, plus gateway-specific services.

pub use ops_pilot_sdk::llm::{
    CompletionResponse, FunctionCall, LlmClient, LlmError, Message, Role, ToolCall,
};

use async_trait::async_trait;
use futures_util::Stream;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

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

// ── Circuit Breaker ──────────────────────────────────────────────────────

/// State of the circuit breaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
}

/// A simple circuit breaker that tracks consecutive failures.
pub struct CircuitBreaker {
    state: RwLock<CircuitState>,
    failure_count: AtomicU64,
    failure_threshold: u64,
    recovery_timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u64, recovery_timeout: Duration) -> Self {
        Self {
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU64::new(0),
            failure_threshold,
            recovery_timeout,
        }
    }

    /// Check if a request is allowed through.
    pub async fn allow_request(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() >= self.recovery_timeout {
                    drop(state);
                    let mut state = self.state.write().await;
                    *state = CircuitState::HalfOpen;
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Record a successful call.
    pub async fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        let mut state = self.state.write().await;
        *state = CircuitState::Closed;
    }

    /// Record a failed call. Opens the circuit if threshold is reached.
    pub async fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        if count >= self.failure_threshold {
            let mut state = self.state.write().await;
            *state = CircuitState::Open {
                opened_at: Instant::now(),
            };
        }
    }

    /// Get current state snapshot.
    pub async fn current_state(&self) -> CircuitState {
        *self.state.read().await
    }
}

// ── Retry Client ─────────────────────────────────────────────────────────

/// Configuration for the retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
        }
    }
}

/// Wraps an `LlmClient` with exponential backoff retry and circuit breaker.
pub struct RetryClient {
    inner: Box<dyn LlmClient>,
    config: RetryConfig,
    circuit: Arc<CircuitBreaker>,
}

impl RetryClient {
    pub fn new(inner: Box<dyn LlmClient>) -> Self {
        Self {
            inner,
            config: RetryConfig::default(),
            circuit: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
        }
    }

    pub fn with_config(inner: Box<dyn LlmClient>, config: RetryConfig) -> Self {
        Self {
            inner,
            config,
            circuit: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
        }
    }

    fn should_retry(err: &LlmError) -> bool {
        match err {
            // Network errors — retry
            LlmError::Http(_) => true,
            // Stream closed — retry
            LlmError::StreamClosed => true,
            // API errors — only retry 5xx
            LlmError::Api { status, .. } => *status >= 500,
            // JSON errors, other — don't retry
            _ => false,
        }
    }

    fn retry_delay(attempt: u32, base: Duration, max: Duration) -> Duration {
        let delay = base * 2u32.pow(attempt - 1);
        delay.min(max)
    }
}

#[async_trait]
impl LlmClient for RetryClient {
    async fn complete(&self, messages: &[Message]) -> Result<String, LlmError> {
        if !self.circuit.allow_request().await {
            return Err(LlmError::Other("circuit breaker is open".into()));
        }

        let mut last_err = None;
        for attempt in 1..=self.config.max_retries {
            match self.inner.complete(messages).await {
                Ok(result) => {
                    self.circuit.record_success().await;
                    return Ok(result);
                }
                Err(e) => {
                    if attempt < self.config.max_retries && Self::should_retry(&e) {
                        let delay =
                            Self::retry_delay(attempt, self.config.base_delay, self.config.max_delay);
                        tokio::time::sleep(delay).await;
                        last_err = Some(e);
                    } else {
                        self.circuit.record_failure().await;
                        return Err(e);
                    }
                }
            }
        }
        Err(last_err.unwrap_or_else(|| LlmError::Other("no attempts made".into())))
    }

    async fn complete_with_tools(
        &self,
        messages: &[Message],
        tools: &[serde_json::Value],
    ) -> Result<CompletionResponse, LlmError> {
        if !self.circuit.allow_request().await {
            return Err(LlmError::Other("circuit breaker is open".into()));
        }

        let mut last_err = None;
        for attempt in 1..=self.config.max_retries {
            match self.inner.complete_with_tools(messages, tools).await {
                Ok(result) => {
                    self.circuit.record_success().await;
                    return Ok(result);
                }
                Err(e) => {
                    if attempt < self.config.max_retries && Self::should_retry(&e) {
                        let delay =
                            Self::retry_delay(attempt, self.config.base_delay, self.config.max_delay);
                        tokio::time::sleep(delay).await;
                        last_err = Some(e);
                    } else {
                        self.circuit.record_failure().await;
                        return Err(e);
                    }
                }
            }
        }
        Err(last_err.unwrap_or_else(|| LlmError::Other("no attempts made".into())))
    }

    async fn complete_stream(
        &self,
        messages: &[Message],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>>, LlmError> {
        if !self.circuit.allow_request().await {
            return Err(LlmError::Other("circuit breaker is open".into()));
        }

        let mut last_err = None;
        for attempt in 1..=self.config.max_retries {
            match self.inner.complete_stream(messages).await {
                Ok(result) => {
                    self.circuit.record_success().await;
                    return Ok(result);
                }
                Err(e) => {
                    if attempt < self.config.max_retries && Self::should_retry(&e) {
                        let delay =
                            Self::retry_delay(attempt, self.config.base_delay, self.config.max_delay);
                        tokio::time::sleep(delay).await;
                        last_err = Some(e);
                    } else {
                        self.circuit.record_failure().await;
                        return Err(e);
                    }
                }
            }
        }
        Err(last_err.unwrap_or_else(|| LlmError::Other("no attempts made".into())))
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

    // ── Circuit Breaker Tests ───────────────────────────────────────────

    #[tokio::test]
    async fn test_circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(1));
        assert!(cb.allow_request().await);
        assert_eq!(cb.current_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_threshold() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        cb.record_failure().await;
        cb.record_failure().await;
        assert!(cb.allow_request().await);
        cb.record_failure().await;
        assert!(!cb.allow_request().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_after_timeout() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50));
        cb.record_failure().await;
        cb.record_failure().await;
        assert!(!cb.allow_request().await);
        tokio::time::sleep(Duration::from_millis(60)).await;
        assert!(cb.allow_request().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_success_resets() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        cb.record_failure().await;
        cb.record_failure().await;
        cb.record_success().await;
        assert_eq!(cb.failure_count.load(Ordering::Relaxed), 0);
        assert_eq!(cb.current_state().await, CircuitState::Closed);
    }

    // ── Retry Client Tests ──────────────────────────────────────────────

    struct RetryableClient {
        attempts: std::sync::Mutex<u32>,
        fail_count: u32,
    }

    impl RetryableClient {
        fn fail_then_succeed(fail_count: u32) -> Self {
            Self {
                attempts: std::sync::Mutex::new(0),
                fail_count,
            }
        }
    }

    #[async_trait]
    impl LlmClient for RetryableClient {
        async fn complete(&self, _messages: &[Message]) -> Result<String, LlmError> {
            let mut attempts = self.attempts.lock().unwrap();
            *attempts += 1;
            if *attempts <= self.fail_count {
                Err(LlmError::Api {
                    status: 500,
                    message: "transient".into(),
                })
            } else {
                Ok("recovered".into())
            }
        }

        async fn complete_stream(
            &self,
            _messages: &[Message],
        ) -> Result<Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>>, LlmError>
        {
            let mut attempts = self.attempts.lock().unwrap();
            *attempts += 1;
            if *attempts <= self.fail_count {
                Err(LlmError::StreamClosed)
            } else {
                Ok(Box::pin(futures_util::stream::empty()))
            }
        }
    }

    #[tokio::test]
    async fn test_retry_succeeds_after_transient_failure() {
        let config = RetryConfig {
            max_retries: 3,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
        };
        let client = RetryableClient::fail_then_succeed(2);
        let retry = RetryClient::with_config(Box::new(client), config);
        let result = retry.complete(&[Message::user("hi")]).await.unwrap();
        assert_eq!(result, "recovered");
    }

    #[tokio::test]
    async fn test_retry_exhausted_returns_last_error() {
        let config = RetryConfig {
            max_retries: 2,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
        };
        let client = RetryableClient::fail_then_succeed(10);
        let retry = RetryClient::with_config(Box::new(client), config);
        let err = retry.complete(&[Message::user("hi")]).await.unwrap_err();
        assert!(matches!(err, LlmError::Api { status: 500, .. }));
    }

    #[tokio::test]
    async fn test_retry_skips_4xx() {
        struct FourOhOneClient;

        #[async_trait]
        impl LlmClient for FourOhOneClient {
            async fn complete(&self, _messages: &[Message]) -> Result<String, LlmError> {
                Err(LlmError::Api {
                    status: 401,
                    message: "unauthorized".into(),
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

        let config = RetryConfig {
            max_retries: 3,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
        };
        let retry = RetryClient::with_config(Box::new(FourOhOneClient), config);
        let err = retry.complete(&[Message::user("hi")]).await.unwrap_err();
        assert!(matches!(err, LlmError::Api { status: 401, .. }));
    }

    #[tokio::test]
    async fn test_retry_delay_calculation() {
        let d1 = RetryClient::retry_delay(1, Duration::from_secs(1), Duration::from_secs(10));
        let d2 = RetryClient::retry_delay(2, Duration::from_secs(1), Duration::from_secs(10));
        let d3 = RetryClient::retry_delay(3, Duration::from_secs(1), Duration::from_secs(10));
        assert_eq!(d1, Duration::from_secs(1));
        assert_eq!(d2, Duration::from_secs(2));
        assert_eq!(d3, Duration::from_secs(4));
    }

    #[tokio::test]
    async fn test_retry_delay_capped() {
        let d = RetryClient::retry_delay(10, Duration::from_secs(1), Duration::from_secs(5));
        assert_eq!(d, Duration::from_secs(5));
    }
}
