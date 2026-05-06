/**
 * Encodes a Uint8Array as URL-safe base64 without padding,
 * matching the Rust `general_purpose::URL_SAFE_NO_PAD` encoding.
 */
export declare function urlSafeBase64(bytes: Uint8Array): string;
/**
 * Decodes a base64 string (standard or URL-safe, with or without padding)
 * into a Uint8Array.
 */
export declare function base64ToBytes(base64: string): Uint8Array<ArrayBuffer>;
/**
 * Normalizes a `data` payload from JSON into a Uint8Array.
 * Accepts either a base64-encoded string or an array of byte values.
 */
export declare function dataToBytes(value: string | number[]): Uint8Array<ArrayBuffer>;
/**
 * Maps a MIME type string to a safe file extension.
 * Falls back to the subtype (e.g. "plain" from "text/plain") or "bin".
 */
export declare function extensionFromMime(mime: string | null | undefined): string;
//# sourceMappingURL=utils.d.ts.map