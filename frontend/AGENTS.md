# AGENTS.md

## Project Overview

This is the frontend for a file-sharing service, built with **SvelteKit**. It provides a browser-based UI that enables users to:

1. **Upload files** with client-side AES-256-GCM encryption and multipart, resumable uploads directly to Cloudflare R2 via presigned URLs.
2. **Download files** exactly once — the backend deletes the object from storage after serving it ("burn after reading").
3. **Share files** via capability URLs that embed the decryption key in the URL hash fragment. The key never touches the server.

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
    │   ├── chunk.ts             ← file chunking generator
    │   ├── crypto.ts            ← AES-GCM key generation & encryption
    │   ├── upload.ts            ← multipart upload orchestrator
    │   ├── wasm.ts              ← capability URL builder (url-safe base64)
    │   └── assets/
    │       └── favicon.svg
    └── routes/
        ├── +layout.svelte       ← root layout (favicon, global CSS)
        ├── +page.svelte         ← upload page (file picker, encrypt & upload)
        └── f/
            └── [id]/
                └── +page.svelte ← download page (fetch, decrypt, save)
```

## Setup Instructions

1. **Prerequisites:** [Deno](https://deno.com/) (2.x or later).
2. **Install dependencies:**
   ```bash
   cd frontend
   deno install
   ```
3. **Environment:** The upload page and download page hardcode the backend base URL as `http://localhost:8000`. Make sure the backend is running, or update the URLs in `src/lib/upload.ts` and `src/routes/f/[id]/+page.svelte` if the backend is deployed elsewhere.

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
  Output appears in `build/` (adapter-auto).

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

- Place test files next to the source files they exercise, using the `.test.ts` or `.spec.ts` extension. For example, tests for `src/lib/crypto.ts` would go in `src/lib/crypto.test.ts`.
- Alternatively, place tests in a `__tests__` directory.
- Since most of the library code is pure TypeScript with no DOM dependencies, tests can import and call the functions directly.
- For encryption tests, use the real Web Crypto API (available in Deno and Node.js 19+) or mock `crypto.subtle` if needed.
- For upload orchestration tests, mock `fetch` to simulate backend responses without requiring a running server.

### Example Test

```typescript
// src/lib/crypto.test.ts
import { describe, it, expect } from 'vitest';
import { generateKey, encryptChunk } from './crypto';

describe('encryptChunk', () => {
  it('should encrypt a chunk and produce IV + ciphertext', async () => {
    const { key } = await generateKey();
    const plaintext = new Uint8Array([1, 2, 3, 4]);
    const { iv, data } = await encryptChunk(key, plaintext);

    expect(iv.length).toBe(12);          // AES-GCM IV
    expect(data.length).toBe(20);        // plaintext + GCM tag
    expect(data).not.toEqual(plaintext);  // encrypted ≠ plaintext
  });
});
```

## Application Flow

### Upload (`/` route)

1. User selects a file via the file picker on `+page.svelte`.
2. On click of "Encrypt & upload", `uploadFile()` in `src/lib/upload.ts` is called:
   - Generates a 256-bit AES-GCM key and a random UUID as the `file_id`.
   - Calls `POST /v1/create-upload` on the backend to initiate a multipart upload.
   - Iterates over the file in 5 MB chunks (`chunkFile()` from `src/lib/chunk.ts`).
   - Each chunk is encrypted with `encryptChunk()` from `src/lib/crypto.ts`, which prepends a 12-byte random IV followed by the ciphertext (including the 16-byte GCM authentication tag).
   - For each chunk, requests a presigned URL from `POST /v1/sign-parts` and PUTs the encrypted payload directly to R2.
   - Collects the ETag from each PUT response.
   - Calls `POST /v1/complete-upload` with the list of part numbers and ETags.
3. Returns the raw AES key bytes and file ID.
4. `createCapabilityUrl()` in `src/lib/wasm.ts` builds the shareable URL by encoding the key as URL-safe base64 (no padding) and appending it as the URL hash fragment.

### Download (`/f/[id]` route)

1. Recipient opens the capability URL. The key is extracted from `location.hash.slice(1)`.
2. `onMount` in `+page.svelte` fetches `GET /v1/f/{id}` from the backend.
3. The backend returns the encrypted blob as base64-encoded JSON, along with the original `content_type`.
4. The frontend:
   - Decodes the base64 key and imports it as a `CryptoKey`.
   - Splits the encrypted data into per-chunk `IV (12 bytes) || ciphertext+tag` segments.
   - Decrypts each chunk with Web Crypto API.
   - Concatenates plaintext chunks.
   - Creates a Blob and an object URL.
