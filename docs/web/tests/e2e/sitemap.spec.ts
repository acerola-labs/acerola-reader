import { expect, test } from '@playwright/test';

test.describe('sitemap', () => {
	test('sitemap.xml is an index of one sitemap per locale', async ({ page }) => {
		const response = await page.request.get('/sitemap.xml');
		expect(response.status()).toBe(200);
		expect(response.headers()['content-type']).toContain('xml');

		const body = await response.text();
		const sitemapCount = await page.evaluate((xml) => {
			const doc = new DOMParser().parseFromString(xml, 'application/xml');
			if (doc.querySelector('parsererror')) throw new Error('sitemap.xml is not valid XML');
			return doc.querySelectorAll('sitemap').length;
		}, body);

		expect(sitemapCount).toBe(2);
	});

	for (const locale of ['pt-br', 'en']) {
		test(`/${locale}/sitemap.xml is a valid, non-empty sitemap`, async ({ page }) => {
			const response = await page.request.get(`/${locale}/sitemap.xml`);
			expect(response.status()).toBe(200);

			const body = await response.text();
			const urlCount = await page.evaluate((xml) => {
				const doc = new DOMParser().parseFromString(xml, 'application/xml');
				if (doc.querySelector('parsererror')) throw new Error('sitemap is not valid XML');
				return doc.querySelectorAll('url').length;
			}, body);

			expect(urlCount).toBeGreaterThan(0);
		});
	}
});
