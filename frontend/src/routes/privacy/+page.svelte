<script lang="ts">
	const lastUpdated = new Date().toLocaleDateString('en-US', {
		month: 'long',
		day: 'numeric',
		year: 'numeric'
	});
</script>

<svelte:head>
	<title>Privacy Policy — filez.zone</title>
	<meta
		name="description"
		content="Privacy Policy for filez.zone. Learn how we handle your data, including end-to-end encryption, burn-after-reading file deletion, optional user accounts, saved URLs, and our use of cookieless, privacy-first analytics."
	/>
	<meta name="robots" content="index, follow" />
	<meta property="og:title" content="Privacy Policy — filez.zone" />
	<meta
		property="og:description"
		content="Privacy Policy for filez.zone. End-to-end encrypted file sharing with cookieless, privacy-first analytics, optional accounts, and automatic file deletion after download."
	/>
	<meta property="og:url" content="https://filez.zone/privacy" />
	<meta property="og:type" content="website" />
	<meta name="twitter:card" content="summary" />
	<meta name="twitter:title" content="Privacy Policy — filez.zone" />
	<meta
		name="twitter:description"
		content="Privacy Policy for filez.zone. Learn how we protect your data with end-to-end encryption, optional accounts, cookieless analytics, and automatic deletion."
	/>
	<link rel="canonical" href="https://filez.zone/privacy" />
</svelte:head>

