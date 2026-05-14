<script lang="ts">
    import {onMount} from 'svelte';
    import {SvelteSet} from 'svelte/reactivity';
    import {auth} from '$lib/auth.svelte';
    import {checkFile, deleteUrl, extractFileId, listUrls, type SavedUrlItem} from '$lib/savedUrls';
    import {goto} from '$app/navigation';

    let urls = $state<SavedUrlItem[]>([]);
    let loading = $state(true);
    let error = $state<string | null>(null);
    let page = $state(1);
    let total = $state(0);
    let perPage = $state(10);
    let copiedId = $state<string | null>(null);
    let consumedIds = new SvelteSet<string>();
    let deletingIds = new SvelteSet<string>();

    const totalPages = $derived(Math.max(1, Math.ceil(total / perPage)));

    async function fetchUrls() {
        if (!auth.token) return;
        loading = true;
        error = null;
        consumedIds.clear();
        try {
            const res = await listUrls(auth.token, page, perPage);
            urls = res.urls;
            total = res.total;
            checkUrls();
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to load saved URLs';
        } finally {
            loading = false;
        }
    }

    async function checkUrls() {
        const checks = urls
            .map((item) => {
                const fileId = extractFileId(item.url);
                return fileId ? {id: item.id, fileId} : null;
            })
            .filter((x): x is { id: string; fileId: string } => x !== null);

        const results = await Promise.all(
            checks.map(async ({id, fileId}) => {
                const exists = await checkFile(fileId);
                return {id, exists};
            })
        );

        for (const {id, exists} of results) {
            if (!exists) {
                consumedIds.add(id);
            }
        }
    }

    function prevPage() {
        if (page > 1) {
            page--;
            fetchUrls();
        }
    }

    function nextPage() {
        if (page < totalPages) {
            page++;
            fetchUrls();
        }
    }

    async function copyUrl(url: string, id: string) {
        try {
            await navigator.clipboard.writeText(url);
            copiedId = id;
            setTimeout(() => {
                copiedId = null;
            }, 2000);
        } catch {
            // Clipboard API may not be available
        }
    }

    async function handleDelete(id: string) {
        if (!auth.token) return;
        if (!confirm('Are you sure you want to delete this saved URL?')) return;

        deletingIds.add(id);
        try {
            await deleteUrl(id, auth.token);
            urls = urls.filter((u) => u.id !== id);
            total--;
            if (urls.length === 0 && page > 1) {
                page--;
                fetchUrls();
            }
        } catch (e) {
            alert(e instanceof Error ? e.message : 'Failed to delete URL');
        } finally {
            deletingIds.delete(id);
        }
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
        if (!auth.isAuthenticated) {
            goto('/login');
            return;
        }
        fetchUrls();
    });
</script>

<svelte:head>
    <title>Saved URLs — filez.zone</title>
    <meta content="Your saved file-sharing capability URLs on filez.zone." name="description"/>
    <meta content="noindex" name="robots"/>
</svelte:head>

<div class="w-full max-w-2xl">
    <!-- Header -->
    <div class="mb-6">
        <h1 class="text-2xl font-semibold text-slate-800">Saved URLs</h1>
        <p class="text-sm text-slate-500 mt-1">
            Your saved file-sharing links. Each link can only be opened once.
        </p>
    </div>

    <!-- Loading state -->
    {#if loading && urls.length === 0}
        <div
                class="bg-white rounded-2xl shadow-lg shadow-slate-200/50 border border-slate-100 p-12 text-center"
        >
            <div
                    class="mx-auto w-8 h-8 border-2 border-violet-200 border-t-violet-500 rounded-full animate-spin"
            ></div>
            <p class="text-sm text-slate-400 mt-4">Loading your saved URLs…</p>
        </div>
    {:else if error}
        <!-- Error state -->
        <div
                class="bg-white rounded-2xl shadow-lg shadow-slate-200/50 border border-red-100 p-8 text-center"
        >
            <div class="mx-auto w-12 h-12 rounded-full bg-red-50 flex items-center justify-center mb-4">
                <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="w-6 h-6 text-red-500"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        stroke-width="1.5"
                >
                    <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z"
                    />
                </svg>
            </div>
            <p class="text-sm font-medium text-red-700 mb-1">Failed to load URLs</p>
            <p class="text-xs text-red-500 mb-4">{error}</p>
            <button
                    onclick={fetchUrls}
                    class="px-4 py-2 text-xs font-medium text-white bg-violet-500 hover:bg-violet-600 rounded-lg transition-colors cursor-pointer"
            >
                Try again
            </button>
        </div>
    {:else if urls.length === 0}
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
                When you upload a file while logged in, the link is saved automatically.
            </p>
            <a
                    href="/"
                    class="inline-block px-4 py-2 text-xs font-medium text-white bg-violet-500 hover:bg-violet-600 rounded-lg transition-colors"
            >
                Upload a file
            </a>
        </div>
    {:else}
        <!-- URL list -->
        <div class="space-y-3">
            {#each urls as item (item.id)}
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

                                <!-- Open link -->
                                <a
                                        href={item.url}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="p-2 rounded-lg text-slate-400 hover:text-violet-600 hover:bg-violet-50 transition-colors"
                                        title="Open link"
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
                                                d="M13.5 6H5.25A2.25 2.25 0 003 8.25v10.5A2.25 2.25 0 005.25 21h10.5A2.25 2.25 0 0018 18.75V10.5m-10.5 6L21 3m0 0h-5.25M21 3v5.25"
                                        />
                                    </svg>
                                </a>

                                <!-- Delete button -->
                                <button
                                        onclick={() => handleDelete(item.id)}
                                        disabled={deletingIds.has(item.id)}
                                        class="p-2 rounded-lg text-slate-400 hover:text-red-600 hover:bg-red-50 transition-colors cursor-pointer disabled:opacity-50"
                                        title="Delete saved URL"
                                        aria-label="Delete saved URL"
                                >
                                    {#if deletingIds.has(item.id)}
                                        <div
                                                class="w-4 h-4 border-2 border-red-200 border-t-red-500 rounded-full animate-spin"
                                        ></div>
                                    {:else}
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
                                            <polyline points="3 6 5 6 21 6"/>
                                            <path
                                                    d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
                                            />
                                            <line x1="10" y1="11" x2="10" y2="17"/>
                                            <line x1="14" y1="11" x2="14" y2="17"/>
                                        </svg>
                                    {/if}
                                </button>
                            {/if}
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

        <!-- Loading overlay for pagination -->
        {#if loading}
            <div class="flex justify-center mt-4">
                <div
                        class="w-5 h-5 border-2 border-violet-200 border-t-violet-500 rounded-full animate-spin"
                ></div>
            </div>
        {/if}
    {/if}
</div>
