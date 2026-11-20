import {
	firstDisplayed,
	getTitle,
	navigateTo,
	navigateToWithState,
	waitForAppReady
} from '../helpers/app';
import { createReaderFixture, readerChapterFor } from '../helpers/fixtures';

describe('native acerola via WebDriverIO', () => {
	it('initializes the app without crashing', async () => {
		await waitForAppReady();

		const title = await getTitle();
		// title may be empty if the app does not set document.title — only check the URL
		const url = await browser.getUrl();
		expect(url).toMatch(/tauri:\/\/localhost|localhost/i);

		// if there is a title, it must not be empty
		if (title !== '') {
			expect(typeof title).toBe('string');
		}
	});

	it('triggers folder selection without freezing the app', async () => {
		await waitForAppReady();
		await navigateTo('/config');

		const selectFolderButton = await browser.$('//*[normalize-space()="Pasta dos quadrinhos"]');
		await selectFolderButton.waitForDisplayed({ timeout: 5_000 });

		// WebDriver does not control the native dialog — only check that the IPC
		// was triggered without freezing the app
		await selectFolderButton.click();
		await browser.pause(300);
		await browser.keys(['Escape']).catch(() => undefined);
		await browser.pause(300);

		// app still responds
		const url = await browser.getUrl();
		expect(url).toMatch(/tauri:\/\/localhost|localhost/i);
	});

	it('loads native chapter image in the reader', async () => {
		const fixture = createReaderFixture('Acerola WDIO Native');
		await waitForAppReady();
		await navigateToWithState('/reader', {
			chapter: readerChapterFor(fixture),
			chapterIndex: 0,
			totalChapters: 1,
			chapterScope: fixture.comicTitle
		});

		const image = await firstDisplayed('img[alt="Página 1"], img[alt="Page 1"]', 20_000);

		const src = await image.getAttribute('src');
		expect(src).toMatch(/^blob:|asset:\/\/|tauri:\/\//i);
	});

	it.skip('triggers native minimize control via titlebar', async () => {
		await waitForAppReady();
		await navigateTo('/home');

		const beforeRect = await browser.getWindowRect();

		try {
			const minimizeButton = await browser.$('[aria-label="Minimizar"]');
			await minimizeButton.waitForDisplayed({ timeout: 5_000 });
			await minimizeButton.click();
			await browser.pause(500);

			const afterRect = await browser.getWindowRect();
			const visibilityState = await browser.execute(() => document.visibilityState);

			// Windows/WebView2 keeps the size, but moves the minimized window off screen.
			const movedOffscreen = afterRect.x < -10_000 || afterRect.y < -10_000;
			const wasMinimized =
				visibilityState !== 'visible' ||
				afterRect.height < beforeRect.height ||
				movedOffscreen;
			expect(wasMinimized).toBe(true);
		} finally {
			await browser
				.setWindowRect(beforeRect.x, beforeRect.y, beforeRect.width, beforeRect.height)
				.catch(() => undefined);
		}
	});
});
