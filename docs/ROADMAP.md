# Project Roadmap

This document outlines the development roadmap for OpsPilot across four phases, from MVP to ecosystem maturity.

---

## Timeline Overview

```
Phase 1          Phase 2          Phase 3          Phase 4
MVP              Modules          Enterprise       Ecosystem
Month 1-2        Month 3-4        Month 5-6        Month 7+
─────────────────────────────────────────────────────────────►

Core Engine      mod-rca          mod-security     Module Marketplace
SSH + Docker     mod-finops       mod-topo         K8s Operator
Web UI           AI Integration   mod-chatops      Multi-Tenant
Basic AI Chat    Event System     RBAC             Enterprise SSO
```

---

## Phase 1: MVP (Month 1–2)

**Goal:** A functional single-server infrastructure operations platform that can manage hosts, execute SSH commands, control Docker containers, and provide basic AI-assisted operations through a web interface.

### Deliverables

| # | Deliverable | Description | Success Metric |
|---|------------|-------------|----------------|
| 1.1 | Core Engine | Rust backend with axum HTTP server, database layer (SQLite), connection pool | Handles 100 concurrent SSH sessions without memory leaks |
| 1.2 | SSH Module | Persistent SSH connections via russh, command execution, terminal PTY | 99.9% command execution success rate |
| 1.3 | Docker Module | Container listing, start/stop/restart, basic stats via Docker API | Works with Docker Engine 24+ |
| 1.4 | Host Management | CRUD for hosts, health checks, status tracking | Supports 50+ hosts per instance |
| 1.5 | Web UI Shell | React SPA with routing, auth, sidebar, dashboard layout | Lighthouse score > 90 |
| 1.6 | Terminal UI | xterm.js WebSocket terminal for SSH sessions | Supports 256 colors, resize, copy/paste |
| 1.7 | Basic AI Chat | Chat interface with Ollama/OpenAI integration, tool calling for SSH | < 3s response time for simple queries |
| 1.8 | Authentication | JWT-based auth, user registration, password hashing | bcrypt with cost factor 12 |
| 1.9 | Audit Trail | Append-only operation logging with query API | Logs all SSH commands and API calls |
| 1.10 | Docker Compose | One-command deployment with all dependencies | `docker compose up` works on Linux/macOS |

### Technical Milestones

- **Week 1-2:** Core engine scaffold, database schema, SSH connection pool
- **Week 3-4:** Docker integration, host management API
- **Week 5-6:** Web UI shell, authentication, terminal WebSocket
- **Week 7-8:** AI chat integration, audit trail, Docker Compose, documentation

### Risk Factors

| Risk | Impact | Mitigation |
|------|--------|------------|
| russh compatibility issues | High | Evaluate SSH2 crate as fallback |
| WebSocket stability under load | Medium | Implement connection limits and heartbeat |
| AI response latency | Medium | Support streaming responses, local Ollama default |

---

## Phase 2: Modules (Month 3–4)

**Goal:** Extract core functionality into a pluggable module system, and deliver the first domain-specific modules (RCA and FinOps) that demonstrate the platform's extensibility.

### Deliverables

| # | Deliverable | Description | Success Metric |
|---|------------|-------------|----------------|
| 2.1 | Module SDK | Trait definitions, lifecycle management, dependency injection, configuration | All existing functionality runs as a module |
| 2.2 | mod-core | Extract host/SSH/Docker into standalone module | Zero regression from Phase 1 |
| 2.3 | mod-rca v0.1 | Automated root cause analysis from alerts and logs | Correctly identifies 70%+ of common issues |
| 2.4 | mod-rca + AI | LLM-powered analysis with log correlation and fix suggestions | < 30s end-to-end analysis time |
| 2.5 | mod-finops v0.1 | Cloud cost data collection (AWS), anomaly detection | Supports AWS Cost Explorer API |
| 2.6 | Event System | tokio broadcast channels, typed events, module pub/sub | < 1ms event propagation latency |
| 2.7 | Module Manager | Enable/disable/reload modules via API and UI | Hot-reload without core restart |
| 2.8 | Module UI | Module browser, configuration editor, health dashboard | Shows real-time module status |
| 2.9 | AI Tool Registry | Modules register tools with AI Gateway for function calling | LLM can invoke module tools automatically |
| 2.10 | Enhanced AI | Multi-turn conversations, context window management, tool execution | Handles 20+ turn conversations |

### Technical Milestones

- **Week 9-10:** Module SDK design, trait definitions, loader implementation
- **Week 11-12:** mod-core extraction, event bus, module manager
- **Week 13-14:** mod-rca development, LLM integration for analysis
- **Week 15-16:** mod-finops development, AI tool registry, module UI

### Success Criteria

- Existing Phase 1 functionality works identically when running as mod-core
- A third-party developer can create a module in < 2 hours using the SDK docs
- mod-rca correctly identifies "disk full" from logs within 15 seconds
- mod-finops detects a 2x cost spike within 1 hour of occurrence

---

## Phase 3: Enterprise (Month 5–6)

**Goal:** Add enterprise-grade features including security scanning, network topology visualization, ChatOps integration, and role-based access control.

### Deliverables

