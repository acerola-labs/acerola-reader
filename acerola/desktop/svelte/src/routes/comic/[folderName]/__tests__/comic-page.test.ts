import { render, screen, waitFor } from '@testing-library/svelte';
import { tick } from 'svelte';
import { userEvent } from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { CONTEXT_KEYS } from '#lib/constants/context-keys.ts';
import { ActiveComicState } from '#lib/state/comic-context.svelte.ts';
import { LIBRARY_EVENTS } from '#lib/contracts/library/chapter.events.ts';
import { LIBRARY_COMMANDS } from '#lib/contracts/library/chapter.commands.ts';
import { METADATA_COMMANDS } from '#lib/contracts/metadata/metadata.commands.ts';
import type { ChapterPayload } from '#lib/contracts/library/chapter.payloads.ts';
import type { ComicSummaryItemPayload } from '#lib/contracts/home/home.payloads.ts';
import { _resetBookmarksState } from '#lib/hooks/store/use-bookmarks.svelte.ts';
import { _resetChapterSelectionState } from '#lib/hooks/store/use-chapter-selection.svelte.ts';
import ComicPage from '../+page.svelte';

const { mockGoto, mockInvalidateAll } = vi.hoisted(() => ({
	mockGoto: vi.fn(),
	mockInvalidateAll: vi.fn()
}));

vi.mock('$app/navigation', () => ({
	goto: mockGoto,
	invalidateAll: mockInvalidateAll
}));

const { mockInvoke, mockListen } = vi.hoisted(() => ({
	mockInvoke: vi.fn(),
	mockListen: vi.fn()
}));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: (cmd: string, args: unknown) => mockInvoke(cmd, args),
	convertFileSrc: (path: string) => `asset://${path}`
}));

// The screen fires listen() for 4 sources on mount (comic:chapters, comic:chapters:error via
// useComicChapters, plus a dozen more via peers.startListening()/p2pSync.startListening()) —
// we capture the callbacks by event name to simulate the data event arriving after invoke()
// (the backend answers via an event, the invoke return itself is discarded).
const listenCallbacks = new Map<string, (event: { payload: unknown }) => void>();
let intersectionCallback: IntersectionObserverCallback | undefined;

vi.mock('@tauri-apps/api/event', () => ({
	listen: (event: string, callback: (event: { payload: unknown }) => void) => {
		listenCallbacks.set(event, callback);
		return Promise.resolve(vi.fn());
	}
}));

const { mockStoreLoad } = vi.hoisted(() => ({ mockStoreLoad: vi.fn() }));

vi.mock('@tauri-apps/plugin-store', () => ({
	load: mockStoreLoad
}));

vi.mock('@tauri-apps/plugin-log', () => ({
	error: vi.fn(),
	debug: vi.fn()
}));

vi.mock('svelte-sonner', () => ({
	toast: {
		info: vi.fn(),
		loading: vi.fn(() => 'mock-toast-id'),
		success: vi.fn(),
		error: vi.fn()
	}
}));

function setupInvokeMock(overrides: Record<string, unknown> = {}, rejects: string[] = []) {
	const defaults: Record<string, unknown> = {
		get_categories: [],
		get_all_comic_categories: [],
		get_comic_category: null,
		history_get_comic: null,
		history_get_read_chapters: [],
		get_paired_peers: [],
		get_network_status: undefined,
		get_sync_history_log: []
	};
	mockInvoke.mockImplementation((cmd: string) => {
		if (rejects.includes(cmd)) return Promise.reject(new Error('offline'));
		return Promise.resolve(cmd in overrides ? overrides[cmd] : (defaults[cmd] ?? undefined));
	});
}

function comic(overrides: Partial<ComicSummaryItemPayload> = {}): ComicSummaryItemPayload {
	return {
		relations: { directoryId: 'dir-1', metadataId: null },
		filesystem: { folderName: 'Acerola' },
		metadata: {
			title: 'Acerola Vol. 1',
			externalSync: false,
			activeSource: 'LOCAL',
			chapterCount: 2,
			description: 'Uma sinopse',
			status: 'ongoing',
			author: 'Autor X',
			rating: null
		},
		artwork: { cover: null, banner: null },
		...overrides
	};
}

