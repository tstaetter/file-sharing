# file-sharing

End-to-end encrypted file sharing with self-destructing downloads. Pick a file, get a link — the recipient downloads it once and it's gone forever.

## How It Works

1. **Uploader** picks a file in the browser. The frontend generates an AES-256-GCM key, encrypts the file chunk-by-chunk, and uploads the ciphertext directly to Cloudflare R2. A capability URL is produced with the decryption key embedded in the hash fragment.
2. **Recipient** opens the link. The browser downloads the encrypted blob from the backend, decrypts it locally using the key from the URL hash, and saves the plaintext. The file is deleted from R2 immediately after the first download.
3. **The server never sees the key or the plaintext.** Hash fragments are never transmitted over HTTP.

```
Browser (encrypt)  ──►  Backend (presigned URLs)  ──►  Cloudflare R2 (encrypted bytes)
Browser (decrypt)  ◄──  Backend (fetch & delete)   ◄──  Cloudflare R2 (encrypted bytes)
```

## Project Structure

| Sub-project | Path        | Language            | Description                                          |
|-------------|-------------|---------------------|------------------------------------------------------|
| Backend API | `backend/`  | Rust (Axum)         | Presigned URL generation, multipart upload orchestration, burn-after-read download |
| Frontend    | `frontend/` | TypeScript (SvelteKit) | Upload UI, client-side AES-256-GCM encryption/decryption, capability URL handling |

## Quick Start

### Prerequisites

- **Backend:** Rust toolchain (stable), [cargo-nextest](https://nexte.st/)
- **Frontend:** [Deno](https://deno.com/) 2.x or later
- **Storage:** A [Cloudflare R2](https://www.cloudflare.com/developer-platform/r2/) bucket with API credentials

### Environment Variables

Create a `.env` file in the project root:

```env
R2_ACCOUNT_ID=<cloudflare account id>
R2_ACCESS_KEY_ID=<r2 access key>
R2_SECRET_ACCESS_KEY=<r2 secret key>
R2_BUCKET=<bucket name>
```

Optionally set `RUST_LOG=info,backend=debug` for verbose backend logging.

### Start Both Services

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

### URLs

| Service  | Development URL         |
|----------|-------------------------|
| Frontend | `http://localhost:5173` |
| Backend  | `http://localhost:8000` |

## API Endpoints

| Method | Path               | Purpose                                                  |
|--------|--------------------|----------------------------------------------------------|
| POST   | `/v1/create-upload`   | Initiate a multipart upload                              |
| POST   | `/v1/sign-parts`      | Generate presigned URLs for part numbers                 |
| POST   | `/v1/complete-upload` | Finalise multipart upload with ETags                     |
| POST   | `/v1/abort-upload`    | Cancel an in-progress multipart upload                   |
| GET    | `/v1/f/:id`           | Download encrypted blob and **delete from R2**           |

For full request/response schemas and curl examples, see [backend/README.md](backend/README.md).

## Encryption

- **Algorithm:** AES-256-GCM
- **Per-chunk IVs:** Each 5 MB chunk gets its own random 12-byte IV prepended to the ciphertext
- **Key distribution:** The symmetric key is embedded in the URL hash fragment (never sent to the server)
- **Format:** Capability URLs follow the pattern `{base}/f/{uuid}#{url-safe-base64(key)}`

## Testing

```bash
# Backend (cargo-nextest)
cd backend && cargo nextest run

# Frontend (Vitest)
cd frontend && deno task test
```

## Further Reading

- [backend/README.md](backend/README.md) — backend-specific setup, API details, and tech stack
- [frontend/README.md](frontend/README.md) — frontend-specific setup, task reference, and how it works
- [AGENTS.md](AGENTS.md) — architecture diagrams, upload/download flows, conventions, and agent guidance

## License

MIT