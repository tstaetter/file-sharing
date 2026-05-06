/**
 * Encodes a Uint8Array as URL-safe base64 (no padding),
 * matching the WASM `general_purpose::URL_SAFE_NO_PAD` encoding.
 */
function urlSafeBase64(bytes: Uint8Array): string {
	let binary = '';
	for (let i = 0; i < bytes.length; i++) {
		binary += String.fromCharCode(bytes[i]);
	}
	return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

/**
 * Builds a capability URL with the encryption key embedded in the hash fragment.
 * The key never touches the server — it lives only in the URL hash.
 */
export function createCapabilityUrl(base: string, fileId: string, keyBytes: Uint8Array): string {
	const keyB64 = urlSafeBase64(keyBytes);
	return `${base}/f/${fileId}#${keyB64}`;
}
