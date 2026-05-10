# syntax=docker/dockerfile:1
# Dockerfile for Koyeb deployment.
# Build: docker build -t file-sharing-backend .
# Koyeb auto-detects Dockerfiles and sets PORT env var.
#
# Key design decisions:
# - tini as init (PID 1) for proper signal forwarding and zombie reaping
# - Multi-stage build: compile in rust image, run in slim debian
# - Dependency caching: Cargo.toml/Cargo.lock copied first, dummy src built
# - No Docker HEALTHCHECK — Koyeb performs its own TCP health checks
#   Configure an HTTP health check in Koyeb dashboard pointing to /health

# Stage 1: Build
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source files to compile and cache dependencies.
# This ensures dependencies are only rebuilt when Cargo.toml or Cargo.lock change.
RUN mkdir -p src/handlers && \
    echo 'fn main() {}' > src/main.rs && \
    echo '' > src/lib.rs && \
    echo '' > src/handlers/mod.rs

RUN cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the real binary.
# Remove the crate's own fingerprint artifacts from the dummy build.
# Without this, cargo may reuse the cached (empty) library compiled from
# the dummy lib.rs, causing "cannot find struct AppState" errors.
RUN rm -rf target/release/.fingerprint/backend-* && \
    cargo build --release && \
    cp target/release/backend /usr/local/bin/backend

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies:
# - ca-certificates: TLS verification for R2 HTTPS connections
# - tini: minimal init process for proper signal handling (SIGTERM → tokio)
# - curl: useful for debugging (optional, can be removed to shrink image)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        tini && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd -r app && useradd -r -g app app

COPY --from=builder /usr/local/bin/backend /usr/local/bin/backend

# OCI labels for container metadata
LABEL org.opencontainers.image.title="file-sharing-backend" \
      org.opencontainers.image.description="Backend API for end-to-end encrypted file sharing" \
      org.opencontainers.image.source="https://github.com/ttstt/file-sharing"

USER app

# Koyeb sets PORT at runtime; EXPOSE is informational only
EXPOSE 8000

# tini as the entrypoint ensures:
# - SIGTERM from Koyeb/Docker is properly forwarded to the backend process
# - Zombie processes are reaped (not critical here but good practice)
# - The container shuts down cleanly instead of exiting code 0 instantly
ENTRYPOINT ["/usr/bin/tini", "--"]

# Use exec form with tini — backend runs as a child of tini (PID 1)
CMD ["/usr/local/bin/backend"]
