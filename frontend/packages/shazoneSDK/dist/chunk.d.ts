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
export declare const DEFAULT_CHUNK_SIZE: number;
export declare function chunkFile(file: File | Blob | undefined, size?: number): AsyncGenerator<Uint8Array, void, undefined>;
//# sourceMappingURL=chunk.d.ts.map