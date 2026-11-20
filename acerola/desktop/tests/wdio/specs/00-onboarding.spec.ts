import {
	clickText,
	exactTextSelector,
	firstDisplayed,
	getPathname,
	getStoreValue,
	navigateTo,
	setStoreValue,
	setStoreValues,
	waitForTauriReady,
	waitForText
} from '../helpers/app';

describe('onboarding and initial tutorial', () => {
	it('renders first open tutorial when opening app without saved data', async () => {
		await waitForTauriReady();
		await navigateTo('/home');
		await setStoreValue('onboarding_completed', false);
		await setStoreValue('library_path', null);
		await browser.execute(() => window.location.reload());
		await waitForTauriReady();

		// Step 0: Welcome
		await waitForText('Acerola');
		await waitForText('Bem-vindo ao Acerola, seu leitor de quadrinhos.');
		await waitForText('Começar');
	});

	it('blocks advance in settings step if no folder is configured', async () => {
		await waitForTauriReady();
		await navigateTo('/home');
		await setStoreValue('onboarding_completed', false);
		await setStoreValue('library_path', null);
		await browser.execute(() => window.location.reload());
		await waitForTauriReady();

		// Advance to Step 3 (Settings)
		await clickText('Começar');
		await waitForText('Selecionar idioma');

		await clickText('Próximo');
		await waitForText('Formatos suportados');

		await clickText('Próximo');
		await waitForText('Configuração inicial');

		// Without a library folder in settings.json, the "Próximo" (Next) button must be disabled
		const nextBtn = await firstDisplayed(
			`${exactTextSelector('Próximo')}/ancestor-or-self::button[1]`
		);
		expect(await nextBtn.isEnabled()).toBe(false);
	});

	it('allows navigating back using the Back button', async () => {
		await waitForTauriReady();
		await navigateTo('/home');
		await setStoreValue('onboarding_completed', false);
		await browser.execute(() => window.location.reload());
		await waitForTauriReady();

		// Advance to Step 2 (Formats)
		await clickText('Começar');
		await waitForText('Selecionar idioma');

		await clickText('Próximo');
		await waitForText('Formatos suportados');

		// Go back to Step 1 (Language)
		await clickText('Voltar');
		await waitForText('Selecionar idioma');

		// Go back to Step 0 (Welcome)
		await clickText('Voltar');
		await waitForText('Bem-vindo ao Acerola, seu leitor de quadrinhos.');
	});
});