<div class="w-full max-w-2xl">
	<div
		class="bg-white rounded-2xl shadow-lg shadow-slate-200/50 border border-slate-100 p-8 sm:p-10"
	>
		<h1 class="text-2xl font-semibold text-slate-800 mb-1">Privacy Policy</h1>
		<p class="text-sm text-slate-400 mb-8">Last updated: {lastUpdated}</p>

		<div class="prose prose-slate prose-sm max-w-none space-y-6 text-slate-600 leading-relaxed">
			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">1. Overview</h2>
				<p>
					filez.zone ("we", "us", or "our") operates the file-sharing service available at <a
						href="/"
						class="text-blue-500 hover:text-blue-600 underline">filez.zone</a
					>. This Privacy Policy explains how we collect, use, and protect information when you use
					our Service.
				</p>
				<p class="mt-2">
					<strong>Core principle:</strong> We designed filez.zone so that we never have access to the
					content of your files. All encryption and decryption happens in your browser. The encryption
					key is never transmitted to our servers.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">2. Information We Collect</h2>

				<h3 class="text-sm font-semibold text-slate-700 mt-4 mb-1">
					Information you provide (optional user accounts)
				</h3>
				<p>
					filez.zone offers <strong>optional user accounts</strong> that enable the ability to save and
					manage shared capability URLs. Creating an account is entirely voluntary — the core file-sharing
					functionality works without one.
				</p>
				<p class="mt-1">When you create an account, you provide:</p>
				<ul class="list-disc pl-5 space-y-1">
					<li>
						<strong>Email address:</strong> Used to identify your account and for authentication.
						Your email is stored in our database and included as the
						<code class="bg-slate-100 px-1 py-0.5 rounded text-xs">sub</code> claim in your JWT token.
					</li>
					<li>
						<strong>Display name:</strong> Used to personalize your experience in the user interface.
					</li>
					<li>
						<strong>Password:</strong> Hashed using bcrypt with a unique per-password salt. The plaintext
						password is never stored — only the cryptographic hash is persisted in our database.
					</li>
				</ul>

				<h3 class="text-sm font-semibold text-slate-700 mt-4 mb-1">
					Information you provide (file data)
				</h3>
				<ul class="list-disc pl-5 space-y-1">
					<li>
						<strong>File data (encrypted only):</strong> When you upload a file, only the encrypted ciphertext
						is stored on our servers. We cannot read, view, or decrypt your files.
					</li>
					<li>
						<strong>Content type:</strong> You may optionally provide a MIME type (e.g., "image/png")
						which is stored as metadata alongside the encrypted file to help recipients identify the file
						type.
					</li>
				</ul>

				<h3 class="text-sm font-semibold text-slate-700 mt-4 mb-1">Saved URLs</h3>
				<p>
					When you are authenticated and upload a file, the resulting capability URL is
					automatically saved to your account. The following information is stored:
				</p>
				<ul class="list-disc pl-5 space-y-1">
					<li>
						<strong>Capability URL:</strong> The full shareable link (including the encrypted key in the
						hash fragment).
					</li>
					<li>
						<strong>Title:</strong> The original filename from the upload, used as a human-readable label
						for the saved link. You may also provide a custom title.
					</li>
					<li>
						<strong>Record metadata:</strong> A unique ID (UUID), your account email (as the
						<code class="bg-slate-100 px-1 py-0.5 rounded text-xs">user_email</code> field), and the timestamp
						when the URL was saved.
					</li>
				</ul>
				<p class="mt-1">
					Saved URL records are stored in our MongoDB database and are only accessible to you
					through authenticated API requests using a valid JWT token in the
					<code class="bg-slate-100 px-1 py-0.5 rounded text-xs"
						>Authorization: Bearer &lt;token&gt;</code
					>
					header. They are permanently deleted when you delete your account.
				</p>

				<h3 class="text-sm font-semibold text-slate-700 mt-4 mb-1">
					Information collected automatically
				</h3>
				<ul class="list-disc pl-5 space-y-1">
					<li>
						<strong>Server logs:</strong> We may log HTTP request metadata (IP address, timestamp, user
						agent) for operational purposes such as debugging, rate limiting, and abuse prevention. These
						logs are retained temporarily and are not used to build user profiles.
					</li>
					<li>
						<strong>Anonymous analytics:</strong> We use OpenPanel, a cookieless, privacy-first analytics
						tool, to collect anonymous page view data. This includes page URLs, referrer information,
						browser and device type, and country-level location derived from anonymized IP addresses.
						No cookies are used, no personal identifiers are collected, and users cannot be tracked across
						sessions or websites.
					</li>
				</ul>

				<h3 class="text-sm font-semibold text-slate-700 mt-4 mb-1">
					Information we do NOT collect
				</h3>
				<ul class="list-disc pl-5 space-y-1">
					<li>
						Encryption keys — these remain in the URL hash fragment and are never sent to our
						servers.
					</li>
					<li>
						Plaintext file content — all file data is encrypted client-side before it ever reaches
						our infrastructure.
					</li>
					<li>
						File names from anonymous uploads — when uploading without an account, the original file
						name is not stored or transmitted to the backend.
					</li>
				</ul>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">
					3. Authentication &amp; Session Management
				</h2>
				<p>
					We use <strong>JSON Web Tokens (JWT)</strong> for authenticating protected API requests.
					When you log in, the backend issues a signed JWT containing your email (as the
					<code class="bg-slate-100 px-1 py-0.5 rounded text-xs">sub</code> claim), an issuance
					timestamp, and an expiration timestamp. The token is signed with a server-side secret (<code
						class="bg-slate-100 px-1 py-0.5 rounded text-xs">JWT_SECRET</code
					>) known only to our backend.
				</p>
				<p class="mt-2">
					The JWT is stored in your browser's <strong>localStorage</strong> and sent to the backend
					via the
					<code class="bg-slate-100 px-1 py-0.5 rounded text-xs"
						>Authorization: Bearer &lt;token&gt;</code
					>
					HTTP header for all protected endpoints (e.g., saving and listing URLs). The token is never
					embedded in URLs or logged by the server. It is automatically invalidated upon expiry (configurable
					via <code class="bg-slate-100 px-1 py-0.5 rounded text-xs">JWT_EXPIRY_MINS</code>, default
					5 minutes).
				</p>
				<p class="mt-2">
					You can log out at any time via the user menu, which clears the JWT and user information
					from localStorage. Deleting your account removes all associated data from our database,
					including saved URLs and your user record.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">4. How Files Are Protected</h2>
				<p>
					Your files are protected by end-to-end encryption using AES-256-GCM, implemented through
					the browser's native Web Crypto API. The encryption process works as follows:
				</p>
				<ol class="list-decimal pl-5 mt-2 space-y-1">
					<li>A unique 256-bit AES key is generated in your browser for each file.</li>
					<li>
						The file is split into chunks and each chunk is encrypted with a random 12-byte
						initialization vector (IV).
					</li>
					<li>
						The encrypted ciphertext is uploaded directly to Cloudflare R2 storage via presigned
						URLs — it never passes through our backend servers in plaintext.
					</li>
					<li>
						The encryption key is encoded into the shareable link's hash fragment (<code
							class="bg-slate-100 px-1 py-0.5 rounded text-xs">#key</code
						>), which browsers never transmit in HTTP requests.
					</li>
				</ol>
				<p class="mt-2">
					This means that even if our servers or storage were compromised, an attacker would not be
					able to decrypt your files without the key from the URL fragment.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">
					5. Data Retention &amp; Deletion
				</h2>
				<ul class="list-disc pl-5 space-y-2">
					<li>
						<strong>Burn after reading:</strong> Files are automatically and permanently deleted from
						our storage immediately after the first successful download. We do not retain any copies.
					</li>
					<li>
						<strong>Orphaned uploads:</strong> If a file is uploaded but never downloaded, it may remain
						on our servers until purged by routine maintenance. We do not guarantee a specific retention
						period for undownloaded files.
					</li>
					<li>
						<strong>User accounts:</strong> Your account information (email, display name, hashed password)
						and saved URL records are retained until you choose to delete your account. Account deletion
						is available from the user menu and removes all associated data permanently.
					</li>
					<li>
						<strong>Server logs:</strong> Operational logs are retained for a limited period and then
						automatically deleted. We do not use these logs for advertising or tracking purposes.
					</li>
				</ul>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">6. Third-Party Services</h2>
				<p>
					We use <strong>Cloudflare R2</strong> for object storage. Encrypted file data is stored on
					Cloudflare's infrastructure. Cloudflare operates independently under its own privacy
					policy (<a
						href="https://www.cloudflare.com/privacypolicy/"
						class="text-blue-500 hover:text-blue-600 underline"
						target="_blank"
						rel="noopener">cloudflare.com/privacypolicy</a
					>). Cloudflare does not have access to your encryption keys or the ability to decrypt your
					files.
				</p>
				<p class="mt-2">
					We use <strong>MongoDB</strong> for persistent storage of user accounts and saved URL records.
					MongoDB is hosted and operated by us. Your account data (email, hashed password, saved URLs)
					is stored exclusively in this database and is never shared with third parties.
				</p>
				<p class="mt-2">
					We use <strong>OpenPanel</strong>, a self-hosted, cookieless analytics tool, to understand
					how our Service is used. OpenPanel collects anonymous page view data without cookies or
					personal identifiers. It is deployed on our own infrastructure and data is not shared with
					any external analytics provider. For more details, see the
					<a
						href="https://openpanel.dev"
						class="text-blue-500 hover:text-blue-600 underline"
						target="_blank"
						rel="noopener">OpenPanel documentation</a
					>.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">7. Cookies &amp; Tracking</h2>
				<p>
					filez.zone does not use cookies for tracking, advertising, or analytics. Our analytics
					tool (OpenPanel) operates entirely without cookies — it does not set, read, or require any
					cookies in your browser. It uses server-side request analysis to count page views
					anonymously, without storing any identifiers on your device.
				</p>
				<p class="mt-2">
					For authenticated users, we use <strong>localStorage</strong> (not cookies) to persist
					your JWT token and basic user information (email, display name) across browser sessions.
					This enables you to remain logged in without re-authenticating on every page load. You can
					clear this data at any time by logging out. For details about the minimal browser storage
					we use for the Service itself, see our
					<a href="/cookies" class="text-blue-500 hover:text-blue-600 underline">Cookie Policy</a>.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">8. Security</h2>
				<p>
					We take reasonable measures to protect the security of our infrastructure and the data
					transmitted through our Service, including:
				</p>
				<ul class="list-disc pl-5 mt-2 space-y-1">
					<li>Enforcing HTTPS for all connections.</li>
					<li>
						Using presigned URLs with short lifetimes for direct uploads, so file data does not pass
						through our backend.
					</li>
					<li>
						Employing end-to-end encryption so that even a server compromise cannot reveal file
						contents.
					</li>
					<li>
						Using bcrypt with unique per-password salts to hash account passwords — plaintext
						passwords are never stored.
					</li>
					<li>
						Validating JWT tokens in middleware before protected endpoint handlers execute,
						rejecting unauthorized requests with <code
							class="bg-slate-100 px-1 py-0.5 rounded text-xs">401 Unauthorized</code
						>.
					</li>
				</ul>
				<p class="mt-2">
					No system is completely secure. We encourage you to share capability links only through
					trusted channels and to be aware that anyone with the link can download the file exactly
					once.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">9. International Users</h2>
				<p>
					Our Service is operated from and data is processed in the European Union. If you access
					the Service from outside the EU, you acknowledge that your data will be processed in the
					EU and that you are providing consent for such processing.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">10. Your Rights</h2>
				<p>Depending on your jurisdiction, you may have the right to:</p>
				<ul class="list-disc pl-5 mt-2 space-y-1">
					<li>Access the personal data we hold about you.</li>
					<li>Request deletion of your data.</li>
					<li>Object to or restrict processing of your data.</li>
					<li>Lodge a complaint with a supervisory authority.</li>
				</ul>
				<p class="mt-2">
					Because our Service is designed to minimize data collection and automatically deletes
					files after download, most of these rights are fulfilled by design. If you have created an
					account, you can delete it at any time via the user menu, which permanently removes all
					associated data (email, display name, hashed password, saved URL records). For any
					inquiries, contact us at <a
						href="mailto:privacy@filez.zone"
						class="text-blue-500 hover:text-blue-600 underline">privacy@filez.zone</a
					>.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">11. Changes to This Policy</h2>
				<p>
					We may update this Privacy Policy from time to time. Changes will be effective immediately
					upon posting. The "Last updated" date at the top of this page indicates when the policy
					was last revised. We encourage you to review this page periodically.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">12. Contact</h2>
				<p>
					For privacy-related questions or concerns, please contact us at <a
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
