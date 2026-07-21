# Stage 1: Build frontend
FROM node:20-alpine AS frontend-builder
WORKDIR /build/frontend
COPY frontend/package*.json ./
RUN npm ci --ignore-scripts && npm cache clean --force
COPY frontend/ ./
RUN npm run build

# Stage 2: Build backend
FROM rust:1.81-slim-bookworm AS backend-builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /build/backend
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/core/ core/
COPY backend/gateway/ gateway/
COPY backend/sdk/ sdk/
COPY backend/modules/ modules/
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/backend/target \
    cargo build --release --bin ops-pilot-gateway && \
    cp target/release/ops-pilot-gateway /app/ops-pilot-gateway

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    nginx libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy frontend build
COPY --from=frontend-builder /build/frontend/dist /usr/share/nginx/html

# Copy backend binary
COPY --from=backend-builder /app/ops-pilot-gateway /usr/local/bin/

# Copy nginx config
COPY nginx.conf /etc/nginx/nginx.conf

# Create data directory
RUN mkdir -p /app/data

EXPOSE 80

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost/ || exit 1

CMD ["sh", "-c", "service nginx start && ops-pilot-gateway"]
