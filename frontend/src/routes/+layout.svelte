<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import '../app.css';

	let { children } = $props();

	const currentYear = new Date().getFullYear();
	const siteUrl = 'https://sha.zone';
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<link rel="apple-touch-icon" href="/favicon.svg" />
	<link rel="icon" type="image/svg+xml" href="/favicon.svg" />
	<meta
		name="description"
		content="End-to-end encrypted file sharing. Files are encrypted in your browser before upload and deleted after download. No accounts, no tracking."
	/>
	<meta name="robots" content="index, follow" />
	<meta property="og:site_name" content="sha.zone" />
	<meta property="og:type" content="website" />
	<meta property="og:title" content="sha.zone — Secure File Sharing" />
	<meta
		property="og:description"
		content="End-to-end encrypted file sharing. Files are encrypted in your browser and deleted after download. No accounts needed."
	/>
	<meta property="og:url" content={siteUrl} />
	<meta property="og:image" content="{siteUrl}/favicon.svg" />
	<meta property="og:image:type" content="image/svg+xml" />
	<meta name="twitter:card" content="summary" />
	<meta name="twitter:title" content="sha.zone — Secure File Sharing" />
	<meta
		name="twitter:description"
		content="End-to-end encrypted file sharing. Files are encrypted in your browser and deleted after download. No accounts needed."
	/>
	<meta name="twitter:image" content="{siteUrl}/favicon.svg" />
	<link rel="canonical" href={siteUrl} />
</svelte:head>

<div
	class="flex min-h-screen flex-col bg-gradient-to-br from-slate-50 via-white to-blue-50 font-sans"
>
	<main class="flex flex-1 items-center justify-center p-4">
		{@render children()}
	</main>

	<footer class="border-t border-slate-200/60 bg-white/60 backdrop-blur-sm">
		<div
			class="mx-auto max-w-4xl px-4 py-4 flex flex-col sm:flex-row items-center justify-between gap-2 text-xs text-slate-400"
		>
			<span>&copy; {currentYear} sha.zone</span>
			<nav class="flex gap-4">
				<a href="/tos" class="hover:text-slate-600 transition-colors">Terms of Service</a>
				<a href="/privacy" class="hover:text-slate-600 transition-colors">Privacy Policy</a>
				<a href="/cookies" class="hover:text-slate-600 transition-colors">Cookie Policy</a>
			</nav>
		</div>
	</footer>
</div>
