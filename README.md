<div align="center">

# OpsPilot

### AI-Powered Modular Infrastructure Operations Platform

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)](https://www.rust-lang.org/)
[![CI](https://img.shields.io/github/actions/workflow/status/OWNER/ops-pilot/ci.yml?label=CI&logo=github)](https://github.com/OWNER/ops-pilot/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![GitHub Stars](https://img.shields.io/github/stars/OWNER/ops-pilot?style=social)](https://github.com/OWNER/ops-pilot)

</div>

---

**OpsPilot** is an open-source, AI-powered infrastructure operations platform that unifies server management, monitoring, incident response, and cost optimization into a single, extensible system. Built in Rust for performance and reliability, it features a pluggable module architecture that lets you tailor the platform to your exact operational needs — from automated root cause analysis to natural language infrastructure control. Whether you manage three servers or three thousand, OpsPilot gives you a single pane of glass with AI-assisted intelligence.

---

## ✨ Features

- 🧠 **AI-Powered Operations** — Natural language infrastructure control, automated RCA, and intelligent remediation powered by LLMs (Ollama, OpenAI, DeepSeek, MiMo)
- 🔌 **Modular Architecture** — Plugin system with hot-loadable modules; extend functionality without touching core code
- 🖥️ **Unified Infrastructure View** — SSH terminal, Docker management, and host monitoring in one dashboard
- 📊 **Cost Intelligence** — FinOps module for cloud spend analysis, anomaly detection, and optimization recommendations
- 🔒 **Security-First** — JWT authentication, RBAC, full audit trail, and secrets vault integration
- 🌐 **Real-Time Web UI** — React dashboard with live terminal, topology visualization, and interactive charts
- 🐳 **One-Command Deploy** — Docker Compose or bare metal; SQLite for small setups, PostgreSQL for scale
- 📡 **MCP Protocol Support** — Connect external AI tools and agents via the Model Context Protocol

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Web UI (React)                       │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────────┐  │
│  │Dashboard │ │ Terminal │ │Topology  │ │ Cost Analytics│  │
│  └──────────┘ └──────────┘ └──────────┘ └───────────────┘  │
└──────────────────────────┬──────────────────────────────────┘
                           │ REST / WebSocket
┌──────────────────────────┴──────────────────────────────────┐
│                     Core Engine (Rust)                       │
│  ┌──────────────┐ ┌──────────────┐ ┌────────────────────┐  │
│  │ Connection   │ │  Event Bus   │ │   Audit Trail      │  │
│  │   Pool       │ │  (tokio)     │ │   (append-only)    │  │
│  └──────────────┘ └──────────────┘ └────────────────────┘  │
│  ┌──────────────┐ ┌──────────────┐ ┌────────────────────┐  │
│  │  Monitoring  │ │   Secrets    │ │   Scheduler        │  │
│  │   Engine     │ │   Vault      │ │                    │  │
│  └──────────────┘ └──────────────┘ └────────────────────┘  │
└──────────┬───────────────┬───────────────┬─────────────────┘
           │               │               │
    ┌──────┴──────┐ ┌──────┴──────┐ ┌──────┴──────┐
    │ Module SDK  │ │ AI Gateway  │ │  Data Layer │
    │ (trait defs)│ │ (LLM route) │ │ (SQLite/PG) │
    └──────┬──────┘ └──────┬──────┘ └─────────────┘
           │               │
    ┌──────┴───────────────┴──────┐
    │      Pluggable Modules      │
    │ ┌─────┐ ┌─────┐ ┌───────┐  │
    │ │ rca │ │finop│ │ sec   │  │
    │ └─────┘ └─────┘ └───────┘  │
    │ ┌──────┐ ┌───────────────┐  │
    │ │ topo │ │  chatops      │  │
    │ └──────┘ └───────────────┘  │
    └─────────────────────────────┘
```

---

## 🚀 Quick Start

### Docker Compose (Recommended)

```bash
git clone https://github.com/OWNER/ops-pilot.git
cd ops-pilot
cp .env.example .env
# Edit .env with your settings
docker compose up -d
```

Open **http://localhost:3000** — the Web UI is ready.

### Manual Install

**Prerequisites:** Rust 1.75+, Node.js 20+, SQLite

```bash
# Clone and build
git clone https://github.com/OWNER/ops-pilot.git
cd ops-pilot
cargo build --release

# Build frontend
cd frontend && npm install && npm run build && cd ..

# Initialize database
./target/release/ops-pilot init

# Start the server
./target/release/ops-pilot serve --config config.toml
```

### First Steps

1. Open the Web UI and log in (default: `admin` / `ops-pilot`)
2. Add a host via **Hosts → Add Host** or use the CLI: `ops-pilot host add --name my-server --ip 192.168.1.100`
3. Connect and get a terminal: click the host card → **Connect**
4. Try AI chat: type "what's the CPU usage on my-server?" in the AI Chat panel

---

## 📦 Module Ecosystem

| Module | Description | Status |
|--------|-------------|--------|
| **mod-core** | Host management, SSH connections, Docker control, monitoring | ✅ Core |
| **mod-rca** | Automated root cause analysis with AI-powered log correlation | 🚧 In Development |
| **mod-finops** | Cloud cost analysis, anomaly detection, optimization recommendations | 🚧 In Development |
| **mod-security** | Vulnerability scanning, compliance checks, secrets rotation | 📋 Planned |
| **mod-topo** | Network topology discovery, dependency mapping, visualization | 📋 Planned |
| **mod-chatops** | Slack/Discord/Telegram integration, incident workflows, on-call | 📋 Planned |

Build your own modules with the [Module SDK](docs/MODULE_SDK.md).

---

## 🛠️ Tech Stack

| Layer | Technology |
|-------|-----------|
| **Backend** | Rust (axum, tokio, russh) |
| **Frontend** | React 19, TypeScript, Vite, Tailwind CSS |
| **Database** | SQLite (default) / PostgreSQL |
| **Terminal** | WebSocket + xterm.js |
| **Topology** | React Flow |
| **Charts** | Recharts |
| **State** | Zustand + TanStack Query |
| **LLM Providers** | Ollama, OpenAI, DeepSeek, MiMo |
| **Container Runtime** | Docker API |
| **SSH** | russh (pure Rust SSH client) |

---

## ⚖️ Comparison

| Feature | **OpsPilot** | SmartBox | K8sGPT | NetBox | Cleric |
|---------|:----------:|:--------:|:------:|:------:|:------:|
| Infrastructure-as-Code | ✅ | ✅ | ❌ | ❌ | ❌ |
| AI-Powered RCA | ✅ | ❌ | ✅ | ❌ | ✅ |
| SSH Terminal | ✅ | ✅ | ❌ | ❌ | ✅ |
| Docker Management | ✅ | ✅ | ✅ | ❌ | ✅ |
| Cost Optimization | ✅ | ❌ | ❌ | ❌ | ❌ |
| Topology Visualization | ✅ | ❌ | ❌ | ✅ | ❌ |
| Module/Plugin System | ✅ | ❌ | ❌ | ✅ | ❌ |
| ChatOps Integration | ✅ | ❌ | ❌ | ❌ | ✅ |
| Multi-LLM Support | ✅ | ❌ | ✅ | ❌ | ✅ |
| Web UI | ✅ | ✅ | ❌ | ✅ | ✅ |
| Self-Hosted | ✅ | ✅ | ✅ | ✅ | ✅ |
| License | MIT | MIT | Apache-2.0 | BSD-3 | MIT |

---

## 📖 Documentation

- [Architecture Overview](docs/ARCHITECTURE.md)
- [Module SDK Guide](docs/MODULE_SDK.md)
- [API Reference](docs/API_REFERENCE.md)
- [Project Roadmap](docs/ROADMAP.md)
- [Contributing Guide](docs/CONTRIBUTING.md)

---

## 🤝 Contributing

We welcome contributions! See the [Contributing Guide](docs/CONTRIBUTING.md) for development setup, coding standards, and PR process.

```bash
# Quick start for contributors
git clone https://github.com/OWNER/ops-pilot.git
cd ops-pilot
make dev  # Sets up everything
```

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).

---

<div align="center">
  <sub>Built with 🦀 Rust and ❤️ by the OpsPilot community</sub>
</div>
