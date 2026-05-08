# syntax=docker/dockerfile:1
# Requires Docker BuildKit. Enable with: DOCKER_BUILDKIT=1 docker build ...

# Stage 1: Build
FROM rust:1.92-slim-bookworm AS builder

ARG REBUILD=false

WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source files to compile and cache dependencies
RUN mkdir src && \
    echo 'fn main() {}' > src/main.rs && \
    echo '' > src/lib.rs && \
    mkdir src/handlers && \
    echo '' > src/handlers/mod.rs

# Build dependencies only — may skip if nothing changed
RUN echo "Build setup: ${REBUILD}" && cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the real binary, always rebuild
RUN echo "Build ID: ${REBUILD}" && cargo build --release && \
    cp target/release/backend /usr/local/bin/backend

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd -r app && useradd -r -g app app

COPY --from=builder /usr/local/bin/backend /usr/local/bin/backend

USER app
EXPOSE 8000

CMD ["backend"]
