/**
 * Splits a `File` or `Blob` into sequential chunks of the specified size.
 *
 * This is an async generator — consume it with a `for await…of` loop to
 * read and process each chunk without loading the entire file into memory.
 *
 * @param file The file or blob to split. If `undefined` or `null`, yields nothing.
 * @param size Maximum chunk size in bytes (default 6 MiB).
 *
 * @example
 * ```ts
 * import { chunkFile, encryptChunk, generateKey } from 'shazoneSDK';
 *
 * const { key } = await generateKey();
 * for await (const chunk of chunkFile(file)) {
 *   const { iv, data } = await encryptChunk(key, chunk);
 *   // upload `iv || data` ...
 * }
 * ```
 */

/** Default chunk size: 6 MiB (6 * 1024 * 1024 = 6,291,456 bytes). */
export const DEFAULT_CHUNK_SIZE = 6 * 1024 * 1024; // 6 MiB

export async function* chunkFile(
	file: File | Blob | undefined,
	size = DEFAULT_CHUNK_SIZE
): AsyncGenerator<Uint8Array, void, undefined> {
	if (!file) return;

	let offset = 0;

	while (offset < file.size) {
		const slice = file.slice(offset, offset + size);
		yield new Uint8Array(await slice.arrayBuffer());
		offset += size;
	}
}
