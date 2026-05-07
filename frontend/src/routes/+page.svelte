<script lang="ts">
	import { uploadFile } from '$lib/upload';
	import { createCapabilityUrl } from '$lib';
	import { PUBLIC_PREFIX } from '$env/static/public';

	let file = $state<File | undefined>(undefined);
	let link = $state('');
	let uploading = $state(false);
	let fileName = $state('');

	function handleFile(e: Event) {
		const f = (e.target as HTMLInputElement).files?.[0];
		file = f;
		fileName = f?.name ?? '';
	}

	async function upload() {
		if (!file) return;
		uploading = true;
		try {
			const { raw, fileId } = await uploadFile(file);
			link = await createCapabilityUrl(PUBLIC_PREFIX, fileId, raw);
		} finally {
			uploading = false;
		}
	}
</script>

<svelte:head>
	<title>sha.zone — Secure End-to-End Encrypted File Sharing</title>
	<meta
		name="description"
		content="Share files securely with end-to-end AES-256-GCM encryption. Files are encrypted in your browser and deleted after download. No accounts, no tracking, no server access to your data."
	/>
	<meta property="og:title" content="sha.zone — Secure End-to-End Encrypted File Sharing" />
	<meta
		property="og:description"
		content="Share files securely with end-to-end AES-256-GCM encryption. Files are encrypted in your browser and deleted after download. No accounts, no tracking."
	/>
	<meta property="og:url" content="https://sha.zone" />
	<meta property="og:type" content="website" />
	<meta name="twitter:card" content="summary" />
	<meta name="twitter:title" content="sha.zone — Secure End-to-End Encrypted File Sharing" />
	<meta
		name="twitter:description"
		content="Share files securely with end-to-end AES-256-GCM encryption. Files are encrypted in your browser and deleted after download."
	/>
	<link rel="canonical" href="https://sha.zone" />
	<script type="application/ld+json">
		{{
			"@context": "https://schema.org",
			"@type": "WebApplication",
			"name": "sha.zone",
			"url": "https://sha.zone",
			"description": "End-to-end encrypted file sharing. Files are encrypted in your browser and deleted after download.",
			"applicationCategory": "UtilityApplication",
			"operatingSystem": "Any",
			"offers": {{
				"@type": "Offer",
				"price": "0",
				"priceCurrency": "USD"
			}},
			"featureList": [
				"End-to-end AES-256-GCM encryption",
				"Client-side encryption — server never sees plaintext",
				"Burn-after-reading — files deleted after first download",
				"No account required",
				"No tracking or analytics"
			]
		}}
	</script>
</svelte:head>

<div class="w-full max-w-lg">
	<!-- Card -->
	<div class="bg-white rounded-2xl shadow-lg shadow-slate-200/50 border border-slate-100 p-8">
		<!-- Icon -->
		<div
			class="mx-auto w-14 h-14 bg-gradient-to-br from-blue-400 to-blue-600 rounded-xl flex items-center justify-center mb-6 shadow-md shadow-blue-200/50"
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
					d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
				/>
			</svg>
		</div>

		<!-- Title -->
		<h1 class="text-2xl font-semibold text-slate-800 text-center mb-2">Share a file securely</h1>
		<p class="text-sm text-slate-500 text-center mb-8">
			End-to-end encrypted. The key stays in your browser.
		</p>

		<!-- File input area -->
		<label
			class="flex flex-col items-center justify-center w-full h-36 border-2 border-dashed rounded-xl cursor-pointer transition-all duration-200
					{file
				? 'border-blue-300 bg-blue-50/50'
				: 'border-slate-200 bg-slate-50/50 hover:border-slate-300 hover:bg-slate-100/50'}"
		>
			<div class="flex flex-col items-center gap-2">
				{#if fileName}
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="w-8 h-8 text-blue-500"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="1.5"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z"
						/>
					</svg>
					<span class="text-sm font-medium text-slate-700">{fileName}</span>
					<span class="text-xs text-slate-400">Click to change file</span>
				{:else}
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="w-8 h-8 text-slate-400"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="1.5"
					>
						<path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
					</svg>
					<span class="text-sm font-medium text-slate-500">Choose a file</span>
					<span class="text-xs text-slate-400">or drag and drop</span>
				{/if}
			</div>
			<input type="file" onchange={handleFile} class="hidden" />
		</label>

		<!-- Upload button -->
		<button
			onclick={upload}
			disabled={!file || uploading}
			class="mt-5 w-full py-3 rounded-xl font-medium text-sm transition-all duration-200
					{file && !uploading
				? 'bg-gradient-to-r from-blue-500 to-blue-600 text-white shadow-md shadow-blue-200/50 hover:from-blue-600 hover:to-blue-700 hover:shadow-lg active:scale-[0.98]'
				: 'bg-slate-100 text-slate-400 cursor-not-allowed'}"
		>
			{#if uploading}
				<span class="flex items-center justify-center gap-2">
					<svg class="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
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
					Encrypting &amp; uploading…
				</span>
			{:else}
				Encrypt &amp; upload
			{/if}
		</button>

		<!-- Result link -->
		{#if link}
			<div class="mt-6 p-4 bg-green-50 border border-green-200 rounded-xl">
				<p class="text-xs font-medium text-green-700 mb-2 uppercase tracking-wide">
					Share this link
				</p>
				<div class="flex items-center gap-2">
					<input
						type="text"
						value={link}
						readonly
						class="flex-1 text-xs bg-white border border-green-200 rounded-lg px-3 py-2 text-slate-700 outline-none"
					/>
					<button
						onclick={() => navigator.clipboard.writeText(link)}
						class="shrink-0 p-2 bg-white border border-green-200 rounded-lg text-green-600 hover:bg-green-100 transition-colors cursor-pointer"
						title="Copy to clipboard"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="w-4 h-4"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
							stroke-width="2"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
							/>
						</svg>
					</button>
				</div>
				<p class="text-xs text-green-600 mt-2">
					The recipient just opens this link — the key is in the URL.
				</p>
			</div>
		{/if}
	</div>

	<p class="text-center text-xs text-slate-400 mt-6">
		Files are encrypted with AES-GCM before upload &middot; keys never touch the server
	</p>
</div>
