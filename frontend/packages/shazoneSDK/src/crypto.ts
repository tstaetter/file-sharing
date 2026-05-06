const ALGORITHM = { name: 'AES-GCM', length: 256 } as const;
const IV_LEN = 12;

/**
 * Generates a new AES-256-GCM key suitable for encrypting and decrypting
 * file chunks. Returns both the CryptoKey (for use with Web Crypto) and its
 * raw byte representation (for embedding in capability URLs).
 */
export async function generateKey(): Promise<{
	key: CryptoKey;
	raw: Uint8Array;
}> {
	const key = await crypto.subtle.generateKey(ALGORITHM, true, ['encrypt', 'decrypt']);

	const raw = new Uint8Array(await crypto.subtle.exportKey('raw', key));

	return { key, raw };
}

/**
 * Imports a raw AES-256-GCM key (32 bytes) from a capability URL back into
 * a CryptoKey for decryption.
 */
export function importKey(rawKey: Uint8Array): Promise<CryptoKey> {
	return crypto.subtle.importKey('raw', rawKey as BufferSource, ALGORITHM, false, ['decrypt']);
}

/**
 * Encrypts a single plaintext chunk with AES-256-GCM. A random 12-byte IV
 * is generated per call. The returned `data` includes the GCM authentication
 * tag appended to the ciphertext (Web Crypto standard behaviour).
 */
export async function encryptChunk(
	key: CryptoKey,
	chunk: Uint8Array
): Promise<{ iv: Uint8Array; data: Uint8Array }> {
	const iv = crypto.getRandomValues(new Uint8Array(IV_LEN));

	const encrypted = await crypto.subtle.encrypt(
		{ name: ALGORITHM.name, iv },
		key,
		chunk as BufferSource
	);

	return { iv, data: new Uint8Array(encrypted) };
}

/**
 * Decrypts a single encrypted chunk using the given key and the 12-byte IV
 * that was prepended during encryption. The input must be `IV || ciphertext || GCM tag`.
 */
export async function decryptChunk(
	key: CryptoKey,
	iv: Uint8Array,
	encrypted: Uint8Array
): Promise<Uint8Array> {
	const plaintext = await crypto.subtle.decrypt(
		{ name: ALGORITHM.name, iv: iv as BufferSource },
		key,
		encrypted as BufferSource
	);

	return new Uint8Array(plaintext);
}