function chapterPayload(overrides: Partial<ChapterPayload> = {}): ChapterPayload {
	return {
		hasVolumeStructure: false,
		showVolumeHeaders: false,
		effectiveViewMode: 'CHAPTER' as never,
		archive: {
			items: [
				{
					id: 'ch-1',
					name: 'Capítulo 1',
					path: '/path/ch1.cbz',
					chapterSort: '1',
					volumeId: null,
					volumeName: null,
					isSpecial: false,
					lastModified: 0
				}
			],
			volumes: [],
			page: 0,
			pageSize: 1_000_000,
			total: 1,
			volumeSections: []
		},
		...overrides
	};
}

function renderComicPage(
	data: { comic: ComicSummaryItemPayload | null },
	networkSyncOverrides: Record<string, unknown> = {}
) {
	const activeComic = new ActiveComicState();

	// `p2pSync` now comes from the context shared with `+layout.svelte` (see
	// `CONTEXT_KEYS.networkSync`), not from a page-owned instance — without providing that
	// context here, `getContext(...)` returns `undefined` and the first access to `p2pSync.log`
	// (inside the page's `$effect`) blows up. Explicit `Map<symbol, unknown>`: without it, TS
	// infers the type from the FIRST tuple (`ActiveComicState`) and rejects the second entry —
	// both sides of `getContext<T>(...)` already cast on read.
	return render(ComicPage, {
		props: { data: { ...data, initialVolumeViewMode: undefined } },
		context: new Map<symbol, unknown>([
			[CONTEXT_KEYS.activeComic, activeComic],
			[
				CONTEXT_KEYS.networkSync,
				{
					log: [],
					isSyncing: () => false,
					lastSyncedAt: () => undefined,
					activeSession: () => undefined,
					activeProgressMessage: () => undefined,
					syncHistory: vi.fn(),
					syncFiles: vi.fn(),
					syncComic: vi.fn(),
					syncAll: vi.fn(),
					startListening: vi.fn(),
					stopListening: vi.fn(),
					...networkSyncOverrides
				}
			]
		])
	});
}

// The title shows up twice (sticky mobile header + ComicMetadataPanel), so findByText fails
// on ambiguity — findAllByText waits for the render to finish just the same.
async function waitForTitle(title: string) {
	await waitFor(() => expect(screen.getAllByText(title).length).toBeGreaterThan(0));
}

// The chapter title is reused as the card description (comic.name becomes both title AND
// fileName on the page) — getAllByText instead of getByText avoids the ambiguity.
async function findChapterCard(text: string) {
	return (await screen.findAllByText(text))[0];
}

async function emitChapters(payload: ChapterPayload) {
	await waitFor(() => expect(listenCallbacks.has(LIBRARY_EVENTS.comicChapters)).toBe(true));
	listenCallbacks.get(LIBRARY_EVENTS.comicChapters)?.({ payload });
	await tick();

	const firstPage = document.querySelector<HTMLElement>('[data-page="0"]');
	if (firstPage) {
		intersectionCallback?.(
			[{ target: firstPage, isIntersecting: true } as unknown as IntersectionObserverEntry],
			{} as IntersectionObserver
		);
		await tick();
	}
}

