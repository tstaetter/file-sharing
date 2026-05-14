# AGENTS.md

## Project Overview

This is the frontend for a file-sharing service, built with **SvelteKit**. It provides a browser-based UI that enables users to:

1. **Upload files** with client-side AES-256-GCM encryption and multipart, resumable uploads directly to Cloudflare R2 via presigned URLs.
2. **Download files** exactly once — the backend deletes the object from storage after serving it ("burn after reading").
3. **Share files** via capability URLs that embed the decryption key in the URL hash fragment. The key never touches the server.
4. **Save and manage capability URLs** — links are auto-saved after upload and stored in the browser's localStorage. No authentication required for the saved URL list.
5. **Authenticate** with JWT-based login/register, with the token persisted in localStorage and sent via `Authorization: Bearer` header for protected endpoints.

All encryption and decryption happens in the browser using the Web Crypto API. The frontend communicates with a Rust/Axum backend (documented separately in `backend/AGENTS.md`).

## Tech Stack

| Layer             | Technology                                              |
|-------------------|---------------------------------------------------------|
| Framework         | [SvelteKit 2](https://svelte.dev/docs/kit)              |
| UI library        | [Svelte 5](https://svelte.dev/docs/svelte) (runes mode) |
| Language          | [TypeScript 6](https://www.typescriptlang.org/)         |
| Build tool        | [Vite 8](https://vite.dev/)                             |
| CSS               | [TailwindCSS 3](https://tailwindcss.com/) + [PostCSS](https://postcss.org/) |
| Linting           | [ESLint 10](https://eslint.org/) + [typescript-eslint](https://typescript-eslint.io/) |
| Formatting        | [Prettier 3](https://prettier.io/) + [prettier-plugin-svelte](https://github.com/sveltejs/prettier-plugin-svelte) |
| Package manager   | [Deno](https://deno.com/)                               |
| Testing           | [Vitest](https://vitest.dev/)                           |
| Type checking     | [svelte-check](https://www.npmjs.com/package/svelte-check) |

## Directory Structure

```
frontend/
├── AGENTS.md                    ← this file
├── README.md
├── deno.json                    ← Deno config & task definitions
├── package.json                 ← npm compat (may be removed after full Deno migration)
├── .prettierrc
├── .prettierignore
├── eslint.config.js
├── svelte.config.js
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js
├── postcss.config.js
├── static/
│   └── (static assets served as-is)
└── src/
    ├── app.css                  ← Tailwind directives
    ├── app.d.ts                 ← ambient type declarations
    ├── app.html                 ← HTML shell
    ├── lib/
    │   ├── index.ts             ← barrel export for $lib
    │   ├── auth.svelte.ts       ← Svelte 5 runes auth store (JWT, localStorage, reactive state)
    │   ├── savedUrls.svelte.ts  ← localStorage-based saved URLs store (reactive runes, no auth)
    │   ├── upload.ts            ← multipart upload orchestrator wrapper
    │   └── assets/
    │       └── logo.webp
    └── routes/
        ├── +layout.svelte       ← root layout (header nav with auth, footer, global CSS)
        ├── +page.svelte         ← upload page (file picker, encrypt & upload, auto-save)
        ├── f/
        │   └── [id]/
        │       └── +page.svelte ← download page (fetch, decrypt, save)
        ├── urls/
        │   └── +page.svelte     ← saved URLs list page (paginated, copy, open, delete — no auth required)
        ├── health/
        │   └── +server.ts       ← health check endpoint
        ├── login/
        │   └── +page.svelte     ← login page
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

The SDK logic (encryption, chunking, capability URL building) lives in `packages/shazoneSDK/` and is imported as a workspace dependency. The thin wrapper in `src/lib/upload.ts` binds the backend URL from the environment.

## Setup Instructions

1. **Prerequisites:** [Deno](https://deno.com/) (2.x or later).
2. **Install dependencies:**
   ```bash
   cd frontend
   deno install
   ```
3. **Environment:** Create a `frontend/.env` file (or set the variable in your deployment environment):

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

## How to Run

- **Development server:**
  ```bash
  deno task dev
  ```
  Opens at `http://localhost:5173` by default. Hot module replacement is enabled.

- **Production build:**
  ```bash
  deno task build
  ```
  Output appears in `build/` (adapter-node).

- **Preview production build locally:**
  ```bash
  deno task preview
  ```

- **Type checking (non-blocking in dev, useful in CI):**
  ```bash
  deno task check
  ```

- **Lint (Prettier + ESLint):**
  ```bash
  deno task lint
  ```

- **Format (Prettier):**
  ```bash
  deno task format
  ```

## Testing

We use **Vitest** for unit and integration tests. Vitest is not yet listed as a dependency; add it with:

```bash
deno add --dev npm:vitest
```

Then define the following tasks in `deno.json`:

```json
{
  "tasks": {
    "test": "vitest run",
    "test:watch": "vitest",
    "test:ui": "vitest --ui"
  }
}
```

### Common Commands

- Run all tests once:
  ```bash
  deno task test
  ```
- Run tests in watch mode (re-runs on file changes):
  ```bash
  deno task test:watch
  ```
- Run tests with the Vitest UI:
  ```bash
  deno task test:ui
  ```

### Writing Tests

- Place test files next to the source files they exercise, using the `.test.ts` or `.spec.ts` extension. For example, tests for `src/lib/savedUrls.ts` would go in `src/lib/savedUrls.test.ts`.
- Alternatively, place tests in a `__tests__` directory.
- Since most of the library code is pure TypeScript with no DOM dependencies, tests can import and call the functions directly.
- For encryption tests, use the real Web Crypto API (available in Deno and Node.js 19+) or mock `crypto.subtle` if needed.
- For upload orchestration and saved URL tests, mock `fetch` to simulate backend responses without requiring a running server.

## Application Flow

### Upload (`/` route)

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
5. **Auto-save:** After a successful upload:
   - Calls `localUrls.add(link, file.name)` from `src/lib/savedUrls.svelte.ts`.
   - Uses the original filename as the URL title.
   - Stores directly in localStorage — instant, no network round-trip.
   - Shows inline feedback: "Saved to your collection" with a link to `/urls`.

### Download (`/f/[id]` route)

1. Recipient opens the capability URL. The key is extracted from `location.hash.slice(1)`.
2. `onMount` in `+page.svelte` fetches `GET /v1/f/{id}` from the backend.
3. The backend returns the encrypted blob as a binary stream, along with the original `content_type` and `chunk_size` in response headers.
4. The frontend:
   - Decodes the URL-safe base64 key and imports it as a `CryptoKey`.
   - Splits the encrypted data into per-chunk `IV (12 bytes) || ciphertext+tag` segments.
   - Decrypts each chunk with the Web Crypto API.
   - Concatenates plaintext chunks.
   - Creates a Blob and triggers a browser download.

### Saved URLs (`/urls` route)

1. Users navigate to `/urls` (via nav link or auto-save confirmation). No authentication required.
2. The page reads from `localUrls.urls` — a reactive `$state` array from `src/lib/savedUrls.svelte.ts`.
3. For each saved URL, calls `checkFile(fileId)` from `src/lib/savedUrls.svelte.ts` to check if the underlying file still exists in storage (via `PUT /v1/check-file`).
4. Displays saved URLs in a paginated list with:
   - **Title** (or truncated URL if no title was set).
   - **Full URL** underneath when a title is present.
   - **Formatted save date**.
   - **"Already used" badge** — shown when the file has been consumed (check-file returns 404).
   - **Copy button** — copies the capability URL to clipboard with checkmark feedback.
   - **Open button** — opens the link in a new tab.
   - **Delete button** — removes the URL from localStorage with confirmation.
5. Client-side pagination with Previous/Next buttons and page counter.
6. Handles empty state (helpful message + upload CTA).

### Encryption Model

- **Algorithm:** AES-256-GCM.
- **Per-chunk IVs:** Each chunk gets its own random 12-byte IV prepended to the ciphertext. Chunks can be decrypted independently.
- **Key distribution:** The symmetric key is embedded in the URL hash fragment. Hash fragments are never sent to the server over HTTP, so the key stays client-side.
- **Backend opacity:** The backend stores and serves raw encrypted bytes. It never sees the key or plaintext.

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
| PUT    | `/v1/check-file`      | `urls/+page.svelte`      | Check if a file still exists in storage |

### Auth endpoints (token in request body)

| Method | Endpoint            | Used in                  | Purpose                             |
|--------|---------------------|--------------------------|-------------------------------------|
| POST   | `/v1/auth/register`    | `auth.svelte.ts`         | Register new user                   |
| POST   | `/v1/auth/login`       | `auth.svelte.ts`         | Authenticate and get JWT            |

### Protected endpoints (Bearer auth required)

These endpoints exist on the backend but are **not used by the default frontend** — the frontend stores saved URLs in localStorage instead.

| Method | Endpoint            | Used in                  | Purpose                             | Auth Header                        |
|--------|---------------------|--------------------------|-------------------------------------|------------------------------------|
| POST   | `/v1/urls`             | (not used by frontend)   | Save a capability URL               | `Authorization: Bearer <token>`    |
| GET    | `/v1/urls`             | (not used by frontend)   | List saved URLs (paginated)         | `Authorization: Bearer <token>`    |
| DELETE | `/v1/delete`           | `auth.svelte.ts`         | Delete user account                 | `Authorization: Bearer <token>`    |

Refer to `backend/AGENTS.md` for the request/response schemas.

## Code Style & Conventions

- **Prettier** is the sole formatter. Configuration is in `.prettierrc`. Run `deno task format` before committing.
- **ESLint** with the `eslint-plugin-svelte` and `typescript-eslint` plugins handles linting. Run `deno task lint` and fix all issues.
- **TypeScript strict mode** is enabled (`"strict": true` in `tsconfig.json`). Do not weaken type safety without a documented reason.
- Use **Svelte 5 runes** (`$state`, `$derived`, `$props`, `$effect`) for all new code. The `svelte.config.js` forces runes mode project-wide.
- **$lib alias:** Import shared modules via `$lib/` (e.g., `import { localUrls } from '$lib/savedUrls.svelte'`). Do not use relative paths to reach into `src/lib/`.
- **File naming:** Use `kebab-case` for route directories and `.ts` / `.svelte` / `.svelte.ts` extensions for modules and components. Use `.svelte.ts` for files that use Svelte 5 runes (`$state`, `$derived`, etc.). Test files use `.test.ts`.
- **Environment-specific URLs:** The backend base URL is configured via the `PUBLIC_API_PREFIX` environment variable in `frontend/.env`, accessible in browser-side code through `$env/static/public`.
- **Bearer token auth:** All protected API calls pass the JWT via `Authorization: Bearer <token>` header. The token is stored in `auth.token` from the reactive auth store.
- **LocalStorage state:** For browser-side persistent state that doesn't need server sync, use a `.svelte.ts` store with reactive runes (see `savedUrls.svelte.ts`).

## Common Tasks for Agents

### Adding a new library module

1. Create the new file in `src/lib/` (e.g., `src/lib/myModule.ts`).
2. If it should be importable via `$lib`, export from `src/lib/index.ts`:
   ```typescript
   export { myFunction } from './myModule';
   ```
3. For API modules, follow the pattern in `upload.ts`:
   - Import `PUBLIC_API_PREFIX` from `$env/static/public`.
   - For protected endpoints, accept a `token: string` parameter and set the `Authorization: Bearer ${token}` header.
   - Define TypeScript interfaces for request/response types.
   - Throw descriptive errors on non-ok responses.
4. For localStorage-based state, follow the pattern in `savedUrls.svelte.ts`:
   - Use `.svelte.ts` extension to enable Svelte 5 runes.
   - Create a factory function returning a singleton with reactive `$state`.
   - Provide `persistToStorage`/`loadFromStorage` helpers for the localStorage key.

### Adding a new route

1. Create a directory under `src/routes/` (e.g., `src/routes/about/`).
2. Add a `+page.svelte` file inside.
3. For dynamic routes, use `[param]` directory naming (e.g., `src/routes/file/[id]/`).
4. Access route params in the component via `page.params.<name>` from `$app/state`.
5. For protected pages (auth required), add an auth guard in `onMount`:
   ```typescript
   import { onMount } from 'svelte';
   import { auth } from '$lib/auth.svelte';
   import { goto } from '$app/navigation';

   onMount(() => {
     if (!auth.isAuthenticated) {
       goto('/login');
       return;
     }
     // fetch data...
   });
   ```

### Adding a test

1. Install vitest if not yet present: `deno add --dev npm:vitest` and add the test tasks to `deno.json` (see Testing section above).
2. Create a `.test.ts` file next to the module being tested.
3. Use `describe`, `it`, `expect` from `vitest`.
4. Mock `fetch` for API module tests to simulate backend responses without requiring a running server.
5. For component tests, use `@testing-library/svelte` or SvelteKit's test helpers.

### Updating the backend URL

1. Update the `PUBLIC_API_PREFIX` value in `frontend/.env`.
2. The value is imported by `upload.ts`, `savedUrls.svelte.ts`, and `f/[id]/+page.svelte` from `$env/static/public`, so no code changes are needed for these modules.
3. For deployments, set `PUBLIC_API_PREFIX` in the production environment or as a build arg.

### Adding a UI dependency

1. Install the package with `deno add npm:<package>` (or `deno add jsr:<package>` for JSR packages).
2. If it's a dev dependency, use `deno add --dev npm:<package>`.
3. Import it only where needed — avoid polluting the global scope.

### Working with the auth store

The auth store (`src/lib/auth.svelte.ts`) uses Svelte 5 runes for reactive state:

```typescript
import { auth } from '$lib/auth.svelte';

// Reactive state (read in components or other modules)
auth.token          // string | null — the JWT
auth.user           // { email, name } | null
auth.loading        // boolean — true during API calls
auth.error          // string | null — last error message
auth.isAuthenticated // boolean — derived from token presence and expiry

// Actions
auth.signUp(email, password, name)  // register and log in
auth.signIn(email, password)        // log in
auth.signOut()                      // clear session
auth.deleteAccount()                // delete account and sign out
auth.clearError()                   // clear error state
```

### Working with the saved URLs store

```typescript
import { localUrls, checkFile, extractFileId, type LocalUrlItem } from '$lib/savedUrls.svelte';

// Reactive state (use in Svelte components)
localUrls.urls           // LocalUrlItem[] — reactive array, newest first

// Actions
localUrls.add(url, title)   // save a URL (returns the new LocalUrlItem)
localUrls.remove(id)        // delete a URL by its ID
localUrls.reload()          // re-read from localStorage (cross-tab sync)

// File existence check (calls backend API — no auth required)
const exists = await checkFile(fileId);  // true or false

// Extract file ID from a capability URL
const fileId = extractFileId('https://filez.zone/f/abc-123#key');  // 'abc-123'
```