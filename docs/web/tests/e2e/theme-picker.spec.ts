import { expect, test } from '@playwright/test';

// The theme is shared by every Acerola site through the acerola-theme cookie (acerola-design
// skill, 11.1): dark without a cookie, and the picker switches dark <-> light.
test.describe('theme picker', () => {
	test('persists the selected theme across reloads', async ({ page }) => {
		// TODO (worked around, not fixed): a doc page, not the landing (`/`) — the landing
		// renders `faulty-terminal.svelte` (animated WebGL/ogl background, see the TODO there)
		// which causes a "GPU stall" under headless Chromium's virtual GPU and crashes the page,
		// mainly on `page.reload()`. The theme picker doesn't depend on the landing to work,
		// so it's tested here — but that means the landing itself is never tested with
		// reloads/heavy interaction, only the smoke test in landing.spec.ts.
		await page.goto('/pt-br/docs/architecture');
		await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');

		await page.getByRole('button', { name: 'Mudar tema' }).click(); // dark -> light

		await expect(page.locator('html')).not.toHaveClass(/dark/);
		await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
		const cookies = await page.context().cookies();
		expect(cookies.find((cookie) => cookie.name === 'acerola-theme')?.value).toBe('light');

		await page.reload();

		await expect(page.locator('html')).not.toHaveClass(/dark/);
		await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
	});
});