describe('comic/[folderName] +page', () => {
	beforeEach(() => {
		vi.resetAllMocks();
		setupInvokeMock();
		listenCallbacks.clear();
		_resetBookmarksState();
		_resetChapterSelectionState();

		mockStoreLoad.mockResolvedValue({
			get: vi.fn().mockResolvedValue(undefined),
			set: vi.fn().mockResolvedValue(undefined),
			delete: vi.fn().mockResolvedValue(undefined),
			save: vi.fn().mockResolvedValue(undefined)
		});

		// acerola-comic-chapter-list.svelte only mounts the real content (AcerolaHeroButton) when
		// the IntersectionObserver reports the page as visible — we capture the callback to
		// simulate that by hand (see revealFirstPage), like acerola-comic-chapter-list.test.ts.
		intersectionCallback = undefined;
		globalThis.IntersectionObserver = class {
			constructor(callback: IntersectionObserverCallback) {
				intersectionCallback = callback;
			}
			observe = vi.fn();
			unobserve = vi.fn();
			disconnect = vi.fn();
		} as unknown as typeof IntersectionObserver;

		// jsdom in this version exposes Element.prototype.animate but returns undefined instead
		// of an Animation — the global setup.ts `if (!Element.prototype.animate)` guard doesn't
		// catch that case, and bits-ui's ToggleGroupItem breaks trying to set `.onfinish` on it.
		Element.prototype.animate = vi.fn().mockImplementation(() => ({
			finished: Promise.resolve(),
			cancel: vi.fn(),
			finish: vi.fn(),
			pause: vi.fn(),
			play: vi.fn(),
			reverse: vi.fn(),
			onfinish: null,
			oncancel: null,
			addEventListener: vi.fn(),
			removeEventListener: vi.fn(),
			dispatchEvent: vi.fn()
		})) as unknown as typeof Element.prototype.animate;
	});

	it('shows the loading spinner while there is no comic data', () => {
		renderComicPage({ comic: null });

		expect(document.querySelector('.animate-spin')).toBeInTheDocument();
	});

	it('renders the comic header and chapter list once data arrives', async () => {
		renderComicPage({ comic: comic() });

		await waitForTitle('Acerola Vol. 1');
		expect(mockInvoke).toHaveBeenCalledWith(
			LIBRARY_COMMANDS.getComicChapters,
			expect.objectContaining({ comicDirectoryFk: 'dir-1' })
		);

		await emitChapters(chapterPayload());

		expect(await findChapterCard('Capítulo 1')).toBeInTheDocument();
	});

	it('opens the reader when a chapter is clicked', async () => {
		const user = userEvent.setup();
		renderComicPage({ comic: comic() });

		await waitForTitle('Acerola Vol. 1');
		await emitChapters(chapterPayload());

		await user.click(await findChapterCard('Capítulo 1'));

		expect(mockGoto).toHaveBeenCalledWith(
			'/reader',
			expect.objectContaining({
				state: expect.objectContaining({
					comicDirectoryId: 'dir-1',
					chapter: expect.objectContaining({ id: 'ch-1' })
				})
			})
		);
	});

	it('switches to the preferences tab', async () => {
		const user = userEvent.setup();
		renderComicPage({ comic: comic() });

		await waitForTitle('Acerola Vol. 1');
		await emitChapters(chapterPayload());

		await user.click(screen.getByRole('radio', { name: /preferences|preferências/i }));

		// ComicPreferences opens on the category list (Reading/Sync/Advanced) by default —
		// checks that the preferences panel really mounted.
		expect(await screen.findByText(/^leitura$/i)).toBeInTheDocument();
	});

	it('shows an error toast when syncing metadata from MangaDex fails', async () => {
		const { toast } = await import('svelte-sonner');
		setupInvokeMock({}, [METADATA_COMMANDS.syncMangadex]);

		const user = userEvent.setup();
		renderComicPage({ comic: comic({ metadata: { ...comic().metadata, externalSync: true } }) });

		await waitForTitle('Acerola Vol. 1');
		await emitChapters(chapterPayload());

		await user.click(screen.getByRole('radio', { name: /preferences|preferências/i }));
		await user.click(await screen.findByText(/^sincronização$/i));
		// card title ("MangaDex Sync" / "Sincronização com MangaDex") — the card description
		// also contains "mangadex", so the full phrase is needed to avoid ambiguity.
		await user.click(await screen.findByText(/mangadex sync|sincronização com mangadex/i));

		await waitFor(() => expect(toast.error).toHaveBeenCalled());
	});

	it('does not call invalidateAll on mount when the latest sync entry is persisted (id < 0)', async () => {
		mockInvalidateAll.mockClear();
		renderComicPage(
			{ comic: comic() },
			{
				log: [
					{
						id: -1,
						peerId: 'peer-1',
						kind: 'files',
						status: 'complete',
						message: 'Completo',
						timestamp: Date.now()
					}
				]
			}
		);

		await waitForTitle('Acerola Vol. 1');
		expect(mockInvalidateAll).not.toHaveBeenCalled();
	});

	it('calls invalidateAll once when a live sync completes for this comic', async () => {
		mockInvalidateAll.mockClear();
		renderComicPage(
			{ comic: comic() },
			{
				log: [
					{
						id: 1,
						peerId: 'peer-1',
						kind: 'comic',
						comicName: 'Acerola Vol. 1',
						status: 'complete',
						message: 'Completo',
						timestamp: Date.now()
					}
				]
			}
		);

		await waitForTitle('Acerola Vol. 1');
		expect(mockInvalidateAll).toHaveBeenCalledTimes(1);
	});
});
