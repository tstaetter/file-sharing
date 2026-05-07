<script lang="ts">
	const lastUpdated = new Date().toLocaleDateString('en-US', {
		month: 'long',
		day: 'numeric',
		year: 'numeric'
	});
</script>

<svelte:head>
	<title>Terms of Service — sha.zone</title>
</svelte:head>

<div class="w-full max-w-2xl">
	<div
		class="bg-white rounded-2xl shadow-lg shadow-slate-200/50 border border-slate-100 p-8 sm:p-10"
	>
		<h1 class="text-2xl font-semibold text-slate-800 mb-1">Terms of Service</h1>
		<p class="text-sm text-slate-400 mb-8">Last updated: {lastUpdated}</p>

		<div class="prose prose-slate prose-sm max-w-none space-y-6 text-slate-600 leading-relaxed">
			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">1. Acceptance of Terms</h2>
				<p>
					By accessing or using the sha.zone file-sharing service ("Service"), you agree to be bound
					by these Terms of Service ("Terms"). If you do not agree to all of these Terms, do not use
					the Service.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">2. Description of Service</h2>
				<p>
					sha.zone provides an end-to-end encrypted file-sharing service. Files are encrypted in
					your browser before upload and can only be decrypted by recipients who possess the unique
					link containing the encryption key. Files are automatically deleted from our servers after
					a single download ("burn after reading").
				</p>
				<p class="mt-2">
					<strong>Important:</strong> We never have access to your encryption keys or the plaintext content
					of your files. The key is embedded in the URL hash fragment, which is never transmitted to our
					servers.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">3. User Responsibilities</h2>
				<p>You are solely responsible for:</p>
				<ul class="list-disc pl-5 mt-2 space-y-1">
					<li>The content of files you upload through the Service.</li>
					<li>Ensuring that you have the right to share any files you upload.</li>
					<li>Maintaining the security of any capability links you generate.</li>
					<li>Any consequences of sharing decryption links with third parties.</li>
				</ul>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">4. Acceptable Use</h2>
				<p>You agree not to use the Service to:</p>
				<ul class="list-disc pl-5 mt-2 space-y-1">
					<li>
						Share content that is illegal, harmful, threatening, abusive, or violates any applicable
						law.
					</li>
					<li>Share content that infringes on the intellectual property rights of others.</li>
					<li>Distribute malware, viruses, or any other harmful code.</li>
					<li>Attempt to compromise the security or integrity of the Service.</li>
					<li>
						Use the Service in any manner that could damage, disable, or overburden our
						infrastructure.
					</li>
				</ul>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">
					5. Data Retention &amp; Deletion
				</h2>
				<p>
					Files uploaded through the Service are <strong
						>automatically and permanently deleted</strong
					> from our storage infrastructure immediately after the first successful download. We do not
					retain copies of uploaded files, encryption keys, or decryption links.
				</p>
				<p class="mt-2">
					If a file is never downloaded, it may remain on our servers until it is purged by routine
					maintenance. We make no guarantees about the duration an undownloaded file will be
					retained.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">6. Privacy &amp; Encryption</h2>
				<p>
					All file encryption and decryption occurs exclusively in your browser using AES-256-GCM
					via the Web Crypto API. The encryption key is never transmitted to our servers — it is
					conveyed to recipients only through the URL hash fragment, which browsers do not send in
					HTTP requests.
				</p>
				<p class="mt-2">
					We cannot read, modify, or decrypt your files. We cannot recover lost capability links.
					For more details, see our <a
						href="/privacy"
						class="text-blue-500 hover:text-blue-600 underline">Privacy Policy</a
					>.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">7. Service Availability</h2>
				<p>
					The Service is provided "as is" and "as available." We do not guarantee uninterrupted or
					error-free operation. We reserve the right to modify, suspend, or discontinue the Service
					(or any part thereof) at any time without prior notice.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">8. Limitation of Liability</h2>
				<p>To the maximum extent permitted by law:</p>
				<ul class="list-disc pl-5 mt-2 space-y-1">
					<li>The Service is provided without warranties of any kind, express or implied.</li>
					<li>
						We are not liable for any loss of data, including files that fail to upload, download,
						or decrypt correctly.
					</li>
					<li>
						We are not liable for any direct, indirect, incidental, or consequential damages arising
						from the use or inability to use the Service.
					</li>
					<li>We are not responsible for the content shared by users through the Service.</li>
				</ul>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">9. Changes to Terms</h2>
				<p>
					We may update these Terms from time to time. Changes will be effective immediately upon
					posting the updated Terms on this page. Your continued use of the Service after any
					changes constitutes your acceptance of the revised Terms.
				</p>
			</section>

			<section>
				<h2 class="text-base font-semibold text-slate-800 mb-2">10. Contact</h2>
				<p>
					If you have questions about these Terms, please reach out to us at <a
						href="mailto:legal@sha.zone"
						class="text-blue-500 hover:text-blue-600 underline">legal@sha.zone</a
					>.
				</p>
			</section>
		</div>
	</div>

	<p class="text-center text-xs text-slate-400 mt-6">
		<a href="/" class="text-blue-500 hover:text-blue-600 transition-colors">← Back to sha.zone</a>
	</p>
</div>
