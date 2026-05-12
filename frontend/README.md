# Frontend

Browser-based UI for the file-sharing service. Built with **SvelteKit** and **Svelte 5** (runes mode), it encrypts files client-side with AES-256-GCM and uploads them directly to Cloudflare R2 via presigned URLs. Recipients open a capability URL with the decryption key embedded in the hash fragment — the key never touches the server.

Authenticated users can save capability URLs to their collection — links are **auto-saved after upload** and browsable in a paginated list view.

## Quick Start

### Prerequisites

- [Deno](https://deno.com/) 2.x or later

### Install

```bash
cd frontend
deno install
```

### Environment

Create a `frontend/.env` file (or set the variable in your deployment environment):

```env
PUBLIC_API_PREFIX=http://localhost:8000/v1
PUBLIC_PREFIX=http://localhost:5173
```

SvelteKit requires browser-accessible environment variables to be prefixed with `PUBLIC_`. They are imported in client-side code via `$env/static/public`:

```typescript
import { PUBLIC_API_PREFIX } from '$env/static/public';
```

- `PUBLIC_API_PREFIX` — the backend API base URL (used by `upload.ts`, `savedUrls.ts`, and `f/[id]/+page.svelte`)
- `PUBLIC_PREFIX` — the frontend base URL for building capability URLs

Make sure the backend is running, or update `PUBLIC_API_PREFIX` if the backend is deployed elsewhere.

### Run

```bash
deno task dev
```

Opens at **`http://localhost:5173`** with hot module replacement.

## All Tasks

| Command              | Purpose                                  |
|----------------------|------------------------------------------|
| `deno task dev`      | Start development server                 |
| `deno task build`    | Create production build (`build/`)       |
| `deno task preview`  | Preview production build locally         |
| `deno task check`    | Type-check with `svelte-check`           |
| `deno task lint`     | Lint with Prettier + ESLint              |
| `deno task format`   | Auto-format with Prettier                |
| `deno task test`     | Run Vitest tests once                    |
| `deno task test:watch` | Run tests in watch mode                |
| `deno task test:ui`  | Run tests with Vitest UI                 |

## Testing

```bash
# Run all tests once
deno task test

# Watch mode (re-runs on file changes)
deno task test:watch

# Vitest UI
deno task test:ui
```

Test files live next to the modules they exercise (e.g., `src/lib/savedUrls.test.ts`).

## How It Works

### Upload (`/`)

1. User selects a file via the file picker on `+page.svelte`.
2. On click of "Encrypt & upload", `uploadFile()` in `src/lib/upload.ts` is called:
   - Delegates to the `shazoneSDK` `uploadFile` function with the backend URL from `PUBLIC_API_PREFIX`.
   - The SDK generates a 256-bit AES-GCM key and a random UUID as the `file_id`.
   - Calls `POST /v1/create-upload` to initiate a multipart upload.
   - Splits the file into 6 MiB chunks, encrypts each with a random IV.
   - For each chunk, requests a presigned URL from `POST /v1/sign-parts` and PUTs the encrypted payload directly to R2.
   - Collects ETags and finalises with `POST /v1/complete-upload`.
3. Returns the raw AES key bytes and file ID.
4. `createCapabilityUrl()` builds the shareable URL: `{PUBLIC_PREFIX}/f/{fileId}#{url-safe-base64(key)}`.
5. **Auto-save (authenticated users only):** After a successful upload, if the user is logged in:
   - Calls `saveUrl(link, file.name, auth.token)` from `src/lib/savedUrls.ts`.
   - Uses the original filename as the URL title.
   - Silently handles failures — the upload still succeeded, saving is a bonus.
   - Shows inline feedback: "Saving link to your collection…" while in progress, then a "Saved to your collection" link to `/urls` on success.

### Download (`/f/[id]`)

1. Recipient opens the capability URL. The key is extracted from the hash fragment.
2. The encrypted blob is fetched from the backend (`GET /v1/f/{id}`), which deletes the object from R2 immediately after serving.
3. The blob is split back into per-chunk `IV + ciphertext` segments and decrypted with the Web Crypto API.
4. Plaintext chunks are assembled into a Blob and triggered as a browser download.

### Saved URLs (`/urls`)

1. Authenticated users navigate to `/urls` (via the nav bar, user dropdown, or auto-save confirmation link).
2. The page checks `auth.isAuthenticated` on mount — redirects to `/login` if not authenticated.
3. Calls `listUrls(auth.token, page, perPage)` from `src/lib/savedUrls.ts` with the JWT in an `Authorization: Bearer <token>` header.
4. Displays saved URLs in a paginated list with:
   - **Title** (or truncated URL if no title was set).
   - **Full URL** underneath when a title is present.
   - **Formatted save date**.
   - **Copy button** — copies the capability URL to clipboard with checkmark feedback.
   - **Open button** — opens the link in a new tab.
5. Supports pagination with Previous/Next buttons and page counter.
6. Handles loading (spinner), empty (helpful message + upload CTA), and error (message with retry) states.

### Zero-Knowledge Architecture

filez.zone implements a **zero-knowledge** encryption model — the server has no technical ability to access your plaintext or encryption keys. Read the full explanation on the [Zero-Knowledge Encryption](https://filez.zone/zero-knowledge) page, or inspect the source code in this repository.

Key properties:

- **Client-side encryption:** All cryptographic operations happen in the browser via the Web Crypto API. The original file never leaves your device.
- **Hash fragment key exchange:** The encryption key is embedded in the URL hash fragment (`#key`), which browsers never transmit in HTTP requests. The server physically cannot receive it.
- **Direct-to-storage uploads:** Encrypted chunks are PUT directly to Cloudflare R2 via presigned URLs. File data never passes through the backend.
- **Burn after reading:** Files are permanently deleted from R2 after the first download. There is no second chance.

### User Authentication

filez.zone supports optional user accounts via JWT-based authentication:

- **Auth store:** `src/lib/auth.svelte.ts` — reactive Svelte 5 runes providing `token`, `user`, `loading`, `error`, and `isAuthenticated` state, plus `signIn()`, `signUp()`, `signOut()`, and `deleteAccount()` actions
- **Token storage:** JWTs persisted in `localStorage` with automatic expiry detection via JWT `exp` claim
- **Pages:** `/login` and `/register` with client-side validation, error display, and automatic redirect when already authenticated
- **API communication:** Auth endpoints use `PUBLIC_API_PREFIX` to call backend `/v1/auth/*` endpoints; protected endpoints (saved URLs) use `Authorization: Bearer <token>` header
- **Header UI:** Shows "Log in"/"Sign up" buttons when signed out; user avatar with dropdown menu (Saved URLs, Log out, Delete account) and a "Saved URLs" nav link when signed in

### Encryption Model

- **Algorithm:** AES-256-GCM.
- **Per-chunk IVs:** Each chunk gets a unique random 12-byte IV prepended to the ciphertext. Chunks can be decrypted independently.
- **Key distribution:** The symmetric key is embedded in the URL hash fragment, which is never transmitted over HTTP.
- **Backend opacity:** The backend stores and serves raw encrypted bytes. It never sees the key or plaintext.

## Project Structure

```
src/
├── app.css                  ← Tailwind directives
├── app.d.ts                 ← ambient type declarations
├── app.html                 ← HTML shell
├── lib/
│   ├── auth.svelte.ts       ← Svelte 5 runes auth store (JWT, localStorage persistence, reactive state)
│   ├── savedUrls.ts         ← API module for saved capability URLs (Bearer auth, paginated list)
│   ├── index.ts             ← barrel export for $lib
│   ├── upload.ts            ← multipart upload orchestrator wrapper (binds PUBLIC_API_PREFIX to SDK)
│   └── assets/
│       └── logo.webp
└── routes/
    ├── +layout.svelte       ← root layout (header nav with auth + saved URLs, footer, global CSS)
    ├── +page.svelte         ← upload page (file picker, encrypt & upload, auto-save for auth users)
    ├── f/
    │   └── [id]/
    │       └── +page.svelte ← download page (fetch, decrypt, save)
    ├── urls/
    │   └── +page.svelte     ← saved URLs list page (paginated, copy, open — auth required)
    ├── health/
    │   └── +server.ts       ← health check endpoint (GET /health → {"status":"ok"})
    ├── login/
    │   └── +page.svelte     ← login page with JWT auth
    ├── register/
    │   └── +page.svelte     ← registration page
    ├── zero-knowledge/
    │   └── +page.svelte     ← zero-knowledge architecture explanation
    ├── privacy/
    │   └── +page.svelte     ← privacy policy
    ├── cookies/
    │   └── +page.svelte     ← cookie policy
    ├── tos/
    │   └── +page.svelte     ← terms of service
    └── sitemap.xml/
        └── +server.ts       ← dynamic sitemap generation
```

The SDK logic (encryption, chunking, capability URL building) lives in `packages/shazoneSDK/` and is imported as a workspace dependency.

## Pages

| Route | Description |
|---|---|
| `/` | Upload page — pick a file, encrypt, and upload with progress bar and auto-save for authenticated users |
| `/f/[id]` | Download page — decrypt and save file after burn-after-read fetch |
| `/urls` | Saved URLs — paginated list of saved capability URLs (auth required) |
| `/health` | Koyeb health check — returns `{"status":"ok"}` with HTTP 200 |
| `/zero-knowledge` | Educational page explaining zero-knowledge encryption architecture |
| `/privacy` | Privacy policy — data collection, encryption, third-party services |
| `/cookies` | Cookie policy — minimal browser storage, cookieless analytics |
| `/login` | Login page — email/password form with JWT token storage in localStorage |
| `/register` | Registration page — name/email/password with client-side validation |
| `/tos` | Terms of service |

## API Contract with Backend

The frontend reads the backend base URL from `PUBLIC_API_PREFIX` in `$env/static/public` (set in `.env`). The upload wrapper, the download page, and the saved URLs module all use this variable.

### File-sharing endpoints (no auth required)

| Method | Endpoint            | Used in                  | Purpose                             |
|--------|---------------------|--------------------------|-------------------------------------|
| POST   | `/v1/create-upload`    | `upload.ts` (SDK)        | Initiate multipart upload           |
| POST   | `/v1/sign-parts`       | `upload.ts` (SDK)        | Get presigned URLs for parts        |
| POST   | `/v1/complete-upload`  | `upload.ts` (SDK)        | Finalise multipart upload           |
| POST   | `/v1/abort-upload`     | `upload.ts` (SDK)        | Cancel multipart upload             |
| GET    | `/v1/f/:id`            | `f/[id]/+page.svelte`    | Fetch encrypted blob (burn-after-read) |

### Auth endpoints (token in request body)

| Method | Endpoint            | Used in                  | Purpose                             |
|--------|---------------------|--------------------------|-------------------------------------|
| POST   | `/v1/auth/register`    | `auth.svelte.ts`         | Register new user                   |
| POST   | `/v1/auth/login`       | `auth.svelte.ts`         | Authenticate and get JWT            |
| POST   | `/v1/auth/delete`      | `auth.svelte.ts`         | Delete user account                 |

### Protected endpoints (Bearer auth required)

| Method | Endpoint            | Used in                  | Purpose                             | Auth Header                        |
|--------|---------------------|--------------------------|-------------------------------------|------------------------------------|
| POST   | `/v1/urls`             | `savedUrls.ts`           | Save a capability URL               | `Authorization: Bearer <token>`    |
| GET    | `/v1/urls`             | `savedUrls.ts`           | List saved URLs (paginated)         | `Authorization: Bearer <token>`    |

## Analytics

We use [OpenPanel](https://openpanel.dev), a self-hosted, **cookieless** analytics tool. It collects anonymous page view data (page URLs, referrer, browser type, country-level location) without cookies, without personal identifiers, and without cross-session tracking. No data is shared with external analytics providers. See the [Privacy Policy](https://filez.zone/privacy) for full details.

## Deployment

The frontend is deployed on [Koyeb](https://www.koyeb.com/) as a Web Service. The `Dockerfile` uses a multi-stage build:

1. **Builder stage:** `node:22-slim` — installs dependencies, builds the SvelteKit app with `adapter-node`
2. **Runtime stage:** `node:22-slim` — copies only the self-contained `build/` output, runs with `tini` as init

Key deployment configuration:

- **Service type:** Web Service
- **Build arg:** `PUBLIC_API_PREFIX` — the backend API URL, baked into the client bundle at build time
- **Environment variables:** `ORIGIN` — set to the public URL (e.g. `https://filez.zone`) to prevent host-header spoofing
- **PORT:** Automatically set by Koyeb; adapter-node reads it at runtime
- **Health check:** Koyeb performs TCP health checks by default; configure an HTTP check to `/health` for faster failure detection

## Tech Stack

| Component       | Technology                                          |
|-----------------|-----------------------------------------------------|
| Framework       | [SvelteKit 2](https://svelte.dev/docs/kit)          |
| UI              | [Svelte 5](https://svelte.dev/docs/svelte) (runes)  |
| Language        | [TypeScript 6](https://www.typescriptlang.org/)     |
| Build           | [Vite 8](https://vite.dev/)                         |
| CSS             | [TailwindCSS 3](https://tailwindcss.com/)           |
| Package manager | [Deno](https://deno.com/)                           |
| Testing         | [Vitest](https://vitest.dev/)                       |
| Linting         | [ESLint 10](https://eslint.org/) + typescript-eslint |
| Formatting      | [Prettier 3](https://prettier.io/) + prettier-plugin-svelte |
| Analytics       | [OpenPanel](https://openpanel.dev) (self-hosted, cookieless) |
| Auth            | JWT tokens, localStorage persistence, Bearer header auth |
| Deployment      | [Koyeb](https://www.koyeb.com/)                     |

See [AGENTS.md](AGENTS.md) for detailed conventions, design decisions, and agent guidance.