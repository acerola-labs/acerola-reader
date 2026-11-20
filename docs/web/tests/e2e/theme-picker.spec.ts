import { expect, test } from '@playwright/test';

// Mode persistence uses localStorage (MODE_STORAGE_KEY from
// src/lib/constants/themes.ts), not cookies. Default state on first load is
// mode "dark" (see use-theme.svelte.ts). The picker cycles
// mode dark -> system -> light -> dark on each click.
test.describe('theme picker', () => {
	test('persists the selected theme mode across reloads', async ({ page }) => {
		// TODO (worked around, not fixed): a doc page, not the landing (`/`) — the landing
		// renders `faulty-terminal.svelte` (animated WebGL/ogl background, see the TODO there)
		// which causes a "GPU stall" under headless Chromium's virtual GPU and crashes the page,
		// mainly on `page.reload()`. The theme picker doesn't depend on the landing to work,
		// so it's tested here — but that means the landing itself is never tested with
		// reloads/heavy interaction, only the smoke test in landing.spec.ts.
		await page.goto('/docs/architecture');

		const button = page.getByRole('button', { name: 'Mudar tema' });
		await button.click(); // dark -> system
		// The icon inside the button is recreated on every mode change (`{#key themeCtx.mode}`
		// in theme-picker.svelte) — clicking again before that re-render settles makes
		// Playwright wait indefinitely for the element to become stable.
		await expect
			.poll(() => page.evaluate(() => localStorage.getItem('acerola-docs:mode')))
			.toBe('system');
		await button.click(); // system -> light

		await expect
			.poll(() => page.evaluate(() => localStorage.getItem('acerola-docs:mode')))
			.toBe('light');
		await expect(page.locator('html')).not.toHaveClass(/dark/);
		await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');

		await page.reload();

		await expect
			.poll(() => page.evaluate(() => localStorage.getItem('acerola-docs:mode')))
			.toBe('light');
		await expect(page.locator('html')).not.toHaveClass(/dark/);
		await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
	});
});
