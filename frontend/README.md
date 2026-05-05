# Frontend

Browser-based UI for the file-sharing service. Built with **SvelteKit** and **Svelte 5** (runes mode), it encrypts files client-side with AES-256-GCM and uploads them directly to Cloudflare R2 via presigned URLs. Recipients open a capability URL with the decryption key embedded in the hash fragment — the key never touches the server.

## Quick Start

### Prerequisites

- [Deno](https://deno.com/) 2.x or later

### Install

```bash
cd frontend
deno install
```

### Environment

The backend base URL is configured via `PUBLIC_API_PREFIX` in `frontend/.env` (default: `http://localhost:8000/v1`). Make sure the backend is running, or update `.env` if the backend is deployed elsewhere.

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

Test files live next to the modules they exercise (e.g., `src/lib/crypto.test.ts`).

## How It Works

### Upload (`/`)

1. User selects a file.
2. An AES-256-GCM key is generated, along with a random UUID as the file ID.
3. The backend initiates a multipart upload (`POST /v1/create-upload`).
4. The file is split into 5 MB chunks. Each chunk gets its own random 12-byte IV, is encrypted, and is uploaded directly to R2 via a presigned URL (`POST /v1/sign-parts`).
5. ETags from each PUT are sent to the backend to complete the upload (`POST /v1/complete-upload`).
6. A capability URL is built: `{base}/f/{fileId}#{url-safe-base64(key)}`.

### Download (`/f/[id]`)

1. Recipient opens the capability URL. The key is extracted from the hash fragment.
2. The encrypted blob is fetched from the backend (`GET /v1/f/{id}`), which deletes the object from R2 immediately after serving.
3. The blob is split back into per-chunk `IV + ciphertext` segments and decrypted with the Web Crypto API.
4. Plaintext chunks are assembled into a Blob and triggered as a browser download.

### Encryption Model

- **Algorithm:** AES-256-GCM.
- **Per-chunk IVs:** Each 5 MB chunk gets a unique random 12-byte IV prepended to the ciphertext.
- **Key distribution:** The symmetric key is embedded in the URL hash fragment, which is never transmitted over HTTP.
- **Backend opacity:** The backend stores and serves raw encrypted bytes. It never sees the key or plaintext.

## Project Structure

```
src/
├── app.css                  ← Tailwind directives
├── app.d.ts                 ← ambient type declarations
├── app.html                 ← HTML shell
├── lib/
│   ├── index.ts             ← barrel export for $lib
│   ├── chunk.ts             ← file chunking generator (5 MB chunks)
│   ├── crypto.ts            ← AES-GCM key generation & per-chunk encryption
│   ├── upload.ts            ← multipart upload orchestrator
│   ├── wasm.ts              ← capability URL builder (URL-safe base64)
│   └── assets/
│       └── favicon.svg
└── routes/
    ├── +layout.svelte       ← root layout (favicon, global CSS)
    ├── +page.svelte         ← upload page (file picker, encrypt & upload)
    └── f/
        └── [id]/
            └── +page.svelte ← download page (fetch, decrypt, save)
```

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

See [AGENTS.md](AGENTS.md) for detailed conventions, design decisions, and agent guidance.