# AGENTS.md

## Project Overview

This is the backend API for a file-sharing service. It provides endpoints that enable a browser-based frontend to:

1. **Upload files** via a multipart, resumable upload mechanism using presigned URLs.
2. **Download files** exactly once — after a successful download the stored object is deleted from cloud storage ("burn after reading").

Files are stored in Cloudflare R2, an S3-compatible object store. The backend generates presigned URLs for individual upload parts so the frontend can upload chunks directly to R2 without the file data passing through the backend server.

**Encryption model:** All file data is encrypted **client-side** using AES-GCM. The backend never sees plaintext. The download handler returns the raw ciphertext plus metadata as JSON; decryption happens entirely in the browser.

## Tech Stack

- **Language:** Rust (edition 2024)
- **Web framework:** [Axum 0.8](https://docs.rs/axum/0.8)
- **Async runtime:** [Tokio](https://tokio.rs) (full features)
- **Object storage:** [aws-sdk-s3](https://docs.rs/aws-sdk-s3) pointed at a Cloudflare R2 endpoint
- **Logging/tracing:** [tracing](https://docs.rs/tracing) + [tracing-subscriber](https://docs.rs/tracing-subscriber) with `env-filter`
- **CORS:** [tower-http](https://docs.rs/tower-http) `CorsLayer::permissive()`
- **Serialization:** [serde](https://serde.rs) + [serde_json](https://docs.rs/serde_json)
- **Base64:** [base64 0.22](https://docs.rs/base64)
- **Environment variables:** [dotenvy](https://docs.rs/dotenvy)

## Directory Structure

```
backend/
├── AGENTS.md              ← this file
├── Cargo.toml
├── Cargo.lock
└── src/
    ├── main.rs            ← initialises tracing, S3 client, CORS policy, starts server
    ├── lib.rs             ← defines AppState, re-exports handlers & routes
    ├── routes.rs          ← Axum Router definition
    └── handlers/
        ├── mod.rs
        ├── create_upload.rs
        ├── sign_parts.rs
        ├── complete_upload.rs
        ├── abort_upload.rs
        └── download.rs
```

The project root (`file-sharing/`) also contains `docker-compose.yml`, a `.gitignore`, and a `frontend/` directory (outside the scope of this AGENTS.md).

## Setup Instructions

1. **Prerequisites:** Rust toolchain (stable), `cargo-nextest` (install via `cargo install cargo-nextest`).
2. **Environment variables:** Create a `backend/.env` file with the following values:

   ```env
   R2_ACCOUNT_ID=<cloudflare account id>
   R2_ACCESS_KEY_ID=<r2 access key>
   R2_SECRET_ACCESS_KEY=<r2 secret key>
   R2_BUCKET=<bucket name>
   ```

   The application reads these via `dotenvy::dotenv().ok()` at startup, so they are only needed when running locally.

3. **Build:** `cargo build` (or `cargo build --release` for production).

## How to Run

```bash
cargo run
```

The server listens on `0.0.0.0:8000`. On startup it also attempts to apply a CORS configuration to the R2 bucket. This operation may fail (e.g. if the credentials lack the required permissions) — the error is logged but the server continues running.

## Testing

We use **`cargo nextest`** for running tests. Install it with `cargo install cargo-nextest` if not already present.

**Common commands:**

- Run all tests:
  ```bash
  cargo nextest run
  ```
- Run a specific test or test module:
  ```bash
  cargo nextest run <test_name>
  ```
- Run with output visible (no capture):
  ```bash
  cargo nextest run -- --nocapture
  ```
- List all tests without running them:
  ```bash
  cargo nextest list
  ```

### Writing tests

- Place unit tests in the same file as the code they exercise, inside a `#[cfg(test)] mod tests { ... }` block.
- Integration tests go in a `tests/` directory at the crate root (`backend/tests/`).
- For handler tests that require an S3 client, prefer mocking or a test-only S3-compatible service (e.g. MinIO, or `aws-smithy-http` test utilities) rather than hitting a real R2 bucket. Document any test fixtures clearly.
- For HTTP endpoint tests, use `axum::test` helpers (`axum::body::Body`, `axum::http::Request`, `axum::routing::into_make_service`) to spin up a test router without a real network socket.

## API Endpoints

All endpoints are served under `http://localhost:8000/` and accept/return JSON unless otherwise noted.

| Method | Path               | Purpose                                                                 | Request body        | Response body          |
|--------|--------------------|-------------------------------------------------------------------------|----------------------|------------------------|
| POST   | `/v1/create-upload`   | Initiate a multipart upload. Returns an `upload_id` and the object key. | `CreateRequest`     | `CreateResponse`       |
| POST   | `/v1/sign-parts`      | Generate presigned URLs for a list of part numbers (valid for 1 hour).  | `SignPartsRequest`  | `Vec<SignedPart>`      |
| POST   | `/v1/complete-upload` | Finalise the multipart upload with the ETags from the client's PUTs.    | `CompleteRequest`   | (empty 200)            |
| POST   | `/v1/abort-upload`    | Abort an in-progress multipart upload and discard all uploaded parts.   | `AbortRequest`      | (empty 200)            |
| GET    | `/v1/f/:id`           | Download the encrypted blob. **Deletes the object from R2 after read.** | —                    | Binary stream (see below) |

Refer to the handler source files for the exact struct definitions. All request/response types derive `serde::Serialize` and/or `serde::Deserialize`.

## Key Design Decisions

### Burn-after-reading download

The `GET /v1/f/{id}` handler:

1. Fetches the object stream from R2 via `get_object()`.
2. **Immediately deletes the object from R2.** The data is already in transit from R2, so the stream continues to work even after deletion.
3. Streams the raw binary ciphertext directly to the client (no buffering, no base64 encoding, no JSON wrapper).

The response body is the raw binary ciphertext — concatenated `IV (12 bytes) || ciphertext || GCM tag (16 bytes)` blocks. Metadata is sent in response headers instead of a JSON payload:

| Header            | Description                                                  | Example          |
|-------------------|--------------------------------------------------------------|------------------|
| `Content-Type`    | Always `application/octet-stream`                            | `application/octet-stream` |
| `X-Content-Type`  | The original file's MIME type (from S3 object metadata)      | `image/png`      |
| `X-Chunk-Size`    | Plaintext chunk size in bytes (from S3 object `chunk-size` metadata) | `6291456`        |
| `Cache-Control`   | `no-store, no-cache, must-revalidate`                        | —                |

A file can be downloaded **exactly once** — any subsequent request will return 404. If the `X-Chunk-Size` header is absent (legacy uploads), the client falls back to a 5 MiB default.

### Multipart upload flow

The frontend orchestrates the upload:

1. Sends `POST /v1/create-upload` with a `file_id` (a UUID generated by the frontend) and an optional `content_type`.
2. Decides how many parts it needs and sends `POST /v1/sign-parts` with the part numbers.
3. Uploads each part directly to R2 using the presigned URLs (one HTTP PUT per part).
4. Collects the ETags from those PUT responses and sends them to `POST /v1/complete-upload`.
5. If the upload is cancelled or fails, the frontend may call `POST /v1/abort-upload` to clean up.

### Encryption

Encryption is entirely **client-side**. The backend is storage-agnostic: it writes the raw (encrypted) bytes the client uploads and returns them as-is on download. The download response includes `content_type` from the original upload so the frontend can reconstruct the correct file extension after decryption.

## Code Style & Conventions

- Follow standard Rust idioms and the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Run `cargo fmt` before committing (no custom `rustfmt.toml` at this time).
- Run `cargo clippy` and fix all warnings before committing.
- Public types used across module boundaries should be defined in the relevant handler module, re-exported from `handlers/mod.rs`, and then re-exported from `lib.rs`.
- Use `tracing::info!`, `tracing::warn!`, and `tracing::error!` for logging. Avoid `println!` in production code.
- Many places currently use `.unwrap()` for simplicity; the long-term goal is proper error propagation with meaningful HTTP status codes via Axum's `IntoResponse` pattern.
- The `AppState` struct (containing the S3 client and bucket name) is managed as Axum state (`axum::extract::State`). It must remain `Clone`.

## Environment Variables

| Variable               | Description                       |
|------------------------|-----------------------------------|
| `R2_ACCOUNT_ID`        | Cloudflare account ID             |
| `R2_ACCESS_KEY_ID`     | R2 API access key                 |
| `R2_SECRET_ACCESS_KEY` | R2 API secret key                 |
| `R2_BUCKET`            | Name of the R2 bucket             |

The `RUST_LOG` environment variable controls tracing verbosity (e.g., `RUST_LOG=info,backend=debug`). The subscriber uses `EnvFilter::from_default_env()`.

## Common Tasks for Agents

### Adding a new endpoint

1. Create a new file in `src/handlers/` with an async handler function that takes `State<AppState>` and any necessary Axum extractors.
2. Add `mod <name>;` and `pub use <name>::*;` to `src/handlers/mod.rs`.
3. Add the re-export to `src/lib.rs` (if the types need to be publicly accessible).
4. Register the route in `src/routes.rs` using `.route("<path>", <method>(<handler>))`.

### Improving error handling

Replace `.unwrap()` calls with proper `Result` handling. Map S3/AWS errors to appropriate HTTP status codes:
- `NoSuchKey` / 404 → file not found
- Transient errors (throttling, 5xx) → 503 Service Unavailable
- Invalid request → 400 Bad Request

Use Axum's `IntoResponse` to return `(StatusCode, String)` tuples or custom error types.

### Writing tests

- Use `cargo nextest` as described in the Testing section above.
- For HTTP endpoint tests, use `axum::test::Server` or the lower-level `axum::body::Body` + `axum::http::Request` helpers to exercise handlers without binding to a real port.
- For S3-dependent tests, use `aws-smithy-mocks` crate to avoid hitting the real R2 bucket.
