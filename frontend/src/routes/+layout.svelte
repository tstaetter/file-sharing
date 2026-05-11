<script lang="ts">
	import '../app.css';

	let { children } = $props();

	const currentYear = new Date().getFullYear();
	const siteUrl = 'https://filez.zone';
</script>

<svelte:head>
	<link rel="icon" type="image/webp" href="/logo.webp" />
	<link rel="apple-touch-icon" href="/logo.webp" />
	<meta
		name="description"
		content="End-to-end encrypted file sharing. Files are encrypted in your browser before upload and deleted after download. No accounts, no tracking."
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
			<nav class="flex gap-4">
				<a href="/zero-knowledge" class="hover:text-slate-600 transition-colors">Zero Knowledge</a>
				<a href="/tos" class="hover:text-slate-600 transition-colors">Terms of Service</a>
				<a href="/privacy" class="hover:text-slate-600 transition-colors">Privacy Policy</a>
				<a href="/cookies" class="hover:text-slate-600 transition-colors">Cookie Policy</a>
			</nav>
		</div>
	</footer>
</div>
