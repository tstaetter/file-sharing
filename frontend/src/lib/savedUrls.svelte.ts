// Reactive store for saved capability URLs backed by localStorage.
// Uses Svelte 5 runes (`$state`) so components react to changes automatically.
// No authentication required — URLs are stored per-browser.

// ── Types ────────────────────────────────────────────────────────────────

/** A single locally-saved URL record. */
export interface LocalUrlItem {
	id: string;
	url: string;
	title: string | null;
	created_at: string;
}

// ── localStorage key ─────────────────────────────────────────────────────

const STORAGE_KEY = 'saved_urls';

// ── Helpers ──────────────────────────────────────────────────────────────

function loadFromStorage(): LocalUrlItem[] {
	if (typeof localStorage === 'undefined') return [];
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		return raw ? (JSON.parse(raw) as LocalUrlItem[]) : [];
	} catch {
		return [];
	}
}

function persistToStorage(items: LocalUrlItem[]) {
	if (typeof localStorage === 'undefined') return;
	localStorage.setItem(STORAGE_KEY, JSON.stringify(items));
}

// ── Reactive store ───────────────────────────────────────────────────────

function createLocalUrlsStore() {
	let urls = $state<LocalUrlItem[]>(loadFromStorage());

	/** Save a new URL to the collection. */
	function add(url: string, title: string | null): LocalUrlItem {
		const item: LocalUrlItem = {
			id: crypto.randomUUID(),
			url,
			title,
			created_at: new Date().toISOString()
		};
		urls = [item, ...urls];
		persistToStorage(urls);
		return item;
	}

	/** Delete a URL by its ID. */
	function remove(id: string) {
		urls = urls.filter((u) => u.id !== id);
		persistToStorage(urls);
	}

	/** Reload from localStorage (useful if another tab modified it). */
	function reload() {
		urls = loadFromStorage();
	}

	return {
		get urls() {
			return urls;
		},
		add,
		remove,
		reload
	};
}

// ── Singleton ────────────────────────────────────────────────────────────

export const localUrls = createLocalUrlsStore();

// ── File-existence check (still uses the backend API) ────────────────────

import { PUBLIC_API_PREFIX } from '$env/static/public';

/**
 * Checks whether a file still exists in storage (has not been consumed
 * by a burn-after-read download). This does NOT require authentication.
 */
export async function checkFile(key: string): Promise<boolean> {
	const res = await fetch(`${PUBLIC_API_PREFIX}/check-file`, {
		method: 'PUT',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ key })
	});
	return res.ok;
}

/**
 * Extracts the file ID (UUID) from a capability URL.
 *
 * Capability URLs have the form `https://example.com/f/{uuid}#key`.
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
