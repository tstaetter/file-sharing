# Cleanup Orphaned Uploads Worker

Background worker that scans Cloudflare R2 for orphaned multipart uploads and aborts them. This frees storage and reduces costs for uploads that were started but never completed or aborted (e.g. user closed the browser tab).

## How It Works

1. Connects to Cloudflare R2 using the same credentials as the backend.
2. Computes a cutoff timestamp: now minus 6 hours.
3. Paginates through all active multipart uploads under the `uploads/` prefix.
4. For each upload older than the cutoff, calls `abort_multipart_upload`.
5. Aborts are dispatched concurrently with a cap of 10 in-flight at a time.
6. Exits after all eligible uploads have been processed.

The 6-hour cutoff is hardcoded in `src/main.rs`. Orphaned uploads are not retried on failure — the next scheduled run will pick them up.

## Configuration

The worker reads the following environment variables at startup:

| Variable | Description |
|---|---|
| `R2_ACCOUNT_ID` | Cloudflare account ID (used to construct the R2 endpoint URL) |
| `R2_ACCESS_KEY_ID` | R2 API access key |
| `R2_SECRET_ACCESS_KEY` | R2 API secret key |
| `R2_BUCKET` | Name of the R2 bucket to scan |
| `RUST_LOG` | Tracing verbosity (e.g. `info`, `debug`). Defaults to `info` if not set. |

These are the same variables used by the backend API. Create a `.env` file in this directory or set them in your deployment environment.

## Development

### Prerequisites

- Rust toolchain (stable)

### Running Locally

```bash
# From the worker directory
cd worker/cleanup-orphaned-uploads
cargo run
```

The worker runs once, processes orphaned uploads, logs progress, and exits. It is not a long-running daemon.

### Testing

```bash
cargo nextest run
```

## Docker

A `Dockerfile` is provided for containerized deployment. It uses a multi-stage build:

- **Builder stage:** `rust:1-slim-bookworm` — compiles dependencies first (cached), then builds the real binary.
- **Runtime stage:** `debian:bookworm-slim` — copies only the compiled binary, runs with Supercronic as PID 1.

### Build

```bash
docker build -t cleanup-orphaned-uploads .
```

### Run (one-off)

```bash
docker run --env-file .env cleanup-orphaned-uploads
```

## Deployment on Koyeb

The worker is deployed on [Koyeb](https://www.koyeb.com/) as a long-running Worker service using [Supercronic](https://github.com/aptible/supercronic) for in-container cron scheduling.

**How it works:** Supercronic runs as PID 1 and executes the cleanup binary on a cron schedule. The container stays alive 24/7 — Supercronic handles SIGTERM for graceful shutdown when Koyeb restarts or redeploys.

### Deployment Steps

1. **Service type:** Worker (not Web Service)
2. **Build context:** `worker/cleanup-orphaned-uploads`
3. **Environment variables (secrets):**
   - `R2_ACCOUNT_ID`
   - `R2_ACCESS_KEY_ID`
   - `R2_SECRET_ACCESS_KEY`
   - `R2_BUCKET`
   - `RUST_LOG=info`
4. **No PORT or health checks needed** — workers don't serve HTTP traffic
5. **No cron schedule in Koyeb** — Supercronic handles scheduling internally

### Crontab

The schedule is defined in the Dockerfile:

```
*/30 * * * * /usr/local/bin/cleanup-orphaned-uploads
```

This runs every 30 minutes. To change the frequency, update the crontab in the `Dockerfile` and redeploy.

Common alternatives:

| Schedule | Cron expression |
|---|---|
| Every 5 minutes | `*/5 * * * *` |
| Every 30 minutes | `*/30 * * * *` |
| Every hour | `0 * * * *` |
| Every 6 hours | `0 */6 * * *` |

### Supercronic Version

The Dockerfile downloads and verifies Supercronic at build time. The version and SHA1 checksum are declared as build args — check the [Supercronic releases page](https://github.com/aptible/supercronic/releases) for updates.

## Project Structure

```
worker/cleanup-orphaned-uploads/
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
├── .dockerignore
├── README.md
└── src/
    ├── main.rs   ← R2 client setup, pagination loop, abort orchestration
    └── lib.rs    ← CleanupError / CleanupResult type definitions
```

## Tech Stack

| Component | Crate |
|---|---|
| Async runtime | `tokio` (full features) |
| Object storage | `aws-sdk-s3` (Cloudflare R2) |
| Date/time | `chrono` |
| DateTime conversion | `aws-smithy-types-convert` |
| Error handling | `thiserror` 2.x |
| Logging | `tracing` + `tracing-subscriber` |
| Env vars | `dotenvy` |
| Cron scheduler | `supercronic` (in Docker) |
| Deployment | `Koyeb` |

See the root [AGENTS.md](../AGENTS.md) for cross-cutting conventions and architecture decisions.