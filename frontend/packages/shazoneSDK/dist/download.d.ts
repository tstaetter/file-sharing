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
 * The backend streams raw binary ciphertext with metadata in response headers.
 * This function processes the stream **incrementally** — it buffers incoming
 * network data only until a complete encrypted chunk (~6 MiB) is assembled,
 * decrypts it immediately, and releases the encrypted bytes.  Peak memory
 * is therefore one encrypted chunk plus the accumulated plaintext, making it
 * safe for files of any size.
 *
 * The backend deletes the file from R2 **immediately after serving it**
 * ("burn after reading"), so a file can only be downloaded once.
 *
 * @param apiPrefix  The base URL of the backend API (e.g. `"https://api.filez.zone/v1"`).
 * @param fileId     The UUID of the file to download.
 * @param rawKey     The raw AES-256 key bytes (extracted from the capability URL hash).
 * @param onProgress Optional callback invoked during download and decryption.
 *                    Receives a number between 0 (just started) and 1 (fully decrypted).
 */
export declare function downloadFile(
	apiPrefix: string,
	fileId: string,
	rawKey: Uint8Array,
	onProgress?: ProgressCallback
): Promise<DownloadResult>;
/**
 * Decrypts raw binary ciphertext (concatenated `IV || ciphertext || GCM tag` blocks)
 * into a plaintext Blob.
 *
 * This is a lower-level utility for when you already have the complete encrypted
 * data in memory.  For large files, prefer `downloadFile()` which streams and
 * decrypts incrementally without buffering the entire ciphertext.
 *
 * @param encryptedBytes  The raw binary ciphertext to decrypt.
 * @param chunkSize       The plaintext chunk size in bytes used during upload.
 * @param rawKey          The raw AES-256 key bytes.
 * @param contentType     The MIME type to assign to the resulting Blob.
 * @param onProgress      Optional callback invoked after each chunk is decrypted.
 *                         Receives a number between 0 and 1.
 * @returns The assembled plaintext Blob and the original content type.
 */
export declare function decryptBytes(
	encryptedBytes: Uint8Array,
	chunkSize: number,
	rawKey: Uint8Array,
	contentType: string,
	onProgress?: ProgressCallback
): Promise<{
	blob: Blob;
	contentType: string;
}>;
//# sourceMappingURL=download.d.ts.map
