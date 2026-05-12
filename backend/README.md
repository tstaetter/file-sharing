# Backend

Backend API for the file-sharing service. Built with **Rust** and **Axum**, it generates presigned URLs for multipart uploads directly to Cloudflare R2 and serves encrypted blobs with self-destructing ("burn after reading") downloads.

The backend never sees plaintext — all encryption and decryption happens client-side in the browser. It stores and serves opaque ciphertext only.

The backend also supports optional user accounts with bcrypt password hashing, JWT authentication, and MongoDB storage. Authenticated users can save capability URLs to their collection and browse them later.

## Quick Start

### Prerequisites

- Rust toolchain (stable)
- [cargo-nextest](https://nexte.st/) (`cargo install cargo-nextest`)
- A Cloudflare R2 bucket with API credentials
- [MongoDB](https://www.mongodb.com/) (optional — only needed for user accounts and saved URLs)

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
JWT_SECRET=<a random secret string for signing JWT tokens>
JWT_EXPIRY_MINS=5
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

### File-sharing endpoints (no auth required)

| Method | Path | Purpose |
|---|---|---|
| GET | `/health` | Health check — returns `{"status":"ok"}` with HTTP 200 |
| POST | `/v1/create-upload` | Initiate a multipart upload. Returns `upload_id` and key. |
| POST | `/v1/sign-parts` | Generate presigned URLs for part numbers (valid 1 hour). |
| POST | `/v1/complete-upload` | Finalise multipart upload with ETags. |
| POST | `/v1/abort-upload` | Cancel an in-progress multipart upload. |
| GET | `/v1/f/:id` | Download encrypted blob. **Deletes object from R2 after reading.** |

The `/health` endpoint is used by Koyeb (and other orchestrators) to determine if the service is healthy. It runs outside the `/v1` prefix and does not require CORS.

The `GET /v1/f/:id` endpoint returns the raw binary ciphertext stream directly (not JSON). Metadata such as the original content type and chunk size is sent in response headers (`X-Content-Type`, `X-Chunk-Size`).

### Auth endpoints (`/v1/auth/*`)

| Method | Path | Purpose | Auth |
|---|---|---|---|
| POST | `/v1/auth/register` | Register a new user. Accepts `{email, password, name}`, returns `{token, user}`. | None |
| POST | `/v1/auth/login` | Authenticate. Accepts `{email, password}`, returns `{token, user}`. | None |
| POST | `/v1/auth/delete` | Delete account. Accepts `{token}` in body. | Token in body |

Auth endpoints pass the JWT in the request body (for login/register) or as a `{token}` field (for delete). This differs from protected endpoints below, which use the standard `Authorization: Bearer` header.

### Protected endpoints (Bearer auth required)

| Method | Path | Purpose | Auth |
|---|---|---|---|
| POST | `/v1/urls` | Save a capability URL. Accepts `{url, title}`, returns `{id, url, title, created_at}`. | `Authorization: Bearer <token>` |
| GET | `/v1/urls` | List saved URLs with pagination. Query params: `page` (default 1), `per_page` (default 10, max 100). Returns `{urls, page, per_page, total}`. | `Authorization: Bearer <token>` |

These endpoints are protected by the `require_auth` middleware, which validates the JWT before the request reaches the handler. Unauthenticated requests receive `401 Unauthorized`. The middleware extracts the token from the `Authorization: Bearer <token>` header, validates it using `JWT_SECRET`, and inserts an `AuthUser` extension so handlers can access the verified claims without re-validating.

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

### Example: Save a URL

```bash
curl -X POST http://localhost:8000/v1/urls \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer eyJ..." \
  -d '{"url":"https://filez.zone/f/abc#key","title":"My file"}'
# → {"id":"550e8400-...","url":"https://filez.zone/f/abc#key","title":"My file","created_at":"2025-07-16T12:00:00+00:00"}
```

### Example: List Saved URLs

```bash
curl http://localhost:8000/v1/urls?page=1&per_page=10 \
  -H "Authorization: Bearer eyJ..."
# → {"urls":[{...}],"page":1,"per_page":10,"total":5}
```

## Authentication Architecture

The backend uses two authentication patterns depending on the route group:

| Route group | Paths | Auth pattern | Why |
|---|---|---|---|
| `auth_routes` | `/v1/auth/*` | Token in request body (`{token}` or part of payload) | Login and register don't have a token yet; delete reuses the existing pattern |
| `protected_routes` | `/v1/urls` | `Authorization: Bearer <token>` header, validated by middleware | Standard REST API pattern; middleware rejects unauthenticated requests before handlers run |
| Unprotected | `/v1/create-upload`, `/v1/sign-parts`, etc. | None | These operations work without authentication for anonymous file sharing |

### How the middleware works

1. The `require_auth` middleware is applied to `protected_routes` via `.layer(middleware::from_fn(require_auth))`.
2. On each request, it extracts the `Authorization` header and checks for the `Bearer ` prefix.
3. It calls `validate_token()` (from `handlers/auth.rs`) which decodes and validates the JWT using `jsonwebtoken` and `JWT_SECRET`.
4. On success, it inserts `AuthUser { claims }` into request extensions and passes the request to the next handler.
5. On failure, it returns `401 Unauthorized` — the handler never runs.
6. Handlers extract `AuthUser` as a standard Axum extractor to access the verified claims (e.g., `auth_user.claims.sub` for the user's email).

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
├── main.rs              → tracing, S3 client, CORS policy (background), MongoDB, PORT binding, graceful shutdown
├── lib.rs               → AppState, re-exports handlers, middleware, and routes
├── routes.rs            → Axum router definition with auth_routes, protected_routes, and unprotected routes
├── db/
│   ├── mod.rs           → database module declarations
│   ├── user.rs          → User model for MongoDB (email, name, password_hash)
│   └── saved_url.rs     → SavedUrl model for MongoDB (id, user_email, url, title, created_at)
├── handlers/
│   ├── mod.rs           → handler module declarations & re-exports
│   ├── errors.rs        → error types (DownloadError, AuthError, SavedUrlError, etc.) with IntoResponse impls
│   ├── health.rs        → GET /health — orchestrator health check
│   ├── create_upload.rs → POST /v1/create-upload
│   ├── sign_parts.rs    → POST /v1/sign-parts
│   ├── complete_upload.rs → POST /v1/complete-upload
│   ├── abort_upload.rs  → POST /v1/abort-upload
│   ├── download.rs      → GET /v1/f/:id — burn-after-read download
│   ├── auth.rs          → POST /v1/auth/* — register, login, delete user, JWT token creation & validation
│   └── saved_urls.rs    → POST /v1/urls, GET /v1/urls — save and list capability URLs
└── middleware/
    ├── mod.rs           → middleware module declarations
    └── auth.rs          → require_auth middleware, AuthUser extractor, AuthMiddlewareError
```

## Deployment

The backend is deployed on [Koyeb](https://www.koyeb.com/) as a Web Service. The `Dockerfile` uses a multi-stage build:

1. **Builder stage:** `rust:1-slim-bookworm` — compiles dependencies first (cached), then builds the real binary with fingerprint cleanup to avoid stale-library errors.
2. **Runtime stage:** `debian:bookworm-slim` — copies only the compiled binary, runs with `tini` as init for proper signal forwarding.

Key deployment configuration:

- **Service type:** Web Service (not Worker — this is an HTTP API)
- **Environment variables (secrets):** `R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `R2_BUCKET`, `JWT_SECRET`
- **Optional env vars:** `MONGODB_URI`, `JWT_EXPIRY_MINS`
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
| Auth | `bcrypt` 0.19, `jsonwebtoken` 10 |
| Database | `mongodb` 3 |
| Deployment | `Koyeb` |

See [AGENTS.md](AGENTS.md) for detailed conventions, design decisions, and agent guidance.