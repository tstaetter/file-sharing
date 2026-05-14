# AGENTS.md

## Project Overview

This is a **file-sharing monorepo** that enables secure, end-to-end encrypted file transfers. A user picks a file in the browser, the frontend encrypts it client-side with AES-256-GCM, and uploads the ciphertext directly to Cloudflare R2 via presigned URLs. The recipient receives a capability URL with the decryption key embedded in the hash fragment — the key never touches the server. Files are deleted from storage immediately after the first download ("burn after reading").

Authenticated users can save capability URLs to their collection — links are auto-saved after upload and browsable in a paginated list view.

Saved capability URLs are stored in the browser's **localStorage** (no server round-trip). The backend still provides API endpoints for saving/listing/deleting URLs for deployments that need server-side storage, but the frontend uses localStorage exclusively.

The repo contains three sub-projects:

| Sub-project | Path          | Language      | Description                                      |
|-------------|---------------|---------------|--------------------------------------------------|
| Backend API | `backend/`    | Rust (Axum)   | Presigned URL generation, multipart upload, burn-after-read download, user auth (JWT + bcrypt + MongoDB), saved URLs API (Bearer token middleware) |
| Frontend    | `frontend/`   | TypeScript (SvelteKit) | Upload UI, client-side encryption/decryption, capability URLs, login/register with JWT auth, localStorage-based saved URLs with auto-save and list view |
| Workers    | `worker/`     | Rust (standalone) | Background maintenance jobs (e.g. orphaned upload cleanup) |

Each sub-project has its own `AGENTS.md` with detailed setup, testing, and contribution guidance:

- `backend/AGENTS.md` — Rust toolchain, `cargo nextest` for testing, auth middleware pattern, saved URLs API endpoints
- `frontend/AGENTS.md` — Deno package manager, Vitest for testing, auth store pattern, localStorage saved URLs store
- `worker/AGENTS.md` — Rust standalone workers, S3/R2 client, periodic cleanup jobs

## Architecture

```
┌──────────────┐          ┌──────────────┐          ┌──────────────┐
│   Browser    │  HTTP    │   Backend    │  S3 API  │  Cloudflare  │
│  (SvelteKit) │◄───────►│  (Rust/Axum) │◄───────►│      R2      │
│              │          │   :8000      │          │              │
│  encrypts    │          │              │          │  stores      │
│  decrypts    │          │  presigned   │          │  encrypted   │
│  client-side │          │  URLs only   │          │  blobs       │
│              │          │              │          │              │
│  auth store  │          │  Bearer auth │          │              │
│  saved URLs  │          │  middleware  │          │              │
│  (localStor) │          │              │          │              │
└──────────────┘          └──────┬───────┘          └──────────────┘
                                 │
                                 │ MongoDB
                                 │
                          ┌──────┴───────┐
                          │   MongoDB    │
                          │  users +     │
                          │  saved_urls  │
                          └──────────────┘
```

Key architectural principles:

1. **The backend never sees plaintext.** All encryption and decryption happens in the browser via the Web Crypto API. The backend stores and serves opaque ciphertext.
2. **The decryption key lives in the URL hash fragment.** Hash fragments are never transmitted over HTTP, so the server never receives the key.
3. **Files are self-destructing.** The download endpoint fetches the object from R2, returns it, then immediately deletes it. A file can only be downloaded once.
4. **Uploads are direct-to-R2.** The backend generates presigned URLs; the frontend PUTs encrypted chunks directly to cloud storage. File data never passes through the backend.
5. **Protected routes use Bearer token middleware.** Authenticated endpoints (saved URLs) validate JWT tokens via `Authorization: Bearer <token>` header in middleware, rejecting unauthenticated requests with `401` before handlers run.
6. **Auto-save for all users.** Capability URLs are automatically saved to localStorage after a successful upload. No authentication is required — the saved URL list is per-browser.

## Upload Flow

