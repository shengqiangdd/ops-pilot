# Stage 1: Build frontend
FROM node:20-alpine AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# Stage 2: Build backend
FROM rust:1.81-slim-bookworm AS backend-builder
WORKDIR /app/backend
COPY backend/ ./
RUN apt-get update && apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/* && \
    cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y nginx libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy frontend build
COPY --from=frontend-builder /app/frontend/dist /usr/share/nginx/html

# Copy backend binary
COPY --from=backend-builder /app/backend/target/release/ops-pilot-gateway /usr/local/bin/

# Copy nginx config
COPY nginx.conf /etc/nginx/nginx.conf

# Create data directory
RUN mkdir -p /app/data

EXPOSE 80 443

CMD ["sh", "-c", "service nginx start && ops-pilot-gateway"]
