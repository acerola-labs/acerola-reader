import { expect, test } from '@playwright/test';

// The sidebar's "flattened" order (src/lib/content/docs.ts) follows SECTION_ORDER and then
// `order` within each section — today that puts "architecture" (Conceitos) between
// "contributing-p2p" (Contribuindo) and "privacy-policy" (Privacidade). If a new doc is
// inserted between those sections, or SECTION_ORDER changes, the neighbors expected here
// change too — it's neither alphabetical nor based on the file name.
test.describe('prev/next doc navigation', () => {
	test('prev/next links navigate between adjacent docs', async ({ page }) => {
		await page.goto('/docs/architecture');
		await expect(page.getByRole('heading', { level: 1, name: 'Arquitetura' })).toBeVisible();

		// The Previous/Next navigation appears twice (above and below the
		// content) — `.first()` takes the top one, but both lead to the same place.
		await page
			.getByRole('link', { name: /Próximo/ })
			.first()
			.click();

		await expect(page).toHaveURL(/\/docs\/privacy-policy$/);
		await expect(
			page.getByRole('heading', { level: 1, name: 'Política de Privacidade' })
		).toBeVisible();

		await page
			.getByRole('link', { name: /Anterior/ })
			.first()
			.click();

		await expect(page).toHaveURL(/\/docs\/architecture$/);
		await expect(page.getByRole('heading', { level: 1, name: 'Arquitetura' })).toBeVisible();
	});
});
