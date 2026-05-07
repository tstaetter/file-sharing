# AGENTS.md

## Project Overview

This is a **file-sharing monorepo** that enables secure, end-to-end encrypted file transfers. A user picks a file in the browser, the frontend encrypts it client-side with AES-256-GCM, and uploads the ciphertext directly to Cloudflare R2 via presigned URLs. The recipient receives a capability URL with the decryption key embedded in the hash fragment — the key never touches the server. Files are deleted from storage immediately after the first download ("burn after reading").

The repo contains three sub-projects:

| Sub-project | Path          | Language      | Description                                      |
|-------------|---------------|---------------|--------------------------------------------------|
| Backend API | `backend/`    | Rust (Axum)   | Presigned URL generation, multipart orchestration, burn-after-read download |
| Frontend    | `frontend/`   | TypeScript (SvelteKit) | Upload UI, client-side encryption/decryption, capability URL handling |
| Workers    | `worker/`     | Rust (standalone) | Background maintenance jobs (e.g. orphaned upload cleanup) |

Each sub-project has its own `AGENTS.md` with detailed setup, testing, and contribution guidance:

- `backend/AGENTS.md` — Rust toolchain, `cargo nextest` for testing
- `frontend/AGENTS.md` — Deno package manager, Vitest for testing
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
└──────────────┘          └──────────────┘          └──────────────┘
```

Key architectural principles:

1. **The backend never sees plaintext.** All encryption and decryption happens in the browser via the Web Crypto API. The backend stores and serves opaque ciphertext.
2. **The decryption key lives in the URL hash fragment.** Hash fragments are never transmitted over HTTP, so the server never receives the key.
3. **Files are self-destructing.** The download endpoint fetches the object from R2, returns it, then immediately deletes it. A file can only be downloaded once.
4. **Uploads are direct-to-R2.** The backend generates presigned URLs; the frontend PUTs encrypted chunks directly to cloud storage. File data never passes through the backend.

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
  │  3. For each 5MB chunk:           │                              │
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

## Running the Full Stack

### Prerequisites

- **Backend:** Rust toolchain (stable), `cargo-nextest`
- **Frontend:** Deno 2.x+
- **Storage:** A Cloudflare R2 bucket with API credentials

### Environment Variables

Create a `.env` file in the project root or in `backend/`. The backend reads these at startup via `dotenvy`:

```env
R2_ACCOUNT_ID=<cloudflare account id>
R2_ACCESS_KEY_ID=<r2 access key>
R2_SECRET_ACCESS_KEY=<r2 secret key>
R2_BUCKET=<bucket name>
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

The frontend reads the backend base URL from the `PUBLIC_API_PREFIX` environment variable (`$env/static/public`), used in two places:
- `frontend/src/lib/upload.ts` (three `fetch` calls for create-upload, sign-parts, complete-upload)
- `frontend/src/routes/f/[id]/+page.svelte` (one `fetch` call)

Set `PUBLIC_API_PREFIX` in `frontend/.env` (or your deployment environment) to point at the backend.

## Sub-Project AGENTS.md References

For language-specific setup, testing commands, code conventions, and contribution patterns, see:

- **[backend/AGENTS.md](backend/AGENTS.md)** — Rust edition 2024, Axum 0.8, `cargo nextest`, tracing, S3/R2 client, handler patterns, error handling conventions
- **[frontend/AGENTS.md](frontend/AGENTS.md)** — Deno, SvelteKit 2, Svelte 5 runes, Vitest, TailwindCSS, ESLint + Prettier, `$lib` module conventions
- **[worker/AGENTS.md](worker/AGENTS.md)** — Rust standalone workers, S3/R2 client, cleanup jobs, Docker deployment
- **[worker/AGENTS.md](worker/AGENTS.md)** — Rust standalone workers, S3/R2 client, cleanup jobs, Docker deployment

## Common Cross-Cutting Tasks

### Adding a feature that touches both frontend and backend

1. Define the API contract first (new endpoint path, request/response shapes).
2. Implement the backend handler following the pattern in `backend/AGENTS.md` (new file in `src/handlers/`, re-export, register route).
3. Implement the frontend integration following the pattern in `frontend/AGENTS.md` (new module in `src/lib/`, call the endpoint, wire into a page).
4. Update both AGENTS.md files if the change introduces a new convention or dependency.

### Setting up a local S3-compatible test bucket

For local development without hitting R2, you can run MinIO or use the `aws-smithy-http` test utilities. Update the backend's S3 config in `main.rs` to point at the local endpoint.

### Debugging encryption issues

1. The frontend encrypts with AES-256-GCM, prepending a 12-byte IV per chunk.
2. Each 5 MB chunk produces: `IV (12 bytes) || ciphertext || GCM tag (16 bytes)`.
3. The download page splits the blob back into chunks using `ENCRYPTED_CHUNK_SIZE = IV_LEN + CHUNK_SIZE + GCM_TAG_LEN`.
4. If decryption fails, check that the key in the URL hash was correctly encoded/decoded as URL-safe base64 without padding.

## Repository Conventions

- **.gitignore** at the root covers build artifacts for both projects (`backend/target`, `frontend/node_modules`, `frontend/.svelte-kit`, IDE configs, `.env` files).
- **docker-compose.yml** exists as a placeholder for future containerized deployment.
- **No shared code** between backend and frontend — they communicate solely over HTTP with JSON.
- **All three AGENTS.md files** should be kept up to date when conventions or dependencies change.