/**
 * Encodes a Uint8Array as URL-safe base64 without padding,
 * matching the Rust `general_purpose::URL_SAFE_NO_PAD` encoding.
 */
export function urlSafeBase64(bytes: Uint8Array): string {
	let binary = '';
	for (let i = 0; i < bytes.length; i++) {
		binary += String.fromCharCode(bytes[i]!);
	}
	return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

/**
 * Decodes a base64 string (standard or URL-safe, with or without padding)
 * into a Uint8Array.
 */
export function base64ToBytes(base64: string): Uint8Array<ArrayBuffer> {
	let standardized = base64.replace(/-/g, '+').replace(/_/g, '/');
	while (standardized.length % 4 !== 0) standardized += '=';
	const binary = atob(standardized);
	const bytes = new Uint8Array(binary.length);
	for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
	return bytes;
}

/**
 * Normalizes a `data` payload from JSON into a Uint8Array.
 * Accepts either a base64-encoded string or an array of byte values.
 */
export function dataToBytes(value: string | number[]): Uint8Array<ArrayBuffer> {
	if (typeof value === 'string') return base64ToBytes(value);
	return new Uint8Array(value);
}

/**
 * Maps a MIME type string to a safe file extension.
 * Falls back to the subtype (e.g. "plain" from "text/plain") or "bin".
 */
export function extensionFromMime(mime: string | null | undefined): string {
	if (!mime) return 'bin';

	const known: Record<string, string> = {
		'image/png': 'png',
		'image/jpeg': 'jpg',
		'image/gif': 'gif',
		'image/webp': 'webp',
		'image/svg+xml': 'svg',
		'image/bmp': 'bmp',
		'image/tiff': 'tiff',
		'application/pdf': 'pdf',
		'application/zip': 'zip',
		'application/gzip': 'gz',
		'application/x-tar': 'tar',
		'application/x-7z-compressed': '7z',
		'application/x-rar-compressed': 'rar',
		'application/json': 'json',
		'application/xml': 'xml',
		'text/plain': 'txt',
		'text/html': 'html',
		'text/css': 'css',
		'text/javascript': 'js',
		'text/csv': 'csv',
		'audio/mpeg': 'mp3',
		'audio/wav': 'wav',
		'audio/ogg': 'ogg',
		'audio/aac': 'aac',
		'audio/flac': 'flac',
		'video/mp4': 'mp4',
		'video/webm': 'webm',
		'video/ogg': 'ogv',
		'video/quicktime': 'mov',
		'video/x-msvideo': 'avi',
		'application/msword': 'doc',
		'application/vnd.openxmlformats-officedocument.wordprocessingml.document':
			'docx',
		'application/vnd.ms-excel': 'xls',
		'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet':
			'xlsx',
		'application/vnd.ms-powerpoint': 'ppt',
		'application/vnd.openxmlformats-officedocument.presentationml.presentation':
			'pptx',
	};

	return known[mime] ?? mime.split('/').pop() ?? 'bin';
}
