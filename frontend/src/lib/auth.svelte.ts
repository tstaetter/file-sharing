// Auth store using Svelte 5 runes ($state) for reactive authentication state.
// Stores the JWT token in localStorage for persistence across page reloads.
// Components can import and use these runes directly — no subscription needed.

import { PUBLIC_API_PREFIX } from '$env/static/public';

// ── Types ───────────────────────────────────────────────────────────────

interface UserInfo {
	email: string;
	name: string;
}

interface AuthResponse {
	token: string;
	user: UserInfo;
}

// ── Reactive state ──────────────────────────────────────────────────────

let token = $state<string | null>(loadToken());
let user = $state<UserInfo | null>(loadUser());
let loading = $state(false);
let error = $state<string | null>(null);

// ── Persistence helpers ─────────────────────────────────────────────────

function loadToken(): string | null {
	if (typeof localStorage === 'undefined') return null;
	return localStorage.getItem('auth_token');
}

function loadUser(): UserInfo | null {
	if (typeof localStorage === 'undefined') return null;
	try {
		const raw = localStorage.getItem('auth_user');
		return raw ? JSON.parse(raw) : null;
	} catch {
		return null;
	}
}

function persistToken(t: string | null) {
	if (typeof localStorage === 'undefined') return;
	if (t) {
		localStorage.setItem('auth_token', t);
	} else {
		localStorage.removeItem('auth_token');
	}
}

function persistUser(u: UserInfo | null) {
	if (typeof localStorage === 'undefined') return;
	if (u) {
		localStorage.setItem('auth_user', JSON.stringify(u));
	} else {
		localStorage.removeItem('auth_user');
	}
}

// ── Token validation ────────────────────────────────────────────────────

function isTokenExpired(t: string): boolean {
	try {
		const payload = JSON.parse(atob(t.split('.')[1]!));
		const exp = payload.exp * 1000; // JWT exp is in seconds
		return Date.now() >= exp;
	} catch {
		return true; // malformed token = expired
	}
}

// ── Derived state ────────────────────────────────────────────────────────

const isAuthenticated = $derived(token !== null && !isTokenExpired(token));

// ── API helpers ──────────────────────────────────────────────────────────

async function apiPost(path: string, body: unknown): Promise<Response> {
	return fetch(`${PUBLIC_API_PREFIX}/auth${path}`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(body)
	});
}

// ── Actions ─────────────────────────────────────────────────────────────

export function createAuth() {
	/** Register a new account and log in immediately. */
	async function signUp(email: string, password: string, name: string): Promise<boolean> {
		loading = true;
		error = null;
		try {
			const res = await apiPost('/register', { email, password, name });
			const body = (await res.json()) as AuthResponse | { error: string };

			if (!res.ok) {
				const msg =
					typeof body === 'object' && body !== null && 'error' in body
						? (body as { error: string }).error
						: `Registration failed (${res.status})`;
				throw new Error(msg);
			}

			const data = body as AuthResponse;
			token = data.token;
			user = data.user;
			persistToken(data.token);
			persistUser(data.user);
			return true;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Registration failed';
			return false;
		} finally {
			loading = false;
		}
	}

	/** Log in with email and password. */
	async function signIn(email: string, password: string): Promise<boolean> {
		loading = true;
		error = null;
		try {
			const res = await apiPost('/login', { email, password });
			const body = (await res.json()) as AuthResponse | { error: string };

			if (!res.ok) {
				const msg =
					typeof body === 'object' && body !== null && 'error' in body
						? (body as { error: string }).error
						: `Login failed (${res.status})`;
				throw new Error(msg);
			}

			const data = body as AuthResponse;
			token = data.token;
			user = data.user;
			persistToken(data.token);
			persistUser(data.user);
			return true;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Login failed';
			return false;
		} finally {
			loading = false;
		}
	}

	/** Log out and clear session. */
	function signOut() {
		token = null;
		user = null;
		error = null;
		persistToken(null);
		persistUser(null);
	}

	/** Delete the account (requires valid token). Returns true on success. */
	async function deleteAccount(): Promise<boolean> {
		if (!token) return false;
		loading = true;
		error = null;
		try {
			const res = await apiPost('/delete', { token });
			if (!res.ok) {
				const body = await res.text();
				throw new Error(body || `Delete failed (${res.status})`);
			}
			signOut();
			return true;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Account deletion failed';
			return false;
		} finally {
			loading = false;
		}
	}

	/** Clear any displayed error. */
	function clearError() {
		error = null;
	}

	return {
		// Reactive state (read by components)
		get token() {
			return token;
		},
		get user() {
			return user;
		},
		get loading() {
			return loading;
		},
		get error() {
			return error;
		},
		get isAuthenticated() {
			return isAuthenticated;
		},

		// Actions
		signUp,
		signIn,
		signOut,
		deleteAccount,
		clearError
	};
}

// ── Singleton instance ──────────────────────────────────────────────────

export const auth = createAuth();
