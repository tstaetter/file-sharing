# AGENTS.md

## Project Overview

This is the backend API for a file-sharing service. It provides endpoints that enable a browser-based frontend to:

1. **Upload files** via a multipart, resumable upload mechanism using presigned URLs.
2. **Download files** exactly once — after a successful download the stored object is deleted from cloud storage ("burn after reading").
3. **Authenticate users** with bcrypt password hashing and JWT tokens.
4. **Save and list capability URLs** for authenticated users. The backend provides these API endpoints, though the default frontend stores saved URLs in the browser's localStorage instead.

Files are stored in Cloudflare R2, an S3-compatible object store. The backend generates presigned URLs for individual upload parts so the frontend can upload chunks directly to R2 without the file data passing through the backend server.

**Encryption model:** All file data is encrypted **client-side** using AES-GCM. The backend never sees plaintext. The download handler returns the raw ciphertext plus metadata in response headers; decryption happens entirely in the browser.

## Tech Stack

- **Language:** Rust (edition 2021)
- **Web framework:** [Axum 0.8](https://docs.rs/axum/0.8)
- **Async runtime:** [Tokio](https://tokio.rs) (full features)
- **Object storage:** [aws-sdk-s3](https://docs.rs/aws-sdk-s3) pointed at a Cloudflare R2 endpoint
- **Logging/tracing:** [tracing](https://docs.rs/tracing) + [tracing-subscriber](https://docs.rs/tracing-subscriber) with `env-filter`
- **CORS:** [tower-http](https://docs.rs/tower-http) `CorsLayer::permissive()`
- **Serialization:** [serde](https://serde.rs) + [serde_json](https://docs.rs/serde_json)
- **Auth:** [jsonwebtoken](https://docs.rs/jsonwebtoken) (JWT) + [bcrypt](https://docs.rs/bcrypt) (password hashing)
- **Database:** [mongodb](https://docs.rs/mongodb) (user accounts, saved URLs)
- **Environment variables:** [dotenvy](https://docs.rs/dotenvy)

## Directory Structure

```
backend/
├── AGENTS.md              ← this file
├── Cargo.toml
├── Cargo.lock
└── src/
    ├── main.rs            ← initialises tracing, S3 client, CORS policy, MongoDB, starts server
    ├── lib.rs             ← defines AppState, re-exports handlers, middleware & routes
    ├── routes.rs          ← Axum Router definition with auth and protected route groups
    ├── db/
    │   ├── mod.rs         ← database module declarations
    │   ├── user.rs        ← User model for MongoDB
    │   └── saved_url.rs   ← SavedUrl model for MongoDB
    ├── handlers/
    │   ├── mod.rs         ← handler module declarations & re-exports
    │   ├── errors.rs      ← error types (DownloadError, AuthError, SavedUrlError, etc.)
    │   ├── health.rs      ← GET /health — orchestrator health check
    │   ├── create_upload.rs   ← POST /v1/create-upload
    │   ├── sign_parts.rs      ← POST /v1/sign-parts
    │   ├── complete_upload.rs ← POST /v1/complete-upload
    │   ├── abort_upload.rs    ← POST /v1/abort-upload
    │   ├── download.rs        ← GET /v1/f/:id — burn-after-read download
    │   ├── auth.rs            ← /v1/auth/* — register, login, delete user, JWT token creation/validation
    │   └── saved_urls.rs      ← POST /v1/urls, GET /v1/urls — save and list capability URLs
    └── middleware/
        ├── mod.rs         ← middleware module declarations
        └── auth.rs        ← require_auth middleware, AuthUser extractor, AuthMiddlewareError
    └── tests/
        ├── health_test.rs
        ├── handlers_test.rs
        ├── create_upload_integration_test.rs
        ├── sign_parts_integration_test.rs
        ├── complete_upload_integration_test.rs
        ├── abort_upload_integration_test.rs
        ├── download_integration_test.rs
        ├── auth_integration_test.rs
        └── saved_urls_integration_test.rs
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
   MONGODB_URI=mongodb://localhost:27017
   JWT_SECRET=<a random secret string for signing JWT tokens>
   JWT_EXPIRY_MINS=5
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
- **MongoDB integration tests** use a **per-test unique database** pattern: each test that needs a database calls `test_state_with_db("unique_db_name")` which creates a fresh MongoDB database, drops `users` and `saved_urls` collections for a clean slate, and returns an `AppState` wired to that database. This enables **parallel-safe execution** — tests never interfere with each other's data. The `test_state()` helper (no arguments) returns an `AppState` with `database: None` for testing behaviour when MongoDB is absent.
- The `.config/nextest.toml` configuration file controls nextest settings such as retries, threads, and failure output. Keep it in sync with the CI environment.

## API Endpoints

All endpoints are served under `http://localhost:8000/` and accept/return JSON unless otherwise noted.

### File-sharing endpoints (no auth required)

| Method | Path               | Purpose                                                                 | Request body        | Response body          |
|--------|--------------------|-------------------------------------------------------------------------|----------------------|------------------------|
| POST   | `/v1/create-upload`   | Initiate a multipart upload. Returns an `upload_id` and the object key. | `CreateRequest`     | `CreateResponse`       |
| POST   | `/v1/sign-parts`      | Generate presigned URLs for a list of part numbers (valid for 1 hour).  | `SignPartsRequest`  | `Vec<SignedPart>`      |
| POST   | `/v1/complete-upload` | Finalise the multipart upload with the ETags from the client's PUTs.    | `CompleteRequest`   | (empty 200)            |
| POST   | `/v1/abort-upload`    | Abort an in-progress multipart upload and discard all uploaded parts.   | `AbortRequest`      | (empty 200)            |
| GET    | `/v1/f/:id`           | Download the encrypted blob. **Deletes the object from R2 after read.** | —                    | Binary stream (see below) |
| PUT    | `/v1/check-file`     | Check whether a file still exists in storage (via head_object).   | `CheckFileRequest`  | 200 OK or 404 NotFound   |

### Auth endpoints (`/v1/auth/*`)

| Method | Path               | Purpose                                                      | Request body                   | Response body          |
|--------|--------------------|--------------------------------------------------------------|---------------------------------|------------------------|
| POST   | `/v1/auth/register`   | Register a new user. Returns JWT token and user info.      | `RegisterRequest`              | `RegisterResponse`     |
| POST   | `/v1/auth/login`      | Authenticate. Returns JWT token and user info.              | `LoginRequest`                 | `LoginResponse`        |

### Protected endpoints (Bearer auth required)

These routes require a valid JWT in the `Authorization: Bearer <token>` header. The `require_auth` middleware validates the token before the request reaches the handler. Unauthenticated requests receive `401 Unauthorized`.

| Method | Path               | Purpose                                                      | Request body                   | Response body          |
|--------|--------------------|--------------------------------------------------------------|---------------------------------|------------------------|
| POST   | `/v1/urls`            | Save a capability URL to the authenticated user's collection. | `SaveUrlRequest { url, title }` | `SaveUrlResponse`      |
| GET    | `/v1/urls`            | List the authenticated user's saved URLs (paginated).         | Query params: `page`, `per_page` | `ListUrlsResponse`     |
| DELETE | `/v1/delete`          | Delete the authenticated user's account (204 No Content).    | —                              | (empty 204)            |

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

### Authentication middleware

Protected routes (currently `/v1/urls`) use the `require_auth` middleware applied via `axum::middleware::from_fn`. The middleware:

1. Extracts the `Authorization: Bearer <token>` header from the incoming request.
2. Validates the JWT using `jsonwebtoken` and the `JWT_SECRET` environment variable.
3. On success, inserts an `AuthUser { claims }` struct into request extensions.
4. On failure, returns `401 Unauthorized` immediately — the handler never runs.

Handlers on protected routes can then extract `AuthUser` directly as an Axum extractor. The `AuthUser` type implements `FromRequestParts<S>` and pulls the pre-validated claims from request extensions. If a handler accidentally uses `AuthUser` on an unprotected route, the extractor returns `AuthMiddlewareError::MissingAuth` (which also maps to `401`).

This design cleanly separates authentication (middleware) from authorization and business logic (handlers). Handlers do not need to validate tokens themselves or read them from request bodies.

### Saved URLs

Authenticated users can save capability URLs to their collection. The `save_url` and `list_urls` handlers in `saved_urls.rs`:

- Require a valid `Authorization: Bearer <token>` header (enforced by middleware).
- Identify the user via the `sub` claim (email) from the validated JWT.
- Store URLs in the `saved_urls` MongoDB collection with fields: `id` (UUID), `user_email`, `url`, `title`, `created_at`.
- Support pagination (`page` and `per_page` query parameters, defaults 1 and 10, max 100).
- Return results in reverse chronological order (newest first).

Note: the default frontend stores saved URLs in the browser's localStorage and does not call these endpoints. They are available for deployments that prefer server-side storage.

## Code Style & Conventions

- Follow standard Rust idioms and the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Run `cargo fmt` before committing (no custom `rustfmt.toml` at this time).
- Run `cargo clippy` and fix all warnings before committing.
- Public types used across module boundaries should be defined in the relevant module, re-exported from `mod.rs`, and then re-exported from `lib.rs`.
- Use `tracing::info!`, `tracing::warn!`, and `tracing::error!` for logging. Avoid `println!` in production code.
- The `AppState` struct (containing the S3 client, bucket name, and optional MongoDB database) is managed as Axum state (`axum::extract::State`). It must remain `Clone`.
- **Route groups:** The router is organised into three groups:
  - `auth_routes` — `/v1/auth/*` (register, login) — authenticated via explicit token in request body
  - `protected_routes` — `/v1/urls` (save, list), `/v1/delete` (delete account) — authenticated via `Authorization: Bearer <token>` header middleware
  - Unprotected routes — `/v1/create-upload`, `/v1/sign-parts`, etc. — no authentication required

## Route Organisation Pattern

```rust
pub fn app(state: AppState) -> Router {
    // Auth routes — token passed in request body (login/register)
    let auth_routes = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .with_state(state.clone());

    // Protected routes — token validated via Bearer auth middleware
    let protected_routes = Router::new()
        .route("/urls", post(save_url).get(list_urls))
        .route("/delete", delete(delete_user))
        .layer(middleware::from_fn(require_auth));

    // All v1 routes
    let routes = Router::new()
        .route("/create-upload", post(create_upload))
        .route("/sign-parts", post(sign_parts))
        .route("/complete-upload", post(complete_upload))
        .route("/abort-upload", post(abort_upload))
        .route("/f/{id}", get(download))
        .nest("/auth", auth_routes)
        .merge(protected_routes)
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Top-level router with health check outside /v1
    Router::new()
        .route("/health", get(health))
        .nest("/v1", routes)
}
```

## Environment Variables

| Variable               | Description                                    | Required | Default |
|------------------------|------------------------------------------------|----------|---------|
| `R2_ACCOUNT_ID`        | Cloudflare account ID                           | Yes      | —       |
| `R2_ACCESS_KEY_ID`     | R2 API access key                               | Yes      | —       |
| `R2_SECRET_ACCESS_KEY` | R2 API secret key                               | Yes      | —       |
| `R2_BUCKET`            | Name of the R2 bucket                           | Yes      | —       |
| `MONGODB_URI`          | MongoDB connection string                       | No       | `mongodb://localhost:27017` |
| `JWT_SECRET`           | Secret key for signing JWT tokens               | Yes*     | —       |
| `JWT_EXPIRY_MINS`      | JWT token expiry in minutes                     | No       | `5`     |
| `PORT`                 | HTTP port to listen on                          | No       | `8000`  |
| `RUST_LOG`             | Tracing verbosity                               | No       | `info`  |

\* `JWT_SECRET` is required when using user authentication or saved URLs. If not set, any endpoint that validates tokens will fail at runtime. The application does not check for it at startup — set it in your `.env` or deployment environment.

## Common Tasks for Agents

### Adding a new endpoint

1. Create a new file in `src/handlers/` with an async handler function that takes the necessary Axum extractors.
2. If the endpoint requires authentication:
   - Add `auth_user: AuthUser` as the first extractor parameter to get the authenticated user's claims.
   - Register the route inside `protected_routes` in `routes.rs` so it gets the `require_auth` middleware.
3. Add `mod <name>;` and `pub use <name>::*;` to `src/handlers/mod.rs`.
4. Add the re-export to `src/lib.rs` (if the types need to be publicly accessible).
5. Register the route in `src/routes.rs` using `.route("<path>", <method>(<handler>))`.

### Adding authentication middleware

The `require_auth` middleware is defined in `src/middleware/auth.rs`. To extend or modify it:

1. The middleware function signature is `async fn require_auth(request: Request, next: Next) -> Result<Response, StatusCode>`.
2. It reads the `Authorization` header, extracts the Bearer token, and calls `validate_token()`.
3. Validated claims are stored as `AuthUser` in request extensions via `request.extensions_mut().insert(auth_user)`.
4. To create a new middleware, follow the same pattern and apply it with `.layer(middleware::from_fn(your_middleware))`.

### Using the AuthUser extractor in handlers

```rust
use crate::middleware::AuthUser;

pub async fn my_handler(
    auth_user: AuthUser,       // extracts validated claims from the middleware
    State(state): State<AppState>,
    Json(req): Json<MyRequest>,
) -> Result<Json<MyResponse>, MyError> {
    let email = auth_user.claims.sub;  // the authenticated user's email
    // ...
}
```

The `AuthUser` extractor implements `FromRequestParts<S>` and pulls the pre-validated claims from request extensions. It returns `AuthMiddlewareError::MissingAuth` (maps to `401`) if the middleware hasn't run.

### Improving error handling

Replace `.unwrap()` calls with proper `Result` handling. Map S3/AWS errors to appropriate HTTP status codes:
- `NoSuchKey` / 404 → file not found
- Transient errors (throttling, 5xx) → 503 Service Unavailable
- Invalid request → 400 Bad Request
- Invalid/expired JWT → 401 Unauthorized

Use Axum's `IntoResponse` to return `(StatusCode, String)` tuples or custom error types.

### Writing tests

- Use `cargo nextest` as described in the Testing section above.
- For HTTP endpoint tests, use `axum::test::Server` or the lower-level `axum::body::Body` + `axum::http::Request` helpers to exercise handlers without binding to a real port.
- For S3-dependent tests, use `aws-smithy-mocks` crate to avoid hitting the real R2 bucket.
- For middleware tests, test token parsing logic as unit tests. For integration, build a test router with the middleware layer and assert on HTTP status codes.
- For **MongoDB-dependent integration tests**, use the `test_state_with_db(db_name)` helper to create a per-test unique database. This pattern drops `users` and `saved_urls` collections at the start so each test gets a clean slate, enabling parallel-safe execution without data conflicts.
- Use `test_state()` (no arguments) for tests that verify behaviour when MongoDB is absent — it returns an `AppState` with `database: None`.
- See `.config/nextest.toml` for nextest configuration (retries, threads, failure output).