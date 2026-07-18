# Changelog

All notable changes to OpsPilot will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **[P0]** SSH `reconnect()` was a no-op — new handle was created but immediately dropped without replacing the old one (`SshConnection.handle` not wrapped in `Arc<RwLock<...>>`)
- **[P1]** Agent `truncate_if_needed()` could orphan tool_call/tool_result message pairs when evicting old messages — now preserves atomicity
- **[P1]** Typo: `truncuate_if_needed` → `truncate_if_needed`

### Security

- **[P0]** SSH `check_server_key()` always returned `true` — vulnerable to man-in-the-middle attacks. Implemented `known_hosts` verification
- **[P0]** Upgraded `russh` from 0.50.x to 0.62.x — fixes OOM DoS via `channel_open_*` and username state reset bypass (russh 0.58.0 security patches)
- Host credentials stored in plaintext in SQLite — added AES-256-GCM encryption at rest

### Changed

- **[P0]** `russh` 0.50.x → 0.62.x, `russh-sftp` → 2.3.x, `russh-config` → 0.58.0
- **[P2]** `sqlx` 0.8.x → 0.9.0 (migration to `sqlx.toml` config, `smol` runtime support)
- **[P2]** `bollard` 0.14.x → 0.19.x (Docker API 1.46, Podman rootless auto-discovery)
- **[P3]** `opentelemetry` 0.29.x → 0.32.0 (Metrics SDK stable, Prometheus exporter → OTLP)
- Unified `EventBus` — removed duplicate definition in `ops-pilot-core::event`, now uses `ops-pilot-sdk::context::EventBus` exclusively
- `ToolRegistry` now maintains a `HashMap<String, Arc<dyn OpsModule>>` index for O(1) tool→module lookup instead of scanning all modules on every `invoke_tool` call
- All SQL queries now use `#[derive(sqlx::FromRow)]` structs instead of positional tuple destructuring

### Added

- `README.md` with architecture diagram, API reference, and configuration guide
- `CHANGELOG.md` (this file)
- `ARCHITECTURE.md` with detailed component diagrams and data flow
- Encrypted credential storage for host SSH passwords/keys using AES-256-GCM
- `known_hosts` file verification for SSH server key checking
- `SshConnectionPool` max connection limit (configurable, default 100)
- `tracing_subscriber` initialization with configurable log levels

## [0.1.0] - 2026-07-15

### Added

- Initial release
- SSH connection management with russh (password + public key auth)
- Docker container management via bollard (list, start, stop, restart, stats)
- Host CRUD with SQLite persistence
- User registration and JWT authentication (Argon2id + jsonwebtoken)
- Module SDK with `OpsModule` trait, `ModuleLoader`, `EventBus`, `ModuleContext`
- Gateway HTTP API (Axum 0.8) with route modules for hosts, modules, agent
- Agent ReAct loop with tool calling and conversation truncation
- LLM client abstraction (`LlmClient` trait) with text and tool-call support
- WebSocket-to-SSH terminal proxy
- Tool registry routing AI function calls to correct module
- React 18 frontend with Zustand state management
- Host management UI with CRUD operations
- AI chat interface with streaming support
- Workflow editor with ReactFlow
- User authentication (login/register) UI
