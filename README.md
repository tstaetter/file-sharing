# filez.zone

End-to-end encrypted file sharing with burn-after-reading downloads. Pick a file, get a link — the recipient downloads it once and it's gone forever. **Zero-knowledge architecture** ensures the server never sees your plaintext or encryption keys.

Authenticated users can **save capability URLs** to their collection — links are automatically saved after upload and browsable in a paginated list view.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![GitHub](https://img.shields.io/badge/GitHub-sha--zone%2Ffile--sharing-181717?logo=github)](https://github.com/sha-zone/file-sharing)

## How It Works

1. **Uploader** picks a file in the browser. The frontend generates an AES-256-GCM key, encrypts the file chunk-by-chunk, and uploads the ciphertext directly to Cloudflare R2 via presigned URLs. A capability URL is produced with the decryption key embedded in the hash fragment — **the key never touches the server**. If the user is authenticated, the capability URL is **auto-saved** to their collection.
2. **Recipient** opens the link. The browser downloads the encrypted blob, decrypts it locally using the key from the URL hash, and saves the plaintext. The file is **deleted from R2 immediately** after the first download.
3. **The server has zero knowledge.** Hash fragments are never transmitted over HTTP. Even if our infrastructure is compromised, your data remains unreadable.

Read more on our [Zero-Knowledge Encryption](https://www.filez.zone/zero-knowledge) page.

```
Browser (encrypt)  ──►  Backend (presigned URLs)  ──►  Cloudflare R2 (encrypted bytes)
Browser (decrypt)  ◄──  Backend (fetch & delete)   ◄──  Cloudflare R2 (encrypted bytes)
Browser (save url) ──►  Backend (Bearer auth)      ──►  MongoDB (saved_urls)
```

## Project Structure

| Sub-project | Path | Language | Description |
|---|---|---|---|
| Backend API | `backend/` | Rust (Axum) | Presigned URL generation, multipart upload, burn-after-read download, health endpoint, user auth (JWT + bcrypt + MongoDB), saved URLs with Bearer token middleware |
| Frontend | `frontend/` | TypeScript (SvelteKit) | Upload UI, client-side encryption/decryption, capability URLs, zero-knowledge page, login/register with JWT auth, auto-save and paginated list view for saved URLs |
| Cleanup Worker | `worker/cleanup-orphaned-uploads/` | Rust (standalone) | Background job that aborts orphaned multipart uploads older than 6 hours, runs via Supercronic on a schedule |

## Quick Start

### Prerequisites

- **Backend:** Rust toolchain (stable), [cargo-nextest](https://nexte.st/)
- **Frontend:** [Deno](https://deno.com/) 2.x or later
- **Cleanup Worker:** Rust toolchain (stable)
- **Storage:** A [Cloudflare R2](https://www.cloudflare.com/developer-platform/r2/) bucket with API credentials
- **Database:** [MongoDB](https://www.mongodb.com/) (optional — required for user accounts and saved URLs)

### Environment Variables

Create a `.env` file in the project root (used by both backend and worker):

```env
R2_ACCOUNT_ID=<cloudflare account id>
R2_ACCESS_KEY_ID=<r2 access key>
R2_SECRET_ACCESS_KEY=<r2 secret key>
R2_BUCKET=<bucket name>
```

Optionally set `RUST_LOG=info` for verbose logging.

For user account and saved URL features, also set:

```env
MONGODB_URI=mongodb://localhost:27017
JWT_SECRET=<a random secret string for signing JWT tokens>
JWT_EXPIRY_MINS=5
```

Create a `frontend/.env` file:

```env
PUBLIC_API_PREFIX=http://localhost:8000/v1
PUBLIC_PREFIX=http://localhost:5173
```

### Start All Services

Terminal 1 — Backend:

```bash
cd backend
cargo run
# → http://0.0.0.0:8000
```

Terminal 2 — Frontend:

```bash
cd frontend
deno install
deno task dev
# → http://localhost:5173
```

Terminal 3 — Cleanup worker (run periodically):

```bash
cd worker/cleanup-orphaned-uploads
cargo run
# Scans R2, aborts orphaned uploads, then exits
```

### URLs

| Service | Development URL |
|---|---|
| Frontend | `http://localhost:5173` |
| Backend | `http://localhost:8000` |

## API Endpoints

### File-sharing endpoints (no auth required)

| Method | Path | Purpose |
|---|---|---|
| GET | `/health` | Health check — returns `{"status":"ok"}` |
| POST | `/v1/create-upload` | Initiate a multipart upload |
| POST | `/v1/sign-parts` | Generate presigned URLs for part numbers (valid 1 hour) |
| POST | `/v1/complete-upload` | Finalise multipart upload with ETags |
| POST | `/v1/abort-upload` | Cancel an in-progress multipart upload |
| GET | `/v1/f/:id` | Download encrypted blob and **delete from R2** |
| PUT | `/v1/check-file` | Check if a file still exists in storage (head_object) |

### Auth endpoints (token in request body)

| Method | Path | Purpose |
|---|---|---|
| POST | `/v1/auth/register` | Create user account (returns JWT) |
| POST | `/v1/auth/login` | Authenticate user (returns JWT) |
| POST | `/v1/auth/delete` | Delete authenticated user's account |

### Protected endpoints (Bearer auth required)

| Method | Path | Purpose | Auth |
|---|---|---|---|
| POST | `/v1/urls` | Save a capability URL to the user's collection | `Authorization: Bearer <token>` |
| GET | `/v1/urls` | List saved URLs with pagination (`?page=&per_page=`) | `Authorization: Bearer <token>` |

Protected endpoints use the `require_auth` middleware which validates the JWT before the request reaches the handler. Unauthenticated requests receive `401 Unauthorized`.

For full request/response schemas and curl examples, see [backend/README.md](backend/README.md).

## User Accounts & Saved URLs (optional)

filez.zone supports optional user accounts for managing shared files. Accounts are protected with:

- **bcrypt** password hashing — no plaintext passwords are ever stored
- **JWT tokens** (expiry configurable via `JWT_EXPIRY_MINS`, default 5 minutes) for session management
- **MongoDB** for persistent user storage and saved URL records

The backend serves auth endpoints at `/v1/auth/register`, `/v1/auth/login`, and `/v1/auth/delete`. The frontend provides login and registration pages at `/login` and `/register`.

Authenticated users automatically get their capability URLs saved after upload. Saved URLs can be browsed at `/urls` with pagination, copy-to-clipboard, and direct-open actions. Each saved URL is checked against storage to show an **"Already used" badge** when the file has been consumed by a burn-after-read download.

### Authentication Architecture

The project uses two authentication patterns:

| Route group | Paths | Auth pattern |
|---|---|---|
| `auth_routes` | `/v1/auth/*` | Token in request body |
| `protected_routes` | `/v1/urls` | `Authorization: Bearer <token>` header, validated by middleware |

The frontend auth store (`src/lib/auth.svelte.ts`) persists JWT tokens in `localStorage` with automatic expiry detection and reactive state via Svelte 5 runes. Protected API calls in `src/lib/savedUrls.ts` send the token via the `Authorization: Bearer <token>` header.

## Encryption

- **Algorithm:** AES-256-GCM
- **Per-chunk IVs:** Each 6 MB chunk gets its own random 12-byte IV prepended to the ciphertext
- **Key distribution:** The symmetric key is embedded in the URL hash fragment (never sent to the server)
- **Format:** Capability URLs follow the pattern `{base}/f/{uuid}#{url-safe-base64(key)}`
- **Zero-knowledge proof:** The server stores and serves opaque encrypted blobs — it has no technical ability to decrypt them

Learn more on the [Zero-Knowledge Encryption](https://www.filez.zone/zero-knowledge) page.

## Deployment

The project is designed for deployment on [Koyeb](https://www.koyeb.com/). Each sub-project has its own `Dockerfile`:

| Component | Dockerfile | Koyeb Service Type | Notes |
|---|---|---|---|
| Backend | `backend/Dockerfile` | Web Service | Uses `tini` as init, reads `PORT` from Koyeb, has `/health` endpoint |
| Frontend | `frontend/Dockerfile` | Web Service | Uses `tini` as init, adapter-node bundles everything into `build/`, has `/health` endpoint |
| Cleanup Worker | `worker/cleanup-orphaned-uploads/Dockerfile` | Worker | Uses Supercronic for in-container cron scheduling (`*/30 * * * *`) |

Key Koyeb environment variables for the frontend: `ORIGIN=https://your-domain.com` (to prevent host-header spoofing). Build args: `PUBLIC_API_PREFIX=https://api.your-domain.com/v1`.

## Analytics

We use [OpenPanel](https://openpanel.dev), a self-hosted, **cookieless** analytics tool, to collect anonymous page view data. No cookies are set, no personal identifiers are collected, and users cannot be tracked across sessions. See our [Privacy Policy](https://www.filez.zone/privacy) for details.

## Testing

```bash
# Backend (cargo-nextest) — 106 tests pass
cd backend && cargo nextest run

# Frontend (Vitest)
cd frontend && deno task test

# Cleanup Worker (cargo-nextest)
cd worker/cleanup-orphaned-uploads && cargo nextest run
```

## Further Reading

- [AGENTS.md](AGENTS.md) — architecture diagrams, upload/download/saved-urls flows, authentication architecture, conventions, and agent guidance
- [backend/README.md](backend/README.md) — backend-specific setup, API details, auth middleware, saved URLs, and tech stack
- [frontend/README.md](frontend/README.md) — frontend-specific setup, task reference, saved URLs page, auto-save, and how it works
- [worker/cleanup-orphaned-uploads/README.md](worker/cleanup-orphaned-uploads/README.md) — worker-specific setup and configuration
- [Zero-Knowledge Encryption](https://www.filez.zone/zero-knowledge) — how zero-knowledge architecture protects your files
- [Privacy Policy](https://www.filez.zone/privacy) — how we handle your data

## License

MIT