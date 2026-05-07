// Re-export the full SDK surface for convenience
export {
	generateKey,
	importKey,
	encryptChunk,
	decryptChunk,
	chunkFile,
	downloadFile,
	decryptBytes,
	createCapabilityUrl,
	urlSafeBase64,
	base64ToBytes,
	dataToBytes,
	extensionFromMime
} from '../../packages/shazoneSDK';

export type {
	CreateUploadResponse,
	SignedUrl,
	PartETag,
	UploadResult,
	DownloadResult,
	ProgressCallback
} from '../../packages/shazoneSDK';

// Convenience wrapper that binds the backend URL from SvelteKit's environment
export { uploadFile } from './upload';
