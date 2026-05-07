<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { PUBLIC_API_PREFIX } from '$env/static/public';
	import { page } from '$app/state';
	import { downloadFile, base64ToBytes } from 'shazoneSDK';
	import logo from '$lib/assets/logo.webp';

	let error = $state<string | null>(null);
	let loading = $state(true);
	let progress = $state(0);
	let downloadUrl = $state<string | null>(null);
	let fileName = $state<string>('download');

	onMount(async () => {
		try {
			const key = location.hash.slice(1);
			if (!key) {
				error = 'Missing decryption key in URL hash';
				loading = false;
				return;
			}

			const id = page.params.id;
			const rawKey = base64ToBytes(key);
			const result = await downloadFile(PUBLIC_API_PREFIX, id, rawKey, (p) => (progress = p));

			fileName = result.fileName;
			downloadUrl = URL.createObjectURL(result.blob);
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	});

	onDestroy(() => {
		if (downloadUrl) URL.revokeObjectURL(downloadUrl);
	});
</script>

<svelte:head>
	<title>Decrypting your file — sha.zone</title>
	<meta
		name="description"
		content="Download your encrypted file. Files are decrypted in your browser and deleted from the server after download."
	/>
	<meta name="robots" content="noindex, nofollow" />
	<meta property="og:title" content="Download your file — sha.zone" />
	<meta
		property="og:description"
		content="Download your encrypted file. Files are decrypted in your browser and deleted from the server after download."
	/>
	<meta name="twitter:card" content="summary" />
</svelte:head>

<div class="w-full max-w-lg">
	<div class="bg-white rounded-2xl shadow-lg shadow-slate-200/50 border border-slate-100 p-8">
		<!-- Logo -->
		<div class="mx-auto w-14 h-14 rounded-xl overflow-hidden mb-6 shadow-md shadow-cyan-200/50">
			<img src={logo} alt="sha.zone" class="w-full h-full object-cover" />
		</div>

		<h1 class="text-2xl font-semibold text-slate-800 text-center mb-6">Decrypting your file</h1>

		{#if loading}
			<div class="py-8">
				<div class="flex items-center justify-between mb-2">
					<span class="text-sm text-cyan-600 font-medium">Fetching &amp; decrypting…</span>
					<span class="text-sm text-cyan-500">{(progress * 100).toFixed(0)}%</span>
				</div>
				<div class="w-full h-2.5 bg-cyan-100 rounded-full overflow-hidden">
					<div
						class="h-full bg-gradient-to-r from-cyan-500 to-cyan-600 rounded-full transition-all duration-300 ease-out"
						style="width: {progress * 100}%"
					></div>
				</div>
			</div>
		{:else if error}
			<div class="p-4 bg-red-50 border border-red-200 rounded-xl">
				<div class="flex items-start gap-3">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="w-5 h-5 text-red-500 shrink-0 mt-0.5"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z"
						/>
					</svg>
					<div>
						<p class="text-sm font-medium text-red-800">Decryption failed</p>
						<p class="text-sm text-red-600 mt-1">{error}</p>
					</div>
				</div>
			</div>
		{:else if downloadUrl}
			<div class="text-center">
				<div
					class="mx-auto w-16 h-16 bg-cyan-100 rounded-full flex items-center justify-center mb-4"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="w-8 h-8 text-cyan-600"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
						/>
					</svg>
				</div>
				<p class="text-sm text-slate-600 mb-2">File decrypted successfully</p>
				<p class="text-xs text-slate-400 mb-6">{fileName}</p>
				<button
					onclick={() => {
						const a = document.createElement('a');
						a.href = downloadUrl!;
						a.download = fileName;
						document.body.appendChild(a);
						a.click();
						document.body.removeChild(a);
					}}
					class="w-full py-3 rounded-xl font-medium text-sm transition-all duration-200 bg-gradient-to-r from-cyan-500 to-cyan-600 text-white shadow-md shadow-cyan-200/50 hover:from-cyan-600 hover:to-cyan-700 hover:shadow-lg active:scale-[0.98] cursor-pointer"
				>
					Download {fileName}
				</button>
				<p class="text-xs text-slate-400 mt-4">This file has been deleted from the server.</p>
			</div>
		{/if}
	</div>

	<p class="text-center text-xs text-slate-400 mt-6">
		<a href="/" class="text-violet-500 hover:text-violet-600 transition-colors">← Upload a file</a>
	</p>
</div>
