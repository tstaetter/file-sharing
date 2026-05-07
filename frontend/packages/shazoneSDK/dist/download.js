import { extensionFromMime } from './utils';
import { importKey, decryptChunk } from './crypto';
// Sizes are deliberately hardcoded to match the Rust upload backend and the
// per-chunk encryption scheme. Changing these will break interop.
const LEGACY_CHUNK_SIZE = 5_000_000;
const IV_LEN = 12;
const GCM_TAG_LEN = 16;
/**
 * Downloads and decrypts a file from the file-sharing backend.
 *
 * The backend now returns raw binary ciphertext with metadata in response
 * headers, rather than a JSON payload. This function:
 * 1. Fetches the encrypted binary stream from `GET /v1/f/{id}`.
 * 2. Reads `X-Content-Type` and `X-Chunk-Size` from response headers.
 * 3. Downloads the binary body, tracking byte-level progress when
 *    `Content-Length` is available.
 * 4. Decrypts the binary data (concatenated `IV || ciphertext || GCM tag` blocks).
 * 5. Assembles the plaintext chunks into a Blob.
 *
 * The backend deletes the file from R2 **immediately after serving it**
 * ("burn after reading"), so a file can only be downloaded once.
 *
 * @param apiPrefix  The base URL of the backend API (e.g. `"https://api.sha.zone/v1"`).
 * @param fileId     The UUID of the file to download.
 * @param rawKey     The raw AES-256 key bytes (extracted from the capability URL hash).
 * @param onProgress Optional callback invoked during download and decryption.
 *                    Receives a number between 0 (just started) and 1 (fully decrypted).
 *
 * @example
 * ```ts
 * import { downloadFile } from 'shazoneSDK';
 *
 * // Key extracted from the URL hash fragment
 * const rawKey = base64ToBytes(location.hash.slice(1));
 * const id = page.params.id; // from routing
 * const { blob, fileName } = await downloadFile('https://api.sha.zone/v1', id, rawKey, (p) => console.log(`${(p * 100).toFixed(0)}%`));
 *
 * const a = document.createElement('a');
 * a.href = URL.createObjectURL(blob);
 * a.download = fileName;
 * a.click();
 * ```
 */
export async function downloadFile(apiPrefix, fileId, rawKey, onProgress) {
    // Milestone-based progress:
    //   0.00 — started
    //   0.05 — response headers received
    //   0.05–0.85 — binary data downloaded (byte-level if Content-Length available)
    //   0.85–0.95 — decrypting chunks
    //   0.95 — assembling result
    //   1.00 — done
    onProgress?.(0);
    const res = await fetch(`${apiPrefix}/f/${fileId}`);
    if (!res.ok) {
        throw new Error(`failed to fetch file: ${res.status} ${res.statusText}`);
    }
    onProgress?.(0.05);
    // Read metadata from response headers
    const contentType = res.headers.get('x-content-type') || 'application/octet-stream';
    const chunkSizeHeader = res.headers.get('x-chunk-size');
    const chunkSize = chunkSizeHeader ? parseInt(chunkSizeHeader, 10) : LEGACY_CHUNK_SIZE;
    // Download the encrypted binary data with progress tracking
    const contentLength = parseInt(res.headers.get('Content-Length') || '0', 10);
    let encryptedBytes;
    if (contentLength > 0 && res.body) {
        // Stream with byte-level progress tracking (5%–85%)
        const reader = res.body.getReader();
        const chunks = [];
        let downloaded = 0;
        while (true) {
            const { done, value } = await reader.read();
            if (done)
                break;
            chunks.push(value);
            downloaded += value.length;
            onProgress?.(0.05 + 0.8 * (downloaded / contentLength));
        }
        // Combine chunks into a single Uint8Array
        encryptedBytes = new Uint8Array(downloaded);
        let offset = 0;
        for (const chunk of chunks) {
            encryptedBytes.set(chunk, offset);
            offset += chunk.length;
        }
    }
    else {
        // No Content-Length, download all at once
        const buffer = await res.arrayBuffer();
        encryptedBytes = new Uint8Array(buffer);
    }
    onProgress?.(0.85);
    // Decrypt phase: 85%–95%
    const { blob } = await decryptBytes(encryptedBytes, chunkSize, rawKey, contentType, (p) => onProgress?.(0.85 + 0.1 * p));
    onProgress?.(0.95);
    const ext = extensionFromMime(contentType);
    const fileName = `download.${ext}`;
    onProgress?.(1);
    return { blob, fileName };
}
/**
 * Decrypts raw binary ciphertext (concatenated `IV || ciphertext || GCM tag` blocks)
 * into a plaintext Blob.
 *
 * This is a lower-level utility useful when you already have the raw encrypted
 * bytes — for example, from a custom fetch pipeline or for testing.
 *
 * @param encryptedBytes  The raw binary ciphertext to decrypt.
 * @param chunkSize       The plaintext chunk size in bytes used during upload.
 * @param rawKey          The raw AES-256 key bytes.
 * @param contentType     The MIME type to assign to the resulting Blob.
 * @param onProgress      Optional callback invoked after each chunk is decrypted.
 *                         Receives a number between 0 and 1.
 * @returns The assembled plaintext Blob and the original content type.
 *
 * @example
 * ```ts
 * import { decryptBytes } from 'shazoneSDK';
 *
 * const res = await fetch('https://api.sha.zone/v1/f/some-uuid');
 * const encryptedBytes = new Uint8Array(await res.arrayBuffer());
 * const contentType = res.headers.get('x-content-type') || 'application/octet-stream';
 * const chunkSize = parseInt(res.headers.get('x-chunk-size') || '5000000', 10);
 * const { blob } = await decryptBytes(encryptedBytes, chunkSize, rawKey, contentType);
 * ```
 */
export async function decryptBytes(encryptedBytes, chunkSize, rawKey, contentType, onProgress) {
    const cryptoKey = await importKey(rawKey);
    const encryptedChunkSize = IV_LEN + chunkSize + GCM_TAG_LEN;
    // Split into encrypted chunks
    const chunks = [];
    let offset = 0;
    while (offset < encryptedBytes.length) {
        const remaining = encryptedBytes.length - offset;
        const size = Math.min(encryptedChunkSize, remaining);
        chunks.push(encryptedBytes.slice(offset, offset + size));
        offset += size;
    }
    // Decrypt each chunk
    const plaintextChunks = [];
    for (let i = 0; i < chunks.length; i++) {
        const chunk = chunks[i];
        const iv = chunk.slice(0, IV_LEN);
        const ciphertext = chunk.slice(IV_LEN);
        const plaintext = await decryptChunk(cryptoKey, iv, ciphertext);
        plaintextChunks.push(plaintext);
        onProgress?.((i + 1) / chunks.length);
    }
    // Assemble
    const totalLen = plaintextChunks.reduce((sum, c) => sum + c.length, 0);
    const decrypted = new Uint8Array(totalLen);
    let writeOffset = 0;
    for (const chunk of plaintextChunks) {
        decrypted.set(chunk, writeOffset);
        writeOffset += chunk.length;
    }
    const blob = new Blob([decrypted], { type: contentType });
    return { blob, contentType };
}
//# sourceMappingURL=download.js.map