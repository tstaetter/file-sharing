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
export declare function downloadFile(apiPrefix: string, fileId: string, rawKey: Uint8Array, onProgress?: ProgressCallback): Promise<DownloadResult>;
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
export declare function decryptBytes(encryptedBytes: Uint8Array, chunkSize: number, rawKey: Uint8Array, contentType: string, onProgress?: ProgressCallback): Promise<{
    blob: Blob;
    contentType: string;
}>;
//# sourceMappingURL=download.d.ts.map