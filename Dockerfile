# ── Stage 1: Frontend build ──────────────────────────────────────────────
FROM node:20-slim AS frontend-builder

WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci --ignore-scripts
COPY frontend/ ./
RUN npm run build

# ── Stage 2: Rust builder ───────────────────────────────────────────────
FROM rust:1.82-slim-bookworm AS rust-builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

# Build the binary
RUN cargo build --release --bin ops-pilot

# ── Stage 3: Production runtime ─────────────────────────────────────────
FROM debian:bookworm-slim AS production

RUN apt-get update && apt-get install -y \
    curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false ops-pilot

WORKDIR /app

# Copy binary
COPY --from=rust-builder /app/target/release/ops-pilot /app/ops-pilot

# Copy frontend assets
COPY --from=frontend-builder /app/frontend/dist /app/static

# Copy migrations
COPY src/core/migrations /app/migrations

RUN mkdir -p /app/data && chown -R ops-pilot:ops-pilot /app

USER ops-pilot

ENV DATABASE_URL=sqlite:///app/data/ops-pilot.db
ENV RUST_LOG=ops_pilot=info,tower_http=info
ENV TZ=UTC

EXPOSE 3000 8080

HEALTHCHECK --interval=30s --timeout=10s --retries=3 --start-period=10s \
    CMD curl -f http://localhost:3000/api/health || exit 1

ENTRYPOINT ["/app/ops-pilot"]