5. The user clicks "Download" to trigger a browser download. The object URL is revoked on component destroy.

### Encryption Model

- **Algorithm:** AES-256-GCM.
- **Per-chunk IVs:** Each 5 MB chunk gets its own random 12-byte IV, which is prepended to the ciphertext. This avoids needing a single nonce and allows chunks to be decrypted independently.
- **Key distribution:** The symmetric key is embedded in the URL hash fragment. Hash fragments are never sent to the server over HTTP, so the key stays client-side.
- **Backend opacity:** The backend stores and serves raw encrypted bytes. It never sees the key or plaintext.

## API Contract with Backend

The frontend expects the backend to be available at `http://localhost:8000` (hardcoded in `upload.ts` and the download page). If the backend URL changes, update both locations.

| Method | Endpoint            | Used in                  | Purpose                             |
|--------|---------------------|--------------------------|-------------------------------------|
| POST   | `/v1/create-upload`    | `upload.ts`             | Initiate multipart upload           |
| POST   | `/v1/sign-parts`       | `upload.ts`             | Get presigned URLs for parts        |
| POST   | `/v1/complete-upload`  | `upload.ts`             | Finalise multipart upload           |
| GET    | `/v1/f/:id`            | `f/[id]/+page.svelte`   | Fetch encrypted blob (burn-after-read) |

Refer to `backend/AGENTS.md` for the request/response schemas.

## Code Style & Conventions

- **Prettier** is the sole formatter. Configuration is in `.prettierrc`. Run `deno task format` before committing.
- **ESLint** with the `eslint-plugin-svelte` and `typescript-eslint` plugins handles linting. Run `deno task lint` and fix all issues.
- **TypeScript strict mode** is enabled (`"strict": true` in `tsconfig.json`). Do not weaken type safety without a documented reason.
- Use **Svelte 5 runes** (`$state`, `$derived`, `$props`, `$effect`) for all new code. The `svelte.config.js` forces runes mode project-wide.
- **$lib alias:** Import shared modules via `$lib/` (e.g., `import { uploadFile } from '$lib/upload'`). Do not use relative paths to reach into `src/lib/`.
- **File naming:** Use `kebab-case` for route directories and `.ts` / `.svelte` extensions for modules and components. Test files use `.test.ts`.
- **Environment-specific URLs:** Currently backend URLs are hardcoded. A future improvement is to use SvelteKit environment variables (`$env/static/public`) for the backend base URL.

## Common Tasks for Agents

### Adding a new library module

1. Create the new file in `src/lib/` (e.g., `src/lib/utils.ts`).
2. If it should be importable via `$lib`, export from `src/lib/index.ts`:
   ```typescript
   export { myFunction } from './utils';
   ```

### Adding a new route

1. Create a directory under `src/routes/` (e.g., `src/routes/about/`).
2. Add a `+page.svelte` file inside.
3. For dynamic routes, use `[param]` directory naming (e.g., `src/routes/file/[id]/`).
4. Access route params in the component via `page.params.<name>` from `$app/state`.
5. If the route needs a shared layout, add a `+layout.svelte` in a parent directory.

### Adding a test

1. Install vitest if not yet present: `deno add --dev npm:vitest` and add the test tasks to `deno.json` (see Testing section above).
2. Create a `.test.ts` file next to the module being tested.
3. Use `describe`, `it`, `expect` from `vitest`.
4. Mock external dependencies (`fetch`, `crypto.subtle`) as needed to keep tests fast and deterministic.
5. Run tests with `deno task test`.

### Updating the backend URL

1. Find the hardcoded URL in `src/lib/upload.ts` (used in three `fetch` calls).
2. Find the hardcoded URL in `src/routes/f/[id]/+page.svelte` (the download fetch).
3. Consider moving the base URL to a SvelteKit environment variable in `$env/static/public` for easier configuration.

### Adding a UI dependency

1. Install the package with `deno add npm:<package>` (or `deno add jsr:<package>` for JSR packages).
2. If it's a dev dependency, use `deno add --dev npm:<package>`.
3. Import it only where needed — avoid polluting the global scope.