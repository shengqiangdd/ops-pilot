# ── Stage 1: Frontend build ──────────────────────────────────────────────
FROM node:20-slim AS frontend-builder

WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci --ignore-scripts
COPY frontend/ ./
RUN npm run build

# ── Stage 2a: Cargo dependency cache ────────────────────────────────────
FROM rust:1.82-slim-bookworm AS cargo-cache

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy only manifests and lockfile for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY src/core/Cargo.toml src/core/Cargo.toml
COPY src/gateway/Cargo.toml src/gateway/Cargo.toml
COPY src/sdk/Cargo.toml src/sdk/Cargo.toml
COPY src/modules/mod-core/Cargo.toml src/modules/mod-core/Cargo.toml
COPY src/modules/mod-rca/Cargo.toml src/modules/mod-rca/Cargo.toml
COPY src/modules/mod-security/Cargo.toml src/modules/mod-security/Cargo.toml

# Create dummy source files for cargo to resolve the workspace
RUN mkdir -p src/core/src src/gateway/src src/sdk/src src/modules/mod-core/src src/modules/mod-rca/src src/modules/mod-security/src \
    && echo 'pub mod host; pub mod auth; pub mod db; pub mod crypto; pub mod ssh; pub mod docker; pub mod monitor; pub mod audit; pub mod vault; pub mod alert;' > src/core/src/lib.rs \
    && touch src/core/src/host.rs src/core/src/auth.rs src/core/src/db.rs src/core/src/crypto.rs src/core/src/ssh.rs src/core/src/docker.rs src/core/src/monitor.rs src/core/src/audit.rs src/core/src/vault.rs src/core/src/alert.rs \
    && echo 'pub mod agent; pub mod llm; pub mod middleware; pub mod routes; pub mod tools; pub mod terminal;' > src/gateway/src/lib.rs \
    && mkdir -p src/gateway/src/llm src/gateway/src/middleware src/gateway/src/routes src/gateway/src/tools \
    && touch src/gateway/src/llm/mod.rs src/gateway/src/middleware/mod.rs src/gateway/src/routes/mod.rs src/gateway/src/tools/mod.rs src/gateway/src/agent.rs src/gateway/src/terminal.rs \
    && echo 'pub mod context; pub mod events; pub mod loader; pub mod traits;' > src/sdk/src/lib.rs \
    && touch src/sdk/src/context.rs src/sdk/src/events.rs src/sdk/src/loader.rs src/sdk/src/traits.rs \
    && echo 'pub mod ssh; pub mod docker; pub mod host; pub mod monitor;' > src/modules/mod-core/src/lib.rs \
    && touch src/modules/mod-core/src/ssh.rs src/modules/mod-core/src/docker.rs src/modules/mod-core/src/host.rs src/modules/mod-core/src/monitor.rs \
    && echo 'pub mod analyzer; pub mod llm_analyzer; pub mod rules;' > src/modules/mod-rca/src/lib.rs \
    && touch src/modules/mod-rca/src/analyzer.rs src/modules/mod-rca/src/llm_analyzer.rs src/modules/mod-rca/src/rules.rs \
    && echo 'pub mod engine; pub mod llm_scanner; pub mod rules;' > src/modules/mod-security/src/lib.rs \
    && touch src/modules/mod-security/src/engine.rs src/modules/mod-security/src/llm_scanner.rs src/modules/mod-security/src/rules.rs

# Build dependencies only (will be cached if Cargo.toml/Cargo.lock unchanged)
RUN cargo build --release --bin ops-pilot 2>/dev/null || true

# ── Stage 2b: Rust builder ──────────────────────────────────────────────
FROM rust:1.82-slim-bookworm AS rust-builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY --from=cargo-cache /app/target target/
COPY src/ src/

RUN cargo build --release --bin ops-pilot

# ── Stage 3: Production runtime ─────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false ops-pilot

WORKDIR /app

COPY --from=rust-builder /app/target/release/ops-pilot /app/ops-pilot
COPY --from=frontend-builder /app/frontend/dist /app/static
COPY src/core/migrations /app/migrations

RUN mkdir -p /app/data && chown -R ops-pilot:ops-pilot /app

USER ops-pilot

ENV DATABASE_URL=sqlite:///app/data/ops-pilot.db
ENV RUST_LOG=ops_pilot=info,tower_http=info
ENV STATIC_DIR=/app/static
ENV LISTEN_ADDR=0.0.0.0:3001
ENV TZ=UTC

EXPOSE 3001

HEALTHCHECK --interval=30s --timeout=10s --retries=3 --start-period=10s \
    CMD curl -f http://localhost:3001/api/v1/health || exit 1

ENTRYPOINT ["/app/ops-pilot"]
