<script lang="ts">
	import '../app.css';
	import { auth } from '$lib/auth.svelte';
	import { goto } from '$app/navigation';

	let { children } = $props();

	const currentYear = new Date().getFullYear();
	const siteUrl = 'https://filez.zone';

	let showMenu = $state(false);

	function toggleMenu() {
		showMenu = !showMenu;
	}

	function closeMenu() {
		showMenu = false;
	}

	async function handleLogout() {
		auth.signOut();
		showMenu = false;
		goto('/');
	}

	async function handleDeleteAccount() {
		const confirmed = confirm(
			'Are you sure you want to delete your account? This action cannot be undone.'
		);
		if (!confirmed) return;
		const ok = await auth.deleteAccount();
		if (ok) {
			showMenu = false;
			goto('/');
		}
	}
</script>

<svelte:head>
	<link rel="icon" type="image/webp" href="/logo.webp" />
	<link rel="apple-touch-icon" href="/logo.webp" />
	<meta
		name="description"
		content="End-to-end encrypted file sharing. Files are encrypted in your browser before upload and deleted after download."
	/>
	<meta name="robots" content="index, follow" />
	<meta property="og:site_name" content="filez.zone" />
	<meta property="og:type" content="website" />
	<meta property="og:title" content="filez.zone — Secure File Sharing" />
	<meta
		property="og:description"
		content="End-to-end encrypted file sharing. Files are encrypted in your browser and deleted after download. No accounts needed."
	/>
	<meta property="og:url" content={siteUrl} />
	<meta property="og:image" content="{siteUrl}/logo.webp" />
	<meta property="og:image:type" content="image/webp" />
	<meta name="twitter:card" content="summary" />
	<meta name="twitter:title" content="filez.zone — Secure File Sharing" />
	<meta
		name="twitter:description"
		content="End-to-end encrypted file sharing. Files are encrypted in your browser and deleted after download. No accounts needed."
	/>
	<meta name="twitter:image" content="{siteUrl}/logo.webp" />
	<link rel="canonical" href={siteUrl} />
</svelte:head>

<div
	class="flex min-h-screen flex-col bg-gradient-to-br from-slate-50 via-white to-violet-50 font-sans"
