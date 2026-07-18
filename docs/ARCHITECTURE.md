# Architecture Overview

This document describes the internal architecture of OpsPilot, covering system layers, data flows, security model, and deployment topologies.

---

## Table of Contents

- [System Overview](#system-overview)
- [Layer Descriptions](#layer-descriptions)
- [Data Flow Diagrams](#data-flow-diagrams)
- [Security Model](#security-model)
- [Deployment Architecture](#deployment-architecture)

---

## System Overview

OpsPilot follows a layered architecture with clear separation of concerns. The system is designed around three principles:

1. **Core stability** — The engine must never crash, regardless of module failures.
2. **Module isolation** — Each module runs in its own async context with controlled access to core services.
3. **AI-first design** — LLM capabilities are woven into every layer, not bolted on as an afterthought.

```
┌─────────────────────────────────────────────────────────────────┐
│                        CLIENT LAYER                             │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────────────┐ │
│  │  Web Browser │  │  CLI Client  │  │  External AI Agents    │ │
│  │  (React SPA) │  │  (ops-pilot) │  │  (MCP Protocol)        │ │
│  └──────┬──────┘  └──────┬───────┘  └───────────┬────────────┘ │
└─────────┼────────────────┼───────────────────────┼──────────────┘
          │ REST / WS      │ HTTP                   │ MCP
┌─────────┴────────────────┴───────────────────────┴──────────────┐
│                      API LAYER (axum)                            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐   │
│  │  Auth    │ │ REST     │ │WebSocket │ │  MCP Handler     │   │
│  │  (JWT)   │ │ Router   │ │ Gateway  │ │  (SSE transport) │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────────────┘   │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────┐
│                      CORE ENGINE                                 │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌──────────────┐ │
│  │ Connection │ │ Event Bus  │ │  Audit     │ │  Scheduler   │ │
│  │   Pool     │ │ (channels) │ │  Trail     │ │  (cron jobs) │ │
│  └────────────┘ └────────────┘ └────────────┘ └──────────────┘ │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌──────────────┐ │
│  │ Monitoring │ │  Secrets   │ │  Module    │ │  Database    │ │
│  │   Engine   │ │  Vault     │ │  Manager   │ │  (SQLx)      │ │
│  └────────────┘ └────────────┘ └────────────┘ └──────────────┘ │
└──────────┬──────────────────────────────┬───────────────────────┘
           │                              │
┌──────────┴──────────┐      ┌───────────┴──────────────────────┐
│    MODULE SDK        │      │        AI GATEWAY                 │
│  ┌────────────────┐  │      │  ┌──────────┐  ┌──────────────┐ │
│  │ Trait Defs     │  │      │  │ LLM      │  │ Agent        │ │
│  │ Lifecycle Mgr  │  │      │  │ Router   │  │ Orchestrator │ │
│  │ DI Container   │  │      │  ├──────────┤  ├──────────────┤ │
│  │ Config Schema  │  │      │  │ Tool     │  │ MCP          │ │
│  └────────────────┘  │      │  │ Registry │  │ Protocol     │ │
└──────────┬───────────┘      │  └──────────┘  └──────────────┘ │
           │                  └───────────────┬──────────────────┘
┌──────────┴──────────────────────────────────┴───────────────────┐
│                    MODULES (Pluggable)                           │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌────────────┐  │
│  │ mod- │ │ mod- │ │ mod- │ │ mod- │ │ mod- │ │ mod-       │  │
│  │ core │ │ rca  │ │finops│ │ sec  │ │ topo │ │ chatops    │  │
│  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘ └────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Layer Descriptions

### 1. Infrastructure Layer

The infrastructure layer provides the raw connectivity and execution capabilities that all higher layers depend on.

**Components:**

| Component | Responsibility | Implementation |
|-----------|---------------|----------------|
| SSH Connector | Persistent SSH sessions with connection pooling | `russh` (pure Rust) |
| Docker Connector | Container lifecycle, image management, stats | Docker Engine API |
| HTTP Client | Outbound HTTP for APIs, webhooks, LLM calls | `reqwest` |
| File System | Local file operations, config loading | `tokio::fs` |

**Key Design Decisions:**

- SSH connections are pooled and reused via `Arc<Mutex<SshSession>>`. Each host maintains one persistent session with automatic reconnection on failure.
- Docker connector uses the Unix socket directly (`/var/run/docker.sock`) without requiring the Docker CLI.
- All infrastructure operations are wrapped in `async fn` with timeout support to prevent hung connections.

```rust
pub struct InfraLayer {
    pub ssh_pool: Arc<SshConnectionPool>,
    pub docker: Arc<DockerClient>,
    pub http: reqwest::Client,
}
```

### 2. Core Engine

The core engine is the backbone of OpsPilot. It manages state, orchestrates modules, and provides shared services.

**Connection Pool:**
- Maintains SSH sessions per host with health checks every 30 seconds.
- Supports lazy connection (connect on first use) and eager pre-connection.
- Tracks connection metadata: latency, last used, error count.

**Event Bus:**
- Built on `tokio::sync::broadcast` channels.
- Decoupled publish/subscribe pattern for module communication.
- Events are typed enums with serialization support for persistence.

**Audit Trail:**
- Append-only log of all operations (SSH commands, API calls, module actions).
- Stored in SQLite/PostgreSQL with time-based partitioning.
- Queryable via REST API with filters for user, action type, and time range.

**Monitoring Engine:**
- Periodic health checks for all connected hosts (configurable interval).
- Metrics collection: CPU, memory, disk, network (via SSH or agent).
- Threshold-based alerting with webhook/email/Slack notification support.

**Scheduler:**
- Cron-based job scheduling for recurring tasks.
- Module actions can be scheduled (e.g., nightly RCA scan, weekly cost report).
- Job history and retry logic with exponential backoff.

### 3. Module SDK

The Module SDK defines the contract between the core engine and modules. Every module must implement the `OpsModule` trait.

**Key Concepts:**

- **Trait Definitions** — Modules implement `OpsModule` with lifecycle hooks, tool registration, and event handlers.
- **Lifecycle Management** — Modules go through `init → start → running → stop` states, managed by the core engine.
- **Dependency Injection** — Modules receive a `ModuleContext` that provides access to core services (SSH, Docker, AI, database) without direct coupling.
- **Configuration** — Each module declares its own TOML-based configuration schema, validated at startup.

**Module Isolation:**

- Each module runs in its own `tokio::spawn` task.
- A module crash is caught and logged without affecting other modules or the core engine.
- Module communication happens exclusively through the event bus.

See [Module SDK Guide](MODULE_SDK.md) for complete specifications.

### 4. AI Gateway

The AI Gateway is the intelligence layer that connects LLM providers to operational capabilities.

**LLM Router:**
- Supports multiple providers: Ollama (local), OpenAI, DeepSeek, MiMo.
- Provider selection based on configuration with fallback chains.
- Request/response streaming for real-time token output.

**Agent Orchestrator:**
- Manages multi-step AI workflows (e.g., RCA → analysis → remediation).
- Tool calling: LLMs can invoke module tools (SSH exec, host query, Docker actions).
- Conversation context management with automatic summarization for long sessions.

**MCP Protocol:**
- Implements the Model Context Protocol for external tool integration.
- External AI agents (Claude, GPT) can discover and invoke OpsPilot tools.
- SSE (Server-Sent Events) transport for real-time streaming.

**Tool Registry:**
- Modules register their tools with the AI Gateway.
- Tools are described with JSON Schema for LLM function calling.
- Permission-based access control (which tools each user/role can invoke).

```
┌─────────────────────────────────────────────────┐
│                AI Gateway                        │
│                                                  │
│  ┌────────────┐     ┌────────────┐              │
│  │  LLM       │────→│  Agent     │              │
│  │  Router    │     │  Orchestr. │              │
│  └─────┬──────┘     └─────┬──────┘              │
│        │                   │                     │
│  ┌─────┴──────┐     ┌─────┴──────┐              │
│  │  Provider  │     │  Tool      │              │
│  │  Adapters  │     │  Registry  │              │
│  │            │     │            │              │
│  │ - Ollama   │     │ - ssh/exec │              │
│  │ - OpenAI   │     │ - host/*   │              │
│  │ - DeepSeek │     │ - docker/* │              │
│  │ - MiMo     │     │ - mod-*    │              │
│  └────────────┘     └────────────┘              │
└─────────────────────────────────────────────────┘
```

### 5. Modules

Modules are self-contained extensions that add domain-specific capabilities.

| Module | Core Responsibility | Key Tools |
|--------|-------------------|-----------|
| **mod-core** | Host CRUD, SSH sessions, Docker management | `host.list`, `ssh.exec`, `docker.ps` |
| **mod-rca** | Root cause analysis from alerts and logs | `rca.analyze`, `rca.suggest_fix`, `rca.correlate` |
| **mod-finops** | Cost analysis and optimization | `cost.analyze`, `cost.anomaly`, `cost.forecast` |
| **mod-security** | Vulnerability scanning and compliance | `security.scan`, `security.compliance`, `security.rotate` |
| **mod-topo** | Network topology discovery and visualization | `topo.discover`, `topo.map`, `topo.dependencies` |
| **mod-chatops** | Chat platform integration | `chat.send`, `chat.acknowledge`, `chat.escalate` |

### 6. Web UI

The frontend is a React single-page application served by the core engine's static file handler.

**Key Views:**

- **Dashboard** — System overview with host health, active alerts, cost summary.
- **Hosts** — Host management with connection status, quick actions, terminal access.
- **Terminal** — Full PTY terminal via WebSocket, powered by xterm.js.
- **Topology** — Interactive network graph with React Flow.
- **Cost Analytics** — Charts and tables for cost data (Recharts).
- **AI Chat** — Conversational interface with tool execution and streaming responses.
- **Modules** — Module browser with enable/disable, configuration, and health status.
- **Audit Log** — Searchable operation history with filters.

**State Management:**

- Zustand for global UI state (theme, sidebar, auth).
- TanStack Query for server state (host data, module configs, costs).
- WebSocket for real-time updates (terminal output, alerts, logs).

---

## Data Flow Diagrams

### Alert → RCA → Auto-Fix Flow

```
                    ┌─────────────┐
                    │  Host       │
                    │  (SSH/Agent)│
                    └──────┬──────┘
                           │ metrics/logs
                           ▼
                    ┌─────────────┐
                    │  Monitoring │
                    │  Engine     │
                    │  (mod-core) │
                    └──────┬──────┘
                           │ threshold exceeded
                           ▼
                    ┌─────────────┐
                    │  Event Bus  │
                    │  alert.*    │
                    └──┬──────┬───┘
                       │      │
              ┌────────┘      └────────┐
              ▼                         ▼
       ┌─────────────┐          ┌─────────────┐
       │  mod-rca    │          │  mod-chatops │
       │  analyze    │          │  notify      │
       └──────┬──────┘          └─────────────┘
              │ AI analysis
              ▼
       ┌─────────────┐
       │  AI Gateway │
       │  (LLM call) │
       └──────┬──────┘
              │ root cause + fix suggestion
              ▼
       ┌─────────────┐     ┌──────────────┐
       │  User       │────→│  mod-core    │
       │  approves   │     │  ssh.exec    │
       └─────────────┘     └──────┬───────┘
                                  │ command executed
                                  ▼
                           ┌─────────────┐
                           │  Audit Log  │
                           └─────────────┘
```

**Flow Description:**

1. Monitoring engine detects a threshold breach (e.g., disk > 90%).
2. Alert event is published to the event bus.
3. `mod-rca` receives the event, collects related logs and metrics.
4. RCA module sends context to AI Gateway for analysis.
5. LLM returns root cause (e.g., "log rotation failed") and suggested fix.
6. Suggestion is presented to the user for approval.
7. On approval, the fix command is executed via SSH.
8. All actions are recorded in the audit trail.

### Natural Language → SSH Command Execution

```
    User Input: "restart nginx on prod-web-01"
              │
              ▼
    ┌─────────────────┐
    │  Web UI / CLI   │
    │  POST /ai/chat  │
    └────────┬────────┘
             │
             ▼
    ┌─────────────────┐
    │  AI Gateway     │
    │  - Parse intent  │
    │  - Select tool   │
    │  - Generate cmd  │
    └────────┬────────┘
             │ tool_call: ssh.exec
             │ params: {host: "prod-web-01", cmd: "systemctl restart nginx"}
             ▼
    ┌─────────────────┐
    │  mod-core       │
    │  ssh.exec       │
    │  - Validate     │
    │  - Execute      │
    │  - Return result│
    └────────┬────────┘
             │
             ▼
    ┌─────────────────┐
    │  AI Gateway     │
    │  - Format output│
    │  - "Nginx has   │
    │    been restart  │
    │    on prod-web-01"│
    └────────┬────────┘
             │
             ▼
    User sees: "✅ Nginx restarted successfully on prod-web-01"
```

**Safety Mechanisms:**

- Destructive commands (`rm -rf`, `mkfs`, `dd`) are blocked by default.
- Users must confirm high-risk operations via the Web UI.
- All AI-generated commands are logged in the audit trail with the original natural language request.
- Role-based access: non-admin users cannot execute arbitrary SSH commands.

### Cost Anomaly Detection Flow

```
    ┌─────────────────────────────────────────────┐
    │              Scheduler (cron)                │
    │  runs mod-finops.cost.scan every 6 hours    │
    └──────────────────────┬──────────────────────┘
                           │
                           ▼
    ┌─────────────────────────────────────────────┐
    │  mod-finops                                  │
    │  1. Fetch cost data from providers           │
    │  2. Compare against baseline                 │
    │  3. Calculate deviation                      │
    │  4. Flag anomalies (μ + 3σ)                 │
    └──────────────────────┬──────────────────────┘
                           │ anomaly detected
                           ▼
    ┌─────────────────────────────────────────────┐
    │  Event Bus: cost.anomaly_detected            │
    └──┬──────────────────────────────────┬───────┘
       │                                  │
       ▼                                  ▼
  ┌─────────┐                    ┌──────────────┐
  │ mod-rca │                    │ mod-chatops  │
  │ Correlate│                    │ Alert team   │
  │ with ops │                    │ via Slack    │
  └────┬────┘                    └──────────────┘
       │
       ▼
  ┌──────────────┐
  │ AI Gateway   │
  │ "EC2 costs   │
  │  up 40% on   │
  │  us-east-1.  │
  │  Root cause: │
  │  12 idle     │
  │  instances   │
  │  from last   │
  │  week's      │
  │  test run"   │
  └──────────────┘
```

---

## Security Model

### Authentication

- **JWT Tokens** — Stateless authentication with configurable expiration.
- **Token Refresh** — Short-lived access tokens (15 min) with longer refresh tokens (7 days).
- **Password Hashing** — bcrypt with configurable cost factor.
- **Session Management** — Active sessions tracked in database, revocable.

### Authorization (RBAC)

```yaml
roles:
  admin:
    permissions: ["*"]
  operator:
    permissions:
      - "host.read"
      - "host.connect"
      - "ssh.exec"
      - "docker.read"
      - "module.read"
      - "audit.read"
  viewer:
    permissions:
      - "host.read"
      - "audit.read"
      - "dashboard.read"
  custom:
    permissions: []  # Admin-defined
```

**Permission Hierarchy:**

```
host.read → host.connect → ssh.exec → ssh.exec.dangerous
docker.read → docker.manage
module.read → module.enable → module.configure
audit.read
dashboard.read
```

### Audit Trail

Every operation generates an audit record:

```json
{
  "id": "evt_abc123",
  "timestamp": "2026-07-15T10:30:00Z",
  "user_id": "user_001",
  "action": "ssh.exec",
  "target": "host:prod-web-01",
  "details": {
    "command": "systemctl restart nginx",
    "exit_code": 0,
    "duration_ms": 1200
  },
  "risk_level": "medium",
  "ai_generated": true
}
```

**Audit Properties:**

- Append-only (no updates or deletes, even by admins).
- Time-partitioned for efficient queries.
- Exportable to external SIEM systems.
- Retention policy configurable (default: 90 days).

### Secrets Vault

Sensitive data (SSH keys, API tokens, passwords) is encrypted at rest:

- **Encryption:** AES-256-GCM with a master key derived from `JWT_SECRET`.
- **Storage:** Encrypted blobs in the database, never logged or exposed via API.
- **Access:** Modules request secrets through `ModuleContext::get_secret()`, which logs the access.
- **Rotation:** Built-in rotation reminders and automated rotation for supported providers.

### Network Security

- HTTPS enforcement (TLS termination at reverse proxy).
- CORS configuration for API endpoints.
- Rate limiting on authentication endpoints.
- WebSocket connection authentication (token in query string or header).

---

## Deployment Architecture

### Single Node (Development / Small Teams)

```
┌───────────────────────────────┐
│          Single Server         │
│                                │
│  ┌──────────┐  ┌───────────┐  │
│  │ops-pilot │  │  SQLite   │  │
│  │ :3000    │  │  (file)   │  │
│  │ :8080    │  └───────────┘  │
│  └──────────┘                  │
│                                │
│  ┌──────────┐                  │
│  │ Ollama   │                  │
│  │ :11434   │                  │
│  └──────────┘                  │
└───────────────────────────────┘
```

**Resources:** 2 CPU cores, 4 GB RAM, 20 GB disk minimum.

### Multi-Node (Production)

```
┌──────────────────┐     ┌──────────────────┐
│   Load Balancer  │     │   Load Balancer   │
│   (nginx/HAProxy)│     │                   │
└────────┬─────────┘     └────────┬──────────┘
         │                        │
    ┌────┴────────────────────────┴────┐
    │                                   │
    ▼                                   ▼
┌──────────┐  ┌──────────┐  ┌──────────────┐
│ops-pilot │  │ops-pilot │  │  PostgreSQL  │
│  (node 1)│  │  (node 2)│  │  (primary)   │
└──────────┘  └──────────┘  └──────┬───────┘
                                   │
                            ┌──────┴───────┐
                            │  PostgreSQL   │
                            │  (replica)    │
                            └──────────────┘
```

### Docker Compose (Recommended for Most Users)

```yaml
# docker-compose.yml
services:
  ops-pilot:
    image: ghcr.io/OWNER/ops-pilot:latest
    ports: ["3000:3000", "8080:8080"]
    volumes:
      - ./data:/app/data
      - /var/run/docker.sock:/var/run/docker.sock
    environment:
      - DATABASE_URL=sqlite:///app/data/ops-pilot.db
      - JWT_SECRET=${JWT_SECRET}
    restart: unless-stopped

  ollama:
    image: ollama/ollama:latest
    ports: ["11434:11434"]
    volumes:
      - ollama_data:/root/.ollama
    restart: unless-stopped
```

### Kubernetes (Production Scale)

```yaml
# Deployed via Helm chart (planned for Phase 4)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ops-pilot
spec:
  replicas: 2
  selector:
    matchLabels:
      app: ops-pilot
  template:
    spec:
      containers:
        - name: ops-pilot
          image: ghcr.io/OWNER/ops-pilot:latest
          ports:
            - containerPort: 3000
            - containerPort: 8080
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: ops-pilot-db
                  key: url
```

**K8s Features (Phase 4):**
- Custom Resource Definitions (CRDs) for hosts and modules.
- Operator pattern for automated lifecycle management.
- Service mesh integration (Istio/Linkerd) for mTLS between nodes.
- Horizontal pod autoscaling based on connection count.

---

## Design Principles

1. **Zero-Downtime Module Updates** — Modules can be enabled/disabled/reloaded without restarting the core engine.
2. **Fail-Safe Defaults** — All modules start in a safe state; destructive operations require explicit opt-in.
3. **Observable Everything** — Every component emits structured logs and OpenTelemetry traces.
4. **Configuration as Code** — All settings are TOML/YAML files, version-controllable and diffable.
5. **Minimal Footprint** — SQLite for small deployments; PostgreSQL only when needed. Single binary distribution.
