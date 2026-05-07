/** Returned by `POST /v1/create-upload`. */
export interface CreateUploadResponse {
    upload_id: string;
    key: string;
}
/** A single presigned URL returned by `POST /v1/sign-parts`. */
export interface SignedUrl {
    url: string;
}
/** An ETag collected from a presigned PUT response. */
export interface PartETag {
    part_number: number;
    etag: string | null;
}
/** The result of a successful upload. */
export interface UploadResult {
    /** The raw AES-256 key bytes — pass this to `createCapabilityUrl`. */
    raw: Uint8Array;
    /** The UUID assigned to this file — pass this to `createCapabilityUrl`. */
    fileId: string;
}
/** Callback for tracking upload progress. Receives a value between 0 and 1. */
export type ProgressCallback = (progress: number) => void;
/**
 * Encrypts a file client-side and uploads it directly to R2 storage via
 * the file-sharing backend API.
 *
 * This function:
 * 1. Generates a fresh AES-256-GCM key.
 * 2. Calls `POST /v1/create-upload` to initiate a multipart upload.
 * 3. Splits the file into 6 MiB chunks, encrypts each with a random IV, and
 *    uploads each one directly to R2 via a presigned URL.
 * 4. Calls `POST /v1/complete-upload` to finalise the upload.
 *
 * **The backend never sees plaintext** — all encryption happens locally
 * in the browser using the Web Crypto API.
 *
 * @param apiPrefix  The base URL of the backend API, e.g. `"https://api.sha.zone/v1"`.
 * @param file       The `File` object to upload (from an `<input type="file">` or drag-and-drop).
 * @param onProgress Optional callback invoked after each chunk is uploaded.
 *                    Receives a number between 0 (just started) and 1 (all chunks uploaded).
 * @returns The raw AES key bytes and the file's UUID, which can be used with
 *          `createCapabilityUrl` to build a shareable download link.
 *
 * @example
 * ```ts
 * import { uploadFile, createCapabilityUrl } from 'shazoneSDK';
 *
 * const input = document.querySelector('input[type=file]');
 * const file = input.files[0];
 * const result = await uploadFile('https://api.sha.zone/v1', file, (p) => console.log(`${(p * 100).toFixed(0)}%`));
 * const url = createCapabilityUrl('https://sha.zone', result.fileId, result.raw);
 * console.log('Share this link:', url);
 * ```
 */
export declare function uploadFile(apiPrefix: string, file: File, onProgress?: ProgressCallback): Promise<UploadResult>;
//# sourceMappingURL=upload.d.ts.map