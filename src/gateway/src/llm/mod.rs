//! LLM provider abstraction: OpenAI, MiMo, Ollama, and routing.

pub mod chat;

pub use chat::{ChatService, LlmClient, LlmError, Message, Role, ToolCall};
