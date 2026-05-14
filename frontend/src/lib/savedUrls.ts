// API module for saved capability URLs.
// Uses the Bearer token from the auth store to authenticate requests.

import { PUBLIC_API_PREFIX } from '$env/static/public';

// ── Types ────────────────────────────────────────────────────────────────

/** A single saved URL record returned by the backend. */
export interface SavedUrlItem {
	id: string;
	url: string;
	title: string | null;
	created_at: string;
}

/** Response from POST /v1/urls (save a URL). */
export interface SaveUrlResponse extends SavedUrlItem {}

/** Response from GET /v1/urls (list saved URLs). */
export interface ListUrlsResponse {
	urls: SavedUrlItem[];
	page: number;
	per_page: number;
	total: number;
}

// ── API helpers ──────────────────────────────────────────────────────────

/**
 * POST /v1/urls
 *
 * Saves a capability URL to the authenticated user's collection.
 * The backend identifies the user from the Bearer token.
 *
 * @param url    The capability URL to save (e.g. `"https://filez.zone/f/abc#key"`).
 * @param title  An optional human-readable title for the URL.
 * @param token  A valid JWT token for authentication.
 * @returns The saved URL record with its server-generated ID and timestamp.
 */
export async function saveUrl(
	url: string,
	title: string | null,
	token: string
): Promise<SaveUrlResponse> {
	const res = await fetch(`${PUBLIC_API_PREFIX}/urls`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json',
			Authorization: `Bearer ${token}`
		},
		body: JSON.stringify({ url, title: title ?? null })
	});

	if (!res.ok) {
		const body = await res.text().catch(() => 'Unknown error');
		throw new Error(body || `Failed to save URL (${res.status})`);
	}

	return res.json() as Promise<SaveUrlResponse>;
}

/**
 * GET /v1/urls?page=&per_page=
 *
 * Lists the authenticated user's saved capability URLs with pagination.
 * Results are ordered newest-first.
 *
 * @param token    A valid JWT token for authentication.
 * @param page     Page number (1-based, default 1).
 * @param perPage  Items per page (1–100, default 10).
 * @returns A paginated list of saved URLs.
 */
export async function listUrls(
	token: string,
	page: number = 1,
	perPage: number = 10
): Promise<ListUrlsResponse> {
	const params = new URLSearchParams({
		page: String(page),
		per_page: String(perPage)
	});

	const res = await fetch(`${PUBLIC_API_PREFIX}/urls?${params}`, {
		method: 'GET',
		headers: {
			Authorization: `Bearer ${token}`
		}
	});

	if (!res.ok) {
		const body = await res.text().catch(() => 'Unknown error');
		throw new Error(body || `Failed to list URLs (${res.status})`);
	}

	return res.json() as Promise<ListUrlsResponse>;
}

/**
 * PUT /v1/check-file
 *
 * Checks whether a file still exists in storage (has not been consumed
 * by a burn-after-read download). This does NOT require authentication.
 *
 * @param key  The file ID (UUID) extracted from the capability URL path.
 * @returns `true` if the file still exists, `false` if it has been
 *          consumed or was never uploaded.
 */
export async function checkFile(key: string): Promise<boolean> {
	const res = await fetch(`${PUBLIC_API_PREFIX}/check-file`, {
		method: 'PUT',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({ key })
	});

	return res.ok;
}

/**
 * Extracts the file ID (UUID) from a capability URL.
 *
 * Capability URLs have the form `https://example.com/f/{uuid}#key`.
 * The hash fragment is never sent to the server, so `URL.pathname`
 * naturally gives us `/f/{uuid}`.
 *
 * @param url  A capability URL (e.g. `"https://filez.zone/f/abc-123-def#key"`).
 * @returns The file UUID, or `null` if the URL does not match the expected format.
 */
export function extractFileId(url: string): string | null {
	try {
		const parsed = new URL(url);
		const match = parsed.pathname.match(/^\/f\/([0-9a-f-]+)$/i);
		return match ? match[1] : null;
	} catch {
		return null;
	}
}

/**
 * DELETE /v1/urls/{id}
 *
 * Deletes a saved URL by its ID. Only the owner can delete it.
 *
 * @param id     The ID of the saved URL record to delete.
 * @param token  A valid JWT token for authentication.
 */
export async function deleteUrl(id: string, token: string): Promise<void> {
	const res = await fetch(`${PUBLIC_API_PREFIX}/urls/${id}`, {
		method: 'DELETE',
		headers: {
			Authorization: `Bearer ${token}`
		}
	});

	if (!res.ok) {
		const body = await res.text().catch(() => 'Unknown error');
		throw new Error(body || `Failed to delete URL (${res.status})`);
	}
}
