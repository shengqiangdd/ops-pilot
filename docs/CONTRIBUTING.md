# Contributing to OpsPilot

Thank you for your interest in contributing to OpsPilot! This guide will help you get started with the development workflow, coding standards, and PR process.

---

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Module Development](#module-development)
- [Code Style](#code-style)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Issue Guidelines](#issue-guidelines)
- [Code of Conduct](#code-of-conduct)

---

## Getting Started

### Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| **Rust** | 1.75+ | Backend language |
| **Node.js** | 20+ | Frontend build |
| **Docker** | 24+ | Container testing |
| **Git** | 2.30+ | Version control |

### Quick Setup

```bash
# 1. Fork and clone
git clone https://github.com/YOUR_USERNAME/ops-pilot.git
cd ops-pilot

# 2. Install Rust toolchain
rustup default stable
rustup component add clippy rustfmt

# 3. Install frontend dependencies
cd frontend && npm install && cd ..

# 4. Copy environment file
cp .env.example .env

# 5. Start development services
docker compose up -d ollama  # Start local LLM (optional)

# 6. Build and run
cargo build
cargo run

# 7. Run tests
cargo test
cd frontend && npm test && cd ..
```

### Makefile Shortcuts

```bash
make dev          # Full development setup
make build        # Build all crates
make test         # Run all tests
make lint         # Run clippy + eslint
make fmt          # Format all code
make run          # Start the server
make docker-up    # Start with Docker Compose
make docker-down  # Stop Docker Compose
```

---

## Development Setup

### Rust Toolchain

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Ensure you have the latest stable
rustup update stable

# Install useful components
rustup component add clippy rustfmt rust-analyzer
```

### Frontend Toolchain

```bash
# Install Node.js (via nvm recommended)
nvm install 20
nvm use 20

# Install dependencies
cd frontend
npm install

# Start dev server (separate terminal)
npm run dev
```

### Database

OpsPilot uses SQLite by default (no setup required). For development with PostgreSQL:

```bash
# Start PostgreSQL via Docker
docker run -d --name ops-pilot-pg \
  -e POSTGRES_DB=ops_pilot \
  -e POSTGRES_USER=ops \
  -e POSTGRES_PASSWORD=dev \
  -p 5432:5432 \
  postgres:16

# Update .env
DATABASE_URL=postgres://ops:dev@localhost:5432/ops_pilot
```

### LLM Provider (Optional)

```bash
# Start Ollama for local AI
docker run -d --name ollama \
  -p 11434:11434 \
  -v ollama_data:/root/.ollama \
  ollama/ollama:latest

# Pull a model
docker exec ollama ollama pull qwen2.5:32b
```

### IDE Configuration

**VS Code (recommended):**

```json
// .vscode/settings.json
{
  "rust-analyzer.cargo.features": ["all"],
  "rust-analyzer.check.command": "clippy",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[typescript][typescriptreact]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  }
}
```

**Recommended Extensions:**
- rust-analyzer
- Even Better TOML
- ESLint
- Prettier
- Tailwind CSS IntelliSense

---

## Project Structure

```
ops-pilot/
├── Cargo.toml                 # Workspace root
├── src/
│   ├── core/                  # Core engine crate
│   │   ├── src/
│   │   │   ├── main.rs        # Entry point
│   │   │   ├── server.rs      # HTTP server (axum)
│   │   │   ├── db.rs          # Database layer
│   │   │   ├── auth.rs        # JWT authentication
│   │   │   ├── config.rs      # Configuration loading
│   │   │   └── ...
│   │   └── Cargo.toml
│   │
│   ├── gateway/               # AI Gateway crate
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── llm.rs         # LLM provider adapters
│   │   │   ├── agent.rs       # Agent orchestration
│   │   │   ├── tools.rs       # Tool registry
│   │   │   └── mcp.rs         # MCP protocol handler
│   │   └── Cargo.toml
│   │
│   ├── sdk/                   # Module SDK crate
│   │   ├── src/
│   │   │   ├── lib.rs         # Re-exports
│   │   │   ├── traits.rs      # OpsModule trait
│   │   │   ├── context.rs     # ModuleContext
│   │   │   ├── events.rs      # OpsEvent enum
│   │   │   ├── tools.rs       # ToolDefinition
│   │   │   ├── health.rs      # HealthStatus
│   │   │   └── error.rs       # ModuleError
│   │   └── Cargo.toml
│   │
│   └── modules/               # Pluggable modules
│       ├── mod-core/          # Host, SSH, Docker management
│       ├── mod-rca/           # Root cause analysis
│       ├── mod-finops/        # Cost optimization
│       ├── mod-security/      # Security scanning
│       ├── mod-topo/          # Topology visualization
│       └── mod-chatops/       # Chat platform integration
│
├── frontend/                  # React frontend
│   ├── src/
│   │   ├── components/        # Reusable UI components
│   │   ├── pages/             # Route-level components
│   │   ├── stores/            # Zustand stores
│   │   ├── hooks/             # Custom React hooks
│   │   ├── api/               # API client functions
│   │   ├── types/             # TypeScript type definitions
│   │   └── utils/             # Utility functions
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   └── tailwind.config.js
│
├── docs/                      # Documentation
│   ├── ARCHITECTURE.md
│   ├── MODULE_SDK.md
│   ├── API_REFERENCE.md
│   ├── ROADMAP.md
│   └── CONTRIBUTING.md
│
├── .github/                   # GitHub configuration
│   ├── ISSUE_TEMPLATE/
│   ├── workflows/
│   └── PULL_REQUEST_TEMPLATE.md
│
├── docker-compose.yml         # Docker deployment
├── Dockerfile                 # Container build
├── Makefile                   # Development shortcuts
└── .env.example               # Environment template
```

### Crate Dependencies

```
ops-pilot (core)
├── ops-pilot-sdk     (trait definitions, no core dependency)
├── ops-pilot-gateway (AI integration, depends on sdk)
├── mod-core          (depends on sdk)
├── mod-rca           (depends on sdk, mod-core)
├── mod-finops        (depends on sdk, mod-core)
├── mod-security      (depends on sdk, mod-core)
├── mod-topo          (depends on sdk, mod-core)
└── mod-chatops       (depends on sdk, mod-core)
```

**Important:** The `sdk` crate must NOT depend on `core`. This prevents circular dependencies and allows modules to be developed independently.

---

## Module Development

### Creating a New Module

```bash
# 1. Create the crate
cargo new src/modules/mod-my-feature --lib

# 2. Add to workspace Cargo.toml
# (already done if using the workspace glob)

# 3. Create module.toml manifest
cat > src/modules/mod-my-feature/module.toml << 'EOF'
[package]
name = "mod-my-feature"
version = "0.1.0"
description = "My custom module"

[module]
id = "mod-my-feature"
category = "custom"
crate = "mod_my_feature"
dependencies = []
EOF

# 4. Implement the trait
# See docs/MODULE_SDK.md for the complete specification
```

### Module Checklist

Before submitting a new module:

- [ ] Implements `OpsModule` trait with all required methods
- [ ] Includes `module.toml` manifest
- [ ] Has unit tests covering all tools
- [ ] Has integration tests (at least one end-to-end test)
- [ ] Documentation comments on all public items
- [ ] Follows code style (clippy, rustfmt)
- [ ] No hardcoded configuration values
- [ ] Handles errors gracefully (no unwrap/expect in production paths)
- [ ] Health check returns meaningful status

---

## Code Style

### Rust

We use default `rustfmt` with `clippy` in strict mode.

```toml
# rustfmt.toml (if customizing)
edition = "2021"
max_width = 100
tab_spaces = 4
```

**Rules:**

- Run `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` with zero warnings
- Use `thiserror` for error types, `anyhow` for application errors
- Prefer `Result<T, E>` over panicking
- Document public items with `///` doc comments
- Use `#[derive]` where possible instead of manual implementations

**Naming:**

```rust
// Types: PascalCase
pub struct HostConnection { }

// Functions/methods: snake_case
pub async fn connect_host() -> Result<()> { }

// Constants: SCREAMING_SNAKE_CASE
const MAX_CONNECTIONS: usize = 100;

// Modules: snake_case
mod connection_pool { }
```

### TypeScript / Frontend

```json
// .eslintrc.json (simplified)
{
  "extends": [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended"
  ],
  "rules": {
    "@typescript-eslint/no-unused-vars": ["error", { "argsIgnorePattern": "^_" }],
    "@typescript-eslint/explicit-function-return-type": "off",
    "react/react-in-jsx-scope": "off"
  }
}
```

**Rules:**

- Run `npm run lint` before committing
- Use functional components with hooks
- TypeScript strict mode — no `any` types
- Props interfaces exported from the component file
- Test files co-located with components: `Component.test.tsx`

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(mod-rca): add log correlation for multi-host incidents

Fixes #123

- Correlate logs across hosts within a 5-minute window
- Use AI to identify patterns in correlated logs
- Add unit tests for correlation engine
```

**Format:**

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `style` | Code style (formatting, no logic change) |
| `refactor` | Code refactoring (no feature change) |
| `test` | Adding or updating tests |
| `chore` | Build process, dependencies, CI |
| `perf` | Performance improvement |

**Scopes:**

`core`, `sdk`, `gateway`, `mod-core`, `mod-rca`, `mod-finops`, `mod-security`, `mod-topo`, `mod-chatops`, `ui`, `api`, `docs`, `ci`

---

## Testing

### Backend Tests

```bash
# Unit tests
cargo test

# Unit tests with output
cargo test -- --nocapture

# Specific test
cargo test test_ssh_connection

# Integration tests
cargo test --test integration

# With coverage
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

**Test Conventions:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ops_pilot_sdk::test::MockModuleContext;

    #[tokio::test]
    async fn test_tool_executes_successfully() {
        let module = MyModule::new();
        let ctx = MockModuleContext::new();

        let result = module
            .execute(&ctx, "my_tool", serde_json::json!({"key": "value"}))
            .await
            .unwrap();

        assert_eq!(result["status"], "ok");
    }

    #[tokio::test]
    async fn test_tool_rejects_invalid_input() {
        let module = MyModule::new();
        let ctx = MockModuleContext::new();

        let result = module
            .execute(&ctx, "my_tool", serde_json::json!({}))
            .await;

        assert!(result.is_err());
    }
}
```

### Frontend Tests

```bash
# Run all tests
npm test

# Watch mode
npm test -- --watch

# Coverage
npm test -- --coverage
```

**Test Conventions:**

```tsx
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { HostCard } from './HostCard';

describe('HostCard', () => {
  it('renders host name and status', () => {
    render(
      <HostCard
        host={{
          id: 'host_001',
          name: 'prod-web-01',
          status: 'online',
          ip: '192.168.1.100',
        }}
      />
    );

    expect(screen.getByText('prod-web-01')).toBeInTheDocument();
    expect(screen.getByText('online')).toBeInTheDocument();
  });
});
```

### Test Coverage Goals

| Component | Minimum Coverage |
|-----------|-----------------|
| Core Engine | 80% |
| Module SDK | 90% |
| Modules | 85% |
| AI Gateway | 75% |
| Frontend Components | 70% |
| Frontend Hooks/Utils | 85% |

---

## Pull Request Process

### Before Submitting

1. **Sync with main:**
   ```bash
   git fetch origin
   git rebase origin/main
   ```

2. **Run all checks:**
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   cd frontend && npm run lint && npm test && cd ..
   ```

3. **Update documentation** if your change affects public APIs or user-facing features.

### PR Template

```markdown
## Description

Brief description of what this PR does.

## Type of Change

- [ ] Bug fix (non-breaking change)
- [ ] New feature (non-breaking change)
- [ ] Breaking change (fix or feature causing existing functionality to change)
- [ ] Documentation update
- [ ] Refactoring (no functional changes)

## Testing

Describe the tests you ran and how to reproduce them.

- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing performed (describe below)

## Checklist

- [ ] Code follows project style guidelines
- [ ] Self-reviewed the code
- [ ] Comments added for complex logic
- [ ] Documentation updated
- [ ] No new warnings from `cargo clippy`
- [ ] Tests added that prove the fix/feature works
- [ ] All existing tests pass
```

### Review Process

1. **Automated checks** must pass (CI: clippy, tests, build)
2. **At least one review** from a maintainer
3. **No unresolved conversations**
4. **Squash and merge** for clean history

### Branch Naming

```
feature/mod-rca-log-correlation
fix/ssh-connection-timeout
docs/api-reference-update
chore/upgrade-axum-0.8
```

---

## Issue Guidelines

### Bug Reports

Use the bug report template. Include:

- **Environment:** OS, Rust version, Node version
- **Steps to reproduce:** Minimal, clear steps
- **Expected behavior:** What should happen
- **Actual behavior:** What actually happens
- **Logs:** Relevant error output (use code blocks)
- **Screenshots:** If applicable

### Feature Requests

Use the feature request template. Include:

- **Problem:** What problem does this solve?
- **Solution:** Proposed solution
- **Alternatives:** Other approaches considered
- **Use case:** Real-world scenario

### Labels

| Label | Description |
|-------|-------------|
| `bug` | Something isn't working |
| `enhancement` | New feature or improvement |
| `documentation` | Documentation needed |
| `good first issue` | Good for newcomers |
| `help wanted` | Community contribution welcome |
| `priority: high` | Critical issue |
| `module: *` | Module-specific issue |
| `roadmap` | Discussed for future roadmap |

---

## Code of Conduct

We follow the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/1/code_of_conduct/).

### Summary

- **Be respectful** — Treat everyone with respect. No harassment, discrimination, or personal attacks.
- **Be constructive** — Provide helpful feedback. Focus on the issue, not the person.
- **Be inclusive** — Welcome newcomers. Help them learn and contribute.
- **Be professional** — Keep discussions focused on the project.

### Enforcement

Reports of unacceptable behavior can be made to the project maintainers. All reports will be reviewed and investigated promptly.

---

## Getting Help

- **GitHub Discussions** — For questions and general discussion
- **GitHub Issues** — For bug reports and feature requests
- **Discord** — Real-time chat with the community (link in README)
- **Module SDK Docs** — [docs/MODULE_SDK.md](MODULE_SDK.md)

Thank you for contributing to OpsPilot! 🚀
