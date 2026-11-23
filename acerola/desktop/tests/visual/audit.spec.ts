import { test, type Page } from '@playwright/test';
import { HOME_COMMANDS } from '../../svelte/src/lib/contracts/home/home.commands';
import { HOME_EVENTS } from '../../svelte/src/lib/contracts/home/home.events';
import { LIBRARY_COMMANDS } from '../../svelte/src/lib/contracts/library/chapter.commands';
import { LIBRARY_EVENTS } from '../../svelte/src/lib/contracts/library/chapter.events';
import { NETWORK_COMMANDS } from '../../svelte/src/lib/contracts/network/network.commands';
import { NETWORK_EVENTS } from '../../svelte/src/lib/contracts/network/network.events';
import { READER_COMMANDS } from '../../svelte/src/lib/contracts/reader/reader.commands';
import { BOOKMARKS_COMMANDS } from '../../svelte/src/lib/contracts/bookmarks/bookmarks.commands';
import { STORE_FILE, STORE_KEYS } from '../../svelte/src/lib/constants/store-plugin';
import {
	e2eComic,
	e2eComicChapters,
	e2eComicSummary,
	e2eLocalAddr,
	e2eLocalDeviceInfo,
	e2eLocalPeerId,
	e2ePairedPeer,
	e2eReaderPage,
	e2eReaderSession,
	e2eRelayInfo
} from '../../svelte/tests/mocks/acerola-e2e-data';
import { installTauriMocks, mockedTauriResponse } from '../../svelte/tests/mocks/tauri-playwright';

// Visual audit: walks every screen and state of the desktop app with mocked Tauri IPC and
// saves one screenshot per state in `OUT` (dark and light), so a person (or Claude) can look
// at each one against the brand book. Not part of CI: run with
// `npx playwright test -c tests/visual/playwright.config.ts`.
const OUT = process.env.AUDIT_OUT ?? 'test-results/visual-audit';

const TITLES = [
	'Ajin Demi-Human',
	'BPRD',
	'Berserk',
	'Chainsaw Man',
	'Claymore',
	'Dungeon Meshi',
	'Goblin Slayer',
	'Mushishi',
	'Frieren',
	'Vagabond'
];
const comics = TITLES.map((title, i) => ({
	...e2eComic,
	relations: { ...e2eComic.relations, directoryId: `dir-${i}` },
	filesystem: { folderName: title },
	metadata: { ...e2eComic.metadata, title, chapterCount: (i + 1) * 12 }
}));

const unmocked = new Set<string>();

async function setup(page: Page, theme: 'dark' | 'light', onboarded = true) {
	await installTauriMocks(
		page,
		{
			[HOME_COMMANDS.getComicSummarySorted]: mockedTauriResponse({
				events: [{ event: HOME_EVENTS.homeData, payload: e2eComicSummary(comics) }]
			}),
			[LIBRARY_COMMANDS.getComicByFolderName]: comics[2],
			[LIBRARY_COMMANDS.getComicChapters]: mockedTauriResponse({
				events: [{ event: LIBRARY_EVENTS.comicChapters, payload: e2eComicChapters() }]
			}),
			[READER_COMMANDS.openChapter]: e2eReaderSession(),
			[READER_COMMANDS.loadPage]: (args) => e2eReaderPage((args as { index: number }).index),
			[BOOKMARKS_COMMANDS.getCategories]: [
				{ id: 1, name: 'Lendo', color: 0xff4caf50 },
				{ id: 2, name: 'Favoritos', color: 0xffe91e63 }
			],
			[NETWORK_COMMANDS.getSecurityStatus]: false,
			[NETWORK_COMMANDS.getLocalId]: e2eLocalPeerId,
			[NETWORK_COMMANDS.getLocalAddr]: e2eLocalAddr,
			[NETWORK_COMMANDS.getLocalDeviceInfo]: e2eLocalDeviceInfo,
			[NETWORK_COMMANDS.getRelayInfo]: e2eRelayInfo,
			[NETWORK_COMMANDS.getPairedPeers]: [e2ePairedPeer],
			[NETWORK_COMMANDS.getNetworkStatus]: mockedTauriResponse({
				events: [{ event: NETWORK_EVENTS.status, payload: { mode: 'local', peers: [] } }]
			})
		},
		{
			// The theme lives in the Tauri store (`use-theme.svelte.ts`), not in localStorage.
			storeSeed: {
				[STORE_FILE]: {
					[STORE_KEYS.mode]: theme,
					...(onboarded ? {} : { [STORE_KEYS.onboardingCompleted]: false })
				}
			},
			fallback: (command) => {
				unmocked.add(command);
				// Commands that list things answer an empty list; the rest answer nothing.
				return /get_all|list|templates|chapters|categories|peers|log/.test(command)
					? []
					: null;
			}
		}
	);
}

/** Runs a step and keeps walking if it fails (a missing button is itself an audit finding). */
async function step(name: string, action: () => Promise<unknown>) {
	try {
		await action();
	} catch (error) {
		console.log(`STEP FAILED ${name}: ${(error as Error).message.split('\n')[0]}`);
	}
}

