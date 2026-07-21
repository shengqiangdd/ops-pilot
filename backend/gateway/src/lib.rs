//! # ops-pilot-gateway
//!
//! HTTP 网关层 —— 基于 Axum 的 REST API 服务器和 WebSocket 终端。
//!
//! 本 crate 负责：
//!
//! - **路由** ([`routes`]) — 主机管理、模块控制、Agent 对话、Vault 操作等 API 端点
//! - **认证** ([`middleware`]) — JWT 令牌验证中间件和请求提取器
//! - **Agent** ([`agent`]) — ReAct 循环编排、会话管理、工具调用分发
//! - **LLM** ([`llm`]) — LLM 提供商配置、HTTP 客户端、流式响应处理
//! - **终端** ([`terminal`]) — SSH 交互式终端的 WebSocket 代理
//! - **工具注册** ([`tools`]) — 模块工具聚合、索引缓存、OpenAI schema 格式化
//!
//! 网关本身不包含业务逻辑 —— 所有计算委托给 `ops-pilot-core` 和 `ops-pilot-mod-*` 模块。

pub mod agent;
pub mod docs;
pub mod llm;
pub mod metrics;
pub mod middleware;
pub mod oauth2;
pub mod routes;
pub mod seed;
pub mod terminal;
pub mod tools;
pub mod ws_events;
