# =============================================================================
# OpsPilot — Multi-stage Docker build
#
# Stage 1 : rust-builder   — compile the Rust backend binary
# Stage 2 : node-builder   — build the frontend static assets
# Stage 3 : runtime        — Debian slim with only the essentials
# =============================================================================

# ── Stage 1: Rust builder ──────────────────────────────────────────────────
FROM rust:1.81-slim-bookworm AS rust-builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy only manifests first to maximize Docker layer caching
COPY backend/Cargo.toml backend/Cargo.lock ./backend/
COPY backend/core/   core/
COPY backend/sdk/    sdk/

# Create dummy main.rs so that `cargo chef` can cache dependencies
# Run cargo fetch to download & cache all dependencies
RUN mkdir -p backend/gateway/src backend/modules/mod-core/src \
    backend/modules/mod-rca/src backend/modules/mod-security/src \
    backend/modules/mod-config/src backend/modules/mod-webhook/src \
    backend/modules/mod-scheduler/src backend/modules/mod-filesync/src \
    backend/modules/mod-advisor/src backend/modules/mod-topo/src \
    backend/modules/mod-monitor/src backend/modules/mod-alert-escalation/src \
    backend/modules/mod-fim/src backend/modules/mod-baseline/src \
    backend/modules/mod-runbook/src backend/modules/mod-knowledge/src \
    && echo "fn main() {}" > backend/gateway/src/main.rs \
    && for d in backend/modules/mod-*/src; do echo "" > "$d/lib.rs"; done \
    && echo "" > backend/core/src/lib.rs \
    && echo "" > backend/sdk/src/lib.rs

# Build dependencies only (cached layer unless Cargo.lock changes)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cd backend && cargo build --release --bin ops-pilot-gateway 2>/dev/null || true

# Now copy the actual source code and rebuild
COPY backend/ ./backend/

# Touch main.rs to force rebuild of the gateway binary
RUN touch backend/gateway/src/main.rs

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/backend/target \
    cd backend && cargo build --release --bin ops-pilot-gateway && \
    cp target/release/ops-pilot-gateway /usr/local/bin/ops-pilot-gateway && \
    strip /usr/local/bin/ops-pilot-gateway

# ── Stage 2: Node builder ──────────────────────────────────────────────────
FROM node:20-alpine AS node-builder

WORKDIR /build/frontend

# Copy package files first
COPY frontend/package*.json ./
RUN npm ci --ignore-scripts && npm cache clean --force

# Copy source and build
COPY frontend/ ./
RUN npm run build && \
    # Remove node_modules after build to save space in layer
    rm -rf node_modules/

# ── Stage 3: Runtime ──────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

# Install only runtime dependencies (no build tools)
RUN apt-get update && apt-get install -y --no-install-recommends \
    nginx-light \
    libssl3 \
    ca-certificates \
    curl \
    tini \
    && rm -rf /var/lib/apt/lists/* && \
    # Remove default nginx site config
    rm -f /etc/nginx/sites-enabled/default

# Create a non-root user for the application
RUN groupadd --system --gid 1001 opspilot && \
    useradd --system --uid 1001 --gid opspilot --no-create-home --home-dir /app opspilot

# Copy artifacts from builder stages
COPY --from=node-builder --chown=opspilot:opspilot /build/frontend/dist /usr/share/nginx/html
COPY --from=rust-builder --chown=opspilot:opspilot /usr/local/bin/ops-pilot-gateway /usr/local/bin/
COPY nginx.conf /etc/nginx/nginx.conf

# Create data directory with proper ownership
RUN mkdir -p /app/data && chown -R opspilot:opspilot /app/data

# Expose HTTP port (nginx reverse-proxies to backend on 3001)
EXPOSE 80

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD curl -sf http://localhost/api/v1/health || exit 1

# Use tini as init for proper signal handling and zombie reaping
ENTRYPOINT ["/usr/bin/tini", "--"]

# Run as non-root user
USER opspilot

# Start nginx in foreground and the gateway binary
CMD ["sh", "-c", "nginx -g 'daemon off;' & exec ops-pilot-gateway"]
