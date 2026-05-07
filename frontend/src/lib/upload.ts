import {
	uploadFile as sdkUploadFile,
	type UploadResult,
	type ProgressCallback
} from '../../packages/shazoneSDK';
import { PUBLIC_API_PREFIX } from '$env/static/public';

export type { UploadResult, ProgressCallback };

/**
 * Encrypts a file client-side and uploads it directly to R2 storage.
 *
 * This is a thin wrapper around the `shazoneSDK` package that binds the
 * backend API URL from the SvelteKit environment (`PUBLIC_API_PREFIX`).
 *
 * @param file       The `File` object to upload (from an `<input type="file">` or
 *                   drag-and-drop).
 * @param onProgress Optional callback invoked after each chunk is uploaded.
 *                    Receives a number between 0 and 1.
 * @returns The raw AES key bytes and the file's UUID, which can be passed to
 *          `createCapabilityUrl` to build a shareable download link.
 */
export async function uploadFile(file: File, onProgress?: ProgressCallback): Promise<UploadResult> {
	return sdkUploadFile(PUBLIC_API_PREFIX, file, onProgress);
}
