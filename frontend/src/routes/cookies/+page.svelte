<script lang="ts">
	const lastUpdated = new Date().toLocaleDateString('en-US', {
		month: 'long',
		day: 'numeric',
		year: 'numeric'
	});
</script>

<svelte:head>
	<title>Cookie Policy — filez.zone</title>
	<meta
		name="description"
		content="Cookie Policy for filez.zone. We don't use tracking, advertising, or analytics cookies. Learn about the minimal browser storage we use and how Cloudflare's security cookies work."
	/>
	<meta name="robots" content="index, follow" />
	<meta property="og:title" content="Cookie Policy — filez.zone" />
	<meta
		property="og:description"
		content="Cookie Policy for filez.zone. No tracking cookies, no analytics, no ads. Learn about our minimal approach to browser storage."
	/>
	<meta property="og:url" content="https://filez.zone/cookies" />
	<meta property="og:type" content="website" />
	<meta name="twitter:card" content="summary" />
	<meta name="twitter:title" content="Cookie Policy — filez.zone" />
	<meta
		name="twitter:description"
		content="Cookie Policy for filez.zone. No tracking, no analytics, no advertising cookies."
	/>
	<link rel="canonical" href="https://filez.zone/cookies" />
</svelte:head>

<div class="w-full max-w-2xl">
	<div
		class="bg-white rounded-2xl shadow-lg shadow-slate-200/50 border border-slate-100 p-8 sm:p-10"
	>
		<h1 class="text-2xl font-semibold text-slate-800 mb-1">Cookie Policy</h1>
		<p class="text-sm text-slate-400 mb-8">Last updated: {lastUpdated}</p>

		<div class="prose prose-slate prose-sm max-w-none space-y-6 text-slate-600 leading-relaxed">
			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">1. Overview</h2>
				<p>
					filez.zone is designed with privacy as a core principle. This Cookie Policy explains what
					browser storage technologies we use, why we use them, and — more importantly — what we <strong
						>do not</strong
					> use.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">2. Cookies We Do NOT Use</h2>
				<p>We do <strong>not</strong> use any of the following:</p>
				<ul class="list-disc pl-5 mt-2 space-y-2">
					<li>
						<strong>Tracking cookies:</strong> We do not place any cookies that track your browsing activity
						across sites.
					</li>
					<li>
						<strong>Advertising cookies:</strong> We do not use advertising networks, ad pixels, or retargeting
						cookies of any kind.
					</li>
					<li>
						<strong>Analytics cookies:</strong> We do not use Google Analytics, Meta Pixel, or any third-party
						analytics cookies.
					</li>
					<li>
						<strong>Social media cookies:</strong> We do not embed social media buttons or tracking pixels.
					</li>
					<li>
						<strong>Third-party cookies:</strong> We do not set or read cookies from any third-party domains.
					</li>
				</ul>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">3. Browser Storage We Use</h2>
				<p>Our Service uses minimal browser storage strictly for essential functionality:</p>

				<h3 class="text-sm font-semibold text-slate-700 mt-4 mb-1">Local & Session Storage</h3>
				<p>
					We may use the browser's <code class="bg-slate-100 px-1 py-0.5 rounded text-xs"
						>localStorage</code
					>
					or <code class="bg-slate-100 px-1 py-0.5 rounded text-xs">sessionStorage</code> APIs for:
				</p>
				<ul class="list-disc pl-5 mt-2 space-y-1">
					<li>
						<strong>Application state:</strong> Temporary UI state (e.g., form inputs, loading indicators)
						to provide a smooth user experience. This data is not transmitted to our servers and is cleared
						when you close your browser or clear your browsing data.
					</li>
				</ul>

				<h3 class="text-sm font-semibold text-slate-700 mt-4 mb-1">IndexedDB / Cache Storage</h3>
				<p>
					We do not currently use IndexedDB or the Cache API. If this changes in the future, we will
					update this policy and provide clear notice.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">4. First-Party Cookies</h2>
				<p>
					We do not set any first-party cookies. Our Service does not use cookies for session
					management, authentication, or any other purpose. All API requests are stateless and do
					not require session cookies.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">5. Third-Party Services</h2>

				<h3 class="text-sm font-semibold text-slate-700 mt-4 mb-1">Cloudflare</h3>
				<p>
					We use <strong>Cloudflare</strong> for content delivery, DDoS protection, and R2 object
					storage. Cloudflare may set its own cookies (such as
					<code class="bg-slate-100 px-1 py-0.5 rounded text-xs">__cf_bm</code>) as part of their
					bot management and security services. These cookies are:
				</p>
				<ul class="list-disc pl-5 mt-2 space-y-1">
					<li>Set by Cloudflare, not by filez.zone.</li>
					<li>Used solely for security and performance purposes.</li>
					<li>Not used for tracking, advertising, or building user profiles.</li>
					<li>
						Subject to Cloudflare's own privacy policy at <a
							href="https://www.cloudflare.com/privacypolicy/"
							class="text-blue-500 hover:text-blue-600 underline"
							target="_blank"
							rel="noopener">cloudflare.com/privacypolicy</a
						>.
					</li>
				</ul>
				<p class="mt-2">
					You can learn more about Cloudflare's cookies at <a
						href="https://www.cloudflare.com/cookie-policy/"
						class="text-blue-500 hover:text-blue-600 underline"
						target="_blank"
						rel="noopener">cloudflare.com/cookie-policy</a
					>.
				</p>

				<h3 class="text-sm font-semibold text-slate-700 mt-4 mb-1">Presigned Upload URLs</h3>
				<p>
					When you upload a file, your browser communicates directly with Cloudflare R2 using
					presigned URLs. These URLs contain authentication parameters in the query string (not
					cookies) and expire after one hour. No cookies are exchanged during the upload process.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">
					6. How to Control Browser Storage
				</h2>
				<p>You can control or delete browser storage at any time through your browser settings:</p>
				<ul class="list-disc pl-5 mt-2 space-y-1">
					<li><strong>Chrome:</strong> Settings → Privacy and security → Clear browsing data</li>
					<li><strong>Firefox:</strong> Settings → Privacy & Security → Cookies and Site Data</li>
					<li><strong>Safari:</strong> Preferences → Privacy → Manage Website Data</li>
					<li>
						<strong>Edge:</strong> Settings → Cookies and site permissions → Manage and delete cookies
					</li>
				</ul>
				<p class="mt-2">
					Since we use minimal browser storage and no tracking cookies, clearing your data will not
					affect your ability to use the Service. File encryption keys are stored exclusively in the
					URL hash fragment, which is never persisted by the browser.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">7. Do Not Track</h2>
				<p>
					We honor Do Not Track (DNT) signals by default — we do not track, profile, or collect
					behavioral data regardless of whether your browser sends a DNT header. Our Service is
					designed to be privacy-first without requiring any opt-out configuration.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">8. Changes to This Policy</h2>
				<p>
					We may update this Cookie Policy from time to time. Any changes will be reflected on this
					page with an updated "Last updated" date. If we introduce significant changes that affect
					how we use browser storage, we will provide clear notice on our website.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">9. Contact</h2>
				<p>
					If you have questions about this Cookie Policy or our use of browser storage technologies,
					please contact us at <a
						href="mailto:privacy@filez.zone"
						class="text-blue-500 hover:text-blue-600 underline">privacy@filez.zone</a
					>.
				</p>
			</section>
		</div>
	</div>

	<p class="text-center text-xs text-slate-400 mt-6">
		<a href="/" class="text-blue-500 hover:text-blue-600 transition-colors">← Back to filez.zone</a>
	</p>
</div>
