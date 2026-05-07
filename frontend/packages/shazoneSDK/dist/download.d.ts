/** The JSON shape returned by `GET /v1/f/{id}`. */
export interface StoredFile {
    /** Base64-encoded ciphertext (concatenated per-chunk `IV || ciphertext` blocks). */
    data: string;
    /**
     * Base64-encoded dummy nonce. Kept for backwards compatibility; the actual
     * AES-GCM IVs are embedded at the start of each chunk inside `data`.
     */
    nonce: string;
    /** The original file's MIME type, if one was provided during upload. */
    content_type?: string | null;
    /** The plaintext chunk size in bytes used during upload, if provided by the server. */
    chunk_size?: number | null;
}
import type { ProgressCallback } from './upload';
/** The result of a successful download and decryption. */
export interface DownloadResult {
    /** The decrypted file contents as a Blob, ready to be saved to disk. */
    blob: Blob;
    /**
     * A suggested file name derived from the MIME type (e.g. `"download.png"`).
     * Callers may override this.
     */
    fileName: string;
}
/**
 * Downloads and decrypts a file from the file-sharing backend.
 *
 * This function:
 * 1. Fetches the encrypted blob from `GET /v1/f/{id}`.
 * 2. Splits the base64-encoded data into per-chunk `IV || ciphertext+tag` blocks.
 * 3. Decrypts each chunk with the provided AES key.
 * 4. Assembles the plaintext chunks into a Blob.
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
export declare function downloadFile(apiPrefix: string, fileId: string, rawKey: Uint8Array, onProgress?: ProgressCallback): Promise<DownloadResult>;
/**
 * Decrypts an already-fetched `StoredFile` payload into a Blob.
 *
 * This is a lower-level function useful when you already have the JSON
 * response in memory. It splits the `data` byte stream into per-chunk
 * `IV (12 bytes) || ciphertext || GCM tag (16 bytes)` blocks and
 * decrypts each one independently.
 *
 * @param stored    The JSON payload returned by `GET /v1/f/{id}`.
 * @param rawKey    The raw AES-256 key bytes.
 * @param onProgress Optional callback invoked after each chunk is decrypted.
 *                    Receives a number between 0 and 1.
 * @returns The assembled plaintext Blob and the original content type.
 *
 * @example
 * ```ts
 * import { decryptFile } from 'shazoneSDK';
 *
 * const response = await fetch('https://api.sha.zone/v1/f/some-uuid');
 * const stored = await response.json();
 * const { blob } = await decryptFile(stored, rawKey);
 * ```
 */
export declare function decryptFile(stored: StoredFile, rawKey: Uint8Array, onProgress?: ProgressCallback): Promise<{
    blob: Blob;
    contentType: string;
}>;
//# sourceMappingURL=download.d.ts.map