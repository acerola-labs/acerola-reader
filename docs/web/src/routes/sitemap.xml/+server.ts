import { SITE_URL } from '#lib/constants/site.ts';
import { FALLBACK_LOCALE, getFlatOrder } from '#lib/content/docs.ts';
import { extractLocaleFromUrl, locales, localizeHref } from '#lib/paraglide/runtime.js';
import type { RequestHandler } from './$types';

export const prerender = true;

const xml = (body: string) =>
	new Response(`<?xml version="1.0" encoding="UTF-8"?>\n${body}`, {
		headers: {
			'Content-Type': 'application/xml',
			'Cache-Control': 'max-age=0, s-maxage=3600'
		}
	});

/**
 * /sitemap.xml is the index of the per-locale sitemaps (/pt-br/sitemap.xml, /en/sitemap.xml);
 * each one lists the pages in its locale, with hreflang links to the other locales and
 * x-default on the path with no prefix (it follows the visitor's acerola-locale cookie).
 */
export const GET: RequestHandler = ({ url }) => {
	const locale = extractLocaleFromUrl(url);

	if (!locale) {
		const maps = locales
			.map(
				(l) =>
					`    <sitemap><loc>${SITE_URL}${localizeHref('/sitemap.xml', { locale: l })}</loc></sitemap>`
			)
			.join('\n');
		return xml(
			`<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${maps}\n</sitemapindex>\n`
		);
	}

	const paths = ['/', ...getFlatOrder(FALLBACK_LOCALE).map((doc) => `/docs/${doc.slug}`)];

	const urls = paths
		.map((path) => {
			const alternates = [
				...locales.map(
					(l) => `
        <xhtml:link rel="alternate" hreflang="${l}" href="${SITE_URL}${localizeHref(path, { locale: l })}" />`
				),
				`
        <xhtml:link rel="alternate" hreflang="x-default" href="${SITE_URL}${path}" />`
			].join('');

			// No <lastmod>: there's no way to know each doc's real modification date here, and
			// sending "today" for every URL on every request is worse than omitting it — Google
			// treats a lastmod that never matches the real change as an untrustworthy signal and
			// starts ignoring it for the whole site.
			return `
    <url>
        <loc>${SITE_URL}${localizeHref(path, { locale })}</loc>
        <changefreq>weekly</changefreq>
        <priority>${path === '/' ? '1.0' : '0.8'}</priority>${alternates}
    </url>`;
		})
		.join('');

	return xml(`<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
        xmlns:xhtml="http://www.w3.org/1999/xhtml">${urls}
</urlset>
`);
};