```
Browser                              Backend                         R2
  │                                    │                              │
  │  1. Generate AES-256 key + UUID    │                              │
  │                                    │                              │
  │  2. POST /v1/create-upload ──────────►│                              │
  │     { file_id, content_type }      │                              │
  │◄────────── upload_id, key ────────│                              │
  │                                    │                              │
  │  3. For each chunk:               │                              │
  │     - encrypt(IV + chunk)         │                              │
  │                                    │                              │
  │  4. POST /v1/sign-parts ─────────────►│                              │
  │     { key, upload_id, parts }      │                              │
  │◄──────── presigned URLs ──────────│                              │
  │                                    │                              │
  │  5. PUT encrypted chunk ─────────────────────────────────────────►│
  │◄────────────────────────────────────── ETag ──────────────────────│
  │                                    │                              │
  │  6. POST /v1/complete-upload ────────►│                              │
  │     { key, upload_id, parts[] }    │   complete multipart ───────►│
  │                                    │                              │
  │  7. Build capability URL:         │                              │
  │     /f/{uuid}#{url-safe-base64(key)}                              │
  │                                    │                              │
  │  8. Save to localStorage           │                              │
  │     (title = original filename)    │                              │
```

## Download Flow

```
Browser                              Backend                         R2
  │                                    │                              │
  │  1. Open /f/{uuid}#{key}          │                              │
  │     Extract key from hash          │                              │
  │                                    │                              │
  │  2. GET /v1/f/{uuid} ────────────────►│                              │
  │                                    │   get_object ───────────────►│
  │                                    │◄─────── encrypted blob ──────│
  │                                    │   delete_object ────────────►│
  │◄──── binary stream ───────────────────────────────────────────  │
  │     Headers: X-Content-Type, X-Chunk-Size                       │
  │                                    │                              │
  │  3. Read headers, import key       │                              │
  │     Decrypt each chunk             │                              │
  │                                    │                              │
  │  4. Assemble plaintext Blob        │                              │
  │     Trigger browser download       │                              │
```

## Saved URLs Flow

```
Browser                              Backend                         R2
  │                                    │                              │
  │  1. User visits /urls             │                              │
  │     Reads from localStorage        │                              │
  │     (no auth required)             │                              │
  │                                    │                              │
  │  2. For each saved URL:           │                              │
  │     PUT /v1/check-file            │                              │
  │     { key: fileId } ─────────────►│   head_object ──────────────►│
  │                                    │◄─────── 200 or 404 ─────────│
  │◄──── 200 (exists) or 404 (gone)   │                              │
  │                                    │                              │
  │  3. Render paginated list with    │                              │
  │     copy, open & delete actions,  │                              │
  │     and "Already used" badges for │                              │
  │     consumed files                │                              │
```

## Running the Full Stack

### Prerequisites

- **Backend:** Rust toolchain (stable), `cargo-nextest`
- **Frontend:** Deno 2.x+
- **Storage:** A Cloudflare R2 bucket with API credentials
- **Database:** MongoDB (optional — required for user accounts; saved URLs use browser localStorage)

### Environment Variables

Create a `.env` file in the project root or in `backend/`. The backend reads these at startup via `dotenvy`:

```env
R2_ACCOUNT_ID=<cloudflare account id>
R2_ACCESS_KEY_ID=<r2 access key>
R2_SECRET_ACCESS_KEY=<r2 secret key>
R2_BUCKET=<bucket name>
MONGODB_URI=mongodb://localhost:27017
JWT_SECRET=<a random secret string for signing JWT tokens>
JWT_EXPIRY_MINS=5
```

Create a `frontend/.env` file:

```env
PUBLIC_API_PREFIX=http://localhost:8000/v1
PUBLIC_PREFIX=http://localhost:5173
```

Optionally set `RUST_LOG` for tracing verbosity (e.g., `RUST_LOG=info,backend=debug`).

### Starting All Services

Terminal 1 — Backend:

```bash
cd backend
cargo run
# Listening on http://0.0.0.0:8000
```

Terminal 2 — Frontend:

```bash
cd frontend
deno task dev
# Listening on http://localhost:5173
```

Terminal 3 — Cleanup worker (run periodically):

```bash
cd worker/cleanup-orphaned-uploads
cargo run
# Scans R2, aborts orphaned uploads older than 6 hours, then exits
```

### URLs

| Service  | Development URL            |
|----------|----------------------------|
| Frontend | `http://localhost:5173`    |
| Backend  | `http://localhost:8000`    |

