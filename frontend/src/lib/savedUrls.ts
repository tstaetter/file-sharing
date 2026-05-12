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
