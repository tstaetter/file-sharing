export async function generateKey(): Promise<{ key: CryptoKey; raw: Uint8Array }> {
	const key = await crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, true, [
		'encrypt',
		'decrypt'
	]);

	const raw = new Uint8Array(await crypto.subtle.exportKey('raw', key));

	return { key, raw };
}

export async function encryptChunk(
	key: CryptoKey,
	chunk: Uint8Array
): Promise<{ iv: Uint8Array; data: Uint8Array }> {
	const iv = crypto.getRandomValues(new Uint8Array(12));

	const encrypted = await crypto.subtle.encrypt(
		{ name: 'AES-GCM', iv },
		key,
		chunk as BufferSource
	);

	return { iv, data: new Uint8Array(encrypted) };
}
