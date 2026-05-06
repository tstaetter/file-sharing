import { generateKey, encryptChunk } from './crypto';
import { chunkFile } from './chunk';
/**
 * Encrypts a file client-side and uploads it directly to R2 storage via
 * the file-sharing backend API.
 *
 * This function:
 * 1. Generates a fresh AES-256-GCM key.
 * 2. Calls `POST /v1/create-upload` to initiate a multipart upload.
 * 3. Splits the file into 5 MB chunks, encrypts each with a random IV, and
 *    uploads each one directly to R2 via a presigned URL.
 * 4. Calls `POST /v1/complete-upload` to finalise the upload.
 *
 * **The backend never sees plaintext** — all encryption happens locally
 * in the browser using the Web Crypto API.
 *
 * @param apiPrefix The base URL of the backend API, e.g. `"https://api.sha.zone/v1"`.
 * @param file      The `File` object to upload (from an `<input type="file">` or drag-and-drop).
 * @returns The raw AES key bytes and the file's UUID, which can be used with
 *          `createCapabilityUrl` to build a shareable download link.
 *
 * @example
 * ```ts
 * import { uploadFile, createCapabilityUrl } from 'shazoneSDK';
 *
 * const input = document.querySelector('input[type=file]');
 * const file = input.files[0];
 * const result = await uploadFile('https://api.sha.zone/v1', file);
 * const url = createCapabilityUrl('https://sha.zone', result.fileId, result.raw);
 * console.log('Share this link:', url);
 * ```
 */
export async function uploadFile(apiPrefix, file) {
    const { key, raw } = await generateKey();
    const fileId = crypto.randomUUID();
    const init = await fetch(`${apiPrefix}/create-upload`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            file_id: fileId,
            content_type: file.type || null,
        }),
    });
    if (!init.ok) {
        throw new Error(`create-upload failed: ${init.status} ${init.statusText}`);
    }
    const { upload_id, key: storageKey } = (await init.json());
    let part = 1;
    const parts = [];
    for await (const chunk of chunkFile(file)) {
        const { iv, data } = await encryptChunk(key, chunk);
        const payload = new Uint8Array(iv.length + data.length);
        payload.set(iv);
        payload.set(data, iv.length);
        const signRes = await fetch(`${apiPrefix}/sign-parts`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                key: storageKey,
                upload_id,
                part_numbers: [part],
            }),
        });
        if (!signRes.ok) {
            // Attempt to clean up on failure.
            await fetch(`${apiPrefix}/abort-upload`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ key: storageKey, upload_id }),
            });
            throw new Error(`sign-parts failed for part ${part}: ${signRes.status} ${signRes.statusText}`);
        }
        const urls = (await signRes.json());
        const url = urls[0]?.url;
        if (!url) {
            await fetch(`${apiPrefix}/abort-upload`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ key: storageKey, upload_id }),
            });
            throw new Error(`sign-parts returned no URL for part ${part}`);
        }
        const uploadRes = await fetch(url, {
            method: 'PUT',
            body: payload,
        });
        if (!uploadRes.ok) {
            await fetch(`${apiPrefix}/abort-upload`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ key: storageKey, upload_id }),
            });
            throw new Error(`part ${part} upload failed: ${uploadRes.status} ${uploadRes.statusText}`);
        }
        parts.push({
            part_number: part,
            etag: uploadRes.headers.get('ETag'),
        });
        part++;
    }
    const completeRes = await fetch(`${apiPrefix}/complete-upload`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            key: storageKey,
            upload_id,
            parts,
        }),
    });
    if (!completeRes.ok) {
        await fetch(`${apiPrefix}/abort-upload`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ key: storageKey, upload_id }),
        });
        throw new Error(`complete-upload failed: ${completeRes.status} ${completeRes.statusText}`);
    }
    return { raw, fileId };
}
//# sourceMappingURL=upload.js.map