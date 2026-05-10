import { extensionFromMime } from './utils';
import { importKey, decryptChunk } from './crypto';
import type { ProgressCallback } from './upload';

// Sizes are deliberately hardcoded to match the Rust upload backend and the
// per-chunk encryption scheme. Changing these will break interop.
const LEGACY_CHUNK_SIZE = 5_000_000;
const IV_LEN = 12;
const GCM_TAG_LEN = 16;

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
 * Consumes exactly `n` bytes from the front of the `pending` chunk list.
 * Mutates `pending` in place, shifting fully-consumed chunks and subarraying
 * partially-consumed ones.  The caller must keep `pendingSize` in sync.
 */
function consumeBytes(pending: Uint8Array[], n: number): Uint8Array {
	const result = new Uint8Array(n);
	let written = 0;
	while (written < n && pending.length > 0) {
		const chunk = pending[0]!;
		const remaining = n - written;
		if (chunk.length <= remaining) {
			// Consume the entire chunk
			result.set(chunk, written);
			written += chunk.length;
			pending.shift();
		} else {
			// Consume part of the chunk
			result.set(chunk.subarray(0, remaining), written);
			written += remaining;
			pending[0] = chunk.subarray(remaining);
		}
	}
	return result;
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
export async function downloadFile(
	apiPrefix: string,
	fileId: string,
	rawKey: Uint8Array,
	onProgress?: ProgressCallback
): Promise<DownloadResult> {
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
	const encryptedChunkSize = IV_LEN + chunkSize + GCM_TAG_LEN;
	const contentLength = parseInt(res.headers.get('Content-Length') || '0', 10);

	// Import the AES key once — reused for every chunk
	const cryptoKey = await importKey(rawKey);

	// Stream-download and decrypt incrementally.
	// We buffer incoming bytes in `pending`, and whenever we have enough for
	// a complete encrypted chunk we extract it, decrypt it, and discard the
	// encrypted bytes.  Peak memory: ~1 encrypted chunk + decrypted chunks.
	const plaintextChunks: Uint8Array[] = [];
	const pending: Uint8Array[] = [];
	let pendingSize = 0;
	let totalDownloaded = 0;
	let chunksDecrypted = 0;

	if (res.body) {
		const reader = res.body.getReader();

		try {
			while (true) {
				const { done, value } = await reader.read();
				if (done) break;

				pending.push(value);
				pendingSize += value.length;
				totalDownloaded += value.length;

				// Process as many complete encrypted chunks as possible.
				while (pendingSize >= encryptedChunkSize) {
					const encryptedChunk = consumeBytes(pending, encryptedChunkSize);
					pendingSize -= encryptedChunkSize;

					const iv = encryptedChunk.subarray(0, IV_LEN);
					const ciphertext = encryptedChunk.subarray(IV_LEN);
					const plaintext = await decryptChunk(cryptoKey, iv, ciphertext);
					plaintextChunks.push(plaintext);
					chunksDecrypted++;
				}

				// Report progress (5%–90%).
				// With Content-Length: precise byte-level progress.
				// Without: decelerating curve based on chunks decrypted — always
				// moves visibly but never falsely reaches 100% before we're done.
				if (contentLength > 0) {
					onProgress?.(0.05 + 0.85 * (totalDownloaded / contentLength));
				} else if (chunksDecrypted > 0) {
					onProgress?.(0.05 + 0.85 * (1 - Math.pow(0.97, chunksDecrypted)));
				}
			}
		} finally {
			reader.releaseLock();
		}
	} else {
		// Fallback for environments without ReadableStream (rare in modern browsers).
		// This loads the entire response into memory — not suitable for very large files.
		const buffer = new Uint8Array(await res.arrayBuffer());
		const totalChunks = Math.ceil(buffer.length / encryptedChunkSize);
		let offset = 0;
		let fallbackChunkIndex = 0;
		while (offset < buffer.length) {
			const size = Math.min(encryptedChunkSize, buffer.length - offset);
			const encryptedChunk = buffer.subarray(offset, offset + size);
			const iv = encryptedChunk.subarray(0, IV_LEN);
			const ciphertext = encryptedChunk.subarray(IV_LEN);
			const plaintext = await decryptChunk(cryptoKey, iv, ciphertext);
			plaintextChunks.push(plaintext);
			fallbackChunkIndex++;
			offset += size;

			// Report progress (5%–90%) — precise if we know the total chunks,
			// otherwise use the decelerating curve.
			if (totalChunks > 0) {
				onProgress?.(0.05 + 0.85 * (fallbackChunkIndex / totalChunks));
			} else {
				onProgress?.(0.05 + 0.85 * (1 - Math.pow(0.97, fallbackChunkIndex)));
			}
		}
	}

	// Process the final (possibly short) encrypted chunk remaining in the buffer.
	// A valid final chunk must have at least IV (12) + GCM tag (16) + 1 byte of ciphertext.
	if (pendingSize > 0) {
		if (pendingSize < IV_LEN + GCM_TAG_LEN + 1) {
			throw new Error(
				`Corrupt stream: remaining ${pendingSize} bytes are too short to form a valid encrypted chunk ` +
					`(minimum is ${IV_LEN + GCM_TAG_LEN + 1} bytes).`
			);
		}
		const remaining = consumeBytes(pending, pendingSize);
		pendingSize = 0;
		const iv = remaining.subarray(0, IV_LEN);
		const ciphertext = remaining.subarray(IV_LEN);
		const plaintext = await decryptChunk(cryptoKey, iv, ciphertext);
		plaintextChunks.push(plaintext);
	}

	onProgress?.(0.95);

	// Assemble the plaintext chunks into a Blob.
	// Blob(chunks) references each chunk's underlying ArrayBuffer without
	// creating a second contiguous copy — the Blob just holds pointers.
	const blob = new Blob(plaintextChunks as BlobPart[], { type: contentType });

	onProgress?.(1);

	const ext = extensionFromMime(contentType);
	const fileName = `download.${ext}`;

	return { blob, fileName };
}

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
export async function decryptBytes(
	encryptedBytes: Uint8Array,
	chunkSize: number,
	rawKey: Uint8Array,
	contentType: string,
	onProgress?: ProgressCallback
): Promise<{ blob: Blob; contentType: string }> {
	const cryptoKey = await importKey(rawKey);
	const encryptedChunkSize = IV_LEN + chunkSize + GCM_TAG_LEN;

	// Split into encrypted chunks
	const chunks: Uint8Array[] = [];
	let offset = 0;
	while (offset < encryptedBytes.length) {
		const remaining = encryptedBytes.length - offset;
		const size = Math.min(encryptedChunkSize, remaining);
		chunks.push(encryptedBytes.subarray(offset, offset + size));
		offset += size;
	}

	// Decrypt each chunk
	const plaintextChunks: Uint8Array[] = [];
	for (let i = 0; i < chunks.length; i++) {
		const chunk = chunks[i]!;
		const iv = chunk.subarray(0, IV_LEN);
		const ciphertext = chunk.subarray(IV_LEN);
		const plaintext = await decryptChunk(cryptoKey, iv, ciphertext);
		plaintextChunks.push(plaintext);
		onProgress?.((i + 1) / chunks.length);
	}

	// Assemble — Blob(chunks) avoids creating a second contiguous copy
	const blob = new Blob(plaintextChunks as BlobPart[], { type: contentType });
	return { blob, contentType };
}