async function shot(page: Page, name: string, full = false) {
	await page.waitForTimeout(700);
	await page.screenshot({ path: `${OUT}/${name}.png`, fullPage: full });
}

for (const theme of ['dark', 'light'] as const) {
	test.describe(`audit ${theme}`, () => {
		test.beforeEach(async ({ page }) => {
			await setup(page, theme);
		});

		test('home and its overlays', async ({ page }) => {
			await page.goto('/home');
			await page.getByRole('button', { name: 'Berserk' }).first().waitFor();
			await shot(page, `${theme}-01-home`);

			const t = { timeout: 4000 };
			await step('filter', async () => {
				await page.getByRole('button', { name: /Filtrar/ }).click(t);
				await shot(page, `${theme}-02-home-filter`);
				await page.keyboard.press('Escape');
			});
			await step('peer picker', async () => {
				await page.getByRole('button', { name: /outro dispositivo/ }).click(t);
				await shot(page, `${theme}-03-home-peer-picker`);
				await page.keyboard.press('Escape');
			});
			await step('selection', async () => {
				await page
					.locator('main')
					.getByRole('button')
					.filter({
						has: page.locator('svg.lucide-more-vertical, svg.lucide-ellipsis-vertical')
					})
					.first()
					.click(t);
				await shot(page, `${theme}-04-home-selection`);
				await page.getByRole('button', { name: /Ações/ }).click(t);
				await shot(page, `${theme}-05-home-actions`);
				await page.keyboard.press('Escape');
				await page.getByRole('button', { name: 'Cancelar' }).click(t);
			});
			await step('search', async () => {
				await page.keyboard.press('Control+K');
				await shot(page, `${theme}-06-search`);
				await page.keyboard.press('Escape');
			});
			await step('notifications', async () => {
				await page.getByRole('button', { name: 'Notificações' }).click(t);
				await shot(page, `${theme}-07-notifications`);
				await page.keyboard.press('Escape');
			});
			await step('language', async () => {
				await page
					.getByRole('button', { name: /idioma/i })
					.first()
					.click(t);
				await shot(page, `${theme}-07b-language`);
				await page.keyboard.press('Escape');
			});
		});

		test('comic page', async ({ page }) => {
			await page.goto('/comic/Berserk');
			await page.waitForTimeout(1500);
			await shot(page, `${theme}-08-comic`);
			await shot(page, `${theme}-09-comic-chapters`, true);
			await step('chapter menu', async () => {
				await page
					.locator('main, body')
					.getByRole('button')
					.filter({
						has: page.locator('svg.lucide-more-vertical, svg.lucide-ellipsis-vertical')
					})
					.first()
					.click({ timeout: 4000 });
				await shot(page, `${theme}-09b-chapter-menu`);
				await page.keyboard.press('Escape');
			});
			await step('preferences', async () => {
				await page
					.getByText('Preferências', { exact: true })
					.first()
					.click({ timeout: 4000 });
				await shot(page, `${theme}-10-comic-preferences`, true);
			});
		});

		test('reader', async ({ page }) => {
			await page.goto('/reader?folder=Berserk&chapter=chapter-1');
			await shot(page, `${theme}-11-reader`);
		});

		test('history', async ({ page }) => {
			await page.goto('/history');
			await shot(page, `${theme}-12-history`, true);
		});

		test('network', async ({ page }) => {
			await page.goto('/network');
			await page.getByText('Meu PC').waitFor();
			await shot(page, `${theme}-13-network`, true);
			await step('network expanded', async () => {
				await page.getByRole('button', { name: 'Gerenciar' }).click({ timeout: 4000 });
				await page
					.getByRole('button', { name: 'Ticket', exact: true })
					.click({ timeout: 4000 });
				await page.getByRole('button', { name: 'Ver código' }).click({ timeout: 4000 });
				await shot(page, `${theme}-14-network-expanded`, true);
			});
			await step('peer menu', async () => {
				await page
					.locator('button, [role=button], span')
					.filter({
						has: page.locator('svg.lucide-more-vertical, svg.lucide-ellipsis-vertical')
					})
					.last()
					.click({ timeout: 4000 });
				await shot(page, `${theme}-14b-peer-menu`);
			});
		});

		test('config', async ({ page }) => {
			await page.goto('/config');
			await shot(page, `${theme}-15-config`, true);
			await page.goto('/config/templates');
			await shot(page, `${theme}-16-templates`, true);
		});

		test('error page', async ({ page }) => {
			await page.goto('/this-route-does-not-exist');
			await shot(page, `${theme}-17-error`);
		});
	});
}

test('onboarding', async ({ page }) => {
	await setup(page, 'dark', false);
	await page.goto('/home');
	await shot(page, 'dark-18-onboarding-1');
	for (let i = 2; i <= 5; i++) {
		await step(`onboarding ${i}`, async () => {
			await page
				.getByRole('button', { name: /Começar|Próximo|Continuar|Avançar|Seguir/ })
				.last()
				.click({ timeout: 4000 });
			await shot(page, `dark-18-onboarding-${i}`);
		});
	}
});

test.afterAll(() => {
	console.log(`UNMOCKED: ${[...unmocked].sort().join(', ')}`);
});