The frontend reads the backend base URL from the `PUBLIC_API_PREFIX` environment variable (`$env/static/public`), used in:
- `frontend/src/lib/upload.ts` (delegates to SDK for create-upload, sign-parts, complete-upload)
- `frontend/src/lib/savedUrls.svelte.ts` (localStorage-based saved URL store with reactive runes)
- `frontend/src/routes/f/[id]/+page.svelte` (download fetch)

Set `PUBLIC_API_PREFIX` in `frontend/.env` (or your deployment environment) to point at the backend.

### API Endpoints Summary

| Method | Path               | Auth                          | Purpose                                 |
|--------|--------------------|-------------------------------|-----------------------------------------|
| GET    | `/health`          | None                          | Health check                            |
| POST   | `/v1/create-upload`   | None                          | Initiate multipart upload               |
| POST   | `/v1/sign-parts`      | None                          | Get presigned URLs for parts            |
| POST   | `/v1/complete-upload` | None                          | Finalise multipart upload               |
| POST   | `/v1/abort-upload`    | None                          | Cancel multipart upload                 |
| GET    | `/v1/f/:id`           | None                          | Download encrypted blob (burn-after-read) |
| PUT    | `/v1/check-file`     | None                          | Check if a file still exists in storage   |
| POST   | `/v1/auth/register`   | None                          | Register new user                       |
| POST   | `/v1/auth/login`      | None                          | Authenticate, get JWT                   |
| DELETE | `/v1/delete`           | `Authorization: Bearer <token>` | Delete user account (204 No Content)   |
| POST   | `/v1/urls`            | `Authorization: Bearer <token>` | Save a capability URL (not used by frontend — see localStorage) |
| GET    | `/v1/urls`            | `Authorization: Bearer <token>` | List saved URLs, paginated (not used by frontend — see localStorage) |

## Sub-Project AGENTS.md References

For language-specific setup, testing commands, code conventions, and contribution patterns, see:

- **[backend/AGENTS.md](backend/AGENTS.md)** — Rust edition 2021, Axum 0.8, `cargo nextest`, tracing, S3/R2 client, auth middleware pattern, `AuthUser` extractor, Bearer token convention, saved URLs handlers, error handling conventions
- **[frontend/AGENTS.md](frontend/AGENTS.md)** — Deno, SvelteKit 2, Svelte 5 runes, Vitest, TailwindCSS, ESLint + Prettier, `$lib` module conventions, auth store pattern, saved URLs store with localStorage and reactive runes
- **[worker/AGENTS.md](worker/AGENTS.md)** — Rust standalone workers, S3/R2 client, cleanup jobs, Docker deployment

## Common Cross-Cutting Tasks

### AppState

The backend uses `AppState` to hold shared state passed to all handlers via Axum extractors:

```rust
#[derive(Clone)]
pub struct AppState {
    pub s3: Client,                        // R2/S3 client for presigned URLs and object operations
    pub bucket: String,                    // R2 bucket name
    pub database: Option<mongodb::Database>, // MongoDB handle for user accounts and saved URLs (optional)
}
```

The `database` field is an `Option` — when `None`, auth and saved URL endpoints return errors but all file-sharing operations continue to work normally. This allows the service to run without MongoDB for deployments that don't need user accounts or URL saving.

### Route Groups in the Backend

The backend router is organised into three groups with different authentication strategies:

```rust
pub fn app(state: AppState) -> Router {
    // Auth routes — token passed in request body (for login/register)
    let auth_routes = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .with_state(state.clone());

    // Protected routes — token validated via Bearer auth middleware
    let protected_routes = Router::new()
        .route("/urls", post(save_url).get(list_urls))
        .route("/delete", delete(delete_user))
        .layer(middleware::from_fn(require_auth));

    // Unprotected routes — no authentication required
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

    Router::new()
        .route("/health", get(health))
        .nest("/v1", routes)
}
```

### Authentication Architecture

The project uses two authentication patterns depending on the route group:

