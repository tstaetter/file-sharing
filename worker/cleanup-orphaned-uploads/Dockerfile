# syntax=docker/dockerfile:1
# Dockerfile for Koyeb deployment of the cleanup-orphaned-uploads worker.
#
# Architecture: Long-running container using Supercronic to schedule the
# cleanup binary every 30 minutes. Supercronic runs as PID 1, handles signals
# properly, and logs job output to stdout for Koyeb log collection.
#
# Key design decisions:
# - Multi-stage build: compile in rust image, run in slim debian
# - Dependency caching: Cargo.toml/Cargo.lock copied first, dummy src built
# - Fingerprint cleanup before real build (avoids "cannot find struct" errors)
# - Supercronic as PID 1 for in-container cron scheduling
# - Non-root user for security
# - No PORT, no health checks — this is a background worker

# ── Stage 1: Build ──────────────────────────────────────────────────────────
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Copy manifests first for dependency caching.
# This layer only rebuilds when Cargo.toml or Cargo.lock change.
COPY Cargo.toml Cargo.lock ./

# Create dummy source files to compile and cache dependencies.
# This ensures all third-party crates are compiled and cached in a
# separate layer, only invalidated when Cargo.toml or Cargo.lock change.
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
# The wildcard covers all fingerprint files for this crate.
RUN rm -rf target/release/.fingerprint/cleanup-orphaned-uploads-* && \
    cargo build --release && \
    cp target/release/cleanup-orphaned-uploads /usr/local/bin/cleanup-orphaned-uploads

# ── Stage 2: Runtime ───────────────────────────────────────────────────────
FROM debian:bookworm-slim

# Supercronic version — check https://github.com/aptible/supercronic/releases
# for the latest release and updated SHA1SUM.
ARG SUPERCRONIC_VERSION=v0.2.45
ARG SUPERCRONIC_SHA1SUM=e894b193bea75a5ee644e700c59e30eedc804cf7

WORKDIR /app

# Install runtime dependencies:
# - ca-certificates: TLS verification for R2 HTTPS connections
# - curl: used to download Supercronic (removed after to keep image slim)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        curl && \
    \
    # Download and verify Supercronic
    curl -fsSL "https://github.com/aptible/supercronic/releases/download/${SUPERCRONIC_VERSION}/supercronic-linux-amd64" \
        -o /usr/local/bin/supercronic && \
    echo "${SUPERCRONIC_SHA1SUM}  /usr/local/bin/supercronic" | sha1sum -c - && \
    chmod +x /usr/local/bin/supercronic && \
    \
    # Clean up: remove curl (no longer needed), create non-root user
    apt-get remove -y curl && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd -r app && useradd -r -g app app

# Copy the compiled worker binary from the builder stage
COPY --from=builder /usr/local/bin/cleanup-orphaned-uploads /usr/local/bin/cleanup-orphaned-uploads

# Create the crontab file that Supercronic will read.
# Runs every 30 minutes.
# The worker reads RUST_LOG from the environment; set RUST_LOG=info on
# Koyeb to see progress output for each run.
COPY --chown=app:app <<CRONTAB /etc/crontab
*/30 * * * * /usr/local/bin/cleanup-orphaned-uploads
CRONTAB

# OCI labels for container metadata
LABEL org.opencontainers.image.title="cleanup-orphaned-uploads" \
      org.opencontainers.image.description="Background worker that aborts orphaned multipart uploads older than 6 hours. Runs every 30 minutes via Supercronic." \
      org.opencontainers.image.source="https://github.com/ttstt/file-sharing"

USER app

# Supercronic runs as PID 1:
# - Handles SIGTERM/SIGINT for graceful container shutdown
# - Reads /etc/crontab and executes jobs on schedule
# - Forwards job stdout/stderr to the container's stdout for log collection
ENTRYPOINT ["/usr/local/bin/supercronic", "/etc/crontab"]
