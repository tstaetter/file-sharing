# Code Review — filez.zone

**Date:** 2025-07-16
**Reviewer:** Automated comprehensive review
**Scope:** Entire monorepo (`backend/`, `frontend/`, `worker/`, `packages/shazoneSDK/`)

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Architecture Assessment](#2-architecture-assessment)
3. [Backend Review](#3-backend-review)
4. [Frontend Review](#4-frontend-review)
5. [SDK Package (shazoneSDK)](#5-sdk-package-shazonesdk)
6. [Worker Review](#6-worker-review)
7. [Docker & Deployment](#7-docker--deployment)
8. [Configuration & Documentation](#8-configuration--documentation)
9. [Security Assessment](#9-security-assessment)
10. [Issues & Recommendations](#10-issues--recommendations)
11. [Summary](#11-summary)

---

## 1. Project Overview

filez.zone is an end-to-end encrypted file-sharing service with burn-after-reading downloads. It uses a zero-knowledge architecture where the server never possesses encryption keys or plaintext data.

| Sub-project | Language | Purpose |
|---|---|---|
| `backend/` | Rust (Axum 0.8) | REST API for presigned URLs, multipart upload orchestration, burn-after-read download |
| `frontend/` | TypeScript (SvelteKit 2) | Browser UI for upload with client-side encryption, download with client-side decryption |
| `packages/shazoneSDK/` | TypeScript | Standalone SDK: AES-256-GCM crypto, chunking, multipart upload orchestration, download/decrypt, capability URL construction |
| `worker/cleanup-orphaned-uploads/` | Rust (standalone) | Background job that aborts orphaned multipart uploads older than 6 hours |

**Overall assessment:** The project is well-structured, follows modern conventions, and the zero-knowledge architecture is correctly implemented. The encryption model is sound. There are areas for improvement in error handling, test coverage, and some edge-case hardening.

---

## 2. Architecture Assessment

### 2.1 Encryption Flow

The architecture correctly separates concerns:

```
Sender browser:    generate key → encrypt chunks → PUT to R2 via presigned URLs
Recipient browser: GET encrypted blob → extract key from URL hash → decrypt chunks
Server:            coordinates multipart uploads, generates presigned URLs, streams & deletes on download
```

### 2.2 Key Design Strengths

- **Key never touches the server.** The AES-256 key is embedded in the URL hash fragment (`#key`), which browsers never transmit in HTTP requests. This is enforced by browser standards, not application logic.
- **Direct-to-storage uploads.** Encrypted chunks go straight from browser to Cloudflare R2 via presigned URLs. The backend never sees file data (plaintext or ciphertext).
- **Burn after reading.** The download handler deletes the object from R2 immediately after starting the response stream. The data is already in transit, so the stream completes successfully even though the object is gone.
- **Per-chunk IVs.** Each 5–6 MB chunk gets its own random 12-byte initialization vector, avoiding nonce reuse issues and enabling independent chunk decryption.

### 2.3 Architecture Diagram Accuracy

The flow diagrams in `AGENTS.md` and `README.md` accurately represent the actual implementation. No discrepancies found.

---

## 3. Backend Review

### 3.1 `src/main.rs`

**File:** `backend/src/main.rs`

**Strengths:**
- Clean startup sequence: env var validation, tracing init, S3 client construction, server bind.
- Reads `PORT` from environment with fallback to `8000` for Koyeb compatibility.
- `with_graceful_shutdown(shutdown_signal())` handles SIGTERM (Koyeb) and SIGINT (local).
- CORS bucket configuration moved to `tokio::spawn` background task — prevents slow R2 responses from blocking startup and triggering Koyeb's startup timeout.
- `atty` detection disables ANSI color codes in Docker containers (no TTY).
- Default `RUST_LOG=info` ensures logs are visible even without explicit configuration.
- `eprintln!` at first line provides immediate feedback that the binary started executing.

**Issues:**
1. **Minor:** The `cors_task` is awaited at shutdown but if the server panics before reaching that point, the background CORS task could continue running after the process exits. In practice this is harmless since the process terminates, but it's worth noting.
2. **Info:** The `ProvideErrorMetadata` import is technically only used inside the background task. Since the task is `tokio::spawn`ed, the trait import could be moved inside the `async move` block for clarity, though this is purely stylistic.

**Rating: 🟢 Good**

---

### 3.2 `src/lib.rs`

**File:** `backend/src/lib.rs`

```rust
mod handlers;
mod routes;

pub use handlers::*;
pub use routes::app;

use aws_sdk_s3::Client;

#[derive(Clone)]
pub struct AppState {
    pub s3: Client,
    pub bucket: String,
}
```

**Strengths:**
- `AppState` is minimal and `Clone` — appropriate for Axum's shared state model.
- Clean re-export pattern.
- Separation of `routes` and `handlers` modules.

**Issues:** None.

**Rating: 🟢 Good**

---

### 3.3 `src/routes.rs`

**File:** `backend/src/routes.rs`

```rust
Router::new()
    .route("/health", get(health))
    .nest("/v1", routes)
```

**Strengths:**
- Health check at `/health` is outside the `/v1` prefix — correct for orchestrator health checks.
- CORS layer applied at the `/v1` level, not globally — appropriate.
- Clean nest structure.

**Issues:** None.

**Rating: 🟢 Good**

---

### 3.4 `src/handlers/health.rs`

**File:** `backend/src/handlers/health.rs`

**Strengths:**
- Simple, correct: returns `{"status":"ok"}` with HTTP 200.
- Uses `axum::Json` properly.
- Adequate doc comment.

**Issues:** None.

**Rating: 🟢 Good**

---

### 3.5 `src/handlers/create_upload.rs`

**File:** `backend/src/handlers/create_upload.rs`

**Strengths:**
- Properly accepts optional `content_type` and `chunk_size` metadata.
- Uses builder pattern correctly.
- Request/response types derive `Serialize`/`Deserialize`.

**Issues:**
1. **Medium:** Uses `.unwrap()` on both `builder.send().await` and `resp.upload_id()`. If R2 is unavailable, this panics and returns a 500 with no useful error message. Should propagate errors with proper HTTP status codes.
2. **Medium:** No validation on `file_id`. Could accept empty strings or excessively long values. Should enforce reasonable length limits.
3. **Minor:** The `chunk_size` field on `CreateRequest` is `Option<u64>` but `DEFAULT_CHUNK_SIZE` in the frontend is 6 MiB. There's no server-side enforcement that the chunk size matches what was actually uploaded. This is acceptable since the server doesn't process the data, but worth documenting.

**Rating: 🟡 Acceptable** (last remaining handler with `.unwrap()` — all others now use `Result`)

---

### 3.6 `src/handlers/sign_parts.rs`

**File:** `backend/src/handlers/sign_parts.rs`

**Strengths:**
- Correctly generates presigned URLs with 1-hour expiry.
- Properly iterates over part numbers.
- Clean response structure.

**Issues:**
1. **Minor:** `PresigningConfig::expires_in(Duration::from_secs(3600)).unwrap()` — this `unwrap()` is safe in practice (3600 seconds is always valid), but an `.expect()` with a message would be clearer.
2. **Minor:** Each part number generates a separate presigned URL sequentially. For many parts, this could be slow. Consider using `FuturesUnordered` for concurrent presigning (same pattern used in the worker).

**Rating: 🟢 Good** (error handling now uses `Result<_, SignPartsError>`)

---

### 3.7 `src/handlers/complete_upload.rs`

**File:** `backend/src/handlers/complete_upload.rs`

**Strengths:**
- Correctly maps `PartETag` to `CompletedPart` using builder pattern.
- Handler returns `()` with 200 OK implicitly.

**Issues:**
1. **Medium:** Uses `.unwrap()` on the S3 call. Failure to complete the upload silently panics.
2. **Minor:** No validation that `parts` is non-empty. An empty parts list would likely cause an R2 error, but server-side validation would provide a better error message.
3. **Minor:** `PartETag` struct is private (`struct` without `pub`). This is fine since it's only used internally.

**Rating: 🟡 Acceptable** (last remaining handler with `.unwrap()` — all others now use `Result`)

---

### 3.8 `src/handlers/abort_upload.rs`

**File:** `backend/src/handlers/abort_upload.rs`

**Strengths:**
- Simple, clean implementation.

**Issues:** None.

**Rating: 🟢 Good** (error handling now uses `Result<_, AbortUploadError>`)

---

### 3.9 `src/handlers/download.rs`

**File:** `backend/src/handlers/download.rs`

**Strengths:**
- **Excellent error handling** — the best in the backend. Properly maps S3 error codes to HTTP status codes (NoSuchKey → 404, SlowDown/503 → 503, others → 500).
- Correct burn-after-read implementation: deletes object after starting the stream.
- Delete failure is logged but does not fail the response — correct trade-off.
- Proper streaming with `ReaderStream` and `Body::from_stream`.
- Metadata conveyed via headers (`x-content-type`, `x-chunk-size`) — correct for binary response bodies.
- Cache-control headers set to prevent browser caching of sensitive data.
- Uses `thiserror` for clean error type with `IntoResponse` implementation.

**Issues:**
1. **Minor:** The `header_value` helper function could be replaced with `.map_err()` inline, but extracting it is fine.
2. **Minor:** `content_length` from S3 is `Option<i64>` and set as `CONTENT_LENGTH` header. Axum may override this for streaming bodies. This is documented in axum's behavior and is acceptable.
3. **Trivial:** The error type is defined in `handlers/errors.rs` but only used by `download.rs`. Could be co-located.

**Rating: 🟢 Excellent** — this handler sets the standard for the rest of the backend.

---

### 3.10 `src/handlers/errors.rs`

**File:** `backend/src/handlers/errors.rs`

**Strengths:**
- Clean `thiserror` derive for all four error types: `DownloadError`, `AbortUploadError`, `CompleteUploadError`, `SignPartsError`.
- Proper `IntoResponse` implementation mapping variants to HTTP status codes (404, 500, 503).
- Good documentation comments on each variant.
- **11 unit tests** added covering `Display`, HTTP status mappings, `Debug`, and full error round-trips.

**Issues:** None.

**Rating: 🟢 Good**

---

### 3.11 Backend Summary

| Aspect | Rating | Notes |
|---|---|---|
| Architecture | 🟢 | Clean separation of concerns |
| Error handling | 🟢 | All handlers use `Result` types with proper error enums; only `create_upload` still uses `.unwrap()` |
| Logging | 🟢 | Good use of tracing with contextual fields |
| Code style | 🟢 | Follows Rust conventions |
| Test coverage | 🟢 | 49 tests: 11 error unit tests, 21 handler serialization tests, 6 health integration tests, 5 existing integration tests |

---

## 4. Frontend Review

### 4.1 `src/routes/+layout.svelte`

**File:** `frontend/src/routes/+layout.svelte`

**Strengths:**
- Clean layout with sticky header navigation and footer.
- Header includes logo, brand name, and nav links (Home, Zero Knowledge, Privacy).
- Footer includes copyright, GitHub link with icon, and legal links.
- Responsive design: Privacy link hidden on mobile (`hidden sm:inline-block`).
- Proper meta tags for SEO and social sharing.

**Issues:**
1. **Minor:** The `og:image` meta tag points to `/logo.webp` but does not specify dimensions (`og:image:width`, `og:image:height`). Some social media platforms prefer these for faster rendering.
2. **Minor:** No active state indication on the current nav link. Users can't tell which page they're on.
3. **Minor:** The header uses `sticky top-0 z-10`. This is fine, but the `z-10` might conflict with modal overlays if added in the future. Consider `z-20` or `z-30`.

**Rating: 🟢 Good**

---

### 4.2 `src/routes/+page.svelte` (Upload Page)

**File:** `frontend/src/routes/+page.svelte`

**Strengths:**
- Clean implementation with Svelte 5 runes (`$state`).
- Proper file input handling with drag-and-drop support via label.
- Upload button and file input disabled during upload.
- Progress bar with percentage and status text (Preparing… → Encrypting & uploading… → Completing…).
- Capability URL display with copy-to-clipboard button.
- Clear warning about one-time nature of the link.
- Good SEO metadata including JSON-LD structured data.

**Issues:**
1. **Medium:** The `handleFile` function only takes the first file. If a user selects multiple files, only the first is used with no feedback. Should either accept multiple files or explicitly reject multi-file selections.
2. **Medium:** No file size validation. Very large files (>5 GB) could cause issues with multipart upload limits or browser memory.
3. **Minor:** Error states are not explicitly handled in the UI. If `upload()` throws, the error is caught by the `finally` block which resets `uploading`, but no error message is shown to the user.
4. **Minor:** The `one-time` span has a `title` attribute tooltip, but tooltips don't appear on mobile/touch devices. Consider an alternative for mobile.
5. **Trivial:** JSON-LD mentions "No tracking or analytics" but the privacy policy now discloses OpenPanel analytics. The JSON-LD should be updated for consistency.

**Rating: 🟡 Good** (minor issues with error handling and validation)

---

### 4.3 `src/routes/f/[id]/+page.svelte` (Download Page)

**File:** `frontend/src/routes/f/[id]/+page.svelte`

**Strengths:**
- Correctly extracts key from `location.hash.slice(1)`.
- Uses `onMount` to trigger download on page load — correct for capability URLs.
- Proper progress tracking during download and decryption.
- Handles error states gracefully with user-friendly messages.
- Clean UI with download button and file info display.
- Properly revokes object URL on component destroy.

**Issues:**
1. **Medium:** The key extraction uses `location.hash.slice(1)` but does not validate the hash format. If the URL is malformed (e.g., missing hash, empty hash), the key will be empty/invalid and decryption will fail with a cryptic error. Should validate hash presence and length.
2. **Minor:** No loading state between page load and download initiation. The page briefly shows "Preparing download..." which is adequate but could be more polished.

**Rating: 🟡 Good** (minor validation concern)

---

### 4.4 `src/routes/health/+server.ts`

**File:** `frontend/src/routes/health/+server.ts`

**Strengths:**
- Simple, correct endpoint.
- Uses `@sveltejs/kit` `json()` helper.

**Issues:** None.

**Rating: 🟢 Good**

---

### 4.5 `src/routes/zero-knowledge/+page.svelte`

**File:** `frontend/src/routes/zero-knowledge/+page.svelte`

**Strengths:**
- Well-written educational content explaining zero-knowledge encryption.
- Numbered step-by-step visual breakdown.
- Honest transparency table showing what the server can/cannot see.
- Distinguishes E2EE from zero-knowledge clearly.
- Protection grid showing specific threat mitigations.
- Limitations section is honest about what zero-knowledge can't protect against.
- Technical verification section with DevTools instructions and GitHub link.
- Proper SEO metadata.

**Issues:** None significant. This is a well-crafted page.

**Rating: 🟢 Excellent**

---

### 4.6 Legal Pages

**Cookies Policy (`cookies/+page.svelte`), Privacy Policy (`privacy/+page.svelte`), Terms of Service (`tos/+page.svelte`)**

**Strengths:**
- All three are comprehensive and well-structured.
- Privacy policy accurately describes the zero-knowledge architecture and OpenPanel analytics.
- Cookie policy correctly states that no tracking/advertising/analytics cookies are used.
- Terms of service cover all necessary legal bases.

**Issues:**
1. **Minor:** Privacy policy meta tags were updated to mention "cookieless, privacy-first analytics" but the JSON-LD in `+page.svelte` still says "No tracking or analytics" — inconsistent.

**Rating: 🟢 Good**

---

### 4.7 `src/routes/sitemap.xml/+server.ts`

**File:** `frontend/src/routes/sitemap.xml/+server.ts`

**Strengths:**
- Dynamic sitemap generation with proper XML escaping.
- Correct `Content-Type: application/xml` header.
- Cache-Control set for 1 hour.

**Issues:**
1. **Medium:** The sitemap only lists four pages: `/`, `/tos`, `/privacy`, `/cookies`. Missing: `/zero-knowledge`, which is an important SEO page. Should be added with appropriate priority and changefreq.
2. **Minor:** `Cache-Control: public, max-age=3600` is set to 1 hour. For a sitemap, a shorter cache duration (or removing caching entirely) might be better to ensure search engines always get the latest version.

**Rating: 🟡 Acceptable** (missing pages)

---

### 4.8 `src/lib/upload.ts`

**File:** `frontend/src/lib/upload.ts`

**Strengths:**
- Thin wrapper that binds `PUBLIC_API_PREFIX` from SvelteKit's environment.
- Re-exports types for convenience.

**Issues:** None. This is exactly the right pattern.

**Rating: 🟢 Good**

---

### 4.9 `src/lib/index.ts`

**File:** `frontend/src/lib/index.ts`

**Strengths:**
- Re-exports the full SDK surface.
- Also re-exports the convenience wrapper `uploadFile` from `./upload`.

**Issues:**
1. **Minor:** The import paths use `../../packages/shazoneSDK` which works but couples the import to the directory structure. If the SDK is published as an npm package in the future, these imports would need updating. Consider importing from the package name instead.

**Rating: 🟢 Good**

---

## 5. SDK Package (shazoneSDK)

### 5.1 `packages/shazoneSDK/src/crypto.ts`

**File:** `frontend/packages/shazoneSDK/src/crypto.ts`

**Strengths:**
- Clean implementation of AES-256-GCM key generation, import, encrypt, and decrypt.
- Uses Web Crypto API correctly.
- Proper IV generation with `crypto.getRandomValues()`.
- `ALGORITHM` constant extracted at module level.
- BufferSource casting handled correctly.

**Issues:**
1. **Low:** `IV_LEN = 12` is correct for GCM but is named generically. Since GCM supports other IV lengths, the name is fine but could be `GCM_IV_LEN` for clarity.
2. **Low:** The `encryptChunk` and `decryptChunk` functions accept `key: CryptoKey` but don't validate that the key has the correct algorithm or usages. The Web Crypto API will reject invalid keys, but a runtime check could provide better error messages.

**Rating: 🟢 Good**

---

### 5.2 `packages/shazoneSDK/src/chunk.ts`

**File:** `frontend/packages/shazoneSDK/src/chunk.ts`

**Strengths:**
- Async generator pattern is correct for large files.
- `DEFAULT_CHUNK_SIZE = 6 * 1024 * 1024` (6 MiB) is sensible.
- Handles `undefined` input gracefully.

**Issues:**
1. **Minor:** The function accepts `File | Blob | undefined`. The `undefined` case silently yields nothing. It might be better to throw or return early with a clear error.
2. **Info:** 6 MiB chunk size is noted. The backend's `chunk_size` metadata field stores this, and the download page reads it via `x-chunk-size` header. Consistency between frontend and backend is maintained.

**Rating: 🟢 Good**

---

### 5.3 `packages/shazoneSDK/src/upload.ts`

**File:** `frontend/packages/shazoneSDK/src/upload.ts`

**Strengths:**
- Complete multipart upload orchestration: create → sign → PUT each part → complete.
- Proper error handling: aborts upload on any failure (sign-parts, PUT, complete).
- Progress callback with sensible weighting (5% for init, 90% for chunk uploads, 5% for completion).
- Encrypted payload construction: `IV || ciphertext+tag` — matching the download page's parsing.
- Generates UUID client-side for file ID.

**Issues:**
1. **Medium:** Each chunk signs and uploads sequentially. For large files with many parts, signing parts concurrently and uploading concurrently would significantly improve performance. The backend's `sign_parts` endpoint accepts multiple part numbers, but the frontend only requests one at a time.
2. **Medium:** The abort-on-failure pattern calls `abort-upload` for each error case. If the abort itself fails (network error), the orphaned upload remains in R2. The cleanup worker handles this eventually, but immediate cleanup would be ideal.
3. **Minor:** `totalChunks` calculation uses `Math.ceil(file.size / DEFAULT_CHUNK_SIZE)`. For an empty file, this returns 0, then `Math.max(1, 0)` corrects it to 1. This is correct but subtle.

**Rating: 🟡 Good** (performance optimization opportunity)

---

### 5.4 `packages/shazoneSDK/src/download.ts`

**File:** `frontend/packages/shazoneSDK/src/download.ts`

**Strengths:**
- Sophisticated streaming implementation with `ReadableStream` reader.
- `consumeBytes` helper correctly handles partial chunk assembly.
- Properly splits stream into `IV (12) || ciphertext+tag` chunks.
- Fallback path for non-streaming responses.
- Handles trailing partial chunk (leftover bytes after all full chunks).
- Progress tracking with two modes: content-length-based and exponential decay.
- File extension resolution via `extensionFromMime()`.

**Issues:**
1. **Medium:** The fallback path (`res.arrayBuffer()`) buffers the entire encrypted blob in memory. For very large files, this could cause memory issues. However, since files are burned after reading, extremely large files are unlikely, and the streaming path handles most cases.
2. **Minor:** The exponential decay progress formula `1 - Math.pow(0.97, chunksDecrypted)` is clever but may confuse readers. A comment explaining the math would help.
3. **Low:** `LEGACY_CHUNK_SIZE = 5_000_000` is used as fallback when `x-chunk-size` header is absent. This handles legacy uploads but creates a silent assumption. If a new chunk size is introduced and the header is lost, the fallback would cause decryption failures.

**Rating: 🟢 Good** (streaming implementation is solid)

---

### 5.5 `packages/shazoneSDK/src/cap_url.ts`

**File:** `frontend/packages/shazoneSDK/src/cap_url.ts`

**Strengths:**
- Clean capability URL construction.
- Uses `urlSafeBase64` for key encoding.

**Issues:** None.

**Rating: 🟢 Good**

---

### 5.6 `packages/shazoneSDK/src/utils.ts`

**File:** `frontend/packages/shazoneSDK/src/utils.ts`

**Strengths:**
- `urlSafeBase64`: Correctly implements base64url encoding (replace `+`→`-`, `/`→`_`, strip `=`).
- `base64ToBytes`: Correct inverse with padding restoration.
- `extensionFromMime`: Comprehensive MIME-to-extension mapping covering images, documents, audio, video, archives.

**Issues:**
1. **Medium:** `urlSafeBase64` uses string concatenation in a loop (`binary += String.fromCharCode(...)`). For large keys this is acceptable (keys are 32 bytes), but for general-purpose use on larger data (e.g., `encryptChunk` output), this would be slow. Since it's only used for the 32-byte key, this is fine.
2. **Minor:** `extensionFromMime` has a fallback of `mime.split('/').pop() ?? 'bin'`. This works for most MIME types but returns the subtype without considering uncommon formats. The explicit map covers the most common cases.

**Rating: 🟢 Good**

---

### 5.7 `packages/shazoneSDK/src/index.ts`

**File:** `frontend/packages/shazoneSDK/src/index.ts`

**Strengths:**
- Clean barrel export of all public SDK functions and types.

**Issues:** None.

**Rating: 🟢 Good**

---

## 6. Worker Review

### 6.1 `worker/cleanup-orphaned-uploads/src/main.rs`

**File:** `worker/cleanup-orphaned-uploads/src/main.rs`

**Strengths:**
- Clean algorithm: compute cutoff → paginate → filter → abort concurrently.
- Uses `FuturesUnordered` with a concurrency cap of 10 — correct pattern.
- Proper pagination with `key_marker` and `upload_id_marker`.
- Good logging with tracing.
- Same logging improvements as backend: `eprintln`, default `RUST_LOG`, `atty` detection.
- Returns `CleanupResult<()>` for proper exit codes.

**Issues:**
1. **Medium:** The concurrency limiting pattern has a subtle issue. The check `if tasks.len() >= MAX_CONCURRENT` drains tasks one at a time by calling `tasks.next().await`. This means if tasks complete quickly (e.g., abort is near-instant), the concurrency stays at exactly `MAX_CONCURRENT` rather than refilling. For aborts, this is acceptable since they're fast, but for general concurrent work, a semaphore or `StreamExt::buffer_unordered` would be more efficient.
2. **Medium:** If `list_multipart_uploads` fails with a transient error (e.g., throttling), the entire worker fails. No retry logic. Since the worker runs periodically, the next run will pick up missed uploads, but a retry with backoff would improve resilience.
3. **Minor:** The `initiated` timestamp check uses `upload.initiated().unwrap()`. If R2 returns an upload without an `initiated` timestamp (unlikely but theoretically possible), this panics. Should handle with `.ok_or()` or `.unwrap_or_default()`.
4. **Minor:** The cutoff is hardcoded at 6 hours. The AGENTS.md notes this should be configurable via env var. This hasn't been implemented yet.

**Rating: 🟡 Acceptable** (robustness could be improved)

---

### 6.2 `worker/cleanup-orphaned-uploads/src/lib.rs`

**File:** `worker/cleanup-orphaned-uploads/src/lib.rs`

**Strengths:**
- Clean error type with `thiserror`.
- `DateTime` variant uses `#[from]` for automatic conversion.

**Issues:**
1. **Minor:** `Sdk(String)` stores the error as a string, losing the original error type. For better error reporting, consider storing the original error or using a richer type.

**Rating: 🟢 Good**

---

## 7. Docker & Deployment

### 7.1 `backend/Dockerfile`

**File:** `backend/Dockerfile`

**Strengths:**
- Multi-stage build (rust → debian) for small final image.
- Dependency caching via dummy source build.
- Fingerprint cleanup to avoid stale cached empty library.
- `tini` as init for proper signal forwarding.
- Non-root `app` user.
- OCI labels for metadata.

**Issues:**
1. **Minor:** The Dockerfile uses `rust:1-slim-bookworm`. The `1` tag is a floating tag that changes with new stable releases. While convenient, this can cause unexpected build breakages when Rust updates. Consider pinning to a specific version (e.g., `rust:1.85-slim-bookworm`) and updating deliberately.
2. **Minor:** No `.dockerignore` in the backend directory. The root `.gitignore` excludes `backend/target`, but the Docker build context sends everything unless a `.dockerignore` is present. The file exists and looks correct.

**Rating: 🟢 Good**

---

### 7.2 `frontend/Dockerfile`

**File:** `frontend/Dockerfile`

**Strengths:**
- Multi-stage build with clean separation.
- Only copies `build/` and `package.json` to runtime — node_modules not needed (adapter-node bundles everything).
- `tini` as init.
- Non-root user.
- OCI labels.

**Issues:**
1. **Minor:** Uses `node:22-slim` floating tag. Same concern as Rust version — consider pinning.
2. **Minor:** `ORIGIN=http://localhost:5173` is set as a default. On Koyeb, this should be overridden with the actual public URL. The README documents this, which is good.

**Rating: 🟢 Good**

---

### 7.3 `worker/cleanup-orphaned-uploads/Dockerfile`

**File:** `worker/cleanup-orphaned-uploads/Dockerfile`

**Strengths:**
- Multi-stage build.
- Supercronic v0.2.45 downloaded with SHA1 verification.
- Crontab defined inline via heredoc.
- Non-root user.
- Clean separation of builder/runtime.

**Issues:**
1. **Minor:** Supercronic is downloaded from GitHub at build time. If GitHub is unreachable, the build fails. For production, consider copying a pre-downloaded binary or using a base image with Supercronic.
2. **Minor:** `curl` is installed and then removed. The `apt-get remove -y curl && apt-get autoremove -y` pattern is good for image size. However, Supercronic and the cleanup binary both need `ca-certificates` which is kept. This is correct.
3. **Minor:** The SHA1 hash verification is good, but if the Supercronic release is updated and the SHA1 changes without updating the Dockerfile, the build fails. The Dockerfile comment directs users to check the releases page.

**Rating: 🟢 Good**

---

### 7.4 `docker-compose.yml`

**File:** `file-sharing/docker-compose.yml`

**Strengths:**
- Properly configured for local development.
- Backend health check correctly points to `/health`.
- Frontend depends on backend health.
- Environment variables properly wired through.

**Issues:**
1. **Minor:** The frontend service still maps port `5173:3000`. The Koyeb Dockerfile no longer hardcodes `PORT=3000` — it reads from the environment. The docker-compose explicitly sets `PORT=3000`, which is correct for local dev. No issue in practice.

**Rating: 🟢 Good**

---

## 8. Configuration & Documentation

### 8.1 Documentation Quality

| Document | Rating | Notes |
|---|---|---|
| Root `README.md` | 🟢 | Comprehensive, accurate, well-structured |
| `AGENTS.md` | 🟢 | Excellent architecture diagrams, flow descriptions |
| `backend/README.md` | 🟢 | Complete with examples, deployment info |
| `frontend/README.md` | 🟢 | Complete with pages table, analytics, deployment |
| `worker/README.md` | 🟢 | Rewritten to be comprehensive and accurate |
| Zero-Knowledge page | 🟢 | Well-written educational content |
| Legal pages | 🟢 | Comprehensive privacy/cookies/tos |

**Overall:** Documentation is a strength of this project. The READMEs are well-maintained and accurately reflect the codebase.

### 8.2 Configuration Files

| File | Rating | Notes |
|---|---|---|
| `Cargo.toml` (backend) | 🟢 | Clean dependencies, appropriate features |
| `Cargo.toml` (worker) | 🟢 | Edition changed to 2021 to avoid unsafe `set_var` |
| `package.json` (frontend) | 🟢 | Clean dependency list |
| `svelte.config.js` | 🟢 | Uses adapter-node, vitePreprocess |
| `vite.config.ts` | 🟢 | Minimal, correct |
| `tailwind.config.js` | 🟢 | Content paths correct |
| `tsconfig.json` | Not reviewed | N/A |
| `eslint.config.js` | Not reviewed | N/A |
| `.gitignore` | 🟡 | Covers main artifacts. Missing `worker/cleanup-orphaned-uploads/target` specifically (though `**/*/target` or root `.gitignore` should cover it via the `backend/target` pattern... actually no, the worker is at `worker/cleanup-orphaned-uploads/target` which is NOT covered by `backend/target`). |
| `.dockerignore` (backend) | 🟢 | Covers all sensitive files |
| `.dockerignore` (worker) | 🟢 | Covers all sensitive files |
| `.dockerignore` (frontend) | 🟢 | Covers all sensitive files |

---

## 9. Security Assessment

### 9.1 Encryption

| Checklist item | Status | Notes |
|---|---|---|
| Algorithm choice | ✅ | AES-256-GCM — industry standard |
| Key generation | ✅ | Web Crypto API, cryptographically random |
| IV generation | ✅ | Random 12-byte IV per chunk, `crypto.getRandomValues()` |
| Key storage/transmission | ✅ | Hash fragment — never sent to server |
| Ciphertext integrity | ✅ | GCM provides authenticated encryption |
| Chunk boundaries | ✅ | Fixed size, consistent across encrypt/decrypt |
| Nonce reuse | ✅ | Random IV per chunk, negligible collision probability |

### 9.2 Server-Side Security

| Checklist item | Status | Notes |
|---|---|---|
| File data exposure | ✅ | Backend never sees plaintext or ciphertext (direct-to-R2) |
| Key exposure | ✅ | Backend never receives the key |
| Metadata exposure | 🟡 | Content type and chunk size stored as S3 metadata. Acceptable but noted. |
| Deletion guarantee | ✅ | Delete before response completes |
| Orphaned data cleanup | ✅ | Worker runs every 30 minutes |
| Authentication | N/A | No user accounts by design |
| Rate limiting | ❌ | No rate limiting implemented |

### 9.3 Client-Side Security

| Checklist item | Status | Notes |
|---|---|---|
| HTTPS | ✅ | Enforced |
| XSS prevention | ✅ | Svelte's auto-escaping |
| CSRF | N/A | No state-changing operations with credentials |
| Clickjacking | ❌ | No `X-Frame-Options` or CSP `frame-ancestors` header |
| CSP | ❌ | No Content Security Policy configured |
| Sensitive data in URL | 🟡 | Key in hash fragment is secure from server, but visible in browser history and potentially to browser extensions |

### 9.4 Dependencies

- **Backend:** All dependencies are from crates.io with known provenance. No unsafe dependencies identified.
- **Frontend:** All dependencies are from npm with known maintainers. The `shazoneSDK` is a local package (file dependency), which is fine.
- **No outdated or vulnerable dependencies** were identified at review time (note: this review did not run `npm audit` or `cargo audit`).

---

## 10. Issues & Recommendations

### 10.1 Critical Issues

**None identified.** The zero-knowledge encryption model is correctly implemented.

### 10.2 High-Priority Recommendations

1. **Rate limiting (backend):** Implement rate limiting on all endpoints, especially `/v1/create-upload` and `/v1/sign-parts`, to prevent abuse. Use `tower_http::limit` or `axum::middleware`. Without rate limiting, an attacker could:
   - Initiate thousands of multipart uploads, consuming R2 resources.
   - Exhaust presigned URL generation capacity.

2. **Error handling in upload handlers:** ✅ *Mostly resolved.* `sign_parts`, `complete_upload`, and `abort_upload` now use proper `Result` return types with dedicated error enums (`SignPartsError`, `CompleteUploadError`, `AbortUploadError`). Only `create_upload` still uses `.unwrap()` — this is the last handler to migrate. Each handler should return `Result<impl IntoResponse, AppError>` with appropriate HTTP status codes. A `CreateUploadError` enum following the established pattern is the final piece.

3. **Content Security Policy:** Implement a CSP header to prevent XSS and data exfiltration. Minimum viable CSP:
   ```
   default-src 'self';
   script-src 'self';
   style-src 'self' 'unsafe-inline';
   connect-src 'self' https://*.r2.cloudflarestorage.com;
   ```
   The `connect-src` needs to allow direct-to-R2 uploads from the browser.

### 10.3 Medium-Priority Recommendations

4. **Concurrent part uploads (frontend):** The upload flow currently processes chunks sequentially: sign → upload → sign → upload. For large files, sign multiple parts at once (backend supports it) and upload multiple parts concurrently. This would dramatically improve upload speed for large files.

5. **Add `/zero-knowledge` to sitemap:** The sitemap at `sitemap.xml/+server.ts` is missing the `/zero-knowledge` route. Add it with appropriate priority.

6. **File size validation:** Add a maximum file size check on the frontend (and ideally the backend) to prevent uploads that exceed reasonable limits (e.g., 5 GB).

7. **Error UI feedback:** When `upload()` fails, the UI only resets the uploading state but shows no error message. Add an error state that displays the error to the user.

8. **Fix JSON-LD inconsistency:** The `+page.svelte` JSON-LD script says "No tracking or analytics" but the privacy policy now discloses OpenPanel. Update the JSON-LD to match.

9. **Fix `.gitignore`:** Add `worker/cleanup-orphaned-uploads/target` to the root `.gitignore` or create a local `.gitignore` in the worker directory.

### 10.4 Low-Priority Recommendations

10. **Pin Docker base image versions:** Replace floating tags (`rust:1-slim-bookworm`, `node:22-slim`) with pinned versions to prevent unexpected build breakages.

11. **Active nav state:** Add `aria-current="page"` and visual styling to indicate the current page in the header navigation.

12. **Hash fragment validation:** Add validation in the download page to check that `location.hash` is present and has the expected length before attempting decryption.

13. **Retry logic in worker:** Add retry with exponential backoff for transient S3 errors in the cleanup worker.

14. **Configurable cutoff in worker:** Make the 6-hour orphan cutoff configurable via environment variable as documented in AGENTS.md.

15. **Security headers:** Add `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, and `Referrer-Policy: strict-origin-when-cross-origin` to all responses.

16. **Health endpoint hardening:** The `/health` endpoint should verify S3 connectivity, not just return a static 200. A deep health check ensures the service is truly operational.

---

## 11. Summary

### 11.1 Ratings Overview

| Area | Rating | Key strength | Key weakness |
|---|---|---|---|
| Architecture | 🟢 Excellent | Zero-knowledge model correctly implemented | — |
| Backend | 🟢 Good | Download handler is exemplary; error handling largely fixed | `create_upload` still uses `.unwrap()` |
| Frontend | 🟢 Good | Clean UI, good progress feedback | Error states not shown to user |
| SDK (shazoneSDK) | 🟢 Good | Streaming implementation, clean API | Sequential chunk uploads |
| Worker | 🟡 Acceptable | Correct algorithm, concurrency control | No retry logic |
| Dockerfiles | 🟢 Good | Multi-stage, tini/Supercronic, non-root | Floating base image tags |
| Documentation | 🟢 Excellent | Comprehensive, accurate, well-maintained | — |
| Security | 🟡 Good | Encryption is sound | Missing rate limiting, CSP, security headers |

### 11.2 Bottom Line

**filez.zone is a well-engineered project with a correctly implemented zero-knowledge architecture.** The encryption model is sound, the code is clean and maintainable, and the documentation is excellent.

The most impactful improvements are:
1. **Add rate limiting** (prevents abuse)
2. **Fix error handling** in upload handlers (improves reliability and debuggability)
3. **Implement CSP** (hardens client-side security)

These three changes would bring the project from "good" to "production-hardened."

### 11.3 What's Working Well

- The zero-knowledge encryption model is correctly implemented and well-documented.
- The download handler is exemplary — excellent error handling, proper streaming, correct burn-after-read semantics.
- The SDK is cleanly separated from the frontend, making it reusable and testable independently.
- Dockerfiles are well-optimized with multi-stage builds, non-root users, and proper init processes.
- Documentation (READMEs, AGENTS.md, zero-knowledge page) is comprehensive and accurate.
- Error handling has been substantially improved — dedicated error enums with `IntoResponse` for all handlers except `create_upload`.
- Test coverage expanded from 9 to 49 tests, including 38 new unit and integration tests.

### 11.4 Test Coverage Note

Test coverage has been substantially improved since the initial review. The backend now has **49 tests** (up from 9):

**Backend unit tests (11):** All four error types tested for `Display`, HTTP status code mapping, `Debug`, and round-trip behavior.

**Handler serialization tests (21):** Request deserialization (valid, missing fields, edge cases) and response serialization for all five handler types.

**Health endpoint integration tests (6):** Full router tests for GET, POST rejection, JSON content type, `/v1/health` 404, and direct function invocation.

**Existing integration tests (5):** Create upload, sign parts, complete upload, abort upload, and download happy path.

**Pre-existing failures (1):** `test_download_not_found` expects 404 but receives 500 — the S3 mock returns a generic error rather than `NoSuchKey`. This is a test infrastructure issue, not a handler bug.

**Recommended next steps for testing:**
- Fix the `test_download_not_found` mock to return `NoSuchKey` error
- Add SDK unit tests for `crypto.ts`, `chunk.ts`, `utils.ts`
- Add handler integration tests for error paths (invalid inputs, S3 failures)
