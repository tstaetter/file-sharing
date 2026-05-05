# Backend

Backend API for the file-sharing service. Built with **Rust** and **Axum**, it generates presigned URLs for multipart uploads directly to Cloudflare R2 and serves encrypted blobs with self-destructing ("burn after reading") downloads.

The backend never sees plaintext — all encryption and decryption happens client-side in the browser.

## Quick Start

### Prerequisites

- Rust toolchain (stable)
- [cargo-nextest](https://nexte.st/) (`cargo install cargo-nextest`)
- A Cloudflare R2 bucket with API credentials

### Environment Variables

Create a `.env` file in the `backend/` directory:

```env
R2_ACCOUNT_ID=<cloudflare account id>
R2_ACCESS_KEY_ID=<r2 access key>
R2_SECRET_ACCESS_KEY=<r2 secret key>
R2_BUCKET=<bucket name>
```

Optionally set `RUST_LOG` for log verbosity:

```env
RUST_LOG=info,backend=debug
```

### Run

```bash
cargo run
```

The server starts on **`http://0.0.0.0:8000`**. On startup it applies a CORS policy to the R2 bucket (logged as a warning if permissions are insufficient).

## API Endpoints

All endpoints accept and return JSON.

| Method | Path               | Purpose                                                  |
|--------|--------------------|----------------------------------------------------------|
| POST   | `/create-upload`   | Initiate a multipart upload. Returns `upload_id` and key. |
| POST   | `/sign-parts`      | Generate presigned URLs for part numbers (valid 1 hour).  |
| POST   | `/complete-upload` | Finalise multipart upload with ETags.                     |
| POST   | `/abort-upload`    | Cancel an in-progress multipart upload.                   |
| GET    | `/f/:id`           | Download encrypted blob. **Deletes object from R2 after reading.** |

### Example: Create Upload

```bash
curl -X POST http://localhost:8000/create-upload \
  -H "Content-Type: application/json" \
  -d '{"file_id": "abc123", "content_type": "image/png"}'
```

### Example: Sign Parts

```bash
curl -X POST http://localhost:8000/sign-parts \
  -H "Content-Type: application/json" \
  -d '{"key": "uploads/abc123", "upload_id": "...", "part_numbers": [1, 2, 3]}'
```

### Example: Download

```bash
curl http://localhost:8000/f/abc123
```

Returns a JSON object with base64-encoded `data`, a `nonce`, and the original `content_type`.

## Testing

```bash
# Run all tests
cargo nextest run

# Run a specific test
cargo nextest run <test_name>

# Run with output
cargo nextest run -- --nocapture
```

## Project Structure

```
src/
├── main.rs        → tracing, S3 client, CORS policy, server startup
├── lib.rs         → AppState, re-exports
├── routes.rs      → Axum router definition
└── handlers/
    ├── create_upload.rs
    ├── sign_parts.rs
    ├── complete_upload.rs
    ├── abort_upload.rs
    └── download.rs
```

## Tech Stack

| Component   | Crate                       |
|-------------|-----------------------------|
| Framework   | `axum` 0.8                 |
| Async       | `tokio` (full features)     |
| Storage     | `aws-sdk-s3` (Cloudflare R2) |
| Logging     | `tracing` + `tracing-subscriber` |
| CORS        | `tower-http`                |
| Serialization | `serde` + `serde_json`    |
| Env vars    | `dotenvy`                   |

See [AGENTS.md](AGENTS.md) for detailed conventions, design decisions, and agent guidance.