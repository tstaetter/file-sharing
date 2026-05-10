# syntax=docker/dockerfile:1
# Dockerfile for Koyeb deployment.
# Build: docker build -t file-sharing-backend .
# Koyeb auto-detects Dockerfiles and sets PORT env var.

# Stage 1: Build
FROM rust:1.92-slim-bookworm AS builder

WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source files to compile and cache dependencies
RUN mkdir src && \
    echo 'fn main() {}' > src/main.rs && \
    echo '' > src/lib.rs && \
    mkdir src/handlers && \
    echo '' > src/handlers/mod.rs

# Build dependencies only — cached unless Cargo.toml/Cargo.lock change
RUN cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the real binary
RUN cargo build --release && \
    cp target/release/backend /usr/local/bin/backend

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies: ca-certificates for TLS, curl for health checks
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd -r app && useradd -r -g app app

COPY --from=builder /usr/local/bin/backend /usr/local/bin/backend

# OCI labels for Koyeb and container metadata
LABEL org.opencontainers.image.title="file-sharing-backend" \
      org.opencontainers.image.description="Backend API for end-to-end encrypted file sharing" \
      org.opencontainers.image.source="https://github.com/ttstt/file-sharing"

USER app

# Koyeb sets PORT at runtime; EXPOSE is informational
EXPOSE 8000

# Health check using the /health endpoint.
# Koyeb also performs its own TCP health checks by default.
# Configure an HTTP health check in Koyeb's dashboard pointing to /health for faster detection.
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:"${PORT:-8000}"/health || exit 1

# Use exec form for proper signal forwarding (SIGTERM → tokio graceful shutdown)
CMD ["backend"]
