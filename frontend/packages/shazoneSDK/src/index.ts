// ── Crypto ──────────────────────────────────────────────────────────
export { generateKey, importKey, encryptChunk, decryptChunk } from './crypto';

// ── File chunking ───────────────────────────────────────────────────
export { chunkFile, DEFAULT_CHUNK_SIZE } from './chunk';

// ── Upload ──────────────────────────────────────────────────────────
export { uploadFile } from './upload';
export type { CreateUploadResponse, SignedUrl, PartETag, UploadResult } from './upload';

// ── Download & decryption ───────────────────────────────────────────
export { downloadFile, decryptFile } from './download';
export type { StoredFile, DownloadResult } from './download';

// ── Capability URLs ─────────────────────────────────────────────────
export { createCapabilityUrl } from './cap_url';

// ── Utilities ───────────────────────────────────────────────────────
export { urlSafeBase64, base64ToBytes, dataToBytes, extensionFromMime } from './utils';