>
	<!-- Header navigation -->
	<header class="border-b border-slate-200/60 bg-white/80 backdrop-blur-sm sticky top-0 z-10">
		<div class="mx-auto max-w-4xl px-4 py-3 flex items-center justify-between">
			<a href="/" class="flex items-center gap-2.5 hover:opacity-80 transition-opacity">
				<div class="w-7 h-7 rounded-lg overflow-hidden shadow-sm shadow-violet-200/50">
					<img src="/logo.webp" alt="filez.zone" class="w-full h-full object-cover" />
				</div>
				<span class="text-sm font-semibold text-slate-800">filez.zone</span>
			</a>

			<nav class="flex items-center gap-1">
				<a
					href="/"
					class="px-3 py-1.5 text-xs font-medium text-slate-500 hover:text-slate-800 hover:bg-slate-100 rounded-lg transition-colors"
				>
					Home
				</a>
				<a
					href="/zero-knowledge"
					class="px-3 py-1.5 text-xs font-medium text-slate-500 hover:text-violet-600 hover:bg-violet-50 rounded-lg transition-colors"
				>
					Zero Knowledge
				</a>
				<a
					href="/privacy"
					class="px-3 py-1.5 text-xs font-medium text-slate-400 hover:text-slate-600 hover:bg-slate-100 rounded-lg transition-colors hidden sm:inline-block"
				>
					Privacy
				</a>

				<!-- Saved URLs (authenticated users only) -->
				{#if auth.isAuthenticated}
					<a
						href="/urls"
						class="px-3 py-1.5 text-xs font-medium text-violet-600 hover:text-violet-800 hover:bg-violet-50 rounded-lg transition-colors"
					>
						Saved URLs
					</a>
				{/if}

				<!-- Auth section -->
				{#if auth.isAuthenticated && auth.user}
					<!-- User menu -->
					<div class="relative ml-2">
						<button
							onclick={toggleMenu}
							class="flex items-center gap-2 px-3 py-1.5 text-xs font-medium text-slate-600 hover:bg-slate-100 rounded-lg transition-colors cursor-pointer"
						>
							<div
								class="w-6 h-6 rounded-full bg-violet-100 text-violet-600 flex items-center justify-center text-[10px] font-bold"
							>
								{auth.user.name.charAt(0).toUpperCase()}
							</div>
							<span class="hidden sm:inline">{auth.user.name}</span>
							<svg
								xmlns="http://www.w3.org/2000/svg"
								class="w-3 h-3 text-slate-400"
								fill="none"
								viewBox="0 0 24 24"
								stroke="currentColor"
								stroke-width="2"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="M19.5 8.25l-7.5 7.5-7.5-7.5"
								/>
							</svg>
						</button>

						<!-- Dropdown -->
						{#if showMenu}
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div
								class="fixed inset-0 z-20"
								onclick={closeMenu}
								onkeydown={(e: KeyboardEvent) => {
									if (e.key === 'Escape') closeMenu();
								}}
								role="presentation"
							></div>
							<div
								class="absolute right-0 top-full mt-1 w-48 bg-white rounded-xl shadow-lg border border-slate-200 py-1 z-30"
							>
								<div class="px-3 py-2 border-b border-slate-100">
									<p class="text-xs font-medium text-slate-700">{auth.user.name}</p>
									<p class="text-[10px] text-slate-400">{auth.user.email}</p>
								</div>
								<a
									href="/urls"
									onclick={closeMenu}
									class="block w-full text-left px-3 py-2 text-xs text-violet-600 hover:bg-violet-50 transition-colors cursor-pointer"
								>
									Saved URLs
								</a>
								<button
									onclick={handleLogout}
									class="w-full text-left px-3 py-2 text-xs text-slate-600 hover:bg-slate-50 transition-colors cursor-pointer"
								>
									Log out
								</button>
								<button
									onclick={handleDeleteAccount}
									class="w-full text-left px-3 py-2 text-xs text-red-600 hover:bg-red-50 transition-colors cursor-pointer"
								>
									Delete account
								</button>
							</div>
						{/if}
					</div>
				{:else}
					<div class="flex items-center gap-1 ml-2">
						<a
							href="/login"
							class="px-3 py-1.5 text-xs font-medium text-slate-500 hover:text-slate-800 hover:bg-slate-100 rounded-lg transition-colors"
						>
							Log in
						</a>
						<a
							href="/register"
							class="px-3 py-1.5 text-xs font-medium text-white bg-violet-500 hover:bg-violet-600 rounded-lg transition-colors shadow-sm"
						>
							Sign up
						</a>
					</div>
				{/if}
			</nav>
		</div>
	</header>

	<main class="flex flex-1 items-center justify-center p-4">
		{@render children()}
	</main>

	<footer class="border-t border-slate-200/60 bg-white/60 backdrop-blur-sm">
		<div
			class="mx-auto max-w-4xl px-4 py-4 flex flex-col sm:flex-row items-center justify-between gap-2 text-xs text-slate-400"
		>
			<span>&copy; {currentYear} filez.zone</span>
			<nav class="flex gap-4 items-center">
				<a
					href="https://github.com/sha-zone/file-sharing"
					target="_blank"
					rel="noopener noreferrer"
					class="hover:text-slate-600 transition-colors inline-flex items-center gap-1.5"
					title="View source on GitHub"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="w-3.5 h-3.5"
						fill="currentColor"
						viewBox="0 0 16 16"
					>
						<path
							d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0016 8c0-4.42-3.58-8-8-8"
						/>
					</svg>
					GitHub
				</a>
				<a href="/zero-knowledge" class="hover:text-slate-600 transition-colors">Zero Knowledge</a>
				<a href="/tos" class="hover:text-slate-600 transition-colors">Terms of Service</a>
				<a href="/privacy" class="hover:text-slate-600 transition-colors">Privacy Policy</a>
				<a href="/cookies" class="hover:text-slate-600 transition-colors">Cookie Policy</a>
			</nav>
		</div>
	</footer>
</div>
