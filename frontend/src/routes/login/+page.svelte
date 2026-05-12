<script lang="ts">
	import { auth } from '$lib/auth.svelte';
	import { goto } from '$app/navigation';

	let email = $state('');
	let password = $state('');
	let submitted = $state(false);

	// Redirect if already authenticated
	$effect(() => {
		if (auth.isAuthenticated) {
			goto('/');
		}
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		submitted = true;
		if (!email || !password) return;
		const ok = await auth.signIn(email, password);
		if (ok) goto('/');
	}
</script>

<svelte:head>
	<title>Log In — filez.zone</title>
	<meta name="description" content="Log in to your filez.zone account." />
	<meta name="robots" content="noindex" />
</svelte:head>

<div class="w-full max-w-sm">
	<div
		class="bg-white rounded-2xl shadow-lg shadow-slate-200/50 border border-slate-100 p-8"
	>
		<h1 class="text-xl font-semibold text-slate-800 text-center mb-1">Welcome back</h1>
		<p class="text-sm text-slate-500 text-center mb-6">Log in to your account</p>

		<form onsubmit={handleSubmit} class="space-y-4">
			<!-- Email -->
			<div>
				<label for="email" class="block text-xs font-medium text-slate-600 mb-1">
					Email
				</label>
				<input
					id="email"
					type="email"
					bind:value={email}
					autocomplete="email"
					required
					class="w-full px-3 py-2.5 text-sm border rounded-xl outline-none transition-colors
						{submitted && !email
						? 'border-red-300 bg-red-50/50 focus:border-red-400'
						: 'border-slate-200 bg-white focus:border-violet-400 focus:ring-2 focus:ring-violet-100'}"
				/>
			</div>

			<!-- Password -->
			<div>
				<label for="password" class="block text-xs font-medium text-slate-600 mb-1">
					Password
				</label>
				<input
					id="password"
					type="password"
					bind:value={password}
					autocomplete="current-password"
					required
					class="w-full px-3 py-2.5 text-sm border rounded-xl outline-none transition-colors
						{submitted && !password
						? 'border-red-300 bg-red-50/50 focus:border-red-400'
						: 'border-slate-200 bg-white focus:border-violet-400 focus:ring-2 focus:ring-violet-100'}"
				/>
			</div>

			<!-- Error -->
			{#if auth.error}
				<div class="p-3 bg-red-50 border border-red-200 rounded-xl">
					<p class="text-xs text-red-600">{auth.error}</p>
				</div>
			{/if}

			<!-- Submit -->
			<button
				type="submit"
				disabled={auth.loading}
				class="w-full py-3 rounded-xl font-medium text-sm transition-all duration-200
					{!auth.loading
					? 'bg-gradient-to-r from-violet-500 to-violet-600 text-white shadow-md shadow-violet-200/50 hover:from-violet-600 hover:to-violet-700 hover:shadow-lg active:scale-[0.98]'
					: 'bg-slate-100 text-slate-400 cursor-not-allowed'}"
			>
				{#if auth.loading}
					Logging in…
				{:else}
					Log in
				{/if}
			</button>
		</form>
	</div>

	<p class="text-center text-xs text-slate-400 mt-4">
		Don't have an account?
		<a
			href="/register"
			class="text-violet-500 hover:text-violet-600 font-medium transition-colors"
		>
			Create one
		</a>
	</p>
</div>
