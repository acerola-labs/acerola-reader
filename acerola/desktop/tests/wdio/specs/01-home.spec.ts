import {
	expandDock,
	firstDisplayed,
	getPathname,
	isTextDisplayed,
	navigateTo,
	waitForAppReady,
	waitForText
} from '../helpers/app';

describe('native home', () => {
	it('renders navigation, search and library state without freezing', async () => {
		await waitForAppReady();
		await navigateTo('/home');

		await waitForText('Acerola');

		// The dock only exposes the links once expanded (hover mode) — the
		// "Início"/"Histórico"/"Configurações" labels only exist as
		// aria-label, not as visible text (see acerola-dock.svelte).
		await expandDock();
		expect(await (await browser.$('[aria-label="Início"]')).isExisting()).toBe(true);
		expect(await (await browser.$('[aria-label="Histórico"]')).isExisting()).toBe(true);
		expect(await (await browser.$('[aria-label="Configurações"]')).isExisting()).toBe(true);

		// Open the search dialog
		const searchButton = await firstDisplayed('button[aria-label="Buscar quadrinho..."]');
		await searchButton.click();

		// Check that the search dialog opened
		await browser.waitUntil(
			async () => {
				const searchDialog = await browser.$('[role="dialog"]');
				return searchDialog.isDisplayed();
			},
			{
				timeout: 5_000,
				interval: 100,
				timeoutMsg: 'Search dialog did not open.'
			}
		);

		expect(await getPathname()).toBe('/home');

		await browser.waitUntil(
			async () => {
				const hasEmptyState = await isTextDisplayed('Nenhum quadrinho encontrado.');
				const comicCards = await browser.$$('main h3');
				const hasComicCard = (await comicCards.length) > 0;
				const hasLoading = await isTextDisplayed('Carregando...');

				return hasEmptyState || hasComicCard || hasLoading;
			},
			{
				timeout: 10_000,
				interval: 100,
				timeoutMsg: 'Home did not show loading, empty state or cards.'
			}
		);
	});

	it('navigates between Home and Settings via sidebar', async () => {
		await waitForAppReady();
		await navigateTo('/home');

		await expandDock();
		await (await firstDisplayed('a[href="/config"]')).click();
		await browser.waitUntil(async () => (await getPathname()) === '/config', {
			timeout: 10_000,
			interval: 100,
			timeoutMsg: 'Sidebar did not navigate to Settings.'
		});
		await waitForText('Configurações');

		await expandDock();
		await (await firstDisplayed('a[href="/home"]')).click();
		await browser.waitUntil(async () => (await getPathname()) === '/home', {
			timeout: 10_000,
			interval: 100,
			timeoutMsg: 'Sidebar did not go back to Home.'
		});
		expect(await (await browser.$('[aria-label="Início"]')).isExisting()).toBe(true);
	});
});
