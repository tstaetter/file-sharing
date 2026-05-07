export { generateKey, importKey, encryptChunk, decryptChunk } from './crypto';
export { chunkFile, DEFAULT_CHUNK_SIZE } from './chunk';
export { uploadFile } from './upload';
export type { CreateUploadResponse, SignedUrl, PartETag, UploadResult } from './upload';
export { downloadFile, decryptFile } from './download';
export type { StoredFile, DownloadResult } from './download';
export { createCapabilityUrl } from './cap_url';
export { urlSafeBase64, base64ToBytes, dataToBytes, extensionFromMime } from './utils';
//# sourceMappingURL=index.d.ts.map