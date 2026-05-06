import { urlSafeBase64 } from './utils';

/**
 * Builds a capability URL with the encryption key embedded in the hash fragment.
 *
 * The key never touches the server — it lives only in the URL hash, which browsers
 * never transmit over HTTP. The resulting URL has the form:
 *
 * ```
 * {base}/f/{fileId}#{url-safe-base64(keyBytes)}
 * ```
 *
 * @param base    The base URL of the file-sharing application (e.g. `"https://sha.zone"`).
 * @param fileId  The UUID of the uploaded file (returned by `uploadFile`).
 * @param keyBytes The raw AES-256 key bytes (returned by `generateKey` or `uploadFile`).
 * @returns A capability URL that can be shared with a recipient.
 *
 * @example
 * ```ts
 * import { uploadFile, createCapabilityUrl } from 'shazoneSDK';
 *
 * const result = await uploadFile(file, 'https://api.example.com/v1');
 * const url = createCapabilityUrl('https://sha.zone', result.fileId, result.raw);
 * // url = "https://sha.zone/f/550e8400-e29b-41d4-a716-446655440000#kJGds83..."
 * ```
 */
export function createCapabilityUrl(
	base: string,
	fileId: string,
	keyBytes: Uint8Array,
): string {
	const keyB64 = urlSafeBase64(keyBytes);
	return `${base}/f/${fileId}#${keyB64}`;
}
