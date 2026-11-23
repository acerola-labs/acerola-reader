import { expect, test } from '@playwright/test';

// This spec needs the production build + Pagefind index to exist on disk (see
// playwright.config.ts) — `npm run build` must run before `playwright test`.
test.describe('search dialog', () => {
	test('opens with Control+K and returns real Pagefind results', async ({ page }) => {
		await page.goto('/');

		await page.keyboard.press('Control+k');
		const dialog = page.getByRole('dialog');
		// The home page runs the WebGL background (faulty-terminal.svelte) since the goto above;
		// under headless Chromium's virtual GPU in CI, with variable shared-runner load, this
		// can exceed 10s before the dialog opens — generous margin to avoid flakiness.
		await expect(dialog).toBeVisible({ timeout: 20_000 });

		await dialog.getByPlaceholder('Digite para pesquisar...').fill('monorepo');

		// The first query loads the Pagefind WASM/index on demand — slower than the default
		// 5s timeout, so this specific assertion gets more margin.
		const result = dialog.locator('a[href*="/docs/architecture"]');
		await expect(result).toBeVisible({ timeout: 20_000 });
		await expect(result).toContainText('Arquitetura');

		// Pagefind indexes the static build and returns the real file URL
		// (e.g. "/docs/architecture.html"), but SvelteKit routes are served without an
		// extension — an href with ".html" here is a 404 in production.
		await expect(result).not.toHaveAttribute('href', /\.html$/);
	});
});
