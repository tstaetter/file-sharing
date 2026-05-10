# syntax=docker/dockerfile:1
# Requires Docker BuildKit. Enable with: DOCKER_BUILDKIT=1 docker build ...

# Stage 1: Build
FROM rust:1.92-slim-bookworm AS builder

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy actual source
COPY . .

# Build application
RUN cargo build --release

# ---------- Runtime ----------
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/backend /app/backend

EXPOSE 8000

CMD ["/app/backend"]
