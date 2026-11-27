import { describe, expect, it } from 'vitest';
import { SITE_URL } from '#lib/constants/site.ts';
import { getFlatOrder, FALLBACK_LOCALE } from '#lib/content/docs.ts';
import { GET } from './+server';

// This endpoint only reads event.url, so a minimal fake RequestEvent is enough to exercise it
// directly. Typed off GET's own parameter so it matches the '/sitemap.xml' route literal.
const eventAt = (path: string) =>
	({ url: new URL(`${SITE_URL}${path}`) }) as unknown as Parameters<typeof GET>[0];

const parse = async (path: string) => {
	const response = await GET(eventAt(path));
	const body = await response.text();
	return { response, body, doc: new DOMParser().parseFromString(body, 'application/xml') };
};

describe('GET /sitemap.xml (index)', () => {
	it('lists one sitemap per locale', async () => {
		const { response, doc } = await parse('/sitemap.xml');

		expect(response.headers.get('Content-Type')).toBe('application/xml');
		expect(doc.querySelector('parsererror')).toBeNull();
		expect([...doc.querySelectorAll('sitemap > loc')].map((loc) => loc.textContent)).toEqual([
			`${SITE_URL}/pt-br/sitemap.xml`,
			`${SITE_URL}/en/sitemap.xml`
		]);
	});
});

describe('GET /<locale>/sitemap.xml', () => {
	it('returns well-formed XML with one <url> per page (home + every doc)', async () => {
		const { doc } = await parse('/en/sitemap.xml');

		expect(doc.querySelector('parsererror')).toBeNull();
		const docCount = getFlatOrder(FALLBACK_LOCALE).length;
		expect(doc.querySelectorAll('url').length).toBe(1 + docCount);
	});

	it('lists the pages under its own locale prefix', async () => {
		const { doc } = await parse('/en/sitemap.xml');

		for (const loc of doc.querySelectorAll('url > loc')) {
			expect(loc.textContent?.startsWith(`${SITE_URL}/en/`)).toBe(true);
		}
	});

	it('gives the home page top priority and every other page a lower one', async () => {
		const { doc } = await parse('/pt-br/sitemap.xml');

		const urls = [...doc.querySelectorAll('url')];
		const home = urls.find(
			(url) => url.querySelector('loc')?.textContent === `${SITE_URL}/pt-br/`
		);
		expect(home?.querySelector('priority')?.textContent).toBe('1.0');

		const others = urls.filter((url) => url !== home);
		expect(others.length).toBeGreaterThan(0);
		for (const url of others) {
			expect(url.querySelector('priority')?.textContent).toBe('0.8');
		}
	});

	it('gives every page an x-default alternate on the path with no prefix', async () => {
		const { doc } = await parse('/pt-br/sitemap.xml');

		for (const url of doc.querySelectorAll('url')) {
			const alternates = [...url.getElementsByTagName('xhtml:link')];
			const hrefs = alternates.map((link) => link.getAttribute('href') ?? '');
			const defaultHref = alternates
				.find((link) => link.getAttribute('hreflang') === 'x-default')
				?.getAttribute('href');

			expect(defaultHref?.startsWith(SITE_URL)).toBe(true);
			expect(defaultHref).not.toMatch(/\/(pt-br|en)(\/|$)/);
			expect(
				hrefs.filter((href) => href.includes('/en/') || href.endsWith('/en'))
			).toHaveLength(1);
		}
	});

	it('does not emit <lastmod> — no per-doc modification date is tracked, so a fabricated one would be misleading', async () => {
		const { body } = await parse('/en/sitemap.xml');

		expect(body).not.toContain('<lastmod>');
	});
});
