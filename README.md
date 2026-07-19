# OpsPilot

> AI-Powered Modular Infrastructure Operations Platform — manage servers, containers, and monitoring through natural language.

[![CI/CD](https://github.com/OWNER/ops-pilot/actions/workflows/ci.yml/badge.svg)](https://github.com/OWNER/ops-pilot/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Rust](https://img.shields.io/badge/Rust-1.82+-orange?logo=rust)
![React](https://img.shields.io/badge/React-19-61DAFB?logo=react)

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    frontend/                            │
│       React 19 + Vite 6 + Tailwind CSS 3 + ReactFlow  │
│       Zustand state mgmt · React Query · React Router 7│
└──────────────────────────┬──────────────────────────────┘
                           │ REST API + WebSocket
┌──────────────────────────▼──────────────────────────────┐
│                   ops-pilot-gateway                     │
│   Axum 0.8 HTTP server · JWT auth middleware            │
│   ┌────────────┐ ┌──────────┐ ┌────────────────────┐   │
│   │ Agent ReAct│ │ LLM Chat │ │ Terminal WS→SSH    │   │
│   │   Loop     │ │ Service  │ │ Proxy              │   │
│   └─────┬──────┘ └────┬─────┘ └────────┬───────────┘   │
│         └─────────────┼────────────────┘                │
│         ┌─────────────▼────────────────┐                │
│         │      ToolRegistry            │                │
│         │  Route tool calls → modules  │                │
│         └─────────────┬────────────────┘                │
└───────────────────────┼─────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────┐
│                    ops-pilot-sdk                        │
│  OpsModule trait · EventBus · ModuleLoader · Context    │
└───────────────────────┬─────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────┐
│                    ops-pilot-core                       │
│  SSH (russh) · Docker (bollard) · Auth (argon2+JWT)   │
│  Host CRUD · SQLite · Vault · Audit · Alert            │
└─────────────────────────────────────────────────────────┘
```

## Modules

| Module | Description |
|--------|-------------|
| **mod-core** | Core operations: SSH connections, Docker container management, Host CRUD, system monitoring |
| **mod-rca** | Root Cause Analysis: rule-based diagnostics + LLM-powered deep analysis from system symptoms |
| **mod-security** | CIS compliance scanning, vulnerability checks, patch management, LLM-powered security reports |

## API Endpoints

### Authentication

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/auth/register` | Register a new user |
| POST | `/api/auth/login` | Login and receive JWT |

### Host Management

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/hosts` | List hosts for current user |
| POST | `/api/hosts` | Create a host |
| GET | `/api/hosts/:id` | Get host details |
| PUT | `/api/hosts/:id` | Update a host |
| DELETE | `/api/hosts/:id` | Delete a host |

### Vault (Encrypted Credentials)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/vault/status` | Check vault status |
| POST | `/api/vault/set-passphrase` | Set vault passphrase |
| POST | `/api/vault/unlock` | Unlock vault |
| POST | `/api/vault/lock` | Lock vault |

### Security Scanning

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/security/scan` | Run CIS compliance scan against a host |
| GET | `/api/security/checks` | List all available security checks |

### Modules

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/modules` | List loaded modules |
| GET | `/api/modules/:name` | Module details |
| POST | `/api/modules/:name/enable` | Enable a module |
| POST | `/api/modules/:name/disable` | Disable a module |
| GET | `/api/modules/:name/health` | Module health check |
| GET | `/api/health` | Aggregate health of all modules |

### Agent (AI Chat)

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/agent/session` | Create agent session |
| POST | `/api/agent/chat/:session_id` | Send message to agent |
| DELETE | `/api/agent/session/:session_id` | Close session |

### Terminal

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/terminal/:host_id` | WebSocket SSH terminal (`?token=` auth) |

## Frontend Features

- **Login / Register** — JWT-based authentication
- **Hosts** — CRUD management of infrastructure hosts with SSH/Docker details
- **Vault** — Encrypted credential storage with passphrase-protected unlock
- **Security Scan** — Run CIS compliance checks, view findings by severity, remediate
- **Health Dashboard** — Real-time module health monitoring with status badges
- **Agent Chat** — AI-powered conversational interface for infrastructure operations
- **Terminal** — In-browser SSH terminal via WebSocket proxy
- **Module Browser** — View and toggle loaded modules

## Quick Start

### Docker (Recommended)

```bash
# Clone and configure
cp .env.example .env
# Edit .env with your secrets (JWT_SECRET, LLM_API_KEY, etc.)

# Start all services
docker compose up -d

# Access at http://localhost:3001
```

### Local Development

#### Prerequisites

- Rust 1.82+
- Node.js 20+
- SQLite (bundled via `libsqlite3-sys`)

#### Backend

```bash
cargo build --workspace
cargo test --workspace
cargo run -p ops-pilot-gateway
```

#### Frontend

```bash
cd frontend
npm install
npm run dev     # Vite dev server at http://localhost:5173
npm run build   # Production build to dist/
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `LISTEN_ADDR` | `0.0.0.0:3001` | Server listen address |
| `DATABASE_URL` | `sqlite:ops-pilot.db` | Database connection string |
| `JWT_SECRET` | (required) | JWT signing secret |
| `LLM_PROVIDER` | `openai` | LLM provider: `openai` / `ollama` / `deepseek` / `mimo` |
| `LLM_BASE_URL` | provider-dependent | LLM API base URL |
| `LLM_API_KEY` | — | LLM API key |
| `LLM_MODEL` | `gpt-4o` | Chat model name |
| `STATIC_DIR` | `./static` | Frontend static files directory |
| `OPSPILOT_MASTER_KEY` | (recommended) | Master key for host credential encryption |

## Security Features

- Passwords hashed with **Argon2id**
- JWT tokens with 24-hour expiry
- All protected routes require `Authorization: Bearer <token>`
- Hosts API enforces per-user isolation
- **Vault** per-user passphrase-derived AES-256 encryption
- Host credentials encrypted with AES-256-GCM
- SSH supports password and public key authentication
- SSH host key verification (`known_hosts`) — MITM protection
- **Rate limiting**: login endpoint 5 requests/minute/IP
- **Audit logging**: SSH connect/disconnect events auto-recorded
- **Alert engine**: off-hours batch operations, high failure rate, first-connect detection
- **CIS compliance scanning**: 24 built-in benchmark rules

## Deployment

See [DEPLOY.md](DEPLOY.md) — covers Docker deployment, TLS configuration, and security checklist.

## License

Apache License 2.0 — see [LICENSE](LICENSE).
