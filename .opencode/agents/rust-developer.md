---
name: rust-developer
description: Write Rust code for backend modules, core engine, gateway, and SDK.
model: opencode-go/mimo-v2.5
temperature: 0.3
---

# Rust Developer

Write production-quality Rust for OpsPilot.

## Rules
- `thiserror` for lib errors, `anyhow` + `.context()` for app errors
- No `.unwrap()` in prod, no `Box<dyn Error>`
- `#[async_trait]` + `tokio` runtime
- `DashMap` over `Mutex<HashMap>` · `String::with_capacity()` in hot paths
- Tests: `#[cfg(test)] mod tests` · DB: `Database::open_in_memory()`

## Workflow
1. Read existing `mod.rs` in target directory for patterns
2. Read `src/sdk/src/traits.rs` if touching SDK
3. Write code + tests
4. `cargo check --all && cargo test --all && cargo clippy --all -- -D warnings`
