//! # ops-pilot-sdk
//!
//! OpsPilot 模块开发 SDK —— 定义模块契约、事件总线和共享上下文。
//!
//! 本 crate 是所有可插拔模块（`mod-*`）的唯一依赖，提供了：
//!
//! - **[`traits::OpsModule`]** — 模块必须实现的核心 trait（工具注册、事件处理、健康检查）
//! - **[`context::ModuleContext`]** — 模块执行时的共享上下文（数据库连接、事件总线、配置目录）
//! - **[`context::EventBus`]** / **[`events::global_event_bus()`]** — 基于 `tokio::broadcast` 的事件发布/订阅
//! - **[`error::OpsError`]** — 统一错误类型，可无缝转换为 `anyhow::Error`
//! - **[`loader::ModuleLoader`]** — 模块注册、依赖解析和生命周期管理
//! - **[`llm`]** — LLM 客户端 trait，供需要 AI 能力的模块使用
//!
//! ## 快速开始
//!
//! ```rust,no_run
//! use ops_pilot_sdk::traits::{OpsModule, ToolDefinition, HealthStatus};
//! use ops_pilot_sdk::context::ModuleContext;
//!
//! pub struct MyModule;
//!
//! #[async_trait::async_trait]
//! impl OpsModule for MyModule {
//!     fn name(&self) -> &str { "my-module" }
//!     fn version(&self) -> &str { "0.1.0" }
//!     fn description(&self) -> &str { "示例模块" }
//!     fn dependencies(&self) -> Vec<&str> { vec![] }
//!     fn tools(&self) -> Vec<ToolDefinition> { vec![] }
//!     async fn execute(&self, _ctx: &ModuleContext, _tool: &str, _params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
//!         Ok(serde_json::json!({}))
//!     }
//!     async fn on_event(&self, _ctx: &ModuleContext, _event: &ops_pilot_sdk::events::OpsEvent) -> Option<ops_pilot_sdk::traits::ModuleAction> { None }
//!     async fn health_check(&self, _ctx: &ModuleContext) -> HealthStatus { HealthStatus::Healthy }
//! }
//! ```

pub mod context;
pub mod error;
pub mod events;
pub mod llm;
pub mod loader;
pub mod traits;

pub use error::OpsError;
pub use events::global_event_bus;
