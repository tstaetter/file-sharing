<script lang="ts">
	import { onMount } from 'svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import { localUrls, checkFile, extractFileId, type LocalUrlItem } from '$lib/savedUrls.svelte';
	import { resolve } from '$app/paths';

	const PER_PAGE = 10;

	let page = $state(1);
	let copiedId = $state<string | null>(null);
	let consumedIds = new SvelteSet<string>();
	let checking = $state(false);

	const paginatedUrls = $derived.by(() => {
		const start = (page - 1) * PER_PAGE;
		return localUrls.urls.slice(start, start + PER_PAGE);
	});

	const totalPages = $derived(Math.max(1, Math.ceil(localUrls.urls.length / PER_PAGE)));

	async function checkUrls() {
		if (checking || localUrls.urls.length === 0) return;
		checking = true;
		consumedIds.clear();

		const checks = localUrls.urls
			.map((item) => {
				const fileId = extractFileId(item.url);
				return fileId ? { id: item.id, fileId } : null;
			})
			.filter((x): x is { id: string; fileId: string } => x !== null);

		const results = await Promise.all(
			checks.map(async ({ id, fileId }) => {
				const exists = await checkFile(fileId);
				return { id, exists };
			})
		);

		for (const { id, exists } of results) {
			if (!exists) consumedIds.add(id);
		}

		checking = false;
	}

	function prevPage() {
		if (page > 1) page--;
	}

	function nextPage() {
		if (page < totalPages) page++;
	}

	// Reset to page 1 if data shrinks below the current window
	$effect(() => {
		if (localUrls.urls.length > 0 && page > totalPages) {
			page = totalPages;
		}
	});

	async function copyUrl(url: string, id: string) {
		try {
			await navigator.clipboard.writeText(url);
			copiedId = id;
			setTimeout(() => (copiedId = null), 2000);
		} catch {
			// Clipboard API may not be available
		}
	}

	function handleDelete(id: string) {
		if (!confirm('Are you sure you want to delete this saved URL?')) return;
		localUrls.remove(id);
		consumedIds.delete(id);
		// $effect above handles page correction
	}

	function formatDate(iso: string): string {
		try {
			const d = new Date(iso);
			return d.toLocaleDateString(undefined, {
				year: 'numeric',
				month: 'short',
				day: 'numeric',
				hour: '2-digit',
				minute: '2-digit'
			});
		} catch {
			return iso;
		}
	}

	onMount(() => {
		checkUrls();
	});
</script>

<svelte:head>
	<title>Saved URLs — filez.zone</title>
	<meta content="Your saved file-sharing capability URLs on filez.zone." name="description" />
	<meta content="noindex" name="robots" />
</svelte:head>

