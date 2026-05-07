<script lang="ts">
	const lastUpdated = new Date().toLocaleDateString('en-US', {
		month: 'long',
		day: 'numeric',
		year: 'numeric'
	});
</script>

<svelte:head>
	<title>Privacy Policy — sha.zone</title>
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
					sha.zone ("we", "us", or "our") operates the file-sharing service available at <a
						href="/"
						class="text-blue-500 hover:text-blue-600 underline">sha.zone</a
					>. This Privacy Policy explains how we collect, use, and protect information when you use
					our Service.
				</p>
				<p class="mt-2">
					<strong>Core principle:</strong> We designed sha.zone so that we never have access to the content
					of your files. All encryption and decryption happens in your browser. The encryption key is
					never transmitted to our servers.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">2. Information We Collect</h2>

				<h3 class="text-sm font-semibold text-slate-700 mt-4 mb-1">Information you provide</h3>
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

				<h3 class="text-sm font-semibold text-slate-700 mt-4 mb-1">
					Information collected automatically
				</h3>
				<ul class="list-disc pl-5 space-y-1">
					<li>
						<strong>Server logs:</strong> We may log HTTP request metadata (IP address, timestamp, user
						agent) for operational purposes such as debugging, rate limiting, and abuse prevention. These
						logs are retained temporarily and are not used to build user profiles.
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
					<li>File names — the original file name is not stored or transmitted.</li>
					<li>
						Personal identification — we do not require accounts, email addresses, or any personal
						information to use the Service.
					</li>
				</ul>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">3. How Files Are Protected</h2>
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
					4. Data Retention &amp; Deletion
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
						<strong>Server logs:</strong> Operational logs are retained for a limited period and then
						automatically deleted. We do not use these logs for advertising or tracking purposes.
					</li>
				</ul>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">5. Third-Party Services</h2>
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
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">6. Cookies</h2>
				<p>
					sha.zone does not use cookies for tracking, advertising, or analytics. For details about
					the minimal browser storage we use, see our <a
						href="/cookies"
						class="text-blue-500 hover:text-blue-600 underline">Cookie Policy</a
					>.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">7. Security</h2>
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
				</ul>
				<p class="mt-2">
					No system is completely secure. We encourage you to share capability links only through
					trusted channels and to be aware that anyone with the link can download the file exactly
					once.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">8. International Users</h2>
				<p>
					Our Service is operated from and data is processed in the European Union. If you access
					the Service from outside the EU, you acknowledge that your data will be processed in the
					EU and that you are providing consent for such processing.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">9. Your Rights</h2>
				<p>Depending on your jurisdiction, you may have the right to:</p>
				<ul class="list-disc pl-5 mt-2 space-y-1">
					<li>Access the personal data we hold about you.</li>
					<li>Request deletion of your data.</li>
					<li>Object to or restrict processing of your data.</li>
					<li>Lodge a complaint with a supervisory authority.</li>
				</ul>
				<p class="mt-2">
					Because our Service is designed to minimize data collection and automatically deletes
					files after download, most of these rights are fulfilled by design. For any inquiries,
					contact us at <a
						href="mailto:privacy@sha.zone"
						class="text-blue-500 hover:text-blue-600 underline">privacy@sha.zone</a
					>.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">10. Changes to This Policy</h2>
				<p>
					We may update this Privacy Policy from time to time. Changes will be effective immediately
					upon posting. The "Last updated" date at the top of this page indicates when the policy
					was last revised. We encourage you to review this page periodically.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">11. Contact</h2>
				<p>
					For privacy-related questions or concerns, please contact us at <a
						href="mailto:privacy@sha.zone"
						class="text-blue-500 hover:text-blue-600 underline">privacy@sha.zone</a
					>.
				</p>
			</section>
		</div>
	</div>

	<p class="text-center text-xs text-slate-400 mt-6">
		<a href="/" class="text-blue-500 hover:text-blue-600 transition-colors">← Back to sha.zone</a>
	</p>
</div>
