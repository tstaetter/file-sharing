<script lang="ts">
	import { auth } from '$lib/auth.svelte';
	import { goto } from '$app/navigation';

	let name = $state('');
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
		if (!name || !email || !password) return;
		const ok = await auth.signUp(email, password, name);
		if (ok) goto('/');
	}
</script>

<svelte:head>
	<title>Create Account — filez.zone</title>
	<meta name="description" content="Create a filez.zone account to manage your encrypted file shares." />
	<meta name="robots" content="noindex" />
</svelte:head>

<div class="w-full max-w-sm">
	<div
		class="bg-white rounded-2xl shadow-lg shadow-slate-200/50 border border-slate-100 p-8"
	>
		<h1 class="text-xl font-semibold text-slate-800 text-center mb-1">Create an account</h1>
		<p class="text-sm text-slate-500 text-center mb-6">Start sharing files securely</p>

		<form onsubmit={handleSubmit} class="space-y-4">
			<!-- Name -->
			<div>
				<label for="name" class="block text-xs font-medium text-slate-600 mb-1">
					Name
				</label>
				<input
					id="name"
					type="text"
					bind:value={name}
					autocomplete="name"
					required
					class="w-full px-3 py-2.5 text-sm border rounded-xl outline-none transition-colors
						{submitted && !name
						? 'border-red-300 bg-red-50/50 focus:border-red-400'
						: 'border-slate-200 bg-white focus:border-violet-400 focus:ring-2 focus:ring-violet-100'}"
				/>
			</div>

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
					autocomplete="new-password"
					minlength={8}
					required
					class="w-full px-3 py-2.5 text-sm border rounded-xl outline-none transition-colors
						{submitted && (!password || password.length < 8)
						? 'border-red-300 bg-red-50/50 focus:border-red-400'
						: 'border-slate-200 bg-white focus:border-violet-400 focus:ring-2 focus:ring-violet-100'}"
				/>
				{#if submitted && password && password.length < 8}
					<p class="text-xs text-red-500 mt-1">Password must be at least 8 characters.</p>
				{/if}
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
					Creating account…
				{:else}
					Create account
				{/if}
			</button>
		</form>
	</div>

	<p class="text-center text-xs text-slate-400 mt-4">
		Already have an account?
		<a
			href="/login"
			class="text-violet-500 hover:text-violet-600 font-medium transition-colors"
		>
			Log in
		</a>
	</p>
</div>
