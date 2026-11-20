import { SITE_URL } from '#lib/constants/site.ts';
import { FALLBACK_LOCALE, getFlatOrder } from '#lib/content/docs.ts';
import { locales, localizeHref } from '#lib/paraglide/runtime.js';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async () => {
	const paths = ['/', ...getFlatOrder(FALLBACK_LOCALE).map((doc) => `/docs/${doc.slug}`)];

	const urls = paths
		.map((path) => {
			// x-default points to the fallback (pt-br) — same rule as <link rel="alternate"> in
			// +layout.svelte, for crawlers that recognize none of the listed hreflangs.
			const alternates = [
				...locales.map(
					(locale) => `
        <xhtml:link rel="alternate" hreflang="${locale}" href="${SITE_URL}${localizeHref(path, { locale })}" />`
				),
				`
        <xhtml:link rel="alternate" hreflang="x-default" href="${SITE_URL}${localizeHref(path, { locale: FALLBACK_LOCALE })}" />`
			].join('');

			// No <lastmod>: there's no way to know each doc's real modification date here, and
			// sending "today" for every URL on every request is worse than omitting it — Google
			// treats a lastmod that never matches the real change as an untrustworthy signal and
			// starts ignoring it for the whole site.
			return `
    <url>
        <loc>${SITE_URL}${localizeHref(path, { locale: FALLBACK_LOCALE })}</loc>
        <changefreq>weekly</changefreq>
        <priority>${path === '/' ? '1.0' : '0.8'}</priority>${alternates}
    </url>`;
		})
		.join('');

	const sitemap = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
        xmlns:xhtml="http://www.w3.org/1999/xhtml">${urls}
</urlset>`;

	return new Response(sitemap, {
		headers: {
			'Content-Type': 'application/xml',
			'Cache-Control': 'max-age=0, s-maxage=3600'
		}
	});
};
