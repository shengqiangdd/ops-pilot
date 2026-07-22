# Changelog

> All notable changes to OpsPilot are documented here.
> This project adheres to [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
> and follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### ✨ Added

- PWA offline support + Service Worker registration
- WebSocket connection status indicator + auto-reconnect
- Global error monitoring and error log collection
- Keyboard shortcuts panel with Mac/Win compatibility + search filtering
- Login page polish + OAuth entry point
- One-click backup / restore / cleanup wizard for system maintenance
- Scheduled task management page
- Email SMTP sending support

### 🧪 Testing

- Backend integration test expansion: auth(8) + health(6) + search(11) + e2e(5)
- Playwright E2E expansion: vault(9) + alerts(15) + hosts(10) + notifications(11) + search(9)
- Frontend hooks tests: useAlerts / useHosts / useKeyboardShortcuts
- Frontend lib tests: cn / health / metrics / pageStates

### 🔒 Security

- Security headers middleware (CSP / XSS / Frame / Referrer / HSTS)
- Content Security Policy configuration
- Sourcemaps disabled in production builds

### 📦 Deployment

- Docker Compose resource limits + log rotation
- `.dockerignore` optimization
- Makefile with development scripts
- Helm chart CI lint integration

### 📚 Documentation

- CONTRIBUTING.md contribution guide
- CODE_OF_CONDUCT.md code of conduct
- SECURITY.md security policy
- GitHub Issue / PR templates
- API endpoint reference table

### Fixed

- **[P0]** Project structure: moved `src/` under `backend/` to align with Cargo.toml workspace member paths — fixes all CI build failures
- **[P0]** CI config: added `working-directory: backend` to Rust job, set Docker build context to project root
- **[P0]** SSH `reconnect()` was a no-op — new handle was discarded immediately after creation without replacing the old handle (`SshConnection.handle` was not wrapped in `Arc<RwLock<...>>`)
- **[P1]** Agent `truncate_if_needed()` could break tool_call / tool_result pairing when evicting old messages — now guarantees atomicity
- **[P1]** Typo: `truncuate_if_needed` → `truncate_if_needed`
- **[P1]** Bare `core` pattern in `.gitignore` incorrectly matched `backend/src/core/`; changed to `/core`
- **[P1]** Dockerfile COPY paths aligned, docker-compose.yml context/dockerfile references fixed

## [0.1.0] - 2026-07-15

### Added

- SSH connection management (based on russh, supports password and public key authentication)
- Docker container management (based on bollard: list, start, stop, restart, stats)
- Host CRUD with SQLite persistence
- User registration and JWT authentication (Argon2id + jsonwebtoken)
- Module SDK: `OpsModule` trait, `ModuleLoader`, `EventBus`, `ModuleContext`
- Gateway HTTP API (Axum 0.8) with hosts / modules / agent route modules
- Agent ReAct loop with tool call support and conversation truncation
- LLM client abstraction (`LlmClient` trait) supporting text and tool calls
- WebSocket-to-SSH terminal proxy
- Tool registry (`ToolRegistry`) routing AI function calls to corresponding modules
- React 19 frontend with Zustand state management
- Host management UI (CRUD operations)
- AI chat interface (streaming output)
- ReactFlow workflow editor
- User authentication UI (login / register)
- 3 built-in modules: mod-core, mod-rca, mod-security
- CI/CD pipeline (GitHub Actions) with frontend lint+test+build, Rust check+clippy+test+build, Docker GHCR publish
- Multi-stage Dockerfile with frontend + Rust dependency caching
- docker-compose production deployment with Ollama integration
- Host health monitoring dashboard (auto-refresh)
- Module configuration editor (real-time JSON validation)
- Toast notification system (Context Provider)
- CI/CD status badges

### Security

- **[P0]** SSH `check_server_key()` always returned `true` — MITM attack risk. Implemented `known_hosts` host key verification
- **[P0]** Upgraded `russh` 0.50.x → 0.62.x — fixes OOM DoS (`channel_open_*`) and username state reset bypass (russh 0.58.0 security patch)
- Host credentials stored in plaintext in SQLite — changed to AES-256-GCM encrypted storage

### Changed

- **[P0]** `russh` 0.50.x → 0.62.x, `russh-sftp` → 2.3.x, `russh-config` → 0.58.0
- **[P1]** `bollard` 0.14.x → 0.19.x (Docker API 1.46, Podman rootless auto-discovery)
- Unified `EventBus` — removed duplicate definitions in `ops-pilot-core::event`, now uses `ops-pilot-sdk::context::EventBus` consistently
- `ToolRegistry` now maintains a `HashMap<String, Arc<dyn OpsModule>>` index, tool→module lookup reduced from O(n) to O(1)
- All SQL queries migrated to `#[derive(sqlx::FromRow)]` structs instead of positional tuple destructuring
