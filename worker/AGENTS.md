# AGENTS.md

## Project Overview

The `worker/` directory contains background workers that perform periodic maintenance tasks for the file-sharing service. These workers operate independently from the backend API and frontend — they run as standalone binaries that interact directly with Cloudflare R2 via the S3 API.

Currently there is one worker:

| Worker | Path | Purpose |
|--------|------|---------|
| `cleanup-orphaned-uploads/` | Scans R2 for multipart uploads older than 6 hours and aborts them, freeing storage and reducing costs |

Workers share the same R2 credentials and bucket as the backend API but do not communicate with it over HTTP. They are intended to be run as scheduled jobs (e.g. via cron, Kubernetes CronJob, or Cloudflare Workers) rather than as long-running services.

## Tech Stack

- **Language:** Rust (edition 2024)
- **Async runtime:** [Tokio](https://tokio.rs) (full features)
- **Object storage:** [aws-sdk-s3](https://docs.rs/aws-sdk-s3) pointed at a Cloudflare R2 endpoint
- **Concurrency:** [futures](https://docs.rs/futures) (`FuturesUnordered` for bounded concurrent aborts)
- **Date/time:** [chrono](https://docs.rs/chrono) for the cutoff calculation
- **Error handling:** [thiserror](https://docs.rs/thiserror) 2.x
- **Logging/tracing:** [tracing](https://docs.rs/tracing) + [tracing-subscriber](https://docs.rs/tracing-subscriber) with `env-filter`
- **Environment variables:** [dotenvy](https://docs.rs/dotenvy)
- **DateTime conversion:** [aws-smithy-types-convert](https://docs.rs/aws-smithy-types-convert) (S3 `DateTime` → `chrono`)

## Directory Structure

```
worker/
├── AGENTS.md                          ← this file
└── cleanup-orphaned-uploads/
    ├── Cargo.toml
    ├── Cargo.lock
    ├── Dockerfile
    ├── README.md
    ├── .gitignore
    └── src/
        ├── main.rs                    ← R2 client setup, pagination loop, abort orchestration
        └── lib.rs                     ← CleanupError / CleanupResult type definitions
```

## Workers

### `cleanup-orphaned-uploads`

When a user starts a multipart upload but never completes or aborts it (e.g. they close the browser tab, lose network, etc.), the partially uploaded chunks remain in R2 indefinitely. This worker scans for such orphaned uploads and aborts them.

**Algorithm:**

1. Build an S3 client from the R2 credentials.
2. Compute a cutoff timestamp: now minus 6 hours.
3. Paginate through `list_multipart_uploads` with the `uploads/` prefix.
4. For each upload whose `initiated` timestamp is older than the cutoff, call `abort_multipart_upload`.
5. Abort calls are dispatched concurrently with a cap of 10 in-flight at a time (`MAX_CONCURRENT`).
6. Wait for all remaining aborts to finish.

**Key constants:**

| Constant | Value | Purpose |
|----------|-------|---------|
| `MAX_CONCURRENT` | `10` | Maximum number of in-flight abort requests |

The 6-hour cutoff is hardcoded in `main.rs`. If this needs to be configurable, add an environment variable (e.g. `ORPHAN_AGE_HOURS`) and read it with `env::var`.

## Setup Instructions

1. **Prerequisites:** Rust toolchain (stable, edition 2024).
2. **Environment variables:** Create a `.env` file in the worker directory (e.g. `worker/cleanup-orphaned-uploads/.env`):

   ```env
   R2_ACCOUNT_ID=<cloudflare account id>
   R2_ACCESS_KEY_ID=<r2 access key>
   R2_SECRET_ACCESS_KEY=<r2 secret key>
   R2_BUCKET=<bucket name>
   ```

   These are the same variables used by the backend API. The worker reads them at startup via `dotenvy::dotenv().ok()`.

3. **Build:** `cargo build --release` (or `cargo build` for a debug build).

## How to Run

```bash
cd worker/cleanup-orphaned-uploads
cargo run
```

The worker runs once, scans for orphaned uploads, aborts them, and exits. It is not a long-running daemon — schedule it to run periodically (e.g. every hour) using cron or your orchestration platform.

Optionally set `RUST_LOG` for tracing verbosity (e.g., `RUST_LOG=info,cleanup_orphaned_uploads=debug`).

### Docker

Build and run the containerised worker:

```bash
cd worker/cleanup-orphaned-uploads
docker build -t cleanup-orphaned-uploads .
docker run --env-file .env cleanup-orphaned-uploads
```

## Environment Variables

| Variable               | Description                       | Required |
|------------------------|-----------------------------------|----------|
| `R2_ACCOUNT_ID`        | Cloudflare account ID (used to construct the R2 endpoint URL) | Yes |
| `R2_ACCESS_KEY_ID`     | R2 API access key                 | Yes |
| `R2_SECRET_ACCESS_KEY` | R2 API secret key                 | Yes |
| `R2_BUCKET`            | Name of the R2 bucket to scan      | Yes |
| `RUST_LOG`             | Tracing verbosity (e.g. `info`, `debug`) | No |

The region is hardcoded to `"auto"` and the endpoint URL is derived from `R2_ACCOUNT_ID` as `https://{account_id}.r2.cloudflarestorage.com`. These match the backend API's configuration.

## Error Handling

The worker defines a `CleanupError` enum in `src/lib.rs`:

```rust
pub enum CleanupError {
    DateTime(/* from aws_smithy_types_convert */),
    Sdk(String),
}
```

- `DateTime` — raised when converting S3 `DateTime` timestamps to `chrono` types fails.
- `Sdk` — raised on any AWS SDK error (e.g. failed `list_multipart_uploads` or `abort_multipart_upload` calls).

The main function returns `CleanupResult<()>`. Any error causes the worker to exit with a non-zero status. Individual abort failures are not retried — the next scheduled run will pick them up.

## Code Style & Conventions

- Follow standard Rust idioms and the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Run `cargo fmt` before committing.
- Run `cargo clippy` and fix all warnings before committing.
- Use `tracing::info!`, `tracing::warn!`, and `tracing::error!` for logging. Avoid `println!`.
- Keep `MAX_CONCURRENT` and the orphan age cutoff as named constants at the top of `main.rs`.
- New error variants should be added to `CleanupError` in `lib.rs`.
- The `CleanupResult` type alias (`Result<T, CleanupError>`) should be used as the return type for fallible functions.

## Common Tasks for Agents

### Adding a new worker

1. Create a new directory under `worker/` (e.g. `worker/my-new-worker/`).
2. Initialise a Cargo project: `cargo init --name my-new-worker`.
3. Add dependencies to `Cargo.toml` following the existing pattern (`aws-sdk-s3`, `tokio`, `tracing`, `dotenvy`, etc.).
4. Implement `src/main.rs` with the same R2 client setup pattern (read env vars → build config → create client).
5. Add a `Dockerfile` following the existing one.
6. Update this AGENTS.md's Workers table with the new entry.
7. Update the root `AGENTS.md` if the change affects cross-cutting documentation.

### Making the orphan age configurable

1. Add a new env var (e.g. `ORPHAN_AGE_HOURS`) read with `env::var`.
2. Parse it with `.parse::<i64>()` and provide a sensible default (6) if the variable is not set.
3. Replace the hardcoded `Duration::hours(6)` in `main.rs`.
4. Update the Environment Variables table above.

### Improving error handling

- Replace `.unwrap()` calls with proper error propagation using `CleanupError`.
- Consider adding retry logic for transient S3 errors (throttling, 5xx) using an exponential backoff strategy.
- Log individual abort failures as warnings instead of failing the entire run, so one bad key doesn't block cleanup of the rest.

### Adding tests

- Place unit tests in `#[cfg(test)] mod tests` blocks within the source files.
- Integration tests go in a `tests/` directory at the crate root.
- For S3-dependent tests, use `aws-smithy-mocks` (add to `dev-dependencies` with `aws-sdk-s3 = { version = "1", features = ["test-util"] }`).
- The `list_multipart_uploads` pagination logic and the cutoff comparison are good candidates for unit testing without hitting R2.