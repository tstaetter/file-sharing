// Re-export the full SDK surface for convenience
export {
	generateKey,
	importKey,
	encryptChunk,
	decryptChunk,
	chunkFile,
	downloadFile,
	decryptFile,
	createCapabilityUrl,
	urlSafeBase64,
	base64ToBytes,
	dataToBytes,
	extensionFromMime
} from 'shazoneSDK';

export type {
	CreateUploadResponse,
	SignedUrl,
	PartETag,
	UploadResult,
	StoredFile,
	DownloadResult
} from 'shazoneSDK';

// Convenience wrapper that binds the backend URL from SvelteKit's environment
export { uploadFile } from './upload';
