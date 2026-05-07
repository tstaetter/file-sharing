import { dataToBytes, extensionFromMime } from './utils';
import { importKey, decryptChunk } from './crypto';

// Sizes are deliberately hardcoded to match the Rust upload backend and the
// per-chunk encryption scheme. Changing these will break interop.
const LEGACY_CHUNK_SIZE = 5_000_000;
const IV_LEN = 12;
const GCM_TAG_LEN = 16;

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
 * @param apiPrefix The base URL of the backend API (e.g. `"https://api.sha.zone/v1"`).
 * @param fileId    The UUID of the file to download.
 * @param rawKey    The raw AES-256 key bytes (extracted from the capability URL hash).
 * @returns A `DownloadResult` containing the decrypted Blob and a suggested file name.
 *
 * @example
 * ```ts
 * import { downloadFile } from 'shazoneSDK';
 *
 * // Key extracted from the URL hash fragment
 * const rawKey = base64ToBytes(location.hash.slice(1));
 * const id = page.params.id; // from routing
 * const { blob, fileName } = await downloadFile('https://api.sha.zone/v1', id, rawKey);
 *
 * const a = document.createElement('a');
 * a.href = URL.createObjectURL(blob);
 * a.download = fileName;
 * a.click();
 * ```
 */
export async function downloadFile(
	apiPrefix: string,
	fileId: string,
	rawKey: Uint8Array
): Promise<DownloadResult> {
	const res = await fetch(`${apiPrefix}/f/${fileId}`);

	if (!res.ok) {
		throw new Error(`failed to fetch file: ${res.status} ${res.statusText}`);
	}

	const body: StoredFile = await res.json();

	const { blob, contentType } = await decryptFile(body, rawKey);

	const ext = extensionFromMime(contentType);
	const fileName = `download.${ext}`;

	return { blob, fileName };
}

/**
 * Decrypts an already-fetched `StoredFile` payload into a Blob.
 *
 * This is a lower-level function useful when you already have the JSON
 * response in memory. It splits the `data` byte stream into per-chunk
 * `IV (12 bytes) || ciphertext || GCM tag (16 bytes)` blocks and
 * decrypts each one independently.
 *
 * @param stored The JSON payload returned by `GET /v1/f/{id}`.
 * @param rawKey The raw AES-256 key bytes.
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
export async function decryptFile(
	stored: StoredFile,
	rawKey: Uint8Array
): Promise<{ blob: Blob; contentType: string }> {
	const cryptoKey = await importKey(rawKey);
	const encryptedBytes = dataToBytes(stored.data);

	// Determine the chunk size used during upload.  Older uploads that
	// don't include `chunk_size` used a 5 MB plaintext chunk size.
	const chunkSize = stored.chunk_size ?? LEGACY_CHUNK_SIZE;
	const encryptedChunkSize = IV_LEN + chunkSize + GCM_TAG_LEN;

	// --- split into per-chunk blocks ---
	const chunks: Uint8Array[] = [];
	let offset = 0;
	while (offset < encryptedBytes.length) {
		const remaining = encryptedBytes.length - offset;
		const size = Math.min(encryptedChunkSize, remaining);
		chunks.push(encryptedBytes.slice(offset, offset + size));
		offset += size;
	}

	// --- decrypt each chunk ---
	const plaintextChunks: Uint8Array[] = [];
	for (const chunk of chunks) {
		const iv = chunk.slice(0, IV_LEN);
		const ciphertext = chunk.slice(IV_LEN);
		const plaintext = await decryptChunk(cryptoKey, iv, ciphertext);
		plaintextChunks.push(plaintext);
	}

	// --- reassemble ---
	const totalLen = plaintextChunks.reduce((sum, c) => sum + c.length, 0);
	const decrypted = new Uint8Array(totalLen);
	let writeOffset = 0;
	for (const chunk of plaintextChunks) {
		decrypted.set(chunk, writeOffset);
		writeOffset += chunk.length;
	}

	const contentType = stored.content_type || 'application/octet-stream';
	const blob = new Blob([decrypted], { type: contentType });

	return { blob, contentType };
}
