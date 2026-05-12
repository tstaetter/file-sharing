# Backend

Backend API for the file-sharing service. Built with **Rust** and **Axum**, it generates presigned URLs for multipart uploads directly to Cloudflare R2 and serves encrypted blobs with self-destructing ("burn after reading") downloads.

The backend never sees plaintext — all encryption and decryption happens client-side in the browser. It stores and serves opaque ciphertext only.

The backend also supports optional user accounts with bcrypt password hashing, JWT authentication, and MongoDB storage.

## Quick Start

### Prerequisites

- Rust toolchain (stable)
- [cargo-nextest](https://nexte.st/) (`cargo install cargo-nextest`)
- A Cloudflare R2 bucket with API credentials
- [MongoDB](https://www.mongodb.com/) (optional — only needed for user accounts)

### Environment Variables

Create a `.env` file in the `backend/` directory:

```env
R2_ACCOUNT_ID=<cloudflare account id>
R2_ACCESS_KEY_ID=<r2 access key>
R2_SECRET_ACCESS_KEY=<r2 secret key>
R2_BUCKET=<bucket name>
```

Optionally add for user account features:

```env
MONGODB_URI=<mongodb connection string>
```

Optionally set `RUST_LOG` for log verbosity:

```env
RUST_LOG=info,backend=debug
```

### Run

```bash
cargo run
```

The server starts on **`http://0.0.0.0:8000`** by default. If the `PORT` environment variable is set (as on Koyeb), the server listens on that port instead. On startup it applies a CORS policy to the R2 bucket (logged as a warning if permissions are insufficient).

## API Endpoints

All endpoints accept and return JSON unless otherwise noted.

| Method | Path | Purpose |
|---|---|---|
| GET | `/health` | Health check — returns `{"status":"ok"}` with HTTP 200 |
| POST | `/v1/create-upload` | Initiate a multipart upload. Returns `upload_id` and key. |
| POST | `/v1/sign-parts` | Generate presigned URLs for part numbers (valid 1 hour). |
| POST | `/v1/complete-upload` | Finalise multipart upload with ETags. |
| POST | `/v1/abort-upload` | Cancel an in-progress multipart upload. |
| GET | `/v1/f/:id` | Download encrypted blob. **Deletes object from R2 after reading.** |
| POST | `/v1/auth/register` | Register a new user. Accepts `{email, password, name}`, returns `{token, user}`. |
| POST | `/v1/auth/login` | Authenticate. Accepts `{email, password}`, returns `{token, user}`. |
| POST | `/v1/auth/delete` | Delete account. Accepts `{token}`. Requires valid JWT. |

The `/health` endpoint is used by Koyeb (and other orchestrators) to determine if the service is healthy. It runs outside the `/v1` prefix and does not require CORS.

The `GET /v1/f/:id` endpoint returns the raw binary ciphertext stream directly (not JSON). Metadata such as the original content type and chunk size is sent in response headers (`X-Content-Type`, `X-Chunk-Size`).

### Example: Health Check

```bash
curl http://localhost:8000/health
# → {"status":"ok"}
```

### Example: Create Upload

```bash
curl -X POST http://localhost:8000/v1/create-upload \
  -H "Content-Type: application/json" \
  -d '{"file_id": "abc123", "content_type": "image/png"}'
```

### Example: Sign Parts

```bash
curl -X POST http://localhost:8000/v1/sign-parts \
  -H "Content-Type: application/json" \
  -d '{"key": "uploads/abc123", "upload_id": "...", "part_numbers": [1, 2, 3]}'
```

### Example: Download

```bash
curl http://localhost:8000/v1/f/abc123 --output file.enc
```

Returns the raw encrypted binary stream. The file is permanently deleted from R2 immediately after this request succeeds. Any subsequent request returns 404.

### Example: Register

```bash
curl -X POST http://localhost:8000/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"secure-password","name":"Alice"}'
# → {"token":"eyJ...","user":{"email":"user@example.com","name":"Alice"}}
```

### Example: Login

```bash
curl -X POST http://localhost:8000/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"secure-password"}'
# → {"token":"eyJ...","user":{"email":"user@example.com","name":"Alice"}}
```

## Testing

```bash
# Run all tests (74 of 75 pass — one pre-existing S3 mock issue)
cargo nextest run

# Run a specific test
cargo nextest run <test_name>

# Run with output
cargo nextest run -- --nocapture
```

## Project Structure

```
src/
├── main.rs        → tracing, S3 client, CORS policy (background), PORT binding, graceful shutdown
├── lib.rs         → AppState, re-exports
├── routes.rs      → Axum router definition (/health + /v1/*)
└── handlers/
    ├── auth.rs               → POST /v1/auth/* — register, login, delete user
    ├── health.rs          → GET /health — orchestrator health check
    ├── create_upload.rs   → POST /v1/create-upload
    ├── sign_parts.rs      → POST /v1/sign-parts
    ├── complete_upload.rs → POST /v1/complete-upload
    ├── abort_upload.rs    → POST /v1/abort-upload
    └── download.rs        → GET /v1/f/:id — burn-after-read download
```

## Deployment

The backend is deployed on [Koyeb](https://www.koyeb.com/) as a Web Service. The `Dockerfile` uses a multi-stage build:

1. **Builder stage:** `rust:1-slim-bookworm` — compiles dependencies first (cached), then builds the real binary with fingerprint cleanup to avoid stale-library errors.
2. **Runtime stage:** `debian:bookworm-slim` — copies only the compiled binary, runs with `tini` as init for proper signal forwarding.

Key deployment configuration:

- **Service type:** Web Service (not Worker — this is an HTTP API)
- **Environment variables (secrets):** `R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `R2_BUCKET`
- **PORT:** Automatically set by Koyeb; the app reads it at runtime with a fallback to `8000`
- **Health check:** Koyeb performs TCP health checks by default; configure an HTTP check to `/health` for faster failure detection
- **Graceful shutdown:** Handles SIGTERM (Koyeb) and SIGINT (local Ctrl+C) via `axum::serve().with_graceful_shutdown()`
- **CORS setup:** Runs in a background `tokio::spawn` task so it never blocks startup or causes Koyeb's startup timeout to trigger
- **Logging:** Defaults to `RUST_LOG=info` if not set, writes to stdout, disables ANSI colors in Docker (no TTY)

## Tech Stack

| Component | Crate |
|---|---|
| Framework | `axum` 0.8 |
| Async | `tokio` (full features) |
| Storage | `aws-sdk-s3` (Cloudflare R2) |
| Logging | `tracing` + `tracing-subscriber` |
| CORS | `tower-http` |
| Serialization | `serde` + `serde_json` |
| Env vars | `dotenvy` |
| Init process | `tini` (in Docker) |
| Auth | `bcrypt`, `jsonwebtoken` 9.x |
| Database | `mongodb` 3.x |
| Deployment | `Koyeb` |

See [AGENTS.md](AGENTS.md) for detailed conventions, design decisions, and agent guidance.