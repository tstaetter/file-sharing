/**
 * Generates a new AES-256-GCM key suitable for encrypting and decrypting
 * file chunks. Returns both the CryptoKey (for use with Web Crypto) and its
 * raw byte representation (for embedding in capability URLs).
 */
export declare function generateKey(): Promise<{
    key: CryptoKey;
    raw: Uint8Array;
}>;
/**
 * Imports a raw AES-256-GCM key (32 bytes) from a capability URL back into
 * a CryptoKey for decryption.
 */
export declare function importKey(rawKey: Uint8Array): Promise<CryptoKey>;
/**
 * Encrypts a single plaintext chunk with AES-256-GCM. A random 12-byte IV
 * is generated per call. The returned `data` includes the GCM authentication
 * tag appended to the ciphertext (Web Crypto standard behaviour).
 */
export declare function encryptChunk(key: CryptoKey, chunk: Uint8Array): Promise<{
    iv: Uint8Array;
    data: Uint8Array;
}>;
/**
 * Decrypts a single encrypted chunk using the given key and the 12-byte IV
 * that was prepended during encryption. The input must be `IV || ciphertext || GCM tag`.
 */
export declare function decryptChunk(key: CryptoKey, iv: Uint8Array, encrypted: Uint8Array): Promise<Uint8Array>;
//# sourceMappingURL=crypto.d.ts.map