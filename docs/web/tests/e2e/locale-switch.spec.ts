import { expect, test } from '@playwright/test';

// Locale rule of every Acerola site (acerola-project skill, 2.1): both locales live under a
// prefix (/pt-br, /en); a prefixed path saves its locale in the acerola-locale cookie and a path
// with no prefix goes to the cookie's locale (pt-br without one). As on the brand book topbar,
// the toggle in nav-controls.svelte shows the locale it switches TO.
test.describe('locale switch', () => {
	test('a path with no prefix opens in pt-br when there is no cookie', async ({ page }) => {
		await page.goto('/docs/architecture');

		await expect(page).toHaveURL(/\/pt-br\/docs\/architecture$/);
	});

	test('toggling locale changes the URL prefix and re-renders content', async ({ page }) => {
		await page.goto('/pt-br');
		const toEnglish = page.getByRole('button', { name: 'Mudar idioma' });
		await expect(toEnglish).toHaveText(/en/i);

		await toEnglish.click();

		await expect(page).toHaveURL(/\/en\/?$/);
		await expect(page.getByRole('button', { name: 'Switch language' })).toHaveText(/pt/i);
		await expect(
			page.getByText('Read your comics and manga on any device. No account, no cloud.')
		).toBeVisible();
	});

	test('opening a locale saves it: the next path with no prefix follows', async ({ page }) => {
		await page.goto('/en/docs/architecture');
		await page.goto('/docs/getting-started');

		await expect(page).toHaveURL(/\/en\/docs\/getting-started$/);
		const cookies = await page.context().cookies();
		expect(cookies.find((cookie) => cookie.name === 'acerola-locale')?.value).toBe('en');
	});
});
