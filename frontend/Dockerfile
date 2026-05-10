# syntax=docker/dockerfile:1
# Dockerfile for Koyeb deployment of the SvelteKit frontend.
#
# Key design decisions:
# - Multi-stage build: compile in node image, run in slim node image
# - tini as init (PID 1) for proper signal forwarding (SIGTERM → graceful shutdown)
# - No Docker HEALTHCHECK — Koyeb performs its own TCP health checks.
#   Configure an HTTP health check in Koyeb dashboard pointing to /health.
# - PORT is read from the environment by adapter-node (Koyeb sets this).
# - ORIGIN should be set in Koyeb's env vars to the public URL of the app.
# - PUBLIC_API_PREFIX is baked in at build time via --build-arg.
# - node_modules is NOT copied to runtime — the adapter-node build output
#   in build/ is self-contained.

# ── Stage 1: Build ──────────────────────────────────────────────────────────
FROM node:22-slim AS builder

WORKDIR /app

# Copy package manifests first for dependency caching.
# This layer only rebuilds when package.json or package-lock.json change.
COPY package.json package-lock.json ./
COPY packages/shazoneSDK ./packages/shazoneSDK/

RUN npm ci

# Copy the rest of the source code
COPY . .

# Build-time environment variables baked into the client bundle by Vite.
# These must be prefixed with PUBLIC_ to be accessible in browser code
# via $env/static/public.
ARG PUBLIC_API_PREFIX=https://api.filez.zone/v1
ENV PUBLIC_API_PREFIX=$PUBLIC_API_PREFIX

ARG PUBLIC_PREFIX=https://www.filez.zone
ENV PUBLIC_PREFIX=$PUBLIC_PREFIX

# Build the SvelteKit app for production.
# Output goes to build/ (adapter-node).
RUN npm run build

# ── Stage 2: Runtime ───────────────────────────────────────────────────────
FROM node:22-slim

WORKDIR /app

# Install runtime dependencies:
# - tini: minimal init process for proper signal handling
RUN apt-get update && \
    apt-get install -y --no-install-recommends tini && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd -r app && useradd -r -g app app

# Copy only the self-contained build output from the builder stage.
# adapter-node bundles everything needed to run into build/.
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./

# OCI labels for container metadata
LABEL org.opencontainers.image.title="file-sharing-frontend" \
      org.opencontainers.image.description="Frontend for end-to-end encrypted file sharing" \
      org.opencontainers.image.source="https://github.com/ttstt/file-sharing"

USER app

# Koyeb sets PORT at runtime; adapter-node reads it automatically.
# EXPOSE is informational only.
EXPOSE 3000

# ORIGIN should be set in Koyeb's environment variables to the public URL
# of the deployed app (e.g. https://filez.zone). This prevents host-header
# spoofing attacks. Falls back to localhost for local development.
ENV ORIGIN=http://localhost:5173

# tini as the entrypoint ensures:
# - SIGTERM from Koyeb/Docker is properly forwarded to the Node process
# - Zombie processes are reaped (good practice)
ENTRYPOINT ["/usr/bin/tini", "--"]

# adapter-node's build/index.js starts an HTTP server on PORT.
CMD ["node", "build"]
