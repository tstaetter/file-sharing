import { generateKey, encryptChunk } from './crypto';
import { chunkFile } from './chunk';
import { PUBLIC_API_PREFIX } from '$env/static/public';

interface CreateUploadResponse {
	upload_id: string;
	key: string;
}

interface SignedUrl {
	url: string;
}

interface PartETag {
	part_number: number;
	etag: string | null;
}

interface UploadResult {
	raw: Uint8Array;
	fileId: string;
}

export async function uploadFile(file: File): Promise<UploadResult> {
	const { key, raw } = await generateKey();
	const fileId = crypto.randomUUID();

	const init = await fetch(`${PUBLIC_API_PREFIX}/create-upload`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ file_id: fileId, content_type: file.type || null })
	});
	const { upload_id, key: storageKey } = (await init.json()) as CreateUploadResponse;

	let part = 1;
	const parts: PartETag[] = [];

	for await (const chunk of chunkFile(file)) {
		const { iv, data } = await encryptChunk(key, chunk);

		const payload = new Uint8Array(iv.length + data.length);
		payload.set(iv);
		payload.set(data, iv.length);

		const res = await fetch(`${PUBLIC_API_PREFIX}/sign-parts`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				key: storageKey,
				upload_id,
				part_numbers: [part]
			})
		});

		const urls = (await res.json()) as SignedUrl[];
		const url = urls[0].url;

		const uploadRes = await fetch(url, {
			method: 'PUT',
			body: payload
		});

		parts.push({
			part_number: part,
			etag: uploadRes.headers.get('ETag')
		});

		part++;
	}

	await fetch(`${PUBLIC_API_PREFIX}/complete-upload`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({
			key: storageKey,
			upload_id,
			parts
		})
	});

	return { raw, fileId };
}