| Route group | Paths | Auth pattern | Rationale |
|---|---|---|---|
| `auth_routes` | `/v1/auth/*` | Token in request body | Login and register don't have a token yet |
| `protected_routes` | `/v1/urls`, `/v1/delete` | `Authorization: Bearer <token>` header, validated by middleware | Standard REST API pattern; middleware rejects unauthenticated requests before handlers run. Note: the frontend uses localStorage for saved URLs; the `/v1/urls` endpoints are available for server-side storage but not called by the default UI. |
| Unprotected | Everything else | None | Anonymous file sharing works without auth |

**How the Bearer auth middleware works:**

1. The `require_auth` middleware is applied to `protected_routes` via `.layer(middleware::from_fn(require_auth))`.
2. It extracts the `Authorization` header and validates the JWT using `jsonwebtoken` + `JWT_SECRET`.
3. On success, it inserts `AuthUser { claims }` into request extensions and passes the request to the handler.
4. On failure, it returns `401 Unauthorized` — the handler never runs.
5. Handlers extract `AuthUser` as a standard Axum extractor (`FromRequestParts`) to access the verified claims.

**How the frontend auth store works:**

1. The auth store (`src/lib/auth.svelte.ts`) uses Svelte 5 runes for reactive state.
2. JWTs are persisted in `localStorage` with automatic expiry detection.
3. Login/register return `{ token, user }` — the token is stored and the user info is made reactive.
4. Protected API calls (`savedUrls.svelte.ts`) are replaced by direct localStorage operations — no server round-trip needed.
5. The `isAuthenticated` derived state enables conditional UI (nav links, auto-save, auth guards).

### Adding a feature that touches both frontend and backend

1. Define the API contract first (new endpoint path, request/response shapes, auth requirements).
2. Implement the backend handler following the pattern in `backend/AGENTS.md`:
   - If it needs auth, add `auth_user: AuthUser` as the first extractor and register in `protected_routes`.
   - If it doesn't need auth, register in the main `routes` router.
3. Implement the frontend integration following the pattern in `frontend/AGENTS.md`:
   - For localStorage-based state, create a `.svelte.ts` module following the `savedUrls.svelte.ts` pattern.
   - For API calls, create a standard `.ts` module and call `fetch` against `PUBLIC_API_PREFIX`.
   - For protected endpoints, accept a `token: string` parameter and set `Authorization: Bearer ${token}` header.
   - Wire into a page component with loading/error/empty states.
4. Update both AGENTS.md files and this root AGENTS.md if the change introduces a new convention or dependency.

### Setting up a local S3-compatible test bucket

For local development without hitting R2, you can run MinIO or use the `aws-smithy-http` test utilities. Update the backend's S3 config in `main.rs` to point at the local endpoint.

### Debugging encryption issues

1. The frontend encrypts with AES-256-GCM, prepending a 12-byte IV per chunk.
2. Each chunk produces: `IV (12 bytes) || ciphertext || GCM tag (16 bytes)`.
3. The download page splits the blob back into chunks using `ENCRYPTED_CHUNK_SIZE = IV_LEN + CHUNK_SIZE + GCM_TAG_LEN`.
4. If decryption fails, check that the key in the URL hash was correctly encoded/decoded as URL-safe base64 without padding.

### Debugging auth issues

1. Check that the JWT is present and not expired — `auth.isAuthenticated` is a derived state in the frontend.
2. Verify the `Authorization: Bearer <token>` header is being sent (check browser DevTools Network tab).
3. The backend `require_auth` middleware logs failures at the `tracing` level — enable debug logging with `RUST_LOG=debug`.
4. Tokens are signed with `JWT_SECRET` — ensure the same secret is set in both development and deployment environments.
5. The `JWT_EXPIRY_MINS` env var controls token lifetime (default 5 minutes). Short-lived tokens mean users will need to re-authenticate.

## Repository Conventions

- **.gitignore** at the root covers build artifacts for both projects (`backend/target`, `frontend/node_modules`, `frontend/.svelte-kit`, IDE configs, `.env` files).
- **docker-compose.yml** exists as a placeholder for future containerized deployment.
- **No shared code** between backend and frontend — they communicate solely over HTTP with JSON.
- **All AGENTS.md files** should be kept up to date when conventions or dependencies change.