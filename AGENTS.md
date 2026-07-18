# OpsPilot — Agent Instructions

## Project

AI-powered modular infrastructure operations platform. Pluggable modules via `OpsModule` trait.

**Stack:** Rust (axum/tokio/russh/bollard/sqlx) · TypeScript/React 19/Zustand · SQLite

## Architecture (6 layers)

```
Web UI → AI Gateway → Module SDK → Modules → Core Engine → DB
```

- `src/sdk/` — `OpsModule` trait, `ModuleContext`, `OpsEvent`
- `src/core/` — SSH, Docker, Monitor, EventBus, Auth, DB
- `src/gateway/` — REST routes, LLM, Agent, ToolRegistry
- `src/modules/` — mod-core, mod-rca, mod-finops, ...

## Module Contract

Every module implements `OpsModule` (see `src/sdk/src/traits.rs`).

## Rules

### Rust
- Library errors: `thiserror` · App errors: `anyhow` + `.context()`
- No `.unwrap()` in prod code · No `Box<dyn Error>`
- `#[async_trait]` for async traits · `tokio` runtime
- `DashMap` over `Mutex<HashMap>` · `String::with_capacity()` in hot paths
- Tests: `#[cfg(test)] mod tests` · DB: `Database::open_in_memory()`

### TypeScript
- `React.FC<Props>` with explicit types · No `any` — use `unknown`
- Zustand for shared state · No inline styles — Tailwind
- File < 300 lines · No `// @ts-ignore`

### Testing
- `cargo test` / `npx vitest run` must pass before commit
- `cargo clippy --all -- -D warnings` · `npx tsc --noEmit`

### Git
- Format: `type(scope): description` (feat/fix/docs/refactor/test)
- No secrets in logs · Parameterized SQL only

### Before Committing
1. `cargo check --all` 2. `cargo test --all` 3. `cargo clippy --all`
