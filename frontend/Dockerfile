# syntax=docker/dockerfile:1

# Build argument: backend API URL baked into client-side code at build time.
# Override with: docker compose build --build-arg PUBLIC_API_PREFIX=https://api.example.com/v1
ARG PUBLIC_API_PREFIX=http://localhost:8000/v1

# ── Stage 1: Build ──────────────────────────────────────────────────────────
FROM node:22-slim AS builder

WORKDIR /app

# Copy manifests and local packages first for dependency caching
COPY package.json package-lock.json ./
COPY packages/shazoneSDK ./packages/shazoneSDK/

RUN npm ci

# Copy source code
COPY . .

# Set the build-time environment variable for Vite ($env/static/public)
ARG PUBLIC_API_PREFIX
ENV PUBLIC_API_PREFIX=$PUBLIC_API_PREFIX
ARG PUBLIC_PREFIX
ENV PUBLIC_PREFIX=$PUBLIC_PREFIX
# Build the SvelteKit app for production
RUN npm run build

# Prune dev-only dependencies from node_modules for a leaner runtime image
RUN npm prune --omit=dev

# ── Stage 2: Runtime ───────────────────────────────────────────────────────
FROM node:22-slim

WORKDIR /app

RUN groupadd -r app && useradd -r -g app app

# Copy production dependencies and built app from builder
COPY --from=builder /app/build ./build
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package.json ./

USER app
EXPOSE 3000

# SvelteKit adapter-node configuration
# PORT  – the port the server listens on (default 3000)
# ORIGIN – the canonical origin of the app, required in production
#          to prevent host-header spoofing. Override for your domain.
ENV PORT=3000
ENV ORIGIN=http://localhost:5173

CMD ["node", "build"]
