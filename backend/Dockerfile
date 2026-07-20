# ── OpsPilot Docker Image ──────────────────────────────────────────────
# In CI, this expects the pre-built binary and frontend dist to be
# mounted/downloaded into the build context as:
#   backend/target/release/ops-pilot   (from the "rust" CI job)
#   frontend/dist/                     (from the "frontend" CI job)
#
# For local builds, run:
#   cd backend && cargo build --release --bin ops-pilot
#   cd frontend && npm run build
#   docker build -f backend/Dockerfile .

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false ops-pilot

WORKDIR /app

COPY backend/target/release/ops-pilot /app/ops-pilot
COPY frontend/dist /app/static
COPY backend/src/core/migrations /app/migrations

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
