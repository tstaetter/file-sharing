# Stage 1: Build
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to cache dependencies
RUN mkdir src && \
    echo 'fn main() {}' > src/main.rs && \
    echo '' > src/lib.rs && \
    mkdir src/handlers && \
    echo '' > src/handlers/mod.rs

# Build dependencies only (this layer is cached)
RUN cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the actual binary (touch main.rs to invalidate cached dummy)
RUN touch src/main.rs && \
    cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd -r app && useradd -r -g app app

COPY --from=builder /app/target/release/backend /usr/local/bin/backend

USER app
EXPOSE 8000

CMD ["backend"]
