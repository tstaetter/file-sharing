export { generateKey, importKey, encryptChunk, decryptChunk } from './crypto';
export { chunkFile, DEFAULT_CHUNK_SIZE } from './chunk';
export { uploadFile } from './upload';
export type { CreateUploadResponse, SignedUrl, PartETag, UploadResult, ProgressCallback } from './upload';
export { downloadFile, decryptBytes } from './download';
export type { DownloadResult } from './download';
export { createCapabilityUrl } from './cap_url';
export { urlSafeBase64, base64ToBytes, dataToBytes, extensionFromMime } from './utils';
//# sourceMappingURL=index.d.ts.map