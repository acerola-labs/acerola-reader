import path from 'node:path';
import fs from 'node:fs';
import {
	firstDisplayed,
	getPathname,
	invokeTauriCommand,
	navigateTo,
	navigateToWithState,
	waitForAppReady
} from '../helpers/app';

const cbzFolder = path.resolve(process.cwd(), 'tests/wdio/comic/cbz');
const cbzFile = path.join(cbzFolder, 'witchcraft.cbz');
const cbzChapter2 = path.join(cbzFolder, 'witchcraft2.cbz');
const cbzChapter3 = path.join(cbzFolder, 'witchcraft3.cbz');

// Ensure cleanup even if the test fails
after(() => {
	if (fs.existsSync(cbzChapter2)) fs.rmSync(cbzChapter2);
	if (fs.existsSync(cbzChapter3)) fs.rmSync(cbzChapter3);
});

describe('Reader Navigation E2E', () => {
	it('tests navigation across chapters and history resuming', async function () {
		this.timeout(90_000);

		if (!fs.existsSync(cbzChapter2)) {
			fs.copyFileSync(cbzFile, cbzChapter2);
		}
		if (!fs.existsSync(cbzChapter3)) {
			fs.copyFileSync(cbzFile, cbzChapter3);
		}

		await waitForAppReady();
		await navigateTo('/home');
		// Wait for onMount to register the scan:complete listener
		await browser.pause(500);

		// 1. Scan the library
		await invokeTauriCommand('refresh_library', { path: cbzFolder });

		// Wait for the 'cbz' card to appear — confirms the scan finished
		await firstDisplayed('//main//h3[normalize-space()="cbz"]', 15_000);

		// Get the comic's directoryId for the reader state
		const comic = await invokeTauriCommand<any>('get_comic_by_folder_name', {
			folderName: 'cbz'
		});

		// 2. Navigate straight to the reader with chapter 1
		await navigateToWithState('/reader', {
			chapter: {
				id: 'cbz-witchcraft-ch1',
				name: 'witchcraft',
				path: cbzFile,
				chapterSort: '0',
				volumeId: null,
				volumeName: null,
				isSpecial: false,
				lastModified: 0
			},
			comicDirectoryId: comic?.relations?.directoryId ?? null,
			chapterIndex: 0,
			totalChapters: 3,
			chapterScope: 'cbz'
		});

		// Confirm page 1 opened in chapter 1
		const image = await firstDisplayed('img[alt="Página 1"], img[alt="Page 1"]', 20_000);
		expect(await image.getAttribute('src')).toMatch(/^blob:|asset:\/\/|tauri:\/\//i);

		// 3. Navigate to the next chapter
		const nextBtn = await firstDisplayed('//*[normalize-space()="Próximo capítulo"]', 5_000);
		await nextBtn.click();

		// Wait for the previous-chapter button to appear (confirms we are on ch. 2)
		await firstDisplayed('//*[normalize-space()="Capítulo anterior"]', 10_000);

		// 4. Go back via the Back button
		const backBtn = await firstDisplayed('button[title="Voltar"]', 5_000);
		await backBtn.click();

		await browser.waitUntil(async () => !(await getPathname()).startsWith('/reader'), {
			timeout: 5_000,
			timeoutMsg: 'Did not leave the reader.'
		});

		// 5. Go to the history and check the entry exists
		await navigateTo('/history');
		const resumeBtn = await firstDisplayed(
			'//*[contains(normalize-space(), "Continuar")]',
			10_000
		);
		expect(await resumeBtn.isExisting()).toBe(true);

		// Simulate resuming by navigating straight to chapter 2
		await navigateToWithState('/reader', {
			chapter: {
				id: 'cbz-witchcraft-ch2',
				name: 'witchcraft2',
				path: cbzChapter2,
				chapterSort: '1',
				volumeId: null,
				volumeName: null,
				isSpecial: false,
				lastModified: 0
			},
			comicDirectoryId: comic?.relations?.directoryId ?? null,
			chapterIndex: 1,
			totalChapters: 3,
			chapterScope: 'cbz'
		});

		// 6. Check we are on chapter 2 (both buttons present)
		await firstDisplayed('//*[normalize-space()="Próximo capítulo"]', 5_000);
		await firstDisplayed('//*[normalize-space()="Capítulo anterior"]', 5_000);

		// 7. Go back to chapter 1 via state — check the previous button does not show on chapter 1
		await navigateToWithState('/reader', {
			chapter: {
				id: 'cbz-witchcraft-ch1',
				name: 'witchcraft',
				path: cbzFile,
				chapterSort: '0',
				volumeId: null,
				volumeName: null,
				isSpecial: false,
				lastModified: 0
			},
			comicDirectoryId: comic?.relations?.directoryId ?? null,
			chapterIndex: 0,
			totalChapters: 3,
			chapterScope: 'cbz'
		});

		// On chapter 1 (index 0), hasPreviousChapter = false — the button must be removed from the DOM
		await browser.waitUntil(
			async () => {
				const btn = await browser.$('//*[normalize-space()="Capítulo anterior"]');
				return !(await btn.isExisting());
			},
			{
				timeout: 5_000,
				interval: 100,
				timeoutMsg: '"Capítulo anterior" wrongly shows on chapter 1.'
			}
		);
	});
});