| # | Deliverable | Description | Success Metric |
|---|------------|-------------|----------------|
| 3.1 | mod-security v0.1 | SSH key auditing, CVE scanning, compliance checks (CIS benchmarks) | Scans 50 hosts in < 5 minutes |
| 3.2 | mod-topo v0.1 | Network topology discovery via SSH, dependency mapping | Auto-discovers 90%+ of network links |
| 3.3 | mod-topo UI | Interactive topology visualization with React Flow | Pan/zoom, search, filter, export |
| 3.4 | mod-chatops v0.1 | Slack/Discord webhook integration, incident notifications | Alerts delivered within 30 seconds |
| 3.5 | mod-chatops commands | Natural language commands via chat platforms | !ops restart nginx works in Slack |
| 3.6 | RBAC | Role-based access control, permission groups, resource scoping | Admin, Operator, Viewer roles |
| 3.7 | Secrets Vault | Encrypted credential storage, rotation reminders, audit | AES-256-GCM encryption at rest |
| 3.8 | Webhook System | Configurable webhooks for events, with retry logic | 99.9% delivery rate with retries |
| 3.9 | API Keys | Long-lived API keys for programmatic access, scoped permissions | Support read-only and admin keys |
| 3.10 | Email Notifications | Email alerts via SMTP, digest mode, templates | Works with major SMTP providers |

### Technical Milestones

- **Week 17-18:** mod-security and mod-chatops scaffolding
- **Week 19-20:** mod-topo discovery engine and visualization
- **Week 21-22:** RBAC system, secrets vault, webhook engine
- **Week 23-24:** Integration testing, performance tuning, documentation

### Success Criteria

- mod-topo can map a 3-tier web application topology automatically
- mod-chatops delivers Slack alerts within 30 seconds of trigger
- RBAC prevents non-admin users from executing destructive commands
- Security scan finds known CVEs within the last 30 days

---

## Phase 4: Ecosystem (Month 7+)

**Goal:** Build the ecosystem for community contributions, scale to multi-tenant deployments, and integrate with Kubernetes for cloud-native operations.

### Deliverables

| # | Deliverable | Description | Success Metric |
|---|------------|-------------|----------------|
| 4.1 | Module Marketplace | Community module registry, search, install, ratings | 10+ community modules published |
| 4.2 | K8s Operator | Kubernetes operator for OpsPilot deployment and management | Works on EKS, GKE, AKS |
| 4.3 | Multi-Tenant | Organization isolation, shared modules, cross-org visibility | Supports 50+ organizations per instance |
| 4.4 | Enterprise SSO | SAML/OIDC integration for enterprise identity providers | Works with Okta, Azure AD, Auth0 |
| 4.5 | Prometheus Exporter | Expose OpsPilot metrics as Prometheus endpoints | Standard /metrics endpoint |
| 4.6 | Terraform Provider | Manage OpsPilot resources via Terraform | Hosts, modules, alerts as code |
| 4.7 | CLI v2 | Full-featured CLI with interactive mode, shell completions | Feature parity with Web UI |
| 4.8 | Mobile-Responsive UI | Optimized dashboard for mobile/tablet access | Functional on iOS/Android browsers |
| 4.9 | Cost Optimization Engine | Automated recommendations with one-click apply | Saves 20%+ on cloud costs |
| 4.10 | Incident Timeline | Visual timeline of incidents with correlated events | Shows full RCA → fix → resolution |

### Success Criteria

- 100+ GitHub stars within 3 months of public launch
- 5+ community-contributed modules
- K8s operator deploys OpsPilot in < 5 minutes on any major cloud
- Multi-tenant deployment handles 1000+ hosts across 10 organizations

---

## Post-Phase 4 Ideas

Ideas being considered for future development:

- **AIOps Pipeline:** Custom ML models for anomaly detection, trained on your infrastructure data
- **Cost Optimization Engine:** Automated spot instance management, reserved instance recommendations
- **Compliance Dashboard:** Real-time compliance status across CIS, SOC2, HIPAA, PCI-DSS
- **GitOps Integration:** Deploy infrastructure changes via Git pull requests
- **Service Mesh Observability:** Istio/Linkerd integration for microservice monitoring
- **Chaos Engineering:** Controlled failure injection with automatic rollback
- **Custom Dashboard Builder:** Drag-and-drop dashboard creation with widget library
- **API Gateway:** Expose OpsPilot tools as a managed API for external consumers

---

## Versioning Strategy

| Version | Phase | Breaking Changes | Support |
|---------|-------|-----------------|---------|
| 0.1.x | Phase 1 (MVP) | N/A | Community |
| 0.2.x | Phase 2 (Modules) | Module API v1 | Community |
| 0.3.x | Phase 3 (Enterprise) | RBAC schema changes | Community + Enterprise |
| 1.0.0 | Phase 4 (Ecosystem) | Stable API contract | Long-term support |

**Semver Policy:**

- **MAJOR** (1.0.0, 2.0.0): Breaking changes to the Module SDK, REST API, or database schema.
- **MINOR** (0.1.0 → 0.2.0): New features, modules, or API endpoints. Backward-compatible.
- **PATCH** (0.1.0 → 0.1.1): Bug fixes, performance improvements, documentation updates.

---

## Contributing to the Roadmap

We welcome community input on priorities. To suggest changes:

1. Open a GitHub Issue with the `roadmap` label
2. Describe the feature or improvement
3. Include use cases and expected impact
4. Community discussion will determine if it fits a future phase

Roadmap items are reviewed monthly by the core team and updated based on community feedback and market needs.
