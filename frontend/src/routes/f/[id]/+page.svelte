<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { PUBLIC_API_PREFIX } from '$env/static/public';
	import { page } from '$app/state';
	import { downloadFile, base64ToBytes } from 'shazoneSDK';

	let error = $state<string | null>(null);
	let loading = $state(true);
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
			const result = await downloadFile(PUBLIC_API_PREFIX, id, rawKey);

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

<div
	class="min-h-screen bg-gradient-to-br from-slate-50 via-white to-blue-50 flex items-center justify-center p-4"
>
	<div class="w-full max-w-lg">
		<div class="bg-white rounded-2xl shadow-lg shadow-slate-200/50 border border-slate-100 p-8">
			<!-- Icon -->
			<div
				class="mx-auto w-14 h-14 bg-gradient-to-br from-emerald-400 to-emerald-600 rounded-xl flex items-center justify-center mb-6 shadow-md shadow-emerald-200/50"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="w-7 h-7 text-white"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
					stroke-width="2"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
					/>
				</svg>
			</div>

			<h1 class="text-2xl font-semibold text-slate-800 text-center mb-6">Decrypting your file</h1>

			{#if loading}
				<div class="flex flex-col items-center gap-4 py-8">
					<svg class="animate-spin w-10 h-10 text-blue-500" fill="none" viewBox="0 0 24 24">
						<circle
							class="opacity-25"
							cx="12"
							cy="12"
							r="10"
							stroke="currentColor"
							stroke-width="4"
						/>
						<path
							class="opacity-75"
							fill="currentColor"
							d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
						/>
					</svg>
					<p class="text-sm text-slate-500">Fetching and decrypting your file…</p>
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
						class="mx-auto w-16 h-16 bg-emerald-100 rounded-full flex items-center justify-center mb-4"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="w-8 h-8 text-emerald-600"
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
						class="w-full py-3 rounded-xl font-medium text-sm transition-all duration-200 bg-gradient-to-r from-emerald-500 to-emerald-600 text-white shadow-md shadow-emerald-200/50 hover:from-emerald-600 hover:to-emerald-700 hover:shadow-lg active:scale-[0.98] cursor-pointer"
					>
						Download {fileName}
					</button>
					<p class="text-xs text-slate-400 mt-4">This file has been deleted from the server.</p>
				</div>
			{/if}
		</div>

		<p class="text-center text-xs text-slate-400 mt-6">
			<a href="/" class="text-blue-500 hover:text-blue-600 transition-colors">← Upload a file</a>
		</p>
	</div>
</div>
