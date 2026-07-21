//! LLM provider abstraction: OpenAI, MiMo, Ollama, and routing.

pub mod chat;
pub mod provider;

pub use chat::{
    ChatService, CompletionResponse, FunctionCall, LlmClient, LlmError, Message, Role, ToolCall,
};
pub use provider::{ProviderConfig, ProviderLlm};
