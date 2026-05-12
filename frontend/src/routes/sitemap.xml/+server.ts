import type { RequestHandler } from './$types';

const BASE_URL = 'https://www.filez.zone';

const staticPages: { path: string; changefreq: string; priority: string }[] = [
	{ path: '/', changefreq: 'weekly', priority: '1.0' },
	{ path: '/tos', changefreq: 'monthly', priority: '0.3' },
	{ path: '/privacy', changefreq: 'monthly', priority: '0.3' },
	{ path: '/cookies', changefreq: 'monthly', priority: '0.3' },
	{ path: '/zero-knowledge', changefreq: 'monthly', priority: '0.3' }
];

function escapeXml(str: string): string {
	return str
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;')
		.replace(/'/g, '&apos;');
}

export const GET: RequestHandler = async () => {
	const today = new Date().toISOString().split('T')[0];

	const urlEntries = staticPages
		.map(
			({ path, changefreq, priority }) => `
  <url>
    <loc>${escapeXml(`${BASE_URL}${path}`)}</loc>
    <lastmod>${today}</lastmod>
    <changefreq>${changefreq}</changefreq>
    <priority>${priority}</priority>
  </url>`
		)
		.join('');

	const xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">${urlEntries}
</urlset>`;

	return new Response(xml.trim(), {
		headers: {
			'Content-Type': 'application/xml',
			'Cache-Control': 'public, max-age=3600'
		}
	});
};
