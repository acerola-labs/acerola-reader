import { expect, test } from '@playwright/test';

// Locale is controlled by paraglide's `url` strategy: the base locale ("pt-br")
// serves from an unprefixed path, and "en" is served from an "/en" prefix. As on the
// brand book topbar, the toggle in nav-controls.svelte shows the locale it switches TO.
test.describe('locale switch', () => {
	test('toggling locale changes the URL prefix and re-renders content', async ({ page }) => {
		await page.goto('/');
		const toEnglish = page.getByRole('button', { name: 'Mudar idioma' });
		await expect(toEnglish).toHaveText(/en/i);

		await toEnglish.click();

		await expect(page).toHaveURL(/\/en\/?$/);
		await expect(page.getByRole('button', { name: 'Switch language' })).toHaveText(/pt/i);
		await expect(
			page.getByText('Read your comics and manga on any device. No account, no cloud.')
		).toBeVisible();
	});
});
