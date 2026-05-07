<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import '../app.css';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div class="flex min-h-screen flex-col bg-gradient-to-br from-slate-50 via-white to-blue-50">
	<main class="flex flex-1 items-center justify-center p-4">
		{@render children()}
	</main>

	<footer class="border-t border-slate-200/60 bg-white/60 backdrop-blur-sm">
		<div
			class="mx-auto max-w-4xl px-4 py-4 flex flex-col sm:flex-row items-center justify-between gap-2 text-xs text-slate-400"
		>
			<span>&copy; {new Date().getFullYear()} sha.zone</span>
			<nav class="flex gap-4">
				<a href="/tos" class="hover:text-slate-600 transition-colors">Terms of Service</a>
				<a href="/privacy" class="hover:text-slate-600 transition-colors">Privacy Policy</a>
				<a href="/cookies" class="hover:text-slate-600 transition-colors">Cookie Policy</a>
			</nav>
		</div>
	</footer>
</div>
