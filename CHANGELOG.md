# 变更日志

> 所有 OpsPilot 的显著变更均记录在此。
> 格式遵守 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
> 版本遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### Fixed 修复

- **[P0]** SSH `reconnect()` 是空操作 —— 新 handle 创建后立即丢弃，未替换旧 handle（`SshConnection.handle` 未用 `Arc<RwLock<...>>` 包裹）
- **[P1]** Agent `truncate_if_needed()` 在驱逐旧消息时可能破坏 tool_call/tool_result 成对关系 —— 现已保证原子性
- **[P1]** 拼写错误：`truncuate_if_needed` → `truncate_if_needed`

## [0.1.0] - 2026-07-15

### Added 新增

- SSH 连接管理（基于 russh，支持密码和公钥认证）
- Docker 容器管理（基于 bollard：列表、启动、停止、重启、统计）
- 主机 CRUD，SQLite 持久化
- 用户注册和 JWT 认证（Argon2id + jsonwebtoken）
- Module SDK：`OpsModule` trait、`ModuleLoader`、`EventBus`、`ModuleContext`
- Gateway HTTP API（Axum 0.8），含 hosts/modules/agent 路由模块
- Agent ReAct 循环，支持工具调用和对话截断
- LLM 客户端抽象（`LlmClient` trait），支持文本和工具调用
- WebSocket-to-SSH 终端代理
- 工具注册表（ToolRegistry），将 AI function call 路由到对应模块
- React 19 前端，Zustand 状态管理
- 主机管理 UI（CRUD 操作）
- AI 对话界面（流式输出）
- ReactFlow 工作流编辑器
- 用户认证 UI（登录/注册）
- 3 个内置模块：mod-core、mod-rca、mod-security
- CI/CD 流水线（GitHub Actions），含前端 lint+test+build、Rust check+clippy+test+build、Docker GHCR 发布
- 多阶段 Dockerfile，前端 + Rust 依赖缓存
- docker-compose 生产部署，支持 Ollama 集成
- 主机健康监控仪表盘（自动刷新）
- 模块配置编辑器（实时 JSON 校验）
- Toast 通知系统（Context Provider）
- CI/CD 状态徽章

### Security 安全

- **[P0]** SSH `check_server_key()` 始终返回 `true` —— 存在中间人攻击风险。已实现 `known_hosts` 主机密钥验证
- **[P0]** 升级 `russh` 0.50.x → 0.62.x —— 修复了 OOM DoS（`channel_open_*`）和用户名状态重置绕过问题（russh 0.58.0 安全补丁）
- 主机凭据在 SQLite 中以明文存储 —— 已改为 AES-256-GCM 加密存储

### Changed 变更

- **[P0]** `russh` 0.50.x → 0.62.x，`russh-sftp` → 2.3.x，`russh-config` → 0.58.0
- **[P1]** `bollard` 0.14.x → 0.19.x（Docker API 1.46，Podman rootless 自动发现）
- `EventBus` 统一 —— 移除 `ops-pilot-core::event` 中的重复定义，统一使用 `ops-pilot-sdk::context::EventBus`
- `ToolRegistry` 现在维护 `HashMap<String, Arc<dyn OpsModule>>` 索引，工具→模块查找从 O(n) 降为 O(1)
- 所有 SQL 查询改用 `#[derive(sqlx::FromRow)]` 结构体代替位置元组解构
