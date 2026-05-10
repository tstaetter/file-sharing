# syntax=docker/dockerfile:1
# Dockerfile for Koyeb deployment of the cleanup-orphaned-uploads worker.
#
# This is a batch job — it runs once, scans R2 for orphaned multipart
# uploads older than 6 hours, aborts them, and exits.
# Deploy on Koyeb as a Worker service type with a cron schedule (e.g. hourly).
#
# Key design decisions:
# - Multi-stage build: compile in rust image, run in slim debian
# - Dependency caching: Cargo.toml/Cargo.lock copied first, dummy src built
# - Fingerprint cleanup before real build (avoids "cannot find struct" errors)
# - tini as init (PID 1) for proper signal forwarding
# - Non-root user for security
# - No PORT, no health checks — workers don't serve HTTP traffic

# ── Stage 1: Build ──────────────────────────────────────────────────────────
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Copy manifests first for dependency caching.
# This layer only rebuilds when Cargo.toml or Cargo.lock change.
COPY Cargo.toml Cargo.lock ./

# Create dummy source files to compile and cache dependencies.
# This ensures dependencies are only rebuilt when Cargo.toml or Cargo.lock change.
RUN mkdir -p src && \
    echo 'fn main() {}' > src/main.rs && \
    echo '' > src/lib.rs

RUN cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the real binary.
# Remove the crate's own fingerprint artifacts from the dummy build.
# Without this, cargo may reuse the cached (empty) library compiled from
# the dummy lib.rs, causing "cannot find struct" errors.
RUN rm -rf target/release/.fingerprint/cleanup-orphaned-uploads-* && \
    cargo build --release && \
    cp target/release/cleanup-orphaned-uploads /usr/local/bin/cleanup-orphaned-uploads

# ── Stage 2: Runtime ───────────────────────────────────────────────────────
FROM debian:bookworm-slim

# Install runtime dependencies:
# - ca-certificates: TLS verification for R2 HTTPS connections
# - tini: minimal init process for proper signal handling
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        tini && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd -r app && useradd -r -g app app

COPY --from=builder /usr/local/bin/cleanup-orphaned-uploads /usr/local/bin/cleanup-orphaned-uploads

# OCI labels for container metadata
LABEL org.opencontainers.image.title="cleanup-orphaned-uploads" \
      org.opencontainers.image.description="Background worker that aborts orphaned multipart uploads older than 6 hours" \
      org.opencontainers.image.source="https://github.com/ttstt/file-sharing"

USER app

# tini as the entrypoint ensures:
# - SIGTERM from Koyeb/Docker is properly forwarded to the worker process
# - The worker can shut down gracefully if it receives a signal mid-run
ENTRYPOINT ["/usr/bin/tini", "--"]

# Run the cleanup worker once and exit.
# Set RUST_LOG=info on Koyeb to see progress logs.
CMD ["/usr/local/bin/cleanup-orphaned-uploads"]