<div class="w-full max-w-2xl">
	<!-- Header -->
	<div class="mb-6">
		<h1 class="text-2xl font-semibold text-slate-800">Saved URLs</h1>
		<p class="text-sm text-slate-500 mt-1">
			Your saved file-sharing links. Each link can only be opened once.
		</p>
	</div>

	{#if localUrls.urls.length === 0}
		<!-- Empty state -->
		<div
			class="bg-white rounded-2xl shadow-lg shadow-slate-200/50 border border-slate-100 p-12 text-center"
		>
			<div
				class="mx-auto w-12 h-12 rounded-full bg-violet-50 flex items-center justify-center mb-4"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="w-6 h-6 text-violet-400"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
					stroke-width="1.5"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M13.19 8.688a4.5 4.5 0 011.242 7.244l-4.5 4.5a4.5 4.5 0 01-6.364-6.364l1.757-1.757m13.35-.622l1.757-1.757a4.5 4.5 0 00-6.364-6.364l-4.5 4.5a4.5 4.5 0 001.242 7.244"
					/>
				</svg>
			</div>
			<p class="text-sm font-medium text-slate-600 mb-1">No saved URLs yet</p>
			<p class="text-xs text-slate-400 mb-6">
				When you upload a file, the link is saved automatically to this browser.
			</p>
			<a
				href={resolve('/')}
				class="inline-block px-4 py-2 text-xs font-medium text-white bg-violet-500 hover:bg-violet-600 rounded-lg transition-colors"
			>
				Upload a file
			</a>
		</div>
	{:else}
		<!-- URL list -->
		<div class="space-y-3">
			{#each paginatedUrls as item (item.id)}
				<div
					class="bg-white rounded-xl shadow-sm border border-slate-100 p-4 hover:shadow-md hover:border-slate-200 transition-all duration-200"
				>
					<div class="flex items-start justify-between gap-3">
						<div class="flex-1 min-w-0">
							<!-- Title or truncated URL -->
							<p class="text-sm font-medium text-slate-800 truncate">
								{item.title ?? item.url}
							</p>
							{#if item.title}
								<p class="text-xs text-slate-400 truncate mt-0.5">{item.url}</p>
							{/if}
							<p class="text-[10px] text-slate-400 mt-1.5">
								Saved {formatDate(item.created_at)}
							</p>
						</div>

						<!-- Actions -->
						<div class="flex items-center gap-1 shrink-0">
							{#if consumedIds.has(item.id)}
								<span
									class="inline-flex items-center px-2 py-1 rounded-md bg-slate-100 text-[10px] font-medium text-slate-400"
								>
									Already used
								</span>
							{:else}
								<!-- Copy button -->
								<button
									onclick={() => copyUrl(item.url, item.id)}
									class="p-2 rounded-lg text-slate-400 hover:text-violet-600 hover:bg-violet-50 transition-colors cursor-pointer"
									title={copiedId === item.id ? 'Copied!' : 'Copy link'}
								>
									{#if copiedId === item.id}
										<svg
											xmlns="http://www.w3.org/2000/svg"
											class="w-4 h-4 text-violet-500"
											fill="none"
											viewBox="0 0 24 24"
											stroke="currentColor"
											stroke-width="2"
										>
											<path
												stroke-linecap="round"
												stroke-linejoin="round"
												d="M4.5 12.75l6 6 9-13.5"
											/>
										</svg>
									{:else}
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
									{/if}
								</button>
							{/if}

							<!-- Delete button (always visible) -->
							<button
								onclick={() => handleDelete(item.id)}
								class="p-2 rounded-lg text-slate-400 hover:text-red-600 hover:bg-red-50 transition-colors cursor-pointer"
								title="Delete saved URL"
								aria-label="Delete saved URL"
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									class="w-4 h-4"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
									stroke-linecap="round"
									stroke-linejoin="round"
								>
									<polyline points="3 6 5 6 21 6" />
									<path
										d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
									/>
									<line x1="10" y1="11" x2="10" y2="17" />
									<line x1="14" y1="11" x2="14" y2="17" />
								</svg>
							</button>
						</div>
					</div>
				</div>
			{/each}
		</div>

		<!-- Pagination -->
		{#if totalPages > 1}
			<div class="flex items-center justify-between mt-6">
				<button
					onclick={prevPage}
					disabled={page <= 1}
					class="px-4 py-2 text-xs font-medium rounded-lg transition-colors cursor-pointer
						{page > 1
						? 'text-slate-600 bg-white border border-slate-200 hover:bg-slate-50'
						: 'text-slate-300 bg-white border border-slate-100 cursor-not-allowed'}"
				>
					← Previous
				</button>

				<span class="text-xs text-slate-400">
					Page {page} of {totalPages}
				</span>

				<button
					onclick={nextPage}
					disabled={page >= totalPages}
					class="px-4 py-2 text-xs font-medium rounded-lg transition-colors cursor-pointer
						{page < totalPages
						? 'text-slate-600 bg-white border border-slate-200 hover:bg-slate-50'
						: 'text-slate-300 bg-white border border-slate-100 cursor-not-allowed'}"
				>
					Next →
				</button>
			</div>
		{/if}
	{/if}
</div>
